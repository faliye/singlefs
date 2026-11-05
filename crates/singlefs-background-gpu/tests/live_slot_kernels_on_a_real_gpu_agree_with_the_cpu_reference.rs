//! 选段打分里每段活槽数那两份 WGSL 核在真显卡上与 CPU 参照（核心层的 `compaction_plan::live_slots_per_cluster_segment_by_the_records`）对拍、
//! 两份核彼此对拍（D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 只开服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）：服务没交这一台的卡时每条打一行
//! `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；交了卡却对不上本机的独显、或开不了判红。源头没声明张数，服务分一张。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
//! 开卡时每张参与的卡的显存上限给 1 GiB（七·3 验收，用户 2026-11-01 定；`common_discrete_gpus_of_this_host`），专门测小上限切片的几条照用更小的。
//!
//! 两块单元区：默认环长下的那一块（6 个整段加末尾 10 个槽），与起点在槽 2^32 − 128 的那一块（第 3 个整段从槽 2^32 起，槽号的高 32 位在段中间进位）。
//! 每块上的记录：起点在单元区之前、罩进头一个段的；段里的普通记录；释放了的；别的盘上的；跨两段的；同一个落点两条（各算一次）；
//! 跨好几个段的长记录；落在末尾凑不满一段那一截的；跨过槽 2^32 的；另有一串别的盘上的记录，只为让记录表大过参与的卡至少要装得下的那一片
//! （区域扫描那一段切开的单元头一片，972 字节）：给小上限、要切段或切记录表的用例，上限不比那一片小、卡照样参与。
//!
//! 用例红了（panic）展开时照常丢后台 GPU：一批等超过上限、重交 3 次都没做完时那次提交可能还在卡上跑，产品的 `BackgroundGpu` 丢的时候不等它、
//! 泄漏那张卡（wgpu-core 30.0.1 `Queue::drop` 调 `vkQueueWaitIdle`，不带上限），用例在等 GPU 的上限（连同重交）之内报出红。
mod common_discrete_gpus_of_this_host;

use common_discrete_gpus_of_this_host::{
    background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run,
    handed_over_gpus_or_report_not_run, opened_background_gpu, selection_over_the_handed_over_gpus,
};
use singlefs_background_gpu::background_gpu::{BackgroundGpu, LiveSlotBatchFailed};
use singlefs_background_gpu::gpus_of_this_host::bytes_of_the_smallest_piece_every_participating_gpu_holds;
use singlefs_background_gpu::live_slot_kernels::{
    cpu_reference_live_slots, live_slot_chunk_footprint, AllocationRecordTablesForTheKernels,
    SEGMENT_BOUND_BYTES,
};
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark, UnitAreaStart};
use singlefs_core::background_compute::GpuMemoryInBytes;
use singlefs_core::compaction_plan::{
    cluster_segment_entry_keys_of_the_unit_area, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::system_configuration::JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION;

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);
const UNIT_AREA_DEVICE: DeviceIdentity = DeviceIdentity(0);
const OTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);
/// 别的盘上垫进记录表的记录条数：连同单元区那 13 条共 50 条、记录表 1200 字节，比参与的卡至少要装得下的那一片（972 字节）大，
/// 小上限的用例才切得到段与记录表。它们不在单元区那块盘上，不改任何一段的活槽数。
const RECORDS_ON_THE_OTHER_DEVICE_TO_FILL_THE_TABLE: u64 = 37;
/// 每块单元区的整段数。
const FULL_SEGMENTS_PER_UNIT_AREA: u64 = 6;
/// 末尾凑不满一段的那一截有几个槽。
const TRAILING_SLOTS_PAST_THE_FULL_SEGMENTS: u64 = 10;

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
    .into_iter()
    .chain(
        (0..RECORDS_ON_THE_OTHER_DEVICE_TO_FILL_THE_TABLE).map(|position_on_the_other_device| {
            record(
                OTHER_DEVICE,
                start + 2 * width + 2 * position_on_the_other_device,
                2,
                false,
            )
        }),
    )
    .collect()
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
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
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
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_kernel_searching_the_sorted_records_matches_the_cpu_reference_on_both_unit_areas",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
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
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_two_live_slot_kernels_confirm_every_segment_on_a_healthy_gpu",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
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

