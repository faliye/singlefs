//! 产品 GPU 包对核心层 `BackgroundGpuCompute` 的实现（`background_gpu_compute::BackgroundGpuComputeThroughBothKernels`）里不要 GPU 的那一半：
//! 两份核那一层换成在 CPU 上算的替身（[`BothKernelsComputedOnTheCpu`]，两份「核」都照本包的 CPU 参照算，按条目键点名的几条让后一份算得不一样，
//! 可注入某一段交不成），比对、连续不确定计数、判定与出错的映射都走产品那一份代码。
//! 对拍的另一边是核心层测试用的假实现（`crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs`，经 `#[path]` 引进来，它只依赖核心层）：
//! 小输入上同样注入「这几条两份核判得不一样」「这一段出错」，三段逐轮交回的判定逐条相同（D26（后台整理与放置回收） 已定项 13 第 4、8 条）。
//! 另有一趟整理到停：同一历史的两个池，一个选 `cpu`、一个选 `gpu` 递进这份实现（背后是 CPU 替身），发的每一版、录制流、停在哪逐项相同。
//! 设置（D26 已定项 13 第 1–3 条）：核心层交来的每一组转到两份核那一层，挂载那一刻交一次、切换只在生效的那一批开头交，一批之内不交；
//! 交新设置不清三段的连续不确定计数（第 8 条）。出错（第 4 条）：报给核心层的错带着卡与原因。
//! 卡清单照本包开卡之后那一份答（`gpus_of_this_host::DiscreteGpusAfterTheOpening`）：挂载那一刻没开成而跳过的卡，挂着切换点到它照第 4 条拒。
#[path = "../../singlefs-verification/tests/common_background_gpu_compute/mod.rs"]
mod common_background_gpu_compute;
mod common_pool_on_memory_devices;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common_background_gpu_compute::{CpuBackedBackgroundGpuComputeForTests, EIGHT_GIBIBYTES};
use common_pool_on_memory_devices::pool_after_a_sequential_write_in_an_earlier_mount;
use singlefs_background_gpu::adapter_inventory::DiscreteGpuOnThisHost;
use singlefs_background_gpu::background_gpu::{
    BatchNotSplitOverTheGpus, ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed,
    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
};
use singlefs_background_gpu::background_gpu_compute::{
    core_failure_of_the_gpu, BackgroundGpuComputeThroughBothKernels, BothKernelsOfTheThreeSegments,
};
use singlefs_background_gpu::checksum_kernels::{
    cpu_reference_outcome, ChecksumKernelOutcome, UnitChecksumToVerify,
};
use singlefs_background_gpu::gpus_of_this_host::{
    DiscreteGpuNotOpenedAtTheMount, DiscreteGpusAfterTheOpening,
};
use singlefs_background_gpu::live_slot_kernels::cpu_reference_live_slots;
use singlefs_background_gpu::region_record_kernels::{
    cpu_reference_classification, RegionRecordToClassify,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark, UnitAreaStart};
use singlefs_core::background_compute::{
    admit_background_compute_settings_with_the_gpu_compute,
    admit_gpu_settings_against_the_inventory, AdmittedGpuSettings, BackgroundComputeBackend,
    BackgroundComputeError, BackgroundComputeOfThisMount, BackgroundComputePathTaken,
    BackgroundComputeSettings, BackgroundComputeSettingsRefused, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, BackgroundTask,
    BackgroundTaskAfterTheComputeError, BackgroundTaskStoppedAfterAComputeError,
    ClusterSegmentEntryKey, GpuComputedSegment, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount,
    GpuEntryCounts, GpuEntryDecision, GpuInventory, GpuLeftOutUnderItsMemoryLimit,
    GpuMemoryInBytes, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
    InputNotSupportedOnTheGpu, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    cluster_segment_entry_keys_of_the_unit_area, CompactionRunStop, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mounted_session::CompactionRunOutcome;
use singlefs_core::pointer::{LocationEntry, PointerHead};
use singlefs_core::records::mapping_key_for_data;
use singlefs_core::region_scan::{
    AllocationInRegion, CentralMappingEntry, CentralMappingIdentityIndex, RegionScanInconsistency,
};
use singlefs_core::unit::{build_data_unit, DataUnitIdentity, WriteOrder};

const REGION_DEVICE: DeviceIdentity = DeviceIdentity(0);
const MIRROR_DEVICE: DeviceIdentity = DeviceIdentity(1);
const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
const FILESYSTEM_IDENTIFIER_OF_THE_UNITS: [u8; 16] = [7; 16];
/// 测试自己定的树号。
const TREE_OF_THE_TEST_DATA: u64 = 11;
/// CPU 替身答的那一张卡。
const CARD_OF_THE_TEST_DOUBLE: &str = "card-of-the-cpu-test-double";
/// 切换用例里挂载那一刻没开成而跳过的那一张卡。
const CARD_NOT_OPENED_AT_THE_MOUNT: &str = "card-not-opened-at-the-mount";

fn card_of_the_test_double() -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(CARD_OF_THE_TEST_DOUBLE.as_bytes().to_vec())
}

/// 本机的一张 8 GiB 独显，总线地址就是 `name`（卡的标识取总线地址，`adapter_inventory` 文件头）。
fn discrete_gpu_of_eight_gibibytes(name: &str) -> DiscreteGpuOnThisHost {
    DiscreteGpuOnThisHost {
        position_in_the_enumeration: 0,
        identifier: GpuDeviceIdentifier::of_opaque_bytes(name.as_bytes().to_vec()),
        pci_bus_address: name.to_string(),
        name: name.to_string(),
        total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
    }
}

/// 只有 CPU 替身那一张卡、开成了。
fn only_the_card_of_the_test_double_opened() -> DiscreteGpusAfterTheOpening {
    DiscreteGpusAfterTheOpening {
        opened: vec![discrete_gpu_of_eight_gibibytes(CARD_OF_THE_TEST_DOUBLE)],
        not_opened: Vec::new(),
    }
}

/// CPU 替身那一层先后收到的：核心层转来的一组设置，或算了一段。
#[derive(Clone, Debug, PartialEq, Eq)]
enum EventOfTheKernelsOnTheCpu {
    SettingsReceived(AdmittedGpuSettings),
    SegmentComputed,
}

