//! 准入读墓碑树、退役树的 H（D3（空间分配） 已定项 2「各从自己的树根节点码 2 头现读」）时，根单元按装 `units` 时记下的下标取
//! （`singlefs_core::transaction::ContainerTreeRootUnitPositions`），不在 `units` 上逐项找（运行时决策路径，`.claude/rules/fs-design.md`
//! 「记账是事务的副产品，不是事后的遍历」：`units` 里每片容器、每个码 2 节点各一项，逐项找随容器总数涨）。
//! 这里把一版的 `units` 头一项改成同一个角色、字节全 0 的假根单元：逐项找的写法先碰到它、解不开码 2 头；按记下的下标取的写法读的还是真根。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::build_pool;
use common_deletion_in_progress::{publish_the_intent_of, unlink_to_the_end};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER};
use std::sync::Arc;

/// 这一版的拷贝，`units` 头一项换成角色是 `root_role`、字节全 0 的假根单元（真根单元在它后面，下标不变）。
fn version_with_a_decoy_root_unit_in_front(
    version: &TransactionOutput,
    root_role: TransactionUnit,
) -> TransactionOutput {
    let real_root_position = version
        .units
        .iter()
        .position(|unit| unit.identity == root_role)
        .expect("这一版有这棵树的根单元");
    assert!(real_root_position > 0, "前提：真根单元不在 units 头一项");
    let mut tampered = version.clone();
    let decoy = &mut tampered.units[0];
    decoy.identity = root_role;
    decoy.bytes = Arc::from(vec![0; decoy.bytes.len()]);
    tampered
}

#[test]
fn the_tombstone_tree_height_is_read_from_the_root_unit_at_its_recorded_position() {
    let mut pool = build_pool("tombstone-root-unit-at-its-recorded-position");
    let intent = publish_the_intent_of(&mut pool, FIRST_INODE_NUMBER);
    assert_eq!(
        intent.tombstone_containers.len(),
        1,
        "墓碑树一片容器、根在层级 1"
    );
    let tampered = version_with_a_decoy_root_unit_in_front(&intent, TransactionUnit::TombstoneRoot);
    assert_eq!(
        tampered
            .tombstone_root_unit()
            .map(|unit| unit.bytes.clone()),
        Some(intent.unit(TransactionUnit::TombstoneRoot).bytes.clone()),
        "按记下的下标取到的是真根单元"
    );
    assert_eq!(
        tampered.tombstone_tree_internal_levels_read_from_its_root_node_header(),
        0,
        "根在层级 1：H = 0，读的是真根单元的码 2 头"
    );
}

#[test]
fn the_retirement_tree_height_is_read_from_the_root_unit_at_its_recorded_position() {
    let mut pool = build_pool("retirement-root-unit-at-its-recorded-position");
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    let emptied = pool.output.clone();
    assert_eq!(
        emptied.retirement_containers.len(),
        1,
        "退役树一片容器、根在层级 1"
    );
    let tampered =
        version_with_a_decoy_root_unit_in_front(&emptied, TransactionUnit::RetirementRoot);
    assert_eq!(
        tampered
            .retirement_root_unit()
            .map(|unit| unit.bytes.clone()),
        Some(emptied.unit(TransactionUnit::RetirementRoot).bytes.clone()),
        "按记下的下标取到的是真根单元"
    );
    assert_eq!(
        tampered.retirement_tree_internal_levels_read_from_its_root_node_header(),
        0,
        "根在层级 1：H = 0，读的是真根单元的码 2 头"
    );
}
