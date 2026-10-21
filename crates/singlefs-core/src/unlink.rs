//! unlink 单文件的计划（D8（核心索引结构） 已定项 16 第 2、6、7 条）：下一步做哪一批、那一批之后 extent 树长什么样。
//!
//! 和 `crate::inode_tree`、`crate::tombstone_tree` 一样只算、不发一个写、不动分配器、不碰落点：条款的几条纪律
//! （从尾部按数据单元个数分批、最后一批释放剩下全部单元、删除批只重写被截掉那一片下段叶与它的祖先）是纯函数的后置条件，单测判得了。
//!
//! 下一步只由「现行那一版这个文件还剩几个数据单元」与批大小定（[`next_unlink_step`]），不看这一串是从哪一步续上的：
//! 会话里的队列每次从现行那一版读、不缓存（第 6 条），续做与不崩溃一路做下来的计划因此逐步相同（第 7 条 ①「计划相等」）。

use std::collections::BTreeSet;

use singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

use crate::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
use crate::extent_tree::lower_segment_nodes_of_a_file_without_holes;
use crate::extent_tree::{ExtentLowerNodePosition, ExtentTreeVersion};
use crate::lineage_tree::LineageContainerCapacity;
use crate::pointer::NodePointer;
use crate::retirement_tree::RetirementContainerCapacity;
use crate::tombstone_tree::TombstoneContainerCapacity;
use crate::transaction::{
    extent_tree_plan_with_the_unified_upper_segment, ExtentTreeNodeOrigin, ExtentTreePlan,
    FIRST_INODE_NUMBER,
};
use crate::unit::data_unit_payload_capacity;

/// 删除与回退那几次发布的只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：每批几个数据单元、一片墓碑容器装满几条、
/// 一片谱系容器装满几条（管理员回退那次追加）、一片退役容器装满几条（最后一批追加）、容器树一个码 2 节点装满几条（墓碑树、谱系树、退役树根满长高）。
/// 产品路径是 [`DeletionSwitches::PRODUCT`]；都只住内存、不落盘（D18（块里携带什么信息） 已定项 24 第 3 条）。
/// 经可写挂载的只供测试的开关链装在分配器上（`mount::mount_writable_with_test_only_switches_and_deletion_switches`，与空间准入开关同一个做法），
/// 挂载续做与这次挂载之后的会话都照它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeletionSwitches {
    pub batch_size: UnlinkBatchSize,
    pub tombstone_container_capacity: TombstoneContainerCapacity,
    pub lineage_container_capacity: LineageContainerCapacity,
    pub retirement_container_capacity: RetirementContainerCapacity,
    pub container_tree_node_entries: ContainerTreeNodeEntries,
}

