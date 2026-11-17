//! 整理的选段与计划：三层里的后两层，都是纯函数——
//! 不读设备、不发写、不动分配器。`choose_region` 按记录给候选段排序、按扫描分类定资格（没扫过的先让调用方经
//! `crate::region_scan::scan_region` 扫），`plan_batch` 从扫描结果里取下一批要搬的（映射 key, 源落点）。**目的落点不在计划里**：由那一批发布时的分配器按 D3（空间分配） 已定项 8 现取
//!
//! 判据显式（`.claude/rules/fs-design.md` 五条硬要求第 1 条）：第一版只有 greedy 一种（[`RegionChoiceCriterion::Greedy`]），
//! 打分是 [`greedy_score`]；cost-benefit 与轮转留给 E10（老化：结构抗老化 vs 事后整理） 的臂。

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::Duration;

use singlefs_format::CLUSTER_SEGMENT_SLOTS;
use singlefs_replay_prefix::compaction_plan_proofs::{
    choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
    CompactionKernelPlannedBatch, CompactionKernelRegionChoice, CompactionKernelScannedAllocation,
    CompactionKernelSegment, CompactionKernelSegmentScan,
};

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use crate::admission::{BytesOnOneDevice, UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED};
use crate::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart};
use crate::background_compute::{BackgroundTaskAfterTheComputeError, ClusterSegmentEntryKey};
use crate::logged_operations::{CompactionIntent, CompactionRegion};
use crate::mounted_session::{BackgroundCompactionTrigger, CompactionRunReach};
use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};
use crate::root_ring::RootRingSlotsPerRegion;

/// 选段判据。封闭集合：加一种判据就加一个成员。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionChoiceCriterion {
    /// 活槽最少的段先。
    Greedy,
}

/// 一块盘的单元区：从哪个槽起、有几个槽。选段只在它的整段里挑（末尾凑不满 64 槽的那一截不是聚簇段，同 `lowest_empty_segment`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitAreaOfOneDevice {
    pub device: DeviceIdentity,
    pub start: UnitAreaStart,
    pub slots: u64,
}

/// greedy 的打分：段里的活槽数（未释放记录罩住的槽数），分越低越先选。
#[must_use]
pub const fn greedy_score(live_slots_in_segment: u64) -> u64 {
    live_slots_in_segment
}

/// 这一次挂载里区域扫描整次拒过的聚簇段（段起点 → 拒的理由）：[`choose_region`] 不再选它们。只增不减，随挂载会话一起丢。
/// 拒整次扫描的是盘上的坏记录（跨段的记录只有盘上读来的才有，归 C411（分配记录的跨度段与单元类别不符没检查） 的域），
/// 整理不替它修。
/// 第一版两盘同槽：按段起点的槽号记。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SegmentsRefusedByTheRegionScan {
    refused: BTreeMap<SlotNumber, RegionScanRefusal>,
}

impl SegmentsRefusedByTheRegionScan {
    /// 记下 `crate::region_scan::scan_region` 对 `region` 整次拒了（同一段再拒一次留第一次的理由）。
    pub fn note_refusal(&mut self, region: CompactionRegion, refusal: RegionScanRefusal) {
        self.refused.entry(region.start_slot()).or_insert(refusal);
    }

    /// 起点是 `segment_start` 的段这次挂载里被扫描拒过没有。
    #[must_use]
    pub fn contains(&self, segment_start: SlotNumber) -> bool {
        self.refused.contains_key(&segment_start)
    }
}

/// [`choose_region`] 这一次的答案。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionChoice {
    /// 选中这一段：它的扫描结果里至少一个能搬的单元。
    Chosen(CompactionRegion),
    /// 按判据排在最前的候选段还没扫过（`region_scans` 里没有它）：调用方先对它跑 `crate::region_scan::scan_region`——
    /// 扫成了把结果并进 `region_scans`，被拒了记进 [`SegmentsRefusedByTheRegionScan`]——再选一次。
    /// 每答一次这个，下一次的输入就多一段有结论，所以连着答的次数不超过单元区的整段数。
    ScanBeforeChoosing(CompactionRegion),
    /// 一段都没资格。
    NoEligibleSegment,
}

/// 挑一个整理区域 R。资格五条全成立的段才算：不在这次挂载开过的聚簇段里（`cluster_segments`，段起点）、
/// 不是活意图的 R、罩住的槽一个都不在 `pinned_slots` 里、这次挂载里没被区域扫描整次拒过（`segments_refused_by_the_region_scan`）、
/// R 里至少一个能搬的单元（按 `region_scans` 里这一段的扫描分类取）；
/// 按判据打分（活槽数：R 那块盘上未释放记录罩住的槽）排候选，同分取起点槽号最小的；排在最前的那一段扫过就交回
/// [`RegionChoice::Chosen`]，没扫过就交回 [`RegionChoice::ScanBeforeChoosing`]；扫过而一个能搬的都没有的段不是候选。
/// `region_scans` 要是对这一份 `allocation_records` 扫出来的（记录变了就重扫），不是 `unit_area.device` 那块盘上的不算。
/// 第一版两盘同槽：活槽数按 `unit_area.device` 那块盘的记录数，`cluster_segments`、活 R、`pinned_slots` 与被拒的段都只比槽号。
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "八个输入各是判决 G2 / S4 点名的一样，没有哪几个合起来是一个带不变量的概念（code-discipline.md「参数不按个数限」）"
)]
pub fn choose_region(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    cluster_segments: &BTreeSet<SlotNumber>,
    live_intent_region: Option<CompactionRegion>,
    pinned_slots: &BTreeSet<SlotNumber>,
    region_scans: &[RegionScan],
    segments_refused_by_the_region_scan: &SegmentsRefusedByTheRegionScan,
    criterion: RegionChoiceCriterion,
) -> RegionChoice {
    let live_slots_per_segment: Vec<LiveSlotsOfAClusterSegment> =
        live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area)
            .into_iter()
            .map(LiveSlotsOfAClusterSegment::Counted)
            .collect();
    choose_region_over_the_live_slots(
        &live_slots_per_segment,
        unit_area,
        cluster_segments,
        live_intent_region,
        pinned_slots,
        region_scans,
        segments_refused_by_the_region_scan,
        criterion,
    )
}

/// 按分配记录数 `unit_area` 那块盘上每个整段的活槽数（未释放记录罩住的槽数；末尾凑不满 64 槽的那一截不算，记录落在单元区之前的槽不算），
/// 按段起点从小到大一段一个（[`choose_region`] 打分前的那一步，D26（后台整理与放置回收） 已定项 13 第 1 条「选段打分」放上 GPU 的那一段的
/// CPU 写法）。纯函数，不读设备。
#[must_use]
pub fn live_slots_per_cluster_segment_by_the_records(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u64> {
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let mut live_slots_per_segment =
        vec![0u64; usize::try_from(full_segments).expect("一块盘的段数装得进 usize")];
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        // 迭代上界是这条记录的跨度；只数落在整段里的槽。
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) else {
                continue;
            };
            let segment = usize::try_from(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS)
                .expect("段号装得进 usize");
            if let Some(live_slots) = live_slots_per_segment.get_mut(segment) {
                *live_slots += 1;
            }
        }
    }
    live_slots_per_segment
}

/// `unit_area` 那块盘上每个整段的条目键，按段起点从小到大（与 [`live_slots_per_cluster_segment_by_the_records`] 一段一个、次序相同）。
#[must_use]
pub fn cluster_segment_entry_keys_of_the_unit_area(
    unit_area: UnitAreaOfOneDevice,
) -> Vec<ClusterSegmentEntryKey> {
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    (0..full_segments)
        .map(|segment| ClusterSegmentEntryKey {
            device: unit_area.device,
            start_slot: SlotNumber(unit_area_start + segment * CLUSTER_SEGMENT_SLOTS),
        })
        .collect()
}

/// 一个整段这一批的活槽数。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveSlotsOfAClusterSegment {
    /// 数出来了（CPU 写法，或 GPU 两份核确认、交 CPU 判的）。
    Counted(u64),
    /// GPU 两份核这一轮数得不一样（D26（后台整理与放置回收） 已定项 13 第 8 条）：这一段这一批不当候选，下一轮重算。
    UncertainThisRound,
}

/// 同 [`choose_region`]，每段的活槽数由调用方算好交进来（`live_slots_per_segment` 按段起点从小到大、一个整段一个，同
/// [`live_slots_per_cluster_segment_by_the_records`] 的次序；GPU 路径上来自丙段）。[`choose_region`] 就是先数再交给它，判法逐项相同。
/// 这一轮数得不确定的段（[`LiveSlotsOfAClusterSegment::UncertainThisRound`]）不进候选：其余的段照原次序交选段核，同分取起点槽号最小的不变。
///
/// # Panics
/// `live_slots_per_segment` 的个数与 `unit_area` 的整段数不等。
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "同 choose_region 的八个输入，只是记录换成了算好的每段活槽数（code-discipline.md「参数不按个数限」）"
)]
pub fn choose_region_over_the_live_slots(
    live_slots_per_segment: &[LiveSlotsOfAClusterSegment],
    unit_area: UnitAreaOfOneDevice,
    cluster_segments: &BTreeSet<SlotNumber>,
    live_intent_region: Option<CompactionRegion>,
    pinned_slots: &BTreeSet<SlotNumber>,
    region_scans: &[RegionScan],
    segments_refused_by_the_region_scan: &SegmentsRefusedByTheRegionScan,
    criterion: RegionChoiceCriterion,
) -> RegionChoice {
    // 按段起点索引一次（同一段有几份时取最前那一份）：下面每个候选段查一次，不逐份线性找。
    let mut scans_by_start_slot: BTreeMap<SlotNumber, &RegionScan> = BTreeMap::new();
    for scan in region_scans
        .iter()
        .filter(|scan| scan.region.device() == unit_area.device)
    {
        scans_by_start_slot
            .entry(scan.region.start_slot())
            .or_insert(scan);
    }
    let scan_of_the_segment =
        |start_slot: SlotNumber| scans_by_start_slot.get(&start_slot).copied();
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
    assert_eq!(
        u64::try_from(live_slots_per_segment.len()).expect("段数装得进 u64"),
        full_segments,
        "每段的活槽数一个整段一个（live_slots_per_cluster_segment_by_the_records 的次序）"
    );
    // 下标按段起点递增：核的“同分取最早下标”等价于原算法的“同分取最小槽号”。
    // 仍按设备过滤扫描，再保留同一起点的第一份；不得在这里重新排序或改成最后一份。
    // 数得不确定的段跳过，其余的照原次序进核：核交回的下标是这几段里的下标，按 `start_slots_of_the_candidates` 换回段起点。
    let mut start_slots_of_the_candidates: Vec<SlotNumber> = Vec::new();
    let segments: Vec<CompactionKernelSegment> = live_slots_per_segment
        .iter()
        .enumerate()
        .filter_map(|(segment, live_slots)| match live_slots {
            LiveSlotsOfAClusterSegment::Counted(live_slots) => Some((segment, live_slots)),
            LiveSlotsOfAClusterSegment::UncertainThisRound => None,
        })
        .map(|(segment, live_slots)| {
            let start_slot = SlotNumber(
                unit_area_start + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
            );
            start_slots_of_the_candidates.push(start_slot);
            let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
            CompactionKernelSegment {
                live_slots: match criterion {
                    RegionChoiceCriterion::Greedy => greedy_score(*live_slots),
                },
                is_opened_in_this_mount: cluster_segments.contains(&start_slot),
                is_the_live_intent_region: live_intent_region
                    .is_some_and(|region| region.start_slot() == start_slot),
                holds_a_pinned_slot: pinned_slots.range(start_slot..end_slot).next().is_some(),
                is_refused_by_the_region_scan: segments_refused_by_the_region_scan
                    .contains(start_slot),
                scan: match scan_of_the_segment(start_slot) {
                    None => CompactionKernelSegmentScan::NotScanned,
                    Some(scan) if scan.movable_units().next().is_some() => {
                        CompactionKernelSegmentScan::ScannedWithAMovableUnit
                    }
                    Some(_) => CompactionKernelSegmentScan::ScannedWithoutAMovableUnit,
                },
            }
        })
        .collect();
    let region_at = |position: usize| {
        CompactionRegion::of_one_cluster_segment(
            unit_area.device,
            start_slots_of_the_candidates[position],
        )
        .expect("单元区的整段起点在聚簇段边界上且装得进六字节")
    };
    match choose_compaction_segment(&segments) {
        CompactionKernelRegionChoice::Chosen(position) => RegionChoice::Chosen(region_at(position)),
        CompactionKernelRegionChoice::ScanBeforeChoosing(position) => {
            RegionChoice::ScanBeforeChoosing(region_at(position))
        }
        CompactionKernelRegionChoice::NoEligibleSegment => RegionChoice::NoEligibleSegment,
    }
}

/// 一批至多搬几个单元。至少 1；取值归 E10（老化：结构抗老化 vs 事后整理），这里只是参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionBatchUnits(NonZeroUsize);

impl CompactionBatchUnits {
    /// 0 交回 `None`：一批一个都不搬，意图永远推不完。
    #[must_use]
    pub fn new(units: usize) -> Option<Self> {
        NonZeroUsize::new(units).map(Self)
    }

    #[must_use]
    pub const fn units(self) -> usize {
        self.0.get()
    }
}

/// 计划里的一项：搬哪个单元（映射 key）、它现在在哪（源落点，两盘同槽）。目的落点不在这里（模块文档）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedRelocation {
    pub mapping_key: Vec<u8>,
    pub source: Placement,
}

/// [`plan_batch`] 的答案：有要搬的，或者一个能搬的都没有时分「搬空」与「搬不动」。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedBatch {
    /// 这一批要搬的，按槽升序，至少一项、至多 K 项。
    Relocate(Vec<PlannedRelocation>),
    /// R 里一条未释放的记录都没有了：R 按记录全空。
    RegionEmptied,
    /// R 里一个能搬的都没有，却还有未释放的记录（钉住、隔离、已死、不一致的）：再推下去也空不了。
    NothingMovableButTheRegionIsNotEmpty,
}

/// 下一批要搬的：扫描结果里可搬的单元按槽升序取前 `batch` 个。钉住、隔离、已死、不一致的一个都不进。
/// 这一批发布之后它们离开 R，下一次扫描就不再有它们：各批互不相交，全部批的并集是 R 里可搬的 key 集合。
/// 一个能搬的都没有时不交空表：R 里没有未释放的记录交 [`PlannedBatch::RegionEmptied`]，还有交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。
///
/// # Panics
/// 扫描结果里能搬的映射 key 有相同的：计划的输入只来自 [`crate::region_scan::scan_region`] 交回的 `Ok`，它见到同一把 key 判成可搬两次
/// 就拒这个 R（`RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey`），所以这里收到的 key 两两不同，一批里的 key 也就互异
#[must_use]
pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> PlannedBatch {
    assert!(
        region_scan.movable_mapping_keys_are_pairwise_distinct(),
        "扫描结果里能搬的映射 key 两两不同"
    );
    let allocations = compaction_kernel_allocations(region_scan);
    match plan_compaction_batch(&allocations, batch.units()) {
        CompactionKernelPlannedBatch::Relocate(positions) => PlannedBatch::Relocate(
            positions
                .into_iter()
                .map(|position| {
                    let allocation = &region_scan.allocations[position];
                    let AllocationInRegion::Movable(unit) = &allocation.class else {
                        panic!("核证过：计划里每一项都是能搬的（P2），这里是第 {position} 项 {allocation:?}");
                    };
                    PlannedRelocation {
                        mapping_key: unit.mapping_key.clone(),
                        source: allocation.placement,
                    }
                })
                .collect(),
        ),
        CompactionKernelPlannedBatch::RegionEmptied => PlannedBatch::RegionEmptied,
        CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
            PlannedBatch::NothingMovableButTheRegionIsNotEmpty
        }
    }
}

/// 保持扫描的条目顺序，并将完整 key 字节的相等关系保留为核身份的相等关系。
/// 独立成函数以便直接验证证明输入；产品输出不能观测仅参与 ghost 规格的身份字段。
fn compaction_kernel_allocations(
    region_scan: &RegionScan,
) -> Vec<CompactionKernelScannedAllocation> {
    // 身份按完整 key 字节相等分配，不按指针或哈希分配；借用输入，避免为整张扫描表复制 key。
    // allocations 与原扫描一一对应且不重排，核交回的下标可直接取回完整 Placement（包括 span）。
    let mut identities_of_the_keys: BTreeMap<&[u8], u64> = BTreeMap::new();
    region_scan
        .allocations
        .iter()
        .map(|allocation| CompactionKernelScannedAllocation {
            source_slot: allocation.placement.slot.0,
            class: match &allocation.class {
                AllocationInRegion::Movable(unit) => {
                    let next_identity = u64::try_from(identities_of_the_keys.len())
                        .expect("内存中的 key 数装得进 u64");
                    CompactionKernelAllocationClass::Movable {
                        mapping_key_identity: *identities_of_the_keys
                            .entry(unit.mapping_key.as_slice())
                            .or_insert(next_identity),
                    }
                }
                AllocationInRegion::Pinned => CompactionKernelAllocationClass::Pinned,
                AllocationInRegion::Quarantined => CompactionKernelAllocationClass::Quarantined,
                AllocationInRegion::Dead => CompactionKernelAllocationClass::Dead,
                AllocationInRegion::Inconsistent(_) => {
                    CompactionKernelAllocationClass::Inconsistent
                }
            },
        })
        .collect()
}

