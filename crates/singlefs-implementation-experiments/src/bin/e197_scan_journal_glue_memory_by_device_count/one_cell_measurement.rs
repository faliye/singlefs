//! 一个进程量一格：在造的池上跑一臂、读量窗，打这一格的结果行。
use super::*;

impl CellOutcome {
    /// 读数 A2（读者被调 `read` 的次数 = 盘数 × 环槽数，不扫环的格是 0）与 A3（交回的表的键数 = 每盘填满槽数，不扫环的格是 0）都对得上；
    /// 经读缓存的两臂另要 A10：项数 = 登记的项数，值的字节和 = 项数 × 4096。
    pub(super) fn anchors_hold(&self) -> bool {
        let cache_holds = match (
            self.cache_entries,
            self.cache_entries_expected,
            self.cache_value_bytes,
        ) {
            (None, None, None) => true,
            (Some(entries), Some(expected), Some(value_bytes)) => {
                entries == expected && value_bytes == entries * JOURNAL_RECORD_BYTES
            }
            _ => false,
        };
        self.reads == self.reads_expected
            && self.records_returned == self.records_expected
            && cache_holds
    }

    pub(super) fn result_line(&self, sizes: StructureSizes) -> String {
        let unreadable = || "unreadable".to_string();
        let (peak, ends, after_return, rss_start, high_water_mark, reset) = match &self.resident {
            Some(reading) => (
                reading.peak_over_start_bytes.to_string(),
                if reading.at_each_device_end_bytes.is_empty() {
                    "none".to_string()
                } else {
                    reading
                        .at_each_device_end_bytes
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                },
                reading.after_return_bytes.to_string(),
                reading.rss_start_kib.to_string(),
                reading.high_water_mark_kib.to_string(),
                "ok".to_string(),
            ),
            None => (
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
            ),
        };
        let number_or_none = |value: Option<u64>| {
            value.map_or_else(|| "none".to_string(), |count| count.to_string())
        };
        format!(
            "E7RESULT name=e197 {} ring_slots={} filled_slots={} reads={} reads_expected={} records_returned={} records_expected={} anchors_hold={} resident_peak_over_start_bytes={} resident_at_each_device_end_bytes={} resident_after_return_bytes={} vm_rss_at_start_kib={} vm_hwm_kib={} high_water_mark_reset={} scan_seconds={:.6} table_digest={} cache_entries={} cache_entries_expected={} cache_value_bytes={} size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={}",
            self.specification_text,
            self.ring_slots,
            self.filled_slots,
            self.reads,
            self.reads_expected,
            self.records_returned,
            self.records_expected,
            self.anchors_hold(),
            peak,
            ends,
            after_return,
            rss_start,
            high_water_mark,
            reset,
            self.scan_seconds,
            self.table_digest,
            number_or_none(self.cache_entries),
            number_or_none(self.cache_entries_expected),
            number_or_none(self.cache_value_bytes),
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
        )
    }
}

/// 量窗起点、终点和池记下的轨迹合成一格的读数；开窗失败、终点读不出、轨迹里有读不出的，都交 `None`（V11：不当 0）。
pub(super) fn resident_reading_of(
    window: Option<ResidentWindow>,
    end: Option<ResidentWindowEnd>,
    pool: &GeneratedPool,
) -> Option<ResidentReading> {
    let (window, end) = (window?, end?);
    let at_each_device_end_bytes = pool
        .resident_kib_at_each_device_end
        .borrow()
        .iter()
        .map(|kibibytes| kibibytes.map(|kibibytes| window.resident_over_the_start_bytes(kibibytes)))
        .collect::<Option<Vec<i64>>>()?;
    Some(ResidentReading {
        peak_over_start_bytes: window.peak_over_the_start_bytes(&end),
        at_each_device_end_bytes,
        after_return_bytes: window.resident_over_the_start_bytes(end.resident_kib),
        rss_start_kib: window.rss_start_kib,
        high_water_mark_kib: end.high_water_mark_kib,
    })
}

/// measurement_pc（登记第五节、修订三第七节 A8）：量窗里分配并逐页写一份 256 MiB 的 `Vec<u8>`；缓冲还在的时候读量窗终点。
fn measure_the_positive_control_allocation() -> Option<ResidentWindowEnd> {
    let mut buffer =
        vec![0u8; usize::try_from(MEASUREMENT_POSITIVE_CONTROL_BYTES).expect("256 MiB")];
    for page_start in (0..buffer.len()).step_by(4096) {
        buffer[page_start] = 0xA5;
    }
    std::hint::black_box(&buffer);
    let end = ResidentWindowEnd::read_now();
    drop(buffer);
    end
}

/// 登记修订三第七节 A8：measurement_pc 的 H 落在 [254 MiB, 260 MiB]。
#[cfg(test)]
fn measurement_peak_is_in_the_registered_window(peak_over_start: i64) -> bool {
    (MEASUREMENT_WINDOW_LOW_BYTES..=MEASUREMENT_WINDOW_HIGH_BYTES).contains(&peak_over_start)
}

