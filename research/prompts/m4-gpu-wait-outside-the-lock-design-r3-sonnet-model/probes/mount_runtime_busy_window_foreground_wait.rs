//! 第一轮基线探针（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/probes/mount_runtime_non_fsync_foreground_wait_baseline.rs`）的放开版：
//! 第一轮那趟取样 56279 次里中间夹了整理发布的只有 46 次（池子几十毫秒就整理完，其余时间维护线程闲着），均值与 p99 量的是闲着的那一段；
//! 前台只有一条线程、取锁之间睡 200 微秒、替身卡只睡 5 毫秒一档。这里放开四样：
//!   PROBE_FOREGROUND_READERS    读线程数（默认 1），每条线程各自取样、各自算「两次取到锁之间后台发了几次整理发布」；
//!   PROBE_FOREGROUND_WRITERS    写线程数（默认 0），每条线程隔 PROBE_WRITER_GAP_MICROSECONDS（默认 2000）经 `publish_user_change` 建一个 inode，量整个调用的时长；
//!   PROBE_SAMPLE_GAP_MICROSECONDS 读线程两次取锁之间睡多久（默认 200；0 是紧循环）；
//!   PROBE_GPU_SLEEP_MILLISECONDS  替身卡每次调用睡多久（默认 5；0、5、50 各跑一趟）。
//! 并且只对「被挡了」的取样（等了不少于 1000 微秒）另报一行分布：个数、p50、p99、最长、它们取到锁之前后台连着发了几次（0、1、2 到 3、4 及以上各几个）。
//! 这是读路径（`with_the_session_and_devices`，读 `read_at` 走它）与单发改动（`publish_user_change`）的量；没摆 fsync（`publish_pending_now`）与 `accept`。
//! 状态：这份源码没编过、没跑过（子 agent 不起 cargo）。`std::thread::scope` 里借 `&MountedRuntime<RecordedMemoryDevice>`，要它是 `Sync`：不是的话编不过，承接的腿改。
//! 用法：拷进仓副本的 `crates/singlefs-mount-runtime/tests/`（`tools/patch_the_copy.py` 做），经任务服务交
//!   cargo test -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- <测试全名> --exact --nocapture --test-threads=1
//! 输出行以 `name=busy_window_foreground_wait` 起头。
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
use singlefs_core::mounted_session::UserChange;
use singlefs_core::region_scan::{
    scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex,
};
use singlefs_mount_runtime::{
    BackgroundGpuNotOpened, IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess,
    MountedRuntime, OpenedBackgroundGpu, OpensTheBackgroundGpu, RuntimeMounted, RuntimeParameters,
};

const BLOCKED_AT_LEAST_MICROSECONDS: u128 = 1_000;

fn environment_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(default)
}

/// 同第一轮：全空聚簇段线取整段数的 100%，周期 5 毫秒、`publish_when_due` 周期 2 毫秒，整理一拍接一拍地跑。
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

struct Sample {
    waited: Duration,
    publishes_since_the_previous_sample: u64,
}

fn sample_as_a_reader(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    deadline: Instant,
    gap: Duration,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    let mut previous_reading: Option<u64> = None;
    while Instant::now() < deadline {
        let started = Instant::now();
        let reading = runtime
            .with_the_session_and_devices(|locked| {
                locked.session.session().compaction.observations.root_ring_slots_used
            })
            .expect("运行库没坏");
        let waited = started.elapsed();
        if let Some(previous) = previous_reading {
            samples.push(Sample { waited, publishes_since_the_previous_sample: reading - previous });
        }
        previous_reading = Some(reading);
        if !gap.is_zero() {
            std::thread::sleep(gap);
        }
    }
    samples
}

/// 写线程：隔一段建一个 inode，量整个 `publish_user_change` 调用的时长（含它自己写盘的时间，分不出取锁那一段）。
fn calls_of_a_writer(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    deadline: Instant,
    gap: Duration,
) -> Vec<Duration> {
    let mut calls = Vec::new();
    while Instant::now() < deadline {
        let started = Instant::now();
        let _ = runtime.publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: 1_788_000_003,
        });
        calls.push(started.elapsed());
        std::thread::sleep(gap);
    }
    calls
}

