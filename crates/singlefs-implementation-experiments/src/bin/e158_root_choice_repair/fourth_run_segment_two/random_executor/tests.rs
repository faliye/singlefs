use super::*;
use crate::fourth_run_segment_two::tests::{row_forty_three, today};

/// 给定状态引擎交回的一份结论换成这一段的判定：红的不变量照违例位、按 `IMPLEMENTED_INVARIANTS` 的次序（不按位号以外的次序）；
/// I-7.4 点名的根按细节次序、重复照留的一份原样交，去重的一份按第一次出现的次序（变异：去重那一句恒真、根的 txg 错一）。
#[test]
fn the_checker_reading_of_a_conclusion_keeps_the_invariant_order_and_dedups_only_the_named_roots_kept_once(
) {
    use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_verification::crash_given_states::{
        AbandonedRootNamedByThePoolChecker, GivenStateConclusion,
    };
    use singlefs_verification::crash_judge_conclusions::{
        CrossCheckWords, LandedRootWords, SCAN_REBUILD_OUTCOME_NOT_REACHED,
    };
    let position_of = |invariant: &str| {
        IMPLEMENTED_INVARIANTS
            .iter()
            .position(|implemented| *implemented == invariant)
            .expect("用例点名的不变量在已实现的表里")
    };
    let later_invariant = "I-7.4";
    let earlier_invariant = IMPLEMENTED_INVARIANTS[0];
    assert!(position_of(earlier_invariant) < position_of(later_invariant));
    let violated_bits =
        (1u128 << position_of(later_invariant)) | (1u128 << position_of(earlier_invariant));
    let named = |instance: u32, checkpoint_txg: u64| AbandonedRootNamedByThePoolChecker {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
    };
    let conclusion = GivenStateConclusion {
        words: CrossCheckWords {
            red_bits: 0,
            pool_checker_violated_bits: violated_bits,
            consulted_pass_landed_root: LandedRootWords::NOT_LANDED,
            ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
            consulted_pass_outcome: 0,
            ignored_pass_outcome: 0,
            consulted_pass_failure_point: 0,
            ignored_pass_failure_point: 0,
            pool_checker_judged_bits: violated_bits,
            scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
            gpu_could_not_judge: false,
        },
        pool_checker_violation_details: Vec::new(),
        abandoned_roots_named_by_the_pool_checker: vec![named(1, 5), named(12, 340), named(1, 5)],
    };
    let reading = checker_reading_of_the_conclusion(&conclusion);
    assert_eq!(
        reading.red_invariants,
        vec![earlier_invariant, later_invariant],
        "红的不变量按已实现那张表的次序"
    );
    assert_eq!(
        reading.abandoned_roots_named_in_the_order_of_the_detail,
        vec![(1, 5), (12, 340), (1, 5)],
        "按细节次序、重复照留的那一份原样交"
    );
    assert_eq!(
        reading.abandoned_roots_named,
        vec![(1, 5), (12, 340)],
        "去重的那一份按第一次出现的次序"
    );
    assert!(reading.abandoned_half_is_red());
}

/// 登记 5.3：这一族判红不截断，第 43 行那条 8 步走完、尾巴跑了（变异 M22 打开「判红就停」）。
/// 每一步的判定由用例给、每一步都红（I-7.4 红、不点名根）：判红不截断是执行器的行为，与判法无关，不经给定状态引擎
/// （配了 GPU 而这份二进制没编进 `gpu` 特性时引擎只录不判，这一条照样判得出）。
#[test]
fn row_forty_three_walks_every_step_and_runs_the_tail() {
    let segment = row_forty_three();
    let every_step_is_red = |_image: &MemoryPool| CheckerReading {
        red_invariants: vec!["I-7.4"],
        abandoned_roots_named: Vec::new(),
        abandoned_roots_named_in_the_order_of_the_detail: Vec::new(),
    };
    let (executor, branch) = run_the_main_branch(
        today(),
        &segment,
        PoolCheckerOnEveryStep::ReadingsGivenByATest(every_step_is_red),
    );
    assert!(
        executor
            .steps
            .iter()
            .filter(|step| step.order >= 1)
            .filter_map(|step| step.checker.as_ref())
            .any(|reading| !reading.red_invariants.is_empty()),
        "用例给的判定逐步都红：执行器至少记下一步判红的（不然「判红不截断」无从核）"
    );
    let walked = executor
        .steps
        .iter()
        .filter(|step| step.order >= 1 && step.order <= 8)
        .count();
    assert_eq!(walked, 8);
    assert!(executor.steps.iter().any(|step| step.operation == "tail"));
    assert_eq!(executor.stopped_at_the_first_red, None);
    assert_eq!(branch.cells.len(), 1, "一次崩溃恢复抛弃根，一格");
}

/// 最短复现、系统配置没见证到被藏的根（崩溃恢复那一步之前每块盘世代号最大的那一槽系统配置写成
/// `crate::tests::system_configuration_slot_before_the_rotation_of_the_newest_publish` 给的字节、不写回；C554 乙合进 `crates` 之后见证在时崩溃恢复那次挂载拒）：
/// 崩溃恢复那次挂载交回 `Ok`、落在没有文件的那一版，Q2 = 1（变异 M36 把更旧写成更新）。
#[test]
fn shortest_reproduction_with_the_witness_unreadable_returns_the_older_version() {
    let history = shortest_reproduction_history();
    let crash_recovery_step = history.operations.len();
    let mut executor = RandomExecutor::start(
        today(),
        history.starting_point,
        SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
        PoolCheckerOnEveryStep::Skipped,
    );
    executor.run_the_operations(&GeneratedHistory {
        seed: history.seed,
        starting_point: history.starting_point,
        operations: history.operations[..crash_recovery_step - 1].to_vec(),
    });
    let image = executor.image();
    for (identity, device) in &mut executor.devices {
        let (offset, rotation_not_written) =
            crate::tests::system_configuration_slot_before_the_rotation_of_the_newest_publish(
                &image, *identity,
            );
        device
            .write_at(
                DeviceOffsetInBytes(offset),
                &rotation_not_written,
                WriteDurability::Plain,
            )
            .expect("最新那一槽写回成轮换之前的那一代");
    }
    let outcome = executor.apply_operation(
        crash_recovery_step,
        crash_recovery_step - 1,
        &history.operations[crash_recovery_step - 1],
    );
    assert!(outcome.starts_with("applied:mounted"), "{outcome}");
    let mount = executor.mounts.last().expect("崩溃恢复那次挂载记下了");
    assert_eq!(mount.operation, "crash_recovery");
    assert_eq!(mount.literal_last, 1);
    assert_eq!(mount.effective_content, Some(0));
    assert!(mount.older_version_returned);
}