/// 下一批要搬的，跳过这次发布本来就重写的节点（
/// 规划时补足）：`rewritten_anyway` 是会话在这一次发布上试算出的、计划过而发布路径不另搬的映射 key（它们照样离开 R，只是不算搬）。
/// 扫描结果里去掉这几把 key 的可搬项，剩下的照 [`plan_batch`] 取（按槽升序取前 `batch` 个，补上 R 里真要搬的单元）；剩下的一项都没有时
/// 分两种：R 里本来就没有未释放的记录交 [`PlannedBatch::RegionEmptied`]；R 里只剩这次发布本来就重写的节点（每条未释放记录都是可搬单元、
/// 它们的 key 都在 `rewritten_anyway` 里）交 [`PlannedBatch::Relocate`] 带上它们全部——计划罩住 R 里每一条记录，调用方照 F2 同一次发布
/// 删意图，它们不受批大小限，因为本来就重写的节点不是搬迁。剩下的只有钉住、隔离、已死、不一致的交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。
/// `rewritten_anyway` 是空的时就是 [`plan_batch`]。
///
/// # Panics
/// 同 [`plan_batch`]：去掉这几把 key 的可搬项之后，能搬的映射 key 有相同的（输入只来自 `scan_region` 交回的 `Ok`，走不到）。
#[must_use]
pub fn plan_batch_skipping_the_units_this_publish_rewrites_anyway(
    region_scan: &RegionScan,
    batch: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
) -> PlannedBatch {
    let is_rewritten_anyway = |allocation: &ScannedAllocation| match &allocation.class {
        AllocationInRegion::Movable(unit) => rewritten_anyway.contains(&unit.mapping_key),
        AllocationInRegion::Pinned
        | AllocationInRegion::Quarantined
        | AllocationInRegion::Dead
        | AllocationInRegion::Inconsistent(_) => false,
    };
    let scan_without_the_units_rewritten_anyway = RegionScan {
        region: region_scan.region,
        allocations: region_scan
            .allocations
            .iter()
            .filter(|allocation| !is_rewritten_anyway(allocation))
            .cloned()
            .collect(),
    };
    match plan_batch(&scan_without_the_units_rewritten_anyway, batch) {
        PlannedBatch::Relocate(relocations) => PlannedBatch::Relocate(relocations),
        PlannedBatch::RegionEmptied if region_scan.allocations.is_empty() => {
            PlannedBatch::RegionEmptied
        }
        // 去掉的那几项之外一条记录都没有：R 里只剩这次发布本来就重写的节点。
        PlannedBatch::RegionEmptied => PlannedBatch::Relocate(
            region_scan
                .movable_units()
                .map(|(source, unit)| PlannedRelocation {
                    mapping_key: unit.mapping_key.clone(),
                    source,
                })
                .collect(),
        ),
        PlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
            PlannedBatch::NothingMovableButTheRegionIsNotEmpty
        }
    }
}

/// 批大小的产品默认：一批至多搬 16 个单元（D26（后台整理与放置回收） 已定项 1；数是拍的，下界由一份 `B_gc` 做得完一条意图算出：
/// 默认几何下一条 64 单元的意图要 1 + ⌈64/16⌉ = 5 次发布）。挂载参数，不存盘（[`CompactionBatchSize::OfTheMount`]）。
pub const COMPACTION_BATCH_UNITS_BY_DEFAULT: CompactionBatchUnits = match NonZeroUsize::new(16) {
    Some(units) => CompactionBatchUnits(units),
    None => panic!("16 不是 0"),
};

/// 一批至多搬几个单元从哪来。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionBatchSize {
    /// 这次挂载给的（挂载参数，默认 [`COMPACTION_BATCH_UNITS_BY_DEFAULT`]）。
    OfTheMount(CompactionBatchUnits),
    /// 只供测试。
    ForcedForTests(CompactionBatchUnits),
}

/// 整理什么时候起跑。封闭集合。产品路径照水位（[`CompactionRunForcing::FollowTheWatermarks`]）；另两种只供测试。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunForcing {
    /// 定时兜底那一次与用户改动发成之后那一次按水位判起不起跑（D26（后台整理与放置回收） 已定项 13 第 5 条，[`CompactionWatermarks`]）；
    /// 后台入口连同不发布的停做的备忘、挂载那一份预算从根环读、破例与回退门都只在这一支判。显式入口
    /// （`mounted_session::MountedSession::run_compaction`）不看水位、不过回退门，照常跑。
    FollowTheWatermarks,
    /// 只供测试：起跑判据不看水位、恒为真——定时兜底查到的是「该跑」，用户改动发成之后记下的也是「该跑了」，由后台入口
    /// （`mounted_session::MountedSession::run_compaction_in_the_background`）去跑。备忘、挂载那一份预算从根环读、破例与回退门这一支都不判：
    /// 用它的用例钉的是一趟怎么推，发布数由用例自己摆。
    ForcedToStartForTests,
    /// 只供测试：两个入口都不起跑，停机原因 [`CompactionRunStop::ForcedToStopForTests`]。
    ForcedToStopForTests,
}

/// 签发之后 R 排不排他（D26（后台整理与放置回收） 已定项 1 第 3 条：`R` 在意图期内不可作分配目的地）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRegionExclusion {
    ExcludedWhileTheIntentIsLive,
    /// 只供测试：活意图的 R 照样当分配目的地。
    SwitchedOffForTests,
}

/// 搬迁的目的落点按哪条政策取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationDestinationPolicy {
    /// 判决 G2 的 D-a：码 1 走用户数据落点函数（D3（空间分配） 已定项 8 第 1 条），节点与打包容器走开放段 bump（已定项 10 ⑤），都不落在活 R 里。
    UserDataPlacementForDataAndOpenSegmentForTheRest,
    /// 只供测试：每个单元都取单元区里起点最小、整跨度没挡的落点（提交内生块回落那一个函数），仍不落在活 R 里。
    LowestUnblockedSlotForEveryUnitForTests,
}

/// 签发之后强制一次「释放前核出对不上」。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForcedReleaseChecksumMismatchAfterIssue {
    NotForced,
    /// 只供测试：签发之后第一批的第一个源落点当作读盘核两份都核出对不上（每块盘一份都隔离，D19（块指针的结构与宽度预算） 已定项 5），
    /// 整个挂载只强制这一次。
    OnceOnTheFirstSourceOfTheFirstBatchForTests,
}

/// 合取 7 `全空聚簇段数 ≥ N₀` 的 `N₀`（D3（空间分配） 已定项 7 登记表第 7 行）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyClusterSegmentsFloor {
    /// 第一版的值 [`EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION`]（算式等计数模型验过再定，E10（老化：结构抗老化 vs 事后整理））。
    OfTheFirstVersion,
    /// 只供测试。
    ForcedForTests(u64),
}

/// D3（空间分配） 已定项 7 登记表第 7 行：「`N₀` 第一版取 1」。
pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;

impl EmptyClusterSegmentsFloor {
    /// 这一档的 `N₀`。
    #[must_use]
    pub const fn segments(self) -> u64 {
        match self {
            EmptyClusterSegmentsFloor::OfTheFirstVersion => {
                EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION
            }
            EmptyClusterSegmentsFloor::ForcedForTests(segments) => segments,
        }
    }
}

/// 一批搬迁的源落点什么时候释放。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationPublishOrder {
    /// 判决 G3：源落点的释放与映射条目换位置在同一次发布里。
    SourceReleasedInThePublishThatMovesTheMapping,
    /// 只供测试（`.claude/kb/milestone/04-fourth-txn.md` 二·3 的必红世界「源落点在映射发布之前就被释放」）：这一批照样释放源落点、
    /// 写目的地与它的分配记录，映射条目与各处指针却留在源落点上。产品路径一处都不走它。
    SourceReleasedWhileTheMappingStillNamesItForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：一次用户改动发成之后，会话为读到只坏一份的单元另发的那一批搬迁
/// （`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
/// 被拒推抬 F 再试那一支在小盘之外的池上造不出「改动放得下、紧接着那一批放不下」，靠它进；
/// 开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationBatchAfterTheChangeRefusedForSpace {
    NotForced,
    /// 接下来这几次发这一批都当作被拒，用完之后照常发。
    ForcedForTheNextAttemptsForTests {
        attempts: NonZeroU64,
    },
    /// 每一次发这一批都当作被拒。
    ForcedOnEveryAttemptForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationBatchAfterTheChangeRefusedForSpace`]）在发之前
/// 当作被拒、而不是空间不够（计划对不上那一版那一种：`transaction::RelocationBatchRefused::PlannedUnitNotInTheVersion`，点名计划里头一把 key）。
/// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace`）
/// 在产品路径上要计划在算出来与发之间失效、或发布路径报空间之外的错才进得去，靠它进；开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。
/// 被空间拒的那个开关同时开着时先照那一个拒（它在前面判）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationBatchAfterTheChangeRefusedNotForSpace {
    NotForced,
    /// 每一次发这一批都当作被拒（不是空间不够：不再试，一批只发这一次）。
    ForcedOnEveryAttemptForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话为另发的那一批搬迁被空间拒而推抬 F 时，那一串抬 F 当作自己报了错
/// （`mount::MountError::InstanceTableUnreadable`：`mount::push_one_floor_raise_within_the_admission_budget` 在任何写之前交得出的那一种），一个写都不发。
/// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace`）
/// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationBatchAfterTheChangeRefusedForSpace`] 合着开。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaiseForTheRelocationBatchAfterTheChangeFails {
    NotForced,
    /// 每一次为这一批推抬 F 都当作抬 F 自己报了错。
    ForcedOnEveryRaiseForTests,
}

/// 整理只供测试的开关。
/// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
/// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
/// （`mount::mount_writable_with_compaction_switches`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionSwitches {
    pub batch_size: CompactionBatchSize,
    /// 只供测试：签发时不按判据选、取这一段（资格那五条照判，不合格就停）。`None` 是产品路径。
    pub forced_region: Option<CompactionRegion>,
    pub run_forcing: CompactionRunForcing,
    pub region_exclusion: CompactionRegionExclusion,
    pub destination_policy: RelocationDestinationPolicy,
    pub release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue,
    pub empty_cluster_segments_floor: EmptyClusterSegmentsFloor,
    pub relocation_publish_order: RelocationPublishOrder,
}

impl CompactionSwitches {
    pub const PRODUCT: CompactionSwitches = CompactionSwitches {
        batch_size: CompactionBatchSize::OfTheMount(COMPACTION_BATCH_UNITS_BY_DEFAULT),
        forced_region: None,
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    };
}

/// 一次整理运行停在哪。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunStop {
    /// 只供测试的开关叫停（[`CompactionRunForcing::ForcedToStopForTests`]）。
    ForcedToStopForTests,
    /// 一段都没资格（[`RegionChoice::NoEligibleSegment`]），或强制选的那一段不合格。
    NoEligibleSegment,
    /// 签发前判不过（D26（后台整理与放置回收） 已定项 1 第 2 条）：R 外非全空段的空洞少于 R 里要搬的单元数，不签。
    HolesOutsideTheRegionFewerThanItsUnitsToMove { holes: u64, units_to_move: u64 },
    /// 上一条意图完成了，全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假：不签下一条。
    NoYieldFromTheLastIntent {
        empty_cluster_segments_now: u64,
        empty_cluster_segments_at_issue: u64,
    },
    /// 这次运行的发布预算用完了。
    PublishBudgetSpent { budget: u64 },
    /// 一批被空间拒（准入或落点）：暂停——单独发一次只删意图的发布丢掉意图（R 排他跟着撤），不推抬 F；下一次整理从头签发、重扫。
    /// 只删意图的那一次归删除类，不判式子、落点照判（`transaction::judge_the_publish_before_any_read_or_write`）。
    PausedShortOfSpace,
    /// 签发那次发布被空间拒（准入或落点）：意图没签成、盘上没有它，R 排他跟着撤；不推抬 F。
    IssueRefusedShortOfSpace,
    /// 可写挂载续做（只续做盘上那一条未完成的意图、不签新的，D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」）：
    /// 那一条删了（达成或丢掉），或本来就没有活意图。
    NoLiveIntentLeftToContinue,
    /// R 里一个能搬的都没有、却还有未释放的记录（[`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]）：意图连同 R 排他丢掉。
    NothingMovableButTheRegionIsNotEmpty,
    /// 用户改动之后起跑的那一趟只跑至多一批，跑完了。
    OneBatchAfterTheUserChangeDone,
    /// 后台入口：前台有 fsync 在等，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
    /// 全空段低于底额、前台写不进时这一趟的第一次发布不为前台 fsync 让路（前台已写不进，让路换不来什么）。
    YieldedToAForegroundFsync,
    /// 后台入口：调用方在停这条驱动后台入口的线程，不发下一批，低于底额的破例也不发（线程很快 join 得上）。
    YieldedToTheCallerStoppingItsThread,
    /// 后台入口：这一个写带宽窗口里整理写的字节已到写带宽上限，不发下一批（D26 已定项 13 第 6 条，[`CompactionWriteBandwidthLimit`]）。
    WriteBandwidthLimitReached {
        bytes_written_in_the_window: u64,
        limit_in_bytes_per_second: u64,
    },
    /// 这一批算着出了错（D26 已定项 13 第 4 条）：这一批放弃、一个单元都没搬；现行那一版里有活意图就单发一次只删意图的发布丢掉它
    /// （R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），没有就一个写都没发。整理这个后台任务怎样了见 `task_after_the_error`：
    /// 一批里同一个条目键出现两次（`background_compute::BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`）只作废这一批、任务不停
    /// （[`BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned`]）；别的错停下，或当场解除停下。下一次整理从头签发。
    BatchAbandonedAfterABackgroundComputeError {
        task_after_the_error: BackgroundTaskAfterTheComputeError,
    },
    /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或两批之间实现答交给它的那一组换上了而一张卡都用不上、答开不成而当场停着的同算），
    /// 而这一刻池子每块盘的已分配占它的容量都不超过补跑阈值：
    /// 停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它（R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），之后一个写都不发；
    /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。显式入口每一趟开头判这一条；后台入口判出停着而有活意图时起跑一趟只为删它
    /// （`mounted_session::BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`），没有活意图就不起跑。挂着切换之后实现答那一组一张卡都用不上或开不成、
    /// 手里没有活意图时，两个入口都在任何写之前停在这一条。一趟之中在 CPU 上补跑过几批之后、或一趟之中两批之间实现答出那一组的结局才判出的，
    /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下；开卡期间出错停着的，实现答那一组换上了也解除。
    BackgroundTaskStoppedAfterAnEarlierComputeError,
    /// GPU 路径上 R 里除了这一轮两份核判得不一样的记录，再没有判定了可搬的（D26 已定项 13 第 8 条「这一批不回收、不搬，留到下一轮重算」）：
    /// 这一批不发，意图照留（不删也不丢），下一批重扫照判。
    OnlyRecordsUncertainThisRoundLeftToMove { records_uncertain_this_round: u64 },
    /// GPU 路径上选中要签的那一段里有这一轮两份核判得不一样的记录（D26 已定项 13 第 8 条）：R 里要搬几个数不准，签发前那一判判不了，
    /// 这一批不签，下一批重扫照判。
    IssueHeldBackByRecordsUncertainThisRound { records_uncertain_this_round: u64 },
    /// 后台入口：这次整理发布要写的根槽与它之后余量（[`root_ring_slots_left_by_a_compaction_publish`]）那几槽里的根一走，根环里不同用户状态就少于 4 个
    /// （D16（发布语义） 已定项 1 的回退候选集；D26 已定项 13 第 5 条的加门）：不发，一个写都没发。全空段低于底额、前台写不进时每趟第一次发布破例不判。
    CompactionPublishWouldLeaveFewerThanFourRollbackStates {
        states_now: u64,
        states_after_the_publish_and_the_margin: u64,
    },
    /// 后台入口：回退门读根环或树表没读通（读不清有几个状态就不发）：一个写都没发，下一次叫起重判。
    RollbackStatesOfTheRingUnreadable,
    /// 后台入口放锁的那一种（`mounted_session::MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）：
    /// 这一趟这一次调用发成了一次，调用方放锁之后接着调。不是停机，这一趟记在会话上没完。
    OnePublishOfTheTripDone,
    /// 后台入口因删除队列不空起跑的那一趟（D26（后台整理与放置回收） 已定项 13）：队列走空了，这一趟只走队列、一次整理发布都不发。整理下一次调用照常判起跑。
    DeletionQueueWalkedWithoutACompactionPublish,
}

