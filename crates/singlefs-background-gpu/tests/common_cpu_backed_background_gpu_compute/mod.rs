//! 核心层 `BackgroundGpuCompute` 的一份假实现，背后照核心层的 CPU 写法算：甲 `checksum::crc32_castagnoli`，乙 `region_scan::scan_region_over_the_identity_index`
//! 对单条记录（钉住、隔离两张集合给空），丙 `compaction_plan::live_slots_per_cluster_segment_by_the_records`。不注入时它交回的每一条都与 CPU 那一条相同、都算确认。
//! 可注入：按条目键点名的几条这一轮「两份核判得不一样」。连续不确定的计数照 D26（后台整理与放置回收） 已定项 13 第 8 条在这里数
//! （[`CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_JUDGES`]：同一个条目连续这么多轮不确定就按 CPU 写法判一次、清零），只在这份实现里，丢掉它就清零。
//! 本包的对拍用例拿它当另一边，与产品那一份（两份核那一层换成 CPU 替身）逐轮逐条比。只依赖核心层；验证包测试里有一份同样做法的，各写各的。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::{BTreeMap, BTreeSet};

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed, ClusterSegmentEntryKey,
    GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes, RegionRecordEntryKey,
    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    live_slots_per_cluster_segment_by_the_records, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{
    scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex,
};

/// 这份假实现报的那一张卡的显存总量：8 GiB。
pub const EIGHT_GIBIBYTES: u64 = 8 << 30;
/// 连续几轮不确定就交 CPU 判（D26 已定项 13 第 8 条的 N）。
pub const CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_JUDGES: u32 = 2;

/// 这份假实现报的那一张卡。
#[must_use]
pub fn card_of_the_cpu_backed_fake() -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(b"card-eight".to_vec())
}

/// 一个条目这一轮怎么交：没注入照 CPU 写法确认、清掉它的计数；注入了不确定的，连续第 N 轮按 CPU 写法判、清零，之前各轮交不确定。
fn decision_of_this_round<EntryKey: Ord + Copy, Outcome>(
    entry_key: EntryKey,
    is_injected_uncertain: bool,
    consecutive_uncertain_rounds: &mut BTreeMap<EntryKey, u32>,
    outcome_by_the_cpu: Outcome,
) -> GpuEntryDecision<Outcome> {
    if !is_injected_uncertain {
        consecutive_uncertain_rounds.remove(&entry_key);
        return GpuEntryDecision::ConfirmedByBothKernels(outcome_by_the_cpu);
    }
    let rounds = consecutive_uncertain_rounds.entry(entry_key).or_insert(0);
    *rounds += 1;
    if *rounds >= CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_JUDGES {
        consecutive_uncertain_rounds.remove(&entry_key);
        return GpuEntryDecision::JudgedByTheCpuReference(outcome_by_the_cpu);
    }
    GpuEntryDecision::UncertainThisRound
}

pub struct CpuBackedBackgroundGpuComputeForTests {
    /// 甲段这几条两份核判得不一样。
    pub unit_checksums_uncertain: BTreeSet<RegionRecordEntryKey>,
    /// 乙段这几条两份核判得不一样。
    pub region_records_uncertain: BTreeSet<RegionRecordEntryKey>,
    /// 丙段这几段两份核数得不一样。
    pub cluster_segments_uncertain: BTreeSet<ClusterSegmentEntryKey>,
    consecutive_uncertain_rounds_of_the_unit_checksums: BTreeMap<RegionRecordEntryKey, u32>,
    consecutive_uncertain_rounds_of_the_region_records: BTreeMap<RegionRecordEntryKey, u32>,
    consecutive_uncertain_rounds_of_the_cluster_segments: BTreeMap<ClusterSegmentEntryKey, u32>,
}

impl CpuBackedBackgroundGpuComputeForTests {
    #[must_use]
    pub fn with_nothing_injected() -> Self {
        Self {
            unit_checksums_uncertain: BTreeSet::new(),
            region_records_uncertain: BTreeSet::new(),
            cluster_segments_uncertain: BTreeSet::new(),
            consecutive_uncertain_rounds_of_the_unit_checksums: BTreeMap::new(),
            consecutive_uncertain_rounds_of_the_region_records: BTreeMap::new(),
            consecutive_uncertain_rounds_of_the_cluster_segments: BTreeMap::new(),
        }
    }
}

impl GpuInventory for CpuBackedBackgroundGpuComputeForTests {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![(
            card_of_the_cpu_backed_fake(),
            GpuMemoryInBytes(EIGHT_GIBIBYTES),
        )]
    }
}

impl BackgroundGpuCompute for CpuBackedBackgroundGpuComputeForTests {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        Ok(units
            .iter()
            .map(|unit| {
                let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
                decision_of_this_round(
                    unit.entry_key,
                    self.unit_checksums_uncertain.contains(&unit.entry_key),
                    &mut self.consecutive_uncertain_rounds_of_the_unit_checksums,
                    UnitChecksumOutcome {
                        computed_crc32c,
                        matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c,
                    },
                )
            })
            .collect())
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
        Ok(records
            .iter()
            .map(|to_classify| {
                let mut scan = scan_region_over_the_identity_index(
                    region,
                    std::slice::from_ref(&to_classify.record),
                    &mapping_index,
                    &BTreeSet::new(),
                    &BTreeSet::new(),
                    |_device, _slot, _unit_bytes| to_classify.unit_bytes.map(<[u8]>::to_vec),
                )
                .expect("单条记录：区域扫描交来的记录不跨 R 的边界，一条记录也不会两条同 key");
                let class = scan
                    .allocations
                    .pop()
                    .expect("区域扫描交来的记录起点在 R 里、未释放：扫出恰好一条")
                    .class;
                decision_of_this_round(
                    to_classify.entry_key,
                    self.region_records_uncertain
                        .contains(&to_classify.entry_key),
                    &mut self.consecutive_uncertain_rounds_of_the_region_records,
                    class,
                )
            })
            .collect())
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let live_slots_per_segment =
            live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
        assert_eq!(
            live_slots_per_segment.len(),
            segments.len(),
            "条目键一个整段一个"
        );
        Ok(segments
            .iter()
            .zip(live_slots_per_segment)
            .map(|(segment, live_slots)| {
                decision_of_this_round(
                    *segment,
                    self.cluster_segments_uncertain.contains(segment),
                    &mut self.consecutive_uncertain_rounds_of_the_cluster_segments,
                    live_slots,
                )
            })
            .collect())
    }

    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
}
