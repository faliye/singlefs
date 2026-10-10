// 正推腿：第 10z 批第三块第 E 条（第 ① 段按轮跑）的模拟。逐句照 crates/singlefs-checker-tier/src/crash_amplification.rs
// judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from（2927-3148 行）与
// judge_on_one_card_and_start_the_next（2221-2391 行）的走法，用 harness 档的纯函数（judge_blocks_on_one_card、BlocksForTheCards、
// BlocksStoredInOrder、after_a_round_of_the_cards）和假卡搭一遍；假卡：每张卡有「建判器时报 OOM」「判了 n 块之后下一块报 OOM」「每块耗时」。
use singlefs_harness::gpu_memory_ledger::{
    after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
    BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
    CardLeftTheBlocks, StopOfTheRun,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct FakeCard {
    index: u32,
    holds_states: u64,            // 一次派活带几个状态（0：这一趟装不下）
    oom_at_build: bool,           // 建判器时报 OOM
    oom_after_blocks: Option<usize>, // 判了这么多块（累计，跨轮）之后下一块报 OOM
    micros_per_block: u64,
}

enum Message {
    Finished(u32),
    OutOfMemory(u32),
    Block(usize),
}

struct Work<'a> {
    cards: &'a [FakeCard],       // 这一轮装了状态的卡（cards_for_the_judges）
    states_placed: &'a [u64],
    blocks: BlocksForTheCards,
    stop: AtomicBool,
    judged_ok: &'a Mutex<Vec<usize>>,           // 每块成功判了几遍
    blocks_done_by_card: &'a Mutex<std::collections::BTreeMap<u32, usize>>,
}

fn start_the_next<'scope, 'w>(
    scope: &'scope std::thread::Scope<'scope, 'w>,
    work: &'w Work<'w>,
    position: usize,
    sender: &mpsc::Sender<Message>,
) {
    let next = (position + 1..work.cards.len()).find(|c| work.states_placed[*c] > 0);
    if let Some(next) = next {
        if work.blocks.any_left() && !work.stop.load(Ordering::SeqCst) {
            let sender = sender.clone();
            scope.spawn(move || judge_on_one_card(scope, work, next, sender));
        }
    }
}

fn judge_on_one_card<'scope, 'w>(
    scope: &'scope std::thread::Scope<'scope, 'w>,
    work: &'w Work<'w>,
    position: usize,
    sender: mpsc::Sender<Message>,
) {
    let card = &work.cards[position];
    if card.oom_at_build {
        let _ = sender.send(Message::OutOfMemory(card.index));
        start_the_next(scope, work, position, &sender);
        return;
    }
    start_the_next(scope, work, position, &sender);
    let left = judge_blocks_on_one_card(
        &work.blocks,
        &|| work.stop.load(Ordering::SeqCst),
        &mut |pending_index| {
            std::thread::sleep(std::time::Duration::from_micros(card.micros_per_block));
            let done = {
                let mut m = work.blocks_done_by_card.lock().unwrap();
                *m.entry(card.index).or_insert(0)
            };
            if card.oom_after_blocks.is_some_and(|n| done >= n) {
                return BlockJudgedOnACard::OutOfMemory { reason: format!("card {} oom", card.index) };
            }
            *work.blocks_done_by_card.lock().unwrap().get_mut(&card.index).unwrap() += 1;
            work.judged_ok.lock().unwrap()[pending_index] += 1;
            BlockJudgedOnACard::Judged(pending_index)
        },
        &mut |pending_index, _| match sender.send(Message::Block(pending_index)) {
            Ok(()) => BlockHandedToTheStore::Handed,
            Err(_) => BlockHandedToTheStore::TheStoreIsGone,
        },
    );
    match left {
        CardLeftTheBlocks::OutOfMemory { .. } => {
            let _ = sender.send(Message::OutOfMemory(card.index));
        }
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {
            let _ = sender.send(Message::Finished(card.index));
        }
        CardLeftTheBlocks::Refused { .. } => unreachable!(),
    }
}

#[derive(Debug)]
struct Outcome {
    result: &'static str,
    stored: usize,
    stored_is_prefix_in_order: bool,
    judged_ok: Vec<usize>,
    rounds: usize,
    finished_entries: Vec<u32>,
    assertion_failures: Vec<String>,
}