impl CompactionRunStop {
    /// 这一种停在现行那一版不变、开关不变时重判还是同一个结论，又不发布：记进备忘，定时兜底下一次不重判
    /// （[`CompactionOfThisMount::note_the_stop_of_a_run`]）。选段、签发前判、无产出与回退门读的（分配记录、映射、快照钉住的槽、根环）都随发布变，
    /// 内存里的隔离集与不合格段表也只在发布里变。GPU 两份核不一致、被节奏或空间截住、出错的那几种下一次可能翻面，不进备忘。
    #[must_use]
    pub const fn publishes_nothing_until_the_next_publish(self) -> bool {
        match self {
            CompactionRunStop::NoEligibleSegment
            | CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
            | CompactionRunStop::NoYieldFromTheLastIntent { .. }
            | CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                ..
            } => true,
            CompactionRunStop::ForcedToStopForTests
            | CompactionRunStop::PublishBudgetSpent { .. }
            | CompactionRunStop::PausedShortOfSpace
            | CompactionRunStop::IssueRefusedShortOfSpace
            | CompactionRunStop::NoLiveIntentLeftToContinue
            | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
            | CompactionRunStop::OneBatchAfterTheUserChangeDone
            | CompactionRunStop::YieldedToAForegroundFsync
            | CompactionRunStop::YieldedToTheCallerStoppingItsThread
            | CompactionRunStop::WriteBandwidthLimitReached { .. }
            | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
            | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
            | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
            | CompactionRunStop::RollbackStatesOfTheRingUnreadable
            | CompactionRunStop::OnePublishOfTheTripDone
            | CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish => false,
        }
    }

    /// 运行时报出停在哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            CompactionRunStop::ForcedToStopForTests => "compaction_stopped_by_the_test_only_switch",
            CompactionRunStop::NoEligibleSegment => "compaction_stopped_no_eligible_segment",
            CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. } => {
                "compaction_stopped_holes_outside_the_region_too_few"
            }
            CompactionRunStop::NoYieldFromTheLastIntent { .. } => {
                "compaction_stopped_no_yield_from_the_last_intent"
            }
            CompactionRunStop::PublishBudgetSpent { .. } => {
                "compaction_stopped_publish_budget_spent"
            }
            CompactionRunStop::PausedShortOfSpace => "compaction_paused_short_of_space",
            CompactionRunStop::IssueRefusedShortOfSpace => {
                "compaction_stopped_the_issue_refused_short_of_space"
            }
            CompactionRunStop::NoLiveIntentLeftToContinue => {
                "compaction_stopped_no_live_intent_left_to_continue"
            }
            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty => {
                "compaction_stopped_nothing_movable_in_the_region"
            }
            CompactionRunStop::OneBatchAfterTheUserChangeDone => {
                "compaction_stopped_after_one_batch_following_the_user_change"
            }
            CompactionRunStop::YieldedToAForegroundFsync => {
                "compaction_yielded_to_a_foreground_fsync"
            }
            CompactionRunStop::YieldedToTheCallerStoppingItsThread => {
                "compaction_yielded_to_the_caller_stopping_its_thread"
            }
            CompactionRunStop::WriteBandwidthLimitReached { .. } => {
                "compaction_paced_by_the_write_bandwidth_limit"
            }
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. } => {
                "compaction_batch_abandoned_after_a_background_compute_error"
            }
            CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError => {
                "compaction_stopped_after_an_earlier_background_compute_error"
            }
            CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. } => {
                "compaction_stopped_only_records_uncertain_on_the_gpu_left_to_move"
            }
            CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. } => {
                "compaction_issue_held_back_by_records_uncertain_on_the_gpu"
            }
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                ..
            } => "compaction_stopped_the_publish_would_leave_fewer_than_four_rollback_states",
            CompactionRunStop::RollbackStatesOfTheRingUnreadable => {
                "compaction_stopped_the_rollback_states_of_the_ring_are_unreadable"
            }
            CompactionRunStop::OnePublishOfTheTripDone => "compaction_one_publish_of_the_trip_done",
            CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish => {
                "compaction_deletion_queue_walked_without_a_compaction_publish"
            }
        }
    }
}

/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（产品值排他；只供测试的开关 [`CompactionRegionExclusion::SwitchedOffForTests`] 不排）。
/// 整理停下（暂停、一批出错停下）删掉意图连同排他，盘上每一版不留「意图在、R 不排他」那一态；只删意图的那一次发布自己取落点时放开 R
/// （[`CompactionSwitchesAndTheExcludedRegion::open_the_region_to_the_publish_deleting_the_intent`]），那一次发布落盘的那一版里已没有意图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveCompactionIntent {
    pub intent: CompactionIntent,
    pub is_the_region_excluded_from_placement: bool,
}

/// 整理的运行时观测（判决 G8「起跑次数、批数、停机原因、用掉的根槽数」；D26（后台整理与放置回收） 已定项 6 的运行时计数器那两样：
/// 一批整理的搬迁量；runs、全空聚簇段数、回落计数、不可回收字节由分配器与记账给，整理进度不存）。只住内存、只增，丢了不影响判定。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompactionObservations {
    pub runs_started: u64,
    pub intents_issued: u64,
    pub intents_completed: u64,
    pub intents_dropped: u64,
    pub batches_published: u64,
    /// 整理的发布用掉的根槽数：一次发布写一个根槽，所以就是整理发布（签发、搬一批、删意图）的次数。
    pub root_ring_slots_used: u64,
    pub units_relocated: u64,
    /// 最近一批搬了几个单元（D26 已定项 6「一批整理的搬迁量」）。
    pub units_relocated_by_the_last_batch: u64,
    pub last_stop: Option<CompactionRunStop>,
}

/// 分配器这一侧的整理状态：只供测试的开关与活意图的 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条「`R` 在意图期内不可作分配目的地」：
/// 分配路径多一条合取，意图住 logged_ops 树、分配器读得到）。活意图的权威在现行那一版的 logged_ops 里（`transaction::TransactionOutput::logged_operations`）；
/// 这里记的是分配器要排除的那一份：会话签发那次发布之前装上（签发那次发布自己的新落点也不进 R）、删意图之后撤掉，可写挂载按挂载那一版的
/// logged_ops 装上（`mount`）。另带两样只有发布路径看得见的：签发之后强制一次「释放前核出对不上」那个开关用过没有，与这次挂载里发布路径
/// 隔离过、还没交给会话的槽（会话把它们并进常驻隔离集，[`CompactionOfThisMount::take_the_quarantines_noted_by_the_publishes`]）。
/// 随分配器只住内存，生命周期一次挂载。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactionSwitchesAndTheExcludedRegion {
    pub switches: CompactionSwitches,
    /// 三条水位线（D26（后台整理与放置回收） 已定项 1，挂载参数，不存盘）。
    pub watermarks: CompactionWatermarks,
    /// 只供测试：不看读数，判起跑判据时当这个量越了线（[`CompactionWatermarkCrossingForcing`]）。
    pub watermark_crossing_forcing: CompactionWatermarkCrossingForcing,
    live_intent: Option<LiveCompactionIntent>,
    has_the_forced_release_checksum_mismatch_been_used: bool,
    quarantines_noted_by_the_publishes: Vec<(SlotNumber, u64)>,
}

impl CompactionSwitchesAndTheExcludedRegion {
    /// 产品路径起步：开关是产品值，没有活意图。
    #[must_use]
    pub fn of_a_new_mount() -> Self {
        Self {
            switches: CompactionSwitches::PRODUCT,
            watermarks: CompactionWatermarks::BY_DEFAULT,
            watermark_crossing_forcing: CompactionWatermarkCrossingForcing::NotForced,
            live_intent: None,
            has_the_forced_release_checksum_mismatch_been_used: false,
            quarantines_noted_by_the_publishes: Vec::new(),
        }
    }

    #[must_use]
    pub fn live_intent(&self) -> Option<LiveCompactionIntent> {
        self.live_intent
    }

    /// 分配器这一侧的签发：记下活意图，R 从此排他（开关关掉 R 排他时不排）。盘上那一条由签发那次发布写进 logged_ops
    /// （`transaction::publish_compaction_step`）；挂载按 logged_ops 里那一条装上。全池活意图至多一条。
    ///
    /// # Panics
    /// 已经有一条活意图：调用方先判过（会话按现行那一版的 logged_ops 判，挂载在新建的分配器上装）。
    pub fn issue(&mut self, intent: CompactionIntent) {
        assert!(
            self.live_intent.is_none(),
            "全池活整理意图至多一条（判决 G1 e1）：签发之前调用方判过没有活意图"
        );
        self.live_intent = Some(LiveCompactionIntent {
            intent,
            is_the_region_excluded_from_placement: match self.switches.region_exclusion {
                CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
                CompactionRegionExclusion::SwitchedOffForTests => false,
            },
        });
    }

    /// 意图删了（达成或丢掉，删它的那次发布已落盘），或签发那次发布没发成：撤掉活意图连同 R 排他，交回撤掉的那一条。
    pub fn withdraw_the_live_intent(&mut self) -> Option<CompactionIntent> {
        self.live_intent.take().map(|live| live.intent)
    }

    /// 只删意图、不搬单元的那一次发布（暂停、出错停下、停着时的删意图，R 已按记录全空的删意图；`transaction::publish_compaction_step_in_the_session`）
    /// 取落点之前放开 R：那一次发布落盘之后盘上就没有这条意图，它自己的新落点落进 R 不违 I-5.6（「活意图的 R 内没有分配代 > `c0` 的记录」只管意图活着的那几版）。
    /// 活意图本身不撤（撤的时机照旧在删意图那一版落盘之后，会话撤）；交回放开之前排不排他，没有活意图交回 `None`。发布返回之后（发成或被拒）
    /// 照交回的那一个经 [`Self::put_back_the_region_exclusion`] 放回，别的发布照旧排除 R。
    pub(crate) fn open_the_region_to_the_publish_deleting_the_intent(&mut self) -> Option<bool> {
        self.live_intent
            .as_mut()
            .map(|live| std::mem::replace(&mut live.is_the_region_excluded_from_placement, false))
    }

    /// 放回 [`Self::open_the_region_to_the_publish_deleting_the_intent`] 放开之前的排他。活意图已撤（或从来没有）就什么都不做。
    pub(crate) fn put_back_the_region_exclusion(
        &mut self,
        was_excluded_before_the_publish: Option<bool>,
    ) {
        if let (Some(live), Some(was_excluded)) =
            (self.live_intent.as_mut(), was_excluded_before_the_publish)
        {
            live.is_the_region_excluded_from_placement = was_excluded;
        }
    }

    /// 分配器此刻不许当目的地的 R（活意图且排他时）。
    #[must_use]
    pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
        self.live_intent
            .filter(|live| live.is_the_region_excluded_from_placement)
            .map(|live| live.intent.region)
    }

    /// 发布路径记下一次释放前核出对不上而隔离的落点（跨度里的每一个槽，两盘同槽只记槽号）：等会话取走、并进常驻隔离集。
    /// 发布没成立时分配器换回进来时那一份，这一笔跟着没了。
    pub fn note_quarantined_placement(&mut self, slot: SlotNumber, span: u64) {
        self.quarantines_noted_by_the_publishes.push((slot, span));
    }

    /// 交出发布路径记下、会话还没取走的隔离落点（取走之后清空）。
    pub fn take_the_quarantines_noted_by_the_publishes(&mut self) -> Vec<(SlotNumber, u64)> {
        std::mem::take(&mut self.quarantines_noted_by_the_publishes)
    }

    /// 签发之后强制一次「释放前核出对不上」这一回该不该用上（用上就记下，整个挂载只一次）。
    pub fn take_the_forced_release_checksum_mismatch(&mut self) -> bool {
        match self.switches.release_checksum_mismatch_after_issue {
            ForcedReleaseChecksumMismatchAfterIssue::NotForced => false,
            ForcedReleaseChecksumMismatchAfterIssue::OnceOnTheFirstSourceOfTheFirstBatchForTests => {
                if self.has_the_forced_release_checksum_mismatch_been_used
                    || self.live_intent.is_none()
                {
                    return false;
                }
                self.has_the_forced_release_checksum_mismatch_been_used = true;
                true
            }
        }
    }
}

/// 会话这一侧的整理状态（丁 1 交判第 2 点：不合格段表挂会话；生命周期一次挂载）：这次挂载里被区域扫描拒过的段、这次挂载里释放前核出对不上
/// 而隔离的槽（常驻隔离集）、上一次用户改动之后整理发了几次布（`B_gc`）、观测。住在挂着的会话上
/// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
/// 整理挪出用户改动的调用线程之后另住四样：起跑判据记下的「该跑了」、上一条完成的意图
/// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。另记这次挂载里读—改—写读回旧数据单元时只坏一份的次数
/// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）、可写挂载读盘建上一版时只坏一份的次数
/// （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`），与会话另发那一批搬迁的三个只供测试的开关（被空间拒、
/// 被拒不是空间不够、为它推的抬 F 自己报错）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactionOfThisMount {
    pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
    quarantined_slots: BTreeSet<SlotNumber>,
    /// 上一次用户改动之后整理发了几次布：会话发成一次用户改动就归零。
    pub publishes_since_the_last_user_change: u64,
    pub observations: CompactionObservations,
    /// 最近一次判起跑判据的结论：哪块盘哪个量越了线、读数与线各是多少（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。还没判过是 `None`。
    /// 判了没起跑（[`WatermarkVerdict::NoLineCrossed`]）也记：它答的就是这一次为什么没起跑，所以不进只记起跑之后那几样的 [`CompactionObservations`]。
    pub last_watermark_verdict: Option<WatermarkVerdict>,
    /// 用户改动发成之后起跑判据查出来的结果，后台入口取走（[`CompactionStartJudgement`]）。
    pub start_judgement: CompactionStartJudgement,
    /// 这次挂载里上一条完成的意图（「上一条无产出就不签」读它的 `segs0`）：跨趟留着，不随一趟结束丢掉，
    /// 可写挂载续做做完的那一条也记在这里、交给会话。
    pub last_completed_intent: Option<CompactionIntent>,
    /// 定时兜底上一次查起跑判据时调用方的钟读数；还没查过是 `None`。
    timer_fallback_last_check: Option<CallerMonotonicClockReading>,
    /// 写带宽上限的这一个窗口（[`COMPACTION_WRITE_BANDWIDTH_WINDOW`]）：开了上限的后台入口推第一批时开、过了一个窗口长就换新的。
    write_bandwidth_window: Option<CompactionWriteBandwidthWindow>,
    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：会话把每次用户改动里各次发布的写入口
    /// 上的数加进来（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`）。与常驻隔离集同是这次挂载里见到的坏副本那一类观测，
    /// 同住这一份挂载期状态；只在内存里，新挂载从 0 起（[`Self::of_a_new_mount`]）。
    old_data_unit_reads_back_with_one_copy_failing: u64,
    /// 可写挂载读盘建上一版时按位置条目读一个单元只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：挂载交回之前记一次
    /// （`mount::mount_writable` 那一串）。与上一样分开记：上一样是读—改—写读回旧数据单元，这一样是挂载读的各类单元。只在内存里，新挂载从 0 起。
    unit_reads_with_one_copy_failing_while_mounting: u64,
    /// 可写挂载读盘建上一版时读到只坏一份、到挂载交回那一版仍在中央映射里（或留存条目里）的那几个单元的映射 key（一个单元一把，按读到的先后）：
    /// 会话下一次用户改动发成之后照快照下另发那批同一条路径把它们搬到新槽、两份新写（
    /// `mounted_session::MountedSession::publish_user_change`），交给那一批就清空。只在内存里，新挂载是空的。
    mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec<Vec<u8>>,
    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。新挂载是产品值。
    relocation_batch_after_the_change_refused_for_space:
        RelocationBatchAfterTheChangeRefusedForSpace,
    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。新挂载是产品值。
    relocation_batch_after_the_change_refused_not_for_space:
        RelocationBatchAfterTheChangeRefusedNotForSpace,
    /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。新挂载是产品值。
    floor_raise_for_the_relocation_batch_after_the_change_fails:
        FloorRaiseForTheRelocationBatchAfterTheChangeFails,
    /// 上一次不发布的停（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）记在哪一版、哪一组开关上：这两样都没变，定时兜底不重判。
    stop_that_publishes_nothing_until_the_next_publish:
        Option<StopThatPublishesNothingUntilTheNextPublish>,
    /// 这次挂载那一份预算还没从根环读：挂载不给新预算（D26 已定项 13 第 5 条的加门），头一次判后台起跑时
    /// 把已经过去的发布数取成根环里最新那个用户状态之后的根数；这之前发成用户改动就不用读了。
    publishes_of_the_mount_are_still_to_be_read_from_the_ring: bool,
    /// 放锁的后台入口上一次调用之后还没完的那一趟（[`BackgroundTripUnderWay`]）。
    background_trip_under_way: Option<BackgroundTripUnderWay>,
}

/// 上一次不发布的停记下的那几样（[`CompactionOfThisMount::note_the_stop_of_a_run`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StopThatPublishesNothingUntilTheNextPublish {
    checkpoint_txg: CheckpointTxg,
    switches: CompactionSwitches,
    stop: CompactionRunStop,
}

/// 放锁的后台入口跨几次调用推的那一趟：一趟只判一次起跑，每次调用至多一次发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundTripUnderWay {
    pub(crate) trigger: BackgroundCompactionTrigger,
    pub(crate) reach: CompactionRunReach,
    /// 这一趟前几次调用已发的批数（用户改动之后那一趟至多一批，跨调用数）。
    pub(crate) batches_published: u64,
    /// 这一趟是全空段低于底额、前台写不进时起跑的，前几次调用还没发过一次整理发布（只走了删除续做、被写带宽截住或排空被拒）：
    /// 下一次调用接着推时，整理的第一次发布照样破例。
    pub(crate) is_the_exception_of_the_trip_still_open: bool,
}

