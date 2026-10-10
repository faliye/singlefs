//! 正推腿：第 10z 批第五块 K1-1 / K1-3。上一轮攻方 attack_10zc_rounds.rs 与正推 rounds_sim4.rs 改到新接口（RoundsOfTheCards、StopOfTheCards）。
//! 外层循环照 crash_amplification.rs `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`（冻结副本第 2892–3168 行）
//! 与 `judge_on_one_card_and_start_the_next`（第 2235–2405 行）、`start_the_next_card_with_states`（第 2458–2474 行）的次序，
//! 用 harness 档 gpu_memory_ledger 的纯函数与类型（一行不改）搭；装卡（placement）与账是模型：每张卡一个「此刻带得了几个状态」。
//! 抄得对不对是推的（checker 档不在这里跑）。叫停 = 一个会在第 n 个事件出现的标志（不消失，单调）。
use singlefs_harness::gpu_memory_ledger::{
    judge_blocks_on_one_card, AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard,
    BlocksForTheCards, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards, StopOfTheRun,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::Duration;

thread_local! {
    static LAST_STOP_READ: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

#[derive(Clone, Debug)]
struct FakeCard {
    index: u32,
    capacity_in_round: Vec<u64>,
    out_of_memory_at_build_in_round: Vec<usize>,
    out_of_memory_at_its_block: Option<usize>,
    yields_each_block: u64,
    yields_before_out_of_memory: u64,
}

struct Script {
    cards: Vec<FakeCard>,
    stop_at_event: Option<usize>,
    unstop_at_event: Option<usize>,
    events: AtomicUsize,
    stop_file: AtomicBool,
    card_0_waits_for_the_others: bool,
    others_left: AtomicUsize,
    judged_times: Mutex<Vec<usize>>,
    inflight: Mutex<BTreeSet<usize>>,
    /// 叫停出现那一刻：已经判完的块与在判的块
    stop_snapshot: Mutex<Option<(Vec<usize>, Vec<usize>)>>,
    blocks_of_each_card: Vec<AtomicUsize>,
    judged_ok_by_card: Vec<AtomicUsize>,
    /// 每块被领走时那条线程最近一次读到的叫停（1 = 调用方叫停）
    taken_under: Mutex<Vec<Option<u8>>>,
}

impl Script {
    fn event(&self) {
        let n = self.events.fetch_add(1, Ordering::SeqCst) + 1;
        if self.unstop_at_event == Some(n) {
            self.stop_file.store(false, Ordering::SeqCst);
        }
        if self.stop_at_event == Some(n) {
            let judged: Vec<usize> = self
                .judged_times
                .lock()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, t)| **t > 0)
                .map(|(i, _)| i)
                .collect();
            let inflight: Vec<usize> = self.inflight.lock().unwrap().iter().copied().collect();
            *self.stop_snapshot.lock().unwrap() = Some((judged, inflight));
            self.stop_file.store(true, Ordering::SeqCst);
        }
    }
}

enum Message {
    Block(usize),
    Finished(u32),
    OutOfMemory { index: u32 },
    Refused,
}

struct Work<'w> {
    script: &'w Script,
    round: usize,
    cards: Vec<usize>,
    states_placed: Vec<u64>,
    blocks: BlocksForTheCards,
    stop: AtomicBool,
}

impl Work<'_> {
    /// 同 `CardsAtWork::stop_of_the_cards`（冻结副本第 2121–2129 行）
    fn stop_of_the_cards(&self) -> StopOfTheCards {
        if self.stop.load(Ordering::SeqCst) {
            StopOfTheCards::TheRunFailed
        } else if self.script.stop_file.load(Ordering::SeqCst) {
            LAST_STOP_READ.with(|c| c.set(1));
            StopOfTheCards::RequestedByTheCaller
        } else {
            LAST_STOP_READ.with(|c| c.set(0));
            StopOfTheCards::NotRequested
        }
    }
}

