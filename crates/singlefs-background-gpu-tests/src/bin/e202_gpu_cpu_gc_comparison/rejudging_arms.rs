//! GPU 多判臂与 CPU 多判臂（阳性对照）：把每条确认的在 CPU 上再判一遍。

use super::*;

// ---- GPU 多判臂的包装（登记第五节「阳性对照」第 1 条）：每个方法原样转给产品那一份，交回之后把每条确认的在 CPU 上再判一遍 ----

/// 一批里三段各自交进去的条目数（第七节甲类第 3 行：确认 + 不确定 + 交 CPU 判 = 交进去的条目数）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct EntriesHandedIn {
    pub(super) unit_checksums: u64,
    pub(super) region_records: u64,
    pub(super) cluster_segments: u64,
}

/// 再判一段的结果：再判了几条、与 GPU 确认的结果对不上几条。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RejudgeTally {
    pub(super) entries_rejudged: u64,
    pub(super) mismatches: u64,
}

impl RejudgeTally {
    pub(super) fn add(&mut self, later: RejudgeTally) {
        self.entries_rejudged += later.entries_rejudged;
        self.mismatches += later.mismatches;
    }
}

/// 包装在 GC 线程上记下的：多判花掉的线程 CPU（里层计时 E）、再判与对不上的条数、每批交进去的条目数、活槽段每次调用交进去的段数对不对。
#[derive(Debug, Default)]
pub(super) struct RejudgeLog {
    pub(super) inner_rejudge_cpu: Duration,
    pub(super) tally: RejudgeTally,
    pub(super) clock_read_failures: u64,
    handed_in_since_the_last_batch: EntriesHandedIn,
    pub(super) handed_in_of_each_batch: Vec<EntriesHandedIn>,
    pub(super) live_slot_calls_not_handing_in_every_full_segment: u64,
}

pub(super) type SharedRejudgeLog = Arc<Mutex<RejudgeLog>>;