impl CompactionOfThisMount {
    /// 一次挂载起步：各样都空，没有「该跑了」。
    #[must_use]
    pub fn of_a_new_mount() -> Self {
        Self {
            segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan::default(),
            quarantined_slots: BTreeSet::new(),
            publishes_since_the_last_user_change: 0,
            observations: CompactionObservations::default(),
            last_watermark_verdict: None,
            start_judgement: CompactionStartJudgement::NotDue,
            last_completed_intent: None,
            timer_fallback_last_check: None,
            write_bandwidth_window: None,
            old_data_unit_reads_back_with_one_copy_failing: 0,
            unit_reads_with_one_copy_failing_while_mounting: 0,
            mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec::new(),
            relocation_batch_after_the_change_refused_for_space:
                RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
            relocation_batch_after_the_change_refused_not_for_space:
                RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced,
            floor_raise_for_the_relocation_batch_after_the_change_fails:
                FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced,
            stop_that_publishes_nothing_until_the_next_publish: None,
            publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,
            background_trip_under_way: None,
        }
    }

    /// 一趟停下时记下不发布的停：`current_txg` 是现行那一版，`switches` 是这一趟用的开关。别的结局清掉备忘（下一次照常判）。
    pub(crate) fn note_the_stop_of_a_run(
        &mut self,
        stop: CompactionRunStop,
        current_txg: CheckpointTxg,
        switches: CompactionSwitches,
    ) {
        self.stop_that_publishes_nothing_until_the_next_publish = stop
            .publishes_nothing_until_the_next_publish()
            .then_some(StopThatPublishesNothingUntilTheNextPublish {
                checkpoint_txg: current_txg,
                switches,
                stop,
            });
    }

    /// 备忘记着的停这一刻还算不算数：现行那一版与开关都没变才算，交回那一次的停。
    #[must_use]
    pub(crate) fn stop_still_holding_at(
        &self,
        current_txg: CheckpointTxg,
        switches: CompactionSwitches,
    ) -> Option<CompactionRunStop> {
        self.stop_that_publishes_nothing_until_the_next_publish
            .filter(|noted| noted.checkpoint_txg == current_txg && noted.switches == switches)
            .map(|noted| noted.stop)
    }

    /// 这次挂载那一份预算还要不要从根环读，并记下读过了（读没读通都只读这一次）。
    pub(crate) fn take_the_reading_of_the_publishes_of_the_mount_from_the_ring(&mut self) -> bool {
        std::mem::replace(
            &mut self.publishes_of_the_mount_are_still_to_be_read_from_the_ring,
            false,
        )
    }

    /// 发成一次用户改动：已经过去的发布数从 0 数，根环里最新的用户状态就是这一次，挂载那一份不用再读。
    pub(crate) fn note_a_user_change_published(&mut self) {
        self.publishes_since_the_last_user_change = 0;
        self.publishes_of_the_mount_are_still_to_be_read_from_the_ring = false;
    }

    /// 取走还没完的那一趟（放锁的后台入口接着推它时）。
    pub(crate) fn take_the_background_trip_under_way(&mut self) -> Option<BackgroundTripUnderWay> {
        self.background_trip_under_way.take()
    }

    /// 记下这一趟这次调用之后还没完。
    pub(crate) fn keep_the_background_trip_under_way(&mut self, trip: BackgroundTripUnderWay) {
        self.background_trip_under_way = Some(trip);
    }

    /// 放锁的后台入口有没有一趟没完（运行库按它决定多久之后再醒）。
    #[must_use]
    pub fn has_a_background_trip_under_way(&self) -> bool {
        self.background_trip_under_way.is_some()
    }

    /// 可写挂载读盘建上一版时只坏一份的次数（[`Self::count_unit_reads_with_one_copy_failing_while_mounting`] 记的）。
    #[must_use]
    pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
        self.unit_reads_with_one_copy_failing_while_mounting
    }

    /// 把可写挂载读盘建上一版时只坏一份的次数加进这一份挂载期状态（挂载交回之前调一次）。
    ///
    /// # Panics
    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
    pub(crate) fn count_unit_reads_with_one_copy_failing_while_mounting(
        &mut self,
        reads_while_mounting: u64,
    ) {
        self.unit_reads_with_one_copy_failing_while_mounting = self
            .unit_reads_with_one_copy_failing_while_mounting
            .checked_add(reads_while_mounting)
            .expect("一次挂载里只坏一份的读次数装得进 u64");
    }

    /// 可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（[`Self::keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting`]
    /// 记的，会话下一次用户改动发成之后取走）。
    #[must_use]
    pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &self,
    ) -> &[Vec<u8>] {
        &self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
    }

    /// 记下读到只坏一份的那几个单元的映射 key（挂载交回之前调一次，接在已记的后面、同一把只记一次）。
    pub(crate) fn keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &mut self,
        mapping_keys: Vec<Vec<u8>>,
    ) {
        for mapping_key in mapping_keys {
            if !self
                .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
                .contains(&mapping_key)
            {
                self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
                    .push(mapping_key);
            }
        }
    }

    /// 取走记着的那几把 key（交给另发那一批），这一份挂载期状态里清空：另发那一批不管搬没搬成都只试这一次，被拒的结局随那次改动报出。
    pub(crate) fn take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &mut self,
    ) -> Vec<Vec<u8>> {
        std::mem::take(
            &mut self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting,
        )
    }

    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。产品路径一处都不调它。
    pub fn force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
        &mut self,
        forcing: RelocationBatchAfterTheChangeRefusedForSpace,
    ) {
        self.relocation_batch_after_the_change_refused_for_space = forcing;
    }

    /// 会话另发那一批的这一次该不该当作被空间准入拒（开关用掉一次就记下：「接下来这几次」少一次，用完回到产品值）。
    pub(crate) fn take_a_forced_space_refusal_of_the_relocation_batch_after_the_change(
        &mut self,
    ) -> bool {
        match self.relocation_batch_after_the_change_refused_for_space {
            RelocationBatchAfterTheChangeRefusedForSpace::NotForced => false,
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedOnEveryAttemptForTests => true,
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts,
            } => {
                self.relocation_batch_after_the_change_refused_for_space =
                    match NonZeroU64::new(attempts.get() - 1) {
                        Some(attempts_left) => {
                            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                                attempts: attempts_left,
                            }
                        }
                        None => RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
                    };
                true
            }
        }
    }

    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。
    /// 产品路径一处都不调它。
    pub fn force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
        &mut self,
        forcing: RelocationBatchAfterTheChangeRefusedNotForSpace,
    ) {
        self.relocation_batch_after_the_change_refused_not_for_space = forcing;
    }

    /// 会话另发那一批的这一次该不该当作被拒、不是空间不够。
    #[must_use]
    pub(crate) fn is_the_relocation_batch_after_the_change_forced_refused_not_for_space(
        &self,
    ) -> bool {
        match self.relocation_batch_after_the_change_refused_not_for_space {
            RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced => false,
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests => true,
        }
    }

    /// 装上只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。产品路径一处都不调它。
    pub fn force_the_floor_raise_for_the_relocation_batch_after_the_change_to_fail_for_tests(
        &mut self,
        forcing: FloorRaiseForTheRelocationBatchAfterTheChangeFails,
    ) {
        self.floor_raise_for_the_relocation_batch_after_the_change_fails = forcing;
    }

    /// 为另发那一批推的这一串抬 F 该不该当作自己报了错。
    #[must_use]
    pub(crate) fn is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail(
        &self,
    ) -> bool {
        match self.floor_raise_for_the_relocation_batch_after_the_change_fails {
            FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced => false,
            FloorRaiseForTheRelocationBatchAfterTheChangeFails::ForcedOnEveryRaiseForTests => true,
        }
    }

    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（各次发布之和）。
    #[must_use]
    pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
        self.old_data_unit_reads_back_with_one_copy_failing
    }

    /// 把一次用户改动里各次发布读回时只坏一份的次数加进这次挂载的累计。
    ///
    /// # Panics
    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
    pub(crate) fn count_old_data_unit_reads_back_with_one_copy_failing(
        &mut self,
        reads_of_the_change: u64,
    ) {
        self.old_data_unit_reads_back_with_one_copy_failing = self
            .old_data_unit_reads_back_with_one_copy_failing
            .checked_add(reads_of_the_change)
            .expect("一次挂载里只坏一份的读回次数装得进 u64");
    }

    /// 定时兜底这一次该不该再查一次起跑判据（D26（后台整理与放置回收） 已定项 13 第 5 条「另有定时兜底，每隔一段时间再查一次」）：
    /// 关着就不查；开着而还没查过、或离上一次查过了一个间隔，就该查，并把 `now` 记成上一次查的读数。调用方的钟往回走时按没过间隔算。
    pub(crate) fn check_the_timer_fallback(
        &mut self,
        timer_fallback: CompactionTimerFallback,
        now: CallerMonotonicClockReading,
    ) -> TimerFallbackCheck {
        let interval = match timer_fallback {
            CompactionTimerFallback::Off => return TimerFallbackCheck::Off,
            CompactionTimerFallback::EveryMilliseconds(interval_in_milliseconds) => {
                Duration::from_millis(interval_in_milliseconds.get())
            }
        };
        let is_due = match self.timer_fallback_last_check {
            None => true,
            Some(last_check) => now.0.saturating_sub(last_check.0) >= interval,
        };
        if is_due {
            self.timer_fallback_last_check = Some(now);
            TimerFallbackCheck::Due
        } else {
            TimerFallbackCheck::IntervalNotElapsed
        }
    }

    /// 写带宽上限放不放下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）：不限就放；限着就看这一个窗口——没有窗口、或离窗口起点过了
    /// [`COMPACTION_WRITE_BANDWIDTH_WINDOW`]，在 `now` 开一个新窗口；窗口里已写的字节到了上限就不放。所以一个窗口里整理至多写上限加一批的字节。
    ///
    /// # Errors
    /// [`CompactionRunStop::WriteBandwidthLimitReached`]：这一个窗口里已写的字节不少于上限。
    pub(crate) fn admit_the_next_batch_under_the_write_bandwidth_limit(
        &mut self,
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        now: CallerMonotonicClockReading,
    ) -> Result<(), CompactionRunStop> {
        let limit_in_bytes_per_second = match write_bandwidth_limit {
            CompactionWriteBandwidthLimit::Unlimited => return Ok(()),
            CompactionWriteBandwidthLimit::BytesPerSecond(limit) => limit.get(),
        };
        let window = match self.write_bandwidth_window {
            Some(window)
                if now.0.saturating_sub(window.started_at.0)
                    < COMPACTION_WRITE_BANDWIDTH_WINDOW =>
            {
                window
            }
            Some(_) | None => CompactionWriteBandwidthWindow {
                started_at: now,
                bytes_written: 0,
            },
        };
        self.write_bandwidth_window = Some(window);
        if window.bytes_written >= limit_in_bytes_per_second {
            Err(CompactionRunStop::WriteBandwidthLimitReached {
                bytes_written_in_the_window: window.bytes_written,
                limit_in_bytes_per_second,
            })
        } else {
            Ok(())
        }
    }

    /// 后台入口发成一次整理发布之后记下它写的字节（限着写带宽时才记；窗口在放这一批时已开好）。
    pub(crate) fn note_the_bytes_written_by_a_compaction_publish(
        &mut self,
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        bytes_written: u64,
    ) {
        match write_bandwidth_limit {
            CompactionWriteBandwidthLimit::Unlimited => {}
            CompactionWriteBandwidthLimit::BytesPerSecond(_) => {
                let window = self
                    .write_bandwidth_window
                    .as_mut()
                    .expect("限着写带宽时放这一批之前开了窗口（admit_the_next_batch_under_the_write_bandwidth_limit）");
                window.bytes_written = window.bytes_written.saturating_add(bytes_written);
            }
        }
    }

    /// 这次挂载里释放前核出对不上而隔离的槽（常驻隔离集）。
    #[must_use]
    pub fn quarantined_slots(&self) -> &BTreeSet<SlotNumber> {
        &self.quarantined_slots
    }

    /// 把发布路径记在分配器上、还没取走的隔离落点并进常驻隔离集（跨度里的每一个槽）。
    pub fn take_the_quarantines_noted_by_the_publishes(
        &mut self,
        on_the_allocator: &mut CompactionSwitchesAndTheExcludedRegion,
    ) {
        for (slot, span) in on_the_allocator.take_the_quarantines_noted_by_the_publishes() {
            for covered in slot.0..slot.0 + span {
                self.quarantined_slots.insert(SlotNumber(covered));
            }
        }
    }
}

/// 一块盘上按记录口径的全空聚簇段数（D3（空间分配） 已定项 10 ① 与 D26（后台整理与放置回收） 已定项 1 第 1 条收口的那一句：
/// 段内 64 槽都没有未释放分配记录的段数，C372）。只数单元区里的整段。整理（签发时的 `segs0`、一条意图完成之后「无产出就停」）
/// 从这一版的分配记录现扫它；记账第 11 项与合取 7 读分配器按同一个定义增量维护的那一份（`allocator::DeviceFreeMap::empty_segments`）：
/// 定义相同，算法不同（准入路径上不许扫，整理在后台可以扫）。
#[must_use]
pub fn empty_cluster_segments_by_the_records(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> u64 {
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let unit_area_start = unit_area.start.slot().0;
    let mut segments_with_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
                let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
                if segment < full_segments {
                    segments_with_an_unreleased_record.insert(segment);
                }
            }
        }
    }
    full_segments - u64::try_from(segments_with_an_unreleased_record.len()).expect("段数")
}

/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条）的左边：R 那块盘上、R 之外、「非全空」的段里没挡的槽数。
/// 「非全空」按记录口径；只装提交内生块的段不算非全空——段里至少一个槽罩在 `slots_holding_user_data`（码 1 数据单元的落点，
/// 两盘同槽只比槽号）里才算。`is_unblocked(槽)` 说那一槽此刻能不能当目的地（已分配、隔离、抬 F 扣住三种位一个都没置）。
#[must_use]
pub fn holes_outside_the_region_in_segments_holding_user_data<IsUnblocked>(
    allocation_records: &[AllocationRecord],
    slots_holding_user_data: &BTreeSet<SlotNumber>,
    unit_area: UnitAreaOfOneDevice,
    region: CompactionRegion,
    is_unblocked: IsUnblocked,
) -> u64
where
    IsUnblocked: Fn(SlotNumber) -> bool,
{
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let mut segments_holding_user_data: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        let holds_user_data = (record.slot.0..record.slot.0 + u64::from(record.span_slots))
            .any(|slot| slots_holding_user_data.contains(&SlotNumber(slot)));
        if !holds_user_data {
            continue;
        }
        if let Some(offset_in_unit_area) = record.slot.0.checked_sub(unit_area_start) {
            let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
            if segment < full_segments {
                segments_holding_user_data.insert(segment);
            }
        }
    }
    segments_holding_user_data
        .into_iter()
        .map(|segment| unit_area_start + segment * CLUSTER_SEGMENT_SLOTS)
        .filter(|segment_start| !region.contains_slot(SlotNumber(*segment_start)))
        .map(|segment_start| {
            u64::try_from(
                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
                    .filter(|slot| is_unblocked(SlotNumber(*slot)))
                    .count(),
            )
            .expect("一段 64 槽")
        })
        .sum()
}

/// 单元区里没有一条未释放记录罩到的整段，各交一份空的扫描结果（[`crate::region_scan::scan_region`] 对这样的段交回的就是它：
/// 没有起点在段里的记录，也没有跨进段里的记录）。选段之前带上它们，[`choose_region`] 就不为每个全空段各要一次「先扫」——
/// 否则一次签发要扫单元区的每一个全空段、每扫一段再选一遍。只读记录，不读设备。
#[must_use]
pub fn region_scans_of_the_segments_without_an_unreleased_record(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<RegionScan> {
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let mut segments_touched_by_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
                segments_touched_by_an_unreleased_record
                    .insert(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS);
            }
        }
    }
    (0..full_segments)
        .filter(|segment| !segments_touched_by_an_unreleased_record.contains(segment))
        .map(|segment| RegionScan {
            region: CompactionRegion::of_one_cluster_segment(
                unit_area.device,
                SlotNumber(unit_area_start + segment * CLUSTER_SEGMENT_SLOTS),
            )
            .expect("单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数"),
            allocations: Vec::new(),
        })
        .collect()
}

/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条）：空洞数 ≥ R 里要搬的单元数才签。
#[must_use]
pub const fn holes_admit_the_issue(holes: u64, units_to_move: u64) -> bool {
    holes >= units_to_move
}

/// 停机谓词（D26（后台整理与放置回收） 已定项 1 第 2 条，判决 G4）：删意图那一刻 R 按记录全空——R 那块盘上起点在 R 里的未释放记录一条都没有。
#[must_use]
pub fn region_is_empty_by_the_records(
    allocation_records: &[AllocationRecord],
    region: CompactionRegion,
) -> bool {
    !allocation_records.iter().any(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    })
}

/// 一条意图完成之后签不签下一条：全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为真才签。
#[must_use]
pub const fn the_completed_intent_yielded(
    empty_cluster_segments_now: u64,
    empty_cluster_segments_at_issue: u64,
) -> bool {
    empty_cluster_segments_now > empty_cluster_segments_at_issue
}

