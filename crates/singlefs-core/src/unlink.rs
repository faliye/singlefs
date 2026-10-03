//! unlink 单文件的计划（D8（核心索引结构） 已定项 16 第 2、6、7 条）：下一步做哪一批、那一批之后 extent 树长什么样。
//!
//! 和 `crate::inode_tree`、`crate::tombstone_tree` 一样只算、不发一个写、不动分配器、不碰落点：条款的几条纪律
//! （从尾部按数据单元个数分批、最后一批释放剩下全部单元、删除批只重写被截掉那一片下段叶与它的祖先）是纯函数的后置条件，单测判得了。
//!
//! 下一步只由「现行那一版这个文件还剩几个数据单元」与批大小定（[`next_unlink_step`]），不看这一串是从哪一步续上的：
//! 会话里的队列每次从现行那一版读、不缓存（第 6 条），续做与不崩溃一路做下来的计划因此逐步相同（第 7 条 ①「计划相等」）。

use singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

use crate::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
use crate::extent_tree::{lower_segment_nodes_of_a_file_without_holes, upper_path_of_inode};
use crate::extent_tree::{ExtentLowerNodePosition, ExtentTreeVersion};
use crate::lineage_tree::LineageContainerCapacity;
use crate::tombstone_tree::TombstoneContainerCapacity;
use crate::transaction::{
    role_of_extent_upper_node, ExtentTreeNodeOrigin, ExtentTreePlan, TransactionUnit,
};
use crate::unit::data_unit_payload_capacity;

/// 删除与回退那几次发布的只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：每批几个数据单元、一片墓碑容器装满几条、
/// 一片谱系容器装满几条（管理员回退那次追加）、容器树一个码 2 节点装满几条（墓碑树、谱系树根满长高）。
/// 产品路径是 [`DeletionSwitches::PRODUCT`]；都只住内存、不落盘（D18（块里携带什么信息） 已定项 24 第 3 条）。
/// 经可写挂载的只供测试的开关链装在分配器上（`mount::mount_writable_with_test_only_switches_and_deletion_switches`，与空间准入开关同一个做法），
/// 挂载续做与这次挂载之后的会话都照它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeletionSwitches {
    pub batch_size: UnlinkBatchSize,
    pub tombstone_container_capacity: TombstoneContainerCapacity,
    pub lineage_container_capacity: LineageContainerCapacity,
    pub container_tree_node_entries: ContainerTreeNodeEntries,
}

impl DeletionSwitches {
    pub const PRODUCT: DeletionSwitches = DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };

    /// 墓碑树与谱系树那三个容量开关，装上写入口用（`transaction::PoolWriter::set_container_tree_capacities`）。
    #[must_use]
    pub const fn container_tree_capacities(self) -> ContainerTreeCapacities {
        ContainerTreeCapacities {
            tombstone_container: self.tombstone_container_capacity,
            lineage_container: self.lineage_container_capacity,
            node_entries: self.container_tree_node_entries,
        }
    }
}

/// 只供测试压小的每批数据单元数（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「每两批之间崩」这一格按产品批大小 144
/// 要 145 个单元以上的文件才走得到）。至少 1、至多产品批大小；只住内存，不落盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnlinkBatchDataUnitsCappedForTests(u64);

impl UnlinkBatchDataUnitsCappedForTests {
    /// 0（一批什么都不释放，永远走不到最后一批）或多于产品批大小交回 `None`。
    #[must_use]
    pub fn new(data_units: u64) -> Option<Self> {
        (1..=EXTENT_TREE_LOWER_LEAF_DATA_UNITS)
            .contains(&data_units)
            .then_some(Self(data_units))
    }

    #[must_use]
    pub const fn data_units(self) -> u64 {
        self.0
    }
}

/// 每批释放尾部几个数据单元（D8（核心索引结构） 已定项 16 第 2 条的 K）：产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），
/// 只供测试的开关把它压小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnlinkBatchSize {
    OneExtentLowerLeaf,
    CappedForTests(UnlinkBatchDataUnitsCappedForTests),
}

