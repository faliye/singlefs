/// 层 0 枚举的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。门禁 54 号显式传进来。
pub const LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_THREADS";

/// 默认切法的片数下限：线程数为 1 时全量用例也按片报进度。
const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;
/// 默认切法里每个工作线程摊到的片数：越往后的状态历史越长、越贵，片切得比线程多，先跑完的线程接着领下一片。
const LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16;
/// 默认切法里每片的状态数下限：平时 `cargo test` 里几十个状态的枚举只切成几片，进度行不刷屏。
const LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16;

/// 工作线程数是从哪来的：门禁 54 号据此判「没显式设成 1、机器多于 1 核却只用了 1 个线程」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0WorkerThreadsSource {
    /// 环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    EnvironmentVariable,
    /// 没设环境变量，取 `available_parallelism`。
    AvailableParallelism,
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同切法对拍）。
    GivenByCaller,
}

impl Layer0WorkerThreadsSource {
    fn name(self) -> &'static str {
        match self {
            Self::EnvironmentVariable => "environment_variable",
            Self::AvailableParallelism => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller => "given_by_caller",
        }
    }
}

/// 每片几个状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0SliceLength {
    /// 片数取 max(64, 16 × 工作线程数)，每片至少 16 个状态。
    ScaledToWorkerThreads,
    /// 每片固定这么多个状态（用例把切法推到两头：每片 1 个，或整条流 1 片）。
    StatesPerSlice(NonZeroU64),
}

/// 层 0 按状态序号区间切片、多线程跑：几个工作线程、每片几个状态。切法与线程数只影响跑得多快，不影响计数与「第一处违例」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer0Parallelism {
    pub worker_threads: NonZeroUsize,
    pub worker_threads_source: Layer0WorkerThreadsSource,
    pub slice_length: Layer0SliceLength,
}

impl Layer0Parallelism {
    /// 线程数取环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`]，没设就取 `available_parallelism`；片长按线程数定。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_environment() -> Self {
        let (worker_threads, worker_threads_source) =
            match std::env::var(LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE) {
                Ok(text) => (
                    text.parse::<NonZeroUsize>().unwrap_or_else(|error| {
                        panic!(
                            "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}：{error}"
                        )
                    }),
                    Layer0WorkerThreadsSource::EnvironmentVariable,
                ),
                Err(std::env::VarError::NotPresent) => match std::thread::available_parallelism() {
                    Ok(available) => (available, Layer0WorkerThreadsSource::AvailableParallelism),
                    Err(_unavailable) => (
                        NonZeroUsize::MIN,
                        Layer0WorkerThreadsSource::AvailableParallelismUnknown,
                    ),
                },
                Err(std::env::VarError::NotUnicode(raw)) => panic!(
                    "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到的不是 UTF-8：{raw:?}"
                ),
            };
        Self {
            worker_threads,
            worker_threads_source,
            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
        }
    }
}

/// 枚举次序里每个状态的序号怎么落到「第几段、段内哪个子集」：次序与单线程逐段逐个子集走相同——
/// 前面的段全持久 + 当前段任意真子集（子集掩码从 0 数到 2^|段| − 2），最后再加全部持久那一个状态。
struct Layer0StatePlan<'segments> {
    segments: &'segments [Vec<usize>],
    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
    state_ranges_by_segment: Vec<Range<u64>>,
    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
    state_count: u64,
}

impl<'segments> Layer0StatePlan<'segments> {
    /// `expand` 只在调用线程上逐段问一次，所以它不必能跨线程。
    fn new(
        segments: &'segments [Vec<usize>],
        expand: &dyn Fn(usize, &[usize]) -> bool,
    ) -> Self {
        let mut next_ordinal = 0u64;
        let state_ranges_by_segment = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| {
                let first_ordinal = next_ordinal;
                if expand(segment_index, segment) {
                    next_ordinal += (1u64 << segment.len()) - 1;
                }
                first_ordinal..next_ordinal
            })
            .collect();
        Self {
            segments,
            state_ranges_by_segment,
            state_count: next_ordinal + 1,
        }
    }

    /// 序号落在哪一段；等于段数说明是最后全部持久那一个状态。
    fn segment_of_state(&self, ordinal: u64) -> usize {
        self.state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }

    /// 这个状态里持久了的写：所在的段之前每一段整段持久（展不展开都一样），所在的段按段内子集掩码（第 k 位对应段里第 k 个写）。
    fn persisted_writes_of_state(&self, ordinal: u64, write_count: usize) -> Vec<bool> {
        let segment_of_state = self.segment_of_state(ordinal);
        let mut persisted = vec![false; write_count];
        for segment in &self.segments[..segment_of_state] {
            for write_index in segment {
                persisted[*write_index] = true;
            }
        }
        if let Some(segment) = self.segments.get(segment_of_state) {
            let subset_mask = ordinal - self.state_ranges_by_segment[segment_of_state].start;
            for (bit, write_index) in segment.iter().enumerate() {
                if subset_mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
        }
        persisted
    }

    /// 进度行里的段号：最后全部持久那一个状态不在任何一段里，报成 `all_persisted`。
    fn segment_label(&self, segment_index: usize) -> String {
        if segment_index < self.segments.len() {
            segment_index.to_string()
        } else {
            "all_persisted".to_string()
        }
    }
}