/// 第 3 条「一批放不下就切成几次交」：开卡就给每张卡「装得下两个段、装不下这块单元区的全部段」的显存上限（由这一批的记录表现算：
/// 整份记录表与参数每段各占一份，每个段另占上下界与两份核的结果、回读暂存），两份核的数照样逐段与 CPU 参照相同。这个上限比量代价那一段小，
/// 开卡时量代价那一段照它切成几片量（用户 2026-10-28 定「装不下应该自适应的分片」），不拒挂载。
#[test]
fn the_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots()
{
    let test_name =
        "the_memory_limit_smaller_than_the_batch_cuts_the_segments_into_chunks_with_the_same_live_slots";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    for unit_area in the_two_unit_areas() {
        let footprint = live_slot_chunk_footprint(
            &AllocationRecordTablesForTheKernels::of_the_allocation_records(&records_on(unit_area)),
        );
        let segment_count =
            u64::try_from(cluster_segment_entry_keys_of_the_unit_area(unit_area).len())
                .expect("段数装得进 u64");
        let two_segments = footprint.bytes_held_by_a_chunk(2 * SEGMENT_BOUND_BYTES, 2);
        let every_segment =
            footprint.bytes_held_by_a_chunk(segment_count * SEGMENT_BOUND_BYTES, segment_count);
        assert!(
            segment_count > 2
                && two_segments < every_segment
                && two_segments >= bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            "上限 {two_segments} 字节装得下两个段、装不下全部 {segment_count} 个段（{every_segment} 字节）"
        );
        // 上限比量代价那一段小：切开量，不拒挂载
        let mut background_gpu = opened_background_gpu(&selection_over_the_handed_over_gpus(
            &handed_over_gpus,
            GpuMemoryInBytes(two_segments),
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        ));
        let (both, reference) =
            both_kernels_and_the_cpu_reference_on(&mut background_gpu, unit_area);
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

/// 用户 2026-10-28 定「装不下应该自适应的分片」：上限取参与的卡至少要装得下的那一片（卡照样参与），连整份记录表加一个段都装不下，记录表按记录切成几片、
/// 各片数出的逐段相加，两份核的数照样逐段与 CPU 参照相同。
#[test]
fn the_memory_limit_smaller_than_the_record_table_cuts_it_into_slices_with_the_same_live_slots() {
    let test_name =
        "the_memory_limit_smaller_than_the_record_table_cuts_it_into_slices_with_the_same_live_slots";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    let memory_limit_holding_the_smallest_piece =
        bytes_of_the_smallest_piece_every_participating_gpu_holds();
    for unit_area in the_two_unit_areas() {
        let whole_table_and_one_segment = live_slot_chunk_footprint(
            &AllocationRecordTablesForTheKernels::of_the_allocation_records(&records_on(unit_area)),
        )
        .bytes_held_by_a_chunk(SEGMENT_BOUND_BYTES, 1);
        assert!(
            memory_limit_holding_the_smallest_piece < whole_table_and_one_segment,
            "整份记录表加一个段（{whole_table_and_one_segment} 字节）装不下：要切"
        );
        // 上限装得下最小的一片：不拒挂载
        let mut background_gpu = opened_background_gpu(&selection_over_the_handed_over_gpus(
            &handed_over_gpus,
            GpuMemoryInBytes(memory_limit_holding_the_smallest_piece),
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        ));
        let (both, reference) =
            both_kernels_and_the_cpu_reference_on(&mut background_gpu, unit_area);
        assert_eq!(
            both,
            reference
                .iter()
                .map(|live_slots| (*live_slots, *live_slots))
                .collect::<Vec<_>>(),
            "记录表切成几片、各片相加，两份核的数照样逐段与 CPU 参照相同：单元区起点 {:?}",
            unit_area.start
        );
    }
}

/// 用户 2026-10-28 定「连一片都装不下就不用这张卡」：每张卡的显存上限都比最小的一片少一个字节——开卡不拒挂载、不 panic，
/// 参与的卡一张都不剩、每张都报出来（卡、上限、至少要多少），交一批报「连最小的一片都装不下」（核心层当 GPU 用不上那一态）。
#[test]
fn the_memory_limit_below_the_smallest_piece_leaves_every_gpu_out_without_refusing_the_opening() {
    let test_name =
        "the_memory_limit_below_the_smallest_piece_leaves_every_gpu_out_without_refusing_the_opening";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
    // 上限装不下最小的一片：不用这些卡，不拒挂载
    let mut background_gpu = opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        GpuMemoryInBytes(smallest_piece_bytes - 1),
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ));
    assert!(
        background_gpu.participating_gpus().is_empty(),
        "参与的卡一张都不剩"
    );
    assert_eq!(
        background_gpu
            .gpus_left_out_under_their_memory_limit()
            .len(),
        handed_over_gpus.len(),
        "交来的每张都报出来"
    );
    assert!(background_gpu
        .gpus_left_out_under_their_memory_limit()
        .iter()
        .all(
            |left_out| left_out.memory_limit == GpuMemoryInBytes(smallest_piece_bytes - 1)
                && left_out.smallest_piece_bytes == smallest_piece_bytes
        ));
    let [unit_area, _high_unit_area] = the_two_unit_areas();
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    assert!(
        matches!(
            background_gpu.both_kernel_live_slots_of_the_cluster_segments(
                &records_on(unit_area),
                unit_area,
                &segments
            ),
            Err(LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(left_out))
                if left_out.as_slice() == background_gpu.gpus_left_out_under_their_memory_limit()
        ),
        "交一批：报参与的卡一张都不剩，带每张被晾下的卡"
    );
}
