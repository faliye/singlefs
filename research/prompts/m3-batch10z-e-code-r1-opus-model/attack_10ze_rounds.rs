//! 第 10z 批第五块代码轮第一轮攻方模型：K1-5 叫停与几轮卡、实现员第 2 处「照叫停收尾」。
//! checker 档那一圈（冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 外层循环第 2941–3156 行、
//! 卡线程 `judge_on_one_card_and_start_the_next` 第 2235–2405 行、`start_the_next_card_with_states` 第 2458–2474 行、
//! `CardsAtWork::stop_of_the_cards` 第 2121–2129 行）照次序抄成不碰卡的一份；harness 档的 `BlocksForTheCards`、
//! `judge_blocks_on_one_card`、`RoundsOfTheCards` 原样调用（一行不改）。checker 档那几行本身不在这里跑：抄得对不对是推的。
//! 线程先后由按种子取的调度器定：卡线程每一步（建判器、问叫停、判一块、交写库）之前停下，调度器在停着的里挑一条放行；
//! 同一种子同一段历史。叫停照 `stop_is_requested_by_the_environment`（第 1384 行：叫停文件在不在，每次现读）：
//! 文件在第 s 个事件出现，「文件被删」一臂另在第 u 个事件消失。
use singlefs_harness::gpu_memory_ledger::{
    judge_blocks_on_one_card, AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard,
    CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards, StopOfTheRun,
};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Condvar, Mutex};

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Clone, Debug)]
struct CardScript {
    index: u32,
    /// 第 r 轮装得下几个状态（0 就是这一轮不上卡）；超出的轮取最后一个。
    placed_in_round: Vec<u64>,
    /// 第 r 轮建判器时报 Out of Memory。
    build_oom_in_round: Option<usize>,
    /// 这张卡判的第 n 块（几轮合计，从 1 数）报 Out of Memory。
    oom_at_its_block: Option<usize>,
}

impl CardScript {
    fn placed(&self, round: usize) -> u64 {
        *self
            .placed_in_round
            .get(round)
            .or(self.placed_in_round.last())
            .unwrap_or(&1)
    }
}

#[derive(Clone, Debug)]
struct RunScript {
    blocks: usize,
    cards: Vec<CardScript>,
    stop_at_event: Option<usize>,
    /// 「文件被删」一臂：叫停文件在第 u 个事件消失。
    unstop_at_event: Option<usize>,
}

struct Scheduler {
    state: Mutex<SchedState>,
    changed: Condvar,
}
struct SchedState {
    alive: usize,
    parked: BTreeSet<usize>,
    granted: Option<usize>,
}

impl Scheduler {
    fn new() -> Self {
        Self { state: Mutex::new(SchedState { alive: 0, parked: BTreeSet::new(), granted: None }), changed: Condvar::new() }
    }
    fn register(&self) {
        self.state.lock().unwrap().alive += 1;
    }
    fn step(&self, id: usize) {
        let mut st = self.state.lock().unwrap();
        st.parked.insert(id);
        self.changed.notify_all();
        while st.granted != Some(id) {
            st = self.changed.wait(st).unwrap();
        }
        st.granted = None;
        st.parked.remove(&id);
    }
    fn exit(&self) {
        let mut st = self.state.lock().unwrap();
        st.alive -= 1;
        self.changed.notify_all();
    }
}

struct ExitGuard<'a>(&'a Scheduler);
impl Drop for ExitGuard<'_> {
    fn drop(&mut self) {
        self.0.exit();
    }
}

enum Msg {
    Block(usize),
    Finished(u32),
    Oom(u32, String),
    CardPanicked(String),
}

/// 同 `CardsAtWork`：这一轮上卡的卡（装了状态的）、一队块、卡自己置的停与调用方的叫停文件。
struct Work<'w> {
    script: &'w RunScript,
    round: usize,
    cards: Vec<&'w CardScript>,
    blocks: singlefs_harness::gpu_memory_ledger::BlocksForTheCards,
    stop: AtomicBool,
    sched: Scheduler,
    next_id: AtomicUsize,
    events: &'w AtomicUsize,
    sender: Sender<Msg>,
    judged_times: &'w Mutex<BTreeMap<usize, usize>>,
    blocks_of_card: &'w Mutex<BTreeMap<u32, usize>>,
}