impl UnlinkBatchSize {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            UnlinkBatchSize::OneExtentLowerLeaf => "unlink_batch_size_one_extent_lower_leaf",
            UnlinkBatchSize::CappedForTests(_) => "unlink_batch_size_capped_for_tests",
        }
    }

    /// 一批释放几个数据单元。
    #[must_use]
    pub const fn data_units_per_batch(self) -> u64 {
        match self {
            UnlinkBatchSize::OneExtentLowerLeaf => EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            UnlinkBatchSize::CappedForTests(capped) => capped.data_units(),
        }
    }
}

/// unlink 意图之后的一步（D8（核心索引结构） 已定项 16 第 2 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnlinkStep {
    /// 不是最后一批：释放尾部一批单元，留下前 `data_units_kept` 个（恒 ≥ 1），inode 记录的 size 改成 `size_after_the_batch_in_bytes`
    /// （= 剩下单元数 × 净荷容量），nlink 仍是 0；inode 记录与 `deleted_inodes` 那条 key 都留着。
    Batch {
        data_units_kept: u64,
        size_after_the_batch_in_bytes: u64,
    },
    /// 最后一批：释放剩下的 `data_units_released` 个单元（可以是 0：没有数据单元的文件只有这一批）、删 inode 记录、
    /// 删 `deleted_inodes` 那条 key，同一次发布。
    FinalBatch { data_units_released: u64 },
}

/// 现行那一版这个文件还剩 `data_units_remaining` 个数据单元时，下一步是哪一批：剩下的多于一批 ⇒ 释放尾部一批；
/// 剩下的不多于一批 ⇒ 最后一批把它们全部释放（「只有这一条路，不设一次做完的快路」：剩下的再少也走最后一批这一个形状）。
#[must_use]
pub fn next_unlink_step(data_units_remaining: u64, batch_size: UnlinkBatchSize) -> UnlinkStep {
    let data_units_per_batch = batch_size.data_units_per_batch();
    if data_units_remaining <= data_units_per_batch {
        return UnlinkStep::FinalBatch {
            data_units_released: data_units_remaining,
        };
    }
    let data_units_kept = data_units_remaining - data_units_per_batch;
    UnlinkStep::Batch {
        data_units_kept,
        size_after_the_batch_in_bytes: data_units_kept
            * u64::try_from(data_unit_payload_capacity()).expect("32634"),
    }
}

/// 从还剩 `data_units_remaining` 个数据单元起走到底的每一步，按先后：最后一步恒是 [`UnlinkStep::FinalBatch`]。
#[must_use]
pub fn every_unlink_step_from(
    data_units_remaining: u64,
    batch_size: UnlinkBatchSize,
) -> Vec<UnlinkStep> {
    let mut steps = Vec::new();
    let mut remaining = data_units_remaining;
    // 迭代上界：每一轮剩下的单元数至少少 1（批大小至少 1），⌈剩下 ÷ 批大小⌉ 轮之内走到最后一批。
    loop {
        let step = next_unlink_step(remaining, batch_size);
        steps.push(step);
        match step {
            UnlinkStep::Batch {
                data_units_kept, ..
            } => {
                assert!(data_units_kept < remaining, "每一批至少释放一个单元");
                remaining = data_units_kept;
            }
            UnlinkStep::FinalBatch { .. } => return steps,
        }
    }
}

