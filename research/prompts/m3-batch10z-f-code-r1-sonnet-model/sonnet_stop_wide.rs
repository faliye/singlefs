//! 崩溃放量产品路径第 ① 段几轮卡的叫停（第 10z 批第六块，第 10z 批第五块代码轮判决 K2-2、攻方 F2）：叫停是一个文件在不在，
//! 文件出现之后中途被删，外层循环、卡线程、一轮收尾、整趟收尾都问这一趟的同一个 [`StopRequestRememberedInThisRun`]，见过一次叫停就记住。
//! 攻方模型 `research/prompts/m3-batch10z-e-code-r1-opus-model/attack_10ze_rounds.rs` 的 `stop-file-removed` 一臂改到这个接口：checker 档那一圈
//! （`singlefs_checker_tier::crash_amplification` 的外层循环、`judge_on_one_card_and_start_the_next`、`start_the_next_card_with_states`、
//! `CardsAtWork::stop_of_the_cards`、整趟收尾那一句）照次序抄成不碰卡的一份，harness 档的 `BlocksForTheCards`、`judge_blocks_on_one_card`、
//! `next_card_to_start`、`RoundsOfTheCards` 原样调用。线程先后由按种子取的调度器定：卡线程每一步（建判器、问叫停、判一块、交写库）之前停下，
//! 调度器在停着的里挑一条放行；同一种子同一段历史。
use singlefs_harness::gpu_memory_ledger::{
    judge_blocks_on_one_card, next_card_to_start, AfterARoundOfTheCards, BlockHandedToTheStore,
    BlockJudgedOnACard, BlocksForTheCards, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
    StopRequestRememberedInThisRun,
};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Condvar, Mutex};

/// 这一臂跑几段脚本、每段几个种子（几千段：攻方 200 000 段里改前两样结局合计 4 168 段，按比例这里改前各有几十段）。
const SCRIPTS: usize = 6_000;
const SEEDS_OF_EACH_SCRIPT: u64 = 4;
/// 攻方 `stop-file-removed` 一臂的头一个种子。
const FIRST_SEED: u64 = 0x5EED_F00D;

/// SplitMix64：按种子定的伪随机数。
struct SplitMixRandom {
    state: u64,
}

impl SplitMixRandom {
    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

/// 一张假卡这一趟怎么走。
#[derive(Clone, Debug)]
struct CardScript {
    index: u32,
    /// 第 r 轮装得下几个状态（0 是这一轮不上卡）；超出的轮取最后一个。
    states_placed_in_each_round: Vec<u64>,
    /// 第几轮建判器时报 Out of Memory。
    out_of_memory_building_the_judge_in_round: Option<usize>,
    /// 这张卡判的第几块（几轮合计，从 1 数）报 Out of Memory。
    out_of_memory_at_its_block: Option<usize>,
}

impl CardScript {
    fn states_placed(&self, round: usize) -> u64 {
        *self
            .states_placed_in_each_round
            .get(round)
            .or(self.states_placed_in_each_round.last())
            .unwrap_or(&1)
    }
}

/// 一趟的脚本：几块、几张卡、叫停文件在第几个事件出现、第几个事件被删。
#[derive(Clone, Debug)]
struct RunScript {
    blocks: usize,
    cards: Vec<CardScript>,
    stop_file_appears_at_event: Option<usize>,
    stop_file_removed_at_event: Option<usize>,
}

/// 叫停文件在第 `event` 个事件那一刻在不在。
fn stop_file_is_there(script: &RunScript, event: usize) -> bool {
    script
        .stop_file_appears_at_event
        .is_some_and(|appears| event >= appears)
        && !script
            .stop_file_removed_at_event
            .is_some_and(|removed| event >= removed)
}

struct SchedulerState {
    threads_alive: usize,
    threads_parked: BTreeSet<usize>,
    thread_granted: Option<usize>,
}

/// 按种子放行的调度器：每条卡线程每一步之前停下，主线程在停着的里挑一条放行。
struct Scheduler {
    state: Mutex<SchedulerState>,
    changed: Condvar,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            state: Mutex::new(SchedulerState {
                threads_alive: 0,
                threads_parked: BTreeSet::new(),
                thread_granted: None,
            }),
            changed: Condvar::new(),
        }
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, SchedulerState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn register_a_thread(&self) {
        self.locked().threads_alive += 1;
    }