/// 签发之后 R 里出现了隔离槽没有（一个槽既钉住又在隔离集里，区域扫描判「钉住」照旧，这一判另外直接查隔离集，
/// 两者同时成立也算有）。
#[must_use]
pub fn region_holds_a_quarantined_slot(
    region: CompactionRegion,
    quarantined_slots: &BTreeSet<SlotNumber>,
) -> bool {
    quarantined_slots
        .range(region.start_slot()..region.end_slot())
        .next()
        .is_some()
}

/// 用户改动发成之后起跑判据查出的结果。整理挪出了用户改动的调用线程（
/// D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：发布路径只查、一批都不发，查出「该跑了」记在会话上，
/// 由后台入口（`mounted_session::MountedSession::run_compaction_in_the_background`）取走去跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionStartJudgement {
    NotDue,
    /// 一次用户改动发成之后起跑判据为真，后台入口还没来取；几次改动之间没来取的并成这一个。
    DueAfterAUserChange,
}

/// 调用方单调钟的一次读数（从调用方自己定的起点算）：核心层不起线程、不读钟，后台入口的定时兜底与写带宽上限只读调用方交进来的读数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallerMonotonicClockReading(pub Duration);

/// 定时兜底（D26（后台整理与放置回收） 已定项 13 第 5 条「另有定时兜底，每隔一段时间再查一次，间隔是挂载参数」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionTimerFallback {
    Off,
    /// 每隔这么多毫秒查一次起跑判据。
    EveryMilliseconds(NonZeroU64),
}

/// 定时兜底间隔的默认：关。核心层不起线程，调用方不给间隔就没有定时兜底；挂载就起的周期线程那一层给它的间隔是周期的一半
/// （`singlefs_mount_runtime`，周期默认 30 秒）。
pub const COMPACTION_TIMER_FALLBACK_BY_DEFAULT: CompactionTimerFallback =
    CompactionTimerFallback::Off;

/// 定时兜底这一次查不查（[`CompactionOfThisMount::check_the_timer_fallback`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerFallbackCheck {
    Off,
    IntervalNotElapsed,
    Due,
}

/// 整理占盘写带宽的上限（D26（后台整理与放置回收） 已定项 13 第 6 条，挂载参数）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionWriteBandwidthLimit {
    Unlimited,
    BytesPerSecond(NonZeroU64),
}

/// 写带宽上限的默认：不限。默认值待 C412（整理的挂钟代价没量过） 与
/// E10（老化：结构抗老化 vs 事后整理）。
pub const COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT: CompactionWriteBandwidthLimit =
    CompactionWriteBandwidthLimit::Unlimited;

/// 写带宽上限按字节每秒给，一个窗口就是一秒（[`CompactionOfThisMount::admit_the_next_batch_under_the_write_bandwidth_limit`]）。
pub const COMPACTION_WRITE_BANDWIDTH_WINDOW: Duration = Duration::from_secs(1);

/// 写带宽的一个窗口：从哪一刻起、窗口里整理已写了多少字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CompactionWriteBandwidthWindow {
    started_at: CallerMonotonicClockReading,
    bytes_written: u64,
}

/// 后台入口的节奏参数（D26（后台整理与放置回收） 已定项 13 第 5、6 条；每次调用由调用方交进来，挂载参数那一层有了之后从那里取）。
/// 定时兜底那几趟没有自己的发布预算：与用户改动共用同一份 `B_gc`（[`publishes_left_of_the_budget_given_by_the_last_user_change`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionPacing {
    pub timer_fallback: CompactionTimerFallback,
    pub write_bandwidth_limit: CompactionWriteBandwidthLimit,
}

impl CompactionPacing {
    /// 两样都取默认（[`COMPACTION_TIMER_FALLBACK_BY_DEFAULT`]、[`COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT`]）。
    pub const BY_DEFAULT: CompactionPacing = CompactionPacing {
        timer_fallback: COMPACTION_TIMER_FALLBACK_BY_DEFAULT,
        write_bandwidth_limit: COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
    };
}

/// 这一批之前调用方的回答（D26（后台整理与放置回收） 已定项 13 第 6 条「前台有 fsync 在等时不发下一批」）：发不发下一批，要不要让路。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerOfTheCallerBeforeTheNextBatch {
    NoForegroundFsyncWaiting,
    ForegroundFsyncIsWaiting,
    /// 前台不一定有 fsync 在等，是调用方在停这条驱动后台入口的线程：照样不发下一批，低于底额的破例也不发（线程很快 join 得上）。
    CallerIsStoppingItsThread,
}

/// 后台入口每推一批之前问调用方的两样：此刻的钟、这一批之前它的回答。核心层不起线程，调用方在自己的线程上驱动后台入口，
/// 钟与前台的 fsync 队列都在调用方手里；测试里由一份假的实现。
pub trait CallerOfTheBackgroundCompaction {
    fn clock_reading(&self) -> CallerMonotonicClockReading;
    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch;
}

/// 回退门给根环留的余量（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：一次整理发布之后，再发这么多次发布，根环里仍有 4 个不同用户状态。
/// 取每区根槽数 S（根环 3S 槽的三分之一）：最新的 4 个状态之后门放 `3S − 4 − S = 2S − 4` 次发布，S = 5 时是 6 次，
/// 产品批大小 16 下一条整段意图（至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布）走得完；S = 4 时是 4 次，走不完就停在门上报出来。
/// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次。S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。
/// 连崩几次用完这几槽之后，挂载自己的发布照样挤掉用户状态。
/// 这笔算术不算 `accept` 被空间拒时当场推的抬 F 那几串（`mounted_session::MountedSessionWithPendingChanges::accept`）：它们不经门，推一串也用掉这几槽。
#[must_use]
pub fn root_ring_slots_left_by_a_compaction_publish(
    root_ring_slots_per_region: RootRingSlotsPerRegion,
) -> u64 {
    root_ring_slots_per_region.count()
}

/// 一次整理运行的发布预算：
/// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。
#[must_use]
pub fn publish_budget_of_a_compaction_run(
    root_ring_slots_per_region: u64,
    publishes_already_passed: u64,
) -> u64 {
    (3 * root_ring_slots_per_region)
        .saturating_sub(4)
        .saturating_sub(publishes_already_passed)
        .max(1)
}

/// 后台入口的一趟还能发几次整理发布（与用户改动共用
/// `B_gc`）：一次用户改动给出一份 `B_gc`（[`publish_budget_of_a_compaction_run`] 在已经过去的发布数为 0 时的值，至少 1），之后用户改动那一趟
/// 与定时兜底那几趟都从这同一份里扣；扣完是 0，等下一次用户改动（会话发成一次用户改动就把已经过去的发布数归零）才再有。
/// 不像显式入口那样每一趟至少 1：那一条加上定时兜底，闲置时整理发布就没有上限（200 拍连着 213 次，线是 `3S − 4`）。
#[must_use]
pub fn publishes_left_of_the_budget_given_by_the_last_user_change(
    root_ring_slots_per_region: u64,
    publishes_since_the_last_user_change: u64,
) -> u64 {
    publish_budget_of_a_compaction_run(root_ring_slots_per_region, 0)
        .saturating_sub(publishes_since_the_last_user_change)
}

/// 整理的两样挂载参数（都不存盘）：批大小与三条水位线（[`crate::mount::mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionMountParameters {
    pub batch_units: CompactionBatchUnits,
    pub watermarks: CompactionWatermarks,
}

impl CompactionMountParameters {
    /// 两样都取产品默认。
    pub const BY_DEFAULT: CompactionMountParameters = CompactionMountParameters {
        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
        watermarks: CompactionWatermarks::BY_DEFAULT,
    };
}

/// 全空聚簇段线的产品默认：整段数的 5%（D26（后台整理与放置回收） 已定项 1，数是拍的）。
pub const EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT: u64 = 5;
/// 全空聚簇段线的下限：2 段（前台写入类发布在全空段低于 `N₀` = 1 时被合取 7 拒，线至少 `N₀ + 1`，不然前台先被拒、整理才起跑）。
pub const EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT: u64 = 2;

/// 全空聚簇段这条水位（越线：全空聚簇段数 < 线）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyClusterSegmentsWatermark {
    Off,
    /// 线 = max(⌈整段数 × 百分比 ÷ 100⌉, 下限)，再不超过整段数（只有一两个整段的小盘刚 mkfs 时不越线）。
    PercentOfTheClusterSegments {
        percent: u64,
        lowest_line_in_segments: u64,
    },
}

impl EmptyClusterSegmentsWatermark {
    /// 这块盘的线（段数）；关着是 `None`。
    #[must_use]
    pub fn line_in_segments(self, cluster_segments_in_the_unit_area: u64) -> Option<u64> {
        match self {
            EmptyClusterSegmentsWatermark::Off => None,
            EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
                percent,
                lowest_line_in_segments,
            } => {
                let percent_of_the_segments = (u128::from(cluster_segments_in_the_unit_area)
                    * u128::from(percent))
                .div_ceil(100);
                let lowest = u128::from(lowest_line_in_segments);
                let line = percent_of_the_segments
                    .max(lowest)
                    .min(u128::from(cluster_segments_in_the_unit_area));
                Some(u64::try_from(line).expect("线不超过整段数"))
            }
        }
    }
}

/// 碎片度这条水位（越线：`空闲段数 ÷ 已分配槽数 > 线`，且空闲段数不少于绝对下限；D26 已定项 1 的 `F = runs / L`，`L` 取已分配槽数，
/// 条款没定义它，归 C326（`runs` 口径换了之后的 L 没定义））。默认关。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentationWatermark {
    Off,
    /// 线是 `free_runs_above ÷ per_allocated_slots`；`at_least_free_runs` 是这一支开了必带的绝对下限：
    /// 用得很少的盘上已分配槽数很小而空闲段数至少 1，只看比值近乎空盘就越线。
    FreeRunsPerAllocatedSlotAbove {
        free_runs_above: NonZeroU64,
        per_allocated_slots: NonZeroU64,
        at_least_free_runs: NonZeroU64,
    },
}

/// 不可回收字节这条水位（越线：不可回收字节 ≥ 线）。读数第一版恒 0（[`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]），开了也不越线；默认关。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnreclaimableBytesWatermark {
    Off,
    AtLeast(NonZeroU64),
}

/// 三条水位线（D26 已定项 1：三水位任一越线起跑，带设备维）。挂载参数，不存盘：盘上系统配置那 24 字节是全 0 的预留。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionWatermarks {
    pub empty_cluster_segments: EmptyClusterSegmentsWatermark,
    pub fragmentation: FragmentationWatermark,
    pub unreclaimable_bytes: UnreclaimableBytesWatermark,
}

impl CompactionWatermarks {
    /// 产品默认：全空聚簇段线 5%、下限 2；碎片度与不可回收字节关。
    pub const BY_DEFAULT: CompactionWatermarks = CompactionWatermarks {
        empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
            percent: EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT,
            lowest_line_in_segments: EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT,
        },
        fragmentation: FragmentationWatermark::Off,
        unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
    };
}

/// 一块盘此刻的三个量（分配器增量维护的计数，读一次 O(1)，与记账树写出去的是同一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatermarkReadingOfADevice {
    pub device: DeviceIdentity,
    pub empty_cluster_segments: u64,
    pub cluster_segments_in_the_unit_area: u64,
    pub free_runs: u64,
    pub allocated_slots: u64,
    pub unreclaimable_bytes: u64,
}

impl WatermarkReadingOfADevice {
    /// 从分配器上这块盘的空闲图读。全空聚簇段数是记录口径：一段里只剩 defer 槽也算全空
    /// （`allocator::DeviceFreeMap::empty_segments`，开段看的是另一个量，`allocator::DeviceFreeMap::lowest_empty_segment`）。
    #[must_use]
    pub fn of_the_device_map(device_map: &DeviceFreeMap) -> Self {
        WatermarkReadingOfADevice {
            device: device_map.device,
            empty_cluster_segments: device_map.empty_segments(),
            cluster_segments_in_the_unit_area: device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS,
            free_runs: device_map.free_runs(),
            allocated_slots: device_map.allocated_slots(),
            unreclaimable_bytes: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0,
        }
    }
}

/// 三个量。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkQuantity {
    EmptyClusterSegments,
    Fragmentation,
    UnreclaimableBytes,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：不看读数，判起跑判据时当这个量越了线。产品路径恒 [`Self::NotForced`]。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionWatermarkCrossingForcing {
    NotForced,
    ForcedForTests(WatermarkQuantity),
}

/// 一块盘上全空聚簇段的段数：全空聚簇段那条水位的读数与线。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmptyClusterSegmentCount(pub u64);

/// 一块盘上空闲段（连续空闲槽的一段）的段数：碎片度那条水位的读数，线换算到这块盘此刻的已分配槽数上也是它。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreeRunCount(pub u64);

/// 头一个越线的量，连同它的读数与线。三个量的单位各不相同，各带各的类型。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkCrossing {
    EmptyClusterSegments {
        reading: EmptyClusterSegmentCount,
        line: EmptyClusterSegmentCount,
    },
    /// `line` 是把线换算到这块盘的已分配槽数上的空闲段数。
    Fragmentation {
        reading: FreeRunCount,
        line: FreeRunCount,
    },
    UnreclaimableBytes {
        reading: BytesOnOneDevice,
        line: BytesOnOneDevice,
    },
}

/// 起跑判据的结论，连同走了哪一支（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkVerdict {
    /// 只供测试的开关强制起跑（[`CompactionRunForcing::ForcedToStartForTests`]）。
    ForcedToStartByTheTestOnlySwitch,
    /// 只供测试的开关强制不起跑（[`CompactionRunForcing::ForcedToStopForTests`]）。
    ForcedToStopByTheTestOnlySwitch,
    /// 只供测试的开关当这个量越了线（[`CompactionWatermarkCrossingForcing::ForcedForTests`]）。
    CrossingForcedByTheTestOnlySwitch(WatermarkQuantity),
    /// 每块盘的三个量都没越线。
    NoLineCrossed,
    /// 按盘的次序、每块盘按全空聚簇段、碎片度、不可回收字节的次序头一个越线的。
    Crossed {
        device: DeviceIdentity,
        crossing: WatermarkCrossing,
    },
}

impl WatermarkVerdict {
    /// 这一结论起不起跑。
    #[must_use]
    pub const fn starts(self) -> bool {
        match self {
            WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
            | WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(_)
            | WatermarkVerdict::Crossed { .. } => true,
            WatermarkVerdict::ForcedToStopByTheTestOnlySwitch | WatermarkVerdict::NoLineCrossed => {
                false
            }
        }
    }
}

/// 一块盘头一个越线的量（全空聚簇段、碎片度、不可回收字节的次序），连同读数与线。
fn watermark_crossed_on_the_device(
    watermarks: &CompactionWatermarks,
    reading: &WatermarkReadingOfADevice,
) -> Option<WatermarkCrossing> {
    if let Some(line) = watermarks
        .empty_cluster_segments
        .line_in_segments(reading.cluster_segments_in_the_unit_area)
    {
        if reading.empty_cluster_segments < line {
            return Some(WatermarkCrossing::EmptyClusterSegments {
                reading: EmptyClusterSegmentCount(reading.empty_cluster_segments),
                line: EmptyClusterSegmentCount(line),
            });
        }
    }
    match watermarks.fragmentation {
        FragmentationWatermark::Off => {}
        FragmentationWatermark::FreeRunsPerAllocatedSlotAbove {
            free_runs_above,
            per_allocated_slots,
            at_least_free_runs,
        } => {
            let line_on_this_device = u128::from(free_runs_above.get())
                * u128::from(reading.allocated_slots)
                / u128::from(per_allocated_slots.get());
            let is_above_the_line = u128::from(reading.free_runs)
                * u128::from(per_allocated_slots.get())
                > u128::from(free_runs_above.get()) * u128::from(reading.allocated_slots);
            if reading.allocated_slots > 0
                && is_above_the_line
                && reading.free_runs >= at_least_free_runs.get()
            {
                return Some(WatermarkCrossing::Fragmentation {
                    reading: FreeRunCount(reading.free_runs),
                    line: FreeRunCount(
                        u64::try_from(line_on_this_device).expect("低于空闲段数读数，装得进 u64"),
                    ),
                });
            }
        }
    }
    match watermarks.unreclaimable_bytes {
        UnreclaimableBytesWatermark::Off => None,
        UnreclaimableBytesWatermark::AtLeast(line) => (reading.unreclaimable_bytes >= line.get())
            .then_some(WatermarkCrossing::UnreclaimableBytes {
                reading: BytesOnOneDevice(reading.unreclaimable_bytes),
                line: BytesOnOneDevice(line.get()),
            }),
    }
}

/// 起跑判据（水位越线再跑；D26（后台整理与放置回收） 已定项 1、已定项 13 第 5 条：定时兜底每隔一段时间查一次，每次用户改动发成之后也查一次，
/// 两处查的都是它）：任一块盘的任一个量越线就起跑。只读交进来的读数，代价随盘数、不随盘容量（`.claude/rules/fs-design.md`「记账是事务的副产品」第一格）。
/// 水位只在起跑时查，停机不靠水位（D26 已定项 1 第 2 条），所以没有回滞。
#[must_use]
pub fn compaction_starts_by_the_watermarks(
    forcing: CompactionRunForcing,
    watermarks: &CompactionWatermarks,
    readings: &[WatermarkReadingOfADevice],
) -> WatermarkVerdict {
    match forcing {
        CompactionRunForcing::ForcedToStartForTests => {
            return WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
        }
        CompactionRunForcing::ForcedToStopForTests => {
            return WatermarkVerdict::ForcedToStopByTheTestOnlySwitch
        }
        CompactionRunForcing::FollowTheWatermarks => {}
    }
    for reading in readings {
        if let Some(crossing) = watermark_crossed_on_the_device(watermarks, reading) {
            return WatermarkVerdict::Crossed {
                device: reading.device,
                crossing,
            };
        }
    }
    WatermarkVerdict::NoLineCrossed
}

