//! 几个模块的测试共用的条目：最小池与小池、合成的调用记录、CPU 顶替的 GPU 计算、合成的卡与交替跑产物。

use super::*;

/// 造数据挂载之前树里已有的 inode 记录条数：新池新建文件建的第一个文件那一条（推的，读 `publish_first_file`；小池用例拿造池时量的回比）。
/// 造池本身现量这个数，不用它；只有一层树的算术用例与小池用例用它，所以只在测试里定义。
pub(super) const INODE_RECORDS_BEFORE_THE_DATA_MOUNT: u64 = 1;

pub(super) fn read_a_chunk(device: &RecordingMemoryDevice, chunk_index: u64) -> Vec<u8> {
    let mut read_back = vec![0u8; usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB")];
    device
        .read_at(
            DeviceOffsetInBytes(chunk_index * MEMORY_DEVICE_CHUNK_BYTES),
            &mut read_back,
        )
        .expect("块内读");
    read_back
}

/// 造法对拍用的最小池：P = 64（碎片段 2 段），盘取默认环长下 mkfs 认的最小一块；造得快，只核造法、不核几何成没成形。
pub(super) const LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL: u64 = 64;

pub(super) fn smallest_pool_on_the_clock(
    storage: ChunkStorage,
    clock: &mut ConstructionClock<'_>,
) -> StartingImage {
    construct_a_starting_image_on_the_clock(
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        smallest_device_bytes_the_make_filesystem_accepts(),
        LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL,
        storage,
        clock,
    )
    .expect("最小池照登记第十二节修订第 14 条的造法造得成：没有一次发布被拒，发布的形状与各阶段的发布数都与登记的算式相符")
}

pub(super) fn call_record_with(
    batches: u64,
    compute_records: Vec<BackgroundComputeOfOneBatch>,
) -> GcCallRecord {
    GcCallRecord {
        after_the_user_change: 0,
        window: None,
        inner_rejudge_cpu: Duration::ZERO,
        rejudge_tally: RejudgeTally::default(),
        started: batches > 0,
        compaction_publishes: batches,
        batches,
        units_relocated_of_each_batch: Vec::new(),
        stop_name: String::new(),
        compute_records,
        has_the_alarm: false,
        empty_segments_before_the_call: Vec::new(),
        empty_segments_after_the_call: Vec::new(),
    }
}

/// 在 CPU 上算的替身：校验和段与活槽段照 CPU 写法交回「两份核确认」，`corrupted_crc32c` 为真时把校验和段的结果算错一位。
pub(super) struct CpuBackedStandIn {
    pub(super) corrupted_crc32c: bool,
}

impl GpuInventory for CpuBackedStandIn {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        Vec::new()
    }
}

impl BackgroundGpuCompute for CpuBackedStandIn {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        Ok(units
            .iter()
            .map(|unit| {
                let computed = crc32_castagnoli(unit.unit_bytes) ^ u32::from(self.corrupted_crc32c);
                GpuEntryDecision::ConfirmedByBothKernels(UnitChecksumOutcome {
                    computed_crc32c: computed,
                    matches_the_expected_crc32c: computed == unit.expected_crc32c,
                })
            })
            .collect())
    }

    fn classify_region_records(
        &mut self,
        _region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        _mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        Ok(records
            .iter()
            .map(|_record| GpuEntryDecision::UncertainThisRound)
            .collect())
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let live_slots =
            live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
        Ok(segments
            .iter()
            .zip(live_slots)
            .map(|(_segment, live)| GpuEntryDecision::ConfirmedByBothKernels(live))
            .collect())
    }

    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
}

pub(super) fn units_to_verify(contents: &[Vec<u8>]) -> Vec<UnitChecksumToVerifyOnTheGpu<'_>> {
    contents
        .iter()
        .zip(0_u64..)
        .map(|(content, slot)| UnitChecksumToVerifyOnTheGpu {
            entry_key: singlefs_core::background_compute::RegionRecordEntryKey {
                device: DeviceIdentity(0),
                slot: SlotNumber(LOCAL_UNIT_AREA_START_SLOT + 2 * slot),
                span_slots: 2,
            },
            unit_bytes: content,
            expected_crc32c: crc32_castagnoli(content),
        })
        .collect()
}