/// 往 CPU 替身里注入的：按条目键点名、这一轮两份「核」算得不一样的几条，与这一段交不成；连同替身那一层先后收到的设置与算的段。
struct InjectedIntoTheKernelsOnTheCpu {
    events: Vec<EventOfTheKernelsOnTheCpu>,
    unit_checksums_disagreeing: BTreeSet<RegionRecordEntryKey>,
    region_records_disagreeing: BTreeSet<RegionRecordEntryKey>,
    cluster_segments_disagreeing: BTreeSet<ClusterSegmentEntryKey>,
    unit_checksum_batch_failure: Option<ChecksumBatchFailed>,
    region_record_batch_failure: Option<RegionRecordBatchFailed>,
    live_slot_batch_failure: Option<LiveSlotBatchFailed>,
    /// 此刻这组设置下一批分不到卡上时答的那一种（`BothKernelsOfTheThreeSegments::batch_not_split_over_the_gpus_under_the_settings_in_use`）；
    /// 三段照产品那一层一样，分不到卡上就一段都不算、交回它。
    batch_not_split_over_the_gpus: Option<BatchNotSplitOverTheGpus>,
    /// 下一次收到核心层转来的设置时换成「此刻这组设置下一批分不到卡上」的那一种：模拟挂着切到一组一张卡都用不上的设置。
    batch_not_split_over_the_gpus_once_the_next_settings_arrive: Option<BatchNotSplitOverTheGpus>,
    /// 每一批算完之后核心层来取的「三段各自在卡上是怎么交的」：替身每次都交回这一份（用例照它核核心层记没记进那一批的记账）。
    submissions_reported_after_each_batch: GpuSubmissionsOfTheBatch,
}

impl InjectedIntoTheKernelsOnTheCpu {
    fn nothing() -> Self {
        Self {
            events: Vec::new(),
            unit_checksums_disagreeing: BTreeSet::new(),
            region_records_disagreeing: BTreeSet::new(),
            cluster_segments_disagreeing: BTreeSet::new(),
            unit_checksum_batch_failure: None,
            region_record_batch_failure: None,
            live_slot_batch_failure: None,
            batch_not_split_over_the_gpus: None,
            batch_not_split_over_the_gpus_once_the_next_settings_arrive: None,
            submissions_reported_after_each_batch: GpuSubmissionsOfTheBatch::default(),
        }
    }
}

/// 两份核那一层的 CPU 替身：两份都照本包的 CPU 参照算（甲 `cpu_reference_outcome`，乙 `cpu_reference_classification`，丙 `cpu_reference_live_slots`），
/// 点名的条目后一份算得不一样；注入了交不成就照注入的交回、一条都不交。卡清单照本包开卡之后那一份答，多半只有一张开成了的 8 GiB 的卡
/// （[`only_the_card_of_the_test_double_opened`]）。
struct BothKernelsComputedOnTheCpu {
    injected: Arc<Mutex<InjectedIntoTheKernelsOnTheCpu>>,
    discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
}

impl GpuInventory for BothKernelsComputedOnTheCpu {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.discrete_gpus_after_the_opening
            .devices_with_total_memory()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.discrete_gpus_after_the_opening
            .devices_not_opened_at_the_mount()
    }
}

impl BothKernelsOfTheThreeSegments for BothKernelsComputedOnTheCpu {
    fn unit_checksums_by_both_kernels(
        &mut self,
        units: &[UnitChecksumToVerify<'_, RegionRecordEntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        let mut injected = self.injected.lock().expect("用例没在别处 panic 过");
        match &injected.batch_not_split_over_the_gpus {
            None => {}
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                return Err(ChecksumBatchFailed::GpuFailed {
                    device: device.clone(),
                    cause: cause.clone(),
                });
            }
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                return Err(
                    ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                        gpus_left_out.clone(),
                    ),
                );
            }
        }
        injected
            .events
            .push(EventOfTheKernelsOnTheCpu::SegmentComputed);
        if let Some(failure) = &injected.unit_checksum_batch_failure {
            return Err(failure.clone());
        }
        Ok(units
            .iter()
            .map(|unit| {
                let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
                let independently_written = if injected
                    .unit_checksums_disagreeing
                    .contains(&unit.entry_key)
                {
                    ChecksumKernelOutcome {
                        computed_crc32c: outcome.computed_crc32c ^ 1,
                        matches_the_expected_crc32c: outcome.matches_the_expected_crc32c,
                    }
                } else {
                    outcome
                };
                (outcome, independently_written)
            })
            .collect())
    }

    fn region_record_classes_by_both_kernels(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, RegionRecordEntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed> {
        let mut injected = self.injected.lock().expect("用例没在别处 panic 过");
        match &injected.batch_not_split_over_the_gpus {
            None => {}
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                return Err(RegionRecordBatchFailed::GpuFailed {
                    device: device.clone(),
                    cause: cause.clone(),
                });
            }
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                return Err(
                    RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                        gpus_left_out.clone(),
                    ),
                );
            }
        }
        injected
            .events
            .push(EventOfTheKernelsOnTheCpu::SegmentComputed);
        if let Some(failure) = &injected.region_record_batch_failure {
            return Err(failure.clone());
        }
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
        Ok(records
            .iter()
            .map(|to_classify| {
                let class = cpu_reference_classification(
                    region,
                    to_classify.record,
                    &mapping_index,
                    to_classify.unit_bytes,
                );
                let independently_written = if injected
                    .region_records_disagreeing
                    .contains(&to_classify.entry_key)
                {
                    match class {
                        AllocationInRegion::Dead => AllocationInRegion::Inconsistent(
                            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
                        ),
                        AllocationInRegion::Movable(_)
                        | AllocationInRegion::Inconsistent(_)
                        | AllocationInRegion::Pinned
                        | AllocationInRegion::Quarantined => AllocationInRegion::Dead,
                    }
                } else {
                    class.clone()
                };
                (class, independently_written)
            })
            .collect())
    }

    fn live_slots_by_both_kernels(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed> {
        let mut injected = self.injected.lock().expect("用例没在别处 panic 过");
        match &injected.batch_not_split_over_the_gpus {
            None => {}
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                return Err(LiveSlotBatchFailed::GpuFailed {
                    device: device.clone(),
                    cause: cause.clone(),
                });
            }
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                return Err(
                    LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                        gpus_left_out.clone(),
                    ),
                );
            }
        }
        injected
            .events
            .push(EventOfTheKernelsOnTheCpu::SegmentComputed);
        if let Some(failure) = &injected.live_slot_batch_failure {
            return Err(failure.clone());
        }
        let reference = cpu_reference_live_slots(allocation_records, unit_area);
        Ok(segments
            .iter()
            .zip(reference)
            .map(|(segment, live_slots)| {
                if injected.cluster_segments_disagreeing.contains(segment) {
                    (live_slots, live_slots + 1)
                } else {
                    (live_slots, live_slots)
                }
            })
            .collect())
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        let mut injected = self.injected.lock().expect("用例没在别处 panic 过");
        injected
            .events
            .push(EventOfTheKernelsOnTheCpu::SettingsReceived(
                settings.clone(),
            ));
        if let Some(not_split) = injected
            .batch_not_split_over_the_gpus_once_the_next_settings_arrive
            .take()
        {
            injected.batch_not_split_over_the_gpus = Some(not_split);
        }
    }

    fn batch_not_split_over_the_gpus_under_the_settings_in_use(
        &self,
    ) -> Option<BatchNotSplitOverTheGpus> {
        self.injected
            .lock()
            .expect("用例没在别处 panic 过")
            .batch_not_split_over_the_gpus
            .clone()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        self.injected
            .lock()
            .expect("用例没在别处 panic 过")
            .submissions_reported_after_each_batch
            .clone()
    }
}

