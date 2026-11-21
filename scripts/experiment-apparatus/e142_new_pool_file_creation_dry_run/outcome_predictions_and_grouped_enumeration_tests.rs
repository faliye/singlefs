use super::*;

// ═════════ E142 第十七次跑第二段准备（问题单第 7 行，登记第五节 5.4）：层 0 主臂单独调用方式的机制 ═════════
// 这一组只在小规模（阳性对照、G6、G7；主臂的两个边界下标）上核这一段新加的机制本身——
// 主臂 67108885 个状态的整轮枚举不在这组测试里，它在 `main()` 里跑（`layer0_main_arm_elapsed` 那一行）。

/// 并行切片的核心：`layer0_state_at_index` 按下标直接算出的持久掩码，要与 `for_each_layer0_state`
/// 按次序访问的完全一样——这条测试要是漏了，`enumerate_layer0_grouped` 的并行切分就是在算另一套状态。
#[test]
fn layer0_state_at_index_matches_for_each_layer0_state_on_the_positive_control() {
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &sample_file(), false);
    let mut via_callback: Vec<Vec<bool>> = Vec::new();
    for_each_layer0_state(control_flow.writes.len(), &control_flow.segments, |persisted| via_callback.push(persisted));
    assert_eq!(via_callback.len() as u64, control_flow.closed_form, "closed_form 应该等于 for_each_layer0_state 实际访问的次数");
    for (global_index, expected) in via_callback.iter().enumerate() {
        let (actual, _) = layer0_state_at_index(control_flow.writes.len(), &control_flow.segments, global_index as u64);
        assert_eq!(&actual, expected, "下标 {global_index} 处两条路径给出的持久掩码必须一样");
    }
}

/// 登记第五节 5.4「并行要有一条并行与串行在阳性对照、G6、G7 上逐格相等的单测」原话。
#[test]
fn layer0_grouped_enumeration_agrees_between_serial_and_parallel_on_the_positive_control_barriers_restored_and_fua_boundary() {
    let flows: [(&str, PoolParameters, bool); 3] = [
        ("positive_control", PoolParameters::control_one_device_no_barriers(), false),
        ("g6", PoolParameters::one_device_with_barriers(), false),
        ("g7", PoolParameters::control_one_device_no_barriers(), true),
    ];
    for (label, parameters, fua_is_boundary) in flows {
        let flow = build_single_publish_flow(&parameters, &sample_file(), fua_is_boundary);
        let (serial_tally, serial_comparison, serial_grouping) = enumerate_layer0_grouped(
            1, &flow.base, &flow.writes, &flow.segments, &sample_file(), ReaderMode::Clause,
            flow.root_index, &[flow.journal_index], &flow.dependency_groups, &flow.role_names,
        );
        let (parallel_tally, parallel_comparison, parallel_grouping) = enumerate_layer0_grouped(
            4, &flow.base, &flow.writes, &flow.segments, &sample_file(), ReaderMode::Clause,
            flow.root_index, &[flow.journal_index], &flow.dependency_groups, &flow.role_names,
        );
        assert_eq!(serial_tally, parallel_tally, "{label}：串行与并行的 Layer0Tally 必须逐格相等");
        assert_eq!(serial_comparison, parallel_comparison, "{label}：串行与并行的 PredictionComparison 必须逐格相等");
        assert_eq!(serial_grouping, parallel_grouping, "{label}：串行与并行的按发布分组必须逐格相等");
    }
}

/// R13「按发布分组」：主臂那条流的三次根槽 FUA 写各自落在哪个段号——E142 第十九次跑登记 7.4 C5 的段序列
/// `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]` 里，三个大小为 1 的段（下标 2、5、9）恰好就是三次根槽。
#[test]
fn publish_group_boundaries_finds_the_three_root_segments_on_the_main_arm() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2], "登记 7.4 C5 的段序列");
    let root_segment_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| segments.iter().position(|segment| segment.contains(&index)).unwrap())
        .collect();
    assert_eq!(root_segment_indices, vec![2, 5, 9], "三次根槽各自单独一段——取号+暖机1、暖机2、主事务");
    let boundaries = publish_group_boundaries(&writes, &segments);
    assert_eq!(boundaries, vec![2, 5, 9]);
    assert_eq!(publish_group_label(&boundaries, Some(0)), "txg1");
    assert_eq!(publish_group_label(&boundaries, Some(2)), "txg1", "根自己那一段仍算在它前面那个 txg 里（根本身还没持久）");
    assert_eq!(publish_group_label(&boundaries, Some(3)), "txg2");
    assert_eq!(publish_group_label(&boundaries, Some(9)), "txg3");
    assert_eq!(publish_group_label(&boundaries, Some(10)), "after_last_root");
    assert_eq!(publish_group_label(&boundaries, None), "every_write_persisted");
}

