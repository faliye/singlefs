use super::*;

/// `system_configuration_slot_positions`：槽 0 恒偏移 0，槽 1 偏移 = 槽距；两块盘各自独立。
#[test]
fn system_configuration_slot_positions_uses_slot_spacing_for_the_second_slot_on_each_device() {
    assert_eq!(
        system_configuration_slot_positions(4096),
        [(0, 0, 0), (0, 1, 4096), (1, 0, 0), (1, 1, 4096)]
    );
    assert_eq!(
        system_configuration_slot_positions(512),
        [(0, 0, 0), (0, 1, 512), (1, 0, 0), (1, 1, 512)]
    );
}

/// `evidence_items_at`：`include_system_configuration_slots` 关着时一条
/// `SystemConfigurationSlot` 证据项都不出现（既有六份副本用的搜索不受影响）；打开时两块盘各自
/// 两个自证过的槽都进来，标签按 (设备, 槽号) 区分。
#[test]
fn evidence_items_at_includes_system_configuration_slots_only_when_asked() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let without_configuration = evidence_items_at(&node.pool, &geometry_view, false);
    assert!(
        without_configuration
            .iter()
            .all(|item| item.kind != EvidenceKind::SystemConfigurationSlot),
        "开关关着时不该有系统配置槽这一类证据项"
    );
    let with_configuration = evidence_items_at(&node.pool, &geometry_view, true);
    let configuration_labels: BTreeSet<String> = with_configuration
        .iter()
        .filter(|item| item.kind == EvidenceKind::SystemConfigurationSlot)
        .map(|item| item.label.clone())
        .collect();
    assert_eq!(
        configuration_labels,
        BTreeSet::from([
            "configuration(device=0,slot=0)".to_string(),
            "configuration(device=0,slot=1)".to_string(),
            "configuration(device=1,slot=0)".to_string(),
            "configuration(device=1,slot=1)".to_string(),
        ]),
        "mkfs 加暖机加首个文件加一次覆盖写之后，两块盘的两个槽都该已经轮换过、都自证得过"
    );
}

/// `candidate_fault_sets_of_weight`：根槽与系统配置槽都是权重 1，合并成一批喂给这个函数之后，
/// 权重 2 的子集要既有「两个根槽」「两个配置槽」，也要有「一个根槽 + 一个配置槽」这种跨类组合，
/// 不能只按证据项个数枚举、漏掉跨类的那一种。
#[test]
fn candidate_fault_sets_of_weight_mixes_root_and_configuration_slots_by_weight() {
    let weight_one_items = vec![
        evidence_item(EvidenceKind::RootRingSlot, "root_a", 1),
        evidence_item(EvidenceKind::SystemConfigurationSlot, "configuration_a", 1),
    ];
    let journal_record_items = Vec::new();
    let candidates = candidate_fault_sets_of_weight(&weight_one_items, &journal_record_items, 2);
    let labels: BTreeSet<Vec<String>> = candidates
        .iter()
        .map(|combination| combination.iter().map(|item| item.label.clone()).collect())
        .collect();
    assert_eq!(
        labels,
        BTreeSet::from([vec!["root_a".to_string(), "configuration_a".to_string()]]),
        "两个权重 1 的证据项只有一种取两个的组合：跨类那一对"
    );
}

fn evidence_item(kind: EvidenceKind, label: &str, target_count: usize) -> EvidenceItem {
    EvidenceItem {
        kind,
        label: label.to_string(),
        targets: (0..target_count)
            .map(|index| {
                (
                    DeviceIdentity(0),
                    DeviceOffsetInBytes(u64::try_from(index).expect("测试里的下标很小")),
                )
            })
            .collect(),
    }
}

/// Q2-1 的权重子集枚举：根槽权重 1、记录权重 2；权重 2 的子集应当既有「两个根槽」也有
/// 「一条记录」，不能只按证据项个数枚举（那样会漏掉「一条记录」这一类、多出「两条记录」这一类）。
#[test]
fn candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight() {
    let root_slot_items = vec![
        evidence_item(EvidenceKind::RootRingSlot, "root_a", 1),
        evidence_item(EvidenceKind::RootRingSlot, "root_b", 1),
    ];
    let journal_record_items = vec![evidence_item(EvidenceKind::JournalRecord, "record_a", 2)];
    let candidates = candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 2);
    let labels: BTreeSet<Vec<String>> = candidates
        .iter()
        .map(|combination| combination.iter().map(|item| item.label.clone()).collect())
        .collect();
    assert_eq!(
        labels,
        BTreeSet::from([
            vec!["root_a".to_string(), "root_b".to_string()],
            vec!["record_a".to_string()],
        ]),
        "权重 2 只能是两个根槽，或一条记录（权重 2），不能是别的组合"
    );
}

/// 权重 0 只有空集合这一种，权重超过全部证据项权重之和时枚举不出任何子集。
#[test]
fn candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set() {
    let root_slot_items = vec![evidence_item(EvidenceKind::RootRingSlot, "root_a", 1)];
    let journal_record_items = Vec::new();
    assert_eq!(
        candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 0),
        vec![Vec::new()]
    );
    assert_eq!(
        candidate_fault_sets_of_weight(&root_slot_items, &journal_record_items, 5),
        Vec::<Vec<EvidenceItem>>::new()
    );
}

fn timeline_with_one_root() -> Vec<TimelineRoot> {
    vec![(1, 3)]
}

