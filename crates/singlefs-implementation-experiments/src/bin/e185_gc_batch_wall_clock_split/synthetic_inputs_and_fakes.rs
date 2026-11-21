use super::*;

pub(super) fn record(slot: u64, span_slots: u16, is_released: bool) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(0),
        slot: SlotNumber(slot),
        span_slots,
        generation: singlefs_core::address::CheckpointTxg(7),
        is_released,
        quarantine_mark: singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
    }
}

pub(super) fn region_at(start_slot: u64) -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start_slot))
        .expect("64 的倍数")
}

pub(super) fn call_reading(
    kind: CallKind,
    scan_read_bytes: u64,
    expected: Option<u64>,
    copy_bytes: u64,
    batches_observed: u64,
) -> CallReading {
    CallReading {
        kind,
        outcome: "test".to_string(),
        wall_nanoseconds: 10_000_000,
        cpu_nanoseconds: 4_000_000,
        is_the_cpu_unreadable: false,
        device_totals: MethodTotals {
            calls: 3,
            bytes: 0,
            wall_nanoseconds: 5_500_000,
            cpu_nanoseconds: 0,
            wait_nanoseconds: 5_500_000,
        },
        per_method: BTreeMap::new(),
        scan_read_bytes_in_the_region: scan_read_bytes,
        scan_read_bytes_elsewhere: 0,
        // 搬迁副本与释放前读盘核都是每个搬走的数据单元两块盘各一份整单元，字节相同。
        release_check_read_bytes: copy_bytes,
        read_bytes_after_the_first_write: 0,
        write_bytes: copy_bytes + 65_536,
        relocation_copy_bytes: copy_bytes,
        is_every_destination_written_whole: true,
        relocated_units: copy_bytes / 65_536,
        expected_scan_read_bytes: expected,
        batches_observed,
        inclusion: inclusion_verdict(10_000_000, 4_000_000, 5_500_000),
    }
}

pub(super) fn intent_of(calls: Vec<CallReading>) -> IntentReading {
    IntentReading {
        region: Some(region_at(50_304)),
        calls,
        mount_failure: None,
        replay_material: Vec::new(),
    }
}

/// 一批里释放前读盘核读的字节：每个搬走的数据单元两块盘各读一份整单元。
pub(super) const fn release_check_read_bytes_of_a_batch(relocated_data_units: u64) -> u64 {
    DEVICES_HOLDING_A_COPY * relocated_data_units * LOCAL_DATA_UNIT_BYTES
}

pub(super) fn preparation_settings_with(
    threads: usize,
    memory_cap_bytes: u64,
) -> PreparationSettings {
    PreparationSettings {
        threads,
        memory_cap: MemoryCap {
            bytes: memory_cap_bytes,
            source: MemoryCapSource::Environment,
        },
    }
}

pub(super) fn small_point(pool_data_units: u64) -> SamplingPoint {
    SamplingPoint {
        fraction: LiveSlotFraction::ThreeQuarters,
        pool_data_units,
    }
}

pub(super) fn constructed_pool_without_devices() -> ConstructedPool {
    ConstructedPool {
        devices: Vec::new(),
        live_data_inode_by_slot: BTreeMap::new(),
        unit_area_start_slot: 0,
        overwritten_units: 0,
        filler_units: 0,
        floor_raises_pushed_by_the_session: 0,
        release_rounds: 0,
        is_every_overwritten_slot_released: true,
        construction_seconds: 0.0,
        overwrite_landing: None,
    }
}

pub(super) fn label_of_round(arm: DeviceArm, round: usize) -> MeasurementLabel {
    MeasurementLabel {
        arm,
        point: small_point(1024),
        batch_units: 8,
        round,
        attempt: 1,
        injection: WriteInjection::Nothing,
        choice: ChoiceOfTheRegion::Greedy,
    }
}

// ───────────── 第 3 次重跑（登记 research/prompts/e185-r3-prereg.md）：计时区间、等法、写输入、比对 ─────────────

/// 假钟的三个计数器（纳秒）：假步骤与假睡按设定往上加。
#[derive(Default)]
pub(super) struct FakeClockState {
    pub(super) thread_cpu: u64,
    pub(super) wall: u64,
}

pub(super) struct FakeClocks(pub(super) Rc<RefCell<FakeClockState>>);

impl SubmissionClocks for FakeClocks {
    fn thread_cpu(&mut self) -> Option<u64> {
        Some(self.0.borrow().thread_cpu)
    }

    fn process_cpu(&mut self) -> Option<u64> {
        Some(self.0.borrow().thread_cpu)
    }

    fn wall(&mut self) -> u64 {
        self.0.borrow().wall
    }
}

