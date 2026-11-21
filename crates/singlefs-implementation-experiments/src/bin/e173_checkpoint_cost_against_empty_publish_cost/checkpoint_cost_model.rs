//! 本地照 D28（挂载期承诺量） 已定项 4 重算的 κ 与新占固定点槽的两条路：分配记录树与中央映射树的位置数、改写的结点数（停机条款 S2、S3）。
use super::*;

/// 分配记录树根之下层级 `level` 上一个位置罩的槽数：812 × 169^层级（D28（挂载期承诺量） 已定项 4）。
fn allocation_record_tree_span_in_slots_at_level(level: u8) -> u64 {
    let mut span = ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL;
    for _climbed in 0..level {
        span *= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL;
    }
    span
}

/// K1（D28（挂载期承诺量） 已定项 4「分配记录树这一项按 K1 算」，登记第七节 A7）：
/// 1 + Σ_盘 Σ_{L = 0 … 根层级 − 1}（⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），单元区按 [s, e) 给。
pub(super) fn local_allocation_record_tree_positions(
    root_level: u8,
    unit_areas: &[(u64, u64)],
) -> u64 {
    let mut positions = 1;
    for (first_slot, end_slot) in unit_areas {
        assert!(
            end_slot > first_slot,
            "单元区至少一槽：[{first_slot}, {end_slot})"
        );
        for level in 0..root_level {
            let span = allocation_record_tree_span_in_slots_at_level(level);
            positions += (end_slot - 1) / span - first_slot / span + 1;
        }
    }
    positions
}

/// 树表 0 条、写过行的那一版上分配记录树的根层级取池几何的（D28（挂载期承诺量） 已定项 4 子条「树表 0 条、写过行的那一版取池几何的根层级」）：
/// 最小的层级 L ≥ 1，使根下那一层（层级 L − 1）每块盘按整盘槽数罩到的格数之和装得进一个内部节点（扇出 169，D8（核心索引结构） 已定项 14）。
pub(super) fn local_root_level_of_the_pool_geometry(device_slot_counts: &[u64]) -> u8 {
    (1..=u8::MAX)
        .find(|candidate_root_level| {
            let child_span =
                allocation_record_tree_span_in_slots_at_level(candidate_root_level - 1);
            let cells: u64 = device_slot_counts
                .iter()
                .map(|slots| slots.div_ceil(child_span))
                .sum();
            cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL
        })
        .expect("两块盘、槽数有上界 2^48：层级至多 6 就装得下每块盘各一格")
}

/// 一层上至多切几次（D8（核心索引结构） 已定项 11 ②：装不下从中间切）：原有节点第一次切至少要加 1 条，
/// 切出来的新节点再切一次至少要加 `additions_per_split_of_a_fresh_node` 条。
fn local_splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits = pre_existing_nodes.min(additions);
    first_splits + (additions - first_splits) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次空发布至多重写的节点数（D28（挂载期承诺量） 已定项 4「中央映射树逐层按可能改的路径数与可能切出来的节点数」）：
/// 每层 min(这一层的节点数, 删 + 插) 加这一层切出来的；根切开就长高一层，新根一个、装不下再切。
fn local_central_mapping_tree_nodes_rewritten_at_most(
    nodes_at_each_level: &[u64],
    entries_before: u64,
    deleted_at_most: u64,
    inserted_at_most: u64,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("叶容量装得进 u64");
    let internal_entries = u64::try_from(capacity.internal_entries).expect("内部容量装得进 u64");
    let leaf_additions_per_split = leaf_entries.div_ceil(2);
    let internal_additions_per_split = internal_entries.div_ceil(2);
    let changed_paths = deleted_at_most + inserted_at_most;
    let mut rewritten = 0;
    let mut splits_at_the_level_below = 0;
    for (level, nodes_here) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_here = if level == 0 {
            if entries_before + inserted_at_most <= leaf_entries {
                0
            } else {
                local_splits_at_one_level_at_most(
                    nodes_here,
                    inserted_at_most,
                    leaf_additions_per_split,
                )
            }
        } else if nodes_at_each_level[level - 1] + splits_at_the_level_below <= internal_entries {
            0
        } else {
            local_splits_at_one_level_at_most(
                nodes_here,
                splits_at_the_level_below,
                internal_additions_per_split,
            )
        };
        rewritten += nodes_here.min(changed_paths) + splits_here;
        splits_at_the_level_below = splits_here;
    }
    while splits_at_the_level_below > 0 {
        let splits_of_the_new_level = if splits_at_the_level_below < internal_entries {
            0
        } else {
            (splits_at_the_level_below - 1) / internal_additions_per_split
        };
        rewritten += 1 + splits_of_the_new_level;
        splits_at_the_level_below = splits_of_the_new_level;
    }
    rewritten
}

