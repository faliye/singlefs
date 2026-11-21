//! 锁外等 GPU 时，GPU 实现那把锁（`BackgroundGpuComputeHandle` 收的 `Arc<Mutex<dyn BackgroundGpuCompute + Send>>`）由维护线程持着、会话锁放开。
//! 前台 `MountedRuntime::switch_background_compute` 先取会话锁，切到 `gpu` 时在校验里再取 GPU 实现那把锁
//! （`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold` 里 `gpu_compute.lock()`）；切到 `cpu` 不取。
//! 这里用一条线程直接持那把锁 `hold` 那么久，代替「锁外那一段在卡上等」（真卡挂住时一段至多约 4 × 等的上限，推的）；不起维护线程、不跑批。
//! 三臂：
//!   released_before_the_session_lock：持锁线程先放 GPU 锁、再取会话锁（第三段回锁之前放）；量前台切换与随后一次读各被挡多久；
//!   held_into_the_session_lock_switch_to_cpu：持着 GPU 锁去取会话锁，前台切到 cpu（对照：切到 cpu 不取 GPU 锁）；
//!   held_into_the_session_lock_switch_to_gpu：持着 GPU 锁去取会话锁（今天 `implementation` 守到整批发完那样），前台切到 gpu；10 秒没齐就记死锁。
use std::sync::mpsc::channel;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier,
    GpuEntryDecision, GpuInventory, GpuMemoryInBytes, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
    UnitChecksumToVerifyOnTheGpu,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_mount_runtime::{
    IgnoringTheMaintenanceEvents, MonotonicClockOfThisProcess, MountedRuntime, RuntimeParameters,
};
use singlefs_verification::memory_pool::SparseBlockDevice;

use crate::pool::{copied, parameters, start_pool};

type SharedGpu = Arc<Mutex<dyn BackgroundGpuCompute + Send>>;
type Runtime = Arc<MountedRuntime<SparseBlockDevice>>;

/// 只答卡清单、从不被叫去算的实现：这一组臂不跑批。
struct GpuThatIsNeverAskedToCompute;

impl GpuInventory for GpuThatIsNeverAskedToCompute {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![(GpuDeviceIdentifier::of_opaque_bytes(vec![0x01]), GpuMemoryInBytes(1 << 30))]
    }
}

impl BackgroundGpuCompute for GpuThatIsNeverAskedToCompute {
    fn verify_unit_checksums(
        &mut self,
        _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        unreachable!("这一组臂不跑批")
    }
    fn classify_region_records(
        &mut self,
        _region: CompactionRegion,
        _records: &[RegionRecordToClassifyOnTheGpu<'_>],
        _mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        unreachable!("这一组臂不跑批")
    }
    fn count_live_slots_per_cluster_segment(
        &mut self,
        _allocation_records: &[AllocationRecord],
        _unit_area: UnitAreaOfOneDevice,
        _segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        unreachable!("这一组臂不跑批")
    }
    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
}

fn gpu_settings() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: Duration::from_secs(10),
    }
}

fn milliseconds(since: Instant) -> u128 {
    since.elapsed().as_millis()
}