/// 把 [0, `state_count`) 按序号切成首尾相接的区间。
fn state_slices(state_count: u64, parallelism: &Layer0Parallelism) -> Vec<Range<u64>> {
    let states_per_slice = match parallelism.slice_length {
        Layer0SliceLength::ScaledToWorkerThreads => {
            let worker_threads =
                u64::try_from(parallelism.worker_threads.get()).expect("线程数装得进 u64");
            let slice_count = LAYER0_MINIMUM_SLICE_COUNT
                .max(worker_threads.saturating_mul(LAYER0_SLICES_PER_WORKER_THREAD));
            state_count
                .div_ceil(slice_count)
                .max(LAYER0_MINIMUM_STATES_PER_SLICE)
        }
        Layer0SliceLength::StatesPerSlice(states_per_slice) => states_per_slice.get(),
    };
    (0..state_count.div_ceil(states_per_slice))
        .map(|slice_index| {
            let first_ordinal = slice_index * states_per_slice;
            first_ordinal..first_ordinal.saturating_add(states_per_slice).min(state_count)
        })
        .collect()
}

/// 工作线程要不要把每个状态的持久集合与看 journal 那一遍恢复的报告带回调用线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateReportRetention {
    /// 有观察者：带回去，由调用线程按序号交给它。
    HandEachStateToObserver,
    /// 没有观察者：只带计数回去（全量流上两百多万个报告不必留）。
    CountOnly,
}

/// 一片跑完交回调用线程的东西。
struct FinishedSlice {
    slice_index: usize,
    tally: Layer0Tally,
    /// 按序号排好的（持久集合，看 journal 那一遍恢复的报告）；`CountOnly` 时是空的。
    observed_states: Vec<(Vec<bool>, RecoveryReport)>,
}

/// 在工作线程上跑一片：每个状态自己建崩溃镜像、自己记进这一片的计数；基线与写表只读、各线程共用。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：基线、写表、状态计划、这一片的区间、被判的根、版本表、要不要带回报告"
)]
fn evaluate_state_slice(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    plan: &Layer0StatePlan<'_>,
    slice_index: usize,
    slice: Range<u64>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    retention: StateReportRetention,
) -> FinishedSlice {
    let mut tally = Layer0Tally::default();
    let mut observed_states = Vec::new();
    for ordinal in slice {
        let persisted = plan.persisted_writes_of_state(ordinal, writes.len());
        match retention {
            StateReportRetention::HandEachStateToObserver => {
                let consulted_report = evaluate_state_for_versions(
                    base,
                    writes,
                    persisted.clone(),
                    judged_root_index,
                    versions,
                    &mut tally,
                );
                observed_states.push((persisted, consulted_report));
            }
            StateReportRetention::CountOnly => {
                evaluate_state_for_versions(
                    base,
                    writes,
                    persisted,
                    judged_root_index,
                    versions,
                    &mut tally,
                );
            }
        }
    }
    FinishedSlice {
        slice_index,
        tally,
        observed_states,
    }
}

/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
/// 按状态序号区间切片、多线程跑，线程数见 [`Layer0Parallelism::from_environment`]。
#[must_use]
pub fn enumerate_layer0_selecting_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expand,
        Layer0Parallelism::from_environment(),
        None,
    )
}

