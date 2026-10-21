//! 整理的选段与计划（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2）：三层里的后两层，都是纯函数——
//! 不读设备、不发写、不动分配器。`choose_region` 按记录给候选段排序、按扫描分类定资格（没扫过的先让调用方经
//! `crate::region_scan::scan_region` 扫），`plan_batch` 从扫描结果里取下一批要搬的（映射 key, 源落点）。**目的落点不在计划里**：由那一批发布时的分配器按 D3（空间分配） 已定项 8 现取
//! （判决 G2，攻方 H-G2-1：缓存目的落点的臂撞上映射重复）。
//!
//! 判据显式（`.claude/rules/fs-design.md` 五条硬要求第 1 条）：第一版只有 greedy 一种（[`RegionChoiceCriterion::Greedy`]），
//! 打分是 [`greedy_score`]；cost-benefit 与轮转留给 E10（老化：结构抗老化 vs 事后整理） 的臂。

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use singlefs_format::CLUSTER_SEGMENT_SLOTS;

use crate::address::{DeviceIdentity, SlotNumber};
use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
use crate::logged_operations::{CompactionIntent, CompactionRegion};
use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};

/// 选段判据。封闭集合：加一种判据就加一个成员（判决 G2 的 S-b、S-c 由 E10 定要不要加）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionChoiceCriterion {
    /// 活槽最少的段先（判决 G2 第一版）。
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
/// 整理不替它修（主 agent 2026-10-13 定，判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S4）。
/// 第一版两盘同槽：按段起点的槽号记（判决 G1 攻方 K-n）。
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

/// 挑一个整理区域 R（判决 G2 `choose_region`）。资格五条全成立的段才算：不在这次挂载开过的聚簇段里（`cluster_segments`，段起点）、
/// 不是活意图的 R、罩住的槽一个都不在 `pinned_slots` 里、这次挂载里没被区域扫描整次拒过（`segments_refused_by_the_region_scan`）、
/// R 里至少一个能搬的单元（按 `region_scans` 里这一段的扫描分类取，判决 G2「R 里至少一个能搬的单元」，攻方 K-b；
/// 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S4）。
/// 按判据打分（活槽数：R 那块盘上未释放记录罩住的槽）排候选，同分取起点槽号最小的；排在最前的那一段扫过就交回
/// [`RegionChoice::Chosen`]，没扫过就交回 [`RegionChoice::ScanBeforeChoosing`]；扫过而一个能搬的都没有的段不是候选。
/// `region_scans` 要是对这一份 `allocation_records` 扫出来的（记录变了就重扫），不是 `unit_area.device` 那块盘上的不算。
/// 第一版两盘同槽：活槽数按 `unit_area.device` 那块盘的记录数，`cluster_segments`、活 R、`pinned_slots` 与被拒的段都只比槽号（判决 G1 攻方 K-n）。
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
                unit_area_start + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
            );
            (start_slot, *live_slots)
        })
        .filter(|(start_slot, _live_slots)| {
            let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
            let opened_in_this_mount = cluster_segments.contains(start_slot);
            let is_the_live_intent_region =
                live_intent_region.is_some_and(|region| region.start_slot() == *start_slot);
            let holds_a_pinned_slot = pinned_slots.range(*start_slot..end_slot).next().is_some();
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

/// 一批至多搬几个单元（判决 G2 的 K）。至少 1；取值归 E10（老化：结构抗老化 vs 事后整理），这里只是参数。
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

/// [`plan_batch`] 的答案：有要搬的，或者一个能搬的都没有时分「搬空」与「搬不动」（判决 m4-batch2-code-r1 S4）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedBatch {
    /// 这一批要搬的，按槽升序，至少一项、至多 K 项。
    Relocate(Vec<PlannedRelocation>),
    /// R 里一条未释放的记录都没有了：R 按记录全空（判决 G4 的停机谓词「删意图那一刻 R 按记录全空」看的就是这个）。
    RegionEmptied,
    /// R 里一个能搬的都没有，却还有未释放的记录（钉住、隔离、已死、不一致的）：再推下去也空不了。
    NothingMovableButTheRegionIsNotEmpty,
}

/// 下一批要搬的：扫描结果里可搬的单元按槽升序取前 `batch` 个（判决 G2 `plan_batch`）。钉住、隔离、已死、不一致的一个都不进。
/// 这一批发布之后它们离开 R，下一次扫描就不再有它们：各批互不相交，全部批的并集是 R 里可搬的 key 集合。
/// 一个能搬的都没有时不交空表：R 里没有未释放的记录交 [`PlannedBatch::RegionEmptied`]，还有交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。
///
/// # Panics
/// 扫描结果里能搬的映射 key 有相同的：计划的输入只来自 [`crate::region_scan::scan_region`] 交回的 `Ok`，它见到同一把 key 判成可搬两次
/// 就拒这个 R（`RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey`），所以这里收到的 key 两两不同，一批里的 key 也就互异
/// （判决 G7 的 P2；Verus 证的核把它写成蕴含的前提 `movable_mapping_keys_are_distinct`）。
#[must_use]
pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> PlannedBatch {
    assert!(
        region_scan.movable_mapping_keys_are_pairwise_distinct(),
        "扫描结果里能搬的映射 key 两两不同：scan_region 见到同一把 key 判成可搬两次就拒这个 R，计划只收它交回的 Ok"
    );
    let relocations: Vec<PlannedRelocation> = region_scan
        .movable_units()
        .take(batch.units())
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

/// 下一批要搬的，跳过这次发布本来就重写的节点（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16
/// 定「规划时补足」）：`rewritten_anyway` 是会话在这一次发布上试算出的、计划过而发布路径不另搬的映射 key（它们照样离开 R，只是不算搬）。
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

/// 一批至多搬几个单元从哪来。封闭集合。产品路径的批大小条款没写（判决 G2 的 K 归 E10（老化：结构抗老化 vs 事后整理））：
/// 产品路径照 [`CompactionBatchSize::NotDecidedByTheClauses`]，整理一次运行在签任何意图、发任何一个写之前停下，停机原因
/// [`CompactionRunStop::BatchSizeNotDecidedByTheClauses`]；只供测试的开关给出一个数才走得进去。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionBatchSize {
    NotDecidedByTheClauses,
    /// 只供测试（判决 G6 `CompactionSwitches` 的「批大小」）。
    ForcedForTests(CompactionBatchUnits),
}

/// 整理什么时候起跑。封闭集合。产品路径照水位（[`CompactionRunForcing::FollowTheWatermarks`]）；另两种只供测试（判决 G6「强制起跑 / 停」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunForcing {
    /// 用户改动发成之后按水位判起不起跑（判决 G8 丁）。第一版三条水位恒 0、0 = 关（判决 G4 V-a；内置默认改非零不在这一批），
    /// 所以这一支在产品路径上不起跑；显式入口（`mounted_session::MountedSession::run_compaction`）不看水位、照常跑。
    FollowTheWatermarks,
    /// 只供测试：用户改动发成之后不看水位，照样跑至多一批。
    ForcedToStartForTests,
    /// 只供测试：两个入口都不起跑，停机原因 [`CompactionRunStop::ForcedToStopForTests`]。
    ForcedToStopForTests,
}

