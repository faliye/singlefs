//! 选段打分里每段活槽数那两份 WGSL 核在真显卡上与 CPU 参照（核心层的 `compaction_plan::live_slots_per_cluster_segment_by_the_records`）对拍、
//! 两份核彼此对拍（D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 要本机至少一张独立显卡：一张都没有时每条打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；
//! 有卡而开不了判红。只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `scripts/crash/gpu-quiesce-run.sh` 起，带 `--test-threads 1`。
//!
//! 两块单元区：默认环长下的那一块（6 个整段加末尾 10 个槽），与起点在槽 2^32 − 128 的那一块（第 3 个整段从槽 2^32 起，槽号的高 32 位在段中间进位）。
//! 每块上的记录：起点在单元区之前、罩进头一个段的；段里的普通记录；释放了的；别的盘上的；跨两段的；同一个落点两条（各算一次）；
//! 跨好几个段的长记录；落在末尾凑不满一段那一截的；跨过槽 2^32 的。

use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused,
};
use singlefs_background_gpu::live_slot_kernels::cpu_reference_live_slots;
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark, UnitAreaStart};
use singlefs_core::background_compute::{BackgroundComputeSettingsRefused, GpuMemoryInBytes};
use singlefs_core::compaction_plan::{
    cluster_segment_entry_keys_of_the_unit_area, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::system_configuration::JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION;

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);
const UNIT_AREA_DEVICE: DeviceIdentity = DeviceIdentity(0);
const OTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);
/// 每块单元区的整段数。
const FULL_SEGMENTS_PER_UNIT_AREA: u64 = 6;
/// 末尾凑不满一段的那一截有几个槽。
const TRAILING_SLOTS_PAST_THE_FULL_SEGMENTS: u64 = 10;

/// 开本机全部独显；一张都没有时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
fn background_gpu_or_report_not_run(
    test_name: &str,
    selection: &BackgroundGpuSelection,
) -> Option<BackgroundGpu> {
    match open_background_gpu(selection) {
        Ok(background_gpu) => Some(background_gpu),
        Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuNotPresent,
        )) => {
            println!(
                "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=本机的 Vulkan 适配器里一张独立显卡都没有"
            );
            None
        }
        Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
    }
}

fn every_discrete_gpu_without_a_memory_limit() -> BackgroundGpuSelection {
    BackgroundGpuSelection {
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

fn segment_slots() -> u64 {
    CompactionRegion::of_one_cluster_segment(
        UNIT_AREA_DEVICE,
        UnitAreaStart::of_the_default_journal_ring().slot(),
    )
    .expect("单元区起点落在段边界上")
    .slot_count()
}

/// 起点在 `start_slot` 的单元区：journal 环照第一版的环起点、长到这个槽为止（单元区起点就是环末尾的下一个槽）。
fn unit_area_starting_at(start_slot: u64) -> UnitAreaOfOneDevice {
    let slot_bytes = SlotNumber(1).to_device_offset().0;
    let journal_ring_bytes =
        (start_slot - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION.0) * slot_bytes;
    let start = UnitAreaStart::following_the_journal_ring(journal_ring_bytes)
        .expect("起点是段宽的整数倍：落在聚簇段边界上");
    assert_eq!(
        start.slot(),
        SlotNumber(start_slot),
        "环末尾的下一个槽就是这一块的起点"
    );
    UnitAreaOfOneDevice {
        device: UNIT_AREA_DEVICE,
        start,
        slots: FULL_SEGMENTS_PER_UNIT_AREA * segment_slots()
            + TRAILING_SLOTS_PAST_THE_FULL_SEGMENTS,
    }
}

fn the_two_unit_areas() -> [UnitAreaOfOneDevice; 2] {
    [
        unit_area_starting_at(UnitAreaStart::of_the_default_journal_ring().slot().0),
        unit_area_starting_at((1_u64 << 32) - 2 * segment_slots()),
    ]
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
        generation: CheckpointTxg(5),
        is_released,
        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
    }
}

/// 一块单元区上的记录（见文件头），按与槽号无关的次序交（两份核一份按表的次序走、一份按排好的序号查）。
fn records_on(unit_area: UnitAreaOfOneDevice) -> Vec<AllocationRecord> {
    let start = unit_area.start.slot().0;
    let width = segment_slots();
    vec![
        record(UNIT_AREA_DEVICE, start + 3 * width + 10, 2, false),
        record(UNIT_AREA_DEVICE, start - 3, 5, false),
        record(UNIT_AREA_DEVICE, start, 2, false),
        record(UNIT_AREA_DEVICE, start + 5, 1, false),
        record(UNIT_AREA_DEVICE, start + 7, 2, true),
        record(OTHER_DEVICE, start + 9, 2, false),
        record(UNIT_AREA_DEVICE, start + width - 1, 2, false),
        record(UNIT_AREA_DEVICE, start + width + 6, 2, false),
        record(UNIT_AREA_DEVICE, start + width + 6, 2, false),
        record(UNIT_AREA_DEVICE, start + width + 36, 200, false),
        record(UNIT_AREA_DEVICE, (1_u64 << 32) - 1, 2, false),
        record(
            UNIT_AREA_DEVICE,
            start + FULL_SEGMENTS_PER_UNIT_AREA * width + 2,
            3,
            false,
        ),
        record(UNIT_AREA_DEVICE, start + 5 * width + 60, 2, true),
    ]
}

/// 两份核在一块单元区上各数一遍，交回每段两份的数与 CPU 参照的数。
fn both_kernels_and_the_cpu_reference_on(
    background_gpu: &mut BackgroundGpu,
    unit_area: UnitAreaOfOneDevice,
) -> (Vec<(u64, u64)>, Vec<u64>) {
    let records = records_on(unit_area);
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    let both = background_gpu
        .both_kernel_live_slots_of_the_cluster_segments(&records, unit_area, &segments)
        .expect("健康的卡上一批交得成");
    let reference = cpu_reference_live_slots(&records, unit_area);
    assert!(
        reference
            .iter()
            .filter(|live_slots| **live_slots > 0)
            .count()
            >= 4,
        "参照里活槽不为 0 的段够多（不然对拍什么都没比）：{reference:?}"
    );
    (both, reference)
}

#[test]
fn the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    for unit_area in the_two_unit_areas() {
        let (both, reference) =
            both_kernels_and_the_cpu_reference_on(&mut background_gpu, unit_area);
        let walking: Vec<u64> = both.iter().map(|(walking, _searching)| *walking).collect();
        assert_eq!(
            walking, reference,
            "逐条走遍记录那一份核与 CPU 参照逐段相同：单元区起点 {:?}",
            unit_area.start
        );
    }
}