fn start_the_next<'s, 'w>(scope: &'s std::thread::Scope<'s, 'w>, work: &'w Work<'w>, position: usize, sender: &Sender<Message>) {
    let next_position = (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
    if let Some(next_position) = next_position {
        if work.blocks.any_left_to_take(work.stop_of_the_cards()) {
            let sender = sender.clone();
            scope.spawn(move || card_thread(scope, work, next_position, sender));
        }
    }
}

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
        &|| work.stop_of_the_cards(),
        &mut |pending_index| {
            script.taken_under.lock().unwrap()[pending_index] = Some(LAST_STOP_READ.with(|c| c.get()));
            script.inflight.lock().unwrap().insert(pending_index);
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
                script.inflight.lock().unwrap().remove(&pending_index);
                return BlockJudgedOnACard::OutOfMemory { reason: format!("卡 {} 判块 {pending_index}", card.index) };
            }
            script.judged_times.lock().unwrap()[pending_index] += 1;
            script.judged_ok_by_card[pool_position].fetch_add(1, Ordering::SeqCst);
            script.inflight.lock().unwrap().remove(&pending_index);
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
    let _ = sender.send(Message::Finished(card.index));
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ended {
    EveryBlockStored,
    StoppedOnRequest,
    NotRun,
    AssertPanicked(String),
    Red,
}

struct Run {
    ended: Ended,
    rounds: usize,
    stored: usize,
    waiting_at_the_end: usize,
    judged_times: Vec<usize>,
    stop_requested_at_the_end: bool,
    cards_used: usize,
    cards_that_judged_a_block: BTreeSet<u32>,
    only_for_waiting_rounds: usize,
    overbooked_states_in_those_rounds: u64,
    stored_in_order_is_prefix: bool,
}

fn run(script: &Script, pending: usize) -> Run {
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(pending);
    let mut round = 0;
    let stored_list = Mutex::new(Vec::<usize>::new());
    let mut only_for_waiting_rounds = 0;
    let mut overbooked = 0u64;
    let mut cards_that_judged_a_block: BTreeSet<u32> = BTreeSet::new();
    let finish = |ended: Ended, rounds_run: usize, rounds: &RoundsOfTheCards<usize>, cards_that_judged: BTreeSet<u32>, only_wait: usize, overbooked: u64| {
        let stored_list = stored_list.lock().unwrap().clone();
        Run {
            ended,
            rounds: rounds_run,
            stored: rounds.blocks_stored(),
            waiting_at_the_end: rounds.blocks_waiting(),
            judged_times: script.judged_times.lock().unwrap().clone(),
            stop_requested_at_the_end: script.stop_file.load(Ordering::SeqCst),
            cards_used: rounds.cards_used(),
            cards_that_judged_a_block: script.cards.iter().enumerate().filter(|(i, _)| script.judged_ok_by_card[*i].load(Ordering::SeqCst) > 0).map(|(_, c)| c.index).collect::<BTreeSet<u32>>(),
            only_for_waiting_rounds: only_wait,
            overbooked_states_in_those_rounds: overbooked,
            stored_in_order_is_prefix: stored_list.iter().enumerate().all(|(k, b)| k == *b),
        }
    };
    loop {
        let run_cards: Vec<usize> = (0..script.cards.len())
            .filter(|position| !rounds.is_short_of_memory(script.cards[*position].index))
            .collect();
        let capacities: Vec<u64> = run_cards
            .iter()
            .map(|position| {
                let card = &script.cards[*position];
                *card.capacity_in_round.get(round).or(card.capacity_in_round.last()).unwrap()
            })
            .collect();
        if capacities.iter().all(|capacity| *capacity == 0) {
            return finish(Ended::NotRun, round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked);
        }
        let pending_states = rounds.blocks_of_the_next_round().len() as u64;
        let mut left = pending_states;
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
            let ended = if rounds.first_out_of_memory().is_some() { Ended::NotRun } else { Ended::Red };
            return finish(ended, round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked);
        }
        let work = Work { script, round, cards, states_placed, blocks: rounds.take_the_blocks_of_the_round(), stop: AtomicBool::new(false) };
        let (finished, cards_out_of_memory, refusal) = std::thread::scope(|scope| {
            let (sender, receiver) = channel::<Message>();
            let work = &work;
            scope.spawn(move || card_thread(scope, work, 0, sender));
            let mut finished: Vec<u32> = Vec::new();
            let mut cards_out_of_memory: Vec<(u32, String)> = Vec::new();
            let mut refusal = false;
            for message in receiver {
                match message {
                    Message::Finished(index) => finished.push(index),
                    Message::Refused => refusal = true,
                    Message::OutOfMemory { index } => cards_out_of_memory.push((index, format!("卡 {index}"))),
                    Message::Block(pending_index) => rounds.stored_in_order().arrive(pending_index, pending_index, &mut |b| {
                        stored_list.lock().unwrap().push(b);
                        Ok(())
                    }),
                }
            }
            (finished, cards_out_of_memory, refusal)
        });
        assert!(!refusal);
        let _ = &mut cards_that_judged_a_block;
        let stop = if script.stop_file.load(Ordering::SeqCst) { StopOfTheRun::Requested } else { StopOfTheRun::NotRequested };
        let run_card_indices: Vec<u32> = run_cards.iter().map(|position| script.cards[*position].index).collect();
        let after = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rounds.after_the_round(&work.blocks, &run_card_indices, finished.clone(), &cards_out_of_memory, stop)
        }));
        let judged_now: BTreeSet<u32> = BTreeSet::new();
        let _ = judged_now;
        let after = match after {
            Ok(after) => after,
            Err(payload) => {
                let message = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                return finish(Ended::AssertPanicked(message), round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked);
            }
        };
        match after {
            AfterARoundOfTheCards::EveryBlockStored => return finish(Ended::EveryBlockStored, round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked),
            AfterARoundOfTheCards::StoppedOnRequest => return finish(Ended::StoppedOnRequest, round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => return finish(Ended::NotRun, round + 1, &rounds, cards_that_judged_a_block, only_for_waiting_rounds, overbooked),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => round += 1,
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
                only_for_waiting_rounds += 1;
                // 下一轮 pending_states 数的是队里所有没领走的块；叫停下真能领的只有比判完的最大块号小的
                let largest = script.judged_times.lock().unwrap().iter().enumerate().filter(|(_, t)| **t > 0).map(|(i, _)| i).max();
                let all = rounds.blocks_of_the_next_round();
                let takeable = all.iter().filter(|b| largest.is_some_and(|l| **b < l)).count();
                overbooked += (all.len() - takeable) as u64;
                round += 1;
            }
        }
        assert!(round <= script.cards.len() + 1, "轮数超过卡数");
    }
}