pub(super) fn data_unit_records_in_the_segment(
    segment_index: u64,
    units: u64,
) -> Vec<AllocationRecord> {
    (0..units)
        .map(|unit_index| AllocationRecord {
            device: DeviceIdentity(0),
            slot: SlotNumber(
                LOCAL_UNIT_AREA_START_SLOT
                    + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
                    + 2 * unit_index,
            ),
            span_slots: 2,
            generation: singlefs_core::address::CheckpointTxg(7),
            is_released: false,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        })
        .collect()
}

/// 小池的活数据单元数：碎片段 64 段（P ÷ 32），一段的偏差压不过几何核的 90%。
pub(super) const SMALL_POOL_LIVE_DATA_UNITS: u64 = 2048;

/// 两条小池用例共用的起步镜像：每块盘取默认环长下 mkfs 认的最小一块（3072 MiB，即最小档的盘），P = 2048。
pub(super) fn small_pool_image() -> StartingImage {
    construct_a_starting_image_of_the_size(
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        smallest_device_bytes_the_make_filesystem_accepts(),
        SMALL_POOL_LIVE_DATA_UNITS,
    )
    .expect("小池照登记第十二节修订第 14 条的造法造得成：没有一次发布被拒，发布的形状与各阶段的发布数都与登记的算式相符")
}

pub(super) fn arguments_of(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_string()).collect()
}

/// 本机看得到的三张独显，按 PCI 地址次序。
pub(super) fn three_discrete_gpus_seen_on_this_host() -> Vec<DiscreteGpuSeenByThisProcess> {
    ["0000:01:00.0", "0000:02:00.0", "0000:03:00.0"]
        .into_iter()
        .map(|pci_bus_address| DiscreteGpuSeenByThisProcess {
            pci_bus_address: pci_bus_address.to_string(),
            model: format!("model at {pci_bus_address}"),
            driver: "driver".to_string(),
            driver_information: format!("driver information at {pci_bus_address}"),
        })
        .collect()
}

/// 服务交来本机全部三张独显，交来的卡串先后是总线地址 03、01、02（与 PCI 地址次序不同）。
pub(super) fn three_cards_handed_over_out_of_the_pci_order() -> CardsSeenByThisProcess {
    let handed_over_pci_bus_addresses_in_the_handed_over_order = vec![
        "0000:03:00.0".to_string(),
        "0000:01:00.0".to_string(),
        "0000:02:00.0".to_string(),
    ];
    CardsSeenByThisProcess {
        discrete_gpus: discrete_gpus_among_the_handed_over(
            three_discrete_gpus_seen_on_this_host(),
            &handed_over_pci_bus_addresses_in_the_handed_over_order,
        )
        .expect("交来的三张都在本机看得到的独显里"),
        handed_over_pci_bus_addresses_in_the_handed_over_order,
        vulkan_instance: None,
    }
}

pub(super) fn pci_bus_addresses_of(discrete_gpus: &[DiscreteGpuSeenByThisProcess]) -> Vec<&str> {
    discrete_gpus
        .iter()
        .map(|gpu| gpu.pci_bus_address.as_str())
        .collect()
}

/// 产物里已有的结果行，照装置写的那种行（`E202 name=<种类> 键=值 …`）读回来。
pub(super) fn lines_already_in_a_product_of(
    lines_without_the_prefix: &[&str],
) -> Vec<KindAndFieldsOfAResultLine> {
    lines_without_the_prefix
        .iter()
        .map(|line| {
            fields_of_a_result_line(&format!("{RESULT_LINE_PREFIX} {line}"))
                .expect("测试里写的是装置的结果行")
        })
        .collect()
}

/// 交替跑里名字是 `arm_names` 的那几条臂，连同它们挂载用的后台计算（按名字取臂，照给的次序）。
pub(super) fn lockstep_arms_requested_named(arm_names: &[&str]) -> Vec<(String, ComputeOfTheArm)> {
    arm_names
        .iter()
        .map(|arm_name| {
            let arm = LockstepArm::of_the_name(arm_name)
                .unwrap_or_else(|| panic!("测试里点的臂 {arm_name} 是交替跑的臂"));
            (arm.name(), arm.compute)
        })
        .collect()
}

