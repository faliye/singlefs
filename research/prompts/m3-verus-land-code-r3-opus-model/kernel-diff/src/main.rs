//! 代码轮第三轮攻方差分：同一个输入喂给第二轮冻结副本的核（old）、第三轮冻结副本的核（new）与几个变异体，
//! 比结局（返回的根、报告五字段、报错种类与内容、panic）与两个回调的调用日志（哪一个、对哪个下标、按什么次序）。
//! 用法：r3-attack-kernel-diff <exhaustive|random|chain|drops> <样本数（exhaustive 不看）> <种子> <线程数>
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[allow(warnings, clippy::all)]
#[path = "kernels/old.rs"]
mod old;
#[allow(warnings, clippy::all)]
#[path = "kernels/new.rs"]
mod new;
#[allow(warnings, clippy::all)]
#[path = "kernels/old_k20.rs"]
mod old_k20;
#[allow(warnings, clippy::all)]
#[path = "kernels/new_k20.rs"]
mod new_k20;
#[allow(warnings, clippy::all)]
#[path = "kernels/new_revert_h52.rs"]
mod new_revert_h52;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Last,
    More,
}

#[derive(Clone, Copy, Debug)]
struct Rec {
    instance: u32,
    counter: u64,
    txg: u64,
    tx: u64,
    commit: bool,
    ordinal: u32,
    place: Place,
}