    fn wait_for_the_grant(&self, thread_number: usize) {
        let mut state = self.locked();
        state.threads_parked.insert(thread_number);
        self.changed.notify_all();
        while state.thread_granted != Some(thread_number) {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        state.thread_granted = None;
        state.threads_parked.remove(&thread_number);
    }

    fn a_thread_exits(&self) {
        let mut state = self.locked();
        state.threads_alive -= 1;
        self.changed.notify_all();
    }
}

/// 卡线程收工（正常收或 panic 了）就不再算活着。
struct ThreadExitsTheScheduler<'scheduler>(&'scheduler Scheduler);

impl Drop for ThreadExitsTheScheduler<'_> {
    fn drop(&mut self) {
        self.0.a_thread_exits();
    }
}

enum CardMessage {
    Block(usize),
    Finished(u32),
    OutOfMemory(u32, String),
    CardPanicked(String),
}

/// 同 `CardsAtWork`：这一轮上卡的卡、一队块、卡自己置的停与这一趟的叫停。
struct CardsAtWork<'work> {
    round: usize,
    cards: Vec<&'work CardScript>,
    blocks: BlocksForTheCards,
    the_run_failed: AtomicBool,
    stop_request: &'work StopRequestRememberedInThisRun<'work>,
    scheduler: Scheduler,
    next_thread_number: AtomicUsize,
    events: &'work AtomicUsize,
    sender: Sender<CardMessage>,
    judged_times: &'work Mutex<BTreeMap<usize, usize>>,
    blocks_judged_by_each_card: &'work Mutex<BTreeMap<u32, usize>>,
}

impl CardsAtWork<'_> {
    fn step(&self, thread_number: usize) {
        self.scheduler.wait_for_the_grant(thread_number);
        self.events.fetch_add(1, Ordering::SeqCst);
    }

    /// 同 `CardsAtWork::stop_of_the_cards`：判红了一块都不再领，叫停了只领判完的块在等的。
    fn stop_of_the_cards(&self) -> StopOfTheCards {
        if self.the_run_failed.load(Ordering::SeqCst) {
            StopOfTheCards::TheRunFailed
        } else if self.stop_request.is_requested() {
            StopOfTheCards::RequestedByTheCaller
        } else {
            StopOfTheCards::NotRequested
        }
    }
}

/// 同 `start_the_next_card_with_states`：照 [`next_card_to_start`] 起下一张。
fn start_the_next_card<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
) {
    let states_placed: Vec<u64> = work
        .cards
        .iter()
        .map(|card| card.states_placed(work.round))
        .collect();
    if let Some(next_position) = next_card_to_start(
        &states_placed,
        position,
        &work.blocks,
        work.stop_of_the_cards(),
    ) {
        let thread_number = work.next_thread_number.fetch_add(1, Ordering::SeqCst);
        work.scheduler.register_a_thread();
        scope.spawn(move || card_thread(scope, work, next_position, thread_number));
    }
}

fn card_thread<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    thread_number: usize,
) {
    let _exits = ThreadExitsTheScheduler(&work.scheduler);
    let body = catch_unwind(AssertUnwindSafe(|| {
        judge_on_one_card_and_start_the_next(scope, work, position, thread_number);
    }));
    if let Err(panic) = body {
        let text = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
            .unwrap_or_default();
        let _sent = work.sender.send(CardMessage::CardPanicked(text));
    }
}