/// 产品那一份实现，两份核那一层是 CPU 替身；连同用例改注入用的那一份。
fn implementation_over_kernels_on_the_cpu() -> (
    BackgroundGpuComputeThroughBothKernels<BothKernelsComputedOnTheCpu>,
    Arc<Mutex<InjectedIntoTheKernelsOnTheCpu>>,
) {
    let injected = Arc::new(Mutex::new(InjectedIntoTheKernelsOnTheCpu::nothing()));
    (
        BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(
            BothKernelsComputedOnTheCpu {
                injected: Arc::clone(&injected),
                discrete_gpus_after_the_opening: only_the_card_of_the_test_double_opened(),
            },
        ),
        injected,
    )
}

fn region_under_test() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(REGION_DEVICE, SlotNumber(64 * 900))
        .expect("64 的倍数")
}

fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
    SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
}

fn unreleased_record(slot: SlotNumber, span_slots: u16) -> AllocationRecord {
    AllocationRecord {
        device: REGION_DEVICE,
        slot,
        span_slots,
        generation: BIRTH_TXG,
        is_released: false,
        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
    }
}

fn data_unit_and_key(transaction: u64) -> (Vec<u8>, Vec<u8>) {
    let write_order = WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    };
    let bytes = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER_OF_THE_UNITS,
        write_order,
        &[0x5A; 100],
    );
    let key = mapping_key_for_data(
        PointerHead {
            birth_tree: TreeIdentifier(TREE_OF_THE_TEST_DATA),
            birth_txg: BIRTH_TXG,
        },
        write_order,
    );
    (bytes, key)
}

fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
    [REGION_DEVICE, MIRROR_DEVICE].map(|device| LocationEntry {
        device,
        slot,
        unit_checksum: 0,
    })
}

/// 三段各一份小输入：甲三个单元（期望值一对两错），乙三条记录（可搬、已死、读不到），丙一块三整段的单元区与三条记录
/// （活槽数按段是 2、2、1：中间那一段有一条一槽的，还有一条两槽的罩住它的末一槽与下一段的头一槽）。
struct SmallInputsOfTheThreeSegments {
    unit_bytes: Vec<(RegionRecordEntryKey, Vec<u8>, u32)>,
    region_records: Vec<(AllocationRecord, Option<Vec<u8>>)>,
    mapping_entries: Vec<CentralMappingEntry>,
    live_slot_records: Vec<AllocationRecord>,
    unit_area: UnitAreaOfOneDevice,
}

fn small_inputs_of_the_three_segments() -> SmallInputsOfTheThreeSegments {
    let (movable_bytes, movable_key) = data_unit_and_key(4);
    let (dead_bytes, _dead_key) = data_unit_and_key(40);
    let region_records = vec![
        (
            unreleased_record(slot_in_region(0), 2),
            Some(movable_bytes.clone()),
        ),
        (
            unreleased_record(slot_in_region(2), 2),
            Some(dead_bytes.clone()),
        ),
        (unreleased_record(slot_in_region(4), 2), None),
    ];
    let unit_bytes = region_records
        .iter()
        .zip([true, false, false])
        .map(|((record, bytes), expected_is_correct)| {
            let bytes = bytes.clone().unwrap_or_else(|| vec![0xC3; 32]);
            let correct = singlefs_core::checksum::crc32_castagnoli(&bytes);
            let expected = if expected_is_correct {
                correct
            } else {
                correct ^ 0xFF
            };
            (RegionRecordEntryKey::of_the_record(record), bytes, expected)
        })
        .collect();
    let segment_slots = region_under_test().slot_count();
    let unit_area = UnitAreaOfOneDevice {
        device: REGION_DEVICE,
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 3 * segment_slots + 7,
    };
    let unit_area_start = unit_area.start.slot().0;
    let live_slot_records = vec![
        unreleased_record(SlotNumber(unit_area_start + 1), 2),
        unreleased_record(SlotNumber(unit_area_start + segment_slots + 3), 1),
        unreleased_record(SlotNumber(unit_area_start + 2 * segment_slots - 1), 2),
    ];
    SmallInputsOfTheThreeSegments {
        unit_bytes,
        region_records,
        mapping_entries: vec![CentralMappingEntry {
            key: movable_key,
            locations: locations_at(slot_in_region(0)),
        }],
        live_slot_records,
        unit_area,
    }
}

/// 一轮三段各交回的判定（出错就是那一段的错）。
#[derive(Debug, PartialEq, Eq)]
struct DecisionsOfTheThreeSegments {
    unit_checksums: Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>,
    region_records: Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed>,
    live_slots: Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed>,
}

fn one_round_of_the_three_segments(
    implementation: &mut dyn BackgroundGpuCompute,
    inputs: &SmallInputsOfTheThreeSegments,
) -> DecisionsOfTheThreeSegments {
    let units: Vec<UnitChecksumToVerifyOnTheGpu<'_>> = inputs
        .unit_bytes
        .iter()
        .map(
            |(entry_key, bytes, expected_crc32c)| UnitChecksumToVerifyOnTheGpu {
                entry_key: *entry_key,
                unit_bytes: bytes,
                expected_crc32c: *expected_crc32c,
            },
        )
        .collect();
    let records: Vec<RegionRecordToClassifyOnTheGpu<'_>> = inputs
        .region_records
        .iter()
        .map(|(record, bytes)| RegionRecordToClassifyOnTheGpu {
            entry_key: RegionRecordEntryKey::of_the_record(record),
            record: *record,
            unit_bytes: bytes.as_deref(),
        })
        .collect();
    DecisionsOfTheThreeSegments {
        unit_checksums: implementation.verify_unit_checksums(&units),
        region_records: implementation.classify_region_records(
            region_under_test(),
            &records,
            &inputs.mapping_entries,
        ),
        live_slots: implementation.count_live_slots_per_cluster_segment(
            &inputs.live_slot_records,
            inputs.unit_area,
            &cluster_segment_entry_keys_of_the_unit_area(inputs.unit_area),
        ),
    }
}