#[derive(Clone, Debug)]
struct Input {
    root_instance: u32,
    root_txg: u64,
    records: Vec<Rec>,
    back_broken: u64,
    named_ok: u64,
    verify: bool,
    limit: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Outcome {
    Ok { above: usize, applied: usize, passed: usize, failed: usize, max_tx: u64, root: u64 },
    Multi(Vec<u64>),
    NoSuccessor,
    Panic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Observed {
    outcome: Outcome,
    calls: Vec<(char, usize)>,
}

macro_rules! runner {
    ($name:ident, $m:ident, $Inst:ident, $Txg:ident, $Ord:ident, $Place:ident) => {
        fn $name(input: &Input) -> Observed {
            let records: Vec<$m::CoreRecord<u64>> = input
                .records
                .iter()
                .enumerate()
                .map(|(i, r)| $m::CoreRecord {
                    instance: $m::$Inst(r.instance),
                    counter: $m::JournalCounter(r.counter),
                    checkpoint_txg: $m::$Txg(r.txg),
                    transaction: $m::TransactionNumber(r.tx),
                    is_commit: r.commit,
                    ordinal_within_publish: $m::$Ord(r.ordinal),
                    place_in_publish: match r.place {
                        Place::Last => $m::$Place::LastRecordOfThePublish,
                        Place::More => $m::$Place::MoreRecordsOfThePublishFollow,
                    },
                    new_root_source: i as u64,
                })
                .collect();
            let calls = RefCell::new(Vec::new());
            let result = catch_unwind(AssertUnwindSafe(|| {
                $m::replay_core(
                    $m::$Inst(input.root_instance),
                    $m::$Txg(input.root_txg),
                    u64::MAX,
                    &records,
                    input.limit,
                    if input.verify { $m::NamedUnitVerification::Verify } else { $m::NamedUnitVerification::Skip },
                    |i: usize| {
                        calls.borrow_mut().push(('B', i));
                        input.back_broken >> i & 1 == 1
                    },
                    |i: usize| {
                        calls.borrow_mut().push(('N', i));
                        input.named_ok >> i & 1 == 1
                    },
                )
            }));
            let outcome = match result {
                Err(_) => Outcome::Panic,
                Ok(Ok(o)) => Outcome::Ok {
                    above: o.report.above_water,
                    applied: o.report.prefix_applied,
                    passed: o.report.verification_passed,
                    failed: o.report.verification_failed,
                    max_tx: o.report.maximum_applied_transaction.0,
                    root: o.root_source_after_replay,
                },
                Ok(Err($m::CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { counters })) => {
                    Outcome::Multi(counters.iter().map(|c| c.0).collect())
                }
                Ok(Err($m::CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor)) => Outcome::NoSuccessor,
            };
            Observed { outcome, calls: calls.into_inner() }
        }
    };
}

runner!(run_old, old, InstanceGeneration, CheckpointTxg, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish);
runner!(run_old_k20, old_k20, InstanceGeneration, CheckpointTxg, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish);
runner!(run_new, new, PrefixKernelInstanceGeneration, PrefixKernelCheckpointTxg, PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish);
runner!(run_new_k20, new_k20, PrefixKernelInstanceGeneration, PrefixKernelCheckpointTxg, PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish);
runner!(run_new_revert_h52, new_revert_h52, PrefixKernelInstanceGeneration, PrefixKernelCheckpointTxg, PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish);

/// 那一类输入里，第一条候选的「本实例内逻辑前一条」（计数器小 1）在入参里：产品里反向链回调这时才真去读盘。
fn previous_counter_present(input: &Input) -> bool {
    let Some(first) = input.records.iter().find(|r| r.instance == input.root_instance && r.txg > input.root_txg) else {
        return false;
    };
    input.records.iter().any(|r| r.instance == first.instance && r.counter + 1 == first.counter)
}

/// H5-2 改了回调调用的那一类输入：锚点读不出、根 txg 不是最大值、在飞上限至少 1，水位之上第一条候选 txg = 根 + 1 而序号不是 1。
fn in_the_h52_shape(input: &Input) -> bool {
    let flagged = input
        .records
        .iter()
        .filter(|r| r.instance == input.root_instance && r.txg == input.root_txg && r.place == Place::Last)
        .count();
    if flagged != 0 || input.root_txg == u64::MAX || input.limit == 0 {
        return false;
    }
    match input.records.iter().find(|r| r.instance == input.root_instance && r.txg > input.root_txg) {
        Some(first) => first.txg == input.root_txg + 1 && first.ordinal != 1,
        None => false,
    }
}

const CATEGORIES: [&str; 15] = [
    "total",
    "h52_shape",
    "new_vs_old_outcome_diff",
    "new_vs_old_calls_diff",
    "new_vs_old_calls_diff_in_h52_shape",
    "new_vs_old_calls_diff_outside_h52_shape",
    "h52_shape_without_calls_diff",
    "revert_h52_vs_old_outcome_or_calls_diff",
    "old_k20_outcome_differs_from_old",
    "new_k20_outcome_differs_from_new",
    "new_k20_differs_while_old_k20_does_not",
    "old_k20_differs_while_new_k20_does_not",
    "any_panic",
    "old_k20_and_new_k20_outcome_diff",
    "h52_shape_with_previous_counter_present",
];

struct Tally {
    counts: Vec<AtomicU64>,
    examples: Mutex<Vec<(usize, String)>>,
}

impl Tally {
    fn bump(&self, category: usize, input: &Input, detail: impl FnOnce() -> String) {
        self.counts[category].fetch_add(1, Ordering::Relaxed);
        if category >= 2 {
            let mut examples = self.examples.lock().unwrap();
            if examples.iter().filter(|(c, _)| *c == category).count() < 2 {
                examples.push((category, format!("{input:?} => {}", detail())));
            }
        }
    }
}

fn judge(input: &Input, tally: &Tally) {
    let o = run_old(input);
    let n = run_new(input);
    let r = run_new_revert_h52(input);
    let ok = run_old_k20(input);
    let nk = run_new_k20(input);
    tally.bump(0, input, String::new);
    let shape = in_the_h52_shape(input);
    if shape {
        tally.bump(1, input, String::new);
        if previous_counter_present(input) {
            tally.bump(14, input, || format!("old {:?} new {:?}", o, n));
        }
    }
    if o.outcome != n.outcome {
        tally.bump(2, input, || format!("old {:?} new {:?}", o, n));
    }
    if o.calls != n.calls {
        tally.bump(3, input, || format!("old {:?} new {:?}", o.calls, n.calls));
        if shape {
            tally.bump(4, input, || format!("old {:?} new {:?}", o, n));
        } else {
            tally.bump(5, input, || format!("old {:?} new {:?}", o, n));
        }
    } else if shape {
        tally.bump(6, input, || format!("old {:?} new {:?}", o, n));
    }
    if r != o {
        tally.bump(7, input, || format!("old {:?} revert {:?}", o, r));
    }
    let old_k20_differs = ok.outcome != o.outcome;
    let new_k20_differs = nk.outcome != n.outcome;
    if old_k20_differs {
        tally.bump(8, input, || format!("old {:?} old_k20 {:?}", o.outcome, ok.outcome));
    }
    if new_k20_differs {
        tally.bump(9, input, || format!("new {:?} new_k20 {:?}", n.outcome, nk.outcome));
    }
    if new_k20_differs && !old_k20_differs {
        tally.bump(10, input, || format!("new {:?} new_k20 {:?} old_k20 {:?}", n.outcome, nk.outcome, ok.outcome));
    }
    if old_k20_differs && !new_k20_differs {
        tally.bump(11, input, || format!("old {:?} old_k20 {:?} new_k20 {:?}", o.outcome, ok.outcome, nk.outcome));
    }
    if [&o, &n, &r, &ok, &nk].iter().any(|x| x.outcome == Outcome::Panic) {
        tally.bump(12, input, || format!("{:?} {:?} {:?} {:?} {:?}", o.outcome, n.outcome, r.outcome, ok.outcome, nk.outcome));
    }
    if ok.outcome != nk.outcome {
        tally.bump(13, input, || format!("old_k20 {:?} new_k20 {:?}", ok.outcome, nk.outcome));
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
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
    fn chance(&mut self, numerator: u64, denominator: u64) -> bool {
        self.below(denominator) < numerator
    }
}

fn sort_and_dedupe(records: &mut Vec<Rec>) {
    records.sort_by_key(|r| (r.instance, r.counter));
    records.dedup_by_key(|r| (r.instance, r.counter));
}

/// 穷举族：实例 1、根 txg 1；n ≤ 3 条，计数器取 {1..5} 的升序子集；每条 txg∈{1,2,3}、序号∈{1,2,3}、末条标志、提交标记、事务号∈{1,2}；
/// 回调：反向链断的下标全部子集 × 点名单元三种（全过、偶数下标过、奇数下标过）× 验 / 不验 × 在飞上限 {1, 16}。
fn exhaustive(threads: usize, tally: &Tally) {
    let mut structures: Vec<(u32, u64)> = Vec::new();
    for subset in 0u32..32 {
        let n = subset.count_ones();
        if n > 3 {
            continue;
        }
        for combo in 0..72u64.pow(n) {
            structures.push((subset, combo));
        }
    }
    let next = AtomicU64::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let start = next.fetch_add(4096, Ordering::Relaxed) as usize;
                if start >= structures.len() {
                    break;
                }
                for &(subset, combo) in &structures[start..(start + 4096).min(structures.len())] {
                    let mut records = Vec::new();
                    let mut c = combo;
                    for counter in 1..=5u64 {
                        if subset >> (counter - 1) & 1 == 0 {
                            continue;
                        }
                        let txg = 1 + c % 3;
                        c /= 3;
                        let ordinal = 1 + (c % 3) as u32;
                        c /= 3;
                        let place = if c % 2 == 0 { Place::Last } else { Place::More };
                        c /= 2;
                        let commit = c % 2 == 0;
                        c /= 2;
                        let tx = 1 + c % 2;
                        c /= 2;
                        records.push(Rec { instance: 1, counter, txg, tx, commit, ordinal, place });
                    }
                    let n = records.len();
                    let full = (1u64 << n) - 1;
                    for back_broken in 0..=full {
                        for named_ok in [full, full & 0b0101_0101, full & 0b1010_1010] {
                            for verify in [true, false] {
                                for limit in [1usize, 16] {
                                    let input = Input { root_instance: 1, root_txg: 1, records: records.clone(), back_broken, named_ok, verify, limit };
                                    judge(&input, tally);
                                }
                            }
                        }
                    }
                }
            });
        }
    });
}