fn stop_file_at(script: &RunScript, event: usize) -> bool {
    script.stop_at_event.is_some_and(|s| event >= s)
        && !script.unstop_at_event.is_some_and(|u| event >= u)
}

impl Work<'_> {
    fn step(&self, id: usize) {
        self.sched.step(id);
        self.events.fetch_add(1, Ordering::SeqCst);
    }
    fn stop_file(&self) -> bool {
        stop_file_at(self.script, self.events.load(Ordering::SeqCst))
    }
    /// 同 `CardsAtWork::stop_of_the_cards`（第 2121–2129 行）。
    fn stop_of_the_cards(&self) -> StopOfTheCards {
        if self.stop.load(Ordering::SeqCst) {
            StopOfTheCards::TheRunFailed
        } else if self.stop_file() {
            StopOfTheCards::RequestedByTheCaller
        } else {
            StopOfTheCards::NotRequested
        }
    }
}

/// 同 `start_the_next_card_with_states`（第 2458–2474 行）：后面还有上卡的卡、队里照此刻的叫停还有领得的块，就起下一张。
fn start_the_next<'scope, 'env>(s: &'scope std::thread::Scope<'scope, 'env>, w: &'env Work<'env>, pos: usize) {
    if pos + 1 < w.cards.len() && w.blocks.any_left_to_take(w.stop_of_the_cards()) {
        let id = w.next_id.fetch_add(1, Ordering::SeqCst);
        w.sched.register();
        s.spawn(move || card_thread(s, w, pos + 1, id));
    }
}

fn card_thread<'scope, 'env>(s: &'scope std::thread::Scope<'scope, 'env>, w: &'env Work<'env>, pos: usize, id: usize) {
    let _exit = ExitGuard(&w.sched);
    let body = catch_unwind(AssertUnwindSafe(|| card_body(s, w, pos, id)));
    if let Err(panic) = body {
        let text = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|t| t.to_string()))
            .unwrap_or_default();
        let _ = w.sender.send(Msg::CardPanicked(text));
    }
}

