//! 切片执行器（登记第十二节修订第 10 条：只改调度、不改判据；输出按片的次序，与线程数无关）：线程数、内存上限与按片并行。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 切片执行器（登记第十二节修订第 10 条：只改调度、不改判据；输出按片的次序，与线程数无关）
// ---------------------------------------------------------------------------------------------------------------------

/// 进度行的间隔（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」：5 分钟）。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);
/// 线程数：先读这个，没设取每台默认重活线程数（`singlefs_verification::heavy_threads`：`SINGLEFS_THREAD_CAP`，再没设取登记值）。
const THREADS_ENVIRONMENT_VARIABLE: &str = "E168_THREADS";
/// 内存上限：先读这个（`40G` 这种写法：正整数加 K / M / G / T，或字节数），没设取本进程控制组的 `memory.max`（服务经 `run-with-memory-cap.sh`
/// 给的上限落在那里），那里是 `max` 或读不到时取 [`DEFAULT_MEMORY_CAP_BYTES`]。取法照 E185（GC 批墙钟拆分）装置的 `memory_cap_given`。
const MEMORY_CAP_ENVIRONMENT_VARIABLE: &str = "E168_MEMORY_CAP";
/// 两处都没给内存上限时取 40 GiB。
const DEFAULT_MEMORY_CAP_BYTES: u64 = 40 << 30;
/// 大片（P4）一片的峰值：4 GiB，按 P4 的盘大小估的，没量过；一片可能比这个大，量出来的峰值只改这个常量。
/// `.claude/memory-bounded-parallelism.tsv` 里这个装置那一行只登记它准许按内存上限定并发与理由，不记峰值；任务服务给的内存上限
/// 只照服务自己库里按命令键的记录定，不读那张表。
const PEAK_BYTES_OF_ONE_LARGE_SLICE: u64 = 4 << 30;
/// 大片之外那一份常驻（主线程、P4 以外的片、进度线程）：4 GiB，推的、没量过（不跑 P4 的删除时整趟峰值约 2.3 GB）；
/// 量出来的数同样只改这个常量。
const RESIDENT_BYTES_BESIDES_THE_LARGE_SLICES: u64 = 4 << 30;

/// 一片的进度：开没开跑、跑完没有、判完的格数与删除发布数（只给进度行读，不进产物）。
#[derive(Default)]
pub(super) struct SliceProgress {
    is_started: AtomicBool,
    is_ended: AtomicBool,
    cells_judged: AtomicU64,
    delete_publishes: AtomicU64,
}