fn script(cards: Vec<FakeCard>, stop_at_event: Option<usize>, card_0_waits_for_the_others: bool, pending: usize) -> Script {
    let count = cards.len();
    Script {
        cards,
        stop_at_event,
        unstop_at_event: std::env::var("UNSTOP_AFTER").ok().and_then(|v| v.parse::<usize>().ok()).zip(stop_at_event).map(|(d, e)| e + d),
        events: AtomicUsize::new(0),
        stop_file: AtomicBool::new(false),
        card_0_waits_for_the_others,
        others_left: AtomicUsize::new(0),
        judged_times: Mutex::new(vec![0; pending]),
        inflight: Mutex::new(BTreeSet::new()),
        stop_snapshot: Mutex::new(None),
        blocks_of_each_card: (0..count).map(|_| AtomicUsize::new(0)).collect(),
        judged_ok_by_card: (0..count).map(|_| AtomicUsize::new(0)).collect(),
        taken_under: Mutex::new(vec![None; pending]),
    }
}

fn card(index: u32, capacity: u64, out_of_memory_at_its_block: Option<usize>) -> FakeCard {
    FakeCard { index, capacity_in_round: vec![capacity], out_of_memory_at_build_in_round: Vec::new(), out_of_memory_at_its_block, yields_each_block: 0, yields_before_out_of_memory: 0 }
}

