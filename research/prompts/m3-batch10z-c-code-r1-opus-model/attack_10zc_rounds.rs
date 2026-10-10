//! 第 10z 批第三块代码轮第一轮攻方模型：K1-5「收尾再起一轮」。
//! checker 档那一圈（crash_amplification.rs `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 第 2919–3148 行的几轮、
//! `judge_on_one_card_and_start_the_next` 第 2221–2391 行的卡线程、`start_the_next_card_with_states` 第 2444–2460 行、
//! `receive_the_cards_messages_and_store_the_blocks_in_order` 第 2050–2093 行）照原样的次序抄成不碰卡的一份，
//! 用 harness 档的 `judge_blocks_on_one_card`、`BlocksForTheCards`、`BlocksStoredInOrder`、`after_a_round_of_the_cards` 原样（一行不改）。
//! checker 档那几行本身不在这里跑（checker 档测试不跑）：抄得对不对是推的，逐步标了行号。
//! 叫停照 `stop_is_requested_by_the_environment`（第 1382 行：叫停文件出现）在任意一刻出现；第 3076–3091 行那两条 assert 照原文判，
//! 判不过记成 `AssertPanicked`（原文是 panic，这一趟判红）。
use singlefs_harness::gpu_memory_ledger::{
    after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
    BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
    CardLeftTheBlocks, StopOfTheRun,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, Debug)]
struct FakeCard {
    index: u32,
    /// 第 r 轮这张卡此刻带得了几个状态（超出的轮取最后一个；0 就是此刻带不了）。
    capacity_in_round: Vec<u64>,
    /// 第 r 轮建判器时报 Out of Memory。
    out_of_memory_at_build_in_round: Vec<usize>,
    /// 这张卡判的第 n 块（从 1 数，几轮合计）报 Out of Memory。
    out_of_memory_at_its_block: Option<usize>,
    /// 判一块之前让出几次（快慢）。
    yields_each_block: u64,
    /// 报 Out of Memory 的那一块报之前再让出几次（报得晚，别的卡先把后面的块判完）。
    yields_before_out_of_memory: u64,
}

struct Script {
    cards: Vec<FakeCard>,
    /// 第 n 个事件（建判器、判一块各算一个，从 1 数）发生时叫停文件出现。
    stop_at_event: Option<usize>,
    events: AtomicUsize,
    stop_file: AtomicBool,
    /// 定形那一格：第 0 张卡判它的第 1 块时等别的卡都收工再报（让交回的那一块落到收尾）。
    card_0_waits_for_the_others: bool,
    others_left: AtomicUsize,
    judged_times: Mutex<Vec<usize>>,
    blocks_of_each_card: Vec<AtomicUsize>,
}

impl Script {
    fn event(&self) {
        let n = self.events.fetch_add(1, Ordering::SeqCst) + 1;
        if self.stop_at_event == Some(n) {
            self.stop_file.store(true, Ordering::SeqCst);
        }
    }
}

enum Message {
    Block(usize),
    Finished,
    OutOfMemory { index: u32 },
    Refused,
}

/// 同 `CardsAtWork`（第 2097–2113 行）：这一轮装了状态的卡（池里的位置）、各装几个、一队块、卡自己叫的停与调用方的叫停。
struct Work<'w> {
    script: &'w Script,
    round: usize,
    cards: Vec<usize>,
    states_placed: Vec<u64>,
    blocks: BlocksForTheCards,
    stop: AtomicBool,
}

impl Work<'_> {
    /// 同 `must_stop`（第 2117–2119 行）。
    fn must_stop(&self) -> bool {
        self.stop.load(Ordering::SeqCst) || self.script.stop_file.load(Ordering::SeqCst)
    }
}