/// 签发之后 R 排不排他（D26（后台整理与放置回收） 已定项 1 第 3 条：`R` 在意图期内不可作分配目的地）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRegionExclusion {
    ExcludedWhileTheIntentIsLive,
    /// 只供测试（判决 G6「关 R 排他」）：活意图的 R 照样当分配目的地。
    SwitchedOffForTests,
}

/// 搬迁的目的落点按哪条政策取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationDestinationPolicy {
    /// 判决 G2 的 D-a：码 1 走用户数据落点函数（D3（空间分配） 已定项 8 第 1 条），节点与打包容器走开放段 bump（已定项 10 ⑤），都不落在活 R 里。
    UserDataPlacementForDataAndOpenSegmentForTheRest,
    /// 只供测试（判决 G6「强制目的地政策」）：每个单元都取单元区里起点最小、整跨度没挡的落点（提交内生块回落那一个函数），仍不落在活 R 里。
    LowestUnblockedSlotForEveryUnitForTests,
}

/// 签发之后强制一次「释放前核出对不上」（判决 G6）。封闭集合。
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
    /// 只供测试（判决 G6「强制合取 7 的 `N₀`」）。
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

/// 整理只供测试的开关（判决 G6 `CompactionSwitches`；`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）。
/// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
/// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
/// （`mount::mount_writable_with_compaction_switches`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionSwitches {
    pub batch_size: CompactionBatchSize,
    /// 只供测试（判决 G6「强制选段」）：签发时不按判据选、取这一段（资格那五条照判，不合格就停）。`None` 是产品路径。
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
        batch_size: CompactionBatchSize::NotDecidedByTheClauses,
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

