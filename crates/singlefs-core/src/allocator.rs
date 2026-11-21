//! 分配器（里程碑步 2 / 步 3）：内存里一张单元区的空闲图，两条落点政策，分配记录与记账增量在提交时增量维护。
//!
//! 用户数据落点（D3（空间分配） 已定项 8 / 已定项 10）：已选设备内起点槽号最小、起点 32768 对齐（偶数槽）、跨度内两槽都空、
//! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。取法不从单元区起点逐个候选看：每块盘增量维护「有两槽都没挡的偶数起点槽对」的段号集合，
//! 按段号升序走这个集合、跳过这次挂载开过的聚簇段与 `R` 那一段（按段起点查），落进第一段里最低的那一对（[`DeviceFreeMap::lowest_user_data_slot_outside_the_region`]）。
//! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
//! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：先取这次发布分配记录树重写集里的叶（[`RewriteSetLeaves`]，每块盘都重写的那几片）里
//! 每块盘都空的最低落点，那些叶里一个都没有再取每块盘都空的槽最多的那片叶（一样多取叶序号小的）里的最低落点；两步都排除 `R`，
//! 与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住）。每片叶没挡的槽数随三种挡位增量维护，取空槽最多的叶不扫单元区的槽。
//! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
//! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
//! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
//! 逐段的已用槽数、全空聚簇段数（逐设备，`DeviceFreeMap::empty_segments`）——这几样在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。

use singlefs_format::{
    ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS,
    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
};

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use crate::admission::SpaceAdmission;
use crate::bytes::ByteWriter;
use crate::compaction_plan::CompactionSwitchesAndTheExcludedRegion;
use crate::instance_table::InstanceTableRecords;
use crate::journal::slot_after_the_journal_ring;
use crate::logged_operations::CompactionRegion;
use crate::pointer::LocationEntry;
use crate::recovery::{PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing};
use crate::root_record::RootRecord;
use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use crate::transaction::FrozenPublish;
use crate::unlink::DeletionSwitches;

mod allocation_record;
mod device_free_map;
mod pool_allocator;
#[cfg(test)]
mod pools_built_for_tests;
mod root_ring_occupancy;

pub use self::allocation_record::*;
pub use self::device_free_map::*;
pub use self::pool_allocator::*;
#[cfg(test)]
use self::pools_built_for_tests::*;
pub use self::root_ring_occupancy::*;

/// 一个单元占几个槽：数据单元与打包容器 2，索引节点 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitFootprint {
    TwoSlotsAligned,
    OneSlot,
}

impl UnitFootprint {
    const fn slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }

    /// 起点对齐的步长（槽）：两槽的单元起点 32768 对齐（D18（块里携带什么信息） 已定项 9 按单元大小分档），一槽的单元每个槽都能起。
    const fn alignment_in_slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }
}

/// 开放段没有或装不下时，一块盘按自己的空闲图给提交内生块的去处。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitGeneratedDeviceAnswer {
    /// 这块盘上最低的全空段的起点：在这里开新段（D3（空间分配） 已定项 10 ①）。
    OpenEmptySegment(SlotNumber),
    /// 这块盘上没有全空段、要回落（D3（空间分配） 已定项 8 第 2 条），带的是回落的落点。各盘只按自己的空闲图答时，它是这块盘单元区里
    /// 起点最低、整跨度没挡、不在活 `R` 里的落点（[`DeviceFreeMap::lowest_commit_generated_fallback_slot_outside_the_region`]）；
    /// 每块盘都答回落、池里按回落政策取到一个每块盘都空的落点时（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），
    /// 每块盘的答案都换成池里取的那一个。
    FallBack(SlotNumber),
}

/// 分配记录树一片叶的序号：盘上第 k 片叶罩槽 `[k × W, (k + 1) × W)`（W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]），即槽号 ÷ W。
/// 与槽号、槽数同为 `u64` 而含义不同，各用各的类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocationRecordTreeLeafIndex(pub u64);

impl AllocationRecordTreeLeafIndex {
    /// 罩着 `slot` 的那片叶。
    #[must_use]
    pub const fn of_slot(slot: SlotNumber) -> Self {
        Self(slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS)
    }