/// 删除的一批（[`UnlinkStep::Batch`]）之后 inode `inode` 那棵 extent 树：下段截到前 `data_units_kept` 个单元——
/// 罩的单元全在留下那一段里的节点一个字节都不变、照抄；罩着被截掉那一段的，在新下段里只剩每层最右那一个（被截掉那一片下段叶与它的祖先），
/// 它们重写；整个落在被截掉那一段里的节点不再出现、换下（K-R3-1：删除批只重写被截掉那一片下段叶与它的祖先）。
/// 只剩一个单元时不建下段（内联，同写路径）。上段：这个文件那片叶到根那一串重写（上段叶条目里下段根的指针或内联指针变了），上段别的照抄，
/// 上段根的层级不变（这一批不改 inode 号的集合）。
///
/// # Panics
/// `data_units_kept` 是 0、或不少于 `data_units_before_the_batch`：0 是最后一批，那一步不走这里；不少于说明这一批什么都没截。
#[must_use]
pub fn plan_the_extent_tree_after_a_deletion_batch(
    previous: &ExtentTreeVersion,
    inode: u64,
    data_units_before_the_batch: u64,
    data_units_kept: u64,
) -> ExtentTreePlan {
    assert!(
        (1..data_units_before_the_batch).contains(&data_units_kept),
        "删除的一批留下至少一个单元、至少截掉一个（留下 0 个是最后一批）"
    );
    let is_entirely_kept = |position: &ExtentLowerNodePosition| {
        let (_, last_data_unit) = position.data_unit_range();
        last_data_unit < data_units_kept
    };
    let lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)> =
        lower_segment_nodes_of_a_file_without_holes(data_units_kept)
            .into_iter()
            .map(|position| {
                if is_entirely_kept(&position) {
                    assert!(
                        previous.lower_pointer_of(position).is_some(),
                        "罩的单元全在留下那一段里的节点，上一版（单元更多）同一个位置上必有"
                    );
                    (
                        position,
                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                    )
                } else {
                    (position, ExtentTreeNodeOrigin::RewrittenThisPublish)
                }
            })
            .collect();
    let upper_root_level = previous.upper_root().level;
    let path_of_the_file = upper_path_of_inode(inode, upper_root_level);
    let upper_nodes = previous
        .upper_nodes
        .iter()
        .map(|(position, _)| {
            let origin = if path_of_the_file.contains(position) {
                ExtentTreeNodeOrigin::RewrittenThisPublish
            } else {
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            };
            (*position, origin)
        })
        .collect();
    let carried_lower: Vec<ExtentLowerNodePosition> = lower_nodes
        .iter()
        .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::CarriedFromThePreviousVersion)
        .map(|(position, _)| *position)
        .collect();
    let replaced_previous_roles = previous
        .lower_nodes
        .iter()
        .filter(|(position, _)| !carried_lower.contains(position))
        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
        .chain(
            previous
                .upper_nodes
                .iter()
                .filter(|(position, _)| path_of_the_file.contains(position))
                .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
        )
        .collect();
    ExtentTreePlan {
        lower_nodes,
        upper_nodes,
        replaced_previous_roles,
    }
}

