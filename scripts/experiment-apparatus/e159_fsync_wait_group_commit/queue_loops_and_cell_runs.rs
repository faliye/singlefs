//! 忙等、组提交队列、预写日志记账、样本记录、发布者与检查点与调用方的循环、一格闭环的整跑。

use e7_index_bench::{truncate_f64_to_u64, truncate_f64_to_usize, u64_from_nanoseconds};
use super::*;

fn busy_wait(duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

// ============================================================
// 组提交队列、WAL checkpoint 触发、闭环调用方与发布者（第五节 5.2、5.5、5.6）
// ============================================================

pub(super) struct QueueEntry {
    pub(super) reply: mpsc::Sender<(Instant, Instant, Instant)>, // (batch_started_at, made_durable_at, released_at)
}

pub(super) struct SharedQueue {
    entries: Mutex<VecDeque<QueueEntry>>,
    condvar: Condvar,
    stop: std::sync::atomic::AtomicBool,
}
impl SharedQueue {
    pub(super) fn new() -> SharedQueue {
        SharedQueue { entries: Mutex::new(VecDeque::new()), condvar: Condvar::new(), stop: std::sync::atomic::AtomicBool::new(false) }
    }
}

pub(super) struct WriteAheadLogAccounting {
    bytes_since_checkpoint: AtomicU64,
    fsync_count_since_cut: AtomicU64,
    batch_count_since_cut: AtomicU64,
    last_checkpoint_instant: Mutex<Instant>,
    pub(super) checkpoint_count: AtomicU64,
    pub(super) checkpoint_bytes_total: AtomicU64,
    stop: std::sync::atomic::AtomicBool,
}
impl WriteAheadLogAccounting {
    pub(super) fn new() -> WriteAheadLogAccounting {
        WriteAheadLogAccounting {
            bytes_since_checkpoint: AtomicU64::new(0),
            fsync_count_since_cut: AtomicU64::new(0),
            batch_count_since_cut: AtomicU64::new(0),
            last_checkpoint_instant: Mutex::new(Instant::now()),
            checkpoint_count: AtomicU64::new(0),
            checkpoint_bytes_total: AtomicU64::new(0),
            stop: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// `T_time` 默认值（D16 已定项 5，第二节 2.2 第二块）：5 s。装置里唯一的权威落点——
/// `run_cell_subcommand` 与真格测试都读这个常量，不各自写一份字面量。
/// M8 的变异目标：读错成 50 s 会让主网格 10 s 窗口里 0 次触发（跑前登记第十一节 V10）。
pub(super) const CHECKPOINT_TIME_THRESHOLD: Duration = Duration::from_millis(5000);

/// WAL checkpoint 的触发判据（D16 已定项 2 抄，第二节 2.2 第一块）。
pub(super) fn should_trigger_checkpoint(now: Instant, last_checkpoint: Instant, bytes_since_checkpoint: u64, checkpoint_time_threshold: Duration, effective_dirty_bytes_threshold: u64) -> bool {
    now.duration_since(last_checkpoint) >= checkpoint_time_threshold || bytes_since_checkpoint >= effective_dirty_bytes_threshold
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SampleRecord {
    pub(super) issued_at: Instant,
    pub(super) batch_started_at: Instant,
    pub(super) made_durable_at: Instant,
    pub(super) released_at: Instant,
    pub(super) returned_at: Instant,
    pub(super) batch_size: u64,
}
impl SampleRecord {
    pub(super) fn wait_nanos(&self) -> u64 {
        u64_from_nanoseconds(self.returned_at.duration_since(self.issued_at).as_nanos())
    }
    /// 「放行到醒」这一段（M14 的变异目标：`returned_at` 若在发布者那边、持久点时刻取，这一段恒为 0）。
    pub(super) fn release_to_wake_nanos(&self) -> u64 {
        u64_from_nanoseconds(self.returned_at.saturating_duration_since(self.released_at).as_nanos())
    }
    /// V5 包含自检：`issued_at <= batch_started_at <= made_durable_at <= released_at <= returned_at`。
    pub(super) fn violates_containment(&self) -> bool {
        !(self.issued_at <= self.batch_started_at && self.batch_started_at <= self.made_durable_at && self.made_durable_at <= self.released_at && self.released_at <= self.returned_at)
    }
}

/// 一批从队列里怎么收（第五节 5.2「组提交」）：组提交开 ⇒ 把队列现有的全部请求收成一批；
/// 组提交关 ⇒ 恰好收 1 个。M4 的变异目标：把「开」那一支也换成只取 1 个（组提交名存实亡）。
pub(super) fn drain_batch_from_queue(queue: &mut VecDeque<QueueEntry>, group_commit_enabled: bool) -> Vec<QueueEntry> {
    if group_commit_enabled {
        queue.drain(..).collect()
    } else {
        let mut one = VecDeque::new();
        if let Some(entry) = queue.pop_front() {
            one.push_back(entry);
        }
        one.into()
    }
}

/// 一批交给哪个臂的规划函数（第五节 5.2「怎么做」）。M3 的变异目标：把某个臂的分支换成
/// 别的臂的规划函数（「臂写混」）——三条臂各自的批必须只由自己的规划函数产出。
pub(super) fn plan_batch_for_arm(arm: Arm, shape: PoolShape, batch_size: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    match arm {
        Arm::Jia => plan_jia_batch(shape, batch_size, shared, random_state),
        Arm::WriteAheadLogFull => plan_write_ahead_log_fsync_batch(shape, WriteAheadLogArm::WriteAheadLogFull, batch_size, shared, random_state),
        Arm::WriteAheadLogLeaf => plan_write_ahead_log_fsync_batch(shape, WriteAheadLogArm::WriteAheadLogLeaf, batch_size, shared, random_state),
    }
}

#[allow(clippy::too_many_arguments, reason = "发布线程要的设备、缓冲、游标、记账与各开关逐个传入，拆结构体不会少传哪一项")]
fn publisher_loop(
    queue: &SharedQueue,
    group_commit_enabled: bool,
    device: &(dyn BlockDevice + Sync),
    buffers: &UnitBuffers,
    cursors: &RegionCursors,
    shape: PoolShape,
    arm: Arm,
    shared: bool,
    worker_count: usize,
    pc1_inject: Option<Duration>,
    random_state: &Mutex<u64>,
    write_ahead_log_accounting: Option<&WriteAheadLogAccounting>,
    batch_index_counter: &AtomicU64,
    bytes_written_total: &AtomicU64,
) {
    loop {
        let batch: Vec<QueueEntry> = {
            let mut guard = queue.entries.lock().unwrap();
            loop {
                if !guard.is_empty() {
                    break;
                }
                if queue.stop.load(Ordering::SeqCst) {
                    return;
                }
                let (next_guard, _timeout) = queue.condvar.wait_timeout(guard, Duration::from_millis(5)).unwrap();
                guard = next_guard;
            }
            drain_batch_from_queue(&mut guard, group_commit_enabled)
        };
        if batch.is_empty() {
            continue;
        }
        let batch_started_at = Instant::now();
        let batch_size = batch.len() as u64;
        let plan = {
            let mut state = random_state.lock().unwrap();
            plan_batch_for_arm(arm, shape, batch_size, shared, &mut state)
        };
        let batch_index = batch_index_counter.fetch_add(1, Ordering::Relaxed);
        let made_durable_at = execute_batch_persist(device, buffers, cursors, &plan, worker_count);
        if let Some(inject) = pc1_inject {
            if batch_index.is_multiple_of(2) {
                busy_wait(inject);
            }
        }
        let released_at = Instant::now();
        for entry in &batch {
            let _ = entry.reply.send((batch_started_at, made_durable_at, released_at));
        }
        execute_batch_finalize_system_configuration(device, buffers, cursors, &plan);
        bytes_written_total.fetch_add(plan.approximate_bytes_single_device(), Ordering::Relaxed);
        if let Some(accounting) = write_ahead_log_accounting {
            accounting.fsync_count_since_cut.fetch_add(batch_size, Ordering::Relaxed);
            accounting.batch_count_since_cut.fetch_add(1, Ordering::Relaxed);
            accounting.bytes_since_checkpoint.fetch_add(plan.approximate_bytes_single_device(), Ordering::Relaxed);
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "checkpoint 线程要的设备、缓冲、游标、记账与各开关逐个传入，拆结构体不会少传哪一项")]
fn checkpoint_loop(accounting: &WriteAheadLogAccounting, device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, shape: PoolShape, write_ahead_log_arm: WriteAheadLogArm, shared: bool, worker_count: usize, checkpoint_time_threshold: Duration, effective_dirty_bytes_threshold: u64, random_state: &Mutex<u64>) {
    loop {
        if accounting.stop.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
        let now = Instant::now();
        let last = *accounting.last_checkpoint_instant.lock().unwrap();
        if !should_trigger_checkpoint(now, last, accounting.bytes_since_checkpoint.load(Ordering::Relaxed), checkpoint_time_threshold, effective_dirty_bytes_threshold) {
            continue;
        }
        let fsync_count = accounting.fsync_count_since_cut.swap(0, Ordering::SeqCst);
        let batch_count = accounting.batch_count_since_cut.swap(0, Ordering::SeqCst).max(1);
        accounting.bytes_since_checkpoint.store(0, Ordering::SeqCst);
        *accounting.last_checkpoint_instant.lock().unwrap() = Instant::now();
        if fsync_count == 0 {
            continue; // 这一格窗口内还没有新 fsync，不必做空 checkpoint
        }
        let concurrent_fsync_count = truncate_f64_to_u64((fsync_count as f64 / batch_count as f64).round()).max(1);
        let plan = {
            let mut state = random_state.lock().unwrap();
            plan_write_ahead_log_checkpoint(shape, write_ahead_log_arm, concurrent_fsync_count, shared, batch_count, IntermediateVersionPolicy::ReleasedAndBacklogged, &mut state)
        };
        let _made_durable_at = execute_batch_persist(device, buffers, cursors, &plan, worker_count);
        execute_batch_finalize_system_configuration(device, buffers, cursors, &plan);
        accounting.checkpoint_count.fetch_add(1, Ordering::SeqCst);
        accounting.checkpoint_bytes_total.fetch_add(plan.approximate_bytes_single_device(), Ordering::SeqCst);
    }
}

fn caller_loop(queue: &SharedQueue, caller_stop: &std::sync::atomic::AtomicBool, samples_out: &Mutex<Vec<SampleRecord>>) {
    loop {
        if caller_stop.load(Ordering::SeqCst) {
            return;
        }
        let issued_at = Instant::now();
        let (reply_sender, rx) = mpsc::channel();
        {
            let mut guard = queue.entries.lock().unwrap();
            guard.push_back(QueueEntry { reply: reply_sender });
            queue.condvar.notify_all();
        }
        match rx.recv() {
            Ok((batch_started_at, made_durable_at, released_at)) => {
                let returned_at = Instant::now();
                samples_out.lock().unwrap().push(SampleRecord { issued_at, batch_started_at, made_durable_at, released_at, returned_at, batch_size: 0 });
            }
            Err(_) => return,
        }
    }
}

/// 开环里 `issued_at` 的语义（第一节「读法写死」）：取排定的到达时刻，不取实际发出的时刻——
/// 发晚了的那一段也算等待。M13 的反例是直接返回 `actual_issue_instant`。
pub(super) fn open_loop_issued_at(scheduled_instant: Instant, _actual_issue_instant: Instant) -> Instant {
    scheduled_instant
}

pub(super) struct CellConfiguration {
    pub(super) arm: Arm,
    pub(super) concurrent_fsync_count: u64,
    pub(super) shared: bool,
    pub(super) group_commit_enabled: bool,
    pub(super) warmup: Duration,
    pub(super) duration: Duration,
    pub(super) worker_count: usize,
    pub(super) pc1_inject_duration: Option<Duration>,
    pub(super) checkpoint_time_threshold: Duration,
    pub(super) effective_dirty_bytes_threshold: u64,
    pub(super) seed: u64,
    pub(super) shape: PoolShape,
}

pub(super) struct CellReport {
    pub(super) samples_in_window: Vec<SampleRecord>,
    pub(super) total_samples: usize,
    pub(super) containment_violations: usize,
    pub(super) checkpoint_count: u64,
    pub(super) checkpoint_bytes_total: u64,
    pub(super) bytes_written_estimate: u64,
}

/// 跑一个闭环格（第五节 5.5）：k 个调用方、零思考时间，跑 `warmup + duration`，只收
/// `issued_at` 落在 `[warmup, warmup+duration)` 里的样本。
pub(super) fn run_closed_loop_cell(device: Arc<dyn BlockDevice>, configuration: CellConfiguration) -> CellReport {
    let buffers = Arc::new(UnitBuffers::new(configuration.seed));
    let cursors = Arc::new(RegionCursors::new());
    let queue = Arc::new(SharedQueue::new());
    let random_state = Arc::new(Mutex::new(configuration.seed | 1));
    let caller_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let samples_out: Arc<Mutex<Vec<SampleRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let batch_index_counter = Arc::new(AtomicU64::new(0));

    let write_ahead_log_arm = configuration.arm.write_ahead_log_arm();
    let write_ahead_log_accounting = write_ahead_log_arm.map(|_| Arc::new(WriteAheadLogAccounting::new()));
    let publisher_bytes_total = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    {
        let queue = Arc::clone(&queue);
        let device = Arc::clone(&device);
        let buffers = Arc::clone(&buffers);
        let cursors = Arc::clone(&cursors);
        let random_state = Arc::clone(&random_state);
        let batch_index_counter = Arc::clone(&batch_index_counter);
        let write_ahead_log_accounting_for_publisher = write_ahead_log_accounting.clone();
        let shape = configuration.shape;
        let arm = configuration.arm;
        let shared = configuration.shared;
        let worker_count = configuration.worker_count;
        let group_commit_enabled = configuration.group_commit_enabled;
        let pc1_inject = configuration.pc1_inject_duration;
        let bytes_written_total = Arc::clone(&publisher_bytes_total);
        handles.push(std::thread::spawn(move || {
            publisher_loop(&queue, group_commit_enabled, &*device, &buffers, &cursors, shape, arm, shared, worker_count, pc1_inject, &random_state, write_ahead_log_accounting_for_publisher.as_deref(), &batch_index_counter, &bytes_written_total);
        }));
    }
    if let (Some(write_ahead_log_arm), Some(accounting)) = (write_ahead_log_arm, write_ahead_log_accounting.clone()) {
        let device = Arc::clone(&device);
        let buffers = Arc::clone(&buffers);
        let cursors = Arc::clone(&cursors);
        let random_state = Arc::clone(&random_state);
        let shape = configuration.shape;
        let shared = configuration.shared;
        let worker_count = configuration.worker_count;
        let checkpoint_time_threshold = configuration.checkpoint_time_threshold;
        let effective_dirty_bytes_threshold = configuration.effective_dirty_bytes_threshold;
        handles.push(std::thread::spawn(move || {
            checkpoint_loop(&accounting, &*device, &buffers, &cursors, shape, write_ahead_log_arm, shared, worker_count, checkpoint_time_threshold, effective_dirty_bytes_threshold, &random_state);
        }));
    }

    let mut caller_handles = Vec::new();
    for _ in 0..configuration.concurrent_fsync_count {
        let queue = Arc::clone(&queue);
        let caller_stop = Arc::clone(&caller_stop);
        let samples_out = Arc::clone(&samples_out);
        caller_handles.push(
            std::thread::Builder::new()
                .stack_size(64 * 1024)
                .spawn(move || caller_loop(&queue, &caller_stop, &samples_out))
                .expect("起调用方线程失败"),
        );
    }

    let run_start = Instant::now();
    let deadline = run_start + configuration.warmup + configuration.duration;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now())));
    }

    caller_stop.store(true, Ordering::SeqCst);
    for handle in caller_handles {
        let _ = handle.join();
    }
    queue.stop.store(true, Ordering::SeqCst);
    queue.condvar.notify_all();
    if let Some(accounting) = &write_ahead_log_accounting {
        accounting.stop.store(true, Ordering::SeqCst);
    }
    for handle in handles {
        let _ = handle.join();
    }

    let all_samples = std::mem::take(&mut *samples_out.lock().unwrap());
    let warmup_cutoff = run_start + configuration.warmup;
    let samples_in_window: Vec<SampleRecord> = all_samples.iter().copied().filter(|sample| sample.issued_at >= warmup_cutoff).collect();
    let containment_violations = samples_in_window.iter().filter(|sample| sample.violates_containment()).count();

    let publisher_bytes = publisher_bytes_total.load(Ordering::SeqCst);
    let checkpoint_bytes = write_ahead_log_accounting.as_ref().map(|accounting| accounting.checkpoint_bytes_total.load(Ordering::SeqCst)).unwrap_or(0);

    CellReport {
        total_samples: samples_in_window.len(),
        containment_violations,
        checkpoint_count: write_ahead_log_accounting.as_ref().map(|accounting| accounting.checkpoint_count.load(Ordering::SeqCst)).unwrap_or(0),
        checkpoint_bytes_total: checkpoint_bytes,
        bytes_written_estimate: publisher_bytes + checkpoint_bytes,
        samples_in_window,
    }
}

// ============================================================
// 统计：分位数、规则 R、J1 / J2a / J4 判定、Mann–Whitney A（第六节）
// ============================================================

/// 最近秩分位数（p ∈ [0,1]），输入必须已排序。
pub(super) fn percentile_nanos(sorted_values: &[u64], percentile_fraction: f64) -> u64 {
    assert!(!sorted_values.is_empty(), "分位数不能在空样本上算——读不到 ≠ 读到 0");
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    let rank = truncate_f64_to_usize((percentile_fraction * (sorted_values.len() - 1) as f64).round());
    sorted_values[rank.min(sorted_values.len() - 1)]
}
