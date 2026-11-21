use super::*;
use crate::allocator::DeviceFreeMap;
use singlefs_format::UNIT_AREA_START_SLOT;

/// ckpt_cost 里分配记录树那一项取几何上整棵树（K1，D28（挂载期承诺量） 已定项 4）：
/// 1 + Σ_盘 Σ_{L < 根层级}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），单元区 [s, e) 从槽 50176 起（叶 61 罩 [49532, 50344)）。
/// - 两块单元区 16 槽的盘：只罩叶 61，根层级 1 ⇒ 1 + 2 × 1 = 3；
/// - 两块 384 槽：叶 61、62 ⇒ 1 + 2 × 2 = 5；
/// - 两块 2600 槽（[50176, 52776)）：叶 61–64 ⇒ 1 + 2 × 4 = 9；
/// - 两块 1 GiB（[50176, 65536)，每块盘 81 格、两盘 162 ≤ 169，根层级 1）：叶 61–80 ⇒ 1 + 2 × 20 = 41；
/// - 两块 4 GiB（[50176, 262144)，根层级 2）：叶 61–322 共 262 个、层级 1 两个 ⇒ 1 + 2 × 264 = 529；三块 4 GiB ⇒ 1 + 3 × 264 = 793。
///
/// 判别力：每块盘两条叶路径（K0，盘数 × 2 × (高 − 1) + 1）是 5 / 5 / 5 / 5 / 9 / 13；每块盘只数叶、不数层级 1 是 3 / 5 / 9 / 41 / 525 / 787。
#[test]
fn allocation_record_tree_term_counts_every_position_the_unit_areas_cover_below_the_root_plus_the_root(
) {
    let term_of_devices = |devices: u32, device_bytes: u64| {
        let allocator = PoolAllocator::new(
            (0..devices)
                .map(|device| DeviceFreeMap::new(DeviceIdentity(device), device_bytes))
                .collect(),
        );
        allocation_record_tree_positions_over_the_unit_areas(
            AllocationRecordTreeGeometry::of_allocator(&allocator).root_level(),
            &allocator,
        )
    };
    let unit_area_of = |slots: u64| (UNIT_AREA_START_SLOT + slots) * SLOT_BYTES;
    assert_eq!(
        [
            term_of_devices(2, unit_area_of(16)),
            term_of_devices(2, unit_area_of(384)),
            term_of_devices(2, unit_area_of(2600)),
            term_of_devices(2, 1 << 30),
            term_of_devices(2, 4 << 30),
            term_of_devices(3, 4 << 30),
        ],
        [3, 5, 9, 41, 529, 793],
        "1 + 每块盘单元区在根之下每一层罩到的位置数之和"
    );
}

/// ckpt_cost 谱系那一项：零根与只有根（内部 0 层）都是容器 2 + 根 1 = 3 块；
/// 长高一层之后 4 块、两层 5 块。
#[test]
fn the_lineage_term_of_the_checkpoint_cost_is_a_container_a_root_and_every_internal_level() {
    assert_eq!(
        [0, 1, 2].map(checkpoint_cost_of_one_lineage_container_copy_on_write),
        [MetadataBlocks(3), MetadataBlocks(4), MetadataBlocks(5)]
    );
}

/// ckpt_cost 退役那一项：容器 2 + 根 1 + 内部层数块，零根与只有根 3 块、
/// 长高一层 4 块、两层 5 块；它在中央映射树里改的条目 = 容器 1 + 根 1 + 内部层数把。
#[test]
fn the_retirement_term_of_the_checkpoint_cost_is_a_container_a_root_and_every_internal_level() {
    assert_eq!(
        [0, 1, 2].map(checkpoint_cost_of_one_retirement_container_copy_on_write),
        [MetadataBlocks(3), MetadataBlocks(4), MetadataBlocks(5)]
    );
    assert_eq!(
        [0, 1, 2].map(central_mapping_entries_of_one_retirement_container_copy_on_write),
        [2, 3, 4]
    );
}

/// 产品容量（叶 294、内部 143，按节点格式算）的中央映射树。
fn central_mapping_capacity_of_the_node_format() -> CodeTwoTreeNodeCapacity {
    MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format()
}