/// 中央映射树每层有几个节点（自叶到根），按这一版根节点头里读出的高。
fn central_mapping_tree_nodes_at_each_level(version: &TransactionOutput) -> Vec<u64> {
    let height =
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let mut counts = vec![0_u64; usize::try_from(height).expect("树高装得进 usize")];
    for node in version.central_mapping_tree.shape.nodes() {
        counts[usize::from(node.position.level)] += 1;
    }
    counts
}

impl LocalCheckpointCost {
    pub(super) fn total(self) -> u64 {
        self.allocation_record_tree
            + self.central_mapping_tree
            + self.accounting_tree
            + self.lineage_container_copy_on_write
            + self.retirement_container_copy_on_write
            + self.tree_table
    }

    pub(super) fn terms_text(self) -> String {
        format!(
            "{}+{}+{}+{}+{}+{}",
            self.allocation_record_tree,
            self.central_mapping_tree,
            self.accounting_tree,
            self.lineage_container_copy_on_write,
            self.retirement_container_copy_on_write,
            self.tree_table
        )
    }
}

/// 一次发布要接在后面的那一版，装置手里有的样子。
#[derive(Clone, Copy)]
pub(super) enum VersionToBuildOn<'version> {
    /// 树表 0 条的一版：核心层按分配器算（分配器记没记着这一版写行时写下的分配记录树，`allocation_record_tree_of_the_version_without_file`），
    /// 本地按装置看到的「这一版上写没写过行」算（[`PublishedVersion::wrote_the_allocation_record_tree_on_a_version_without_file`]）。
    WithoutFile {
        allocation_record_tree_written: bool,
        allocator: &'version PoolAllocator,
    },
    /// 有树的一版：两边都从这一版现读。
    WithTrees(&'version TransactionOutput),
}

/// κ 是接在哪一种版本后面算的（产物与汇总只拿有树的那几版定 κ_G，登记修订）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BuiltOn {
    VersionWithoutFile,
    VersionWithTrees,
}

impl BuiltOn {
    pub(super) fn label(self) -> &'static str {
        match self {
            BuiltOn::VersionWithoutFile => "version_without_file",
            BuiltOn::VersionWithTrees => "version_with_trees",
        }
    }
}

/// 本地 κ（D28（挂载期承诺量） 已定项 4）：单元区与池几何的根层级按这个几何本地算，各棵树的形状从那一版读。
fn local_checkpoint_cost(geometry: Geometry, version: VersionToBuildOn<'_>) -> LocalCheckpointCost {
    let unit_area = geometry.local_unit_area();
    let unit_areas = [unit_area, unit_area];
    match version {
        VersionToBuildOn::WithoutFile {
            allocation_record_tree_written,
            ..
        } => {
            let device_slots = geometry.device_bytes() / SLOT_BYTES_LOCAL;
            let root_level = local_root_level_of_the_pool_geometry(&[device_slots, device_slots]);
            LocalCheckpointCost {
                allocation_record_tree: if allocation_record_tree_written {
                    local_allocation_record_tree_positions(root_level, &unit_areas)
                } else {
                    0
                },
                central_mapping_tree: 0,
                accounting_tree: 0,
                lineage_container_copy_on_write: 0,
                retirement_container_copy_on_write: 0,
                tree_table: TREE_TABLE_UNITS_PER_PUBLISH_LOCAL,
            }
        }
        VersionToBuildOn::WithTrees(previous) => {
            let root_level = u8::try_from(
                previous
                    .position_addressed_tree_heights_read_from_the_root_node_headers()
                    .allocation_record_tree
                    - 1,
            )
            .expect("根层级装得进 u8");
            let allocation_record_tree =
                local_allocation_record_tree_positions(root_level, &unit_areas);
            let capacities = CodeTwoTreeNodeCapacities::FromTheNodeFormat;
            let accounting_nodes_before =
                u64::try_from(previous.accounting_tree.node_count()).expect("节点数装得进 u64");
            let accounting_tree = accounting_tree_nodes_after_the_publish_built_on(
                previous,
                capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
            );
            let lineage_levels = previous.lineage_tree_internal_levels();
            let retirement_levels = previous.retirement_tree_internal_levels();
            let lineage_keys = MAPPED_KEYS_OF_ONE_CONTAINER_LOCAL
                + MAPPED_KEYS_OF_ONE_CONTAINER_TREE_ROOT_LOCAL
                + lineage_levels;
            let retirement_keys = MAPPED_KEYS_OF_ONE_CONTAINER_LOCAL
                + MAPPED_KEYS_OF_ONE_CONTAINER_TREE_ROOT_LOCAL
                + retirement_levels;
            let entries_before = u64::try_from(
                previous.mapped_units.len() + previous.mapping_entries_kept_for_the_snapshots.len(),
            )
            .expect("映射条目数装得进 u64");
            let central_mapping_tree = local_central_mapping_tree_nodes_rewritten_at_most(
                &central_mapping_tree_nodes_at_each_level(previous),
                entries_before,
                allocation_record_tree + accounting_nodes_before + lineage_keys + retirement_keys,
                allocation_record_tree + accounting_tree + lineage_keys + retirement_keys,
                capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
            );
            LocalCheckpointCost {
                allocation_record_tree,
                central_mapping_tree,
                accounting_tree,
                lineage_container_copy_on_write: LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                    + LINEAGE_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                    + lineage_levels,
                retirement_container_copy_on_write:
                    RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                        + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COPY_ON_WRITE_LOCAL
                        + retirement_levels,
                tree_table: TREE_TABLE_UNITS_PER_PUBLISH_LOCAL,
            }
        }
    }
}