/// 同 `judge_on_one_card_and_start_the_next`（第 2235–2405 行）的次序：建判器（可报 Out of Memory）、起下一张、判块的那一圈、收工报。
fn card_body<'scope, 'env>(s: &'scope std::thread::Scope<'scope, 'env>, w: &'env Work<'env>, pos: usize, id: usize) {
    let card = w.cards[pos];
    w.step(id);
    if card.build_oom_in_round == Some(w.round) {
        let _ = w.sender.send(Msg::Oom(card.index, format!("卡 {} 建判器时", card.index)));
        start_the_next(s, w, pos);
        return;
    }
    start_the_next(s, w, pos);
    let left = judge_blocks_on_one_card(
        &w.blocks,
        &|| {
            w.step(id);
            w.stop_of_the_cards()
        },
        &mut |pending_index| {
            w.step(id);
            let mut of_card = w.blocks_of_card.lock().unwrap();
            let n = of_card.entry(card.index).or_insert(0);
            *n += 1;
            if card.oom_at_its_block == Some(*n) {
                return BlockJudgedOnACard::OutOfMemory { reason: format!("卡 {} 判块 {pending_index} 时", card.index) };
            }
            *w.judged_times.lock().unwrap().entry(pending_index).or_insert(0) += 1;
            BlockJudgedOnACard::Judged(pending_index)
        },
        &mut |pending_index, _judged| {
            w.step(id);
            match w.sender.send(Msg::Block(pending_index)) {
                Ok(()) => BlockHandedToTheStore::Handed,
                Err(_) => BlockHandedToTheStore::TheStoreIsGone,
            }
        },
    );
    match left {
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {
            let _ = w.sender.send(Msg::Finished(card.index));
        }
        CardLeftTheBlocks::OutOfMemory { reason } => {
            let _ = w.sender.send(Msg::Oom(card.index, reason));
        }
        CardLeftTheBlocks::Refused { .. } => {
            w.stop.store(true, Ordering::SeqCst);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RunResult {
    pub outcome: String,
    pub judged_twice: usize,
    /// 判完了却没写进库的块（这一趟收尾时）。
    pub judged_not_stored: usize,
    pub stored: usize,
    pub rounds: usize,
    /// 这一趟收尾时 `stopped_early` 照第 5591 行再读一次叫停文件得的值（显存不够那一支恒真）。
    pub stopped_early: bool,
    /// 最后一轮有卡报 Out of Memory。
    pub oom_in_the_last_round: bool,
    /// 叫停文件在这一趟里出现过、卡看到过（卡线程问叫停时）。
    pub card_saw_the_stop: bool,
}

/// 同外层循环（第 2941–3156 行）：每轮挑没报过的卡、上卡的是装得下的那几张，第 0 张起、一张带一张；收工之后照
/// `RoundsOfTheCards::after_the_round` 走；断言 panic 记成 `AssertPanicked`（原文是 panic，这一趟判红）。
fn run_once(script: &RunScript, seed: u64) -> RunResult {
    let mut rng = Rng(seed);
    let events = AtomicUsize::new(0);
    let judged_times = Mutex::new(BTreeMap::new());
    let blocks_of_card = Mutex::new(BTreeMap::new());
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(script.blocks);
    let mut stored_blocks: Vec<usize> = Vec::new();
    let mut result = RunResult::default();
    let mut round = 0usize;
    let finish = |result: &mut RunResult, outcome: String, rounds: &RoundsOfTheCards<usize>, stored: &[usize], short: bool| {
        result.outcome = outcome;
        let judged = judged_times.lock().unwrap();
        result.judged_twice = judged.values().filter(|n| **n > 1).count();
        result.judged_not_stored = judged.keys().filter(|b| !stored.contains(b)).count();
        result.stored = rounds.blocks_stored();
        // 第 5591 行：stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment()（再读一次）
        let event = events.fetch_add(1, Ordering::SeqCst) + 1;
        result.stopped_early = short || stop_file_at(script, event);
        result.card_saw_the_stop = script.stop_at_event.is_some_and(|s| event >= s);
    };
    loop {
        result.rounds = round + 1;
        if round > script.cards.len() + 1 {
            finish(&mut result, "Loop".into(), &rounds, &stored_blocks, false);
            return result;
        }
        let run_cards: Vec<&CardScript> = script.cards.iter().filter(|c| !rounds.is_short_of_memory(c.index)).collect();
        let placed: Vec<&CardScript> = run_cards.iter().copied().filter(|c| c.placed(round) > 0).collect();
        if placed.is_empty() {
            if rounds.first_out_of_memory().is_some() {
                finish(&mut result, "NotRun(no card holds a state)".into(), &rounds, &stored_blocks, true);
            } else {
                finish(&mut result, "Err(no card)".into(), &rounds, &stored_blocks, false);
            }
            return result;
        }
        let (sender, receiver) = channel::<Msg>();
        let work = Work {
            script,
            round,
            cards: placed,
            blocks: rounds.take_the_blocks_of_the_round(),
            stop: AtomicBool::new(false),
            sched: Scheduler::new(),
            next_id: AtomicUsize::new(1),
            events: &events,
            sender,
            judged_times: &judged_times,
            blocks_of_card: &blocks_of_card,
        };
        let mut finished: Vec<u32> = Vec::new();
        let mut ooms: Vec<(u32, String)> = Vec::new();
        let mut card_panic: Option<String> = None;
        let mut drain = |rounds: &mut RoundsOfTheCards<usize>, stored_blocks: &mut Vec<usize>, msg: Msg| match msg {
            Msg::Block(i) => rounds.stored_in_order().arrive(i, i, &mut |b| {
                stored_blocks.push(b);
                Ok(())
            }),
            Msg::Finished(c) => finished.push(c),
            Msg::Oom(c, r) => ooms.push((c, r)),
            Msg::CardPanicked(t) => card_panic = Some(t),
        };
        std::thread::scope(|s| {
            let w = &work;
            w.sched.register();
            s.spawn(move || card_thread(s, w, 0, 0));
            loop {
                {
                    let mut st = w.sched.state.lock().unwrap();
                    while !(st.granted.is_none() && st.parked.len() == st.alive) {
                        st = w.sched.changed.wait(st).unwrap();
                    }
                    if st.alive == 0 {
                        break;
                    }
                    let n = st.parked.len() as u64;
                    let pick = *st.parked.iter().nth(rng.below(n) as usize).unwrap();
                    st.granted = Some(pick);
                    w.sched.changed.notify_all();
                }
                while let Ok(msg) = receiver.try_recv() {
                    drain(&mut rounds, &mut stored_blocks, msg);
                }
            }
        });
        while let Ok(msg) = receiver.try_recv() {
            drain(&mut rounds, &mut stored_blocks, msg);
        }
        if let Some(text) = card_panic {
            finish(&mut result, format!("CardPanicked({text})"), &rounds, &stored_blocks, false);
            return result;
        }
        // 第 3105–3109 行：收工之后现读一次叫停
        let event = events.fetch_add(1, Ordering::SeqCst) + 1;
        let stop = if stop_file_at(script, event) { StopOfTheRun::Requested } else { StopOfTheRun::NotRequested };
        let run_indices: Vec<u32> = run_cards.iter().map(|c| c.index).collect();
        result.oom_in_the_last_round = !ooms.is_empty();
        let after = catch_unwind(AssertUnwindSafe(|| {
            rounds.after_the_round(&work.blocks, &run_indices, finished.clone(), &ooms, stop)
        }));
        match after {
            Err(panic) => {
                let text = panic.downcast_ref::<String>().cloned().unwrap_or_default();
                finish(&mut result, format!("AssertPanicked({text})"), &rounds, &stored_blocks, false);
                return result;
            }
            Ok(AfterARoundOfTheCards::EveryBlockStored) => {
                finish(&mut result, "EveryBlockStored".into(), &rounds, &stored_blocks, false);
                return result;
            }
            Ok(AfterARoundOfTheCards::StoppedOnRequest) => {
                finish(&mut result, "StoppedOnRequest".into(), &rounds, &stored_blocks, false);
                return result;
            }
            Ok(AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
                finish(&mut result, "NotRun".into(), &rounds, &stored_blocks, true);
                return result;
            }
            Ok(AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory)
            | Ok(AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor) => {
                round += 1;
            }
        }
    }
}

fn random_script(rng: &mut Rng, stop_removed: bool) -> RunScript {
    let blocks = 3 + rng.below(5) as usize;
    let card_count = 2 + rng.below(2) as usize;
    let cards = (0..card_count)
        .map(|i| CardScript {
            index: i as u32,
            placed_in_round: if rng.below(4) == 0 { vec![1, 0] } else { vec![1] },
            build_oom_in_round: match rng.below(10) {
                0 => Some(0),
                1 => Some(1),
                _ => None,
            },
            oom_at_its_block: if rng.below(2) == 0 { None } else { Some(1 + rng.below(3) as usize) },
        })
        .collect();
    let stop_at_event = if rng.below(4) == 0 { None } else { Some(1 + rng.below(40) as usize) };
    let unstop_at_event = if stop_removed { stop_at_event.map(|s| s + 1 + rng.below(20) as usize) } else { None };
    RunScript { blocks, cards, stop_at_event, unstop_at_event }
}

#[derive(Default)]
struct Tally {
    runs: usize,
    outcomes: BTreeMap<String, usize>,
    examples: BTreeMap<&'static str, Vec<String>>,
    counts: BTreeMap<&'static str, usize>,
    scripts_with_both_stopped_and_not_run: usize,
}

impl Tally {
    fn hit(&mut self, what: &'static str, script: &RunScript, seed: u64, r: &RunResult) {
        *self.counts.entry(what).or_insert(0) += 1;
        let list = self.examples.entry(what).or_default();
        if list.len() < 2 {
            list.push(format!("seed={seed} script={script:?} result={r:?}"));
        }
    }
}

fn sweep(arm: &str, scripts: usize, seeds_per_script: u64, stop_removed: bool, first_seed: u64) -> BTreeMap<&'static str, usize> {
    let mut rng = Rng(first_seed);
    let mut tally = Tally::default();
    for _ in 0..scripts {
        let script = random_script(&mut rng, stop_removed);
        let mut classes = BTreeSet::new();
        for k in 0..seeds_per_script {
            let seed = rng.next() ^ k;
            let r = run_once(&script, seed);
            tally.runs += 1;
            let class = r.outcome.split('(').next().unwrap_or("").to_string();
            classes.insert(class.clone());
            *tally.outcomes.entry(class.clone()).or_insert(0) += 1;
            if matches!(class.as_str(), "AssertPanicked" | "CardPanicked" | "Loop" | "Err") {
                tally.hit("panic_or_loop_or_err", &script, seed, &r);
            }
            if r.judged_twice > 0 {
                tally.hit("judged_twice", &script, seed, &r);
            }
            if class == "EveryBlockStored" && r.stored != script.blocks {
                tally.hit("every_block_stored_but_short", &script, seed, &r);
            }
            if class == "StoppedOnRequest" && r.judged_not_stored > 0 {
                tally.hit("stopped_with_judged_blocks_not_stored", &script, seed, &r);
            }
            if class == "StoppedOnRequest" && !r.stopped_early {
                tally.hit("stopped_on_request_but_run_not_marked_stopped_early", &script, seed, &r);
                if r.oom_in_the_last_round {
                    tally.hit("same_with_an_out_of_memory_in_the_last_round", &script, seed, &r);
                }
            }
            if class == "StoppedOnRequest" && r.oom_in_the_last_round {
                tally.hit("stopped_on_request_with_an_out_of_memory_in_the_last_round", &script, seed, &r);
            }
            if class == "NotRun" && r.judged_not_stored > 0 {
                tally.hit("not_run_with_judged_blocks_not_stored", &script, seed, &r);
            }
        }
        if classes.contains("StoppedOnRequest") && classes.contains("NotRun") {
            tally.scripts_with_both_stopped_and_not_run += 1;
        }
    }
    println!("ATTACK arm={arm} scripts={scripts} runs={} outcomes={:?} counts={:?} scripts_with_both_stopped_and_not_run={}",
        tally.runs, tally.outcomes, tally.counts, tally.scripts_with_both_stopped_and_not_run);
    for (what, list) in &tally.examples {
        for example in list {
            println!("EXAMPLE arm={arm} {what}: {example}");
        }
    }
    tally.counts
}

fn scripts_from_env(default: usize) -> usize {
    std::env::var("ATTACK_10ZE_SCRIPTS").ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[test]
fn k1_5_monotonic_stop_random_rounds() {
    std::panic::set_hook(Box::new(|_| {}));
    let counts = sweep("monotonic-stop", scripts_from_env(200), 4, false, 0x10E5_0001);
    // 叫停照设计只出现不消失时，K1-5 判据的四条：不 panic、不判两遍、照叫停收尾时判完的块都入库、全入库时块数对
    for what in ["panic_or_loop_or_err", "judged_twice", "stopped_with_judged_blocks_not_stored", "every_block_stored_but_short"] {
        assert_eq!(counts.get(what).copied().unwrap_or(0), 0, "{what}");
    }
}

#[test]
fn k1_5_stop_file_removed_random_rounds() {
    std::panic::set_hook(Box::new(|_| {}));
    let _ = sweep("stop-file-removed", scripts_from_env(200), 4, true, 0x10E5_0002);
}