/// 校验和段的再判：`crc32_castagnoli(单元字节)` 与 `expected_crc32c` 比，与 GPU 确认的结局逐条比。
fn rejudged_unit_checksums(
    units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    decisions: &[GpuEntryDecision<UnitChecksumOutcome>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    for (unit, decision) in units.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(outcome) = decision {
            let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
            let cpu_outcome = UnitChecksumOutcome {
                computed_crc32c,
                matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c,
            };
            tally.entries_rejudged += 1;
            if cpu_outcome != *outcome {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// 区域记录段的再判：映射条目照这一次交进来的那一份建按身份的索引（每次调用建一次，算在多判的活里），
/// 对每条确认的记录单独调 `scan_region_over_the_identity_index`（产品 GPU 包的 CPU 参照 `cpu_reference_classification` 就是这一调）。
fn rejudged_region_records(
    region: CompactionRegion,
    records: &[RegionRecordToClassifyOnTheGpu<'_>],
    mapping_entries: &[CentralMappingEntry],
    decisions: &[GpuEntryDecision<AllocationInRegion>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
    for (record, decision) in records.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(class) = decision {
            let cpu_class = cpu_reference_classification(
                region,
                record.record,
                &mapping_index,
                record.unit_bytes,
            );
            tally.entries_rejudged += 1;
            if cpu_class != *class {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// 活槽段的再判：对全部分配记录调一次 `live_slots_per_cluster_segment_by_the_records`，取每个确认段的那一个数。
fn rejudged_live_slots(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    segments: &[ClusterSegmentEntryKey],
    decisions: &[GpuEntryDecision<u64>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let live_slots = live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
    let unit_area_start_slot = unit_area.start.slot().0;
    for (segment, decision) in segments.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(gpu_live_slots) = decision {
            let segment_index =
                (segment.start_slot.0 - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS;
            let cpu_live_slots = live_slots
                .get(usize::try_from(segment_index).expect("段下标"))
                .copied();
            tally.entries_rejudged += 1;
            if cpu_live_slots != Some(*gpu_live_slots) {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// GPU 多判臂的那一份实现：里面是产品那一份（`BackgroundGpuComputeOnTheParticipatingGpus`；测试里换成在 CPU 上算的替身）。
pub(super) struct GpuComputeRejudgedOnTheCpu<Inner> {
    pub(super) inner: Inner,
    pub(super) log: SharedRejudgeLog,
}

impl<Inner> GpuComputeRejudgedOnTheCpu<Inner> {
    fn locked_log(&self) -> std::sync::MutexGuard<'_, RejudgeLog> {
        self.log
            .lock()
            .expect("多判记录只在 GC 线程上写，没在别处 panic 过")
    }

    /// 在这条线程的 CPU 钟里量 `rejudge` 用掉的 CPU，记进里层计时 E。
    fn timed_rejudge(&self, rejudge: impl FnOnce() -> RejudgeTally) {
        let (tally, spent) = thread_cpu_spent_in(rejudge);
        let mut log = self.locked_log();
        log.tally.add(tally);
        match spent {
            Some(spent) => log.inner_rejudge_cpu += spent,
            None => log.clock_read_failures += 1,
        }
    }
}

impl<Inner: GpuInventory> GpuInventory for GpuComputeRejudgedOnTheCpu<Inner> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.inner.devices_with_total_memory()
    }
}

impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeRejudgedOnTheCpu<Inner> {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        let decisions = self.inner.verify_unit_checksums(units)?;
        self.locked_log()
            .handed_in_since_the_last_batch
            .unit_checksums += u64::try_from(units.len()).expect("条目数");
        self.timed_rejudge(|| rejudged_unit_checksums(units, &decisions));
        Ok(decisions)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        let decisions = self
            .inner
            .classify_region_records(region, records, mapping_entries)?;
        self.locked_log()
            .handed_in_since_the_last_batch
            .region_records += u64::try_from(records.len()).expect("条目数");
        self.timed_rejudge(|| {
            rejudged_region_records(region, records, mapping_entries, &decisions)
        });
        Ok(decisions)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let decisions = self.inner.count_live_slots_per_cluster_segment(
            allocation_records,
            unit_area,
            segments,
        )?;
        {
            let mut log = self.locked_log();
            log.handed_in_since_the_last_batch.cluster_segments +=
                u64::try_from(segments.len()).expect("条目数");
            if u64::try_from(segments.len()).expect("段数")
                != unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS
            {
                log.live_slot_calls_not_handing_in_every_full_segment += 1;
            }
        }
        self.timed_rejudge(|| {
            rejudged_live_slots(allocation_records, unit_area, segments, &decisions)
        });
        Ok(decisions)
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.inner.use_the_settings_from_the_next_batch(settings);
    }

    fn use_no_gpu_from_the_next_batch(&mut self) {
        self.inner.use_no_gpu_from_the_next_batch();
    }

    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
        &mut self,
    ) -> LatestGpuSettingsHandedIn {
        self.inner
            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        self.inner.failure_under_the_settings_handed_in()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        let submissions = self.inner.take_the_submissions_of_the_batch();
        let mut log = self.locked_log();
        let handed_in = std::mem::take(&mut log.handed_in_since_the_last_batch);
        log.handed_in_of_each_batch.push(handed_in);
        submissions
    }
}

// ---- CPU 多判臂（登记第五节「阳性对照」第 2 条）：每次 GC 调用返回之后、窗口之内，对这次调用发出的每一批把三段的 CPU 写法各再跑一遍 ----

/// 一批搬的单元所在的 R：第一个搬走的单元的源落点所在的那一段。
fn region_of_the_batch(
    version: &TransactionOutput,
    unit_area_start_slot: u64,
) -> Option<CompactionRegion> {
    let relocated = version.units_relocated_by_this_publish.first()?;
    let source = relocated.source[0];
    CompactionRegion::of_one_cluster_segment(
        source.device,
        SlotNumber(segment_start_of(source.slot.0, unit_area_start_slot)),
    )
    .ok()
}

/// 三段的 CPU 写法在一批上各跑一遍：R 的区域扫描（映射取 `central_mapping_as_the_compaction_reads_it`，读单元从内存盘取）、
/// 对扫描判成可搬的每个单元算整单元 CRC 与位置项比、对每块盘的分配记录各调一次活槽段。交回再判了几条、对不上几条。
pub(super) fn rejudged_batch_on_the_cpu(
    version: &TransactionOutput,
    devices: &PoolDevices,
    unit_areas: &[UnitAreaOfOneDevice],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let Some(unit_area_start) = unit_areas.first().map(|unit_area| unit_area.start.slot().0) else {
        return tally;
    };
    let Some(region) = region_of_the_batch(version, unit_area_start) else {
        return tally;
    };
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
    let read_unit_bytes =
        |device: DeviceIdentity, slot: SlotNumber, length: usize| -> Option<Vec<u8>> {
            let (_identity, memory_device) = devices
                .iter()
                .find(|(identity, _device)| *identity == device)?;
            let mut bytes = vec![0u8; length];
            memory_device
                .read_at(slot.to_device_offset(), &mut bytes)
                .ok()?;
            Some(bytes)
        };
    let scan = scan_region_over_the_identity_index(
        region,
        &version.allocation_records,
        &mapping_index,
        &BTreeSet::new(),
        &BTreeSet::new(),
        read_unit_bytes,
    );
    if let Ok(scan) = scan {
        let expected_crc_by_slot: HashMap<u64, u32> = mapping_entries
            .iter()
            .flat_map(|entry| entry.locations.iter())
            .filter(|location| location.device == region.device())
            .map(|location| (location.slot.0, location.unit_checksum))
            .collect();
        for (placement, _unit) in scan.movable_units() {
            let length = usize::try_from(placement.span * LOCAL_SLOT_BYTES).expect("单元字节");
            let Some(bytes) = read_unit_bytes(region.device(), placement.slot, length) else {
                tally.mismatches += 1;
                continue;
            };
            tally.entries_rejudged += 1;
            if expected_crc_by_slot.get(&placement.slot.0) != Some(&crc32_castagnoli(&bytes)) {
                tally.mismatches += 1;
            }
        }
    } else {
        tally.mismatches += 1;
    }
    for unit_area in unit_areas {
        let device_records: Vec<AllocationRecord> = version
            .allocation_records
            .iter()
            .filter(|record| record.device == unit_area.device)
            .copied()
            .collect();
        let live_slots = live_slots_per_cluster_segment_by_the_records(&device_records, *unit_area);
        tally.entries_rejudged += u64::try_from(live_slots.len()).expect("段数");
    }
    tally
}

#[cfg(test)]
mod tests;