fn random_input(rng: &mut Rng) -> Input {
    let n = rng.below(9) as usize;
    let mut records = Vec::new();
    for _ in 0..n {
        let instance = match rng.below(10) { 0 => 0, 1 => 2, _ => 1 };
        records.push(Rec {
            instance,
            counter: 1 + rng.below(12),
            txg: rng.below(5),
            tx: 1 + rng.below(3),
            commit: rng.chance(4, 5),
            ordinal: if rng.chance(1, 20) { 0 } else { 1 + rng.below(3) as u32 },
            place: if rng.chance(1, 2) { Place::Last } else { Place::More },
        });
    }
    sort_and_dedupe(&mut records);
    let root_txg = match rng.below(20) { 0 => u64::MAX, 1..=6 => 0, 7..=14 => 1, _ => 2 };
    Input { root_instance: 1, root_txg, records, back_broken: rng.next() & rng.next(), named_ok: rng.next() | rng.next(), verify: rng.chance(3, 4), limit: [0, 1, 2, 3, 16][rng.below(5) as usize] }
}

/// 合法写者写出的样子再打扰：若干次发布（每次 1–3 条、序号 1..k、末条带标志、事务可跨两条），再按概率丢记录、改字段。
fn chain_input(rng: &mut Rng, perturb_fields: bool) -> Input {
    let mut records = Vec::new();
    let mut counter = 1 + rng.below(3);
    let publishes = 1 + rng.below(5);
    let mut tx = 1;
    for txg in 0..publishes {
        let k = 1 + rng.below(3);
        for ordinal in 1..=k {
            let spills = ordinal < k && rng.chance(1, 4);
            records.push(Rec { instance: 1, counter, txg, tx, commit: !spills, ordinal: ordinal as u32, place: if ordinal == k { Place::Last } else { Place::More } });
            counter += 1;
            if !spills {
                tx += 1;
            }
        }
    }
    let mut kept: Vec<Rec> = Vec::new();
    for mut r in records {
        if rng.chance(1, 5) {
            continue;
        }
        if !perturb_fields {
            kept.push(r);
            continue;
        }
        if rng.chance(1, 20) { r.ordinal = 1 + rng.below(3) as u32; }
        if rng.chance(1, 20) { r.place = if r.place == Place::Last { Place::More } else { Place::Last }; }
        if rng.chance(1, 20) { r.txg = rng.below(publishes + 1); }
        if rng.chance(1, 20) { r.commit = !r.commit; }
        if rng.chance(1, 30) { r.instance = 2; }
        kept.push(r);
    }
    sort_and_dedupe(&mut kept);
    let root_txg = rng.below(publishes);
    let mut back_broken = 0;
    let mut named_ok = u64::MAX;
    for i in 0..kept.len() {
        if rng.chance(1, 10) { back_broken |= 1 << i; }
        if rng.chance(1, 10) { named_ok &= !(1 << i); }
    }
    Input { root_instance: 1, root_txg, records: kept, back_broken, named_ok, verify: rng.chance(4, 5), limit: [1, 2, 3, 16][rng.below(4) as usize] }
}