/// P331：所选根不在时间线上记「倒挂」，不看内容对不对。
#[test]
fn judge_recovery_outcome_flags_a_root_off_the_timeline_as_rootback_before_checking_content() {
    let outcome = RecoveryOutcome::FileRead {
        root: (InstanceGeneration(9), CheckpointTxg(9)),
        content: vec![1, 2, 3],
    };
    let (passed, detail) = judge_recovery_outcome(
        "check",
        &outcome,
        &timeline_with_one_root(),
        &Some(vec![1, 2, 3]),
    );
    assert!(!passed, "所选根 (9,9) 不在时间线 [(1,3)] 上，必须判失败");
    assert!(
        detail.contains("倒挂"),
        "失败理由要写明是倒挂，实得：{detail}"
    );
}

/// P331：所选根在时间线上、但读回内容对不上时间线末端记「读不回」。
#[test]
fn judge_recovery_outcome_flags_wrong_content_as_unreadable() {
    let outcome = RecoveryOutcome::FileRead {
        root: (InstanceGeneration(1), CheckpointTxg(3)),
        content: vec![9, 9, 9],
    };
    let (passed, detail) = judge_recovery_outcome(
        "check",
        &outcome,
        &timeline_with_one_root(),
        &Some(vec![1, 2, 3]),
    );
    assert!(!passed, "内容对不上时间线末端的期望内容，必须判失败");
    assert!(
        detail.contains("读不回"),
        "失败理由要写明是读不回，实得：{detail}"
    );
}

/// P331：所选根在时间线上、内容也对得上，判通过。
#[test]
fn judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches() {
    let outcome = RecoveryOutcome::FileRead {
        root: (InstanceGeneration(1), CheckpointTxg(3)),
        content: vec![1, 2, 3],
    };
    let (passed, _) = judge_recovery_outcome(
        "check",
        &outcome,
        &timeline_with_one_root(),
        &Some(vec![1, 2, 3]),
    );
    assert!(passed, "所选根在时间线上、内容也对得上，应当判通过");
}

/// P331：没有文件的一版，期望内容是 `None`，`NoFile` 判通过。
#[test]
fn judge_recovery_outcome_passes_on_no_file_when_no_file_is_expected() {
    let outcome = RecoveryOutcome::NoFile {
        root: (InstanceGeneration(1), CheckpointTxg(3)),
    };
    let (passed, _) = judge_recovery_outcome("check", &outcome, &timeline_with_one_root(), &None);
    assert!(passed, "没有文件的一版，期望内容也是 None，应当判通过");
}

/// PC2 排查出的根因：`MultiOffsetReadFailingBlockDevice` 一次记住多个精确偏移，
/// 每一个都要让读失败——不能只认第一个。
#[test]
fn multi_offset_read_failing_block_device_fails_every_named_offset() {
    let inner = SparseBlockDevice::new(1 << 20, PhysicalBlockSizeInBytes(512));
    let device = MultiOffsetReadFailingBlockDevice {
        inner,
        failing_offsets: BTreeSet::from([512u64, 4096u64, 8192u64]),
    };
    let mut buffer = [0u8; 512];
    for offset in [512u64, 4096, 8192] {
        assert!(
            device
                .read_at(DeviceOffsetInBytes(offset), &mut buffer)
                .is_err(),
            "点名的偏移 {offset} 应当每一个都读失败，不能只有第一个"
        );
    }
}

/// 对照：没被点名的偏移原样读得通（读失败不是把整块盘弄坏）。
#[test]
fn multi_offset_read_failing_block_device_passes_through_offsets_not_named() {
    let inner = SparseBlockDevice::new(1 << 20, PhysicalBlockSizeInBytes(512));
    let device = MultiOffsetReadFailingBlockDevice {
        inner,
        failing_offsets: BTreeSet::from([512u64]),
    };
    let mut buffer = [0u8; 512];
    assert!(
        device
            .read_at(DeviceOffsetInBytes(16384), &mut buffer)
            .is_ok(),
        "没被点名的偏移不应该被这份包装挡下来"
    );
}

/// （`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：权重上限机制——
/// 显式传 `Some(0)` 时只穷举权重 0 那一档就停（不管这一档打不打中），`stopped_by_weight_ceiling`
/// 如实反映「还有没搜到的档」；`full_space_subset_count` 恒等于完整证据空间的子集数，不随上限变，
/// 且不管停在哪一档，`subsets_tried` 都不超过它——这三条合起来就是「完整空间多大、穷举到权重几、
/// 没有截断」要报的东西。
#[test]
fn search_minimum_weight_that_triggers_rootback_honors_an_explicit_weight_ceiling() {
    let geometry = &GEOMETRY_PRIMARY;
    let parameters = parameters_for(geometry);
    let node = bootstrap(geometry, 0).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");

    let full = search_minimum_weight_that_triggers_rootback(
        &node,
        &parameters,
        &geometry_view,
        1,
        None,
        false,
    );
    assert!(
        full.full_space_subset_count > 1,
        "这段历史至少有一个证据项，完整空间子集数要大于 1"
    );
    assert!(
        full.subsets_tried <= full.full_space_subset_count,
        "枚举过的子集数不能超过完整空间"
    );

    let capped = search_minimum_weight_that_triggers_rootback(
        &node,
        &parameters,
        &geometry_view,
        1,
        Some(0),
        false,
    );
    assert_eq!(
        capped.full_space_subset_count, full.full_space_subset_count,
        "权重上限不改变完整空间的大小，只改变搜到哪一档就停"
    );
    assert_eq!(
        capped.highest_weight_examined, 0,
        "传 Some(0) 时只穷举权重 0 那一档"
    );
    if capped.weight_at_first_hit.is_none() {
        assert!(
            capped.stopped_by_weight_ceiling,
            "权重 0 没打中、且完整空间的最高权重大于 0 时，必须标『因权重上限而停』"
        );
    }
    assert!(
        capped.subsets_tried <= full.subsets_tried,
        "权重上限更紧时枚举过的子集数不该比不设上限时更多"
    );
}