fn percentile(sorted: &[u128], percent: usize) -> u128 {
    if sorted.is_empty() {
        0
    } else {
        sorted[(sorted.len() * percent / 100).min(sorted.len() - 1)]
    }
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

struct Collected {
    samples: Vec<Sample>,
    writer_calls: Vec<Duration>,
    maintenance_publishes: u64,
}

fn report(route: &str, sleep_milliseconds: u64, collected: &Collected) {
    let readers = environment_number("PROBE_FOREGROUND_READERS", 1);
    let writers = environment_number("PROBE_FOREGROUND_WRITERS", 0);
    let gap = environment_number("PROBE_SAMPLE_GAP_MICROSECONDS", 200);
    let mut waits: Vec<u128> = collected.samples.iter().map(|sample| sample.waited.as_micros()).collect();
    waits.sort_unstable();
    let mean = if waits.is_empty() { 0 } else { waits.iter().sum::<u128>() / waits.len() as u128 };
    let blocked: Vec<&Sample> = collected
        .samples
        .iter()
        .filter(|sample| sample.waited.as_micros() >= BLOCKED_AT_LEAST_MICROSECONDS)
        .collect();
    let mut blocked_waits: Vec<u128> = blocked.iter().map(|sample| sample.waited.as_micros()).collect();
    blocked_waits.sort_unstable();
    let blocked_with = |low: u64, high: u64| {
        blocked
            .iter()
            .filter(|sample| (low..=high).contains(&sample.publishes_since_the_previous_sample))
            .count()
    };
    let mut call_micros: Vec<u128> = collected.writer_calls.iter().map(|call| call.as_micros()).collect();
    call_micros.sort_unstable();
    println!(
        "name=busy_window_foreground_wait route={route} sleep_ms_per_gpu_call={sleep_milliseconds} readers={readers} writers={writers} gap_us={gap} \
         samples={} wait_micros_mean={mean} wait_micros_p99={} wait_micros_max={} maintenance_publishes_total={} \
         blocked_samples={} blocked_wait_micros_p50={} blocked_wait_micros_p99={} blocked_wait_micros_max={} \
         blocked_publishes_none={} blocked_publishes_one={} blocked_publishes_two_to_three={} blocked_publishes_four_or_more={} \
         writer_calls={} writer_call_micros_p99={} writer_call_micros_max={}",
        waits.len(),
        percentile(&waits, 99),
        waits.last().copied().unwrap_or(0),
        collected.maintenance_publishes,
        blocked.len(),
        percentile(&blocked_waits, 50),
        percentile(&blocked_waits, 99),
        blocked_waits.last().copied().unwrap_or(0),
        blocked_with(0, 0),
        blocked_with(1, 1),
        blocked_with(2, 3),
        blocked_with(4, u64::MAX),
        call_micros.len(),
        percentile(&call_micros, 99),
        call_micros.last().copied().unwrap_or(0),
    );
}

fn collect_samples(
    runtime_parameters: &RuntimeParameters,
    opener: Option<&dyn OpensTheBackgroundGpu>,
) -> Collected {
    let window = Duration::from_millis(environment_number("PROBE_WINDOW_MILLISECONDS", 3_000));
    let gap = Duration::from_micros(environment_number("PROBE_SAMPLE_GAP_MICROSECONDS", 200));
    let writer_gap = Duration::from_micros(environment_number("PROBE_WRITER_GAP_MICROSECONDS", 2_000));
    let readers = environment_number("PROBE_FOREGROUND_READERS", 1);
    let writers = environment_number("PROBE_FOREGROUND_WRITERS", 0);
    let mut collected = Collected { samples: Vec::new(), writer_calls: Vec::new(), maintenance_publishes: 0 };
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
        let deadline = Instant::now() + window;
        let (reader_samples, writer_calls) = std::thread::scope(|scope| {
            let reader_handles: Vec<_> = (0..readers)
                .map(|_| scope.spawn(|| sample_as_a_reader(&runtime, deadline, gap)))
                .collect();
            let writer_handles: Vec<_> = (0..writers)
                .map(|_| scope.spawn(|| calls_of_a_writer(&runtime, deadline, writer_gap)))
                .collect();
            let samples: Vec<Sample> = reader_handles
                .into_iter()
                .flat_map(|handle| handle.join().expect("读线程没 panic"))
                .collect();
            let calls: Vec<Duration> = writer_handles
                .into_iter()
                .flat_map(|handle| handle.join().expect("写线程没 panic"))
                .collect();
            (samples, calls)
        });
        collected.samples.extend(reader_samples);
        collected.writer_calls.extend(writer_calls);
        collected.maintenance_publishes += runtime.observations().compaction_publishes;
        runtime.unmount().expect("卸载干净");
    }
    collected
}

/// CPU 路径：整批在锁里算，没有卡。
#[test]
fn busy_window_on_the_cpu_route() {
    let collected = collect_samples(&runtime_parameters_with_a_short_period(), None);
    report("cpu", 0, &collected);
}

/// 替身卡路径：每调一次三段里的任一段先睡 N 毫秒，整批在会话锁里等它。
#[test]
fn busy_window_on_a_sleeping_gpu_double() {
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
    let collected = collect_samples(&runtime_parameters, Some(&opener));
    report("gpu_double", sleep_milliseconds, &collected);
}