fn sampled(threads: usize, samples: u64, seed: u64, family: &str, tally: &Tally) {
    std::thread::scope(|scope| {
        for thread in 0..threads as u64 {
            scope.spawn(move || {
                let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (thread + 1).wrapping_mul(0xD1B5_4A32_D192_ED03) | 1);
                let mut index = thread;
                while index < samples {
                    let input = match family {
                        "chain" => chain_input(&mut rng, true),
                        "drops" => chain_input(&mut rng, false),
                        _ => random_input(&mut rng),
                    };
                    judge(&input, tally);
                    index += threads as u64;
                }
            });
        }
    });
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let family = arguments[1].as_str();
    let samples: u64 = arguments[2].parse().unwrap();
    let seed: u64 = arguments[3].parse().unwrap();
    let threads: usize = arguments[4].parse().unwrap();
    let tally = Tally { counts: (0..CATEGORIES.len()).map(|_| AtomicU64::new(0)).collect(), examples: Mutex::new(Vec::new()) };
    let started = std::time::Instant::now();
    match family {
        "exhaustive" => exhaustive(threads, &tally),
        "random" | "chain" | "drops" => sampled(threads, samples, seed, family, &tally),
        other => panic!("不认得的族 {other}"),
    }
    println!("family={family} samples={samples} seed={seed} threads={threads} seconds={}", started.elapsed().as_secs());
    for (index, name) in CATEGORIES.iter().enumerate() {
        println!("{name}={}", tally.counts[index].load(Ordering::Relaxed));
    }
    let mut examples = tally.examples.into_inner().unwrap();
    examples.sort_by_key(|(c, _)| *c);
    for (category, text) in examples {
        println!("EXAMPLE {} {}", CATEGORIES[category], text);
    }
}
