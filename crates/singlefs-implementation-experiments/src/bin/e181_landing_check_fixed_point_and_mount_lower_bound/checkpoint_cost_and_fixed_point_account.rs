//! 普通映射的改动、本地算的检查点代价与 core 的代价，和不动点的账。

use super::*;

/// 一次发布在中央映射树里多删多插的普通分配条目数（F9 的 P1 在 ckpt_cost 的删插数上加的两项）：
/// 删 = 上一版有、这一版没有的普通分配角色的映射 key（这次换下的数据单元、被重写的 extent 与 inode 单元），
/// 插 = 这一版有、上一版没有的（这次新写的同几类单元）。按两版 `mapped_units` 里普通分配角色的 key 集合作差，
/// 不读发布内部，所以与「走固定点之前就定下了的普通分配那一半」是同一个数（普通分配的角色在固定点之前就定，
/// `transaction::prepare_the_version_publish`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct OrdinaryMappingChanges {
    pub(super) deleted: u64,
    pub(super) inserted: u64,
}

fn ordinary_mapping_keys(version: &TransactionOutput) -> BTreeSet<&[u8]> {
    version
        .mapped_units
        .iter()
        .filter(|(role, _key)| {
            space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation
        })
        .map(|(_role, key)| key.as_slice())
        .collect()
}

pub(super) fn ordinary_mapping_changes(
    previous: &TransactionOutput,
    output: &TransactionOutput,
) -> OrdinaryMappingChanges {
    let keys_before = ordinary_mapping_keys(previous);
    let keys_after = ordinary_mapping_keys(output);
    OrdinaryMappingChanges {
        deleted: u64::try_from(keys_before.difference(&keys_after).count()).expect("条数"),
        inserted: u64::try_from(keys_after.difference(&keys_before).count()).expect("条数"),
    }
}

/// 本地重算的 ckpt_cost（D28（挂载期承诺量） 已定项 4，登记 5.1 节）：分配记录树按 K1、中央映射树按逐层路径式子、
/// 记账树与谱系、退役两项取核心层同名公开函数（读了的共用项，B3 逐版对拍它们），加树表 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LocalCheckpointCost {
    allocation_record_tree_nodes: u64,
    central_mapping_tree_nodes: u64,
    accounting_tree_nodes: u64,
    lineage_container_copy_on_write: u64,
    retirement_container_copy_on_write: u64,
    pub(super) total: u64,
}

/// 分配记录树根之下每个层级一个位置的槽数：812 × 169^层级。
fn allocation_record_tree_span_in_slots_at_level(level: u8) -> u64 {
    let mut span = ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL;
    for _climbed in 0..level {
        span *= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL;
    }
    span
}

/// K1（D28（挂载期承诺量） 已定项 4）：1 + Σ_盘 Σ_{层级 0 … 根层级 − 1} 这块盘单元区 [起点, 起点 + 槽数) 在这一层罩到的位置数。
pub(super) fn local_allocation_record_tree_nodes(root_level: u8, unit_areas: &[(u64, u64)]) -> u64 {
    let mut nodes = 1;
    for (first_slot, slot_count) in unit_areas {
        let last_slot = first_slot + slot_count - 1;
        for level in 0..root_level {
            let span = allocation_record_tree_span_in_slots_at_level(level);
            nodes += last_slot / span - first_slot / span + 1;
        }
    }
    nodes
}

/// 中央映射树每层有几个节点（自叶到根）。
fn local_nodes_at_each_level(version: &TransactionOutput) -> Vec<u64> {
    let height =
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let mut counts = vec![0_u64; usize::try_from(height).expect("树高装得进 usize")];
    for node in version.central_mapping_tree.shape.nodes() {
        counts[usize::from(node.position.level)] += 1;
    }
    assert!(
        counts.iter().all(|count| *count > 0),
        "中央映射树每一层至少一个节点：{counts:?}"
    );
    counts
}

/// 一层上至多切几次：原有的节点第一次切至少要加 1 条，切出来的新节点再切一次至少要加 `additions_per_split_of_a_fresh_node` 条。
pub(super) fn local_splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits = pre_existing_nodes.min(additions);
    first_splits + (additions - first_splits) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次发布至多重写的节点数：每层 min(这一层的节点数, 删 + 插) 加这一层切出来的，根切开就长高。
pub(super) fn local_central_mapping_tree_nodes_rewritten_at_most(
    nodes_at_each_level: &[u64],
    entries_before: u64,
    deleted: u64,
    inserted: u64,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("容量");
    let internal_entries = u64::try_from(capacity.internal_entries).expect("容量");
    let leaf_additions = leaf_entries.div_ceil(2);
    let internal_additions = internal_entries.div_ceil(2);
    let changed_paths = deleted + inserted;
    let mut rewritten = 0;
    let mut splits_below = 0;
    for (level, nodes_here) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_here = if level == 0 {
            if entries_before + inserted <= leaf_entries {
                0
            } else {
                local_splits_at_one_level_at_most(nodes_here, inserted, leaf_additions)
            }
        } else if nodes_at_each_level[level - 1] + splits_below <= internal_entries {
            0
        } else {
            local_splits_at_one_level_at_most(nodes_here, splits_below, internal_additions)
        };
        rewritten += nodes_here.min(changed_paths) + splits_here;
        splits_below = splits_here;
    }
    while splits_below > 0 {
        let children_of_the_new_root = splits_below + 1;
        let splits_of_the_new_level = if children_of_the_new_root <= internal_entries {
            0
        } else {
            (splits_below - 1) / internal_additions
        };
        rewritten += 1 + splits_of_the_new_level;
        splits_below = splits_of_the_new_level;
    }
    rewritten
}

