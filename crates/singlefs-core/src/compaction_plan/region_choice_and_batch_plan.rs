//! 整理的选段（`choose_region` 一族）与取下一批（`plan_batch` 一族）：纯函数，调 Verus 证过的选段核、计划核；
//! 连同与两个核逐项对拍的测试。

use super::*;

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

    /// 把 `other` 里记下的段并进来（同一段两边都记了，留这边先记的理由）：放锁的三段入口锁外那一段在票里那一份上记，回锁时并回会话。
    pub fn note_the_refusals_of(&mut self, other: &SegmentsRefusedByTheRegionScan) {
        for (segment_start, refusal) in &other.refused {
            self.refused.entry(*segment_start).or_insert(*refusal);
        }
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