impl DeletionSwitches {
    pub const PRODUCT: DeletionSwitches = DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity: RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };

    /// 墓碑树、谱系树与退役树那四个容量开关，装上写入口用（`transaction::PoolWriter::set_container_tree_capacities`）。
    #[must_use]
    pub const fn container_tree_capacities(self) -> ContainerTreeCapacities {
        ContainerTreeCapacities {
            tombstone_container: self.tombstone_container_capacity,
            lineage_container: self.lineage_container_capacity,
            retirement_container: self.retirement_container_capacity,
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

/// 删快照的一批放几条 deadlist 条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4：批大小 K = 144、测试开关压小，
/// 「`DeletionSwitches` 旁」）：与 unlink 每批的数据单元数共用同一个开关（[`UnlinkBatchSize`]），产品 144、压小时取压的那个数——
/// 删快照照 unlink 的过程做（第七节岔路 4），两件事的批同一个大小，开关不另立一个。
///
/// # Panics
/// 批大小装不进 `usize`：产品 144、压小的至多 144。
#[must_use]
pub fn deadlist_entries_per_snapshot_destruction_batch(batch_size: UnlinkBatchSize) -> usize {
    usize::try_from(batch_size.data_units_per_batch()).expect("批大小至多 144")
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

/// 删除的一批（[`UnlinkStep::Batch`]）之后一个文件的下段（第一个文件与第一个文件之外的文件共用，判决
/// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1）：`nodes` 是这一版的节点与来历（bump 次序），
/// `previous_nodes_rewritten` 是上一版里这次重写的那几个（有同名后继：被截掉那一片叶与它的祖先），`previous_nodes_dropped` 是上一版里
/// 不再出现的那几个（没有同名后继：整个落在被截掉那一段里的），都按上一版的 bump 次序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LowerSegmentAfterADeletionBatch {
    pub nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    pub previous_nodes_rewritten: Vec<ExtentLowerNodePosition>,
    pub previous_nodes_dropped: Vec<ExtentLowerNodePosition>,
}

impl LowerSegmentAfterADeletionBatch {
    /// 上一版里被这次换下的全部下段节点（重写的与不再出现的），按上一版的 bump 次序。
    #[must_use]
    pub fn previous_nodes_replaced(&self) -> Vec<ExtentLowerNodePosition> {
        let mut replaced: Vec<ExtentLowerNodePosition> = self
            .previous_nodes_rewritten
            .iter()
            .chain(&self.previous_nodes_dropped)
            .copied()
            .collect();
        replaced.sort_unstable();
        replaced
    }
}

/// 删除的一批之后一个文件的下段：截到前 `data_units_kept` 个单元——罩的单元全在留下那一段里的节点一个字节都不变、照抄；罩着被截掉那一段的，
/// 在新下段里只剩每层最右那一个（被截掉那一片下段叶与它的祖先），它们重写；整个落在被截掉那一段里的节点不再出现、换下
/// （K-R3-1：删除批只重写被截掉那一片下段叶与它的祖先）。只剩一个单元时不建下段（内联，同写路径）。`previous_lower_nodes` 是这个文件
/// 上一版下段的节点（bump 次序）。
///
/// # Panics
/// `data_units_kept` 是 0、或不少于 `data_units_before_the_batch`：0 是最后一批，那一步不走这里；不少于说明这一批什么都没截。
#[must_use]
pub fn plan_the_lower_segment_after_a_deletion_batch(
    previous_lower_nodes: &[(ExtentLowerNodePosition, NodePointer)],
    data_units_before_the_batch: u64,
    data_units_kept: u64,
) -> LowerSegmentAfterADeletionBatch {
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
                        previous_lower_nodes
                            .iter()
                            .any(|(previous_position, _)| *previous_position == position),
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
    let carried_lower: Vec<ExtentLowerNodePosition> = lower_nodes
        .iter()
        .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::CarriedFromThePreviousVersion)
        .map(|(position, _)| *position)
        .collect();
    let (previous_nodes_rewritten, previous_nodes_dropped): (
        Vec<ExtentLowerNodePosition>,
        Vec<ExtentLowerNodePosition>,
    ) = previous_lower_nodes
        .iter()
        .filter(|(position, _)| !carried_lower.contains(position))
        .map(|(position, _)| *position)
        .partition(|position| {
            lower_nodes
                .iter()
                .any(|(position_after_the_batch, _)| position_after_the_batch == position)
        });
    LowerSegmentAfterADeletionBatch {
        nodes: lower_nodes,
        previous_nodes_rewritten,
        previous_nodes_dropped,
    }
}

/// 删 `inode` 的一批（[`UnlinkStep::Batch`]）之后第一个文件那一份 extent 计划（D8（核心索引结构） 已定项 16 第 2 条，判决 m3-multi-file-unlink-r1
/// 第三节 M1、M2）：`inode` 是第一个文件时它的下段按 [`plan_the_lower_segment_after_a_deletion_batch`] 截（`data_units_before_the_batch`、
/// `data_units_kept` 是它的单元数），别的文件时第一个文件的下段整段照抄（那个文件自己的下段由发布路径按它上一版的下段截，进
/// `transaction::FilesOtherThanTheFirstPlan`）。上段走统一计划：这一批不改有条目的 inode 号集合 `inodes_with_an_entry`（截完至少留一个单元），
/// 条目变了的是 `inode`（上段叶条目里下段根的指针或内联指针变了），它那片叶到根那一串重写，别的照抄。
///
/// # Panics
/// `inode` 是第一个文件而 `data_units_kept` 不在 `1..data_units_before_the_batch` 里（[`plan_the_lower_segment_after_a_deletion_batch`]）。
#[must_use]
pub fn plan_the_extent_tree_after_a_deletion_batch(
    previous: &ExtentTreeVersion,
    inodes_with_an_entry: &BTreeSet<u64>,
    inode: u64,
    data_units_before_the_batch: u64,
    data_units_kept: u64,
) -> ExtentTreePlan {
    let (first_file_lower_nodes, first_file_lower_nodes_replaced) = if inode == FIRST_INODE_NUMBER {
        let lower = plan_the_lower_segment_after_a_deletion_batch(
            &previous.lower_nodes,
            data_units_before_the_batch,
            data_units_kept,
        );
        let replaced = lower.previous_nodes_replaced();
        (lower.nodes, replaced)
    } else {
        (lower_nodes_carried_whole(previous), Vec::new())
    };
    extent_tree_plan_with_the_unified_upper_segment(
        Some(previous),
        first_file_lower_nodes,
        first_file_lower_nodes_replaced,
        inodes_with_an_entry,
        &BTreeSet::from([inode]),
    )
}

/// 删 `inode` 的最后一批（[`UnlinkStep::FinalBatch`]）之后第一个文件那一份 extent 计划（D8（核心索引结构） 已定项 16 第 2、5 条，判决
/// m3-multi-file-unlink-r1 第三节 M1、M3）：`inode` 是第一个文件时它的下段整段换下、一个节点都不剩，别的文件时第一个文件的下段整段照抄
/// （那个文件的数据单元与下段由发布路径点名换下）。上段走统一计划：`inode` 出了有条目的号集合（`inodes_with_an_entry_before` 是上一版的；
/// 它不在里面时——没有数据单元的 inode——集合不变、上段整段照抄），它的条目去掉、不写标签 0（D8（核心索引结构） 已定项 14），
/// 集合空了是层级 0 的零条目根兼叶，上段跟着降层（上一版里降后层级的第 0 个节点照抄成根，旧根与脊线上方摘掉的节点换下）。
#[must_use]
pub fn plan_the_extent_tree_after_the_final_batch(
    previous: &ExtentTreeVersion,
    inodes_with_an_entry_before: &BTreeSet<u64>,
    inode: u64,
) -> ExtentTreePlan {
    let (first_file_lower_nodes, first_file_lower_nodes_replaced) = if inode == FIRST_INODE_NUMBER {
        (
            Vec::new(),
            previous
                .lower_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect(),
        )
    } else {
        (lower_nodes_carried_whole(previous), Vec::new())
    };
    let mut inodes_with_an_entry_after = inodes_with_an_entry_before.clone();
    let inodes_whose_entry_changed = if inodes_with_an_entry_after.remove(&inode) {
        BTreeSet::from([inode])
    } else {
        BTreeSet::new()
    };
    extent_tree_plan_with_the_unified_upper_segment(
        Some(previous),
        first_file_lower_nodes,
        first_file_lower_nodes_replaced,
        &inodes_with_an_entry_after,
        &inodes_whose_entry_changed,
    )
}

/// 第一个文件的下段整段照抄（删的不是它的那几步）。
fn lower_nodes_carried_whole(
    previous: &ExtentTreeVersion,
) -> Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)> {
    previous
        .lower_nodes
        .iter()
        .map(|(position, _)| {
            (
                *position,
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extent_tree::ExtentUpperNodePosition;
    use crate::transaction::TransactionUnit;

    const PAYLOAD_IN_BYTES: u64 = 32634;

    /// 从还剩 `data_units_remaining` 个数据单元起走到底的每一步，按先后：最后一步恒是 [`UnlinkStep::FinalBatch`]。
    fn every_unlink_step_from(
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

    fn upper(level: u8, index: u64) -> ExtentUpperNodePosition {
        ExtentUpperNodePosition { level, index }
    }

    /// 只有第一个文件在 extent 树里有条目时有条目的 inode 号集合。
    fn only_the_first_file() -> BTreeSet<u64> {
        BTreeSet::from([FIRST_INODE_NUMBER])
    }

    /// 上段是 `upper_nodes` 这几个位置（bump 次序）、第一个文件下段是 `first_file_data_units` 个单元的 extent 树，指针取全 0。
    fn extent_tree_with_the_upper_nodes(
        upper_nodes: &[ExtentUpperNodePosition],
        first_file_data_units: u64,
    ) -> ExtentTreeVersion {
        ExtentTreeVersion {
            upper_nodes: upper_nodes
                .iter()
                .map(|position| (*position, NodePointer::empty_root()))
                .collect(),
            lower_nodes: lower_segment_nodes_of_a_file_without_holes(first_file_data_units)
                .into_iter()
                .map(|position| (position, NodePointer::empty_root()))
                .collect(),
        }
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
        let plan = plan_the_extent_tree_after_a_deletion_batch(
            &previous,
            &only_the_first_file(),
            FIRST_INODE_NUMBER,
            300,
            200,
        );
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
        let plan = plan_the_extent_tree_after_a_deletion_batch(
            &previous,
            &only_the_first_file(),
            FIRST_INODE_NUMBER,
            300,
            144,
        );
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
        let plan = plan_the_extent_tree_after_a_deletion_batch(
            &previous,
            &only_the_first_file(),
            FIRST_INODE_NUMBER,
            3,
            1,
        );
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

    /// 删第一个文件之外的 inode 500 的一批（判决 m3-multi-file-unlink-r1 第三节 M1）：第一个文件的下段整段照抄、一个都不换下，
    /// 上段只重写罩着 500 的那一串（它那片叶 (0, 3) 与根 (1, 0)），第一个文件那片叶 (0, 0) 照抄；有条目的号集合不变、根层级不变。
    #[test]
    fn a_deletion_batch_of_another_inode_carries_the_first_file_and_rewrites_only_the_upper_path_of_that_inode(
    ) {
        let previous =
            extent_tree_with_the_upper_nodes(&[upper(0, 0), upper(0, 3), upper(1, 0)], 3);
        let plan = plan_the_extent_tree_after_a_deletion_batch(
            &previous,
            &BTreeSet::from([FIRST_INODE_NUMBER, 500]),
            500,
            300,
            200,
        );
        assert!(
            plan.lower_nodes
                .iter()
                .all(|(_, origin)| *origin == ExtentTreeNodeOrigin::CarriedFromThePreviousVersion),
            "第一个文件的下段整段照抄"
        );
        assert_eq!(
            plan.upper_nodes,
            vec![
                (
                    upper(0, 0),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (upper(0, 3), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (upper(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentUpperNodeBelowTheRoot(upper(0, 3)),
                TransactionUnit::ExtentRoot,
            ],
            "只换下 500 那一串：第一个文件的下段与它那片叶都不换下"
        );
    }

    /// 别的文件 500 还在时删第一个文件的最后一批（判决 m3-multi-file-unlink-r1 第三节 M3）：第一个文件的下段整段换下，上段去掉 inode 1、
    /// 第 0 片叶不再出现（换下），根 (1, 0) 少了一个孩子、重写，500 那片叶照抄；根层级不降（500 要层级 1）。
    #[test]
    fn the_final_batch_of_the_first_file_beside_another_file_drops_the_leaf_of_inode_one_and_rewrites_the_root(
    ) {
        let previous =
            extent_tree_with_the_upper_nodes(&[upper(0, 0), upper(0, 3), upper(1, 0)], 3);
        let plan = plan_the_extent_tree_after_the_final_batch(
            &previous,
            &BTreeSet::from([FIRST_INODE_NUMBER, 500]),
            FIRST_INODE_NUMBER,
        );
        assert!(plan.lower_nodes.is_empty(), "第一个文件的下段一个不剩");
        assert_eq!(
            plan.upper_nodes,
            vec![
                (
                    upper(0, 3),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (upper(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentUpperNodeBelowTheRoot(upper(0, 0)),
                TransactionUnit::ExtentRoot,
            ]
        );
    }

    /// 第一个文件还在时删 inode 500 的最后一批：上段降层——上一版第 0 片叶 (0, 0) 照抄成根（它不罩 500，内容没变），500 那片叶与旧根换下，
    /// 旧根按上一版的根层级点名（`ExtentRoot`），第 0 片叶这一版是根、上一版是根之下的节点。第一个文件的下段整段照抄。
    #[test]
    fn the_final_batch_of_another_file_lowers_the_upper_segment_and_carries_the_first_leaf_as_the_root(
    ) {
        let previous =
            extent_tree_with_the_upper_nodes(&[upper(0, 0), upper(0, 3), upper(1, 0)], 3);
        let plan = plan_the_extent_tree_after_the_final_batch(
            &previous,
            &BTreeSet::from([FIRST_INODE_NUMBER, 500]),
            500,
        );
        assert_eq!(
            plan.upper_nodes,
            vec![(
                upper(0, 0),
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            )]
        );
        assert_eq!(
            plan.upper_root_level(),
            0,
            "根层级降到罩得住 inode 1 的最低一层"
        );
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentUpperNodeBelowTheRoot(upper(0, 3)),
                TransactionUnit::ExtentRoot,
            ]
        );
        assert!(
            plan.rewritten_roles().is_empty(),
            "降层那次一个上段节点都不重写"
        );
    }

    /// 删一个没有数据单元的 inode（size 0，上段里没有它的条目）的最后一批：有条目的号集合不变，extent 树整棵照抄、一个都不换下。
    #[test]
    fn the_final_batch_of_an_inode_without_data_units_leaves_the_extent_tree_carried() {
        let previous = extent_tree_of_a_file_without_holes(3);
        let plan = plan_the_extent_tree_after_the_final_batch(&previous, &only_the_first_file(), 7);
        assert!(plan.rewritten_roles().is_empty());
        assert!(plan.replaced_previous_roles.is_empty());
    }

    /// 删除的一批截别的文件的下段（共用那一份）：300 截到 200 时上一版第 1 片叶与根重写（有同名后继），第 2 片叶不再出现（没有同名后继）。
    #[test]
    fn the_lower_segment_of_a_deletion_batch_splits_the_replaced_nodes_by_whether_they_have_a_successor(
    ) {
        let previous = extent_tree_of_a_file_without_holes(300);
        let lower_segment =
            plan_the_lower_segment_after_a_deletion_batch(&previous.lower_nodes, 300, 200);
        assert_eq!(
            lower_segment.previous_nodes_rewritten,
            vec![lower(0, 1), lower(1, 0)]
        );
        assert_eq!(lower_segment.previous_nodes_dropped, vec![lower(0, 2)]);
    }
}