/// 同 `judge_on_one_card_and_start_the_next` 的次序：建判器（可报 Out of Memory）、起下一张、判块的那一圈、收工报。
fn judge_on_one_card_and_start_the_next<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    thread_number: usize,
) {
    let card = work.cards[position];
    work.step(thread_number);
    if card.out_of_memory_building_the_judge_in_round == Some(work.round) {
        let _sent = work.sender.send(CardMessage::OutOfMemory(
            card.index,
            format!("卡 {} 建判器时", card.index),
        ));
        start_the_next_card(scope, work, position);
        return;
    }
    start_the_next_card(scope, work, position);
    let card_left = judge_blocks_on_one_card(
        &work.blocks,
        &|| {
            work.step(thread_number);
            work.stop_of_the_cards()
        },
        &mut |pending_index| {
            work.step(thread_number);
            let mut blocks_judged = work
                .blocks_judged_by_each_card
                .lock()
                .expect("假卡不 panic 在锁里");
            let blocks_judged_by_this_card = blocks_judged.entry(card.index).or_insert(0);
            *blocks_judged_by_this_card += 1;
            if card.out_of_memory_at_its_block == Some(*blocks_judged_by_this_card) {
                return BlockJudgedOnACard::OutOfMemory {
                    reason: format!("卡 {} 判块 {pending_index} 时", card.index),
                };
            }
            *work
                .judged_times
                .lock()
                .expect("假卡不 panic 在锁里")
                .entry(pending_index)
                .or_insert(0) += 1;
            BlockJudgedOnACard::Judged(pending_index)
        },
        &mut |pending_index, _judged| {
            work.step(thread_number);
            match work.sender.send(CardMessage::Block(pending_index)) {
                Ok(()) => BlockHandedToTheStore::Handed,
                Err(_) => BlockHandedToTheStore::TheStoreIsGone,
            }
        },
    );
    match card_left {
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {
            let _sent = work.sender.send(CardMessage::Finished(card.index));
        }
        CardLeftTheBlocks::OutOfMemory { reason } => {
            let _sent = work
                .sender
                .send(CardMessage::OutOfMemory(card.index, reason));
        }
        CardLeftTheBlocks::Refused { .. } => {
            work.the_run_failed.store(true, Ordering::SeqCst);
        }
    }
}

/// 一趟的结局。
#[derive(Clone, Debug)]
enum EndOfTheRun {
    EveryBlockStored,
    StoppedOnRequest,
    NotRun,
    /// 轮末断言 panic（这一趟判红）。
    AssertPanicked(String),
    CardPanicked(String),
    NoCardHoldsAState,
    TooManyRounds,
}

#[derive(Clone, Debug)]
struct RunResult {
    end: EndOfTheRun,
    /// 整趟收尾时 `stopped_early`：显存不够那一支恒真，别的照这一趟的叫停（实现第 5591 行那一句，改到 [`StopRequestRememberedInThisRun`]）。
    stopped_early: bool,
    /// 整趟收尾那一刻叫停文件已经不在、这一趟却见过叫停（这一臂要钉的窗口真走到了）。
    stop_seen_and_the_file_gone_at_the_end: bool,
}