/// 这个分配器此刻的起跑判据结论：开关、线与每块盘的读数都从它取（[`compaction_starts_by_the_watermarks`]）。
#[must_use]
pub fn compaction_watermark_verdict_of_the_allocator(
    allocator: &PoolAllocator,
) -> WatermarkVerdict {
    let compaction = allocator.compaction();
    match (
        compaction.switches.run_forcing,
        compaction.watermark_crossing_forcing,
    ) {
        (
            CompactionRunForcing::FollowTheWatermarks,
            CompactionWatermarkCrossingForcing::ForcedForTests(quantity),
        ) => WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(quantity),
        (
            CompactionRunForcing::FollowTheWatermarks
            | CompactionRunForcing::ForcedToStartForTests
            | CompactionRunForcing::ForcedToStopForTests,
            CompactionWatermarkCrossingForcing::NotForced
            | CompactionWatermarkCrossingForcing::ForcedForTests(_),
        ) => {
            let readings: Vec<WatermarkReadingOfADevice> = allocator
                .devices
                .iter()
                .map(WatermarkReadingOfADevice::of_the_device_map)
                .collect();
            compaction_starts_by_the_watermarks(
                compaction.switches.run_forcing,
                &compaction.watermarks,
                &readings,
            )
        }
    }
}

/// 全空段低于底额、前台写入类发布已写不进（合取 7，`N₀` 是开关给的那一档）：任一块盘低于就算。后台入口的破例（每趟至少发 1 次）判它，代价随盘数。
#[must_use]
pub fn foreground_writes_are_below_the_empty_cluster_segments_floor(
    allocator: &PoolAllocator,
) -> bool {
    let floor = allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments();
    allocator
        .devices
        .iter()
        .any(|device_map| device_map.empty_segments() < floor)
}

