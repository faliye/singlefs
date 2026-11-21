//! 真线程、真卡、前台并发（D26（后台整理与放置回收） 已定项 13 第 5 条）：产品的开卡器构造（`GpuOpenerOfThisProcess::of_this_process`）开服务交来的卡，运行库起维护线程，
//! 整理的批在 GPU 上算（GPU 在锁内等一批），前台同时一次次收改动、fsync：前台都做成，整理发布过、走的是 GPU 那一条路，一个报警都没有，卸载干净、再挂得上。
//! 卸载之前核这个进程只在交来的卡上有设备（D26（后台整理与放置回收） 已定项 13 第 2 条「挂载时只开参与的卡」，读法在 `common_discrete_gpus_of_this_host`），上了别的卡判红。
//! 这一条不入对拍结论：没有 CPU 一侧可比（前台与整理在真线程上交错，交错次序取决于调度），只在真卡上跑通，每一趟判段都开卡现跑。
//! 拍子推的那一条（块序列逐字节比）在 `mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs`。服务没交卡时打 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功。只在开了特性 `tests-on-a-real-gpu` 时编。
#![cfg(feature = "tests-on-a-real-gpu")]

mod common_discrete_gpus_of_this_host;
mod common_history_pool_through_the_library;
mod common_pairing_input_through_the_library;
mod common_pool_on_memory_devices;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common_discrete_gpus_of_this_host::{
    assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run,
    handed_over_gpus_or_report_not_run, identifiers_of_the_handed_over_gpus,
    memory_limits_of_one_gibibyte_each, GpusThisProcessIsOn, HandedOverGpusSeenThroughTheInstance,
};
use common_history_pool_through_the_library::history_pool_read_back_through_the_pairing_library;
use common_pool_on_memory_devices::{parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_background_gpu::gpu_opener_of_this_process::GpuOpenerOfThisProcess;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::background_compute::{
    BackgroundComputeBackend, BackgroundComputePathTaken, BackgroundComputeSettings,
};
use singlefs_core::compaction_plan::{
    CompactionMountParameters, CompactionSwitches, CompactionWatermarks,
    EmptyClusterSegmentsWatermark, FragmentationWatermark, UnreclaimableBytesWatermark,
    COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{BackgroundCompactionCall, UserChange};
use singlefs_mount_runtime::{
    MaintenanceEvent, MonotonicClockOfThisProcess, MountedRuntime, ReceiverOfTheMaintenanceEvents,
    RuntimeMounted, RuntimeParameters,
};

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: Duration = Duration::from_secs(60);
/// 等线程做成这么多事的上限：只防挂住，这个长度只决定红得多快。
const THREAD_DEADLINE: Duration = Duration::from_secs(120);
/// 整理在 GPU 上算过这么多批才收：前台与批真交错过。
const GPU_BATCHES_TO_SEE: u64 = 2;
/// 整理的周期：压短，量的是锁上的交错。
const SHORT_COMPACTION_PERIOD: Duration = Duration::from_millis(20);
const SHORT_PUBLISH_POLL_PERIOD: Duration = Duration::from_millis(5);

/// 数维护线程上在 GPU 上算过的批。
#[derive(Debug, Default)]
struct CountingTheBatchesOnTheGpu {
    batches_on_the_gpu: AtomicU64,
}

impl ReceiverOfTheMaintenanceEvents for CountingTheBatchesOnTheGpu {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        if let MaintenanceEvent::CompactionCallReturned(BackgroundCompactionCall::Ran {
            run, ..
        }) = event
        {
            for batch in &run.background_compute_of_each_batch {
                match batch.path_taken {
                    BackgroundComputePathTaken::Gpu(_) => {
                        self.batches_on_the_gpu.fetch_add(1, Ordering::SeqCst);
                    }
                    BackgroundComputePathTaken::Cpu(_) => {}
                }
            }
        }
    }
}

const NAME_OF_THE_TEST: &str =
    "the_maintenance_thread_compacts_on_the_real_gpus_while_the_foreground_fsyncs";

// 不切片并行：一个池、一条维护线程、一个前台，被测的就是 GPU 批与前台在一把锁上的交错，没有可切的区间。
#[test]
fn the_maintenance_thread_compacts_on_the_real_gpus_while_the_foreground_fsyncs() {
    // 分两段：那份历史先过判定库，只录那一趟到此返回、不开卡；判段从库里读回。
    let Some(pool) = history_pool_read_back_through_the_pairing_library(NAME_OF_THE_TEST) else {
        return;
    };
    let Some(HandedOverGpusSeenThroughTheInstance {
        vulkan_instance,
        gpus: handed_over_gpus,
    }) = handed_over_gpus_or_report_not_run(NAME_OF_THE_TEST, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    // 对卡用的实例不交给开卡器：这一条要走产品的构造，由开卡器第一次开卡时自己建。
    drop(vulkan_instance);
    let opener = GpuOpenerOfThisProcess::of_this_process();
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(identifiers_of_the_handed_over_gpus(&handed_over_gpus)),
        background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(
            handed_over_gpus.len(),
        )),
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    };
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
    runtime_parameters.compaction_period = SHORT_COMPACTION_PERIOD;
    runtime_parameters.publish_poll_period = SHORT_PUBLISH_POLL_PERIOD;
    let events = Arc::new(CountingTheBatchesOnTheGpu::default());
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        runtime_parameters,
        Some(&opener),
        Arc::new(MonotonicClockOfThisProcess::starting_now()),
        Arc::clone(&events) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    )
    .expect("服务交来的卡、全合法的设置：挂载照常做成");
    assert_eq!(opener.instances_created(), 1, "开卡器自己建了一个实例");
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
    let given_up_at = Instant::now() + THREAD_DEADLINE;
    let mut foreground_fsyncs: u64 = 0;
    while events.batches_on_the_gpu.load(Ordering::SeqCst) < GPU_BATCHES_TO_SEE {
        assert!(
            Instant::now() < given_up_at,
            "线程在 GPU 上算过 {GPU_BATCHES_TO_SEE} 批（前台 fsync 了 {foreground_fsyncs} 次）：{:?}",
            runtime.observations()
        );
        runtime
            .accept(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            })
            .expect("前台收得下一次改动");
        runtime.publish_pending_now().expect("前台 fsync 做得成");
        foreground_fsyncs += 1;
    }
    let observations = runtime.observations();
    assert!(foreground_fsyncs > 0, "前台与批真交错过：{observations:?}");
    assert!(
        observations.compaction_publishes > 0,
        "整理发布过：{observations:?}"
    );
    assert_eq!(
        observations.alarms_received, 0,
        "一个报警都没有：{observations:?}"
    );
    // 这一条不入对拍结论（没有 CPU 一侧可比，只在真卡上跑通）：读不成这个进程在哪几张卡上时打过的 NOT_RUN_NO_GPU_ADAPTER 那一行就是它的本次未跑
    match assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run(
        NAME_OF_THE_TEST,
        &handed_over_gpus,
    ) {
        GpusThisProcessIsOn::OnlyTheHandedOver | GpusThisProcessIsOn::NotReadSoThisRunIsNotRun => {}
    }
    let mut unmounted = runtime.unmount().expect("卸载干净");
    let _mounted_again =
        mount_writable(&parameters(), &mut unmounted.devices).expect("卸载之后再挂得上");
}