/// 同 `start_the_next_card_with_states`（第 2444–2460 行）。
fn start_the_next<'s, 'w>(scope: &'s std::thread::Scope<'s, 'w>, work: &'w Work<'w>, position: usize, sender: &Sender<Message>) {
    let next_position = (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
    if let Some(next_position) = next_position {
        if work.blocks.any_left() && !work.must_stop() {
            let sender = sender.clone();
            scope.spawn(move || card_thread(scope, work, next_position, sender));
        }
    }
}

/// 同 `judge_on_one_card_and_start_the_next`（第 2221–2391 行）。
fn card_thread<'s, 'w>(scope: &'s std::thread::Scope<'s, 'w>, work: &'w Work<'w>, position: usize, sender: Sender<Message>) {
    let script = work.script;
    let pool_position = work.cards[position];
    let card = &script.cards[pool_position];
    script.event();
    if card.out_of_memory_at_build_in_round.contains(&work.round) {
        let _ = sender.send(Message::OutOfMemory { index: card.index });
        start_the_next(scope, work, position, &sender);
        if pool_position != 0 {
            script.others_left.fetch_add(1, Ordering::SeqCst);
        }
        return;
    }
    start_the_next(scope, work, position, &sender);
    let card_left = judge_blocks_on_one_card(
        &work.blocks,
        &|| work.must_stop(),
        &mut |pending_index| {
            script.event();
            for _ in 0..card.yields_each_block {
                std::thread::yield_now();
            }
            let nth = script.blocks_of_each_card[pool_position].fetch_add(1, Ordering::SeqCst) + 1;
            if pool_position == 0 && nth == 1 && script.card_0_waits_for_the_others {
                let others = script.cards.len() - 1;
                let since = std::time::Instant::now();
                while script.others_left.load(Ordering::SeqCst) < others && since.elapsed() < Duration::from_secs(20) {
                    std::thread::yield_now();
                }
            }
            if card.out_of_memory_at_its_block == Some(nth) {
                for _ in 0..card.yields_before_out_of_memory {
                    std::thread::yield_now();
                }
                return BlockJudgedOnACard::OutOfMemory { reason: format!("卡 {} 判块 {pending_index}", card.index) };
            }
            script.judged_times.lock().unwrap()[pending_index] += 1;
            BlockJudgedOnACard::Judged(pending_index)
        },
        &mut |pending_index, _judged: usize| match sender.send(Message::Block(pending_index)) {
            Ok(()) => BlockHandedToTheStore::Handed,
            Err(_) => BlockHandedToTheStore::TheStoreIsGone,
        },
    );
    if pool_position != 0 {
        script.others_left.fetch_add(1, Ordering::SeqCst);
    }
    match card_left {
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {}
        CardLeftTheBlocks::OutOfMemory { .. } => {
            let _ = sender.send(Message::OutOfMemory { index: card.index });
            return;
        }
        CardLeftTheBlocks::Refused { .. } => {
            work.stop.store(true, Ordering::SeqCst);
            let _ = sender.send(Message::Refused);
            return;
        }
    }
    let _ = sender.send(Message::Finished);
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ended {
    EveryBlockStored,
    StoppedOnRequest,
    NotRun,
    /// 第 3076–3091 行的 assert 判不过（原文 panic）：带那一句。
    AssertPanicked(String),
    /// 第 3007–3021 行：没有一张卡装了状态、而之前没人报过 Out of Memory（判红）；这里不会走到，留着核。
    Red,
}

struct Run {
    ended: Ended,
    rounds: usize,
    stored: usize,
    judged_times: Vec<usize>,
    stop_requested_at_the_end: bool,
}

/// 同第 2919–3148 行的几轮：每轮挑没报过的卡、照这一刻每张卡带得了的按优先级装状态（`place_states_on_cards_by_priority` 的走法：先装满第一张），
/// 装了状态的卡起成一串、判、收；收完照 `after_a_round_of_the_cards` 走。`pending` 块，每块一个状态。
fn run(script: &Script, pending: usize) -> Run {
    let mut stored_in_order: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
    let mut blocks_of_the_round: Vec<usize> = (0..pending).collect();
    let mut cards_short_of_memory: BTreeSet<u32> = BTreeSet::new();
    let mut first_out_of_memory = false;
    let mut round = 0;
    let finish = |ended: Ended, rounds: usize, stored: usize| Run {
        ended,
        rounds,
        stored,
        judged_times: script.judged_times.lock().unwrap().clone(),
        stop_requested_at_the_end: script.stop_file.load(Ordering::SeqCst),
    };
    loop {
        let run_cards: Vec<usize> = (0..script.cards.len())
            .filter(|position| !cards_short_of_memory.contains(&script.cards[*position].index))
            .collect();
        let capacities: Vec<u64> = run_cards
            .iter()
            .map(|position| {
                let card = &script.cards[*position];
                *card.capacity_in_round.get(round).or(card.capacity_in_round.last()).unwrap()
            })
            .collect();
        // 第 2957–2989 行（第 2981 行那一支）：每张卡此刻都带不了一个状态（空着时带得了）：本次未跑
        if capacities.iter().all(|capacity| *capacity == 0) {
            return finish(Ended::NotRun, round + 1, stored_in_order.blocks_stored());
        }
        let mut left = blocks_of_the_round.len() as u64;
        let mut cards = Vec::new();
        let mut states_placed = Vec::new();
        for (position, capacity) in run_cards.iter().zip(&capacities) {
            let placed = left.min(*capacity);
            if placed > 0 {
                cards.push(*position);
                states_placed.push(placed);
                left -= placed;
            }
        }
        if cards.is_empty() {
            return finish(if first_out_of_memory { Ended::NotRun } else { Ended::Red }, round + 1, stored_in_order.blocks_stored());
        }
        let work = Work { script, round, cards, states_placed, blocks: BlocksForTheCards::new(blocks_of_the_round.iter().copied()), stop: AtomicBool::new(false) };
        let (cards_out_of_memory, assert_failed) = std::thread::scope(|scope| {
            let (sender, receiver) = channel::<Message>();
            let work = &work;
            scope.spawn(move || card_thread(scope, work, 0, sender));
            let mut cards_out_of_memory: Vec<u32> = Vec::new();
            let mut refusal = false;
            for message in receiver {
                match message {
                    Message::Finished => {}
                    Message::Refused => refusal = true,
                    Message::OutOfMemory { index } => cards_out_of_memory.push(index),
                    Message::Block(pending_index) => stored_in_order.arrive(pending_index, pending_index, &mut |_| Ok(())),
                }
            }
            // 第 3076–3091 行
            let mut assert_failed = None;
            if stored_in_order.storing_error().is_none() && !refusal && cards_out_of_memory.is_empty() {
                if stored_in_order.blocks_waiting() != 0 {
                    assert_failed = Some(format!("领走的块每一块都写进了库（等着的 {} 块）", stored_in_order.blocks_waiting()));
                } else if work.blocks.any_left() && !script.stop_file.load(Ordering::SeqCst) {
                    assert_failed = Some(format!("没人叫停，却有 {} 块没判", work.blocks.blocks_not_taken().len()));
                }
            }
            (cards_out_of_memory, assert_failed)
        });
        if let Some(message) = assert_failed {
            return finish(Ended::AssertPanicked(message), round + 1, stored_in_order.blocks_stored());
        }
        for index in &cards_out_of_memory {
            cards_short_of_memory.insert(*index);
            first_out_of_memory = true;
        }
        blocks_of_the_round = work.blocks.blocks_not_taken();
        let cards_not_short_of_memory = run_cards
            .iter()
            .filter(|position| !cards_short_of_memory.contains(&script.cards[**position].index))
            .count();
        let stop = if script.stop_file.load(Ordering::SeqCst) { StopOfTheRun::Requested } else { StopOfTheRun::NotRequested };
        match after_a_round_of_the_cards(pending - stored_in_order.blocks_stored(), cards_out_of_memory.len(), stop, cards_not_short_of_memory) {
            AfterARoundOfTheCards::EveryBlockStored => return finish(Ended::EveryBlockStored, round + 1, stored_in_order.blocks_stored()),
            AfterARoundOfTheCards::StoppedOnRequest => return finish(Ended::StoppedOnRequest, round + 1, stored_in_order.blocks_stored()),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => return finish(Ended::NotRun, round + 1, stored_in_order.blocks_stored()),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => round += 1,
        }
        assert!(round <= script.cards.len(), "轮数超过卡数");
    }
}

fn script(cards: Vec<FakeCard>, stop_at_event: Option<usize>, card_0_waits_for_the_others: bool, pending: usize) -> Script {
    let count = cards.len();
    Script {
        cards,
        stop_at_event,
        events: AtomicUsize::new(0),
        stop_file: AtomicBool::new(false),
        card_0_waits_for_the_others,
        others_left: AtomicUsize::new(0),
        judged_times: Mutex::new(vec![0; pending]),
        blocks_of_each_card: (0..count).map(|_| AtomicUsize::new(0)).collect(),
    }
}

fn card(index: u32, capacity: u64, out_of_memory_at_its_block: Option<usize>) -> FakeCard {
    FakeCard { index, capacity_in_round: vec![capacity], out_of_memory_at_build_in_round: Vec::new(), out_of_memory_at_its_block, yields_each_block: 0, yields_before_out_of_memory: 0 }
}

/// 定形：两张卡、5 块。第 0 张装 1 个状态、判它的第 1 块（块 0）时等第 1 张收工再报 Out of Memory、交回块 0；第 1 张判完块 1–4（等着块 0）。
/// 头一轮有 7 个事件（两次建判器、5 次判块）。`stop_at_event` 取 8：叫停文件在收尾再起的那一轮第 1 张卡建判器时出现。
#[test]
fn k1_5_a_stop_during_the_round_started_at_the_end_trips_the_assert() {
    for (name, stop_at_event) in [("no-stop", None), ("stop-at-round-1-last-block", Some(7)), ("stop-at-round-2-build", Some(8)), ("stop-at-round-2-first-block", Some(9))] {
        let s = script(vec![card(0, 1, Some(1)), card(1, 10, None)], stop_at_event, true, 5);
        let r = run(&s, 5);
        let judged: usize = r.judged_times.iter().sum();
        println!(
            "ATTACK k1_5=stop-in-the-closing-round cell={name} ended={:?} rounds={} stored={} judged_once_or_more={} judged_not_stored={} stop_requested={}",
            r.ended, r.rounds, r.stored, judged, judged - r.stored, r.stop_requested_at_the_end
        );
    }
}

#[test]
fn k1_5_random_rounds() {
    let mut x = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move |bound: u64| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % bound
    };
    let mut tally: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let (mut runs, mut judged_twice, mut every_stored_but_missing, mut assert_without_stop, mut stopped_without_stop, mut more_rounds_than_cards) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let mut judged_not_stored_at_assert = 0u64;
    let mut judged_not_stored_elsewhere = 0u64;
    for _ in 0..6_000 {
        let card_count = 1 + next(4) as usize;
        let pending = 1 + next(8) as usize;
        let cards: Vec<FakeCard> = (0..card_count)
            .map(|index| FakeCard {
                index: index as u32,
                capacity_in_round: (0..5).map(|_| [0, 1, 2, 8][next(4) as usize]).collect(),
                out_of_memory_at_build_in_round: (0..5).filter(|_| next(6) == 0).map(|r| r as usize).collect(),
                out_of_memory_at_its_block: (next(3) == 0).then(|| 1 + next(4) as usize),
                yields_each_block: next(200),
                yields_before_out_of_memory: [0, 2_000, 20_000][next(3) as usize],
            })
            .collect();
        let stop_at_event = (next(3) == 0).then(|| 1 + next(20) as usize);
        let s = script(cards, stop_at_event, false, pending);
        let r = run(&s, pending);
        runs += 1;
        let kind = match &r.ended {
            Ended::AssertPanicked(message) => format!("AssertPanicked({})", message.split('（').next().unwrap()),
            other => format!("{other:?}"),
        };
        *tally.entry(format!("{kind} stop_requested={}", r.stop_requested_at_the_end)).or_insert(0) += 1;
        if r.judged_times.iter().any(|times| *times > 1) {
            judged_twice += 1;
        }
        if r.ended == Ended::EveryBlockStored && r.stored != pending {
            every_stored_but_missing += 1;
        }
        if matches!(r.ended, Ended::AssertPanicked(_)) {
            judged_not_stored_at_assert += r.judged_times.iter().sum::<usize>() as u64 - r.stored as u64;
            if !r.stop_requested_at_the_end {
                assert_without_stop += 1;
            }
        } else {
            judged_not_stored_elsewhere += r.judged_times.iter().sum::<usize>() as u64 - r.stored as u64;
        }
        if r.ended == Ended::StoppedOnRequest && !r.stop_requested_at_the_end {
            stopped_without_stop += 1;
        }
        if r.rounds > card_count {
            more_rounds_than_cards += 1;
        }
    }
    for (kind, count) in &tally {
        println!("ATTACK k1_5=random-rounds ended={kind} runs={count}");
    }
    println!(
        "ATTACK k1_5=random-rounds runs={runs} judged_twice={judged_twice} every_stored_but_missing={every_stored_but_missing} assert_without_stop={assert_without_stop} stopped_without_stop={stopped_without_stop} more_rounds_than_cards={more_rounds_than_cards} judged_not_stored_at_assert={judged_not_stored_at_assert} judged_not_stored_elsewhere={judged_not_stored_elsewhere}"
    );
}