pub(super) fn local_checkpoint_cost(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    capacities: CodeTwoTreeNodeCapacities,
    extra: OrdinaryMappingChanges,
) -> LocalCheckpointCost {
    let root_level = u8::try_from(
        previous
            .position_addressed_tree_heights_read_from_the_root_node_headers()
            .allocation_record_tree
            - 1,
    )
    .expect("层级装得进 u8");
    let unit_areas: Vec<(u64, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.unit_area_start().slot().0,
                device_map.unit_area_slots(),
            )
        })
        .collect();
    let allocation_record_tree_nodes = local_allocation_record_tree_nodes(root_level, &unit_areas);
    let accounting_nodes_before =
        u64::try_from(previous.accounting_tree.node_count()).expect("节点数");
    let accounting_nodes_after = accounting_tree_nodes_after_the_publish_built_on(
        previous,
        capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
    );
    let lineage_levels = previous.lineage_tree_internal_levels();
    let retirement_levels = previous.retirement_tree_internal_levels();
    let lineage_entries =
        central_mapping_entries_of_one_lineage_container_copy_on_write(lineage_levels);
    let retirement_entries =
        central_mapping_entries_of_one_retirement_container_copy_on_write(retirement_levels);
    let deleted = allocation_record_tree_nodes
        + accounting_nodes_before
        + lineage_entries
        + retirement_entries
        + extra.deleted;
    let inserted = allocation_record_tree_nodes
        + accounting_nodes_after
        + lineage_entries
        + retirement_entries
        + extra.inserted;
    let entries_before = u64::try_from(
        previous.mapped_units.len() + previous.mapping_entries_kept_for_the_snapshots.len(),
    )
    .expect("条数");
    let central_mapping_tree_nodes = local_central_mapping_tree_nodes_rewritten_at_most(
        &local_nodes_at_each_level(previous),
        entries_before,
        deleted,
        inserted,
        capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
    );
    let lineage_container_copy_on_write =
        checkpoint_cost_of_one_lineage_container_copy_on_write(lineage_levels).0;
    let retirement_container_copy_on_write =
        checkpoint_cost_of_one_retirement_container_copy_on_write(retirement_levels).0;
    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
    LocalCheckpointCost {
        allocation_record_tree_nodes,
        central_mapping_tree_nodes,
        accounting_tree_nodes: accounting_nodes_after,
        lineage_container_copy_on_write,
        retirement_container_copy_on_write,
        total: allocation_record_tree_nodes
            + central_mapping_tree_nodes
            + accounting_nodes_after
            + lineage_container_copy_on_write
            + retirement_container_copy_on_write
            + TREE_TABLE_UNITS_PER_PUBLISH,
    }
}

/// 核心层的 ckpt_cost（落得下那一判今天拿它顶固定点）。
pub(super) fn core_checkpoint_cost(
    previous: &TransactionOutput,
    allocator: &PoolAllocator,
    capacities: CodeTwoTreeNodeCapacities,
) -> u64 {
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        Some(previous),
        allocator,
        capacities,
    )
    .0
}

/// 这次发布在每块盘上新占的固定点槽数，两条独立的路各算一遍：
/// 路径一「角色」：这次真写出的角色里不是普通分配的那几个，各自跨度相加（每个单元落每一块盘，所以逐盘相同）；
/// 路径二「记录」（登记第一节「读法写死」）：这一版分配记录里分配代等于这次 txg、没释放的记录，逐盘加跨度，再减普通分配角色的槽数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FixedPointAccount {
    pub(super) by_roles: u64,
    pub(super) by_records_per_device: Vec<(DeviceIdentity, u64)>,
    pub(super) ordinary_slots: u64,
    pub(super) instance_switch_slots: u64,
}

pub(super) fn fixed_point_account(
    output: &TransactionOutput,
    devices: &[DeviceIdentity],
) -> FixedPointAccount {
    let mut by_roles = 0;
    let mut ordinary_slots = 0;
    let mut instance_switch_slots = 0;
    for role in &output.rewritten {
        match space_budget_of_role(*role) {
            SpaceBudgetOfARole::CheckpointReservePool => by_roles += role.span_slots(),
            SpaceBudgetOfARole::OrdinaryAllocation => ordinary_slots += role.span_slots(),
            SpaceBudgetOfARole::InstanceSwitchReserve => instance_switch_slots += role.span_slots(),
        }
    }
    let txg = output.root.checkpoint_txg;
    let by_records_per_device = devices
        .iter()
        .map(|device| {
            let allocated_in_this_publish: u64 = output
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == *device && record.generation == txg && !record.is_released
                })
                .map(|record| u64::from(record.span_slots))
                .sum();
            (
                *device,
                allocated_in_this_publish - ordinary_slots - instance_switch_slots,
            )
        })
        .collect();
    FixedPointAccount {
        by_roles,
        by_records_per_device,
        ordinary_slots,
        instance_switch_slots,
    }
}
