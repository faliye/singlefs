//! 端到端（D26（后台整理与放置回收） 已定项 13 第 5 条，里程碑四七·4 经运行库那一格）：同一历史的两个池各经运行库（`singlefs_mount_runtime`）挂上，
//! 一个选 `cpu`、一个选 `gpu`（开卡器 `gpu_opener_of_this_process`，开卡在挂载之前），钟由用例摆、一拍一拍推（真线程下 GC 与前台的次序取决于调度，块序列只在拍子下比得上）：
//! 写出的块序列与卸载之后的盘逐字节相同，整理真发布过。开卡与挂载只点名服务交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）；
//! 服务没交卡时打 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功。选 `gpu` 那一边推完拍子、卸载之前核这个进程只在交来的卡上有设备
//! （D26（后台整理与放置回收） 已定项 13 第 2 条「挂载时只开参与的卡」，读法在 `common_discrete_gpus_of_this_host`），上了别的卡判红、读不成记本次未跑。只在开了特性 `tests-on-a-real-gpu` 时编。
//! 对拍分两段、经判定库交接（`common_history_pool_through_the_library`）：只录那一段把起步的那份历史写进库、选 `cpu` 的那一趟在 CPU 上跑完、
//! 它留下的三样也写进库，到此返回、不开卡；判段的对拍结论按键入库复用（读回输入的摘要、参照一侧与这条用例 GPU 一侧的判法单元键），
//! 库里有两边一致的不开卡；要现判的从库里读回历史与那三样，开卡只跑选 `gpu` 的那一趟、逐样比。持卡判入口是
//! [`judged_on_the_cards_that_the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu`]（登记在 `src/gpu_side_units_entries.rs`）；
//! 交来的卡里没有这一台的、或核不成这个进程在哪几张卡上的那一趟交回没跑成，不入库。
mod common_discrete_gpus_of_this_host;
mod common_history_pool_through_the_library;
mod common_pairing_input_through_the_library;
mod common_pool_on_memory_devices;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common_discrete_gpus_of_this_host::{
    assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run,
    handed_over_gpus_or_report_not_run, identifiers_of_the_handed_over_gpus,
    memory_limits_of_one_gibibyte_each, GpusThisProcessIsOn, HandedOverGpusSeenThroughTheInstance,
};
use common_history_pool_through_the_library::{
    assert_the_pieces_equal_the_reference,
    history_pool_concluded_on_the_cards_through_the_pairing_library,
    history_pool_of_the_input_read_back, reference_bytes_of_the_pieces,
};
use common_pairing_input_through_the_library::{
    judgement_of_the_assertions_on_the_cards, pairing_input_bytes_of_the_written_sectors,
    written_sectors_of_the_pairing_input_bytes, ComparisonOnTheCards,
};
use common_pool_on_memory_devices::{
    parameters, PoolOnMemoryDevices, RecordedDeviceOperation, RecordedMemoryDevice,
};
use singlefs_background_gpu::gpu_opener_of_this_process::GpuOpenerOfThisProcess;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::background_compute::{BackgroundComputeBackend, BackgroundComputeSettings};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CompactionMountParameters, CompactionSwitches,
    CompactionWatermarks, EmptyClusterSegmentsWatermark, FragmentationWatermark,
    UnreclaimableBytesWatermark, COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MountedRuntime, OpensTheBackgroundGpu,
    RuntimeMounted, RuntimeParameters,
};
use singlefs_verification::crash_library::PairingJudged;

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: Duration = Duration::from_secs(60);
/// 一拍之间钟走多久。
const ONE_PERIOD: Duration = Duration::from_secs(30);
/// 推几拍。
const MAINTENANCE_PASSES: usize = 6;

#[derive(Debug, Default)]
struct ClockMovedByTheTest {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheTest {
    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheTest {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

/// 一趟跑完留下的：录制流与卸载之后两块盘写过的扇区，整理发了几次。
struct WhatTheRuntimeWrote {
    operations: Vec<RecordedDeviceOperation>,
    sectors: Vec<(DeviceIdentity, BTreeMap<u64, Vec<u8>>)>,
    compaction_publishes: u64,
}

/// 挂载（`background_compute` 那一组设置，水位线 100% 让它越线）、强制选数据单元最多的那一段（只顺序写的池上段差不多都满，选段判据归另一条用例）、
/// 一拍一拍推、调 `before_the_unmount`（还挂着、开着的卡都在）、卸载。
fn what_the_runtime_writes(
    pool: PoolOnMemoryDevices,
    background_compute: BackgroundComputeSettings,
    gpu_opener: Option<&dyn OpensTheBackgroundGpu>,
    before_the_unmount: &dyn Fn(),
) -> WhatTheRuntimeWrote {
    let stream = Arc::clone(&pool.stream);
    let clock = Arc::new(ClockMovedByTheTest::default());
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute = background_compute;
    runtime_parameters.compaction = CompactionMountParameters {
        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
        watermarks: CompactionWatermarks {
            empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
                percent: 100,
                lowest_line_in_segments: 2,
            },
            fragmentation: FragmentationWatermark::Off,
            unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
        },
    };
    let RuntimeMounted { runtime, .. } =
        MountedRuntime::<RecordedMemoryDevice>::mount_without_the_maintenance_thread(
            &parameters(),
            pool.devices,
            runtime_parameters,
            gpu_opener,
            Arc::clone(&clock) as Arc<dyn ClockOfTheRuntime>,
            Arc::new(IgnoringTheMaintenanceEvents),
        )
        .expect("服务交来的卡、全合法的设置：挂载照常做成");
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let version = session.current.version_of_trees().expect("有树的一版");
            let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
            for record in version.allocation_records.iter().filter(|record| {
                record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
            }) {
                *data_units_by_segment
                    .entry(record.slot.0 / 64 * 64)
                    .or_default() += 1;
            }
            let (fullest_segment, _data_units) = data_units_by_segment
                .into_iter()
                .max_by_key(|(_segment, data_units)| *data_units)
                .expect("写了数据单元");
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = Some(
                CompactionRegion::of_one_cluster_segment(
                    DeviceIdentity(0),
                    SlotNumber(fullest_segment),
                )
                .expect("按 64 取整的段起点"),
            );
            session.allocator.set_compaction_switches(switches);
        })
        .expect("运行库没坏");
    for _pass in 0..MAINTENANCE_PASSES {
        clock.advance_by(ONE_PERIOD);
        runtime.run_one_maintenance_pass().expect("运行库没坏");
    }
    let compaction_publishes = runtime.observations().compaction_publishes;
    before_the_unmount();
    let unmounted = runtime.unmount().expect("卸载干净");
    let pool_after_the_unmount = PoolOnMemoryDevices {
        devices: unmounted.devices,
        stream,
    };
    WhatTheRuntimeWrote {
        operations: pool_after_the_unmount.recorded_operations(),
        sectors: pool_after_the_unmount.written_sectors_of_every_device(),
        compaction_publishes,
    }
}