/// 测试里的主机硬件读数。
pub(super) fn host_hardware_of_the_tests() -> HostHardware {
    HostHardware {
        cpu_model: "test_cpu_model".to_string(),
        logical_cpus: 64,
    }
}

/// 只交来一张卡（0000:01:00.0）、它的驱动信息是 `driver_information` 的这一台。
pub(super) fn one_card_handed_over_with_the_driver_information(
    driver_information: &str,
) -> CardsSeenByThisProcess {
    CardsSeenByThisProcess {
        discrete_gpus: vec![DiscreteGpuSeenByThisProcess {
            pci_bus_address: "0000:01:00.0".to_string(),
            model: "model at 0000:01:00.0".to_string(),
            driver: "driver".to_string(),
            driver_information: driver_information.to_string(),
        }],
        handed_over_pci_bus_addresses_in_the_handed_over_order: vec!["0000:01:00.0".to_string()],
        vulkan_instance: None,
    }
}

/// 交替跑第 `session_number` 趟、4GiB 那一档取卡之后那一段（[`plan_of_the_run_after_the_cards`]）。
pub(super) fn plan_of_a_lockstep_session(
    lines_already_in_the_product: &[KindAndFieldsOfAResultLine],
    session_number: u64,
    arms_requested: &[(String, ComputeOfTheArm)],
    cards_of_the_every_card_arm: CardsOfTheEveryCardArm,
    cards: &CardsSeenByThisProcess,
    host_hardware: &HostHardware,
) -> Result<PlanOfTheRun, RefusalBeforeTheRun> {
    plan_of_the_run_after_the_cards(&InputsBeforeTheRun {
        lines_already_in_the_product,
        session_number,
        run: RunResumedByGroup::Lockstep {
            tier: UnitAreaTier::FourGibibytes,
        },
        arms_requested,
        cards_of_the_every_card_arm,
        cards,
        host_hardware,
    })
}

/// 合成的交替跑一趟（4GiB 那一档，CPU 臂与点名一张卡臂）：`name=session_start`、这一组的 `name=hardware_group`，
/// 再从 `first_repeat` 起 5 次做完的重复，每次两条 `name=run` 与一行 `name=repeat_done`。CPU 臂每次用 1000000 纳秒 CPU，
/// 点名一张卡臂用 1000000 − `saving_nanoseconds`：量 S 每次都是 `saving_nanoseconds`。
pub(super) fn lines_of_a_synthetic_lockstep_session(
    session_number: u64,
    group: &HardwareGroup,
    first_repeat: u32,
    saving_nanoseconds: i64,
) -> Vec<String> {
    let mut lines = vec![
        format!("name=session_start session={session_number} command=lockstep"),
        format!(
            "name=hardware_group {}",
            group.fields_of_the_line(session_number, UnitAreaTier::FourGibibytes)
        ),
    ];
    for repeat in first_repeat..first_repeat + REPEATS_PER_ARM {
        for (arm, process_cpu_nanoseconds) in [
            ("cpu", 1_000_000_i64),
            ("gpu_one_card", 1_000_000 - saving_nanoseconds),
        ] {
            lines.push(format!(
                "name=run tier=4GiB arm={arm} repeat={repeat} session={session_number} void=no batches=100 \
                     gc_wall_nanoseconds=2000000 process_cpu_nanoseconds={process_cpu_nanoseconds} inner_rejudge_cpu_nanoseconds=0"
            ));
        }
        lines.push(format!(
            "name=repeat_done tier=4GiB repeat={repeat} session={session_number} group={}",
            group.identifier
        ));
    }
    lines
}

/// 合成产物读回来（[`lines_already_in_a_product_of`]）。
pub(super) fn lines_already_in_a_product_of_the_owned(
    lines: &[String],
) -> Vec<KindAndFieldsOfAResultLine> {
    lines_already_in_a_product_of(&lines.iter().map(String::as_str).collect::<Vec<&str>>())
}

/// 只跑 CPU 臂与点名一张卡臂、点名一张卡臂那一张驱动信息是 `driver_information` 的那一组。
pub(super) fn group_of_the_one_card_with_the_driver_information(
    driver_information: &str,
) -> HardwareGroup {
    hardware_group_of(
        &host_hardware_of_the_tests(),
        one_card_handed_over_with_the_driver_information(driver_information)
            .discrete_gpus
            .first(),
        None,
    )
}