/// 核心层的 κ（`checkpoint_cost_of_the_version_to_build_on`，产品路径的节点容量）与本地那一份。
/// 有树的一版上核心层只拿分配器的几何（各盘单元区），`allocator_of_the_geometry` 取这条历史手里任何一份都一样。
pub(super) fn checkpoint_cost_reading(
    geometry: Geometry,
    version: VersionToBuildOn<'_>,
    allocator_of_the_geometry: &PoolAllocator,
) -> CheckpointCostReading {
    let (built_on, core) = match version {
        VersionToBuildOn::WithoutFile { allocator, .. } => (
            BuiltOn::VersionWithoutFile,
            checkpoint_cost_of_the_version_to_build_on(None, allocator).0,
        ),
        VersionToBuildOn::WithTrees(previous) => (
            BuiltOn::VersionWithTrees,
            checkpoint_cost_of_the_version_to_build_on(Some(previous), allocator_of_the_geometry).0,
        ),
    };
    CheckpointCostReading {
        built_on,
        core,
        local: local_checkpoint_cost(geometry, version),
    }
}

/// 路一数固定点角色的次数：产品恒数一次；阳性对照 PC7 用只供测试的那一臂把每个固定点角色记两次，S3 那一判必须判红（登记第五节「阳性对照」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RouteOneCount {
    EachFixedPointRoleOnce,
    EachFixedPointRoleTwiceForThePositiveControl,
}

impl FixedPointAccount {
    /// 两条路在每块盘上相同（停机条款 S3）；路二取不出时 `None`。
    pub(super) fn routes_agree_on_every_device(&self) -> Option<bool> {
        let by_roles = i64::try_from(self.by_roles).expect("槽数装得进 i64");
        self.by_records_per_device
            .as_ref()
            .map(|per_device| per_device.iter().all(|by_records| *by_records == by_roles))
    }

    pub(super) fn by_records_text(&self) -> String {
        match &self.by_records_per_device {
            Some(per_device) => per_device
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
            None => "records_not_handed_in".to_string(),
        }
    }
}

pub(super) fn fixed_point_account(
    rewritten: &[TransactionUnit],
    records: Option<&[AllocationRecord]>,
    txg: CheckpointTxg,
    devices: &[DeviceIdentity],
    route_one_count: RouteOneCount,
) -> FixedPointAccount {
    let mut fixed_point_slots_by_roles = 0;
    let mut ordinary_slots = 0;
    let mut instance_switch_slots = 0;
    for role in rewritten {
        match space_budget_of_role(*role) {
            SpaceBudgetOfARole::CheckpointReservePool => {
                fixed_point_slots_by_roles += role.span_slots()
            }
            SpaceBudgetOfARole::OrdinaryAllocation => ordinary_slots += role.span_slots(),
            SpaceBudgetOfARole::InstanceSwitchReserve => instance_switch_slots += role.span_slots(),
        }
    }
    let by_roles = match route_one_count {
        RouteOneCount::EachFixedPointRoleOnce => fixed_point_slots_by_roles,
        RouteOneCount::EachFixedPointRoleTwiceForThePositiveControl => {
            2 * fixed_point_slots_by_roles
        }
    };
    let not_fixed_point_slots =
        i64::try_from(ordinary_slots + instance_switch_slots).expect("槽数装得进 i64");
    let by_records_per_device = records.map(|records_of_the_version| {
        devices
            .iter()
            .map(|device| {
                let allocated_in_this_publish: u64 = records_of_the_version
                    .iter()
                    .filter(|record| {
                        record.device == *device && record.generation == txg && !record.is_released
                    })
                    .map(|record| u64::from(record.span_slots))
                    .sum();
                i64::try_from(allocated_in_this_publish).expect("槽数装得进 i64")
                    - not_fixed_point_slots
            })
            .collect()
    });
    FixedPointAccount {
        by_roles,
        by_records_per_device,
        ordinary_slots,
        instance_switch_slots,
    }
}

#[cfg(test)]
mod tests;