/// 三段各点名一条：甲是 R 里最后那一条记录的单元（读不到的那一条，核的是垫上的字节），乙是已死的那一条记录，丙是中间那一段。
fn entries_named_in_each_segment(
    inputs: &SmallInputsOfTheThreeSegments,
) -> (
    RegionRecordEntryKey,
    RegionRecordEntryKey,
    ClusterSegmentEntryKey,
) {
    let (last_record, _bytes) = inputs.region_records.last().expect("乙段三条记录");
    let middle_segment = cluster_segment_entry_keys_of_the_unit_area(inputs.unit_area)[1];
    (
        RegionRecordEntryKey::of_the_record(last_record),
        RegionRecordEntryKey::of_the_record(&inputs.region_records[1].0),
        middle_segment,
    )
}

fn inject_disagreements(
    injected: &Arc<Mutex<InjectedIntoTheKernelsOnTheCpu>>,
    fake: &mut CpuBackedBackgroundGpuComputeForTests,
    named: Option<(
        RegionRecordEntryKey,
        RegionRecordEntryKey,
        ClusterSegmentEntryKey,
    )>,
) {
    let (unit_checksums, region_records, cluster_segments) = match named {
        Some((unit_checksum, region_record, cluster_segment)) => (
            BTreeSet::from([unit_checksum]),
            BTreeSet::from([region_record]),
            BTreeSet::from([cluster_segment]),
        ),
        None => (BTreeSet::new(), BTreeSet::new(), BTreeSet::new()),
    };
    let mut injected = injected.lock().expect("用例没在别处 panic 过");
    injected.unit_checksums_disagreeing = unit_checksums.clone();
    injected.region_records_disagreeing = region_records.clone();
    injected.cluster_segments_disagreeing = cluster_segments.clone();
    fake.unit_checksums_uncertain = unit_checksums;
    fake.region_records_uncertain = region_records;
    fake.cluster_segments_uncertain = cluster_segments;
}

/// 第 8 条：三段各点名一条，前两轮两份核判得不一样、最后一轮一样。产品那一份（背后 CPU 替身）与核心层测试用的假实现逐轮逐条交回相同的判定：
/// 头一轮那几条不确定、接着一轮按 CPU 判、最后一轮确认，别的条目每轮确认。
#[test]
fn decisions_over_three_rounds_agree_with_the_fake_of_the_core_on_every_segment() {
    let inputs = small_inputs_of_the_three_segments();
    let named = entries_named_in_each_segment(&inputs);
    let (mut implementation, injected) = implementation_over_kernels_on_the_cpu();
    let mut fake = CpuBackedBackgroundGpuComputeForTests::with_nothing_injected();
    let mut rounds_of_the_implementation = Vec::new();
    let mut rounds_of_the_fake = Vec::new();
    for named_this_round in [Some(named), Some(named), None] {
        inject_disagreements(&injected, &mut fake, named_this_round);
        rounds_of_the_implementation.push(one_round_of_the_three_segments(
            &mut implementation,
            &inputs,
        ));
        rounds_of_the_fake.push(one_round_of_the_three_segments(&mut fake, &inputs));
    }
    assert_eq!(
        rounds_of_the_implementation, rounds_of_the_fake,
        "三轮三段逐条相同"
    );
    let second_round = &rounds_of_the_implementation[1];
    assert!(
        matches!(
            second_round.unit_checksums.as_deref(),
            Ok([_, _, GpuEntryDecision::JudgedByTheCpuReference(_)])
        ) && matches!(
            second_round.region_records.as_deref(),
            Ok([
                _,
                GpuEntryDecision::JudgedByTheCpuReference(AllocationInRegion::Dead),
                _
            ])
        ) && matches!(
            second_round.live_slots.as_deref(),
            Ok([_, GpuEntryDecision::JudgedByTheCpuReference(2), _])
        ),
        "第二轮：点名的那几条按 CPU 判（不然这条用例没比到 N = 2 那一步）：{second_round:?}"
    );
    assert!(
        matches!(
            rounds_of_the_implementation[0].region_records.as_deref(),
            Ok([
                GpuEntryDecision::ConfirmedByBothKernels(AllocationInRegion::Movable(_)),
                GpuEntryDecision::UncertainThisRound,
                _
            ])
        ),
        "头一轮：可搬的确认、点名的那一条不确定：{:?}",
        rounds_of_the_implementation[0]
    );
}

/// 第 4 条：这一轮某一段交不成（GPU 出错、两种超时）时三段各自报成核心层的 `GpuFailed`，计数不动：前一轮不确定的那一条，出错之后再不确定一轮
/// 就按 CPU 判（出错那一轮不算一轮、也不清零），与核心层测试用的假实现逐轮相同。第一版不支持的输入报成 `InputNotSupportedOnTheGpu`。
#[test]
fn the_batch_that_fails_on_the_gpu_or_times_out_is_reported_as_gpu_failed_and_leaves_the_consecutive_counts(
) {
    let inputs = small_inputs_of_the_three_segments();
    let named = entries_named_in_each_segment(&inputs);
    let card = card_of_the_test_double();
    let wait_limit = Duration::from_secs(10);
    // 循环里同一套输入、一份实现一份假实现逐种注入，不建池
    for cause in [
        GpuFailureCause::UncapturedErrorOrDeviceLost {
            reason: "假的：设备丢了".to_string(),
        },
        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
        GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit },
    ] {
        let (mut implementation, injected) = implementation_over_kernels_on_the_cpu();
        let mut fake = CpuBackedBackgroundGpuComputeForTests::with_nothing_injected();
        inject_disagreements(&injected, &mut fake, Some(named));
        let first_round = (
            one_round_of_the_three_segments(&mut implementation, &inputs),
            one_round_of_the_three_segments(&mut fake, &inputs),
        );
        {
            let mut injected = injected.lock().expect("用例没在别处 panic 过");
            injected.unit_checksum_batch_failure = Some(ChecksumBatchFailed::GpuFailed {
                device: card.clone(),
                cause: cause.clone(),
            });
            injected.region_record_batch_failure = Some(RegionRecordBatchFailed::GpuFailed {
                device: card.clone(),
                cause: cause.clone(),
            });
            injected.live_slot_batch_failure = Some(LiveSlotBatchFailed::GpuFailed {
                device: card.clone(),
                cause: cause.clone(),
            });
        }
        let failed_round_of_the_implementation =
            one_round_of_the_three_segments(&mut implementation, &inputs);
        let reported_with_the_card_and_the_cause = core_failure_of_the_gpu(&card, &cause);
        assert_eq!(
            failed_round_of_the_implementation,
            DecisionsOfTheThreeSegments {
                unit_checksums: Err(reported_with_the_card_and_the_cause.clone()),
                region_records: Err(reported_with_the_card_and_the_cause.clone()),
                live_slots: Err(reported_with_the_card_and_the_cause),
            },
            "三段都报出这张卡与这个原因：{cause:?}"
        );
        {
            let mut injected = injected.lock().expect("用例没在别处 panic 过");
            injected.unit_checksum_batch_failure = None;
            injected.region_record_batch_failure = None;
            injected.live_slot_batch_failure = None;
        }
        let after_the_failure = (
            one_round_of_the_three_segments(&mut implementation, &inputs),
            one_round_of_the_three_segments(&mut fake, &inputs),
        );
        assert_eq!(
            first_round.0, first_round.1,
            "出错之前那一轮与假实现相同：{cause:?}"
        );
        assert_eq!(
            after_the_failure.0, after_the_failure.1,
            "出错之后那一轮与假实现相同（假实现那一轮没交活）：{cause:?}"
        );
        assert!(
            matches!(
                after_the_failure.0.live_slots.as_deref(),
                Ok([_, GpuEntryDecision::JudgedByTheCpuReference(_), _])
            ),
            "出错那一轮不算一轮、不清零：前后两轮连续不确定，按 CPU 判：{cause:?}"
        );
    }
    let (mut implementation, injected) = implementation_over_kernels_on_the_cpu();
    {
        let mut injected = injected.lock().expect("用例没在别处 panic 过");
        let smallest_piece_not_held = |smallest_piece_bytes: u64| SmallestPieceNotHeldOnTheGpu {
            device: card.clone(),
            entry_position: 0,
            memory_limit: Some(GpuMemoryInBytes(50)),
            smallest_piece_bytes,
        };
        injected.unit_checksum_batch_failure = Some(
            ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held(64)),
        );
        injected.region_record_batch_failure =
            Some(RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(
                smallest_piece_not_held(32 << 10),
            ));
        injected.live_slot_batch_failure = Some(
            LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held(104)),
        );
    }
    let entry_larger_than_a_chunk = |input_bytes: u64| {
        BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: card_of_the_test_double(),
                entry_position: 0,
                input_bytes,
            },
        )
    };
    assert_eq!(
        one_round_of_the_three_segments(&mut implementation, &inputs),
        DecisionsOfTheThreeSegments {
            unit_checksums: Err(entry_larger_than_a_chunk(64)),
            region_records: Err(entry_larger_than_a_chunk(32 << 10)),
            live_slots: Err(entry_larger_than_a_chunk(104)),
        },
        "一张卡连最小的一片都装不下（一张卡都没交活）：报出哪张卡、哪一条、最小的一片多少字节"
    );
}