    /// 这片叶罩的第一个槽。
    #[must_use]
    pub const fn first_slot(self) -> SlotNumber {
        SlotNumber(self.0 * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
    }

    /// 这片叶罩的最后一个槽的下一个槽（下一片叶的第一个槽）。
    #[must_use]
    pub const fn past_the_last_slot(self) -> SlotNumber {
        SlotNumber((self.0 + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
    }
}

/// 一片分配记录树叶里的槽数（没挡的槽数、每块盘都空的槽数），至多 [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]；是槽数，不是槽号，也不是槽对数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotCountInALeaf(pub u64);

/// 这次发布分配记录树重写集里的叶（回落先取的去处，D3（空间分配） 已定项 8 第 2 条）：叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，
/// 只含池里每块盘上都在重写集里的叶——提交内生块的落点各盘同槽，落在这样的叶里，每块盘上都不多弄脏一片叶。
/// 空集 = 没有重写集可先取，回落直接取空槽最多的叶。发布路径按这次发布的重写角色算（演练与真发读同一串角色，得同一个集合）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteSetLeaves {
    leaf_indices: BTreeSet<AllocationRecordTreeLeafIndex>,
}

impl RewriteSetLeaves {
    /// 空集：没有重写集可先取（不在发布路径上取落点的调用方，与重写集里还没有叶的那一轮）。
    #[must_use]
    pub const fn none() -> Self {
        Self {
            leaf_indices: BTreeSet::new(),
        }
    }

    /// 每块盘各自重写集里的叶序号（`leaf_indices_per_device` 一块盘一项）取交集：一片叶只有在池里每块盘上都重写，才算数。
    #[must_use]
    pub fn present_on_every_device(
        leaf_indices_per_device: &[BTreeSet<AllocationRecordTreeLeafIndex>],
    ) -> Self {
        let Some((leaf_indices_of_one_device, leaf_indices_of_the_other_devices)) =
            leaf_indices_per_device.split_first()
        else {
            return Self::none();
        };
        Self {
            leaf_indices: leaf_indices_of_one_device
                .iter()
                .copied()
                .filter(|leaf_index| {
                    leaf_indices_of_the_other_devices
                        .iter()
                        .all(|leaf_indices| leaf_indices.contains(leaf_index))
                })
                .collect(),
        }
    }

    /// 集合里的叶序号，升序。
    #[must_use]
    pub fn leaf_indices(&self) -> &BTreeSet<AllocationRecordTreeLeafIndex> {
        &self.leaf_indices
    }
}

/// 分配器拒绝一次分配的原因。拒绝发生在动任何状态之前：记录、位图、计数、开放段与 bump 游标都照旧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementRefusal {
    /// 每块盘上都没有合政策的落点。
    NoFreeSlotOnAnyDevice,
    /// 有的盘上没有合政策的落点、别的盘上还有（盘不等大时小盘先满）：第一版的设备集合就是池里全部的盘，
    /// 小盘写满之后「当时可写的设备集合」怎么选是 D2（RAID 条带策略） 已定项 2 ⚠️ 那一半，没有条款；第一版不支持。
    SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec<DeviceIdentity> },
    /// 各盘按自己的空闲图取出的用户数据落点不同：D3（空间分配） 已定项 8 要各盘各取、D2（RAID 条带策略） 已定项 10 每个副本一条位置条目，
    /// 而 `Placement` 两盘同槽、发布路径的位置条目只带一个槽号；第一版不支持。
    UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
        slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
    },
    /// 开放段装不下之后各盘给的去处不同，而不是每块盘都答回落（一块开段一块回落，或开的段不同）：各盘上的聚簇段要不要对齐是
    /// D3（空间分配） 已定项 8 待办 ①，没有条款；`Placement` 两盘同槽；第一版不支持。
    CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
        answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
    },
    /// 每块盘都答回落，池里却取不到一个每块盘都空的落点：每块盘自己的空闲图上还有回落的落点（`fallback_slot_per_device`，各盘自己答的那一个），
    /// 可没有一个槽号在每块盘上都空（D3（空间分配） 已定项 8 末段「池里取不到每块盘都空的落点时，照各盘自己的答案判一致，不一致拒绝」）。
    /// 与段对齐那一条分开报：这一条不是没有条款的池形状，是各盘的空槽错开了——腾出空间（推抬 F、整理、删）之后同一次写可能落得下。
    CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
        fallback_slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
    },
}

