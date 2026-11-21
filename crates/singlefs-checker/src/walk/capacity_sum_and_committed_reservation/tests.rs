use super::*;

/// C70 的镜像(a)：空闲 + 已分配 == 容量（I-5.2 那一和对得上）而不可回收写成 16384 ⇒ I-3.20 红两句；
/// 不可回收那一行缺 ⇒ 红；三项都对 ⇒ 成立。
#[test]
fn free_allocated_and_unreclaimable_must_sum_to_the_capacity_with_the_unreclaimable_row_at_zero() {
    let invariant = FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM;
    let violations_of = |unreclaimable: Option<u64>| {
        let mut judgements = Judgements::default();
        judge_free_allocated_and_unreclaimable_of_a_device_sum_to_the_capacity(
            0,
            Some(1_000 * SLOT_BYTES),
            Some(24 * SLOT_BYTES),
            unreclaimable,
            Some(1_024 * SLOT_BYTES),
            &mut judgements,
        );
        judgements.violation_count(invariant)
    };
    assert_eq!(violations_of(Some(0)), 0, "空闲 + 已分配 + 0 == 容量");
    assert_eq!(
            violations_of(Some(SLOT_BYTES)),
            2,
            "空闲 + 已分配 == 容量（I-5.2 那一和对得上）而不可回收写成 16384：非 zoned 要 0、三项之和超出容量，两句都红"
        );
    assert_eq!(violations_of(None), 2, "不可回收那一行缺：两句都红");
}

/// C70 的镜像(b)：两块盘、两棵容器树都只有根、一条 nlink 1 的记录 ⇒ 闭式
/// 2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 + 35) = 196 760；标签 6 等于它成立，多 1 红；树表 0 条取 0（树表非空而没有 `deleted_inodes` 条目那一形报部分没判，钉在
/// `crates/singlefs-verification/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
#[test]
fn the_committed_reservation_must_equal_its_closed_form() {
    let invariant = FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM;
    let basis = CommittedReservationBasis::VersionWithADeletedInodesTreeTableEntry {
        tombstone_tree_internal_levels: 0,
        retirement_tree_internal_levels: 0,
        live_objects: 1,
        inode_records: 1,
    };
    assert_eq!(
        committed_reservation_of_the_closed_form(basis, 2),
        Some(196_760),
        "2 × (6 × 16384 + 41 + 35)"
    );
    let violations_of = |recorded: Option<u64>, basis_of_the_version: CommittedReservationBasis| {
        let mut judgements = Judgements::default();
        judge_the_committed_reservation_against_its_closed_form(
            recorded,
            Ok(basis_of_the_version),
            2,
            7,
            &mut judgements,
        );
        judgements.violation_count(invariant)
    };
    assert_eq!(violations_of(Some(196_760), basis), 0, "标签 6 等于闭式");
    assert_eq!(
        violations_of(Some(196_761), basis),
        1,
        "标签 6 多 1：I-3.20 红"
    );
    assert_eq!(
        violations_of(
            Some(0),
            CommittedReservationBasis::VersionWithAnEmptyTreeTable
        ),
        0,
        "树表 0 条：取 0"
    );
}