/// 一次空发布重写十个进映射的节点（分配记录树与记账树的节点合起来十个）时改的映射条目：各删一把旧 key、插一把新 key。
const CHANGES_OF_TEN_REWRITTEN_MAPPED_NODES: CentralMappingEntryChangesOfAnEmptyPublish =
    CentralMappingEntryChangesOfAnEmptyPublish {
        deleted_at_most: 10,
        inserted_at_most: 10,
    };

/// 中央映射树逐层按可能改的路径数与可能切出来的节点数计（D28（挂载期承诺量） 已定项 4）：
/// - 一层、条目加插的装得进一片叶：一次都切不了，重写根一个；
/// - 一层、装不下：根（叶）切一次（10 条插不满第二次，⌊9 ÷ 147⌋ = 0），长出新根：1 + 1 + 1 = 3；
/// - 两层（叶 2、根 1）：叶层 min(2, 20) + 切 min(2, 10) = 4，根层装得下 4 个孩子不切、1 个 ⇒ 5；
/// - 三层（叶 100、内部 10、根 1）：叶层 min(100, 20) + 切 10 = 30，内部层 110 个孩子装得下（143）不切、min(10, 20) = 10，根 1 ⇒ 41。
///
/// 判别力：只按树高算是 1 / 1 / 2 / 3；不算切出来的节点是 1 / 1 / 3 / 31；不算长出来的新根是 1 / 2 / 5 / 41。
#[test]
fn central_mapping_term_counts_changed_paths_and_splits_on_every_level() {
    let capacity = central_mapping_capacity_of_the_node_format();
    assert_eq!(
        (capacity.leaf_entries, capacity.internal_entries),
        (294, 143),
        "节点格式算出来的中央映射树容量"
    );
    let changes = CHANGES_OF_TEN_REWRITTEN_MAPPED_NODES;
    assert_eq!(
        [
            (vec![1], 20),
            (vec![1], 290),
            (vec![2, 1], 312),
            (vec![100, 10, 1], 20_000),
        ]
        .map(|(nodes_at_each_level, entries)| {
            central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                &nodes_at_each_level,
                entries,
                changes,
                capacity,
            )
        }),
        [1, 3, 5, 41],
        "每层 min(节点数, 删 + 插) + 切出来的，根切开再加新根"
    );
}

/// 压小容量下内部层照样切、连着长高：叶 2、内部 2（`CodeTwoTreeNodeCapacities::CappedForTests` 最小的那一档）、插 2 删 2。
/// - 一层（一片装满 2 条的叶）：叶层 min(1, 4) + 切 ≤ min(1, 2) + ⌊1 ÷ 1⌋ = 2 ⇒ 3；新根起步两个孩子、再多一个就装不下：
///   叶层之上的新根 1 + 切 ⌊(2 − 1) ÷ 1⌋ = 1 ⇒ 2，它上面再长一层：1 + 0 ⇒ 1。合计 6。
/// - 两层（叶 3、根 1，条目 6）：叶层 min(3, 4) + 切 min(3, 2) + 0 = 5；根层下一层至多 3 + 2 = 5 个节点装不下 2 个孩子，
///   切 min(1, 2) + ⌊1 ÷ 1⌋ = 2 ⇒ 1 + 2 = 3；新根 1 + 切 1 ⇒ 2，它上面再长一层：1。合计 11。
///
/// 判别力：新根只算一个、不看它还会不会切是 4 与 9；内部层一次都不按会切算，两层那一格是 6；叶层一次都不按会切算是 1 与 4。
#[test]
fn central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities() {
    let smallest_capacity = CodeTwoTreeNodeCapacity {
        leaf_entries: 2,
        internal_entries: 2,
    };
    let changes = CentralMappingEntryChangesOfAnEmptyPublish {
        deleted_at_most: 2,
        inserted_at_most: 2,
    };
    assert_eq!(
        [(vec![1], 2), (vec![3, 1], 6)].map(|(nodes_at_each_level, entries)| {
            central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                &nodes_at_each_level,
                entries,
                changes,
                smallest_capacity,
            )
        }),
        [6, 11],
        "叶层、内部层、长出来的每一层新根都按会切算"
    );
}
