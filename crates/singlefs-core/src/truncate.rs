//! 截断（truncate）与变长的计划（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5、K11、K12）：
//! 意图那次要不要重写尾单元、之后每一批留几个单元、改 size 之后 extent 树哪几个节点重写。
//!
//! 和 `crate::unlink` 一样只算、不发一个写、不动分配器、不碰落点：条款的几条纪律（先落目标：意图那次写 size = N 并重写尾单元；
//! 每批只从尾部放 K 个、不碰 inode 记录；最后一批放完剩下的、可以 0 个；罩着被重写的单元 k 的下段叶到根恒重写）是纯函数的后置条件，单测判得了。
//!
//! 下一步只由「现行那一版这个文件还挂着几个单元」、目标 size 与批大小定（[`next_truncation_step`]），不看这一串是从哪一步续上的：
//! 续做与不崩溃一路做下来的计划逐步相同（D8（核心索引结构） 已定项 16 第 7 条 ①「计划相等」，截断照它的幂等纪律）。

use crate::address::{CheckpointTxg, DataUnitIndexInFile};
use crate::extent_tree::{
    lower_segment_nodes_of_a_file_without_holes, upper_path_of_inode, ExtentLowerNodePosition,
    ExtentTreeVersion, TailOfAFile,
};
use crate::records::InodeRecordTruncationField;
use crate::transaction::{
    role_of_extent_upper_node, ExtentTreeNodeOrigin, ExtentTreePlan, TransactionUnit,
};
use crate::unit::data_unit_payload_capacity;
use crate::unlink::UnlinkBatchSize;
use crate::write_request_split::data_unit_count_of_a_sequential_write;

fn payload_capacity_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// size 为 `size_in_bytes` 的文件要几个数据单元：⌈size ÷ 净荷⌉.max(1)（与写侧切分同一条除法，截到 0 留一个声明长度 0 的单元，K3）。
#[must_use]
pub fn data_units_a_size_needs(size_in_bytes: u64) -> u64 {
    data_unit_count_of_a_sequential_write(size_in_bytes)
}

/// 第 `unit` 个数据单元在 size 为 `size_in_bytes` 的文件里装几字节：min(size − 起点, 净荷)，起点在 size 之外是 0。
#[must_use]
pub fn declared_length_within_a_size(unit: DataUnitIndexInFile, size_in_bytes: u64) -> u64 {
    let payload = payload_capacity_in_bytes();
    size_in_bytes
        .saturating_sub(unit.first_file_byte(payload).0)
        .min(payload)
}

/// 截断目标 size 的上限（K12）：⌈N ÷ 净荷⌉ × 净荷装得进 u64。超了在任何写之前拒。
#[must_use]
pub fn target_size_rounded_up_to_a_whole_data_unit_fits(target_size_in_bytes: u64) -> bool {
    let payload = payload_capacity_in_bytes();
    target_size_in_bytes
        .div_ceil(payload)
        .checked_mul(payload)
        .is_some()
}

/// 一次改 size 的请求走哪一条（K2、K5、K11、K12）：只看现在的 size 与目标 size，两样在操作期间不变（`.claude/rules/fs-design.md`
/// 分支的三条约束第 1 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeChangeOfARequest {
    /// N = S：一次只改记录（改动计数、时间）的发布（K12）。
    SizeUnchanged,
    /// N > S：一次发布、不立意图（K5）。
    Extension,
    /// N < S：意图 + 若干批 + 最后一批（K2、K11：放 0 个单元的也走这一条）。
    Truncation,
}

impl SizeChangeOfARequest {
    #[must_use]
    pub fn of(size_before_in_bytes: u64, target_size_in_bytes: u64) -> Self {
        match target_size_in_bytes.cmp(&size_before_in_bytes) {
            std::cmp::Ordering::Equal => SizeChangeOfARequest::SizeUnchanged,
            std::cmp::Ordering::Greater => SizeChangeOfARequest::Extension,
            std::cmp::Ordering::Less => SizeChangeOfARequest::Truncation,
        }
    }

    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            SizeChangeOfARequest::SizeUnchanged => "size_change_size_unchanged",
            SizeChangeOfARequest::Extension => "size_change_extension",
            SizeChangeOfARequest::Truncation => "size_change_truncation",
        }
    }
}

/// 改 size 那次要重写的那一个尾单元：哪一个、重写之后的声明长度（截断取旧净荷的前这么多字节，变长在旧净荷之后补零到这么长）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailDataUnitRewrite {
    pub unit: DataUnitIndexInFile,
    pub declared_length_in_bytes: u64,
}

