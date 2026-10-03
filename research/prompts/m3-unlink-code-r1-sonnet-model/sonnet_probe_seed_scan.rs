//! 正推腿探针（m3-unlink-code-r1，Sonnet）K6.a：在入库用例钉住的种子区间之外，扫一批新种子，看带 unlink 的随机历史有没有「模型与实现对不上」或 checker 红。
//! 只扫、不断言：把每个配置里结局不是 Completed 的历史（有没有 unlink、第几步、签名）打出来。
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, PerStepChecker, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;
use std::sync::Mutex;

fn scan(name: &str, weights: GenerationWeights, execution: HistoryExecution, operations: usize, first_seed: u64, seeds: u64) {
    let report = Mutex::new((0u64, 0u64, 0u64, Vec::<String>::new())); // 段数、含 unlink 的段数、非 Completed 段数、明细
    let next = Mutex::new(first_seed);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| loop {
                let seed = {
                    let mut guard = next.lock().unwrap();
                    if *guard >= first_seed + seeds { return; }
                    *guard += 1;
                    *guard - 1
                };
                let history = generate_history_with_weights(HistorySeed(seed), operations, &weights);
                let first_unlink = history.operations.iter().position(|op| *op == HistoryOperation::Unlink);
                let run = execute_history_with(&history, execution, &SharedStream::new(), &mut |_| {});
                let mut guard = report.lock().unwrap();
                guard.0 += 1;
                if first_unlink.is_some() { guard.1 += 1; }
                match &run.ending {
                    HistoryEnding::Completed => {}
                    HistoryEnding::KnownRed { form, .. } => { guard.2 += 1; guard.3.push(format!("种子 {seed}：已知红第 {form} 条；第一个 unlink 在 {first_unlink:?}")); }
                    HistoryEnding::NewFinding { signature, observation } => {
                        guard.2 += 1;
                        guard.3.push(format!("种子 {seed}：新发现 {signature:?}；第一个 unlink 在 {first_unlink:?}；{:.300}", format!("{observation:?}")));
                    }
                }
            });
        }
    });
    let guard = report.lock().unwrap();
    println!("{name}：{} 段 × {operations} 步，含 unlink {} 段，结局不是 Completed {} 段", guard.0, guard.1, guard.2);
    let mut details = guard.3.clone();
    details.sort();
    for line in details.iter().take(12) { println!("   {line}"); }
}

#[test]
fn scan_new_seeds_for_histories_with_unlink() {
    let base = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 100_000;
    let seeds = std::env::var("SONNET_SCAN_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
    for (name, weights, width, admission, operations) in [
        ("单元区墙 256 槽 准入判着", GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::JudgedByTheFormula, 150),
        ("单元区墙 256 槽 准入关着", GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, HistoryDeviceWidth::UnitAreaOf256Slots, SpaceAdmission::SkippedByTheTestOnlySwitch, 150),
        ("单元区墙 384 槽 准入判着", GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, HistoryDeviceWidth::UnitAreaOf384Slots, SpaceAdmission::JudgedByTheFormula, 150),
        ("复用取样点 4 GiB", GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR, HistoryDeviceWidth::FourGibibytes, SpaceAdmission::JudgedByTheFormula, 30),
        ("快档 BROAD 4 GiB", GenerationWeights::BROAD, HistoryDeviceWidth::FourGibibytes, SpaceAdmission::JudgedByTheFormula, 60),
    ] {
        scan(name, weights, HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: admission }, operations, base, seeds);
    }
}
