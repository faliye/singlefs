use super::*;

/// 一条 86 字节的节点指针，只填出生 txg（I-5.7 那几条用例只读它与全不全零）。
fn node_pointer_born_at(birth_txg: u64) -> [u8; 86] {
    let mut pointer = [0u8; 86];
    pointer[34] = 3;
    pointer[42..50].copy_from_slice(&birth_txg.to_le_bytes());
    pointer
}

/// I-5.7 在一版上的判定。
fn allocation_record_tree_root_birth_verdict_of(
    checkpoint_txg: u64,
    pointer: &[u8],
    publish_names_some_unit: bool,
) -> InvariantVerdict {
    let mut judgements = Judgements::default();
    judge_allocation_record_tree_roots_born_with_the_versions_applied_only_by_records(
        &[VersionToJudgeTheAllocationRecordTreeRootBirthOf {
            name: format!("由记录施加出来的那一版（实例 3、txg {checkpoint_txg}）"),
            checkpoint_txg,
            allocation_record_tree_root_held_outside_the_tree_table: pointer,
            publish_names_some_unit,
        }],
        &mut judgements,
    );
    judgements
            .into_report()
            .into_iter()
            .find(|(invariant, _)| {
                *invariant
                    == ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION
            })
            .map(|(_, verdict)| verdict)
            .expect("I-5.7 在清单里")
}

/// C706 的镜像：第 3 次挂载写行那次发布（txg 9，树表 0 条，点名项非空）记录已落、根槽没落；
/// 写者重建分配记录树，新根段里那条指针出生在 9 ⇒ 成立。把它换成第 2 次写行那一版（txg 5）的合法旧树 ⇒ I-5.7 红。
/// 点名项为空（零单元发布照抄上一版的树）不判；指针全零（带树的一版）不判。
#[test]
fn the_allocation_record_tree_root_of_a_version_applied_only_by_records_must_be_born_with_it() {
    assert_eq!(
        allocation_record_tree_root_birth_verdict_of(9, &node_pointer_born_at(9), true),
        InvariantVerdict::Holds,
        "写者重建的树，指针出生在这一版"
    );
    assert!(
        matches!(
            allocation_record_tree_root_birth_verdict_of(9, &node_pointer_born_at(5), true),
            InvariantVerdict::Violated(_)
        ),
        "换成 txg 5 那一版的旧树：I-5.7 红"
    );
    assert_eq!(
        allocation_record_tree_root_birth_verdict_of(9, &node_pointer_born_at(5), false),
        InvariantVerdict::NotApplicable(
            NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE
        ),
        "点名项为空：零单元发布照抄上一版的树，不判"
    );
    assert_eq!(
        allocation_record_tree_root_birth_verdict_of(9, &[0u8; 86], true),
        InvariantVerdict::NotApplicable(
            NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE
        ),
        "指针全零：带树的一版，分配记录树在树表里，不判"
    );
}

/// I-5.5 的落点不进开放段（D3（空间分配） 已定项 10 ②）：U 起的那一段 64 槽全被更早的版本占着，下一段是这一版的开放段
/// （树表单元在里面），落点越过开放段、落在再下一段的起点。
#[test]
fn the_first_data_unit_slot_skips_the_open_cluster_segment_of_the_version() {
    let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
    let version_checkpoint_txg = 7;
    let allocation_records = [
        allocation_record_on_device_zero(unit_area_start, CLUSTER_SEGMENT_SLOTS, 5, false),
        allocation_record_on_device_zero(
            unit_area_start + CLUSTER_SEGMENT_SLOTS + 12,
            1,
            version_checkpoint_txg,
            false,
        ),
    ];
    assert_eq!(
        first_data_unit_slot_the_allocation_records_leave_on_a_device(
            &allocation_records,
            0,
            version_checkpoint_txg,
            unit_area_start,
            Some(OpenClusterSegment {
                first_slot: unit_area_start + CLUSTER_SEGMENT_SLOTS,
            }),
        ),
        unit_area_start + 2 * CLUSTER_SEGMENT_SLOTS,
        "U 那一段占满、下一段是开放段：落点是再下一段的起点"
    );
}

/// 已释放的记录一律当占着（回收门槛复算不出的那个近似）：U、U + 1 与 U + 2 两条记录是这一版自己释放的（释放代等于这一版的 txg），
/// 落点不取 U，取 U + 4。
#[test]
fn released_allocation_record_keeps_its_slots_out_of_the_first_data_unit_slot() {
    let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
    let version_checkpoint_txg = 29;
    let allocation_records = [
        allocation_record_on_device_zero(unit_area_start, 2, version_checkpoint_txg, true),
        allocation_record_on_device_zero(unit_area_start + 2, 1, version_checkpoint_txg, true),
    ];
    assert_eq!(
        first_data_unit_slot_the_allocation_records_leave_on_a_device(
            &allocation_records,
            0,
            version_checkpoint_txg,
            unit_area_start,
            None,
        ),
        unit_area_start + 4,
        "U、U + 1 与 U + 2 已释放（当占着）：落点是 U + 4"
    );
}