/// 逐句照产品代码的外层循环。`stop_after_round_with_blocks_left`：叫停请求在第几轮收工后为真（None 是不叫停）。
fn run(
    pending: usize,
    states_per_block: u64,
    cards: &[FakeCard],
    stop_at_round: Option<usize>,
    stop_from_round: Option<usize>,
) -> Outcome {
    let judged_ok = Mutex::new(vec![0usize; pending]);
    let blocks_done_by_card = Mutex::new(std::collections::BTreeMap::new());
    let mut stored_in_order: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
    let stored_list = Mutex::new(Vec::<usize>::new());
    let mut blocks_of_the_round: Vec<usize> = (0..pending).collect();
    let mut short: BTreeSet<u32> = BTreeSet::new();
    let mut finished_entries = Vec::new();
    let mut round = 0usize;
    let stop_flag = AtomicUsize::new(0);
    let result;
    let mut assertion_failures: Vec<String> = Vec::new();
    loop {
        round += 1;
        assert!(round <= cards.len() + 1, "轮数超过卡数 + 1：无限再起的苗头 round={round}");
        let pending_states: u64 = blocks_of_the_round.len() as u64 * states_per_block;
        let run_cards: Vec<FakeCard> = cards.iter().filter(|c| !short.contains(&c.index)).cloned().collect();
        // 按优先级装：装到带得了的状态数为止
        let mut left = pending_states;
        let mut placed = Vec::new();
        for c in &run_cards {
            let p = left.min(c.holds_states);
            placed.push(p);
            left -= p;
        }
        let used: Vec<(FakeCard, u64)> = run_cards
            .iter()
            .cloned()
            .zip(placed.iter().copied())
            .filter(|(_, p)| *p > 0)
            .collect();
        if used.is_empty() {
            // 产品代码：没有一张装得下 -> 有 first_out_of_memory 则本次未跑
            result = if short.is_empty() { "ERR-no-card-holds-a-state" } else { "NotRun" };
            break;
        }
        let used_cards: Vec<FakeCard> = used.iter().map(|(c, _)| c.clone()).collect();
        let used_placed: Vec<u64> = used.iter().map(|(_, p)| *p).collect();
        let work = Work {
            cards: &used_cards,
            states_placed: &used_placed,
            blocks: BlocksForTheCards::new(blocks_of_the_round.iter().copied()),
            stop: AtomicBool::new(stop_from_round.is_some_and(|r| round >= r)),
            judged_ok: &judged_ok,
            blocks_done_by_card: &blocks_done_by_card,
        };
        let mut oom_this_round: Vec<u32> = Vec::new();
        std::thread::scope(|scope| {
            let (sender, receiver) = mpsc::channel::<Message>();
            let w = &work;
            scope.spawn(move || judge_on_one_card(scope, w, 0, sender));
            for message in receiver {
                match message {
                    Message::Block(i) => stored_in_order.arrive(i, i, &mut |b| {
                        stored_list.lock().unwrap().push(b);
                        Ok(())
                    }),
                    Message::Finished(i) => finished_entries.push(i),
                    Message::OutOfMemory(i) => oom_this_round.push(i),
                }
            }
        });
        // 产品代码 3076-3091 行的两条断言：没有卡报 OOM、没拒判、写库没错时，等着的块要是 0；没人叫停时队里不能有剩的
        if oom_this_round.is_empty() {
            let waiting = stored_in_order.blocks_waiting();
            if waiting != 0 {
                assertion_failures.push(format!("round {round}: blocks_waiting()={waiting} != 0（产品代码 3081 行断言会 panic）"));
            }
        }
        for i in &oom_this_round {
            assert!(short.insert(*i), "报过 OOM 的卡 {i} 又被用了一轮");
        }
        blocks_of_the_round = work.blocks.blocks_not_taken();
        let not_short = run_cards.iter().filter(|c| !short.contains(&c.index)).count();
        let stop = if stop_at_round == Some(round) { StopOfTheRun::Requested } else { StopOfTheRun::NotRequested };
        let _ = &stop_flag;
        match after_a_round_of_the_cards(pending - stored_in_order.blocks_stored(), oom_this_round.len(), stop, not_short) {
            AfterARoundOfTheCards::EveryBlockStored => { result = "EveryBlockStored"; break; }
            AfterARoundOfTheCards::StoppedOnRequest => { result = "StoppedOnRequest"; break; }
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {}
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => { result = "NotRun"; break; }
        }
    }
    let stored = stored_list.into_inner().unwrap();
    let in_order = stored.iter().enumerate().all(|(k, b)| k == *b);
    Outcome {
        result,
        stored: stored.len(),
        stored_is_prefix_in_order: in_order,
        judged_ok: judged_ok.into_inner().unwrap(),
        rounds: round,
        finished_entries,
        assertion_failures,
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
}

#[test]
fn random_cards_and_oom_schedules() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let mut cells = 0u64;
    let mut by_result = std::collections::BTreeMap::<String, u64>::new();
    let mut violations: Vec<String> = Vec::new();
    let mut dup_cards_in_finished = 0u64;
    let mut notrun_cells = 0u64;
    let mut notrun_cells_with_judged_but_unstored = 0u64;
    let mut max_wasted = 0usize;
    for _ in 0..3000 {
        let n_cards = 1 + rng.below(4) as usize;
        let pending = 1 + rng.below(12) as usize;
        let states_per_block = 1 + rng.below(4);
        let mut cards = Vec::new();
        for k in 0..n_cards {
            let holds = if rng.below(6) == 0 { 0 } else { 1 + rng.below(8) };
            cards.push(FakeCard {
                index: k as u32,
                holds_states: holds,
                oom_at_build: rng.below(6) == 0,
                oom_after_blocks: if rng.below(3) == 0 { Some(rng.below(8) as usize) } else { None },
                micros_per_block: 50 + rng.below(400),
            });
        }
        let o = run(pending, states_per_block, &cards, None, None);
        cells += 1;
        *by_result.entry(o.result.to_string()).or_insert(0) += 1;
        let desc = format!("pending={pending} spb={states_per_block} cards={cards:?}");
        // 1) 没有一块成功判两遍
        if o.judged_ok.iter().any(|t| *t > 1) {
            violations.push(format!("判两遍 {:?} :: {desc}", o.judged_ok));
        }
        // 2) 写库的是 0..k 的前缀，按块序
        if !o.stored_is_prefix_in_order {
            violations.push(format!("写库不按块序 :: {desc}"));
        }
        // 3) EveryBlockStored => 全写进、每块判一遍
        if o.result == "EveryBlockStored" && (o.stored != pending || o.judged_ok.iter().any(|t| *t != 1)) {
            violations.push(format!("EveryBlockStored 却 stored={} judged={:?} :: {desc}", o.stored, o.judged_ok));
        }
        // 4) 判完的块都入库或在「本次未跑」之前：成功判了的块数 = 写库数 + 等着的；本次未跑时库里是前缀
        // 5) 放得下不该报本次未跑：存在一张从不报 OOM、且装得下状态的卡 => 必须 EveryBlockStored
        let has_a_safe_card = cards.iter().any(|c| !c.oom_at_build && c.oom_after_blocks.is_none() && c.holds_states > 0);
        if has_a_safe_card && o.result != "EveryBlockStored" {
            violations.push(format!("有一张不报 OOM 的卡却 {} stored={} :: {desc}", o.result, o.stored));
        }
        if o.result == "NotRun" {
            notrun_cells += 1;
            let judged_total: usize = o.judged_ok.iter().sum();
            if judged_total > o.stored {
                notrun_cells_with_judged_but_unstored += 1;
                max_wasted = max_wasted.max(judged_total - o.stored);
            }
        }
        // 6) 同一张卡在 finished 里出现两次（cards_used 数两遍）
        let mut seen = BTreeSet::new();
        for i in &o.finished_entries {
            if !seen.insert(*i) { dup_cards_in_finished += 1; break; }
        }
    }
    println!("ROUNDS_SIM cells={cells} results={by_result:?} dup_finished_cells={dup_cards_in_finished} violations={} notrun_cells={notrun_cells} notrun_cells_with_judged_but_unstored={notrun_cells_with_judged_but_unstored} max_wasted_blocks={max_wasted}", violations.len());
    for v in violations.iter().take(8) {
        println!("ROUNDS_SIM violation: {v}");
    }
}

#[test]
fn stop_requested_between_round_one_and_two_with_blocks_waiting_behind_a_given_back_block() {
    // 2 张卡、10 块：第 0 张慢、判了 1 块之后下一块报 OOM；第 1 张快、把别的块都判了先收工
    let cards = vec![
        FakeCard { index: 0, holds_states: 8, oom_at_build: false, oom_after_blocks: Some(0), micros_per_block: 30000 },
        FakeCard { index: 1, holds_states: 8, oom_at_build: false, oom_after_blocks: None, micros_per_block: 100 },
    ];
    for stop_from in [None, Some(2usize)] {
        let o = run(10, 1, &cards, None, stop_from);
        println!(
            "STOPSIM stop_from_round={stop_from:?} result={} stored={} rounds={} judged_ok={:?} assertion_failures={:?}",
            o.result, o.stored, o.rounds, o.judged_ok, o.assertion_failures
        );
    }
}