/// 这条线程在跑的那一片判完一格（不在片里不记）。
pub(super) fn count_a_judged_cell_on_this_slice() {
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| {
        if let Some(progress) = cell.borrow().as_ref() {
            progress.cells_judged.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// 这条线程在跑的那一片发完一次删除发布（不在片里不记）。
pub(super) fn count_a_delete_publish_on_this_slice() {
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| {
        if let Some(progress) = cell.borrow().as_ref() {
            progress.delete_publishes.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// 一片在调度上的样子：进度行里的区间名、算不算大片、开跑的先后（大的先跑；只排开跑次序，不排输出次序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SliceDescription {
    pub(super) label: String,
    pub(super) is_large: bool,
    pub(super) start_rank: u64,
}

/// 一片的结局：跑完（攒下的行与值）、停在停机条款（攒下的行与停机行）、panic（攒下的行与载荷）。
enum SliceEnd<Value> {
    Finished {
        lines: Vec<String>,
        value: Value,
    },
    Stopped {
        lines: Vec<String>,
        stop_line: String,
    },
    Panicked {
        lines: Vec<String>,
        payload: Box<dyn Any + Send>,
    },
}

/// 在这条线程上跑一片：开攒行、挂上这一片的进度计数，跑完、停机或 panic 都取回攒下的行。
fn run_one_slice_capturing_its_lines<Slice, Value>(
    slice: &Slice,
    progress: &Arc<SliceProgress>,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
) -> SliceEnd<Value> {
    LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD
        .with(|cell| *cell.borrow_mut() = Some(Arc::clone(progress)));
    progress.is_started.store(true, Ordering::SeqCst);
    let result = catch_unwind(AssertUnwindSafe(|| run_one_slice(slice)));
    let lines = LINES_OF_THE_SLICE_ON_THIS_THREAD
        .with(|cell| cell.borrow_mut().take())
        .expect("开跑时放进去的缓冲，跑的时候只有 emit_line 往里推、不取走");
    PROGRESS_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = None);
    progress.is_ended.store(true, Ordering::SeqCst);
    match result {
        Ok(value) => SliceEnd::Finished { lines, value },
        Err(payload) => match payload.downcast::<StopInsideASlice>() {
            Ok(stop) => SliceEnd::Stopped {
                lines,
                stop_line: stop.stop_line,
            },
            Err(payload) => SliceEnd::Panicked { lines, payload },
        },
    }
}

/// 调度的两个上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SliceScheduling {
    pub(super) thread_count: usize,
    pub(super) large_slice_thread_count: usize,
}

/// 一个工作线程池：它按次序取的片号与它的线程数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkerPool {
    positions_in_start_order: Vec<usize>,
    pub(super) thread_count: usize,
}

/// 把片分进线程池。线程数为 1：一个池、一条线程、按片的次序跑全部片（与改并行之前同一个次序）。否则大片与其余片各一个池：
/// 大片池的线程数不超过大片上限，也不吃光全部线程（还有别的片时至少给它们留一条）；池里按 `start_rank` 从大到小开跑，同值按片号。
fn worker_pools(descriptions: &[SliceDescription], scheduling: SliceScheduling) -> Vec<WorkerPool> {
    if scheduling.thread_count <= 1 {
        return vec![WorkerPool {
            positions_in_start_order: (0..descriptions.len()).collect(),
            thread_count: 1,
        }];
    }
    let positions_in_start_order_of = |wants_large: bool| -> Vec<usize> {
        let mut positions: Vec<usize> = (0..descriptions.len())
            .filter(|position| descriptions[*position].is_large == wants_large)
            .collect();
        positions.sort_by_key(|position| {
            (
                std::cmp::Reverse(descriptions[*position].start_rank),
                *position,
            )
        });
        positions
    };
    let large = positions_in_start_order_of(true);
    let small = positions_in_start_order_of(false);
    let large_cap = scheduling.large_slice_thread_count.max(1);
    let large_threads = match (large.is_empty(), small.is_empty()) {
        (true, _) => 0,
        (false, true) => large_cap.min(large.len()),
        (false, false) => large_cap.min(scheduling.thread_count - 1).min(large.len()),
    };
    let small_threads = (scheduling.thread_count - large_threads).min(small.len());
    [(large, large_threads), (small, small_threads)]
        .into_iter()
        .filter(|(positions, _)| !positions.is_empty())
        .map(|(positions_in_start_order, thread_count)| WorkerPool {
            positions_in_start_order,
            thread_count,
        })
        .collect()
}

/// 停机载荷的 unwind 不是 bug，不让默认的 panic 钩子往标准错误打「thread panicked」；别的 panic 照默认钩子打。装一次。
pub(super) fn silence_the_panic_hook_for_stops_inside_slices() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            if !information.payload().is::<StopInsideASlice>() {
                default_hook(information);
            }
        }));
    });
}

/// 进度行（标准错误，不进产物）：每片报区间、状态、已判的格数与删除发布数。
fn progress_line(
    position: usize,
    slice_count: usize,
    label: &str,
    progress: &SliceProgress,
) -> String {
    let state = match (
        progress.is_started.load(Ordering::SeqCst),
        progress.is_ended.load(Ordering::SeqCst),
    ) {
        (false, _) => "waiting",
        (true, false) => "running",
        (true, true) => "ended",
    };
    format!(
        "E168-progress slice={}/{slice_count} range={label} state={state} cells_judged={} delete_publishes={}",
        position + 1,
        progress.cells_judged.load(Ordering::Relaxed),
        progress.delete_publishes.load(Ordering::Relaxed)
    )
}