/// 第 4 条：核心层挂载与切换照这份实现答的卡清单校验——点名它答的那张卡照收，点名别的卡照「点名的卡不存在」拒。
#[test]
fn the_core_admits_gpu_settings_against_the_inventory_the_implementation_answers() {
    let (implementation, _injected) = implementation_over_kernels_on_the_cpu();
    let handle = BackgroundGpuComputeHandle::of_an_implementation(implementation);
    let settings_naming = |device: GpuDeviceIdentifier| BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![device]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    assert!(
        admit_background_compute_settings_with_the_gpu_compute(
            settings_naming(card_of_the_test_double()),
            Some(handle.clone()),
        )
        .is_ok(),
        "点名实现答的那张卡、上限等于它的显存总量：照收"
    );
    let other_card = GpuDeviceIdentifier::of_opaque_bytes(b"another-card".to_vec());
    assert_eq!(
        admit_background_compute_settings_with_the_gpu_compute(
            settings_naming(other_card.clone()),
            Some(handle),
        )
        .map(|_admitted| ()),
        Err(
            BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                device: other_card
            }
        ),
        "点名实现不认的卡：拒"
    );
}

/// 第 4 条「挂载或切换那一刻……点名的卡不存在，就拒绝挂载（或拒绝切换）」：
/// 挂载时给了卡列表、没点名的那张卡没开成而被跳过；挂着切换点到它（点名它，或不给卡列表、本机全部独显都参与）照拒，校验同挂载那一刻，
/// 生效的与待生效的都不动，实现只收到挂载那一组。经核心层的切换入口与产品那一份实现（两份核那一层是 CPU 替身）走。
#[test]
fn the_switch_to_a_card_not_opened_at_the_mount_is_refused() {
    let injected = Arc::new(Mutex::new(InjectedIntoTheKernelsOnTheCpu::nothing()));
    let cause_at_the_mount = GpuFailureCause::DeviceOrPipelineDidNotStart {
        reason: "起不了设备：造出来的坏卡".to_string(),
    };
    let implementation = BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(
        BothKernelsComputedOnTheCpu {
            injected: Arc::clone(&injected),
            discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening {
                opened: vec![discrete_gpu_of_eight_gibibytes(CARD_OF_THE_TEST_DOUBLE)],
                not_opened: vec![DiscreteGpuNotOpenedAtTheMount {
                    discrete_gpu: discrete_gpu_of_eight_gibibytes(CARD_NOT_OPENED_AT_THE_MOUNT),
                    cause: cause_at_the_mount.clone(),
                }],
            },
        },
    );
    let mount_settings = settings_on_the_card_of_the_test_double(None, Duration::from_secs(10));
    let mut background_compute = BackgroundComputeOfThisMount::of_a_mount(
        admit_background_compute_settings_with_the_gpu_compute(
            mount_settings.clone(),
            Some(BackgroundGpuComputeHandle::of_an_implementation(
                implementation,
            )),
        )
        .expect("点名开成了的那张卡：挂载照收"),
    );
    let card_not_opened =
        GpuDeviceIdentifier::of_opaque_bytes(CARD_NOT_OPENED_AT_THE_MOUNT.as_bytes().to_vec());
    let refused_as_not_opened_at_the_mount = Err(
        BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
            GpuDeviceNotOpenedAtTheMount {
                device: card_not_opened.clone(),
                failure_at_the_mount: core_failure_of_the_gpu(
                    &card_not_opened,
                    &cause_at_the_mount,
                ),
            },
        ),
    );
    assert_eq!(
        background_compute.request_switch_from_the_next_batch(
            BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(vec![card_not_opened.clone()]),
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: Duration::from_secs(10),
            },
            None,
        ),
        refused_as_not_opened_at_the_mount,
        "点名挂载那一刻没开成的卡：拒绝切换，报出它与没开成的原因"
    );
    assert_eq!(
        background_compute.request_switch_from_the_next_batch(
            BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: None,
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: Duration::from_secs(10),
            },
            None,
        ),
        refused_as_not_opened_at_the_mount,
        "不给卡列表：本机全部独显都参与，里面有没开成的那张，拒绝切换"
    );
    assert_eq!(
        background_compute.requested_from_the_next_batch(),
        None,
        "两次都被拒：没有记成待生效"
    );
    assert_eq!(
        background_compute.in_effect().settings(),
        &mount_settings,
        "生效的仍是挂载那一组"
    );
    assert_eq!(
        injected.lock().expect("用例没在别处 panic 过").events,
        vec![EventOfTheKernelsOnTheCpu::SettingsReceived(
            admitted_on_the_card_of_the_test_double(None, Duration::from_secs(10))
        )],
        "实现只收到挂载那一组"
    );
}

