//! 基线探针（今天的代码上量 Q6 第一条）：非 fsync 的前台（只取锁、什么都不改，等同读路径 `read_at` 走的 `with_the_session_and_devices`）在维护线程一拍里等多久，
//! 拿到锁之前后台又发了几次整理发布。两条路：CPU 路径，与替身卡路径（每次调三段里任何一段先睡 N 毫秒，N 取 `PROBE_GPU_SLEEP_MILLISECONDS`，默认 5；
//! D24 已定项 2 射程写 E185 量到一次真卡提交约 5.3 ms，替身只摆这一个数量级，不等于真卡）。
//!
//! 状态：这份源码没编过、没跑过（子 agent 不起 cargo）。用法：主 agent 在仓副本里把它拷进 `crates/singlefs-mount-runtime/tests/`
//! （同目录已有 `common_pool_on_memory_devices`），再整条经任务服务交：
//!   task-server/task submit --cwd <仓副本根> -- cargo test -p singlefs-mount-runtime --test mount_runtime_non_fsync_foreground_wait_baseline -- --nocapture --test-threads=1
//! 点名这一份测试目标里的全部测试，属「点名测试函数」那一条，交的时候照共用约束补范围声明。输出行以 `name=baseline_foreground_wait` 起头。
//! 局限：只有一个池、每次重挂都是新池，整理做完（池里没有可搬的段）之后后台不再发布，采样只在整理还在跑的那一段里有意义，所以重复 `PROBE_REPEATS` 次；
//! 前台两次取锁之间睡 `PROBE_SAMPLE_GAP_MICROSECONDS`（默认 200 微秒），「两次取锁之间后台发了几次」含这段睡眠，是拿到锁之前发布数的上界。
mod common_pool_on_memory_devices;

use std::sync::Arc;
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_with_holes_in_the_data_segments, RecordedMemoryDevice,
};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, ClusterSegmentEntryKey,
    GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    live_slots_per_cluster_segment_by_the_records, CompactionMountParameters, CompactionWatermarks,
    EmptyClusterSegmentsWatermark, FragmentationWatermark, UnitAreaOfOneDevice,
    UnreclaimableBytesWatermark, COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{
    scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex,
};
use singlefs_mount_runtime::{
    BackgroundGpuNotOpened, IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess,
    MountedRuntime, OpenedBackgroundGpu, OpensTheBackgroundGpu, RuntimeMounted, RuntimeParameters,
};

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(default)
}

/// 全空聚簇段线取整段数的 100%（有一段不全空就越线），周期 5 毫秒、`publish_when_due` 周期 2 毫秒：整理一拍接一拍地跑。
fn runtime_parameters_with_a_short_period() -> RuntimeParameters {
    let mut runtime_parameters = RuntimeParameters::by_default();
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
    runtime_parameters.compaction_period = Duration::from_millis(5);
    runtime_parameters.publish_poll_period = Duration::from_millis(2);
    runtime_parameters
}

/// 前台取锁一次的读数：等了多久，与这一次取到锁时整理已发的发布总数。
struct Sample {
    waited: Duration,
    publishes_since_the_previous_sample: u64,
}

fn sample_the_foreground_wait(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    window: Duration,
    gap: Duration,
) -> Vec<Sample> {
    let deadline = Instant::now() + window;
    let mut samples = Vec::new();
    let mut previous_reading: Option<u64> = None;
    while Instant::now() < deadline {
        let started = Instant::now();
        let reading = runtime
            .with_the_session_and_devices(|locked| {
                locked
                    .session
                    .session()
                    .compaction
                    .observations
                    .root_ring_slots_used
            })
            .expect("运行库没坏");
        let waited = started.elapsed();
        if let Some(previous) = previous_reading {
            samples.push(Sample {
                waited,
                publishes_since_the_previous_sample: reading - previous,
            });
        }
        previous_reading = Some(reading);
        std::thread::sleep(gap);
    }
    samples
}

fn report(route: &str, sleep_milliseconds: u64, samples: &[Sample]) {
    let mut waits: Vec<u128> = samples.iter().map(|sample| sample.waited.as_micros()).collect();
    waits.sort_unstable();
    let count = waits.len();
    let mean = if count == 0 { 0 } else { waits.iter().sum::<u128>() / count as u128 };
    let p99 = if count == 0 { 0 } else { waits[(count * 99 / 100).min(count - 1)] };
    let maximum = waits.last().copied().unwrap_or(0);
    let with_a_publish = samples
        .iter()
        .filter(|sample| sample.publishes_since_the_previous_sample > 0)
        .count();
    let most_publishes = samples
        .iter()
        .map(|sample| sample.publishes_since_the_previous_sample)
        .max()
        .unwrap_or(0);
    println!(
        "name=baseline_foreground_wait route={route} sleep_ms_per_gpu_call={sleep_milliseconds} samples={count} \
         wait_micros_mean={mean} wait_micros_p99={p99} wait_micros_max={maximum} \
         samples_with_a_publish_in_between={with_a_publish} max_publishes_in_between={most_publishes}"
    );
}