/// 按片并行跑 `slices`，每片的行攒着、按片的次序整片经 [`emit_line`] 打出（线程数不同输出逐字相同）；交回每片的值（按片的次序）。
/// 按片的次序第一个停机的片：打完它前面各片与它自己攒下的行，不再取新片，对它的停机行调 [`stop_with_the_line`]（主线程不攒行时
/// 打停机行、退出进程；攒行时带着停机行 unwind，等在跑的片跑完再出这个函数）。一片 panic：打完它之前的行再把 panic 原样抛出。
/// 每过 `progress_interval` 给每片调一次 `report_progress`。
fn run_slices_in_order<Slice: Sync, Value: Send>(
    slices: &[Slice],
    describe: &dyn Fn(&Slice) -> SliceDescription,
    scheduling: SliceScheduling,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
    report_progress: &(dyn Fn(&str) + Sync),
    progress_interval: Duration,
) -> Vec<Value> {
    silence_the_panic_hook_for_stops_inside_slices();
    let descriptions: Vec<SliceDescription> = slices.iter().map(describe).collect();
    let pools = worker_pools(&descriptions, scheduling);
    let progresses: Vec<Arc<SliceProgress>> = slices
        .iter()
        .map(|_| Arc::new(SliceProgress::default()))
        .collect();
    let ends: Mutex<Vec<Option<SliceEnd<Value>>>> =
        Mutex::new(slices.iter().map(|_| None).collect());
    let slice_ended = Condvar::new();
    let is_abandoned = AtomicBool::new(false);
    let is_merge_over = Mutex::new(false);
    let merge_over = Condvar::new();
    let (ends, slice_ended, is_abandoned, progresses, descriptions) = (
        &ends,
        &slice_ended,
        &is_abandoned,
        &progresses,
        &descriptions,
    );
    let (is_merge_over, merge_over) = (&is_merge_over, &merge_over);
    std::thread::scope(|scope| {
        for pool in &pools {
            let next_in_the_pool = Arc::new(AtomicUsize::new(0));
            for _ in 0..pool.thread_count {
                let next_in_the_pool = Arc::clone(&next_in_the_pool);
                scope.spawn(move || {
                    while !is_abandoned.load(Ordering::SeqCst) {
                        let taken = next_in_the_pool.fetch_add(1, Ordering::SeqCst);
                        let Some(&position) = pool.positions_in_start_order.get(taken) else {
                            break;
                        };
                        let end = run_one_slice_capturing_its_lines(
                            &slices[position],
                            &progresses[position],
                            run_one_slice,
                        );
                        ends.lock()
                            .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒")
                            [position] = Some(end);
                        slice_ended.notify_all();
                    }
                });
            }
        }
        scope.spawn(move || {
            let mut is_over = is_merge_over
                .lock()
                .expect("进度线程与主线程都不在持锁时 panic");
            loop {
                let (guard, _) = merge_over
                    .wait_timeout_while(is_over, progress_interval, |is_over| !*is_over)
                    .expect("进度线程与主线程都不在持锁时 panic");
                is_over = guard;
                if *is_over {
                    break;
                }
                for (position, progress) in progresses.iter().enumerate() {
                    report_progress(&progress_line(
                        position,
                        progresses.len(),
                        &descriptions[position].label,
                        progress,
                    ));
                }
            }
        });
        let merged = catch_unwind(AssertUnwindSafe(|| {
            let mut values = Vec::with_capacity(slices.len());
            for position in 0..slices.len() {
                let end = {
                    let mut ends_now = ends
                        .lock()
                        .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒");
                    loop {
                        if let Some(end) = ends_now[position].take() {
                            break end;
                        }
                        ends_now = slice_ended
                            .wait(ends_now)
                            .expect("工作线程的 panic 在 catch_unwind 里接住，锁不会中毒");
                    }
                };
                let (lines, ending) = match end {
                    SliceEnd::Finished { lines, value } => (lines, Ok(value)),
                    SliceEnd::Stopped { lines, stop_line } => (lines, Err(Ok(stop_line))),
                    SliceEnd::Panicked { lines, payload } => (lines, Err(Err(payload))),
                };
                for line in lines {
                    emit_line(line);
                }
                match ending {
                    Ok(value) => values.push(value),
                    Err(Ok(stop_line)) => {
                        is_abandoned.store(true, Ordering::SeqCst);
                        stop_with_the_line(stop_line);
                    }
                    Err(Err(payload)) => end_the_run_after_a_slice_panicked(payload),
                }
            }
            values
        }));
        is_abandoned.store(true, Ordering::SeqCst);
        *is_merge_over.lock().expect("进度线程不在持锁时 panic") = true;
        merge_over.notify_all();
        merged.unwrap_or_else(|payload| resume_unwind(payload))
    })
}