const NAME_OF_THE_TEST: &str =
    "the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu";

const PIECE_COMPACTION_PUBLISHES: &str = "整理发布了几次";
const PIECE_OPERATIONS: &str = "写出的块序列";
const PIECE_SECTORS: &str = "卸载之后盘上的扇区";

/// 一趟跑完留下的三样各编成字节：发布次数与块序列是 `{:?}` 的文字，末一样是每块盘写过的扇区。
fn pieces_of_what_the_runtime_wrote(written: &WhatTheRuntimeWrote) -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            PIECE_COMPACTION_PUBLISHES,
            format!("{:?}", written.compaction_publishes).into_bytes(),
        ),
        (
            PIECE_OPERATIONS,
            format!("{:?}", written.operations).into_bytes(),
        ),
        (
            PIECE_SECTORS,
            pairing_input_bytes_of_the_written_sectors(&written.sectors),
        ),
    ]
}

/// 对拍参照一侧的入口（登记在 `src/reference_units_entries.rs`）：照对拍输入里的扇区搭回那份历史，选 `cpu` 经运行库挂上、推完拍子、卸载，
/// 留下的三样编成字节。只录那一段调它。
fn reference_bytes_of_the_runtime_on_the_cpu(input_bytes: &[u8]) -> Vec<u8> {
    let on_the_cpu = what_the_runtime_writes(
        PoolOnMemoryDevices::of_the_written_sectors(written_sectors_of_the_pairing_input_bytes(
            input_bytes,
        )),
        BackgroundComputeSettings::PRODUCT,
        None,
        &|| {},
    );
    assert!(on_the_cpu.compaction_publishes > 0, "整理真发布过");
    reference_bytes_of_the_pieces(&pieces_of_what_the_runtime_wrote(&on_the_cpu))
}

#[test]
fn the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu() {
    history_pool_concluded_on_the_cards_through_the_pairing_library(
        NAME_OF_THE_TEST,
        "cpu_path_of_the_mount_runtime",
        reference_bytes_of_the_runtime_on_the_cpu,
        judged_on_the_cards_that_the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：照读回的扇区搭池，开交来的卡、选 `gpu` 经运行库挂上、推完拍子，卸载之前核这个进程只在交来的卡上，
/// 卸载之后留下的三样与读回的选 `cpu` 那一趟逐样比。交来的卡里没有这一台的、或核不成这个进程在哪几张卡上，交回没跑成（不入库，下一趟照判）。
fn judged_on_the_cards_that_the_runtime_on_the_real_gpus_writes_the_same_block_sequence_as_on_the_cpu(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let gpu_pool = history_pool_of_the_input_read_back(input_bytes);
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let opener = GpuOpenerOfThisProcess::over_the_vulkan_instance(vulkan_instance);
        let gpu_settings = BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(identifiers_of_the_handed_over_gpus(&handed_over_gpus)),
            background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(
                handed_over_gpus.len(),
            )),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        };
        let gpus_this_process_is_on: Cell<Option<GpusThisProcessIsOn>> = Cell::new(None);
        let on_the_gpus = what_the_runtime_writes(gpu_pool, gpu_settings, Some(&opener), &|| {
            gpus_this_process_is_on.set(Some(
                assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run(
                    test_name,
                    &handed_over_gpus,
                ),
            ));
        });
        match gpus_this_process_is_on.get() {
            Some(GpusThisProcessIsOn::OnlyTheHandedOver) => {}
            Some(GpusThisProcessIsOn::NotReadSoThisRunIsNotRun) => {
                return ComparisonOnTheCards::NotRunOnTheseCards;
            }
            None => panic!("卸载之前没核这个进程在哪几张卡上"),
        }
        assert!(on_the_gpus.compaction_publishes > 0, "整理真发布过");
        // 选 cpu 的那一趟在只录那一段跑过、留下的三样在库里：发布次数、块序列、卸载之后的盘逐样相同
        assert_the_pieces_equal_the_reference(
            &pieces_of_what_the_runtime_wrote(&on_the_gpus),
            reference_bytes,
            &[PIECE_COMPACTION_PUBLISHES, PIECE_OPERATIONS, PIECE_SECTORS],
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}