fn card_of_the_double() -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(b"card-of-the-double".to_vec())
}

const MEMORY_OF_THE_CARD_OF_THE_DOUBLE: GpuMemoryInBytes = GpuMemoryInBytes(8 << 30);

/// 替身卡：三段都照 CPU 写法算（同 `singlefs-core/src/region_scan.rs` 里 `CpuWritingsStandingInForTheGpu` 的写法），每次调用先睡一段，摆「等卡」的时长。
struct GpuDoubleThatSleepsLikeACard {
    sleep_for_every_call: Duration,
}

impl GpuInventory for GpuDoubleThatSleepsLikeACard {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![(card_of_the_double(), MEMORY_OF_THE_CARD_OF_THE_DOUBLE)]
    }
}

impl BackgroundGpuCompute for GpuDoubleThatSleepsLikeACard {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        std::thread::sleep(self.sleep_for_every_call);
        Ok(units
            .iter()
            .map(|unit| {
                let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
                GpuEntryDecision::ConfirmedByBothKernels(UnitChecksumOutcome {
                    computed_crc32c,
                    matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c,
                })
            })
            .collect())
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        std::thread::sleep(self.sleep_for_every_call);
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
        Ok(records
            .iter()
            .map(|to_classify| {
                let mut scan = scan_region_over_the_identity_index(
                    region,
                    std::slice::from_ref(&to_classify.record),
                    &mapping_index,
                    &std::collections::BTreeSet::new(),
                    &std::collections::BTreeSet::new(),
                    |_device, _slot, _unit_bytes| to_classify.unit_bytes.map(<[u8]>::to_vec),
                )
                .expect("单条记录不跨 R 的边界、不会两条同 key");
                GpuEntryDecision::ConfirmedByBothKernels(
                    scan.allocations
                        .pop()
                        .expect("单条记录起点在 R 里、未释放：扫出恰好一条")
                        .class,
                )
            })
            .collect())
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        _segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        std::thread::sleep(self.sleep_for_every_call);
        Ok(
            live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area)
                .into_iter()
                .map(GpuEntryDecision::ConfirmedByBothKernels)
                .collect(),
        )
    }

    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
}

struct OpenerHandingOverTheSleepingDouble {
    gpu_compute: BackgroundGpuComputeHandle,
}

impl OpensTheBackgroundGpu for OpenerHandingOverTheSleepingDouble {
    fn open_the_background_gpu(
        &self,
        _settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        Ok(OpenedBackgroundGpu {
            gpu_compute: self.gpu_compute.clone(),
            report_of_the_opening: String::new(),
            gpus_leaked: None,
        })
    }
}

fn collect_samples(
    runtime_parameters: &RuntimeParameters,
    opener: Option<&dyn OpensTheBackgroundGpu>,
) -> Vec<Sample> {
    let window = Duration::from_millis(environment_number("PROBE_WINDOW_MILLISECONDS", 3_000));
    let gap = Duration::from_micros(environment_number("PROBE_SAMPLE_GAP_MICROSECONDS", 200));
    let mut every_sample = Vec::new();
    for _repeat in 0..environment_number("PROBE_REPEATS", 5) {
        let pool = pool_with_holes_in_the_data_segments();
        let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
            &parameters(),
            pool.devices,
            runtime_parameters.clone(),
            opener,
            Arc::new(MonotonicClockOfThisProcess::starting_now()),
            Arc::new(IgnoringTheMaintenanceEvents),
        )
        .expect("全合法的参数：挂载照常做成");
        every_sample.extend(sample_the_foreground_wait(&runtime, window, gap));
        runtime.unmount().expect("卸载干净");
    }
    every_sample
}

/// CPU 路径：整批在锁里算（读盘、建索引、区域扫描、发布），没有卡。
// 不切片并行：被测的是一条维护线程与一个前台在一把锁上的交错。
#[test]
fn foreground_wait_against_a_running_maintenance_thread_on_the_cpu_route() {
    let samples = collect_samples(&runtime_parameters_with_a_short_period(), None);
    report("cpu", 0, &samples);
}

/// 替身卡路径：每调一次三段里的任一段先睡 N 毫秒，整批在会话锁里等它。
#[test]
fn foreground_wait_against_a_running_maintenance_thread_on_a_sleeping_gpu_double() {
    let sleep_milliseconds = environment_number("PROBE_GPU_SLEEP_MILLISECONDS", 5);
    let opener = OpenerHandingOverTheSleepingDouble {
        gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(GpuDoubleThatSleepsLikeACard {
            sleep_for_every_call: Duration::from_millis(sleep_milliseconds),
        }),
    };
    let mut runtime_parameters = runtime_parameters_with_a_short_period();
    runtime_parameters.background_compute = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_of_the_double()]),
        background_gpu_memory_limit: Some(vec![MEMORY_OF_THE_CARD_OF_THE_DOUBLE]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    let samples = collect_samples(&runtime_parameters, Some(&opener));
    report("gpu_double", sleep_milliseconds, &samples);
}