/// 一片 panic 之后：主线程在攒行（单测）时把 panic 原样抛出；不攒行（装置）时照 Rust panic 的退出码当场退，不等别的片跑完
/// （panic 的消息已由默认钩子在那一片的线程里打出；改并行之前单线程 panic 也是当场退）。
fn end_the_run_after_a_slice_panicked(payload: Box<dyn Any + Send>) -> ! {
    let is_capturing_lines = LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| cell.borrow().is_some());
    if is_capturing_lines {
        resume_unwind(payload);
    }
    std::process::exit(PANIC_EXIT_CODE)
}

/// 线程数：`E168_THREADS`，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，设了照它，再没设取
/// `.claude/heavy-thread-default` 的登记值；那一级在 `singlefs_verification::heavy_threads`，这里不另留一份）；设了而不是正整数就 panic。
fn thread_count_from(e168_threads: Option<&str>, thread_cap_value: Option<&str>) -> usize {
    match e168_threads {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|thread_count| *thread_count > 0)
            .unwrap_or_else(|| {
                panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，收到的是「{text}」")
            }),
        None => singlefs_verification::heavy_threads::default_heavy_threads_of_the_cap_value(
            thread_cap_value,
        )
        .get(),
    }
}

/// 内存上限从哪来（进度行报它）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MemoryCapSource {
    /// `E168_MEMORY_CAP`。
    Environment,
    /// 本进程控制组的 `memory.max`。
    ControlGroup,
    /// 两处都没有：[`DEFAULT_MEMORY_CAP_BYTES`]。
    Default,
}

impl MemoryCapSource {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::ControlGroup => "control_group",
            Self::Default => "default",
        }
    }
}

/// 给到的内存上限：字节数与从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MemoryCap {
    pub(super) bytes: u64,
    pub(super) source: MemoryCapSource,
}

/// `28G`、`8589934592` 这种写法的字节数：正整数加 K / M / G / T（1024 进），或不带单位的字节数；0、读不懂、乘出来装不下 u64 交 `None`。
fn bytes_of_a_memory_size_text(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    let units: [(char, u64); 4] = [
        ('K', 1 << 10),
        ('M', 1 << 20),
        ('G', 1 << 30),
        ('T', 1 << 40),
    ];
    let (digits, bytes_per_unit) = units
        .iter()
        .find_map(|(suffix, bytes_per_unit)| {
            trimmed
                .strip_suffix(*suffix)
                .map(|digits| (digits, *bytes_per_unit))
        })
        .unwrap_or((trimmed, 1));
    digits
        .parse::<u64>()
        .ok()
        .filter(|count| *count > 0)?
        .checked_mul(bytes_per_unit)
}

