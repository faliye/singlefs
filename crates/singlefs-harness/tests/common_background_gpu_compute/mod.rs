//! 整理的 GPU 路径用的一份假 GPU 实现（`background_compute::BackgroundGpuCompute`，产品 GPU 包要实现的那一份）：背后照核心层的 CPU 写法算——
//! 甲 `checksum::crc32_castagnoli`，乙 `region_scan::scan_region_over_the_identity_index` 对单条记录（钉住、隔离两张集合给空），
//! 丙 `compaction_plan::live_slots_per_cluster_segment_by_the_records`——所以不注入故障时它交回的每一条都与 CPU 那一条相同、都算确认。
//! 可注入：某一段交回出错（交回哪一个错由用例给，带卡与原因）；按条目键点名的几条这一轮「两份核判得不一样」；甲段按条目键点名的几条报成 CRC 对不上。
//! 连续不确定的计数照 D26（后台整理与放置回收） 已定项 13 第 8 条在这里数（N = 2：同一个条目连续第 2 轮不确定就按 CPU 写法判一次、清零），
//! 只在这份实现里，丢掉它就清零。卡清单是两张卡：「card-eight」8 GiB 在前、「card-sixteen」16 GiB 在后。
//! 每一段每次调用交进来几条记进 `entries_handed_in`，用例拿它判三段都经它算了。
//! 核心层交来的设置（`use_the_settings_from_the_next_batch`）按先后记进 `settings_received`，每一段每次调用时照的是哪一组记进
//! `settings_in_use_at_each_call`：用例拿它判「切换从下一批起生效、一批之内不换」。连续不确定的计数不随设置清零。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, ClusterSegmentEntryKey,
    GpuComputedSegment, GpuDeviceIdentifier, GpuEntryDecision, GpuFailureOnTheDevice, GpuInventory,
    GpuMemoryInBytes, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
    UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
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

pub const EIGHT_GIBIBYTES: u64 = 8 << 30;
pub const SIXTEEN_GIBIBYTES: u64 = 16 << 30;
/// 连续几轮不确定就交 CPU 判（第 8 条的 N）。
pub const CONSECUTIVE_UNCERTAIN_ROUNDS_BEFORE_THE_CPU_JUDGES: u32 = 2;

pub fn card_named(name: &str) -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(name.as_bytes().to_vec())
}

/// 一组全合法的 `gpu` 设置：两张卡都点名，上限各取那张卡的显存总量，等 GPU 的上限取默认。
pub fn admissible_gpu_settings_naming_both_cards() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-sixteen"), card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![
            GpuMemoryInBytes(SIXTEEN_GIBIBYTES),
            GpuMemoryInBytes(EIGHT_GIBIBYTES),
        ]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// 一个条目这一轮怎么交：没注入照 CPU 写法确认；注入了不确定的，连续第 N 轮按 CPU 写法判、清零，之前各轮交不确定。
fn decide<EntryKey: Ord + Copy, Outcome>(
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

/// 用例不在意交回哪一个错时注入的那一个：「card-eight」那张卡上设备丢了。
pub fn device_lost_on_the_card_with_eight_gibibytes() -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::GpuFailed {
        device: card_named("card-eight"),
        cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
            reason: "假的：设备丢了".to_string(),
        },
    }
}

pub struct CpuBackedBackgroundGpuComputeForTests {
    /// 算到这一段就交回这个错（这一段的判定一条都不交、计数不动）。
    pub failing_segment: Option<(GpuComputedSegment, BackgroundGpuComputeFailed)>,
    /// 乙段这几条两份核判得不一样。
    pub region_records_uncertain: BTreeSet<RegionRecordEntryKey>,
    /// 甲段这几条两份核判得不一样。
    pub unit_checksums_uncertain: BTreeSet<RegionRecordEntryKey>,
    /// 甲段这几条两份核一致判成整单元 CRC 与位置项对不上（照 CPU 写法算出的值照交，只把判定翻成对不上）。
    pub unit_checksums_reported_mismatching: BTreeSet<RegionRecordEntryKey>,
    /// 丙段这几段两份核数得不一样。
    pub cluster_segments_uncertain: BTreeSet<ClusterSegmentEntryKey>,
    /// 每一段每次调用交进来几条，按调用的先后。
    pub entries_handed_in: Vec<(GpuComputedSegment, usize)>,
    /// 核心层交来的每一组设置，按先后。
    pub settings_received: Vec<AdmittedGpuSettings>,
    /// 每一段每次调用时照的是哪一组（最近交来的那一组；还没交来过是 `None`），按调用的先后。
    pub settings_in_use_at_each_call: Vec<(GpuComputedSegment, Option<AdmittedGpuSettings>)>,
    consecutive_uncertain_rounds_of_the_region_records: BTreeMap<RegionRecordEntryKey, u32>,
    consecutive_uncertain_rounds_of_the_unit_checksums: BTreeMap<RegionRecordEntryKey, u32>,
    consecutive_uncertain_rounds_of_the_cluster_segments: BTreeMap<ClusterSegmentEntryKey, u32>,
}