/// 同上，每个状态评完之后把这个崩溃镜像与看 journal 那一遍恢复的报告交给 `observe_state`：
/// 用例按自己独立算的谓词逐状态核恢复（预置的残留记录该不该施加、改坏 tail 之后终态与没改坏的是否逐项相等）。
/// 观察者在调用线程上、按状态序号从小到大调用，次序与单线程逐个跑相同，所以它不必能跨线程。
#[must_use]
pub fn enumerate_layer0_selecting_versions_observing_each_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    observe_state: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expand,
        Layer0Parallelism::from_environment(),
        Some(observe_state),
    )
}

/// 层 0 枚举的本体：把 [0, 状态数) 按 `parallelism` 切成首尾相接的序号区间，工作线程按片号从小到大领片、各自跑完交回；
/// 调用线程收到一片就打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、已跑完的片数与状态数），再按片号从小到大并计数、调观察者。
/// 计数按片的次序相加，「第一处违例」取序号最小的那一处（[`Layer0Tally::absorb_following_slice`]），结果与线程数、切法无关。
/// 开跑与跑完各打一行 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（状态数、片数、实际起的工作线程数、线程数从哪来、耗时）。
///
/// # Panics
/// 某个工作线程在评状态时 panic（恢复或 checker 里的断言）；观察者 panic；有一片领了却没交回（不变量被破坏）。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个与单线程时的枚举相同，多出来的是切法与观察者"
)]
#[must_use]
pub fn enumerate_layer0_in_state_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    parallelism: Layer0Parallelism,
    mut observe_state: Option<&mut dyn FnMut(&CrashImage<'_>, &RecoveryReport)>,
) -> Layer0Tally {
    let plan = Layer0StatePlan::new(segments, expand);
    let slices = state_slices(plan.state_count, &parallelism);
    let spawned_worker_threads = parallelism.worker_threads.get().min(slices.len());
    let retention = match observe_state {
        Some(_) => StateReportRetention::HandEachStateToObserver,
        None => StateReportRetention::CountOnly,
    };
    let started = Instant::now();
    println!(
        "LAYER0_PARALLEL_START states={} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        plan.state_count,
        slices.len(),
        slices.first().map_or(0, |slice| slice.end - slice.start),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slice_sender, finished_slice_receiver) = mpsc::channel::<FinishedSlice>();
    let (tally, merged_slice_count) = std::thread::scope(|scope| {
        for _ in 0..spawned_worker_threads {
            let finished_slice_sender = finished_slice_sender.clone();
            let plan = &plan;
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let finished = evaluate_state_slice(
                    base,
                    writes,
                    plan,
                    slice_index,
                    slice.clone(),
                    judged_root_index,
                    versions,
                    retention,
                );
                // 发不出去说明调用线程已经不收了（观察者 panic）：不再领新的片。
                if finished_slice_sender.send(finished).is_err() {
                    break;
                }
            });
        }
        // 只留工作线程手里的发送端：它们都退出之后，下面的接收循环才结束。
        drop(finished_slice_sender);
        let mut merged = Layer0Tally::default();
        let mut waiting_for_earlier_slices: BTreeMap<usize, FinishedSlice> = BTreeMap::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_states = 0u64;
        for (finished_slice_count, finished) in finished_slice_receiver.iter().enumerate() {
            let slice = &slices[finished.slice_index];
            finished_states += slice.end - slice.start;
            println!(
                "LAYER0_PROGRESS slice={}/{} states=[{},{}) segments={}..={} finished_slices={}/{} finished_states={finished_states}/{} elapsed_seconds={:.1}",
                finished.slice_index + 1,
                slices.len(),
                slice.start,
                slice.end,
                plan.segment_label(plan.segment_of_state(slice.start)),
                plan.segment_label(plan.segment_of_state(slice.end - 1)),
                finished_slice_count + 1,
                slices.len(),
                plan.state_count,
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(finished.slice_index, finished);
            while let Some(in_order) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                if let Some(observe) = observe_state.as_mut() {
                    for (persisted, consulted_report) in in_order.observed_states {
                        observe(
                            &CrashImage {
                                base,
                                writes,
                                persisted,
                            },
                            &consulted_report,
                        );
                    }
                }
                merged.absorb_following_slice(in_order.tally);
                next_slice_to_merge += 1;
            }
        }
        (merged, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "LAYER0_PARALLEL_FINISHED states={} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} elapsed_seconds={:.1}",
        plan.state_count,
        slices.len(),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name(),
        started.elapsed().as_secs_f64()
    );
    tally
}

