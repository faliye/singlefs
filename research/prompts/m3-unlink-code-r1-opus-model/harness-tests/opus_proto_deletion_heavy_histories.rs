//! 攻方原型（m3-unlink-code-r1，opus 腿）：E4 偏删除的随机历史（unlink、回退、崩溃恢复、可写挂载占大头），看模型与实现在哪种历史上分叉，
//! 与执行器为删除中的版本跳过了多少步。种子区间与比重是这一条腿自己起的，只在草稿副本上跑。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    run_history_campaign, FindingShrinking, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
    HistoryOperationKind as K, HistoryStartingPoint, NamedGenerationWeights, PerStepChecker,
    RollbackTargetDraw,
};

const DELETE_HEAVY: GenerationWeights = GenerationWeights {
    name: "opus 攻方：偏删除",
    named: NamedGenerationWeights::Broad,
    starting_points: &[(HistoryStartingPoint::AfterFirstFile, 3), (HistoryStartingPoint::AfterMakeFilesystem, 1)],
    with_session_closed: &[(K::CloseAndMountWritable, 90), (K::ColdStartRecover, 10)],
    with_session_open_without_file: &[
        (K::PublishFirstFile, 60),
        (K::PublishWithoutUnits, 10),
        (K::CloseAndMountWritable, 8),
        (K::CrashRecoveryAbandoningTheNewestRoot, 4),
        (K::RollBackWhileMounted, 8),
        (K::ColdStartRecover, 4),
        (K::RaiseRollbackFloor, 6),
    ],
    with_session_open_with_file: &[
        (K::PublishOverwrite, 30),
        (K::Unlink, 25),
        (K::RollBackWhileMounted, 15),
        (K::RaiseRollbackFloor, 10),
        (K::CloseAndMountWritable, 8),
        (K::CrashRecoveryAbandoningTheNewestRoot, 6),
        (K::ColdStartRecover, 3),
        (K::PublishFirstFile, 2),
        (K::PublishWithoutUnits, 1),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (K::CreateInode, 20),
        (K::RollBackWhileMounted, 30),
        (K::CloseAndMountWritable, 15),
        (K::CrashRecoveryAbandoningTheNewestRoot, 10),
        (K::RaiseRollbackFloor, 10),
        (K::ColdStartRecover, 5),
        (K::PublishOverwrite, 4),
        (K::PublishFirstFile, 3),
        (K::PublishWithoutUnits, 3),
    ]),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
};

fn seeds_and_ops() -> (u64, u64, usize) {
    let first = std::env::var("OPUS_FIRST_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(7_300_000_000);
    let count = std::env::var("OPUS_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let ops = std::env::var("OPUS_OPS").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    (first, count, ops)
}

fn run(width: HistoryDeviceWidth, weights: &GenerationWeights) {
    let (first, count, ops) = seeds_and_ops();
    let threads = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let started = std::time::Instant::now();
    let report = run_history_campaign(first, count, ops, weights, execution, threads, FindingShrinking::ReportSeedsOnly);
    let t = &report.tally;
    println!("E4 width={width:?} weights={} seeds=[{first},{}) ops={ops} secs={:.1}", weights.name, first + count, started.elapsed().as_secs_f64());
    println!("E4   completed={} new_findings={} known_red={} checker_runs={} model_judged_steps={}", t.histories_completed, report.new_findings.len(), report.known_red_hits.len(), t.checker_runs, t.model_judged_steps);
    println!("E4   not_applicable={:?}", t.not_applicable_by_precondition);
    println!("E4   refusals={:?}", t.refusals_by_member);
    for (kind, tally) in &t.operations_by_kind {
        println!("E4   op {kind:?}: {tally:?}");
    }
    for finding in &report.new_findings {
        println!("E4 FINDING\n{}", finding.render());
    }
}

#[test]
fn e4_delete_heavy_on_four_gibibytes() {
    run(HistoryDeviceWidth::FourGibibytes, &DELETE_HEAVY);
}

#[test]
fn e4_delete_heavy_on_384_slot_devices() {
    run(HistoryDeviceWidth::UnitAreaOf384Slots, &DELETE_HEAVY);
}

#[test]
fn e4_delete_heavy_on_240_slot_devices() {
    run(HistoryDeviceWidth::UnitAreaOf240Slots, &DELETE_HEAVY);
}

#[test]
fn e4_broad_on_four_gibibytes() {
    run(HistoryDeviceWidth::FourGibibytes, &GenerationWeights::BROAD);
}