/// 起三条线程：持锁线程（持 GPU 锁 `hold`，`keeps_it_into_the_session_lock` 为真时持着它去取会话锁，否则先放再取）、前台切换（5 毫秒后）、
/// 前台读（15 毫秒后，`with_the_session_and_devices` 一次空闭包）。交回三条各自取到会话锁之前等了几毫秒；`deadline` 内没齐的记 `None`。
fn three_threads(
    runtime: &Runtime,
    gpu: &SharedGpu,
    hold: Duration,
    keeps_it_into_the_session_lock: bool,
    switch_to_the_gpu: bool,
    deadline: Duration,
) -> [Option<u128>; 3] {
    let barrier = Arc::new(Barrier::new(3));
    let (done, finished) = channel::<(usize, u128)>();
    {
        let (runtime, gpu, barrier, done) = (Arc::clone(runtime), Arc::clone(gpu), Arc::clone(&barrier), done.clone());
        thread::spawn(move || {
            let guard = gpu.lock().expect("这把锁没在别处 panic 过");
            barrier.wait();
            thread::sleep(hold);
            if !keeps_it_into_the_session_lock {
                drop(guard);
                let asked = Instant::now();
                runtime.with_the_session_and_devices(|_| ()).expect("运行库没坏");
                let _ = done.send((0, milliseconds(asked)));
            } else {
                let asked = Instant::now();
                runtime.with_the_session_and_devices(|_| ()).expect("运行库没坏");
                let waited = milliseconds(asked);
                drop(guard);
                let _ = done.send((0, waited));
            }
        });
    }
    {
        let (runtime, gpu, barrier, done) = (Arc::clone(runtime), Arc::clone(gpu), Arc::clone(&barrier), done.clone());
        thread::spawn(move || {
            barrier.wait();
            thread::sleep(Duration::from_millis(5));
            let asked = Instant::now();
            let switched = if switch_to_the_gpu {
                runtime.switch_background_compute(
                    gpu_settings(),
                    Some(BackgroundGpuComputeHandle::of_a_shared_implementation(gpu)),
                    None,
                )
            } else {
                runtime.switch_background_compute(BackgroundComputeSettings::PRODUCT, None, None)
            };
            switched.expect("切换照收");
            let _ = done.send((1, milliseconds(asked)));
        });
    }
    {
        let (runtime, barrier, done) = (Arc::clone(runtime), Arc::clone(&barrier), done);
        thread::spawn(move || {
            barrier.wait();
            thread::sleep(Duration::from_millis(15));
            let asked = Instant::now();
            runtime.with_the_session_and_devices(|_| ()).expect("运行库没坏");
            let _ = done.send((2, milliseconds(asked)));
        });
    }
    let mut waits = [None, None, None];
    let started = Instant::now();
    while waits.iter().any(Option::is_none) {
        let left = deadline.saturating_sub(started.elapsed());
        match finished.recv_timeout(left) {
            Ok((which, waited)) => waits[which] = Some(waited),
            Err(_timed_out) => break,
        }
    }
    waits
}

fn shown(wait: Option<u128>) -> String {
    wait.map_or_else(|| "not_done".to_string(), |milliseconds| milliseconds.to_string())
}

pub fn run() {
    let started = Instant::now();
    let start = start_pool();
    let runtime: Runtime = Arc::new(
        MountedRuntime::mount_without_the_maintenance_thread(
            &parameters(),
            copied(&start),
            RuntimeParameters::by_default(),
            None,
            Arc::new(MonotonicClockOfThisProcess::starting_now()),
            Arc::new(IgnoringTheMaintenanceEvents),
        )
        .unwrap_or_else(|_refused| panic!("运行库挂不上"))
        .runtime,
    );
    let gpu: SharedGpu = Arc::new(Mutex::new(GpuThatIsNeverAskedToCompute));
    let deadline = Duration::from_secs(10);
    for hold_milliseconds in [100_u64, 400, 1600] {
        for switch_to_the_gpu in [true, false] {
            let hold = Duration::from_millis(hold_milliseconds);
            let [holder, switch, read] = three_threads(&runtime, &gpu, hold, false, switch_to_the_gpu, deadline);
            println!(
                "LOCK name=lock_order arm=released_before_the_session_lock switch_to={} hold_ms={hold_milliseconds} holder_waited_ms={} switch_waited_ms={} read_waited_ms={}",
                if switch_to_the_gpu { "gpu" } else { "cpu" }, shown(holder), shown(switch), shown(read)
            );
        }
    }
    let [holder, switch, read] = three_threads(&runtime, &gpu, Duration::from_millis(50), true, false, deadline);
    println!(
        "LOCK name=lock_order arm=held_into_the_session_lock_switch_to_cpu hold_ms=50 holder_waited_ms={} switch_waited_ms={} read_waited_ms={} all_done={}",
        shown(holder), shown(switch), shown(read), u8::from(holder.is_some() && switch.is_some() && read.is_some())
    );
    let [holder, switch, read] = three_threads(&runtime, &gpu, Duration::from_millis(50), true, true, deadline);
    let deadlocked = holder.is_none() && switch.is_none();
    println!(
        "LOCK name=lock_order arm=held_into_the_session_lock_switch_to_gpu hold_ms=50 holder_waited_ms={} switch_waited_ms={} read_waited_ms={} deadline_s={} deadlocked={}",
        shown(holder), shown(switch), shown(read), deadline.as_secs(), u8::from(deadlocked)
    );
    println!("RESULT name=lock_order deadlocked_when_held_into_the_session_lock={}", u8::from(deadlocked));
    println!("WALL_SECONDS name=lock_order seconds={:.1}", started.elapsed().as_secs_f64());
    // 死锁的两条线程收不回来：直接退出进程（不析构运行库）。
    std::process::exit(0);
}