/// 一次整理运行停在哪（判决 G8 观测「停机原因」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunStop {
    /// 产品路径的批大小条款没写（[`CompactionBatchSize::NotDecidedByTheClauses`]）：一个意图都没签、一个写都没发。
    BatchSizeNotDecidedByTheClauses,
    /// 只供测试的开关叫停（[`CompactionRunForcing::ForcedToStopForTests`]）。
    ForcedToStopForTests,
    /// 一段都没资格（[`RegionChoice::NoEligibleSegment`]），或强制选的那一段不合格。
    NoEligibleSegment,
    /// 签发前判不过（D26（后台整理与放置回收） 已定项 1 第 2 条，攻方 K-g）：R 外非全空段的空洞少于 R 里要搬的单元数，不签。
    HolesOutsideTheRegionFewerThanItsUnitsToMove { holes: u64, units_to_move: u64 },
    /// 上一条意图完成了，全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假（攻方 K-h）：不签下一条。
    NoYieldFromTheLastIntent {
        empty_cluster_segments_now: u64,
        empty_cluster_segments_at_issue: u64,
    },
    /// 这次运行的发布预算用完了（判决 G8 `B_gc`）。
    PublishBudgetSpent { budget: u64 },
    /// 一批被空间拒（准入或落点）：暂停——撤掉 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）。
    PausedShortOfSpace,
    /// 签发那次发布被空间拒（准入或落点）：意图没签成、盘上没有它，R 排他跟着撤；不推抬 F（判决 G8 g1）。
    IssueRefusedShortOfSpace,
    /// 可写挂载续做（只续做盘上那一条未完成的意图、不签新的，D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」）：
    /// 那一条删了（达成或丢掉），或本来就没有活意图。
    NoLiveIntentLeftToContinue,
    /// R 里一个能搬的都没有、却还有未释放的记录（[`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]）：意图连同 R 排他丢掉。
    NothingMovableButTheRegionIsNotEmpty,
    /// 用户改动之后那一次只跑至多一批（判决 G8 丁），跑完了。
    OneBatchAfterTheUserChangeDone,
}

impl CompactionRunStop {
    /// 运行时报出停在哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            CompactionRunStop::BatchSizeNotDecidedByTheClauses => {
                "compaction_stopped_batch_size_not_decided_by_the_clauses"
            }
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
        }
    }
}