/// R10 依赖集甲在主事务上的写法：`write_unit_to_every_device`（逐单元两盘连写）使得 `writes` 里的
/// `UnitWrite` 恰好两两相邻，对应 `output.units_by_slot` 同一个逻辑单元的两份镜像。
#[test]
fn main_transaction_dependency_groups_pairs_each_units_by_slot_entry_with_both_devices() {
    let BuiltPool { recording, output, mkfs_operation_count, .. } = built_pool();
    let (writes, _) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    let groups = main_transaction_dependency_groups(&writes, 2);
    assert_eq!(groups.len(), output.units_by_slot.len(), "每个逻辑单元一组（第七节 B19「U=12」）");
    for group in &groups {
        assert_eq!(group.len(), 2, "两盘镜像各一份");
        let devices: Vec<u32> = group.iter().map(|&index| writes[index].device.0).collect();
        assert_eq!(devices, vec![0, 1], "write_unit_to_every_device 按 parameters.devices() 的次序：先设备 0 后设备 1");
        for &index in group {
            assert_eq!(writes[index].kind, StepKind::UnitWrite);
        }
    }
}

/// 端到端的两个边界下标（不跑整轮枚举）：下标 0（只有取号自己的第一个非满子集持久，根、记录都没持久）
/// 预言与实测都该是「合法、没有文件」；最后一个下标（全部持久）预言与实测都该是「合法、读到文件、
/// 内容对」。这两点各只调一次 `recover`（经 `evaluate_state`），不是重活。
#[test]
fn layer0_main_arm_boundary_states_agree_with_the_predictor_and_the_checker() {
    let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
    let mkfs_pool = mkfs(&PoolParameters::settled_two_devices()).0.pool;
    let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
    let closed_form = closed_form_state_count(&segments);
    assert_eq!(closed_form, 16_777_240, "登记 7.4 C5 / C12");
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("根槽写");
    let all_journal_indices: Vec<usize> = writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::JournalRecord).map(|(index, _)| index).collect();
    let journal_indices = all_journal_indices[all_journal_indices.len() - 2..].to_vec();
    let dependency_groups = main_transaction_dependency_groups(&writes, 2);
    let boundaries = publish_group_boundaries(&writes, &segments);

    let (persisted_first, segment_first) = layer0_state_at_index(writes.len(), &segments, 0);
    assert_eq!(publish_group_label(&boundaries, segment_first), "txg1");
    assert_eq!(predict_single_publish(&persisted_first, root_index, &journal_indices, &dependency_groups), PredictedOutcomeClass::LegalNoFile);
    let mut tally_first = Layer0Tally::default();
    let evaluated_first = evaluate_state(&mkfs_pool, &writes, persisted_first, root_index, &sample_file(), ReaderMode::Clause, &mut tally_first);
    assert!(!evaluated_first.is_violation, "{:?}", tally_first.first_violation);
    assert!(matches!(evaluated_first.outcome, RecoveryOutcome::NoFile { .. }));

    let (persisted_last, segment_last) = layer0_state_at_index(writes.len(), &segments, closed_form - 1);
    assert_eq!(publish_group_label(&boundaries, segment_last), "every_write_persisted");
    assert_eq!(predict_single_publish(&persisted_last, root_index, &journal_indices, &dependency_groups), PredictedOutcomeClass::LegalFileRead);
    let mut tally_last = Layer0Tally::default();
    let evaluated_last = evaluate_state(&mkfs_pool, &writes, persisted_last, root_index, &sample_file(), ReaderMode::Clause, &mut tally_last);
    assert!(!evaluated_last.is_violation, "{:?}", tally_last.first_violation);
    assert!(matches!(&evaluated_last.outcome, RecoveryOutcome::FileRead { content, .. } if *content == sample_file()));
}