/// 截断意图那次要不要重写尾单元（K3）：截到 N 之后最后一个单元是 k = ⌈N ÷ 净荷⌉.max(1) − 1，它存在、而它在旧 size 里的声明长度
/// 不等于 N − k × 净荷（截到 0 时 k = 0、留一个声明长度 0 的单元）时重写它。k 不存在（N 落在尾部的洞里）不重写：洞前那个单元是满的。
#[must_use]
pub fn tail_data_unit_rewrite_of_a_truncation(
    data_units_existing: u64,
    size_before_in_bytes: u64,
    target_size_in_bytes: u64,
) -> Option<TailDataUnitRewrite> {
    let unit = DataUnitIndexInFile(data_units_a_size_needs(target_size_in_bytes) - 1);
    if unit.0 >= data_units_existing {
        return None;
    }
    let declared_length_in_bytes = declared_length_within_a_size(unit, target_size_in_bytes);
    (declared_length_within_a_size(unit, size_before_in_bytes) != declared_length_in_bytes)
        .then_some(TailDataUnitRewrite {
            unit,
            declared_length_in_bytes,
        })
}

/// 截断意图那次 inode 记录的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2 与第六节用户定案「放了单元才写、
/// 整个替换」）：这次放单元之前存在的单元数多于 ⌈N ÷ 净荷⌉.max(1)（这次截断要放掉单元）时写成（这次 txg, N），否则照抄上一版
/// （截到洞里、截在最后一个单元之内都不放单元）。各批与最后一批不碰记录，变长与 N = S 照抄，整份写清零，都不经这里。
#[must_use]
pub fn truncation_field_after_the_truncation_intent(
    truncation_field_before: InodeRecordTruncationField,
    data_units_existing_before_the_intent: u64,
    target_size_in_bytes: u64,
    txg_of_the_intent: CheckpointTxg,
) -> InodeRecordTruncationField {
    if data_units_existing_before_the_intent > data_units_a_size_needs(target_size_in_bytes) {
        InodeRecordTruncationField::written_by_the_truncation_intent_of(
            txg_of_the_intent,
            target_size_in_bytes,
        )
    } else {
        truncation_field_before
    }
}

/// 变长那次要不要重写尾单元（K5）：最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写到那个长度；之后到 N 是洞、不写单元。
/// 一个单元都不存在（上段叶标签 0：整个文件都是洞，K5、S1）时没有单元可补：不重写、不加单元，变长只改记录、标签留 0
/// （判决 `research/prompts/m4-validate-code-r1-main-verification.md` 用户定案「丑：写者接得住这一形」）。
#[must_use]
pub fn tail_data_unit_rewrite_of_an_extension(
    data_units_existing: u64,
    size_before_in_bytes: u64,
    target_size_in_bytes: u64,
) -> Option<TailDataUnitRewrite> {
    let unit = DataUnitIndexInFile(data_units_existing.checked_sub(1)?);
    let declared_length_in_bytes = declared_length_within_a_size(unit, target_size_in_bytes);
    (declared_length_within_a_size(unit, size_before_in_bytes) < declared_length_in_bytes)
        .then_some(TailDataUnitRewrite {
            unit,
            declared_length_in_bytes,
        })
}

/// 截断意图之后的一步（K2、K11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncationStep {
    /// 不是最后一批：从尾部放掉一批，留下前 `data_units_kept` 个；不碰 inode 记录，截断条目留着。
    Batch { data_units_kept: u64 },
    /// 最后一批：放掉剩下 size 之外的全部单元（可以 0 个），留下前 `data_units_kept` 个，同一次发布删截断条目。
    FinalBatch { data_units_kept: u64 },
}

/// 现行那一版这个文件挂着 `data_units_existing` 个单元、截断目标是 `target_size_in_bytes` 时下一步是哪一批：size 之外还挂着的单元
/// 多于一批 ⇒ 从尾部放一批；不多于一批 ⇒ 最后一批把它们全部放掉（可以 0 个：N 落在洞里、或截在单元边界上什么都不挂）。
#[must_use]
pub fn next_truncation_step(
    data_units_existing: u64,
    target_size_in_bytes: u64,
    batch_size: UnlinkBatchSize,
) -> TruncationStep {
    let data_units_kept_at_the_end =
        data_units_existing.min(data_units_a_size_needs(target_size_in_bytes));
    let data_units_beyond_the_size = data_units_existing - data_units_kept_at_the_end;
    let data_units_per_batch = batch_size.data_units_per_batch();
    if data_units_beyond_the_size <= data_units_per_batch {
        return TruncationStep::FinalBatch {
            data_units_kept: data_units_kept_at_the_end,
        };
    }
    TruncationStep::Batch {
        data_units_kept: data_units_existing - data_units_per_batch,
    }
}