/// D26 已定项 13「两份实现在测试里对拍」的不起卡那一半：同一历史的两个池，一个重挂选 `cpu`、一个选 `gpu` 递进产品那一份实现（背后是 CPU 替身），
/// 各推一趟整理到停。两趟发的每一版、录制流、盘上字节、停在哪、搬了几批、整理观测与 discard 逐项相同；`gpu` 那一趟每一批走 GPU、三段都算了、全部确认。
#[test]
fn the_compaction_run_through_the_implementation_over_kernels_on_the_cpu_ends_exactly_as_the_cpu_path(
) {
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut cpu_pool = history.copied_with_an_empty_stream();
    let mut gpu_pool = history.copied_with_an_empty_stream();
    drop(history);
    let mut cpu_session = cpu_pool
        .remounted_with_compaction_forced_to_start(BackgroundComputeSettings::PRODUCT, None);
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_pool.devices)
        .expect("整理一趟");
    let (implementation, _injected) = implementation_over_kernels_on_the_cpu();
    let mut gpu_session = gpu_pool.remounted_with_compaction_forced_to_start(
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_of_the_test_double()]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    let gpu_run = gpu_session
        .run_compaction(&mut gpu_pool.devices)
        .expect("整理一趟");
    assert!(
        gpu_run.batches_published > 0,
        "这一趟签了意图、搬了批（不然这条用例什么都没比）：{:?}",
        gpu_run.outcome
    );
    assert_eq!(
        format!("{:?}", gpu_run.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        gpu_run.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        gpu_run.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        format!("{:?}", gpu_run.discards),
        format!("{:?}", cpu_run.discards),
        "发布之后的 discard 相同"
    );
    assert_eq!(
        gpu_session.compaction.observations, cpu_session.compaction.observations,
        "整理观测逐项相同"
    );
    assert_eq!(
        gpu_pool.recorded_operations(),
        cpu_pool.recorded_operations(),
        "重挂之后的录制流逐步相同"
    );
    assert_eq!(
        gpu_pool.written_sectors_of_every_device(),
        cpu_pool.written_sectors_of_every_device(),
        "盘上逐字节相同"
    );
    assert!(
        cpu_run
            .background_compute_of_each_batch
            .iter()
            .all(|batch| matches!(batch.path_taken, BackgroundComputePathTaken::Cpu(_))),
        "cpu 那一趟每一批走 CPU"
    );
    let mut entries_of_the_run: BTreeMap<&str, GpuEntryCounts> = BTreeMap::new();
    for batch in &gpu_run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Gpu(gpu_work) = &batch.path_taken else {
            panic!("gpu 那一趟每一批走 GPU：{batch:?}");
        };
        for (segment_name, counts) in [
            ("甲", gpu_work.unit_checksum_verification),
            ("乙", gpu_work.region_record_classification),
            ("丙", gpu_work.live_slots_per_cluster_segment),
        ] {
            let total = entries_of_the_run.entry(segment_name).or_default();
            total.confirmed_by_both_kernels += counts.confirmed_by_both_kernels;
            total.uncertain_this_round += counts.uncertain_this_round;
            total.judged_by_the_cpu_reference += counts.judged_by_the_cpu_reference;
        }
    }
    for (segment_name, counts) in &entries_of_the_run {
        assert!(
            counts.confirmed_by_both_kernels > 0
                && counts.uncertain_this_round + counts.judged_by_the_cpu_reference == 0,
            "{segment_name} 段经实现算了、全部确认：{counts:?}"
        );
    }
    assert_eq!(
        entries_of_the_run.len(),
        3,
        "三段都算了：{entries_of_the_run:?}"
    );
}

/// 用例给的一组 `gpu` 设置照 CPU 替身答的那一张卡校验过、交给实现的样子。
fn admitted_on_the_card_of_the_test_double(
    memory_limit: Option<GpuMemoryInBytes>,
    wait_limit: Duration,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        &settings_on_the_card_of_the_test_double(memory_limit, wait_limit),
        &BothKernelsComputedOnTheCpu {
            injected: Arc::new(Mutex::new(InjectedIntoTheKernelsOnTheCpu::nothing())),
            discrete_gpus_after_the_opening: only_the_card_of_the_test_double_opened(),
        },
    )
    .expect("点名替身答的那张卡、上限不超过它的显存总量、等的上限不是 0")
}

fn settings_on_the_card_of_the_test_double(
    memory_limit: Option<GpuMemoryInBytes>,
    wait_limit: Duration,
) -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_of_the_test_double()]),
        background_gpu_memory_limit: memory_limit.map(|limit| vec![limit]),
        background_gpu_wait_limit: wait_limit,
    }
}

/// 第 1、8 条：两轮之间交一组新设置（换了显存上限与等的上限），设置转到两份核那一层，三段的连续不确定计数照旧留着：
/// 交之前那一轮不确定的那几条，交之后再不确定一轮就按 CPU 判，与没交过设置的假实现逐条相同。
#[test]
fn settings_handed_over_between_two_rounds_reach_the_kernels_and_keep_the_consecutive_uncertainty_counts(
) {
    let inputs = small_inputs_of_the_three_segments();
    let named = entries_named_in_each_segment(&inputs);
    let (mut implementation, injected) = implementation_over_kernels_on_the_cpu();
    let mut fake = CpuBackedBackgroundGpuComputeForTests::with_nothing_injected();
    inject_disagreements(&injected, &mut fake, Some(named));
    let before_the_switch = one_round_of_the_three_segments(&mut implementation, &inputs);
    one_round_of_the_three_segments(&mut fake, &inputs);
    assert!(
        matches!(
            before_the_switch.live_slots.as_deref(),
            Ok([_, GpuEntryDecision::UncertainThisRound, _])
        ),
        "交设置之前那一轮：点名的那一段不确定：{before_the_switch:?}"
    );
    let switched = admitted_on_the_card_of_the_test_double(
        Some(GpuMemoryInBytes(EIGHT_GIBIBYTES / 2)),
        Duration::from_secs(3),
    );
    implementation.use_the_settings_from_the_next_batch(&switched);
    assert_eq!(
        injected
            .lock()
            .expect("用例没在别处 panic 过")
            .events
            .last(),
        Some(&EventOfTheKernelsOnTheCpu::SettingsReceived(switched)),
        "交来的设置转到两份核那一层"
    );
    let after_the_switch = one_round_of_the_three_segments(&mut implementation, &inputs);
    assert_eq!(
        after_the_switch,
        one_round_of_the_three_segments(&mut fake, &inputs),
        "交设置之后那一轮与假实现逐条相同"
    );
    assert!(
        matches!(
            after_the_switch.unit_checksums.as_deref(),
            Ok([_, _, GpuEntryDecision::JudgedByTheCpuReference(_)])
        ) && matches!(
            after_the_switch.region_records.as_deref(),
            Ok([_, GpuEntryDecision::JudgedByTheCpuReference(_), _])
        ) && matches!(
            after_the_switch.live_slots.as_deref(),
            Ok([_, GpuEntryDecision::JudgedByTheCpuReference(_), _])
        ),
        "交设置不清计数：前后两轮连续不确定，三段点名的那一条都按 CPU 判：{after_the_switch:?}"
    );
}