#[test]
fn k1_5_a_stop_around_the_closing_round() {
    for (name, stop_at_event) in [("no-stop", None), ("stop-at-round-1-last-block", Some(7)), ("stop-at-round-2-build", Some(8)), ("stop-at-round-2-first-block", Some(9)), ("stop-at-event-1", Some(1)), ("stop-at-event-3", Some(3))] {
        let s = script(vec![card(0, 1, Some(1)), card(1, 10, None)], stop_at_event, true, 5);
        let r = run(&s, 5);
        let judged: usize = r.judged_times.iter().sum();
        println!(
            "ATTACK_NEW k1_5=stop-in-the-closing-round cell={name} ended={:?} rounds={} stored={} judged_once_or_more={} judged_not_stored={} stop_requested={} only_for_waiting_rounds={} cards_used={}",
            r.ended, r.rounds, r.stored, judged, judged - r.stored, r.stop_requested_at_the_end, r.only_for_waiting_rounds, r.cards_used
        );
    }
}

/// 正推 rounds_sim4 的 STOPSIM：2 张卡 10 块，第 0 张慢、判第 1 块就报 OOM；第 1 张快。叫停从第 2 轮起。这里用事件叫停近似：事件 n 出现叫停。
#[test]
fn stopsim_two_cards_ten_blocks() {
    let mut slow = card(0, 8, Some(1));
    slow.yields_each_block = 3000;
    let fast = card(1, 8, None);
    for stop_at_event in [None, Some(2), Some(4), Some(8), Some(12), Some(14)] {
        let s = script(vec![slow.clone(), fast.clone()], stop_at_event, false, 10);
        let r = run(&s, 10);
        let judged: usize = r.judged_times.iter().sum();
        println!(
            "STOPSIM_NEW stop_at_event={stop_at_event:?} ended={:?} rounds={} stored={} judged={} judged_not_stored={} waiting_at_end={} only_for_waiting_rounds={} overbooked_states={} cards_used={} cards_that_judged={:?}",
            r.ended, r.rounds, r.stored, judged, judged - r.stored, r.waiting_at_the_end, r.only_for_waiting_rounds, r.overbooked_states_in_those_rounds, r.cards_used, r.cards_that_judged_a_block
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
    let mut c = std::collections::BTreeMap::<&'static str, u64>::new();
    let mut examples: Vec<String> = Vec::new();
    let mut overbooked_total = 0u64;
    let mut after_stop_total = 0u64;
    let mut runs = 0u64;
    let n_runs: u64 = std::env::var("RUNS").ok().and_then(|v| v.parse().ok()).unwrap_or(6000);
    for _ in 0..n_runs {
        let card_count = 1 + next(4) as usize;
        let pending = 1 + next(10) as usize;
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
        let stop_at_event = (next(2) == 0).then(|| 1 + next(24) as usize);
        let s = script(cards.clone(), stop_at_event, false, pending);
        let r = run(&s, pending);
        runs += 1;
        let kind = match &r.ended {
            Ended::AssertPanicked(message) => format!("AssertPanicked({})", message.split('（').next().unwrap()),
            other => format!("{other:?}"),
        };
        *tally.entry(format!("{kind} stop_requested={}", r.stop_requested_at_the_end)).or_insert(0) += 1;
        let judged_total: usize = r.judged_times.iter().sum();
        let mut flag = |name: &'static str, cond: bool, desc: String| {
            if cond {
                *c.entry(name).or_insert(0) += 1;
                if examples.iter().filter(|e| e.starts_with(name)).count() < 3 {
                    examples.push(format!("{name}: {desc}"));
                }
            }
        };
        let desc = format!("cards={cards:?} pending={pending} stop_at_event={stop_at_event:?} ended={:?} stored={} judged={:?}", r.ended, r.stored, r.judged_times);
        flag("assert_panicked", matches!(r.ended, Ended::AssertPanicked(_)), desc.clone());
        flag("judged_twice", r.judged_times.iter().any(|t| *t > 1), desc.clone());
        flag("store_not_prefix", !r.stored_in_order_is_prefix, desc.clone());
        flag("red", r.ended == Ended::Red, desc.clone());
        flag("every_stored_but_missing", r.ended == Ended::EveryBlockStored && (r.stored != pending || judged_total != pending), desc.clone());
        flag("stopped_with_judged_unstored", r.ended == Ended::StoppedOnRequest && (judged_total != r.stored || r.waiting_at_the_end != 0), desc.clone());
        flag("stopped_without_stop", r.ended == Ended::StoppedOnRequest && !r.stop_requested_at_the_end, desc.clone());
        flag("notrun_judged_unstored", r.ended == Ended::NotRun && judged_total > r.stored, desc.clone());
        flag("more_rounds_than_cards", r.rounds > card_count + 1, desc.clone());
        // 叫停读到之后才领走的块（领块之前最近一次读到调用方叫停）必须比「判完的块里最大的」小（那一块要它）：
        // 判完的块里最大的那块自己必须是叫停之前读到的（NotRequested）领走的
        {
            let taken = s.taken_under.lock().unwrap().clone();
            let max_judged = (0..pending).filter(|b| r.judged_times[*b] > 0).max();
            let violators: Vec<usize> = (0..pending)
                .filter(|b| r.judged_times[*b] > 0 && taken[*b] == Some(1) && max_judged.is_some_and(|m| *b >= m))
                .collect();
            flag("judged_block_taken_under_stop_not_below_the_max_judged", !violators.is_empty(), format!("{desc} violators={violators:?} taken_under={taken:?}"));
            // 对照：叫停读到之后领走并判完的块有几块（叫停之后多判的）
            let after_stop = (0..pending).filter(|b| r.judged_times[*b] > 0 && taken[*b] == Some(1)).count();
            after_stop_total += after_stop as u64;
        }
        // cards_used 与「真判过块的卡」
        flag("cards_used_lt_cards_that_judged", r.cards_used < r.cards_that_judged_a_block.len(), format!("{desc} cards_used={} judged_by={:?}", r.cards_used, r.cards_that_judged_a_block));
        flag("cards_used_gt_cards_that_judged", r.cards_used > r.cards_that_judged_a_block.len(), format!("{desc} cards_used={} judged_by={:?}", r.cards_used, r.cards_that_judged_a_block));
        flag("cards_used_gt_card_count", r.cards_used > card_count, desc.clone());
        overbooked_total += r.overbooked_states_in_those_rounds;
    }
    for (kind, count) in &tally {
        println!("ATTACK_NEW k1_5=random-rounds ended={kind} runs={count}");
    }
    println!("ATTACK_NEW k1_5=random-rounds runs={runs} flags={c:?} overbooked_states_total={overbooked_total} blocks_judged_after_the_stop_was_read_total={after_stop_total}");
    for e in &examples {
        println!("ATTACK_NEW example {e}");
    }
}

/// cards_used：一张卡判了几块之后报 Out of Memory，别的卡判完剩下的；用到的卡是两张，cards_used 数几张。
#[test]
fn cards_used_when_a_card_judges_some_blocks_and_then_runs_out() {
    let mut a = card(0, 2, Some(4));
    a.yields_each_block = 0;
    let mut b = card(1, 8, None);
    b.yields_each_block = 2000;
    let s = script(vec![a, b], None, false, 10);
    let r = run(&s, 10);
    println!(
        "CARDS_USED_NEW ended={:?} rounds={} stored={} cards_used={} cards_that_judged_a_block={:?} judged_by_card={:?}",
        r.ended, r.rounds, r.stored, r.cards_used, r.cards_that_judged_a_block,
        s.judged_ok_by_card.iter().map(|c| c.load(Ordering::SeqCst)).collect::<Vec<_>>()
    );
}