/// 从挂着 `data_units_existing` 个单元起走到底的每一步，按先后：最后一步恒是 [`TruncationStep::FinalBatch`]。
#[must_use]
pub fn every_truncation_step_from(
    data_units_existing: u64,
    target_size_in_bytes: u64,
    batch_size: UnlinkBatchSize,
) -> Vec<TruncationStep> {
    let mut steps = Vec::new();
    let mut existing = data_units_existing;
    // 迭代上界：每一批至少放一个单元（批大小至少 1），⌈挂着的 ÷ 批大小⌉ 轮之内走到最后一批。
    loop {
        let step = next_truncation_step(existing, target_size_in_bytes, batch_size);
        steps.push(step);
        match step {
            TruncationStep::Batch { data_units_kept } => {
                assert!(data_units_kept < existing, "每一批至少放一个单元");
                existing = data_units_kept;
            }
            TruncationStep::FinalBatch { .. } => return steps,
        }
    }
}

/// 一次发布重写的那一段数据单元：单元号 `[first, end_exclusive)`，空段是一个都不重写。截断意图与变长重写的尾单元是一个单元的一段
/// （K3、K5）；按偏移的部分写从偏移 o 所在的单元起到写之后最后一个单元（判决 `research/prompts/m4-partial-write-r1-main-verification.md`
/// 第二节 R4），段里单元号不小于写之前单元数的那几个是这次新加的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitsRewrittenInOnePublish {
    pub first: u64,
    pub end_exclusive: u64,
}

impl DataUnitsRewrittenInOnePublish {
    /// 一个都不重写。
    pub const NONE: Self = DataUnitsRewrittenInOnePublish {
        first: 0,
        end_exclusive: 0,
    };

    /// 只重写 `unit` 那一个。
    #[must_use]
    pub const fn of_one_unit(unit: DataUnitIndexInFile) -> Self {
        DataUnitsRewrittenInOnePublish {
            first: unit.0,
            end_exclusive: unit.0 + 1,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.end_exclusive <= self.first
    }

    #[must_use]
    pub const fn contains(self, unit: DataUnitIndexInFile) -> bool {
        self.first <= unit.0 && unit.0 < self.end_exclusive
    }

    /// 这一段与单元号 `[first_data_unit, last_data_unit]`（含两端）有没有交。
    #[must_use]
    pub const fn meets(self, first_data_unit: u64, last_data_unit: u64) -> bool {
        !self.is_empty() && self.first <= last_data_unit && first_data_unit < self.end_exclusive
    }

    /// 这一段里的单元号，升序。
    #[must_use]
    pub const fn units(self) -> std::ops::Range<u64> {
        self.first..self.end_exclusive
    }
}

/// 一个文件改 size（或按偏移部分写）之前与之后的样子（[`plan_the_extent_tree_after_a_size_change`] 与 [`plan_the_lower_segment_after_a_size_change`]
/// 的输入）：单元数、这次重写哪一段单元、上段叶条目声明的尾部。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeChangeOfTheFile {
    pub data_units_before: u64,
    pub data_units_after: u64,
    pub rewritten_data_units: DataUnitsRewrittenInOnePublish,
    pub tail_before: TailOfAFile,
    pub tail_after: TailOfAFile,
}

impl SizeChangeOfTheFile {
    /// 这个文件在上段叶里那一条这次变不变（变了那一串到根重写）：单元数变了、有单元重写（下段叶到根或内联的那一个指针变了）、
    /// 或标签声明的尾部变了（有没有洞）。都没变（N = S、标签 0 上的改 size、截在洞里而什么都不放的最后一批）时那一条照抄。
    #[must_use]
    pub fn rewrites_the_upper_leaf_entry(self) -> bool {
        !self.rewritten_data_units.is_empty()
            || self.data_units_after != self.data_units_before
            || self.tail_before != self.tail_after
    }
}