/// Q142.37：判决行的七个字段各自只挂一个门槛，互不影响——每个字段单独翻一次面，其余六个不动。
#[test]
fn layer0_main_arm_verdict_flags_every_threshold_independently() {
    // ⑪（登记 R19C-4、变异 M200）：states = closed_form = 登记 7.4 C5 的两态 16777240 时 layer0_states_ok 为真。
    let mut tally = Layer0Tally { states: 16_777_240, violations: 0, root_persisted_states: 4, file_read_states: 7, journal_differing_states: 3, ..Layer0Tally::default() };
    let comparison = PredictionComparison::default();
    let baseline = layer0_main_arm_verdict(&tally, 16_777_240, &comparison);
    assert!(baseline.layer0_states_ok && baseline.root_persisted_ok && baseline.file_read_ok && baseline.file_read_wrong_content_zero && baseline.main_outcome_matrix_ok);
    assert_eq!(baseline.layer0_violations, "zero");
    assert_eq!(baseline.journal_reading, "j_jia");

    let wrong_states = layer0_main_arm_verdict(&tally, 16_777_239, &comparison);
    assert!(!wrong_states.layer0_states_ok);
    assert!(wrong_states.root_persisted_ok, "只应该翻 layer0_states_ok 这一格");
    // C577 之前的整条流闭式 67108885（登记 7.2 B5），states 与它相等也不算过。
    let pre_trailing_barrier_tally = Layer0Tally { states: 67_108_885, ..tally.clone() };
    assert!(!layer0_main_arm_verdict(&pre_trailing_barrier_tally, 67_108_885, &comparison).layer0_states_ok);

    tally.violations = 1;
    let with_violation = layer0_main_arm_verdict(&tally, 16_777_240, &comparison);
    assert_eq!(with_violation.layer0_violations, "nonzero");
    assert!(with_violation.layer0_states_ok, "违例数不该影响 layer0_states_ok");
    tally.violations = 0;

    tally.root_persisted_states = 5;
    assert!(!layer0_main_arm_verdict(&tally, 16_777_240, &comparison).root_persisted_ok);
    tally.root_persisted_states = 4;

    tally.file_read_states = 6;
    assert!(!layer0_main_arm_verdict(&tally, 16_777_240, &comparison).file_read_ok);
    tally.file_read_states = 7;

    tally.file_read_wrong_content_states = 1;
    assert!(!layer0_main_arm_verdict(&tally, 16_777_240, &comparison).file_read_wrong_content_zero);
    tally.file_read_wrong_content_states = 0;

    tally.journal_differing_states = 9;
    assert_eq!(layer0_main_arm_verdict(&tally, 16_777_240, &comparison).journal_reading, "j_yi");
    tally.journal_differing_states = 4;
    assert_eq!(layer0_main_arm_verdict(&tally, 16_777_240, &comparison).journal_reading, "neither");
    tally.journal_differing_states = 3;

    let dirty_comparison = PredictionComparison { off_diagonal: 1, ..Default::default() };
    assert!(!layer0_main_arm_verdict(&tally, 16_777_240, &dirty_comparison).main_outcome_matrix_ok);
}

/// `PublishBucket`/`PublishGrouping` 的加法与合并本身：`record` 三个字段独立累加，`merge` 是两份的
/// 逐格相加，`get` 对没出现过的标签给全零默认值（不 panic）。
#[test]
fn publish_grouping_records_and_merges_independently() {
    let mut accumulated_grouping = PublishGrouping::default();
    accumulated_grouping.record("txg1", false, false);
    accumulated_grouping.record("txg1", true, true);
    accumulated_grouping.record("txg2", false, false);
    assert_eq!(accumulated_grouping.get("txg1"), PublishBucket { states: 2, violations: 1, journal_differing_states: 1 });
    assert_eq!(accumulated_grouping.get("txg2"), PublishBucket { states: 1, violations: 0, journal_differing_states: 0 });
    assert_eq!(accumulated_grouping.get("after_last_root"), PublishBucket::default(), "没出现过的标签给全零默认值");

    let mut merged_in_grouping = PublishGrouping::default();
    merged_in_grouping.record("txg1", true, false);
    merged_in_grouping.record("after_last_root", false, false);
    accumulated_grouping.merge(&merged_in_grouping);
    assert_eq!(accumulated_grouping.get("txg1"), PublishBucket { states: 3, violations: 2, journal_differing_states: 1 });
    assert_eq!(accumulated_grouping.get("after_last_root"), PublishBucket { states: 1, violations: 0, journal_differing_states: 0 });
}