/// 跑一臂：交回的表，与经读缓存的两臂放掉缓存之前的读数（别的臂 `None`）。
pub(super) fn run_the_arm_over_the_pool(
    arm: Arm,
    pool: &GeneratedPool,
    system_configuration: &SystemConfiguration,
) -> (RecordTable, Option<CacheReading>) {
    let without_a_cache = |table: RecordTable| (table, None);
    match arm {
        Arm::Baseline => without_a_cache(scan_journal(pool, system_configuration)),
        Arm::BaselinePc => without_a_cache(scan_journal_keeping_the_scan_until_return(
            pool,
            system_configuration,
        )),
        Arm::Split => without_a_cache(scan_journal_merging_device_by_device(
            pool,
            system_configuration,
        )),
        Arm::SplitPc => without_a_cache(
            scan_journal_merging_device_by_device_keeping_the_duplicates(
                pool,
                system_configuration,
            ),
        ),
        Arm::ReadStageCache => {
            let (table, reading) =
                scan_journal_through_the_read_stage_cache(pool, system_configuration);
            (table, Some(reading))
        }
        Arm::ReadStageCacheKeepsZerosPc => {
            let (table, reading) =
                scan_journal_through_the_cache_keeping_zeros(pool, system_configuration);
            (table, Some(reading))
        }
        Arm::Empty | Arm::MeasurementPc => without_a_cache(RecordTable::new()),
    }
}

/// A10：经读缓存的两臂登记的项数。read_stage_cache 只收非 0 的槽：盘数 × 每盘填满槽数；阳性对照全收：盘数 × 环槽数。
fn expected_cache_entries(
    arm: Arm,
    devices: u64,
    filled_slots: u64,
    ring_slots: u64,
) -> Option<u64> {
    if !arm.reads_through_the_cache() {
        return None;
    }
    match arm {
        Arm::ReadStageCache => Some(devices * filled_slots),
        Arm::ReadStageCacheKeepsZerosPc => Some(devices * ring_slots),
        Arm::Empty
        | Arm::MeasurementPc
        | Arm::Baseline
        | Arm::BaselinePc
        | Arm::Split
        | Arm::SplitPc => None,
    }
}

/// 跑一格。`ring_slots` 生产取 [`RING_SLOTS`]，单测取小环。池与系统配置在量窗之前建好；
/// 空跑格还编一条记录（让编码那段代码进内存），量窗里只有扫环那一截。
/// 量窗（登记修订三）：建池、编探针记录之后、调臂之前开窗（重置 `VmHWM`、读 `VmRSS` 当起点）；交回之后、放掉交回值之前读终点
/// （经读缓存的两臂在放掉缓存之前，那一刻由 `scan_through` 读）。
pub(super) fn measure_one_cell_in_one_process(
    specification: CellSpecification,
    ring_slots: u64,
) -> CellOutcome {
    let ring_bytes = ring_slots * JOURNAL_RECORD_BYTES;
    let filled_slots = specification.fill.filled_slots_per_device(ring_slots);
    let specification_text = format!(
        "arm={} devices={} fill={} named={} repeat={}",
        specification.arm.name(),
        specification.devices,
        specification.fill.name(),
        specification.named,
        specification.repeat
    );
    let system_configuration = system_configuration_for(ring_bytes, specification.devices);
    let pool = GeneratedPool::new(
        specification.devices,
        TEST_IMAGE_DEFAULT_BYTES,
        ring_slots,
        production_slot_source(filled_slots, specification.named),
    );
    let probe = journal_record_for(PRODUCTION_INSTANCE, 1, 1, specification.named).to_bytes();
    assert!(
        JournalRecord::parse(&probe, unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER)).is_some(),
        "装置造的记录过不了 JournalRecord::parse"
    );
    drop(probe);
    let scans = specification.arm.scans();
    let devices = u64::from(specification.devices);
    let mut outcome = CellOutcome {
        specification_text,
        ring_slots,
        filled_slots,
        reads: 0,
        reads_expected: if scans { devices * ring_slots } else { 0 },
        records_returned: 0,
        records_expected: if scans { filled_slots } else { 0 },
        resident: None,
        scan_seconds: 0.0,
        table_digest: "none".to_string(),
        cache_entries: None,
        cache_entries_expected: expected_cache_entries(
            specification.arm,
            devices,
            filled_slots,
            ring_slots,
        ),
        cache_value_bytes: None,
    };
    let window = ResidentWindow::open();
    if specification.arm == Arm::MeasurementPc {
        let end = measure_the_positive_control_allocation();
        outcome.resident = resident_reading_of(window, end, &pool);
        return outcome;
    }
    if !scans {
        outcome.resident = resident_reading_of(window, ResidentWindowEnd::read_now(), &pool);
        return outcome;
    }
    let clock = Instant::now();
    let (table, cache_reading) =
        run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
    outcome.scan_seconds = clock.elapsed().as_secs_f64();
    // H 与期末值（登记修订三）：交回之后、放掉交回值之前；经读缓存的两臂取放掉缓存之前那一刻（缓存活到读阶段判完，登记修订一第五节）。
    let end = match cache_reading {
        Some(reading) => reading.resident_end,
        None => ResidentWindowEnd::read_now(),
    };
    outcome.resident = resident_reading_of(window, end, &pool);
    outcome.cache_entries = cache_reading.map(|reading| reading.entries);
    outcome.cache_value_bytes = cache_reading.map(|reading| reading.value_bytes);
    outcome.reads = pool.reads.get();
    outcome.records_returned = u64::try_from(table.len()).expect("表长装得进 u64");
    outcome.table_digest = table_digest(&table);
    outcome
}

#[cfg(test)]
mod tests;