/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（暂停 = 撤掉 R 排他、意图留着，攻方 K-k）。
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
    /// （`transaction::publish_compaction_step`）；挂载按 logged_ops 里那一条装上。全池活意图至多一条（判决 G1 e1）。
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

    /// 暂停：撤掉 R 排他、意图留着（攻方 K-k）。下一次运行接着推它时重新排他。
    pub fn pause_the_live_intent(&mut self) {
        if let Some(live) = self.live_intent.as_mut() {
            live.is_the_region_excluded_from_placement = false;
        }
    }

    /// 接着推一条暂停过的意图：照开关重新排他。
    pub fn resume_the_live_intent(&mut self) {
        let is_excluded = match self.switches.region_exclusion {
            CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
            CompactionRegionExclusion::SwitchedOffForTests => false,
        };
        if let Some(live) = self.live_intent.as_mut() {
            live.is_the_region_excluded_from_placement = is_excluded;
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
/// 而隔离的槽（常驻隔离集，攻方 K-c 读它）、上一次用户改动之后整理发了几次布（`B_gc`）、观测。住在挂着的会话上
/// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactionOfThisMount {
    pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
    quarantined_slots: BTreeSet<SlotNumber>,
    /// 上一次用户改动之后整理发了几次布（判决 G8 `B_gc` 式子里「已经过去的发布数」）：会话发成一次用户改动就归零。
    pub publishes_since_the_last_user_change: u64,
    pub observations: CompactionObservations,
}

impl CompactionOfThisMount {
    /// 一次挂载起步：三样都空。
    #[must_use]
    pub fn of_a_new_mount() -> Self {
        Self {
            segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan::default(),
            quarantined_slots: BTreeSet::new(),
            publishes_since_the_last_user_change: 0,
            observations: CompactionObservations::default(),
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

/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条，攻方 K-g）的左边：R 那块盘上、R 之外、「非全空」的段里没挡的槽数。
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

/// 一条意图完成之后签不签下一条（攻方 K-h）：全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为真才签。
#[must_use]
pub const fn the_completed_intent_yielded(
    empty_cluster_segments_now: u64,
    empty_cluster_segments_at_issue: u64,
) -> bool {
    empty_cluster_segments_now > empty_cluster_segments_at_issue
}

/// 签发之后 R 里出现了隔离槽没有（攻方 K-c；丁 1 交判第 4 点：一个槽既钉住又在隔离集里，区域扫描判「钉住」照旧，这一判另外直接查隔离集，
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

/// 一次整理运行的发布预算（判决 G8 `B_gc`，正推 O11：整理的发布不改用户可见状态，连发会把回退候选集挤成 1 个状态）：
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

/// 用户改动发成之后起不起跑整理（判决 G8 丁：水位越线再跑至多一批）。第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制。
#[must_use]
pub const fn compaction_starts_after_a_user_change(forcing: CompactionRunForcing) -> bool {
    match forcing {
        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStopForTests => {
            false
        }
        CompactionRunForcing::ForcedToStartForTests => true,
    }
}

/// 与 Verus 证过的选段核、计划核逐项对拍（`.claude/kb/milestone/04-fourth-txn.md` 二·4；判决 `research/prompts/m4-gc-r1-main-verification.md`
/// 第三节 G7 的 P1、P2、P3 证在 `singlefs_replay_prefix::compaction_plan_proofs` 的两个核上）：小输入上每一种形状都喂给这里的
/// [`choose_region`]、[`plan_batch`] 与核，答案逐项相同。核那一侧的入参由用例按形状直接写出，不经这里的代码算，
/// 所以两边对上就是这里的实现在这些输入上合核证过的性质。核心层不调核：`crates/mutations.tsv` 里锚在这里现在写法上的变异行要原样留着。
#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use singlefs_format::CLUSTER_SEGMENT_SLOTS;
    use singlefs_replay_prefix::compaction_plan_proofs::{
        choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
        CompactionKernelPlannedBatch, CompactionKernelRegionChoice,
        CompactionKernelScannedAllocation, CompactionKernelSegment, CompactionKernelSegmentScan,
    };

    use super::{
        choose_region, plan_batch, plan_batch_skipping_the_units_this_publish_rewrites_anyway,
        CompactionBatchUnits, PlannedBatch, PlannedRelocation, RegionChoice, RegionChoiceCriterion,
        SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
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
    /// 一段里各条活记录的跨度（槽数），一条一项（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F8：输入域加跨度 2 的记录、
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

    fn record(
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
                allocation_records.push(record(
                    DEVICE_OF_THE_UNIT_AREA,
                    start_slot + 2 * live_record,
                    *span_slots,
                    false,
                ));
                allocation_records.push(record(
                    ANOTHER_DEVICE,
                    start_slot + 2 * live_record + 1,
                    1,
                    false,
                ));
            }
            allocation_records.push(record(
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
        choose_region(
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
        )
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

    /// 同一份扫描结果写成核的入参（映射 key 换成身份：同一把 key 同一个数），核交回的下标换回（映射 key, 源落点）。
    fn plan_of_the_verified_kernel(region_scan: &RegionScan, batch_units: usize) -> PlannedBatch {
        let mut identities_of_the_keys: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
        let allocations: Vec<CompactionKernelScannedAllocation> = region_scan
            .allocations
            .iter()
            .map(|allocation| CompactionKernelScannedAllocation {
                source_slot: allocation.placement.slot.0,
                class: match &allocation.class {
                    AllocationInRegion::Movable(unit) => {
                        let next_identity =
                            u64::try_from(identities_of_the_keys.len()).expect("至多四项");
                        CompactionKernelAllocationClass::Movable {
                            mapping_key_identity: *identities_of_the_keys
                                .entry(unit.mapping_key.clone())
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
            .collect();
        match plan_compaction_batch(&allocations, batch_units) {
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
                    let verified_kernel = plan_of_the_verified_kernel(&region_scan, batch_units);
                    assert_eq!(
                        this_implementation, verified_kernel,
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
    /// 不让同一把 key 进同一批两次（判决 G7 的 P2）。
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

    /// 跳过的集合是空的时，跳过本来就重写的节点那一份计划与 `plan_batch` 逐项相同（判决 m4-segment-code-r2 第三节 H4：没有试算出本来就重写的
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

    /// 打头那一个可搬单元是这次发布本来就重写的节点：计划跳过它，补上后面真要搬的单元，补满批大小（判决 m4-segment-code-r2 第三节 H4「补上 R 里
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
}