/// 一个文件的下段在改 size（截断三步、变长、只改记录）之后的样子（[`plan_the_lower_segment_after_a_size_change`] 的结果）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LowerSegmentAfterASizeChange {
    /// 这次之后的节点与来历，bump 次序（先叶后根）；只剩一个单元（内联）或一个都没有时为空。
    pub nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    /// 上一版里有、这次重写的位置（同名后继），按上一版的 bump 次序。
    pub previous_nodes_rewritten: Vec<ExtentLowerNodePosition>,
    /// 上一版里有、这次之后没有的位置（被截掉那一段里的节点、降成内联时的整段），按上一版的 bump 次序。
    pub previous_nodes_dropped: Vec<ExtentLowerNodePosition>,
}

/// 截断（意图、各批、最后一批）与变长、只改记录那几次发布之后一个文件的下段（K3：截断的 extent 计划自己写，不调
/// `unlink::plan_the_extent_tree_after_a_deletion_batch`），按偏移的部分写（判决 m4-partial-write-r1 第二节 R4、R5）也走这一份。
/// 入参是这个文件自己上一版下段的节点位置（bump 次序）：第一个文件与第一个文件之外的
/// 文件各交自己那一份（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3「抽出下段计划」）。
/// 下段按这次之后的单元数排，一个节点照抄的条件是它罩着的、存在的单元两版一样（罩的单元全在两版都有的那一段里，或这次单元数没变）**而且**不罩着
/// 这次重写的那一段里的任何一个单元——罩着被重写单元的下段叶到根恒重写，留下的单元数正在叶边界上（144 的倍数）时也一样（K9 叶边界对任意 inode）；
/// 被截掉那一段里的节点换下、没有同名后继。只剩一个单元时不建下段（内联）。部分写加单元时，新加的单元都在重写的那一段里。
///
/// 之前一个单元都没有（上段叶标签 0：整个文件都是洞，K5、S1）的：截短、N = S、变长都不加单元、不重写单元，下段为空；部分写加的单元整段重写。
///
/// # Panics
/// 之前有单元而之后是 0 个：截断留至少一个单元。这次之后的单元数多于之前的、而多出来的单元不全在重写的那一段里：只有部分写加单元，它重写到最后一个。
/// 照抄的位置不在 `previous_lower_nodes` 里：罩的存在单元两版一样的节点，上一版同一个位置上必有。
#[must_use]
pub fn plan_the_lower_segment_after_a_size_change(
    previous_lower_nodes: &[ExtentLowerNodePosition],
    change: SizeChangeOfTheFile,
) -> LowerSegmentAfterASizeChange {
    let SizeChangeOfTheFile {
        data_units_before,
        data_units_after,
        rewritten_data_units,
        ..
    } = change;
    let keeps_at_least_one_unit_or_had_none = data_units_after >= 1 || data_units_before == 0;
    assert!(
        keeps_at_least_one_unit_or_had_none,
        "截断留至少一个单元（一个单元都没有的整文件洞照旧 0 个）"
    );
    assert!(
        data_units_after <= data_units_before
            || (rewritten_data_units.first <= data_units_before
                && rewritten_data_units.end_exclusive == data_units_after),
        "加单元的只有部分写：新加的单元都在重写的那一段里、重写到最后一个"
    );
    let covers_the_rewritten_unit = |position: &ExtentLowerNodePosition| {
        let (first_data_unit, last_data_unit) = position.data_unit_range();
        rewritten_data_units.meets(first_data_unit, last_data_unit)
    };
    let covers_the_same_existing_units = |position: &ExtentLowerNodePosition| {
        let (_, last_data_unit) = position.data_unit_range();
        last_data_unit < data_units_after.min(data_units_before)
            || data_units_after == data_units_before
    };
    let nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)> =
        lower_segment_nodes_of_a_file_without_holes(data_units_after)
            .into_iter()
            .map(|position| {
                if covers_the_same_existing_units(&position)
                    && !covers_the_rewritten_unit(&position)
                {
                    assert!(
                        previous_lower_nodes.contains(&position),
                        "罩的存在单元两版一样的节点，上一版（单元不更少）同一个位置上必有"
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
    let origin_after_this_publish = |position: &ExtentLowerNodePosition| {
        nodes
            .iter()
            .find(|(position_after, _)| position_after == position)
            .map(|(_, origin)| *origin)
    };
    let previous_nodes_rewritten = previous_lower_nodes
        .iter()
        .filter(|position| {
            origin_after_this_publish(position) == Some(ExtentTreeNodeOrigin::RewrittenThisPublish)
        })
        .copied()
        .collect();
    let previous_nodes_dropped = previous_lower_nodes
        .iter()
        .filter(|position| origin_after_this_publish(position).is_none())
        .copied()
        .collect();
    LowerSegmentAfterASizeChange {
        nodes,
        previous_nodes_rewritten,
        previous_nodes_dropped,
    }
}

/// 截断（意图、各批、最后一批）与变长、只改记录那几次发布之后第一个文件那棵 extent 树：下段照 [`plan_the_lower_segment_after_a_size_change`]
/// （入参是第一个文件上一版的下段），上段第一个文件那一串在那一条变了时（[`SizeChangeOfTheFile::rewrites_the_upper_leaf_entry`]）重写，
/// 否则整棵照抄、什么都不换下。上段有条目的号集合不变（第一个文件改 size 时那条记录一直在、条目一直在）：第一个文件之外的文件改 size
/// 不经这里，它的上段走统一计划（`transaction::extent_tree_plan_with_the_unified_upper_segment`，号集合可以多出它那一条粘住的标签 0）。
///
/// # Panics
/// 同 [`plan_the_lower_segment_after_a_size_change`]。
#[must_use]
pub fn plan_the_extent_tree_after_a_size_change(
    previous: &ExtentTreeVersion,
    inode: u64,
    change: SizeChangeOfTheFile,
) -> ExtentTreePlan {
    let previous_lower_positions: Vec<ExtentLowerNodePosition> = previous
        .lower_nodes
        .iter()
        .map(|(position, _)| *position)
        .collect();
    let lower = plan_the_lower_segment_after_a_size_change(&previous_lower_positions, change);
    let upper_path_is_rewritten = change.rewrites_the_upper_leaf_entry();
    let upper_root_level = previous.upper_root().level;
    let path_of_the_file = upper_path_of_inode(inode, upper_root_level);
    let upper_nodes = previous
        .upper_nodes
        .iter()
        .map(|(position, _)| {
            let origin = if upper_path_is_rewritten && path_of_the_file.contains(position) {
                ExtentTreeNodeOrigin::RewrittenThisPublish
            } else {
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            };
            (*position, origin)
        })
        .collect();
    let carried_lower: Vec<ExtentLowerNodePosition> = lower
        .nodes
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
                .filter(|(position, _)| {
                    upper_path_is_rewritten && path_of_the_file.contains(position)
                })
                .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
        )
        .collect();
    ExtentTreePlan {
        lower_nodes: lower.nodes,
        upper_nodes,
        replaced_previous_roles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extent_tree::ExtentUpperNodePosition;
    use crate::pointer::NodePointer;
    use crate::transaction::FIRST_INODE_NUMBER;
    use crate::unlink::UnlinkBatchDataUnitsCappedForTests;

    const PAYLOAD_IN_BYTES: u64 = 32634;

    fn capped(data_units: u64) -> UnlinkBatchSize {
        UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(data_units).expect("1..=144"),
        )
    }

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

    /// 判决 m4-trunc-r2 R2 与第六节用户定案：截断意图那次存在的单元数多于 ⌈N ÷ 净荷⌉.max(1) 才写（这次 txg, N）、整个替换，否则照抄上一版。
    /// 期望值按条款手算：三个单元截到单元 0 中间（要一个，放两个）写；截到 0（要一个）写成 T ≠ 0、N_w = 0；截在最后一个单元之内（三个要三个）、
    /// 截到洞里（一个单元的文件变长到三个净荷再截到两个净荷：要两个，有一个）照抄；恰好多一个写、恰好相等照抄（`>` 的边界）；
    /// 上一版写过的字段在照抄时原样留着，写的时候整个换掉、不取 min。
    #[test]
    fn the_truncation_intent_writes_the_field_only_when_it_releases_data_units_and_otherwise_carries_it(
    ) {
        let txg_of_the_intent = CheckpointTxg(9);
        let written_before = InodeRecordTruncationField::written_by_the_truncation_intent_of(
            CheckpointTxg(5),
            3 * PAYLOAD_IN_BYTES,
        );
        let written_now = |size| {
            InodeRecordTruncationField::written_by_the_truncation_intent_of(txg_of_the_intent, size)
        };
        let cases: [(
            InodeRecordTruncationField,
            u64,
            u64,
            InodeRecordTruncationField,
            &str,
        ); 7] = [
            (
                InodeRecordTruncationField::NOT_WRITTEN,
                3,
                100,
                written_now(100),
                "三个单元截到单元 0 中间：放两个，写",
            ),
            (
                InodeRecordTruncationField::NOT_WRITTEN,
                3,
                0,
                written_now(0),
                "截到 0：放两个，写成 T = 9、N_w = 0",
            ),
            (
                InodeRecordTruncationField::NOT_WRITTEN,
                3,
                2 * PAYLOAD_IN_BYTES + 1,
                InodeRecordTruncationField::NOT_WRITTEN,
                "截在最后一个单元之内：不放单元，照抄",
            ),
            (
                InodeRecordTruncationField::NOT_WRITTEN,
                1,
                2 * PAYLOAD_IN_BYTES,
                InodeRecordTruncationField::NOT_WRITTEN,
                "截到洞里：要两个、有一个，照抄",
            ),
            (
                InodeRecordTruncationField::NOT_WRITTEN,
                3,
                2 * PAYLOAD_IN_BYTES,
                written_now(2 * PAYLOAD_IN_BYTES),
                "三个截到两个净荷整：恰好多一个，写",
            ),
            (
                written_before,
                2,
                2 * PAYLOAD_IN_BYTES,
                written_before,
                "两个截到两个净荷整：恰好相等，照抄上一版写过的那一份",
            ),
            (
                written_before,
                3,
                100,
                written_now(100),
                "上一版写过：这次放单元，整个换成这次的（txg, N），不取 min",
            ),
        ];
        for (before, data_units_existing, target_size, expected, what) in cases {
            assert_eq!(
                truncation_field_after_the_truncation_intent(
                    before,
                    data_units_existing,
                    target_size,
                    txg_of_the_intent
                ),
                expected,
                "{what}"
            );
        }
    }

    fn lower(level: u8, index: u64) -> ExtentLowerNodePosition {
        ExtentLowerNodePosition { level, index }
    }

    /// 截断意图那次的计划：单元数不变、只重写尾单元 k（这里 k 由调用方给），尾部没有洞。
    fn intent_rewriting(data_units: u64, unit: u64) -> ExtentTreePlan {
        plan_the_extent_tree_after_a_size_change(
            &extent_tree_of_a_file_without_holes(data_units),
            FIRST_INODE_NUMBER,
            SizeChangeOfTheFile {
                data_units_before: data_units,
                data_units_after: data_units,
                rewritten_data_units: DataUnitsRewrittenInOnePublish::of_one_unit(
                    DataUnitIndexInFile(unit),
                ),
                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
            },
        )
    }

    /// 三个单元截到单元 0 中间（k = 0）：重写单元 0，声明长度 N；截到单元 1 中间（k = 1）：重写单元 1，声明长度 N − 净荷；
    /// 截在单元边界上（N = 净荷）：单元 0 原来就满，不重写；截到 0：留单元 0、声明长度 0。
    #[test]
    fn the_truncation_intent_rewrites_the_unit_holding_the_new_end_unless_it_already_ends_there() {
        let size_before = 2 * PAYLOAD_IN_BYTES + PAYLOAD_IN_BYTES / 2;
        assert_eq!(
            tail_data_unit_rewrite_of_a_truncation(3, size_before, 100),
            Some(TailDataUnitRewrite {
                unit: DataUnitIndexInFile(0),
                declared_length_in_bytes: 100
            })
        );
        assert_eq!(
            tail_data_unit_rewrite_of_a_truncation(3, size_before, PAYLOAD_IN_BYTES + 7),
            Some(TailDataUnitRewrite {
                unit: DataUnitIndexInFile(1),
                declared_length_in_bytes: 7
            })
        );
        assert_eq!(
            tail_data_unit_rewrite_of_a_truncation(3, size_before, PAYLOAD_IN_BYTES),
            None,
            "截在单元边界上：单元 0 原来就装满一个净荷"
        );
        assert_eq!(
            tail_data_unit_rewrite_of_a_truncation(3, size_before, 0),
            Some(TailDataUnitRewrite {
                unit: DataUnitIndexInFile(0),
                declared_length_in_bytes: 0
            }),
            "截到 0 留一个声明长度 0 的单元"
        );
        assert_eq!(
            tail_data_unit_rewrite_of_a_truncation(
                1,
                3 * PAYLOAD_IN_BYTES,
                2 * PAYLOAD_IN_BYTES - 1
            ),
            None,
            "截到洞里：单元 1 不存在，不重写"
        );
    }

    /// 变长：最后一个存在的单元补零到 min(N − 起点, 净荷)；原来就满的不重写。
    #[test]
    fn the_extension_rewrites_the_last_existing_unit_only_when_it_is_shorter_than_the_new_size_reaches(
    ) {
        assert_eq!(
            tail_data_unit_rewrite_of_an_extension(1, 0, 10),
            Some(TailDataUnitRewrite {
                unit: DataUnitIndexInFile(0),
                declared_length_in_bytes: 10
            }),
            "size 0 的文件变长到单元内"
        );
        assert_eq!(
            tail_data_unit_rewrite_of_an_extension(1, 10, 3 * PAYLOAD_IN_BYTES),
            Some(TailDataUnitRewrite {
                unit: DataUnitIndexInFile(0),
                declared_length_in_bytes: PAYLOAD_IN_BYTES
            }),
            "跨单元：单元 0 补满，之后是洞"
        );
        assert_eq!(
            tail_data_unit_rewrite_of_an_extension(2, 2 * PAYLOAD_IN_BYTES, 5 * PAYLOAD_IN_BYTES),
            None,
            "最后一个单元原来就满"
        );
    }

    /// 批大小压成 1：挂着五个、截到一个半单元（留两个）⇒ 放到四个、放到三个、最后一批放到两个；截到洞里或边界上什么都不挂 ⇒ 只有一个放 0 个的最后一批。
    #[test]
    fn truncation_batches_release_from_the_tail_and_the_final_batch_may_release_nothing() {
        assert_eq!(
            every_truncation_step_from(5, PAYLOAD_IN_BYTES + 1, capped(1)),
            vec![
                TruncationStep::Batch { data_units_kept: 4 },
                TruncationStep::Batch { data_units_kept: 3 },
                TruncationStep::FinalBatch { data_units_kept: 2 },
            ]
        );
        assert_eq!(
            every_truncation_step_from(2, 2 * PAYLOAD_IN_BYTES, capped(1)),
            vec![TruncationStep::FinalBatch { data_units_kept: 2 }],
            "截在最后一个单元末尾之后（落在洞里或边界上）：放 0 个"
        );
        assert_eq!(
            every_truncation_step_from(3, 0, UnlinkBatchSize::OneExtentLowerLeaf),
            vec![TruncationStep::FinalBatch { data_units_kept: 1 }],
            "截到 0 留一个单元"
        );
    }

    /// k = 0：单元 0 在下段第 0 片叶里，那片叶与下段根、上段那一串重写；别的叶照抄。
    #[test]
    fn rewriting_unit_zero_rewrites_the_first_lower_leaf_and_its_ancestors() {
        let plan = intent_rewriting(300, 0);
        assert_eq!(
            plan.lower_nodes,
            vec![
                (lower(0, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (
                    lower(0, 1),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (
                    lower(0, 2),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert_eq!(
            plan.rewritten_roles(),
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentLowerNode(lower(1, 0)),
                TransactionUnit::ExtentRoot,
            ]
        );
    }

    /// k = 1 落在只有一片下段叶的三单元文件里：那片叶（兼下段根）与上段那一串重写。
    #[test]
    fn rewriting_unit_one_of_a_three_unit_file_rewrites_its_only_lower_leaf() {
        let plan = intent_rewriting(3, 1);
        assert_eq!(
            plan.rewritten_roles(),
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentRoot
            ]
        );
        assert_eq!(
            plan.replaced_previous_roles,
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentRoot
            ]
        );
    }

    /// 留下 144 个单元、单元 k = 143 正在第 0 片叶的末尾（叶边界上）：意图那次重写第 0 片叶与根；之后那一批截掉第 1 片叶时
    /// 第 0 片叶照抄（单元 143 意图那次已经重写、那片叶已经指着它），它成了下段根。
    #[test]
    fn keeping_one_hundred_and_forty_four_units_rewrites_the_leaf_holding_unit_one_hundred_and_forty_three(
    ) {
        let intent = intent_rewriting(150, 143);
        assert_eq!(
            intent.rewritten_roles(),
            vec![
                TransactionUnit::ExtentLowerNode(lower(0, 0)),
                TransactionUnit::ExtentLowerNode(lower(1, 0)),
                TransactionUnit::ExtentRoot,
            ],
            "罩着被重写单元 k 的叶到根恒并进重写集"
        );
        let final_batch = plan_the_extent_tree_after_a_size_change(
            &extent_tree_of_a_file_without_holes(150),
            FIRST_INODE_NUMBER,
            SizeChangeOfTheFile {
                data_units_before: 150,
                data_units_after: 144,
                rewritten_data_units: DataUnitsRewrittenInOnePublish::NONE,
                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
            },
        );
        assert_eq!(
            final_batch.lower_nodes,
            vec![(
                lower(0, 0),
                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
            )]
        );
        assert_eq!(
            final_batch.rewritten_roles(),
            vec![TransactionUnit::ExtentRoot]
        );
    }

    /// 留下 288 个单元、k = 287 在第 1 片叶的末尾：意图那次重写第 1 片叶与下段根，第 0、2 片叶照抄。
    #[test]
    fn keeping_two_hundred_and_eighty_eight_units_rewrites_the_leaf_holding_unit_two_hundred_and_eighty_seven(
    ) {
        let plan = intent_rewriting(300, 287);
        assert_eq!(
            plan.lower_nodes,
            vec![
                (
                    lower(0, 0),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (lower(0, 1), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (
                    lower(0, 2),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
    }

    /// 没有单元重写、单元数与尾部都没变（最后一批放 0 个）：整棵照抄、什么都不换下。
    #[test]
    fn a_final_batch_releasing_nothing_carries_the_whole_extent_tree() {
        let plan = plan_the_extent_tree_after_a_size_change(
            &extent_tree_of_a_file_without_holes(3),
            FIRST_INODE_NUMBER,
            SizeChangeOfTheFile {
                data_units_before: 3,
                data_units_after: 3,
                rewritten_data_units: DataUnitsRewrittenInOnePublish::NONE,
                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
            },
        );
        assert!(plan.rewritten_roles().is_empty());
        assert!(plan.replaced_previous_roles.is_empty());
    }

    /// 部分写加单元（判决 m4-partial-write-r1 第二节 R4、R5）：144 个单元正好一片叶，写到第 145 个单元（单元 143 满、不重写）——
    /// 第 0 片叶罩的单元两版一样、照抄，第 1 片新叶与新的下段根重写，上一版那片叶（兼下段根）不被换下；从单元 100 起重写到 145 个——
    /// 第 0 片叶罩着重写的单元，叶与根都重写、上一版那一片换下。
    #[test]
    fn a_partial_write_adding_units_carries_only_the_leaves_below_the_first_rewritten_unit() {
        let past_a_full_leaf = plan_the_lower_segment_after_a_size_change(
            &[lower(0, 0)],
            SizeChangeOfTheFile {
                data_units_before: 144,
                data_units_after: 145,
                rewritten_data_units: DataUnitsRewrittenInOnePublish {
                    first: 144,
                    end_exclusive: 145,
                },
                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
            },
        );
        assert_eq!(
            past_a_full_leaf.nodes,
            vec![
                (
                    lower(0, 0),
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
                ),
                (lower(0, 1), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert!(past_a_full_leaf.previous_nodes_rewritten.is_empty());
        assert!(past_a_full_leaf.previous_nodes_dropped.is_empty());
        let from_unit_one_hundred = plan_the_lower_segment_after_a_size_change(
            &[lower(0, 0)],
            SizeChangeOfTheFile {
                data_units_before: 144,
                data_units_after: 145,
                rewritten_data_units: DataUnitsRewrittenInOnePublish {
                    first: 100,
                    end_exclusive: 145,
                },
                tail_before: TailOfAFile::NoHoleAfterTheLastDataUnit,
                tail_after: TailOfAFile::NoHoleAfterTheLastDataUnit,
            },
        );
        assert_eq!(
            from_unit_one_hundred.nodes,
            vec![
                (lower(0, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (lower(0, 1), ExtentTreeNodeOrigin::RewrittenThisPublish),
                (lower(1, 0), ExtentTreeNodeOrigin::RewrittenThisPublish),
            ]
        );
        assert_eq!(
            from_unit_one_hundred.previous_nodes_rewritten,
            vec![lower(0, 0)]
        );
    }

    /// K12：⌈N ÷ 净荷⌉ × 净荷装得进 u64 的上限；N 走哪一条只看 N 与 S。
    #[test]
    fn the_target_size_limit_and_the_three_branches_of_a_size_change() {
        assert!(target_size_rounded_up_to_a_whole_data_unit_fits(0));
        let largest_whole_units = u64::MAX / PAYLOAD_IN_BYTES * PAYLOAD_IN_BYTES;
        assert!(target_size_rounded_up_to_a_whole_data_unit_fits(
            largest_whole_units
        ));
        assert!(!target_size_rounded_up_to_a_whole_data_unit_fits(
            largest_whole_units + 1
        ));
        assert_eq!(
            SizeChangeOfARequest::of(5, 5),
            SizeChangeOfARequest::SizeUnchanged
        );
        assert_eq!(
            SizeChangeOfARequest::of(5, 6),
            SizeChangeOfARequest::Extension
        );
        assert_eq!(
            SizeChangeOfARequest::of(5, 4),
            SizeChangeOfARequest::Truncation
        );
    }
}