/// 与 Verus 证过的选段核、计划核逐项对拍（`.claude/kb/milestone/04-fourth-txn.md` 二·4；
/// 第三节 G7 的 P1、P2、P3 证在 `singlefs_replay_prefix::compaction_plan_proofs` 的两个核上）：小输入上每一种形状都喂给这里的
/// [`choose_region`]、[`plan_batch`] 与核，答案逐项相同。核那一侧的入参由用例按形状直接写出，不经这里的代码算，
/// 所以两边对上就是这里的实现在这些输入上合核证过的性质。产品直接调用核；测试保留独立的原算法，核的证明不覆盖输入转换。
#[cfg(test)]
mod tests {
    #[allow(
        clippy::too_many_arguments,
        reason = "独立参考保留原选段算法的完整输入"
    )]
    fn reference_choose_region(
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        cluster_segments: &BTreeSet<SlotNumber>,
        live_intent_region: Option<CompactionRegion>,
        pinned_slots: &BTreeSet<SlotNumber>,
        region_scans: &[RegionScan],
        segments_refused_by_the_region_scan: &SegmentsRefusedByTheRegionScan,
        criterion: RegionChoiceCriterion,
    ) -> RegionChoice {
        // 按段起点索引一次（同一段有几份时取最前那一份）：下面每个候选段查一次，不逐份线性找。
        let mut scans_by_start_slot: BTreeMap<SlotNumber, &RegionScan> = BTreeMap::new();
        for scan in region_scans
            .iter()
            .filter(|scan| scan.region.device() == unit_area.device)
        {
            scans_by_start_slot
                .entry(scan.region.start_slot())
                .or_insert(scan);
        }
        let scan_of_the_segment =
            |start_slot: SlotNumber| scans_by_start_slot.get(&start_slot).copied();
        let unit_area_start = unit_area.start.slot().0;
        let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
        let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let mut live_slots_per_segment =
            vec![0u64; usize::try_from(full_segments).expect("一块盘的段数装得进 usize")];
        for record in allocation_records
            .iter()
            .filter(|record| record.device == unit_area.device && !record.is_released)
        {
            // 迭代上界是这条记录的跨度；只数落在整段里的槽。
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) else {
                    continue;
                };
                let segment = usize::try_from(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS)
                    .expect("段号装得进 usize");
                if let Some(live_slots) = live_slots_per_segment.get_mut(segment) {
                    *live_slots += 1;
                }
            }
        }
        let eligible_segments = live_slots_per_segment
            .iter()
            .enumerate()
            .map(|(segment, live_slots)| {
                let start_slot = SlotNumber(
                    unit_area_start
                        + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
                );
                (start_slot, *live_slots)
            })
            .filter(|(start_slot, _live_slots)| {
                let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
                let opened_in_this_mount = cluster_segments.contains(start_slot);
                let is_the_live_intent_region =
                    live_intent_region.is_some_and(|region| region.start_slot() == *start_slot);
                let holds_a_pinned_slot =
                    pinned_slots.range(*start_slot..end_slot).next().is_some();
                let refused_by_the_region_scan =
                    segments_refused_by_the_region_scan.contains(*start_slot);
                // 没扫过的段还不知道有没有能搬的单元：留作候选，轮到它时让调用方先扫。
                let may_hold_a_movable_unit = match scan_of_the_segment(*start_slot) {
                    Some(scan) => scan.movable_units().next().is_some(),
                    None => true,
                };
                !opened_in_this_mount
                    && !is_the_live_intent_region
                    && !holds_a_pinned_slot
                    && !refused_by_the_region_scan
                    && may_hold_a_movable_unit
            });
        let first_start_slot = match criterion {
            RegionChoiceCriterion::Greedy => eligible_segments
                .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))
                .map(|(start_slot, _live_slots)| start_slot),
        };
        let Some(first_start_slot) = first_start_slot else {
            return RegionChoice::NoEligibleSegment;
        };
        let first_region = CompactionRegion::of_one_cluster_segment(unit_area.device, first_start_slot)
            .expect(
                "单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数；槽号来自一块盘的单元区，装得进 6 字节",
            );
        match scan_of_the_segment(first_start_slot) {
            Some(_scan_with_a_movable_unit) => RegionChoice::Chosen(first_region),
            None => RegionChoice::ScanBeforeChoosing(first_region),
        }
    }

    use std::collections::{BTreeMap, BTreeSet};

    use singlefs_format::CLUSTER_SEGMENT_SLOTS;
    use singlefs_replay_prefix::compaction_plan_proofs::{
        choose_compaction_segment, CompactionKernelRegionChoice, CompactionKernelSegment,
        CompactionKernelSegmentScan,
    };

    use super::{
        choose_region, choose_region_over_the_live_slots, compaction_kernel_allocations,
        greedy_score, live_slots_per_cluster_segment_by_the_records, plan_batch,
        plan_batch_skipping_the_units_this_publish_rewrites_anyway, CompactionBatchUnits,
        LiveSlotsOfAClusterSegment, PlannedBatch, PlannedRelocation, RegionChoice,
        RegionChoiceCriterion, SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
    use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
    use crate::logged_operations::CompactionRegion;
    use crate::region_scan::{
        AllocationInRegion, MappedUnitClass, MovableUnit, RegionScan, RegionScanInconsistency,
        RegionScanRefusal, ScannedAllocation,
    };

    const DEVICE_OF_THE_UNIT_AREA: DeviceIdentity = DeviceIdentity(0);
    /// 另一块盘上的记录：选段只数单元区那块盘上的活槽，核的入参里没有它们。
    const ANOTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);
    const MOST_SEGMENTS_IN_THE_UNIT_AREA: usize = 3;
    /// 一段里各条活记录的跨度（槽数），一条一项（输入域加跨度 2 的记录、
    /// 段内多于 2 条活记录）。选段数的是活槽、不是条数：跨度 2 的一条与跨度 1 的两条活槽相同、条数不同，跨度 2 的一条与跨度 1 的一条条数相同、
    /// 活槽不同，打分改成数条数时这两种并排的段排序就变了（`crates/mutations.tsv` 一行）。
    const LIVE_RECORD_SPANS_OF_A_SEGMENT: [&[u16]; 5] = [&[], &[1], &[2], &[1, 1], &[1, 1, 1]];
    const MOST_ALLOCATIONS_IN_A_REGION: usize = 4;
    const LARGEST_BATCH_UNITS: usize = 3;
    /// 每段里一条已释放的记录落在这个偏移上：选段不数它，核的入参里也没有它。
    const RELEASED_RECORD_OFFSET_IN_THE_SEGMENT: u64 = 40;
    /// 钉住的槽落在段里的这个偏移上。
    const PINNED_SLOT_OFFSET_IN_THE_SEGMENT: u64 = 50;
    /// 计划用例里一条记录的跨度（码 1 数据单元）。
    const SPAN_OF_A_PLANNED_ALLOCATION: u64 = 2;

    /// 一段不合资格的那一条（每段至多一条：几条一起不合格时核与这里都只看合取，各条单独对得上就够）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SegmentExclusion {
        NotExcluded,
        OpenedInThisMount,
        HoldsAPinnedSlot,
        RefusedByTheRegionScan,
    }

    const SEGMENT_EXCLUSIONS: [SegmentExclusion; 4] = [
        SegmentExclusion::NotExcluded,
        SegmentExclusion::OpenedInThisMount,
        SegmentExclusion::HoldsAPinnedSlot,
        SegmentExclusion::RefusedByTheRegionScan,
    ];

    const SEGMENT_SCANS: [CompactionKernelSegmentScan; 3] = [
        CompactionKernelSegmentScan::NotScanned,
        CompactionKernelSegmentScan::ScannedWithAMovableUnit,
        CompactionKernelSegmentScan::ScannedWithoutAMovableUnit,
    ];

    #[derive(Clone, Copy, Debug)]
    struct SegmentShape {
        live_record_spans: &'static [u16],
        exclusion: SegmentExclusion,
        scan: CompactionKernelSegmentScan,
    }

    impl SegmentShape {
        /// 这一段的活槽数：各条活记录的跨度之和（核的入参照它直接写出，不经被测的选段算）。
        fn live_slots(&self) -> u64 {
            self.live_record_spans
                .iter()
                .map(|span_slots| u64::from(*span_slots))
                .sum()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum AllocationShape {
        Movable,
        Pinned,
        Quarantined,
        Dead,
        Inconsistent,
    }

    const ALLOCATION_SHAPES: [AllocationShape; 5] = [
        AllocationShape::Movable,
        AllocationShape::Pinned,
        AllocationShape::Quarantined,
        AllocationShape::Dead,
        AllocationShape::Inconsistent,
    ];

    /// `alphabet_size` 个字母组成的、长 `length` 的全部序列（每个位置一个字母下标）。
    fn every_sequence(alphabet_size: usize, length: usize) -> Vec<Vec<usize>> {
        let mut sequences: Vec<Vec<usize>> = vec![Vec::new()];
        for _position in 0..length {
            sequences = sequences
                .into_iter()
                .flat_map(|prefix| {
                    (0..alphabet_size).map(move |letter| {
                        let mut longer = prefix.clone();
                        longer.push(letter);
                        longer
                    })
                })
                .collect();
        }
        sequences
    }

    fn unit_area_start_slot() -> u64 {
        UnitAreaStart::of_the_default_journal_ring().slot().0
    }

    fn segment_start_slot(segment_index: usize) -> u64 {
        unit_area_start_slot()
            + u64::try_from(segment_index).expect("至多三段") * CLUSTER_SEGMENT_SLOTS
    }

    fn region_of_the_segment(segment_index: usize) -> CompactionRegion {
        CompactionRegion::of_one_cluster_segment(
            DEVICE_OF_THE_UNIT_AREA,
            SlotNumber(segment_start_slot(segment_index)),
        )
        .expect("默认单元区起点在聚簇段边界上，整段起点都是 64 的倍数")
    }

    fn allocation_record_of_generation_one(
        device: DeviceIdentity,
        slot: u64,
        span_slots: u16,
        is_released: bool,
    ) -> AllocationRecord {
        AllocationRecord {
            device,
            slot: SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(1),
            is_released,
            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    fn movable_unit(mapping_key: Vec<u8>) -> MovableUnit {
        MovableUnit {
            mapping_key,
            unit_class: MappedUnitClass::Data,
            birth_txg: CheckpointTxg(1),
        }
    }

    /// 一种形状喂给这里的 `choose_region` 交回的答案。
    fn choice_of_this_implementation(
        shapes: &[SegmentShape],
        live_intent_segment: Option<usize>,
    ) -> RegionChoice {
        let mut allocation_records = Vec::new();
        let mut cluster_segments = BTreeSet::new();
        let mut pinned_slots = BTreeSet::new();
        let mut region_scans = Vec::new();
        let mut segments_refused = SegmentsRefusedByTheRegionScan::default();
        for (segment_index, shape) in shapes.iter().enumerate() {
            let start_slot = segment_start_slot(segment_index);
            // 第 i 条活记录起在段起点之后 2i：跨度至多 2，各条互不重叠。
            for (live_record, span_slots) in (0_u64..).zip(shape.live_record_spans) {
                allocation_records.push(allocation_record_of_generation_one(
                    DEVICE_OF_THE_UNIT_AREA,
                    start_slot + 2 * live_record,
                    *span_slots,
                    false,
                ));
                allocation_records.push(allocation_record_of_generation_one(
                    ANOTHER_DEVICE,
                    start_slot + 2 * live_record + 1,
                    1,
                    false,
                ));
            }
            allocation_records.push(allocation_record_of_generation_one(
                DEVICE_OF_THE_UNIT_AREA,
                start_slot + RELEASED_RECORD_OFFSET_IN_THE_SEGMENT,
                1,
                true,
            ));
            let region = region_of_the_segment(segment_index);
            match shape.exclusion {
                SegmentExclusion::NotExcluded => {}
                SegmentExclusion::OpenedInThisMount => {
                    cluster_segments.insert(SlotNumber(start_slot));
                }
                SegmentExclusion::HoldsAPinnedSlot => {
                    pinned_slots.insert(SlotNumber(start_slot + PINNED_SLOT_OFFSET_IN_THE_SEGMENT));
                }
                SegmentExclusion::RefusedByTheRegionScan => segments_refused.note_refusal(
                    region,
                    RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                        device: DEVICE_OF_THE_UNIT_AREA,
                        slot: SlotNumber(start_slot),
                        span_slots: 2,
                    },
                ),
            }
            let scanned_class = match shape.scan {
                CompactionKernelSegmentScan::NotScanned => None,
                CompactionKernelSegmentScan::ScannedWithAMovableUnit => {
                    Some(AllocationInRegion::Movable(movable_unit(vec![
                        1,
                        u8::try_from(segment_index).expect("至多三段"),
                    ])))
                }
                CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => {
                    Some(AllocationInRegion::Dead)
                }
            };
            if let Some(class) = scanned_class {
                region_scans.push(RegionScan {
                    region,
                    allocations: vec![ScannedAllocation {
                        placement: Placement {
                            slot: SlotNumber(start_slot),
                            span: 1,
                        },
                        class,
                    }],
                });
            }
        }
        let reference = reference_choose_region(
            &allocation_records,
            UnitAreaOfOneDevice {
                device: DEVICE_OF_THE_UNIT_AREA,
                start: UnitAreaStart::of_the_default_journal_ring(),
                slots: u64::try_from(shapes.len()).expect("至多三段") * CLUSTER_SEGMENT_SLOTS,
            },
            &cluster_segments,
            live_intent_segment.map(region_of_the_segment),
            &pinned_slots,
            &region_scans,
            &segments_refused,
            RegionChoiceCriterion::Greedy,
        );
        let actual = choose_region(
            &allocation_records,
            UnitAreaOfOneDevice {
                device: DEVICE_OF_THE_UNIT_AREA,
                start: UnitAreaStart::of_the_default_journal_ring(),
                slots: u64::try_from(shapes.len()).expect("至多三段") * CLUSTER_SEGMENT_SLOTS,
            },
            &cluster_segments,
            live_intent_segment.map(region_of_the_segment),
            &pinned_slots,
            &region_scans,
            &segments_refused,
            RegionChoiceCriterion::Greedy,
        );
        assert_eq!(actual, reference, "产品输入转换必须与独立参考一致");
        actual
    }

    /// 同一种形状直接写成核的入参，核交回的下标换回段。
    fn choice_of_the_verified_kernel(
        shapes: &[SegmentShape],
        live_intent_segment: Option<usize>,
    ) -> RegionChoice {
        let segments: Vec<CompactionKernelSegment> = shapes
            .iter()
            .enumerate()
            .map(|(segment_index, shape)| CompactionKernelSegment {
                live_slots: shape.live_slots(),
                is_opened_in_this_mount: shape.exclusion == SegmentExclusion::OpenedInThisMount,
                is_the_live_intent_region: live_intent_segment == Some(segment_index),
                holds_a_pinned_slot: shape.exclusion == SegmentExclusion::HoldsAPinnedSlot,
                is_refused_by_the_region_scan: shape.exclusion
                    == SegmentExclusion::RefusedByTheRegionScan,
                scan: shape.scan,
            })
            .collect();
        match choose_compaction_segment(&segments) {
            CompactionKernelRegionChoice::Chosen(segment_index) => {
                RegionChoice::Chosen(region_of_the_segment(segment_index))
            }
            CompactionKernelRegionChoice::ScanBeforeChoosing(segment_index) => {
                RegionChoice::ScanBeforeChoosing(region_of_the_segment(segment_index))
            }
            CompactionKernelRegionChoice::NoEligibleSegment => RegionChoice::NoEligibleSegment,
        }
    }

    /// 一到三段、每段的活记录取 [`LIVE_RECORD_SPANS_OF_A_SEGMENT`] 那五种（零到三条、跨度 1 或 2）、至多一条不合资格、三种扫描结果、
    /// 活意图的 R 是哪一段（或没有），全部组合：这里的选段与证过 P1 的核逐项相同；三种答案都至少出现过一次（对拍不空转）。
    #[test]
    fn choose_region_agrees_with_the_verified_segment_choice_on_every_small_unit_area() {
        let shapes: Vec<SegmentShape> = LIVE_RECORD_SPANS_OF_A_SEGMENT
            .into_iter()
            .flat_map(|live_record_spans| {
                SEGMENT_EXCLUSIONS.into_iter().flat_map(move |exclusion| {
                    SEGMENT_SCANS.into_iter().map(move |scan| SegmentShape {
                        live_record_spans,
                        exclusion,
                        scan,
                    })
                })
            })
            .collect();
        let mut answers_by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
        for segments in 1..=MOST_SEGMENTS_IN_THE_UNIT_AREA {
            for shape_indices in every_sequence(shapes.len(), segments) {
                let segment_shapes: Vec<SegmentShape> = shape_indices
                    .iter()
                    .map(|shape_index| shapes[*shape_index])
                    .collect();
                let live_intent_choices = std::iter::once(None).chain((0..segments).map(Some));
                for live_intent_segment in live_intent_choices {
                    let this_implementation =
                        choice_of_this_implementation(&segment_shapes, live_intent_segment);
                    let verified_kernel =
                        choice_of_the_verified_kernel(&segment_shapes, live_intent_segment);
                    assert_eq!(
                        this_implementation, verified_kernel,
                        "段的形状 {segment_shapes:?}、活意图在段 {live_intent_segment:?}：这里的选段与证过的核不同"
                    );
                    let kind = match this_implementation {
                        RegionChoice::Chosen(_) => "chosen",
                        RegionChoice::ScanBeforeChoosing(_) => "scan_before_choosing",
                        RegionChoice::NoEligibleSegment => "no_eligible_segment",
                    };
                    *answers_by_kind.entry(kind).or_insert(0) += 1;
                }
            }
        }
        assert_eq!(
            answers_by_kind.len(),
            3,
            "三种答案都该出现过，实际 {answers_by_kind:?}"
        );
    }

    /// 一种形状的扫描结果：第 `position` 项的源落点在段起点之后 `2 × position`，能搬的 key 两两不同。
    fn region_scan_of(shapes: &[AllocationShape]) -> RegionScan {
        let start_slot = segment_start_slot(0);
        RegionScan {
            region: region_of_the_segment(0),
            allocations: shapes
                .iter()
                .enumerate()
                .map(|(position, shape)| ScannedAllocation {
                    placement: Placement {
                        slot: SlotNumber(
                            start_slot
                                + SPAN_OF_A_PLANNED_ALLOCATION
                                    * u64::try_from(position).expect("至多四项"),
                        ),
                        span: SPAN_OF_A_PLANNED_ALLOCATION,
                    },
                    class: match shape {
                        AllocationShape::Movable => {
                            AllocationInRegion::Movable(movable_unit(vec![
                                1,
                                u8::try_from(position).expect("至多四项"),
                            ]))
                        }
                        AllocationShape::Pinned => AllocationInRegion::Pinned,
                        AllocationShape::Quarantined => AllocationInRegion::Quarantined,
                        AllocationShape::Dead => AllocationInRegion::Dead,
                        AllocationShape::Inconsistent => AllocationInRegion::Inconsistent(
                            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
                        ),
                    },
                })
                .collect(),
        }
    }

    /// 独立参考保留直接 filter/take 的算法，避免产品调用核后与自己对拍。
    fn plan_of_the_independent_reference(
        region_scan: &RegionScan,
        batch_units: usize,
    ) -> PlannedBatch {
        assert!(
            region_scan.movable_mapping_keys_are_pairwise_distinct(),
            "扫描结果里能搬的映射 key 两两不同：scan_region 见到同一把 key 判成可搬两次就拒这个 R，计划只收它交回的 Ok"
        );
        let relocations: Vec<PlannedRelocation> = region_scan
            .movable_units()
            .take(batch_units)
            .map(|(source, unit)| PlannedRelocation {
                mapping_key: unit.mapping_key.clone(),
                source,
            })
            .collect();
        if !relocations.is_empty() {
            return PlannedBatch::Relocate(relocations);
        }
        if region_scan.allocations.is_empty() {
            PlannedBatch::RegionEmptied
        } else {
            PlannedBatch::NothingMovableButTheRegionIsNotEmpty
        }
    }

    /// R 里零到四条记录、每条五种分类之一、K 取 1 到 3，全部组合：这里的计划与证过 P2、P3 的核逐项相同；
    /// 三种答案都至少出现过一次（对拍不空转）。
    #[test]
    fn plan_batch_agrees_with_the_verified_batch_plan_on_every_small_region_scan() {
        let mut answers_by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
        for allocations in 0..=MOST_ALLOCATIONS_IN_A_REGION {
            for shape_indices in every_sequence(ALLOCATION_SHAPES.len(), allocations) {
                let shapes: Vec<AllocationShape> = shape_indices
                    .iter()
                    .map(|shape_index| ALLOCATION_SHAPES[*shape_index])
                    .collect();
                let region_scan = region_scan_of(&shapes);
                assert!(
                    region_scan.movable_mapping_keys_are_pairwise_distinct(),
                    "核证的 P2「一批里的 key 互异」是蕴含，前提是扫描结果里能搬的 key 两两不同（scan_region 交回 Ok 时成立）：\
                     喂给对拍的形状 {shapes:?} 也要满足它"
                );
                for batch_units in 1..=LARGEST_BATCH_UNITS {
                    let this_implementation = plan_batch(
                        &region_scan,
                        CompactionBatchUnits::new(batch_units).expect("K 从 1 起"),
                    );
                    let reference = plan_of_the_independent_reference(&region_scan, batch_units);
                    assert_eq!(
                        this_implementation, reference,
                        "扫描结果的形状 {shapes:?}、K = {batch_units}：这里的计划与证过的核不同"
                    );
                    let kind = match this_implementation {
                        PlannedBatch::Relocate(_) => "relocate",
                        PlannedBatch::RegionEmptied => "region_emptied",
                        PlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
                            "nothing_movable_but_the_region_is_not_empty"
                        }
                    };
                    *answers_by_kind.entry(kind).or_insert(0) += 1;
                }
            }
        }
        assert_eq!(
            answers_by_kind.len(),
            3,
            "三种答案都该出现过，实际 {answers_by_kind:?}"
        );
    }

    /// 两项能搬、映射 key 相同的扫描结果（`scan_region` 交回 `Ok` 时走不到：它见到这样两项就拒这个 R）进计划就断言，
    /// 不让同一把 key 进同一批两次。
    #[test]
    #[should_panic(expected = "扫描结果里能搬的映射 key 两两不同")]
    fn plan_batch_refuses_a_region_scan_holding_two_movable_units_under_one_mapping_key() {
        let mut region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Movable]);
        let AllocationInRegion::Movable(first_unit) = region_scan.allocations[0].class.clone()
        else {
            panic!("第 0 项照形状是能搬的");
        };
        region_scan.allocations[1].class = AllocationInRegion::Movable(first_unit);
        let _planned = plan_batch(
            &region_scan,
            CompactionBatchUnits::new(2).expect("K 从 1 起"),
        );
    }

    /// 扫描结果里第 `position` 项（照 `region_scan_of` 的形状）的计划项：映射 key 与源落点。
    fn planned_relocation_at(region_scan: &RegionScan, position: usize) -> PlannedRelocation {
        let allocation = &region_scan.allocations[position];
        let AllocationInRegion::Movable(unit) = &allocation.class else {
            panic!("第 {position} 项照形状是能搬的");
        };
        PlannedRelocation {
            mapping_key: unit.mapping_key.clone(),
            source: allocation.placement,
        }
    }

    /// 跳过的集合是空的时，跳过本来就重写的节点那一份计划与 `plan_batch` 逐项相同（没有试算出本来就重写的
    /// 单元时，计划照证过的核取）。
    #[test]
    fn plan_batch_skipping_no_unit_agrees_with_plan_batch_on_every_small_region_scan() {
        let mut region_scans_compared: u64 = 0;
        for allocations in 0..=MOST_ALLOCATIONS_IN_A_REGION {
            for shape_indices in every_sequence(ALLOCATION_SHAPES.len(), allocations) {
                let shapes: Vec<AllocationShape> = shape_indices
                    .iter()
                    .map(|shape_index| ALLOCATION_SHAPES[*shape_index])
                    .collect();
                let region_scan = region_scan_of(&shapes);
                for batch_units in 1..=LARGEST_BATCH_UNITS {
                    let batch = CompactionBatchUnits::new(batch_units).expect("K 从 1 起");
                    assert_eq!(
                        plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                            &region_scan,
                            batch,
                            &BTreeSet::new(),
                        ),
                        plan_batch(&region_scan, batch),
                        "扫描结果的形状 {shapes:?}、K = {batch_units}：一把都不跳过时与 plan_batch 不同"
                    );
                    region_scans_compared += 1;
                }
            }
        }
        assert!(region_scans_compared > 0, "比过的形状不为 0");
    }

    /// 打头那一个可搬单元是这次发布本来就重写的节点：计划跳过它，补上后面真要搬的单元，补满批大小（「补上 R 里
    /// 真要搬的单元」）。
    #[test]
    fn plan_batch_skipping_a_unit_rewritten_anyway_fills_the_batch_with_the_next_movable_units() {
        let region_scan = region_scan_of(&[
            AllocationShape::Movable,
            AllocationShape::Movable,
            AllocationShape::Movable,
        ]);
        let rewritten_anyway: BTreeSet<Vec<u8>> =
            [planned_relocation_at(&region_scan, 0).mapping_key]
                .into_iter()
                .collect();
        assert_eq!(
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                &region_scan,
                CompactionBatchUnits::new(2).expect("K 从 1 起"),
                &rewritten_anyway,
            ),
            PlannedBatch::Relocate(vec![
                planned_relocation_at(&region_scan, 1),
                planned_relocation_at(&region_scan, 2),
            ]),
            "跳过打头那一个，K = 2 补上后面两个"
        );
    }

    /// R 里每一条记录都是可搬单元、它们都是这次发布本来就重写的节点：计划带上它们全部、罩住 R 里每一条记录（调用方照 F2 同一次发布删意图），
    /// 不受批大小限——本来就重写的节点不是搬迁。
    #[test]
    fn plan_batch_skipping_every_unit_plans_them_all_when_only_units_rewritten_anyway_are_left() {
        let region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Movable]);
        let rewritten_anyway: BTreeSet<Vec<u8>> = [0, 1]
            .into_iter()
            .map(|position| planned_relocation_at(&region_scan, position).mapping_key)
            .collect();
        assert_eq!(
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                &region_scan,
                CompactionBatchUnits::new(1).expect("K 从 1 起"),
                &rewritten_anyway,
            ),
            PlannedBatch::Relocate(vec![
                planned_relocation_at(&region_scan, 0),
                planned_relocation_at(&region_scan, 1),
            ]),
            "R 里只剩本来就重写的节点：K = 1 也带上两个，计划罩住 R 里每一条记录"
        );
    }

    /// 能搬的都是这次发布本来就重写的节点，R 里另有一条钉住的记录：再推下去也空不了，交「搬不动」（调用方丢意图）。
    #[test]
    fn plan_batch_skipping_every_movable_unit_with_a_pinned_record_left_has_nothing_movable() {
        let region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Pinned]);
        let rewritten_anyway: BTreeSet<Vec<u8>> =
            [planned_relocation_at(&region_scan, 0).mapping_key]
                .into_iter()
                .collect();
        assert_eq!(
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                &region_scan,
                CompactionBatchUnits::new(1).expect("K 从 1 起"),
                &rewritten_anyway,
            ),
            PlannedBatch::NothingMovableButTheRegionIsNotEmpty,
            "能搬的都跳过、还剩一条钉住的记录"
        );
    }

    /// 专门核对适配层：重复扫描的第一份、异盘过滤、半开 pin 区间和不足整段的尾部。
    #[test]
    fn verified_choice_adapter_preserves_scan_precedence_and_segment_boundaries() {
        let start = unit_area_start_slot();
        let area = UnitAreaOfOneDevice {
            device: DEVICE_OF_THE_UNIT_AREA,
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 2 * CLUSTER_SEGMENT_SLOTS + CLUSTER_SEGMENT_SLOTS - 1,
        };
        let records = vec![
            allocation_record_of_generation_one(DEVICE_OF_THE_UNIT_AREA, start, 2, false),
            allocation_record_of_generation_one(
                DEVICE_OF_THE_UNIT_AREA,
                segment_start_slot(1),
                1,
                false,
            ),
            allocation_record_of_generation_one(ANOTHER_DEVICE, segment_start_slot(1), 20, false),
            allocation_record_of_generation_one(
                DEVICE_OF_THE_UNIT_AREA,
                segment_start_slot(1) + 2,
                20,
                true,
            ),
            allocation_record_of_generation_one(DEVICE_OF_THE_UNIT_AREA, start - 2, 1, false),
            allocation_record_of_generation_one(
                DEVICE_OF_THE_UNIT_AREA,
                segment_start_slot(2),
                1,
                false,
            ),
        ];
        let scan = |region: CompactionRegion, class| RegionScan {
            region,
            allocations: vec![ScannedAllocation {
                placement: Placement {
                    slot: region.start_slot(),
                    span: 1,
                },
                class,
            }],
        };
        let other_region =
            CompactionRegion::of_one_cluster_segment(ANOTHER_DEVICE, SlotNumber(start))
                .expect("对齐起点");
        let scans = vec![
            scan(other_region, AllocationInRegion::Dead),
            scan(
                region_of_the_segment(0),
                AllocationInRegion::Movable(movable_unit(vec![9])),
            ),
            scan(region_of_the_segment(0), AllocationInRegion::Dead),
            scan(region_of_the_segment(1), AllocationInRegion::Dead),
            scan(
                region_of_the_segment(1),
                AllocationInRegion::Movable(movable_unit(vec![8])),
            ),
        ];
        let opened = BTreeSet::new();
        let refused = SegmentsRefusedByTheRegionScan::default();
        let check = |scans_in_this_order: &[RegionScan], pins: &BTreeSet<SlotNumber>, expected| {
            let actual = choose_region(
                &records,
                area,
                &opened,
                None,
                pins,
                scans_in_this_order,
                &refused,
                RegionChoiceCriterion::Greedy,
            );
            assert_eq!(actual, expected);
            assert_eq!(
                actual,
                reference_choose_region(
                    &records,
                    area,
                    &opened,
                    None,
                    pins,
                    scans_in_this_order,
                    &refused,
                    RegionChoiceCriterion::Greedy
                )
            );
        };
        check(
            &scans,
            &BTreeSet::new(),
            RegionChoice::Chosen(region_of_the_segment(0)),
        );
        // 第一段末端属于第二段，不能误把第一段钉住。
        check(
            &scans,
            &BTreeSet::from([SlotNumber(segment_start_slot(1))]),
            RegionChoice::Chosen(region_of_the_segment(0)),
        );
        check(
            &scans,
            &BTreeSet::from([SlotNumber(start)]),
            RegionChoice::NoEligibleSegment,
        );
        // 交换同段两份扫描，结果必须随“第一份”改变；异盘扫描仍不得抢先。
        let mut reversed = scans.clone();
        reversed.swap(1, 2);
        reversed.swap(3, 4);
        check(
            &reversed,
            &BTreeSet::new(),
            RegionChoice::Chosen(region_of_the_segment(1)),
        );
        check(
            &[],
            &BTreeSet::new(),
            RegionChoice::ScanBeforeChoosing(region_of_the_segment(1)),
        );
        for slots in [0, CLUSTER_SEGMENT_SLOTS - 1] {
            assert_eq!(
                choose_region(
                    &records,
                    UnitAreaOfOneDevice { slots, ..area },
                    &opened,
                    None,
                    &BTreeSet::new(),
                    &scans,
                    &refused,
                    RegionChoiceCriterion::Greedy
                ),
                RegionChoice::NoEligibleSegment
            );
        }
    }

    /// 字节 key 的大小次序不能重排计划；空 key、共同前缀与内嵌零均按完整字节比较。
    #[test]
    fn verified_plan_adapter_preserves_byte_keys_input_order_and_full_placements() {
        let keys = [vec![255, 0], vec![], vec![1, 0], vec![1]];
        let mut scan = region_scan_of(&[
            AllocationShape::Movable,
            AllocationShape::Pinned,
            AllocationShape::Movable,
            AllocationShape::Movable,
            AllocationShape::Movable,
        ]);
        for (ordinal, position) in [0, 2, 3, 4].into_iter().enumerate() {
            scan.allocations[position].class =
                AllocationInRegion::Movable(movable_unit(keys[ordinal].clone()));
            scan.allocations[position].placement = Placement {
                slot: SlotNumber(
                    unit_area_start_slot() + 12 - 2 * u64::try_from(ordinal).expect("四项"),
                ),
                span: if ordinal % 2 == 0 { 2 } else { 1 },
            };
        }
        for limit in [1, 2, 4, usize::MAX] {
            let actual = plan_batch(&scan, CompactionBatchUnits::new(limit).expect("正批大小"));
            let expected = PlannedBatch::Relocate(
                [0, 2, 3, 4]
                    .into_iter()
                    .take(limit)
                    .map(|position| planned_relocation_at(&scan, position))
                    .collect(),
            );
            assert_eq!(actual, expected);
            assert_eq!(actual, plan_of_the_independent_reference(&scan, limit));
        }
    }

    /// 即使重复 key 在本批范围以外，也必须保持原来的全输入拒绝，不能把前提检查改成只验 K 项。
    #[test]
    #[should_panic(expected = "扫描结果里能搬的映射 key 两两不同")]
    fn verified_plan_adapter_rejects_a_duplicate_key_beyond_the_batch() {
        let mut scan = region_scan_of(&[
            AllocationShape::Movable,
            AllocationShape::Pinned,
            AllocationShape::Movable,
        ]);
        scan.allocations[2].class = scan.allocations[0].class.clone();
        let _plan = plan_batch(&scan, CompactionBatchUnits::new(1).expect("一项"));
    }

    /// 独立检查证明输入本身：ID 不参与运行时选取，不能只靠最终计划输出检查它。
    #[test]
    fn verified_plan_conversion_preserves_key_equality_in_the_proof_input() {
        let keys = [vec![], vec![1], vec![1, 0], vec![], vec![255], vec![1]];
        let scan = RegionScan {
            region: region_of_the_segment(0),
            allocations: keys
                .iter()
                .enumerate()
                .map(|(position, key)| ScannedAllocation {
                    placement: Placement {
                        slot: SlotNumber(
                            unit_area_start_slot() + u64::try_from(position).expect("六项"),
                        ),
                        span: 1,
                    },
                    class: AllocationInRegion::Movable(movable_unit(key.clone())),
                })
                .collect(),
        };
        let converted = compaction_kernel_allocations(&scan);
        assert_eq!(converted.len(), scan.allocations.len());
        for (allocation_index, allocation) in converted.iter().enumerate() {
            assert_eq!(
                allocation.source_slot,
                scan.allocations[allocation_index].placement.slot.0
            );
            let singlefs_replay_prefix::compaction_plan_proofs::CompactionKernelAllocationClass::Movable {
                mapping_key_identity: identity,
            } = allocation.class else { panic!("所有输入均为可搬项"); };
            for (other_allocation_index, other) in converted.iter().enumerate() {
                let singlefs_replay_prefix::compaction_plan_proofs::CompactionKernelAllocationClass::Movable {
                    mapping_key_identity: other_identity,
                } = other.class else { panic!("所有输入均为可搬项"); };
                assert_eq!(
                    identity == other_identity,
                    keys[allocation_index] == keys[other_allocation_index],
                    "输入 {allocation_index}、{other_allocation_index} 的字节相等关系必须保留"
                );
            }
        }
    }

    /// 拆开之前的 `choose_region`，逐字留在这里当参照（D26（后台整理与放置回收） 已定项 13 第 1 条「选段打分」放上 GPU 时把它拆成
    /// [`super::live_slots_per_cluster_segment_by_the_records`] 与 [`super::choose_region_over_the_live_slots`] 两步）：每段的活槽数内联着数。
    #[allow(
        clippy::too_many_arguments,
        reason = "与被测的 choose_region 同八个输入，逐字照抄拆开之前的写法"
    )]
    fn choose_region_with_the_live_slots_counted_inline_as_before_the_split(
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments_opened_in_this_mount: &BTreeSet<SlotNumber>,
        region_of_the_live_intent: Option<CompactionRegion>,
        slots_pinned_by_the_roots: &BTreeSet<SlotNumber>,
        region_scans: &[RegionScan],
        segments_the_region_scan_refused: &SegmentsRefusedByTheRegionScan,
        criterion: RegionChoiceCriterion,
    ) -> RegionChoice {
        let mut scans_by_start_slot: BTreeMap<SlotNumber, &RegionScan> = BTreeMap::new();
        for scan in region_scans
            .iter()
            .filter(|scan| scan.region.device() == unit_area.device)
        {
            scans_by_start_slot
                .entry(scan.region.start_slot())
                .or_insert(scan);
        }
        let scan_of_the_segment =
            |start_slot: SlotNumber| scans_by_start_slot.get(&start_slot).copied();
        let unit_area_start = unit_area.start.slot().0;
        let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
        let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let mut live_slots_per_segment =
            vec![0u64; usize::try_from(full_segments).expect("一块盘的段数装得进 usize")];
        for record in allocation_records
            .iter()
            .filter(|record| record.device == unit_area.device && !record.is_released)
        {
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) else {
                    continue;
                };
                let segment = usize::try_from(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS)
                    .expect("段号装得进 usize");
                if let Some(live_slots) = live_slots_per_segment.get_mut(segment) {
                    *live_slots += 1;
                }
            }
        }
        let segments: Vec<CompactionKernelSegment> = live_slots_per_segment
            .iter()
            .enumerate()
            .map(|(segment, live_slots)| {
                let start_slot = SlotNumber(
                    unit_area_start
                        + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
                );
                let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
                CompactionKernelSegment {
                    live_slots: match criterion {
                        RegionChoiceCriterion::Greedy => greedy_score(*live_slots),
                    },
                    is_opened_in_this_mount: segments_opened_in_this_mount.contains(&start_slot),
                    is_the_live_intent_region: region_of_the_live_intent
                        .is_some_and(|region| region.start_slot() == start_slot),
                    holds_a_pinned_slot: slots_pinned_by_the_roots
                        .range(start_slot..end_slot)
                        .next()
                        .is_some(),
                    is_refused_by_the_region_scan: segments_the_region_scan_refused
                        .contains(start_slot),
                    scan: match scan_of_the_segment(start_slot) {
                        None => CompactionKernelSegmentScan::NotScanned,
                        Some(scan_of_this_segment)
                            if scan_of_this_segment.movable_units().next().is_some() =>
                        {
                            CompactionKernelSegmentScan::ScannedWithAMovableUnit
                        }
                        Some(_) => CompactionKernelSegmentScan::ScannedWithoutAMovableUnit,
                    },
                }
            })
            .collect();
        let region_at = |position: usize| {
            CompactionRegion::of_one_cluster_segment(
                unit_area.device,
                SlotNumber(
                    unit_area_start
                        + u64::try_from(position * segment_slots).expect("槽号装得进 u64"),
                ),
            )
            .expect("单元区的整段起点在聚簇段边界上且装得进六字节")
        };
        match choose_compaction_segment(&segments) {
            CompactionKernelRegionChoice::Chosen(position) => {
                RegionChoice::Chosen(region_at(position))
            }
            CompactionKernelRegionChoice::ScanBeforeChoosing(position) => {
                RegionChoice::ScanBeforeChoosing(region_at(position))
            }
            CompactionKernelRegionChoice::NoEligibleSegment => RegionChoice::NoEligibleSegment,
        }
    }

    /// 确定的伪随机源（xorshift64*，种子写死）：同一个种子每次造出同一串输入，红了能原样重放。
    struct DeterministicRandomSource {
        state: u64,
    }

    impl DeterministicRandomSource {
        fn below(&mut self, bound: u64) -> u64 {
            self.state ^= self.state >> 12;
            self.state ^= self.state << 25;
            self.state ^= self.state >> 27;
            self.state.wrapping_mul(0x2545_F491_4F6C_DD1D) % bound
        }

        fn one_in(&mut self, odds: u64) -> bool {
            self.below(odds) == 0
        }
    }

    /// 随机造的一份选段输入：单元区（整段数加凑不满 64 槽的尾巴）、两块盘上的记录（有落在单元区之前与之后的、跨度 1 到 3、四分之一已释放）、
    /// 开过的段、活 R、钉住的槽、扫过的段（有能搬的、没有能搬的、另一块盘上的）、被扫描拒过的段。
    struct RandomChoiceInput {
        unit_area: UnitAreaOfOneDevice,
        allocation_records: Vec<AllocationRecord>,
        cluster_segments: BTreeSet<SlotNumber>,
        live_intent_region: Option<CompactionRegion>,
        pinned_slots: BTreeSet<SlotNumber>,
        region_scans: Vec<RegionScan>,
        segments_refused: SegmentsRefusedByTheRegionScan,
    }

    const FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA: u64 = 6;
    const RANDOM_CHOICE_INPUTS: u64 = 3000;
    const RANDOM_CHOICE_SEED: u64 = 0x5EED_0D26_0013_0001;

    fn random_choice_input(random: &mut DeterministicRandomSource) -> RandomChoiceInput {
        let start = unit_area_start_slot();
        let unit_area = UnitAreaOfOneDevice {
            device: DEVICE_OF_THE_UNIT_AREA,
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA * CLUSTER_SEGMENT_SLOTS
                + random.below(CLUSTER_SEGMENT_SLOTS),
        };
        let segment_start = |segment: u64| SlotNumber(start + segment * CLUSTER_SEGMENT_SLOTS);
        let region_of = |segment: u64, device: DeviceIdentity| {
            CompactionRegion::of_one_cluster_segment(device, segment_start(segment))
                .expect("单元区起点在聚簇段边界上")
        };
        let records_count = random.below(48);
        let allocation_records = (0..records_count)
            .map(|_record| {
                let device = if random.one_in(3) {
                    ANOTHER_DEVICE
                } else {
                    DEVICE_OF_THE_UNIT_AREA
                };
                // 从单元区起点之前 8 槽到整段之后一段多：有落在单元区之前、尾巴里与单元区之外的。
                let slot = start - 8
                    + random.below(
                        (FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA + 1) * CLUSTER_SEGMENT_SLOTS + 16,
                    );
                let span_slots = u16::try_from(1 + random.below(3)).expect("1 到 3");
                allocation_record_of_generation_one(device, slot, span_slots, random.one_in(4))
            })
            .collect();
        let segments_of_the_area = FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA + 1;
        let cluster_segments = (0..segments_of_the_area)
            .filter(|_segment| random.one_in(5))
            .map(segment_start)
            .collect();
        let live_intent_region = if random.one_in(3) {
            Some(region_of(
                random.below(FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA),
                DEVICE_OF_THE_UNIT_AREA,
            ))
        } else {
            None
        };
        let pinned_slots = (0..random.below(3))
            .map(|_pinned| {
                SlotNumber(start + random.below(segments_of_the_area * CLUSTER_SEGMENT_SLOTS))
            })
            .collect();
        let mut region_scans = Vec::new();
        let mut segments_refused = SegmentsRefusedByTheRegionScan::default();
        for segment in 0..FULL_SEGMENTS_OF_THE_RANDOM_UNIT_AREA {
            match random.below(6) {
                0 => region_scans.push(RegionScan {
                    region: region_of(segment, DEVICE_OF_THE_UNIT_AREA),
                    allocations: vec![ScannedAllocation {
                        placement: Placement {
                            slot: segment_start(segment),
                            span: 1,
                        },
                        class: AllocationInRegion::Movable(movable_unit(vec![u8::try_from(
                            segment,
                        )
                        .expect("至多六段")])),
                    }],
                }),
                1 => region_scans.push(RegionScan {
                    region: region_of(segment, DEVICE_OF_THE_UNIT_AREA),
                    allocations: Vec::new(),
                }),
                2 => region_scans.push(RegionScan {
                    region: region_of(segment, ANOTHER_DEVICE),
                    allocations: Vec::new(),
                }),
                3 => segments_refused.note_refusal(
                    region_of(segment, DEVICE_OF_THE_UNIT_AREA),
                    RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                        device: DEVICE_OF_THE_UNIT_AREA,
                        slot: segment_start(segment),
                        span_slots: 2,
                    },
                ),
                _ => {}
            }
        }
        RandomChoiceInput {
            unit_area,
            allocation_records,
            cluster_segments,
            live_intent_region,
            pinned_slots,
            region_scans,
            segments_refused,
        }
    }

    #[test]
    fn choosing_over_the_live_slots_counted_per_segment_picks_the_segment_the_inline_scoring_picked(
    ) {
        let mut random = DeterministicRandomSource {
            state: RANDOM_CHOICE_SEED,
        };
        let mut answers_seen: BTreeSet<String> = BTreeSet::new();
        for input_number in 0..RANDOM_CHOICE_INPUTS {
            let input = random_choice_input(&mut random);
            let before_the_split =
                choose_region_with_the_live_slots_counted_inline_as_before_the_split(
                    &input.allocation_records,
                    input.unit_area,
                    &input.cluster_segments,
                    input.live_intent_region,
                    &input.pinned_slots,
                    &input.region_scans,
                    &input.segments_refused,
                    RegionChoiceCriterion::Greedy,
                );
            let live_slots_per_segment: Vec<LiveSlotsOfAClusterSegment> =
                live_slots_per_cluster_segment_by_the_records(
                    &input.allocation_records,
                    input.unit_area,
                )
                .into_iter()
                .map(LiveSlotsOfAClusterSegment::Counted)
                .collect();
            let over_the_live_slots = choose_region_over_the_live_slots(
                &live_slots_per_segment,
                input.unit_area,
                &input.cluster_segments,
                input.live_intent_region,
                &input.pinned_slots,
                &input.region_scans,
                &input.segments_refused,
                RegionChoiceCriterion::Greedy,
            );
            let through_choose_region = choose_region(
                &input.allocation_records,
                input.unit_area,
                &input.cluster_segments,
                input.live_intent_region,
                &input.pinned_slots,
                &input.region_scans,
                &input.segments_refused,
                RegionChoiceCriterion::Greedy,
            );
            assert_eq!(
                over_the_live_slots, before_the_split,
                "第 {input_number} 份输入：先数每段活槽数、再吃它选段，与拆开之前内联着数的选出的相同"
            );
            assert_eq!(
                through_choose_region, before_the_split,
                "第 {input_number} 份输入：拆开之后的 choose_region 与拆开之前的选出的相同"
            );
            answers_seen.insert(format!("{before_the_split:?}").chars().take(12).collect());
        }
        assert_eq!(
            answers_seen.len(),
            3,
            "随机输入走到了选中、先扫、一段都没资格三种答案：{answers_seen:?}"
        );
    }

    /// GPU 路径上一段的活槽数这一轮不确定（D26（后台整理与放置回收） 已定项 13 第 8 条）：它这一批不进候选，选的是其余各段里活槽最少的；
    /// 同样的输入把它记成数出来的，它活槽最少、选的就是它。
    #[test]
    fn segment_whose_live_slots_are_uncertain_this_round_is_left_out_of_the_candidates() {
        let unit_area = UnitAreaOfOneDevice {
            device: DEVICE_OF_THE_UNIT_AREA,
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 3 * CLUSTER_SEGMENT_SLOTS,
        };
        let region_scans: Vec<RegionScan> = (0..3)
            .map(|segment_index| RegionScan {
                region: region_of_the_segment(segment_index),
                allocations: vec![ScannedAllocation {
                    placement: Placement {
                        slot: SlotNumber(segment_start_slot(segment_index)),
                        span: 1,
                    },
                    class: AllocationInRegion::Movable(movable_unit(vec![u8::try_from(
                        segment_index,
                    )
                    .expect("三段")])),
                }],
            })
            .collect();
        let choose_over = |live_slots_per_segment: &[LiveSlotsOfAClusterSegment]| {
            choose_region_over_the_live_slots(
                live_slots_per_segment,
                unit_area,
                &BTreeSet::new(),
                None,
                &BTreeSet::new(),
                &region_scans,
                &SegmentsRefusedByTheRegionScan::default(),
                RegionChoiceCriterion::Greedy,
            )
        };
        assert_eq!(
            choose_over(&[
                LiveSlotsOfAClusterSegment::Counted(9),
                LiveSlotsOfAClusterSegment::Counted(1),
                LiveSlotsOfAClusterSegment::Counted(5),
            ]),
            RegionChoice::Chosen(region_of_the_segment(1)),
            "三段都数出来了：活槽最少的中间那一段"
        );
        assert_eq!(
            choose_over(&[
                LiveSlotsOfAClusterSegment::Counted(9),
                LiveSlotsOfAClusterSegment::UncertainThisRound,
                LiveSlotsOfAClusterSegment::Counted(5),
            ]),
            RegionChoice::Chosen(region_of_the_segment(2)),
            "中间那一段这一轮不确定：不进候选，选其余两段里活槽最少的末一段"
        );
        assert_eq!(
            choose_over(&[
                LiveSlotsOfAClusterSegment::UncertainThisRound,
                LiveSlotsOfAClusterSegment::UncertainThisRound,
                LiveSlotsOfAClusterSegment::UncertainThisRound,
            ]),
            RegionChoice::NoEligibleSegment,
            "三段都不确定：这一批一段都不选"
        );
    }
}