impl CpuBackedBackgroundGpuComputeForTests {
    pub fn with_nothing_injected() -> Self {
        Self {
            failing_segment: None,
            region_records_uncertain: BTreeSet::new(),
            unit_checksums_uncertain: BTreeSet::new(),
            unit_checksums_reported_mismatching: BTreeSet::new(),
            cluster_segments_uncertain: BTreeSet::new(),
            entries_handed_in: Vec::new(),
            settings_received: Vec::new(),
            settings_in_use_at_each_call: Vec::new(),
            consecutive_uncertain_rounds_of_the_region_records: BTreeMap::new(),
            consecutive_uncertain_rounds_of_the_unit_checksums: BTreeMap::new(),
            consecutive_uncertain_rounds_of_the_cluster_segments: BTreeMap::new(),
        }
    }

    /// 某一段交进来的条目数之和。
    pub fn entries_handed_in_to(&self, segment: GpuComputedSegment) -> usize {
        self.entries_handed_in
            .iter()
            .filter(|(handed_to, _)| *handed_to == segment)
            .map(|(_, entries)| entries)
            .sum()
    }

    fn fail_if_injected(
        &mut self,
        segment: GpuComputedSegment,
        entries: usize,
    ) -> Result<(), BackgroundGpuComputeFailed> {
        self.entries_handed_in.push((segment, entries));
        self.settings_in_use_at_each_call
            .push((segment, self.settings_received.last().cloned()));
        match &self.failing_segment {
            Some((failing_segment, failure)) if *failing_segment == segment => Err(failure.clone()),
            Some(_) | None => Ok(()),
        }
    }

    /// 交来的设置里参与的卡，按每一段每次调用的先后（还没交来过的那几次记成空）。
    pub fn participating_devices_at_each_call(&self) -> Vec<Vec<GpuDeviceIdentifier>> {
        self.settings_in_use_at_each_call
            .iter()
            .map(|(_segment, settings)| {
                settings.as_ref().map_or_else(Vec::new, |settings| {
                    settings
                        .participating_devices()
                        .iter()
                        .map(|participating| participating.device.clone())
                        .collect()
                })
            })
            .collect()
    }
}

impl GpuInventory for CpuBackedBackgroundGpuComputeForTests {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![
            (card_named("card-eight"), GpuMemoryInBytes(EIGHT_GIBIBYTES)),
            (
                card_named("card-sixteen"),
                GpuMemoryInBytes(SIXTEEN_GIBIBYTES),
            ),
        ]
    }
}

impl BackgroundGpuCompute for CpuBackedBackgroundGpuComputeForTests {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.fail_if_injected(GpuComputedSegment::UnitChecksumVerification, units.len())?;
        Ok(units
            .iter()
            .map(|unit| {
                let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
                let outcome = UnitChecksumOutcome {
                    computed_crc32c,
                    matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c
                        && !self
                            .unit_checksums_reported_mismatching
                            .contains(&unit.entry_key),
                };
                decide(
                    unit.entry_key,
                    self.unit_checksums_uncertain.contains(&unit.entry_key),
                    &mut self.consecutive_uncertain_rounds_of_the_unit_checksums,
                    outcome,
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
        self.fail_if_injected(
            GpuComputedSegment::RegionRecordClassification,
            records.len(),
        )?;
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
                decide(
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
        self.fail_if_injected(
            GpuComputedSegment::LiveSlotsPerClusterSegment,
            segments.len(),
        )?;
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
                decide(
                    *segment,
                    self.cluster_segments_uncertain.contains(segment),
                    &mut self.consecutive_uncertain_rounds_of_the_cluster_segments,
                    live_slots,
                )
            })
            .collect())
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.settings_received.push(settings.clone());
    }
}

/// 一份没注入故障的假实现，连同指着它的把手：用例留着前一个改注入、读记下的调用，后一个递给挂载或切换。
pub fn shared_cpu_backed_gpu_compute() -> (
    Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
    BackgroundGpuComputeHandle,
) {
    let shared = Arc::new(Mutex::new(
        CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
    ));
    let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(shared.clone());
    (shared, handle)
}

/// 等 GPU 结果的上限换成 `wait_limit` 的 [`admissible_gpu_settings_naming_both_cards`]。
pub fn admissible_gpu_settings_naming_both_cards_waiting_at_most(
    wait_limit: Duration,
) -> BackgroundComputeSettings {
    let BackgroundComputeSettings {
        background_compute,
        background_gpu_devices,
        background_gpu_memory_limit,
        background_gpu_wait_limit: _,
    } = admissible_gpu_settings_naming_both_cards();
    BackgroundComputeSettings {
        background_compute,
        background_gpu_devices,
        background_gpu_memory_limit,
        background_gpu_wait_limit: wait_limit,
    }
}