/// 假步骤：① 走 `preparation_nanoseconds`、⑤ 走 `wait_nanoseconds`，别的每步走 `other_nanoseconds`（线程钟与挂钟一起走）。
pub(super) struct FakeSteps {
    pub(super) clock: Rc<RefCell<FakeClockState>>,
    pub(super) preparation_nanoseconds: u64,
    pub(super) wait_nanoseconds: u64,
    pub(super) other_nanoseconds: u64,
}

impl FakeSteps {
    fn advance(&self, nanoseconds: u64) {
        let mut clock = self.clock.borrow_mut();
        clock.thread_cpu += nanoseconds;
        clock.wall += nanoseconds;
    }
}

impl SubmissionSteps for FakeSteps {
    fn prepare(&mut self) -> u64 {
        self.advance(self.preparation_nanoseconds);
        8 * LOCAL_GPU_SLOT_BYTES
    }

    fn encode_and_submit(&mut self) {
        self.advance(self.other_nanoseconds);
    }

    fn register_the_readback(&mut self) {
        self.advance(self.other_nanoseconds);
    }

    fn wait(&mut self) -> Option<u64> {
        self.advance(self.wait_nanoseconds);
        Some(0)
    }

    fn read_and_compare(&mut self) -> SubmissionComparison {
        self.advance(self.other_nanoseconds);
        SubmissionComparison {
            confirmed: 8,
            uncertain_entries: Vec::new(),
            kernel_one_differs_from_the_expected: 0,
            kernel_two_differs_from_the_expected: 0,
            are_the_sequence_numbers_holding: true,
        }
    }

    fn unmap(&mut self) {
        self.advance(self.other_nanoseconds);
    }
}

// ───────────── 第 3 次重跑：带卡的单测（gpu 特性，整条经 task-server/task submit 交；没有卡报 NOT_RUN_NO_GPU_ADAPTER） ─────────────

/// `units` 个按种子造的单元（偶数号 32 KiB、奇数号 16 KiB），各放一个 32 KiB 槽：（槽连放的字节、长度表）。CPU 参照 CRC 不在这里算，
/// 由只录那一段的 `cpu_reference_of_the_slots` 算、写进库。
pub(super) fn seeded_slots(units: u64) -> (Vec<u8>, Vec<u32>) {
    let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
    let mut slots = vec![0_u8; usize::try_from(units).expect("单元数") * slot_bytes];
    let mut lengths = Vec::new();
    for (unit, slot) in (0_u64..).zip(slots.chunks_mut(slot_bytes)) {
        let length = disagreement_unit_length(unit);
        disagreement_unit_content(3, InputForm::Split, 5, unit, &mut slot[..length]);
        lengths.push(u32::try_from(length).expect("长度"));
    }
    (slots, lengths)
}

/// 带卡单测的名字：取卡时报「本次未跑」用它，过库的项名在它前面接装置名。
pub(super) const SEEDED_UNITS_TEST_NAME: &str =
    "resident_kernels_agree_with_the_cpu_and_kernel_one_agrees_with_the_crates_on_seeded_units";

/// 九样计时件的合成读数（纳秒）：今天的 CRC 4、单路 1、三路 0.5；释放前读盘核 2、0.6、0.3；扫描 10、查索引 0.125、建索引 1（都取二进制里写得准的数）。
pub(super) fn synthetic_cpu_writing_pieces() -> CpuWritingPieces {
    CpuWritingPieces {
        crc_today: Some(4.0),
        crc_hardware_single: Some(1.0),
        crc_hardware_three_way: Some(0.5),
        release_check_crc_today: Some(2.0),
        release_check_crc_hardware_single: Some(0.6),
        release_check_crc_hardware_three_way: Some(0.3),
        scan_today: Some(10.0),
        scan_identity_index_lookup: Some(0.125),
        identity_index_build: Some(1.0),
    }
}

/// 一次调用的合成读数：两种硬件写法与 crates 都对上、字节相同。
pub(super) fn synthetic_cpu_writings_of_a_call() -> CpuWritingsOfTheCall {
    CpuWritingsOfTheCall {
        pieces: synthetic_cpu_writing_pieces(),
        choose_cpu_nanoseconds: Some(7.0),
        crc_bytes: 65_536,
        release_check_crc_bytes: 32_768,
        hardware_crc_bytes: [65_536, 65_536],
        hardware_release_check_crc_bytes: [32_768, 32_768],
        identity_index_entries: 1030,
        identity_index_location_keys: 2060,
        hardware_crc_mismatches: 0,
        identity_index_scan_matches: Some(true),
    }
}