/// 删 inode 1 的最后一批之后的 extent 树（D8（核心索引结构） 已定项 16 第 2、5 条）：下段整段换下、一个节点都不剩；上段那片根兼叶重写成
/// 零条目根（key 区间照旧按位置规定）。第一版只有第一个文件在 extent 树里有条目、它在第 0 片叶里，上段因此只有根兼叶那一个节点。
///
/// # Panics
/// 上一版上段不止根兼叶一个节点：那一版有别的 inode 的条目，第一版写不出来（只有第一个文件写数据）。
#[must_use]
pub fn plan_the_extent_tree_after_the_final_batch_of_the_first_file(
    previous: &ExtentTreeVersion,
) -> ExtentTreePlan {
    let upper_root = previous.upper_root();
    assert!(
        upper_root.level == 0 && previous.upper_nodes.len() == 1,
        "第一版只有第一个文件在 extent 树里有条目：上段只有根兼叶一个节点"
    );
    ExtentTreePlan {
        lower_nodes: Vec::new(),
        upper_nodes: vec![(upper_root, ExtentTreeNodeOrigin::RewrittenThisPublish)],
        replaced_previous_roles: previous
            .lower_nodes
            .iter()
            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
            .chain(std::iter::once(role_of_extent_upper_node(
                upper_root,
                upper_root.level,
            )))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extent_tree::ExtentUpperNodePosition;
    use crate::pointer::NodePointer;
    use crate::transaction::FIRST_INODE_NUMBER;

    const PAYLOAD_IN_BYTES: u64 = 32634;

    fn capped(data_units: u64) -> UnlinkBatchSize {
        UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(data_units).expect("1..=144"),
        )
    }

    /// 一个 `data_units` 个单元、没有洞的文件那棵 extent 树（上段只有根兼叶：第一个文件在上段第 0 片叶里），指针不进计划、取全 0。
    fn extent_tree_of_a_file_without_holes(data_units: u64) -> ExtentTreeVersion {
        ExtentTreeVersion {
            upper_nodes: vec![(
                ExtentUpperNodePosition { level: 0, index: 0 },
                NodePointer::empty_root(),
            )],
            lower_nodes: lower_segment_nodes_of_a_file_without_holes(data_units)
                .into_iter()
                .map(|position| (position, NodePointer::empty_root()))
                .collect(),
        }
    }

    fn lower(level: u8, index: u64) -> ExtentLowerNodePosition {
        ExtentLowerNodePosition { level, index }
    }

    /// 产品批大小是一片下段叶的单元数 144；只供测试的开关至少 1、至多 144，两条分支各报自己的名字。
    #[test]
    fn the_product_batch_is_one_extent_lower_leaf_and_the_test_switch_lies_between_one_and_it() {
        assert_eq!(
            UnlinkBatchSize::OneExtentLowerLeaf.data_units_per_batch(),
            144
        );
        assert_eq!(UnlinkBatchDataUnitsCappedForTests::new(0), None);
        assert_eq!(UnlinkBatchDataUnitsCappedForTests::new(145), None);
        assert_eq!(capped(144).data_units_per_batch(), 144);
        assert_ne!(
            UnlinkBatchSize::OneExtentLowerLeaf.name(),
            capped(1).name(),
            "两条分支运行时分得开"
        );
    }

    /// 三个单元、每批 1 个：两批各释放尾部 1 个、size 改成剩下单元数 × 32634，最后一批释放剩下那 1 个。
    #[test]
    fn three_data_units_in_batches_of_one_take_two_batches_and_a_final_batch_of_one() {
        assert_eq!(
            every_unlink_step_from(3, capped(1)),
            vec![
                UnlinkStep::Batch {
                    data_units_kept: 2,
                    size_after_the_batch_in_bytes: 2 * PAYLOAD_IN_BYTES,
                },
                UnlinkStep::Batch {
                    data_units_kept: 1,
                    size_after_the_batch_in_bytes: PAYLOAD_IN_BYTES,
                },
                UnlinkStep::FinalBatch {
                    data_units_released: 1
                },
            ]
        );
    }

    /// 剩下的不多于一批就是最后一批：产品批大小下 144 个单元的文件只有最后一批，145 个的先截掉尾部 144 个；
    /// 没有数据单元的文件只有一个释放 0 个的最后一批（不设一次做完的快路，形状同一个）。
    #[test]
    fn a_file_of_at_most_one_batch_goes_straight_to_the_final_batch() {
        let product = UnlinkBatchSize::OneExtentLowerLeaf;
        assert_eq!(
            next_unlink_step(144, product),
            UnlinkStep::FinalBatch {
                data_units_released: 144
            }
        );
        assert_eq!(
            next_unlink_step(145, product),
            UnlinkStep::Batch {
                data_units_kept: 1,
                size_after_the_batch_in_bytes: PAYLOAD_IN_BYTES,
            }
        );
        assert_eq!(
            every_unlink_step_from(0, product),
            vec![UnlinkStep::FinalBatch {
                data_units_released: 0
            }]
        );
    }

    /// 第 7 条 ①「计划相等」：从走到一半的那一版续做（只看还剩几个单元）算出的每一步，与不崩溃一路做下来的后半截逐步相同。
    #[test]
    fn the_plan_continued_from_any_intermediate_version_equals_the_rest_of_the_uninterrupted_plan()
    {
        let batch_size = capped(2);
        let uninterrupted = every_unlink_step_from(7, batch_size);
        assert_eq!(uninterrupted.len(), 4, "7 → 5 → 3 → 1 → 最后一批");
        for (steps_done, step) in uninterrupted.iter().enumerate() {
            let UnlinkStep::Batch {
                data_units_kept, ..
            } = *step
            else {
                continue;
            };
            assert_eq!(
                every_unlink_step_from(data_units_kept, batch_size),
                uninterrupted[steps_done + 1..].to_vec(),
                "做完第 {steps_done} 批（剩 {data_units_kept} 个）再续做"
            );
        }
    }

    /// 300 个单元截到 200：下段三片叶加一个根变成两片叶加一个根。第 0 片叶罩 0..=143、全在留下那一段里 ⇒ 照抄；
    /// 第 1 片叶罩 144..=287、被截掉一半 ⇒ 重写；根罩着被截掉那一段 ⇒ 重写；第 2 片叶整片被截 ⇒ 换下、不再出现。
    /// 上段这个文件那一串（这里只有根兼叶）重写。
    #[test]
    fn a_deletion_batch_rewrites_only_the_cut_lower_leaf_and_its_ancestors() {
        let previous = extent_tree_of_a_file_without_holes(300);
        let plan =
            plan_the_extent_tree_after_a_deletion_batch(&previous, FIRST_INODE_NUMBER, 300, 200);
        assert_eq!(
            plan.lower_nodes,
            vec![
                (
                    lower(0, 0),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (lower(0, 1), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert_eq!(
            plan.rewritten_roles(),
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 1)),
                TransactionUnit::ExtentLowerNode(lower(1, 0)),
                TransactionUnit::ExtentRoot,
            ],
            "重写角色集合钉绝对值：被截掉那片叶、下段根、上段根"
        );
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 1)),
                TransactionUnit::ExtentLowerNode(lower(0, 2)),
                TransactionUnit::ExtentLowerNode(lower(1, 0)),
                TransactionUnit::ExtentRoot,
            ]
        );
    }

    /// 截在叶边界上（300 截到 144）：留下的第 0 片叶一个字节都不变 ⇒ 照抄，它成了下段根（下段矮一层）；
    /// 下段一个节点都不重写，上段那一串照样重写（上段叶条目里下段根的指针换成第 0 片叶的）。
    #[test]
    fn a_deletion_batch_cut_on_a_leaf_boundary_carries_the_kept_leaf_as_the_new_lower_root() {
        let previous = extent_tree_of_a_file_without_holes(300);
        let plan =
            plan_the_extent_tree_after_a_deletion_batch(&previous, FIRST_INODE_NUMBER, 300, 144);
        assert_eq!(
            plan.lower_nodes,
            vec![(
                lower(0, 0),
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            )]
        );
        assert_eq!(plan.rewritten_roles(), vec![TransactionUnit::ExtentRoot]);
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 1)),
                TransactionUnit::ExtentLowerNode(lower(0, 2)),
                TransactionUnit::ExtentLowerNode(lower(1, 0)),
                TransactionUnit::ExtentRoot,
            ]
        );
    }

    /// 截到只剩一个单元：不建下段（内联，同写路径），上一版下段每个节点都换下。
    #[test]
    fn a_deletion_batch_down_to_one_data_unit_drops_the_whole_lower_segment() {
        let previous = extent_tree_of_a_file_without_holes(3);
        let plan = plan_the_extent_tree_after_a_deletion_batch(&previous, FIRST_INODE_NUMBER, 3, 1);
        assert!(plan.lower_nodes.is_empty(), "一个单元内联、不建下段");
        assert_eq!(plan.rewritten_roles(), vec![TransactionUnit::ExtentRoot]);
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentRoot,
            ]
        );
    }
}