/// 开放段装不下之后各盘答得不一致（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）时报哪一种拒绝：
/// 每块盘都答回落（池里取不到每块盘都空的落点，各盘自己的回落落点又不同）报
/// [`PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`]，别的（有的盘开段）报段对齐那一条。
fn refusal_of_commit_generated_answers_that_differ(
    answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
) -> PlacementRefusal {
    let fallback_slot_per_device: Option<Vec<(DeviceIdentity, SlotNumber)>> = answer_per_device
        .iter()
        .map(|(device, answer)| match answer {
            CommitGeneratedDeviceAnswer::FallBack(slot) => Some((*device, *slot)),
            CommitGeneratedDeviceAnswer::OpenEmptySegment(_) => None,
        })
        .collect();
    match fallback_slot_per_device {
        Some(fallback_slot_per_device) => {
            PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
                fallback_slot_per_device,
            }
        }
        None => PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            answer_per_device,
        },
    }
}

/// 各盘各答一个之后合成的结论。
enum DeviceAgreement<Answer> {
    Agreed(Answer),
    NoAnswerOnAnyDevice,
    SomeDevicesWithoutAnswer(Vec<DeviceIdentity>),
    AnswersDiffer(Vec<(DeviceIdentity, Answer)>),
}

/// 每块盘的答案（`None` = 这块盘答不出）合成一个结论：全答不出、部分答不出、答得不同、一致。
fn agreement_across_devices<Answer: Copy + PartialEq>(
    answer_per_device: &[(DeviceIdentity, Option<Answer>)],
) -> DeviceAgreement<Answer> {
    let devices_without_answer: Vec<DeviceIdentity> = answer_per_device
        .iter()
        .filter(|(_, answer)| answer.is_none())
        .map(|(device, _)| *device)
        .collect();
    if devices_without_answer.len() == answer_per_device.len() {
        return DeviceAgreement::NoAnswerOnAnyDevice;
    }
    if !devices_without_answer.is_empty() {
        return DeviceAgreement::SomeDevicesWithoutAnswer(devices_without_answer);
    }
    let answers: Vec<(DeviceIdentity, Answer)> = answer_per_device
        .iter()
        .filter_map(|(device, answer)| answer.map(|present| (*device, present)))
        .collect();
    let first_answer = answers
        .first()
        .expect("上面两个判断之后每块盘都有答案、且至少一块盘")
        .1;
    if answers.iter().all(|(_, answer)| *answer == first_answer) {
        DeviceAgreement::Agreed(first_answer)
    } else {
        DeviceAgreement::AnswersDiffer(answers)
    }
}

impl PoolAllocator {
    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：把 `region` 那一段记成这次挂载开过的聚簇段
    /// （[`PoolAllocator::cluster_segments`] 那一份，提交内生块开一个全空段时记的同一个集合）。只往这一个集合里加，位图、记录、逐段计数与开放段的
    /// bump 游标一样都不动。产品路径一处都不调它。放锁的三段入口回锁签发时重判资格的「这次挂载开过的段」那一项（`mounted_session` 回锁签发那一支）
    /// 在产品路径上要锁外期间选中那一段的记录先全放掉、再被下一次提交内生块恰好开成新段才进得去，靠它进。
    pub fn note_a_cluster_segment_opened_in_this_mount_for_tests(
        &mut self,
        region: CompactionRegion,
    ) {
        self.cluster_segments.insert(region.start_slot());
    }

    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：把 `region` 那一段当开放段、bump 游标放到它的起点，
    /// 之后提交内生块（钉住的映射树节点、树表、实例表片也在其中）先 bump 进这一段里没挡的槽，这一段装不下了才照产品路径开新段或回落；
    /// 这一段不记进 [`PoolAllocator::cluster_segments`]（产品路径上开段时同时记进去，这里不记）。位图、记录与逐段计数只随之后真分配出的落点变。
    /// 产品路径一处都不调它。放锁的三段入口回锁签发时重判资格的「钉住的槽」那一项（`mounted_session` 回锁签发那一支）在产品路径上要
    /// 锁外期间一次发布的钉住单元落进选中那一段、而那一段又不是这次挂载开过的段，只有每块盘都没有全空段、提交内生块回落时才走得到，靠它进。
    pub fn bump_commit_generated_units_into_a_segment_not_noted_as_opened_in_this_mount_for_tests(
        &mut self,
        region: CompactionRegion,
    ) {
        self.open_segment = Some(region.start_slot());
        self.bump_cursor = region.start_slot().0;
    }
}