/// 第 1 条「从下一批开始生效；一批之内不换」，经核心层走：选 `gpu` 挂载那一刻核心层把挂载那一组交给实现；头一批读完设置、开算之前请求切换
/// （等的上限与显存上限都换了），头一批算的每一段都在切换那一组交过来之前，切换那一组在下一批开头、开算之前交过来，一趟里只交这两次。
#[test]
fn the_switch_requested_in_the_middle_of_a_batch_reaches_the_kernels_only_at_the_start_of_the_next_batch(
) {
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut gpu_pool = history.copied_with_an_empty_stream();
    drop(history);
    let (implementation, injected) = implementation_over_kernels_on_the_cpu();
    let mount_settings = settings_on_the_card_of_the_test_double(None, Duration::from_secs(10));
    let switched_settings = settings_on_the_card_of_the_test_double(
        Some(GpuMemoryInBytes(EIGHT_GIBIBYTES / 4)),
        Duration::from_secs(3),
    );
    let mut gpu_session = gpu_pool.remounted_with_compaction_forced_to_start(
        mount_settings,
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    let mut events_before_each_batch_computes: Vec<usize> = Vec::new();
    let mut switch_in_the_middle_of_the_first_batch =
        |background_compute: &mut BackgroundComputeOfThisMount| {
            events_before_each_batch_computes
                .push(injected.lock().expect("用例没在别处 panic 过").events.len());
            if events_before_each_batch_computes.len() == 1 {
                background_compute
                    .request_switch_from_the_next_batch(switched_settings.clone(), None)
                    .expect("全合法的一组：切换收下，记成待生效");
            }
        };
    let run = gpu_session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut gpu_pool.devices,
            &mut switch_in_the_middle_of_the_first_batch,
        )
        .expect("整理一趟");
    assert!(
        events_before_each_batch_computes.len() >= 2,
        "签发一次、至少搬一批：至少两批（{:?}）",
        run.outcome
    );
    let events = injected
        .lock()
        .expect("用例没在别处 panic 过")
        .events
        .clone();
    let received: Vec<&AdmittedGpuSettings> = events
        .iter()
        .filter_map(|event| match event {
            EventOfTheKernelsOnTheCpu::SettingsReceived(settings) => Some(settings),
            EventOfTheKernelsOnTheCpu::SegmentComputed => None,
        })
        .collect();
    assert_eq!(
        received,
        vec![
            &admitted_on_the_card_of_the_test_double(None, Duration::from_secs(10)),
            &admitted_on_the_card_of_the_test_double(
                Some(GpuMemoryInBytes(EIGHT_GIBIBYTES / 4)),
                Duration::from_secs(3),
            ),
        ],
        "只交两次：挂载那一组、切换那一组"
    );
    assert_eq!(
        events_before_each_batch_computes[0], 1,
        "挂载那一组在头一批开算之前就交了"
    );
    let first_batch_events =
        &events[events_before_each_batch_computes[0]..events_before_each_batch_computes[1] - 1];
    assert!(
        !first_batch_events.is_empty()
            && first_batch_events
                .iter()
                .all(|event| *event == EventOfTheKernelsOnTheCpu::SegmentComputed),
        "头一批算的每一段都在切换那一组交过来之前：{events:?}"
    );
    assert!(
        matches!(
            &events[events_before_each_batch_computes[1] - 1],
            EventOfTheKernelsOnTheCpu::SettingsReceived(_)
        ),
        "切换那一组在下一批开头、开算之前交过来：{events:?}"
    );
    assert!(
        run.background_compute_of_each_batch
            .iter()
            .all(|batch| matches!(batch.path_taken, BackgroundComputePathTaken::Gpu(_))),
        "每一批都走 GPU"
    );
}

/// 在间隔内重试三次，gpu 仍无法用就报警：丙段交回「这张卡重交 3 次都等超过上限」（两份核那一层是 CPU 替身），
/// 经产品那一份实现报给核心层是「这张卡用不了」那一个成员，带卡、重交了几次、等的上限与最后那一次是这一批的那一次提交；整理这一批放弃、
/// 一个写都没发、停着，`stopped_after_a_compute_error` 读得出它。
#[test]
fn the_gpu_unusable_after_its_resubmissions_stops_the_compaction_reporting_the_card_resubmissions_and_wait_limit(
) {
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let (implementation, injected) = implementation_over_kernels_on_the_cpu();
    injected
        .lock()
        .expect("用例没在别处 panic 过")
        .live_slot_batch_failure = Some(LiveSlotBatchFailed::GpuFailed {
        device: card_of_the_test_double(),
        cause: GpuFailureCause::UnusableAfterResubmissions {
            resubmissions: 3,
            last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            }),
        },
    });
    let mut session = pool.remounted_with_compaction_forced_to_start(
        settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    let run = session.run_compaction(&mut pool.devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && run.versions_published.is_empty(),
        "这一批放弃、一个写都没发、整理停下：{:?}",
        run.outcome
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: BackgroundComputeError::GpuSegmentFailed {
                segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
                failure: BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                    device: card_of_the_test_double(),
                    resubmissions: 3,
                    wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                    last_submission_waited_for:
                        SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                },
            },
            batches_compensated_on_the_cpu: 0,
        }],
        "停着那一态报的是「这张卡用不了」：卡、重交 3 次、等的上限、最后那一次是这一批的提交"
    );
}

