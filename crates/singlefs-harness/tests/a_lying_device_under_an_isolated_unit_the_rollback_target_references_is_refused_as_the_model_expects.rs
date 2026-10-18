//! 故障注入里说谎的设备吞了一次单元写、那一份落在管理员回退的目标引用的单元上：回退照 D23（journal 的角色与格式） 已定项 14
//! 「在任何写之前拒」逐盘验，拒成 `RollbackError::IsolatedUnitCopyUnreadableOrMismatched`（隔离单元那一份对不上，C572），模型不知道那一份没落、
//! 说该成。这一拒是设备丢了那一份的投影，按注入点认（`fault_injection::left_by_the_lying_device` 拿实现点名的那一份与被吞的写落在哪比），
//! 不算新发现（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4，采纳；复活集那一拒
//! `ResurrectedUnitCopyUnreadableOrMismatched` 走同一个判据）。
//!
//! 取样点是 C600（吞写之后可写挂载被拒而模型说该成） 登记那组参数的故障注入大档撞出的一段（2026-10-15 主 agent 重跑，记在
//! `records/2026-10-13-代码欠账批.md`「五·6 实测」那一条）：种子 7463871032432355312，规模与执行方式与大档没设环境变量时逐项相同。
//! 改之前模型那一侧把这一拒判成没有理由（`model_comparison.rs` 映射成 `ObservedRefusalReason::Unexplained`），故障注入照报新发现
//! 「模型 RollbackWhileMounted 该成；实现 拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched」。
//! 随机历史的生成器一改，这个种子重放的就可能不再是这一段：那时第一条断言（这一段里有回退被认成说谎的设备留下的）先红，
//! 照「钉值是不是还钉着那一形」重找取样点，不改成放过。不随生成器漂的判别力在 `fault_injection.rs` 的单测
//! `a_refused_rollback_is_left_by_the_lying_device_only_when_the_unit_copy_it_names_overlaps_a_swallowed_write`。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::fault_injection::{
    run_fault_injection_campaign, FaultInjectionCampaign, FaultInjectionWorkerThreads,
};
use singlefs_harness::history::{
    GenerationWeights, HistoryDeviceWidth, HistoryExecution, PerStepChecker,
};

/// 大档撞出这一段的种子（C600 登记那组参数从 7463871032432355113 起 512 段里的一段）。
const SEED_WHERE_A_SWALLOWED_UNIT_WRITE_LANDS_UNDER_THE_ROLLBACK_TARGET: u64 =
    7_463_871_032_432_355_312;

/// 大档每段的步数与注入次数（`tests/fault_injection_fast_tier.rs` 的大档默认，C600 登记那组参数同值）。
const LARGE_TIER_OPERATIONS_PER_HISTORY: usize = 30;
const LARGE_TIER_FAULTS_PER_HISTORY: usize = 6;

#[test]
fn a_rollback_refused_on_a_unit_copy_whose_write_the_lying_device_swallowed_is_left_by_the_lying_device_and_no_new_finding(
) {
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed: SEED_WHERE_A_SWALLOWED_UNIT_WRITE_LANDS_UNDER_THE_ROLLBACK_TARGET,
        seed_count: 1,
        operations_per_history: LARGE_TIER_OPERATIONS_PER_HISTORY,
        faults_per_history: LARGE_TIER_FAULTS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    println!("{rendered}");
    assert_eq!(
        report.tally.faults_that_panicked, 0,
        "注入之后 core panic 了：\n{rendered}"
    );
    assert!(
        report.new_findings.is_empty(),
        "回退拒在被吞的那一份上要记进说谎那一格，不算新发现：\n{rendered}"
    );
    assert_eq!(
        report
            .tally
            .faults_by_injection_point
            .get("PublishOverwrite/unit_write"),
        Some(&2),
        "这一段抽出的注入点里有两次落在覆盖写的单元写上（其中整池第 146 次写那一次被吞）：\n{rendered}"
    );
    assert_eq!(
        report
            .tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"模型说该成、实现拒了\" }"),
        Some(&1),
        "回退那一拒（模型说该成、实现拒了）要按点名的那一份落在被吞的写上放过、记进说谎那一格，一次：\n{rendered}"
    );
}