/// 内存上限的取法：`E168_MEMORY_CAP` 设了照它，没设取本进程控制组的 `memory.max`，那里是 `max` 或读不到时取 [`DEFAULT_MEMORY_CAP_BYTES`]。
///
/// # Panics
/// `E168_MEMORY_CAP` 设了而读不懂（不悄悄换成默认）。
fn memory_cap_given(
    environment_value: Option<&str>,
    control_group_memory_limit_text: Option<&str>,
) -> MemoryCap {
    if let Some(value) = environment_value {
        let bytes = bytes_of_a_memory_size_text(value).unwrap_or_else(|| {
            panic!("{MEMORY_CAP_ENVIRONMENT_VARIABLE} 要写成正整数加 K / M / G / T，或字节数，拿到「{value}」")
        });
        return MemoryCap {
            bytes,
            source: MemoryCapSource::Environment,
        };
    }
    match control_group_memory_limit_text.and_then(bytes_of_a_memory_size_text) {
        Some(bytes) => MemoryCap {
            bytes,
            source: MemoryCapSource::ControlGroup,
        },
        None => MemoryCap {
            bytes: DEFAULT_MEMORY_CAP_BYTES,
            source: MemoryCapSource::Default,
        },
    }
}

/// 本进程所在控制组（cgroup v2，`/proc/self/cgroup` 里 `0::` 那一行）的 `memory.max` 原文。
fn control_group_memory_limit_of_this_process() -> Option<String> {
    let membership = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?;
    std::fs::read_to_string(format!("/sys/fs/cgroup{group}/memory.max")).ok()
}

/// 大片同时跑几片：内存上限扣掉大片之外那一份常驻（[`RESIDENT_BYTES_BESIDES_THE_LARGE_SLICES`]），按一片的峰值（[`PEAK_BYTES_OF_ONE_LARGE_SLICE`]）
/// 往下取整；装不下一片也开一片（上限给小了照撞上限办，不悄悄不跑 P4）。
fn large_slice_threads_within_the_memory_cap(memory_cap_bytes: u64) -> usize {
    let large_slices_that_fit = memory_cap_bytes
        .saturating_sub(RESIDENT_BYTES_BESIDES_THE_LARGE_SLICES)
        / PEAK_BYTES_OF_ONE_LARGE_SLICE;
    usize::try_from(large_slices_that_fit.max(1)).unwrap_or(usize::MAX)
}

/// 读环境变量与控制组定调度，并往标准错误报一行（线程数不进产物：产物与线程数无关）。
pub(super) fn slice_scheduling_from_the_environment() -> SliceScheduling {
    let read = |variable: &str| std::env::var(variable).ok();
    let memory_cap = memory_cap_given(
        read(MEMORY_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        control_group_memory_limit_of_this_process().as_deref(),
    );
    let scheduling = SliceScheduling {
        thread_count: thread_count_from(
            read(THREADS_ENVIRONMENT_VARIABLE).as_deref(),
            read(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        ),
        large_slice_thread_count: large_slice_threads_within_the_memory_cap(memory_cap.bytes),
    };
    eprintln!(
        "E168-progress threads={} large_slice_threads={} memory_cap_bytes={} memory_cap_source={}",
        scheduling.thread_count,
        scheduling.large_slice_thread_count,
        memory_cap.bytes,
        memory_cap.source.name()
    );
    scheduling
}

/// 装置里的切片执行：产物行经 [`emit_line`] 打到标准输出，进度行写标准错误，停机就退 [`STOP_EXIT_CODE`]。交回每片的值（按片的次序）。
pub(super) fn run_slices_of_the_apparatus<Slice: Sync, Value: Send>(
    slices: &[Slice],
    describe: &dyn Fn(&Slice) -> SliceDescription,
    scheduling: SliceScheduling,
    run_one_slice: &(dyn Fn(&Slice) -> Value + Sync),
) -> Vec<Value> {
    run_slices_in_order(
        slices,
        describe,
        scheduling,
        run_one_slice,
        &|line: &str| eprintln!("{line}"),
        PROGRESS_INTERVAL,
    )
}

#[cfg(test)]
mod tests;