/// 装不下不拒绝、连一片都装不下就不用这张卡：挂载选 `gpu`，两份核那一层答「参与的卡一张都不剩」（显存上限装不下最小的一片）：
/// 挂载照常做成，挂载那一刻整理就记成出错停着、读得出原因（「参与的卡一张都不剩」，带那张卡、上限与至少要多少），不等头一批；之后一趟整理停在先前出错停着上、
/// 一个写都不发，两份核那一层一段都没算。
#[test]
fn the_mount_whose_participating_gpus_are_all_left_out_under_their_memory_limit_starts_with_the_compaction_stopped(
) {
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let (implementation, injected) = implementation_over_kernels_on_the_cpu();
    let gpus_left_out = vec![GpuLeftOutUnderItsMemoryLimit {
        device: card_of_the_test_double(),
        memory_limit: GpuMemoryInBytes(64),
        smallest_piece_bytes: 976,
        bytes_already_held_on_the_device: 0,
    }];
    injected
        .lock()
        .expect("用例没在别处 panic 过")
        .batch_not_split_over_the_gpus = Some(
        BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out.clone()),
    );
    let mut session = pool.remounted_with_compaction_forced_to_start(
        settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                failure:
                    BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
                        gpus_left_out,
                    },
            },
            batches_compensated_on_the_cpu: 0,
        }],
        "挂载那一刻就停着、读得出原因：参与的卡一张都不剩"
    );
    let run = session.run_compaction(&mut pool.devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.versions_published.is_empty()
            && run.background_compute_of_each_batch.is_empty(),
        "停着而占用不超过阈值：一个写都不发、一批都不开：{:?}",
        run.outcome
    );
    assert!(
        injected
            .lock()
            .expect("用例没在别处 panic 过")
            .events
            .iter()
            .all(|event| matches!(event, EventOfTheKernelsOnTheCpu::SettingsReceived(_))),
        "两份核那一层只收到挂载那一组设置，一段都没算"
    );
}

/// 装不下不拒绝：挂着切到一组一张卡都用不上的设置（显存上限 64 字节，两份核那一层收到这一组之后答「参与的卡一张都不剩」），
/// 切换照收；切换生效的那一批开头整理当场停着，报的成员与挂载那一刻一张都用不上时相同
/// （`the_mount_whose_participating_gpus_are_all_left_out_under_their_memory_limit_starts_with_the_compaction_stopped`）。那一批不开、
/// 一趟停在先前出错停着上，切换之前那一批照常走 GPU；两份核那一层收到切换那一组之后一段都没算。
#[test]
fn the_switch_to_settings_whose_participating_gpus_are_all_left_out_stops_the_compaction_at_the_next_batch_as_at_the_mount(
) {
    let mut pool = pool_after_a_sequential_write_in_an_earlier_mount();
    let (implementation, injected) = implementation_over_kernels_on_the_cpu();
    let memory_limit_holding_no_piece = GpuMemoryInBytes(64);
    let gpus_left_out = vec![GpuLeftOutUnderItsMemoryLimit {
        device: card_of_the_test_double(),
        memory_limit: memory_limit_holding_no_piece,
        smallest_piece_bytes: 976,
        bytes_already_held_on_the_device: 0,
    }];
    let mut session = pool.remounted_with_compaction_forced_to_start(
        settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "挂载那一组用得上：挂载那一刻没停着"
    );
    let mut batches_begun = 0_usize;
    let mut switch_in_the_middle_of_the_first_batch =
        |background_compute: &mut BackgroundComputeOfThisMount| {
            batches_begun += 1;
            if batches_begun == 1 {
                background_compute
                    .request_switch_from_the_next_batch(
                        settings_on_the_card_of_the_test_double(
                            Some(memory_limit_holding_no_piece),
                            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                        ),
                        None,
                    )
                    .expect("上限不超过那张卡的显存总量：切换照收，不因为装不下就拒");
                injected
                    .lock()
                    .expect("用例没在别处 panic 过")
                    .batch_not_split_over_the_gpus_once_the_next_settings_arrive = Some(
                    BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out.clone()),
                );
            }
        };
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut pool.devices,
            &mut switch_in_the_middle_of_the_first_batch,
        )
        .expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.background_compute_of_each_batch.len() == 1
            && matches!(
                run.background_compute_of_each_batch[0].path_taken,
                BackgroundComputePathTaken::Gpu(_)
            ),
        "切换之前那一批走 GPU；切换生效的那一批开头停着、不开：{:?}，{:?}",
        run.outcome,
        run.background_compute_of_each_batch
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                failure:
                    BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
                        gpus_left_out,
                    },
            },
            batches_compensated_on_the_cpu: 0,
        }],
        "切换生效的那一批开头停着，成员与挂载那一刻一张都用不上时相同"
    );
    let events = injected
        .lock()
        .expect("用例没在别处 panic 过")
        .events
        .clone();
    let switch_received_at = events
        .iter()
        .rposition(|event| matches!(event, EventOfTheKernelsOnTheCpu::SettingsReceived(_)))
        .expect("收到过设置");
    assert!(
        switch_received_at > 0
            && events[switch_received_at + 1..].is_empty()
            && events[1..switch_received_at].contains(&EventOfTheKernelsOnTheCpu::SegmentComputed),
        "切换之前那一批算过段；切换那一组交过来之后一段都没算：{events:?}"
    );
}

/// 一批三段在卡上是怎么交的（交了几片、重交了几次、显存不足重切了几次、用了哪几张卡）随那一批的记账报出来——
/// 核心层每一批算完之后经实现取一次（`BackgroundGpuCompute::take_the_submissions_of_the_batch`，产品那一份转给两份核那一层），记进
/// `GpuWorkOfTheBatch::submissions_on_the_gpus`。替身每次都交回同一份，选 `gpu` 推一趟整理：每一批的记账里都是它，逐项相同。
#[test]
fn the_submissions_on_the_gpus_of_each_batch_are_recorded_in_the_accounting_of_the_batch() {
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut gpu_pool = history.copied_with_an_empty_stream();
    drop(history);
    let (implementation, injected) = implementation_over_kernels_on_the_cpu();
    let submissions_reported = GpuSubmissionsOfTheBatch {
        unit_checksum_verification: GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 5,
            resubmissions_after_timeouts: 1,
            recuts_after_running_out_of_memory: 1,
            devices_given_work: vec![card_of_the_test_double()],
        },
        region_record_classification: GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 3,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_of_the_test_double()],
        },
        live_slots_per_cluster_segment: GpuSubmissionsOfOneSegment::default(),
    };
    injected
        .lock()
        .expect("用例没在别处 panic 过")
        .submissions_reported_after_each_batch = submissions_reported.clone();
    let mut gpu_session = gpu_pool.remounted_with_compaction_forced_to_start(
        settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
        Some(BackgroundGpuComputeHandle::of_an_implementation(
            implementation,
        )),
    );
    let gpu_run = gpu_session
        .run_compaction(&mut gpu_pool.devices)
        .expect("整理一趟");
    assert!(
        !gpu_run.background_compute_of_each_batch.is_empty(),
        "这一趟至少算了一批：{:?}",
        gpu_run.outcome
    );
    for batch in &gpu_run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Gpu(gpu_work) = &batch.path_taken else {
            panic!("选 gpu 的一批走 GPU：{batch:?}");
        };
        assert_eq!(
            gpu_work.submissions_on_the_gpus, submissions_reported,
            "每一批的记账带着实现交回的那一份"
        );
    }
}