#[test]
fn the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    for unit_area in the_two_unit_areas() {
        let (both, reference) =
            both_kernels_and_the_cpu_reference_on(&mut background_gpu, unit_area);
        let searching: Vec<u64> = both
            .iter()
            .map(|(_walking, searching)| *searching)
            .collect();
        assert_eq!(
            searching, reference,
            "二分查排好的序号那一份核与 CPU 参照逐段相同：单元区起点 {:?}",
            unit_area.start
        );
    }
}

#[test]
fn the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let [unit_area, _high_unit_area] = the_two_unit_areas();
    let records = records_on(unit_area);
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
    let reconciled = background_gpu
        .count_live_slots_per_cluster_segment(&records, unit_area, &segments, &mut uncertainty)
        .expect("健康的卡上一批交得成");
    assert_eq!(
        (
            reconciled.confirmed_entries,
            reconciled.uncertain_entries,
            reconciled.entries_judged_by_the_cpu_reference
        ),
        (segments.len(), 0, 0),
        "两份核彼此逐段相同：全部确认"
    );
    let confirmed: Vec<u64> = reconciled
        .decisions
        .iter()
        .map(|decision| match decision {
            EntryDecision::ConfirmedByBothKernels(live_slots) => *live_slots,
            EntryDecision::UncertainThisRound | EntryDecision::JudgedByTheCpuReference(_) => {
                panic!("健康的卡上不该有不确定的段：{decision:?}")
            }
        })
        .collect();
    assert_eq!(confirmed, cpu_reference_live_slots(&records, unit_area));
}

#[test]
fn a_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots() {
    let test_name =
        "a_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots";
    let Some(unlimited) =
        background_gpu_or_report_not_run(test_name, &every_discrete_gpu_without_a_memory_limit())
    else {
        return;
    };
    let participating_gpus = unlimited.participating_gpus().len();
    drop(unlimited);
    // 整份记录表（13 条，每条 24 字节）连同参数约 330 字节；每段另占 16 + 4 × 12 = 64 字节：500 字节装得下两段、装不下六段
    let small_memory_limit = GpuMemoryInBytes(500);
    let mut limited = background_gpu_or_report_not_run(
        test_name,
        &BackgroundGpuSelection {
            background_gpu_devices: None,
            background_gpu_memory_limit: Some(vec![small_memory_limit; participating_gpus]),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        },
    )
    .expect("同一台机器上刚开过一次");
    for unit_area in the_two_unit_areas() {
        let (both, reference) = both_kernels_and_the_cpu_reference_on(&mut limited, unit_area);
        assert_eq!(
            both,
            reference
                .iter()
                .map(|live_slots| (*live_slots, *live_slots))
                .collect::<Vec<_>>(),
            "切成几段交，两份核的数照样逐段与 CPU 参照相同：单元区起点 {:?}",
            unit_area.start
        );
    }
}