/// 同外层循环：每轮挑没报过的卡、上卡的是装得下的那几张，第 0 张起、一张带一张；收工之后照 `RoundsOfTheCards::after_the_round` 走。
fn run_once(script: &RunScript, seed: u64) -> RunResult {
    let mut random = SplitMixRandom { state: seed };
    let events = AtomicUsize::new(0);
    let read_the_stop_file_now = || stop_file_is_there(script, events.load(Ordering::SeqCst));
    let stop_request = StopRequestRememberedInThisRun::new(&read_the_stop_file_now);
    let judged_times = Mutex::new(BTreeMap::new());
    let blocks_judged_by_each_card = Mutex::new(BTreeMap::new());
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(script.blocks);
    let finish = |end: EndOfTheRun, gpu_memory_was_short: bool| {
        events.fetch_add(1, Ordering::SeqCst);
        let stopped_early = gpu_memory_was_short || stop_request.is_requested();
        RunResult {
            end,
            stopped_early,
            stop_seen_and_the_file_gone_at_the_end: stop_request.is_requested()
                && !read_the_stop_file_now(),
        }
    };
    let mut round = 0usize;
    loop {
        if round > script.cards.len() + 1 {
            return finish(EndOfTheRun::TooManyRounds, false);
        }
        let cards_of_the_round: Vec<&CardScript> = script
            .cards
            .iter()
            .filter(|card| !rounds.is_short_of_memory(card.index))
            .collect();
        let cards_placed: Vec<&CardScript> = cards_of_the_round
            .iter()
            .copied()
            .filter(|card| card.states_placed(round) > 0)
            .collect();
        if cards_placed.is_empty() {
            if rounds.first_out_of_memory().is_some() {
                return finish(EndOfTheRun::NotRun, true);
            }
            return finish(EndOfTheRun::NoCardHoldsAState, false);
        }
        let (sender, receiver) = channel::<CardMessage>();
        let work = CardsAtWork {
            round,
            cards: cards_placed,
            blocks: rounds.take_the_blocks_of_the_round(),
            the_run_failed: AtomicBool::new(false),
            stop_request: &stop_request,
            scheduler: Scheduler::new(),
            next_thread_number: AtomicUsize::new(1),
            events: &events,
            sender,
            judged_times: &judged_times,
            blocks_judged_by_each_card: &blocks_judged_by_each_card,
        };
        let mut cards_finished: Vec<u32> = Vec::new();
        let mut cards_out_of_memory: Vec<(u32, String)> = Vec::new();
        let mut card_panic: Option<String> = None;
        let mut receive =
            |rounds_receiving: &mut RoundsOfTheCards<usize>, message: CardMessage| match message {
                CardMessage::Block(pending_index) => {
                    rounds_receiving.stored_in_order().arrive(
                        pending_index,
                        pending_index,
                        &mut |_block| Ok(()),
                    );
                }
                CardMessage::Finished(index) => cards_finished.push(index),
                CardMessage::OutOfMemory(index, reason) => {
                    cards_out_of_memory.push((index, reason))
                }
                CardMessage::CardPanicked(text) => card_panic = Some(text),
            };
        std::thread::scope(|scope| {
            let work = &work;
            work.scheduler.register_a_thread();
            scope.spawn(move || card_thread(scope, work, 0, 0));
            loop {
                {
                    let mut state = work.scheduler.locked();
                    while !(state.thread_granted.is_none()
                        && state.threads_parked.len() == state.threads_alive)
                    {
                        state = work
                            .scheduler
                            .changed
                            .wait(state)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                    if state.threads_alive == 0 {
                        break;
                    }
                    let parked = u64::try_from(state.threads_parked.len()).expect("几条线程");
                    let pick = *state
                        .threads_parked
                        .iter()
                        .nth(usize::try_from(random.below(parked)).expect("比线程数小"))
                        .expect("挑的是停着的里的一条");
                    state.thread_granted = Some(pick);
                    work.scheduler.changed.notify_all();
                }
                while let Ok(message) = receiver.try_recv() {
                    receive(&mut rounds, message);
                }
            }
        });
        while let Ok(message) = receiver.try_recv() {
            receive(&mut rounds, message);
        }
        if let Some(text) = card_panic {
            return finish(EndOfTheRun::CardPanicked(text), false);
        }
        // 一轮收尾：问这一趟的叫停
        events.fetch_add(1, Ordering::SeqCst);
        let stop = stop_request.stop_of_the_run();
        let card_indices_of_the_round: Vec<u32> =
            cards_of_the_round.iter().map(|card| card.index).collect();
        let after = catch_unwind(AssertUnwindSafe(|| {
            rounds.after_the_round(
                &work.blocks,
                &card_indices_of_the_round,
                cards_finished.clone(),
                &cards_out_of_memory,
                stop,
            )
        }));
        match after {
            Err(panic) => {
                let text = panic.downcast_ref::<String>().cloned().unwrap_or_default();
                return finish(EndOfTheRun::AssertPanicked(text), false);
            }
            Ok(AfterARoundOfTheCards::EveryBlockStored) => {
                return finish(EndOfTheRun::EveryBlockStored, false);
            }
            Ok(AfterARoundOfTheCards::StoppedOnRequest) => {
                return finish(EndOfTheRun::StoppedOnRequest, false);
            }
            Ok(AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
                return finish(EndOfTheRun::NotRun, true);
            }
            Ok(
                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
                | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
            ) => {
                round += 1;
            }
        }
    }
}

/// 同攻方 `random_script`（`stop-file-removed` 一臂：叫停文件出现之后再过 1 到 20 个事件被删）。
fn random_script_with_the_stop_file_removed(random: &mut SplitMixRandom) -> RunScript {
    let blocks = 3 + usize::try_from(random.below(5)).expect("小于 5");
    let card_count = 2 + random.below(2);
    let cards = (0..card_count)
        .map(|index| CardScript {
            index: u32::try_from(index).expect("几张卡"),
            states_placed_in_each_round: if random.below(4) == 0 {
                vec![1, 0]
            } else {
                vec![1]
            },
            out_of_memory_building_the_judge_in_round: match random.below(10) {
                0 => Some(0),
                1 => Some(1),
                _ => None,
            },
            out_of_memory_at_its_block: if random.below(2) == 0 {
                None
            } else {
                Some(1 + usize::try_from(random.below(3)).expect("小于 3"))
            },
        })
        .collect();
    let stop_file_appears_at_event = if random.below(4) == 0 {
        None
    } else {
        Some(1 + usize::try_from(random.below(40)).expect("小于 40"))
    };
    let stop_file_removed_at_event = stop_file_appears_at_event
        .map(|appears| appears + 1 + usize::try_from(random.below(20)).expect("小于 20"));
    RunScript {
        blocks,
        cards,
        stop_file_appears_at_event,
        stop_file_removed_at_event,
    }
}

/// 攻方 `stop-file-removed` 一臂（判决第三节第 1 条要它的两样结局都为 0）：叫停文件出现、中途被删，固定种子跑 6 000 段。
/// 轮末断言 panic 的段数、照叫停收尾而整趟没记成提早收尾的段数都是 0；这一臂要钉的窗口（见过叫停、收尾时文件已经不在）至少走到一段。
#[test]
fn a_stop_file_removed_mid_run_neither_panics_the_round_nor_leaves_the_stopped_run_unmarked() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut random = SplitMixRandom { state: FIRST_SEED };
    let mut ends: BTreeMap<String, usize> = BTreeMap::new();
    let mut asserts_panicked = 0usize;
    let mut cards_panicked = 0usize;
    let mut stopped_on_request_but_run_not_marked_stopped_early = 0usize;
    let mut stop_seen_and_the_file_gone_at_the_end = 0usize;
    let mut first_example: Option<String> = None;
    for _ in 0..SCRIPTS {
        let script = random_script_with_the_stop_file_removed(&mut random);
        for seed_number in 0..SEEDS_OF_EACH_SCRIPT {
            let seed = random.next_value() ^ seed_number;
            let result = run_once(&script, seed);
            *ends
                .entry(
                    format!("{:?}", result.end)
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
                .or_insert(0) += 1;
            let failure = match &result.end {
                EndOfTheRun::AssertPanicked(text) => {
                    asserts_panicked += 1;
                    Some(format!("轮末断言 panic（{text}）"))
                }
                EndOfTheRun::CardPanicked(text) => {
                    cards_panicked += 1;
                    Some(format!("卡线程 panic（{text}）"))
                }
                EndOfTheRun::StoppedOnRequest if !result.stopped_early => {
                    stopped_on_request_but_run_not_marked_stopped_early += 1;
                    Some("照叫停收尾而整趟没记成提早收尾".to_string())
                }
                EndOfTheRun::EveryBlockStored
                | EndOfTheRun::StoppedOnRequest
                | EndOfTheRun::NotRun
                | EndOfTheRun::NoCardHoldsAState
                | EndOfTheRun::TooManyRounds => None,
            };
            if let (Some(failure), None) = (failure, &first_example) {
                first_example = Some(format!(
                    "{failure}：seed={seed} script={script:?} result={result:?}"
                ));
            }
            if result.stop_seen_and_the_file_gone_at_the_end {
                stop_seen_and_the_file_gone_at_the_end += 1;
            }
        }
    }
    let _default_hook = std::panic::take_hook();
    println!(
        "STOP_FILE_REMOVED runs={} ends={ends:?} asserts_panicked={asserts_panicked} cards_panicked={cards_panicked} \
         stopped_on_request_but_run_not_marked_stopped_early={stopped_on_request_but_run_not_marked_stopped_early} \
         stop_seen_and_the_file_gone_at_the_end={stop_seen_and_the_file_gone_at_the_end}",
        SCRIPTS * usize::try_from(SEEDS_OF_EACH_SCRIPT).expect("几个种子")
    );
    assert_eq!(asserts_panicked, 0, "轮末断言 panic：{first_example:?}");
    assert_eq!(cards_panicked, 0, "卡线程 panic：{first_example:?}");
    assert_eq!(
        stopped_on_request_but_run_not_marked_stopped_early, 0,
        "照叫停收尾而整趟没记成提早收尾：{first_example:?}"
    );
    assert!(
        stop_seen_and_the_file_gone_at_the_end > 0,
        "见过叫停、收尾时文件已经不在的段一段都没走到：这一臂没钉到要钉的窗口"
    );
}
