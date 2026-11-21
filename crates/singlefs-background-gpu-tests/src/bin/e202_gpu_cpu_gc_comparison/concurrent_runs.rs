//! 并发跑的一次跑：前台延迟、GC 线程让出、一次跑的汇总与甲类锚点。

use super::*;

// ---- 并发跑（登记第五节「两种跑法」的并发跑，补问题单第 3 行；量 F、量 F 底，第七节甲类第 5 行，第十条第 4 行）----

/// 并发跑一次的用户改动次数（登记第五节「前台那串用户改动」）。
pub(super) const CONCURRENT_USER_CHANGES: u64 = 256;
/// 前台计划时刻的间隔：主取样点与几何敏感性那一点（登记第五节「并发跑」、第八节）。
pub(super) const FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_MAIN_SAMPLING_POINT: u64 = 50;
pub(super) const FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT: u64 = 500;
/// GC 线程一次调用一次整理发布都没发时，在条件变量上至多等这么久（登记第五节「并发跑」）。
const CONCURRENT_GC_IDLE_WAIT: Duration = Duration::from_millis(1);
/// 第十条第 4 行：一次用户改动「拿到锁 − 置上标志」不许超过（最长一批的挂钟 + 最长一次 discard）的这么多倍。
const FLAG_TO_LOCK_WAIT_PER_LONGEST_BATCH_AND_DISCARD_AT_MOST: u32 = 2;
/// 量 F 报的高分位：第 99 百分位，按最近秩取（排好序的第 ⌈99 × n ÷ 100⌉ 个，从 1 数）。
const HIGH_PERCENTILE_PER_HUNDRED: usize = 99;

/// 并发跑的三条臂（登记第五节「真实基线」：CPU 臂、GPU 点名一张卡臂与不跑 GC 臂）。封闭集合，次序是轮换的基准次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum ConcurrentArm {
    Cpu,
    GpuOnTheCardWithTheLowestPciAddress,
    /// GC 线程不起，同一串前台改动（量 F 底）。挂载照 CPU 臂的设置。
    WithoutCompaction,
}

pub(super) const CONCURRENT_ARMS: [ConcurrentArm; 3] = [
    ConcurrentArm::Cpu,
    ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
    ConcurrentArm::WithoutCompaction,
];

impl ConcurrentArm {
    pub(super) const fn name(self) -> &'static str {
        match self {
            ConcurrentArm::Cpu => "cpu",
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => "gpu_one_card",
            ConcurrentArm::WithoutCompaction => "without_compaction",
        }
    }

    pub(super) fn of_the_name(name: &str) -> Option<ConcurrentArm> {
        CONCURRENT_ARMS.into_iter().find(|arm| arm.name() == name)
    }

    /// 挂载用哪一种后台计算（不跑 GC 臂照 CPU 臂挂）。
    pub(super) const fn compute_of_the_arm(self) -> ComputeOfTheArm {
        match self {
            ConcurrentArm::Cpu | ConcurrentArm::WithoutCompaction => ComputeOfTheArm::Cpu,
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => {
                ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress
            }
        }
    }

    const fn runs_the_compaction(self) -> bool {
        match self {
            ConcurrentArm::Cpu | ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => true,
            ConcurrentArm::WithoutCompaction => false,
        }
    }

    /// 开卡与多判包装照交替跑那一份的臂（并发跑不多判）。
    const fn as_a_lockstep_arm(self) -> LockstepArm {
        LockstepArm {
            compute: self.compute_of_the_arm(),
            rejudging: RejudgingOfTheArm::NotRejudged,
        }
    }
}

/// 一次用户改动的四个时刻，都从这次跑开跑那一刻起算（登记第五节「计时与计数」第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UserChangeTimings {
    planned: Duration,
    flag_set: Duration,
    lock_acquired: Duration,
    publish_returned: Duration,
}

/// 量 F：延迟 = 发布返回 − 计划时刻（开环，落后了照算，登记第一节「前台 fsync」；第九节变异「延迟从拿到锁算起」改的是这里）。
/// 读出负的交 `None`（作废第 1 条）。
fn latency_of_the_user_change(timings: UserChangeTimings) -> Option<Duration> {
    timings.publish_returned.checked_sub(timings.planned)
}

/// 锁里发布用了多久 = 发布返回 − 拿到锁。
fn publish_under_the_lock_of_the_user_change(timings: UserChangeTimings) -> Option<Duration> {
    timings.publish_returned.checked_sub(timings.lock_acquired)
}

/// 第十条第 4 行的读数：拿到锁 − 置上标志。
fn wait_from_the_flag_to_the_lock(timings: UserChangeTimings) -> Option<Duration> {
    timings.lock_acquired.checked_sub(timings.flag_set)
}

/// 最近秩的高分位（第 99 百分位）：`sorted_values` 从小到大排好、至少一个。
fn high_percentile_of_the_sorted(sorted_values: &[f64]) -> f64 {
    let rank = (HIGH_PERCENTILE_PER_HUNDRED * sorted_values.len()).div_ceil(100);
    sorted_values[rank.max(1) - 1]
}

/// 一份延迟分布：条数、中位、第 99 百分位、最大（登记第六节量 F）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LatencyDistribution {
    count: usize,
    pub(super) median: f64,
    pub(super) high_percentile: f64,
    pub(super) maximum: f64,
}

/// 从一组延迟的分布里取一个统计量（第 99 百分位、最大值）。
pub(super) type StatisticOfALatencyDistribution = fn(LatencyDistribution) -> f64;

pub(super) fn latency_distribution_of(
    values_in_nanoseconds: &[f64],
) -> Option<LatencyDistribution> {
    if values_in_nanoseconds.is_empty() {
        return None;
    }
    let mut sorted_values = values_in_nanoseconds.to_vec();
    sorted_values.sort_by(f64::total_cmp);
    Some(LatencyDistribution {
        count: sorted_values.len(),
        median: median_of(&sorted_values),
        high_percentile: high_percentile_of_the_sorted(&sorted_values),
        maximum: maximum_of(&sorted_values),
    })
}

/// `name=` 行里一份分布的四个字段（没有数时四个都写 `none`）。
pub(super) fn distribution_fields(name: &str, distribution: Option<LatencyDistribution>) -> String {
    match distribution {
        Some(distribution) => format!(
            "{name}_count={} {name}_median={:.0} {name}_ninety_ninth_percentile={:.0} {name}_maximum={:.0}",
            distribution.count,
            distribution.median,
            distribution.high_percentile,
            distribution.maximum
        ),
        None => format!(
            "{name}_count=0 {name}_median=none {name}_ninety_ninth_percentile=none {name}_maximum=none"
        ),
    }
}

/// 并发跑里调用方的钟与前台：钟是真的单调钟（`CLOCK_MONOTONIC_RAW`，从开跑起算），`answer_before_the_next_batch` 读前台置的那个标志，每问一次记下问的时刻与答的（窗口外读）。
struct CallerOfTheConcurrentRun<'run> {
    started: StartOnTheMonotonicRawClock,
    fsync_is_waiting: &'run AtomicBool,
    queries_of_the_call: RefCell<Vec<(Duration, AnswerOfTheCallerBeforeTheNextBatch)>>,
}

impl CallerOfTheBackgroundCompaction for CallerOfTheConcurrentRun<'_> {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.started.elapsed())
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        let answer = if self.fsync_is_waiting.load(Ordering::SeqCst) {
            AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
        } else {
            AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
        };
        self.queries_of_the_call
            .borrow_mut()
            .push((self.started.elapsed(), answer));
        answer
    }
}

/// 并发跑里一次 GC 调用记下的：整理记账（同交替跑那一份），调用开始与返回的时刻，这次调用里每一次问前台的时刻与答的。
struct ConcurrentCallRecord {
    call: GcCallRecord,
    started_at: Duration,
    returned_at: Duration,
    fsync_queries: Vec<(Duration, AnswerOfTheCallerBeforeTheNextBatch)>,
}

/// 第七节甲类第 5 行：答了「在等」的那次调用只答了这一次、它是这次调用问的最后一次（答了之后不再发批），停机原因是让路。
fn the_call_yielded_after_answering_waiting(call: &ConcurrentCallRecord) -> bool {
    let waiting_answers = call
        .fsync_queries
        .iter()
        .filter(|(_at, answer)| {
            *answer == AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
        })
        .count();
    match waiting_answers {
        0 => true,
        1 => {
            call.fsync_queries.last().is_some_and(|(_at, answer)| {
                *answer == AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
            }) && call.call.stop_name == CompactionRunStop::YieldedToAForegroundFsync.name()
        }
        _more_than_one => false,
    }
}

/// 甲类第 5 行不成立的调用数。
fn calls_not_yielding_after_answering_waiting(calls: &[ConcurrentCallRecord]) -> u64 {
    let not_yielding = calls
        .iter()
        .filter(|call| !the_call_yielded_after_answering_waiting(call))
        .count();
    u64::try_from(not_yielding).expect("调用数")
}

/// 第八节轨迹「前台『在等』标志被答成『在等』的次数」：一次调用里的峰值、答成「在等」的调用次数、期末值（最后一次调用的）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WaitingAnswersTrajectory {
    peak_in_one_call: u64,
    calls_answered_waiting: u64,
    in_the_last_call: u64,
}

fn waiting_answers_trajectory(calls: &[ConcurrentCallRecord]) -> WaitingAnswersTrajectory {
    let answers_of_each_call: Vec<u64> = calls
        .iter()
        .map(|call| {
            let waiting = call
                .fsync_queries
                .iter()
                .filter(|(_at, answer)| {
                    *answer == AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
                })
                .count();
            u64::try_from(waiting).expect("次数")
        })
        .collect();
    WaitingAnswersTrajectory {
        peak_in_one_call: answers_of_each_call.iter().copied().max().unwrap_or(0),
        calls_answered_waiting: u64::try_from(
            answers_of_each_call
                .iter()
                .filter(|answers| **answers > 0)
                .count(),
        )
        .expect("调用数"),
        in_the_last_call: answers_of_each_call.last().copied().unwrap_or(0),
    }
}

/// 第十条第 4 行的上界取法：核心层每一批之前问一次前台，一次调用里相邻两次「问」之间（连同调用开始到头一次问）的那一段至多是一批
/// （连同选段与签发）的挂钟；最后一次问到调用返回（一次都没问的调用是整次调用）那一段含调用末尾的 discard。交回（最长一批，最长收尾）。
fn longest_batch_and_longest_tail_of_the_calls(
    calls: &[ConcurrentCallRecord],
) -> (Duration, Duration) {
    let mut longest_batch = Duration::ZERO;
    let mut longest_tail = Duration::ZERO;
    for call in calls {
        let mut previous = call.started_at;
        for (asked_at, _answer) in &call.fsync_queries {
            longest_batch = longest_batch.max(asked_at.saturating_sub(previous));
            previous = *asked_at;
        }
        longest_tail = longest_tail.max(call.returned_at.saturating_sub(previous));
    }
    (longest_batch, longest_tail)
}

/// 第十条第 4 行：前台等锁比（最长一批 + 最长收尾）的两倍还久（条款与实现对不上，停机条款第 4 条）。
fn the_wait_exceeds_twice_the_longest_batch_and_tail(
    longest_wait_from_the_flag_to_the_lock: Duration,
    longest_batch: Duration,
    longest_tail: Duration,
) -> bool {
    longest_wait_from_the_flag_to_the_lock
        > (longest_batch + longest_tail) * FLAG_TO_LOCK_WAIT_PER_LONGEST_BATCH_AND_DISCARD_AT_MOST
}

/// 第十条第 4 行这一次跑的结论（登记第十二节修订第 6 条）：最长一批与最长收尾取的是上界（相邻两次问前台之间、最后一次问到返回），
/// 判得比登记口径宽；打中照登记判，没打中只说「上界口径下没打中」，不说登记口径下没打中。
pub(super) fn yield_bound_failure_clause_reading(
    the_foreground_waited_past_the_yield_bound: bool,
) -> &'static str {
    if the_foreground_waited_past_the_yield_bound {
        "hit"
    } else {
        "not_hit_under_the_upper_bound_reading"
    }
}

/// 并发跑 GC 线程放锁之后让一下（登记第十二节修订第 7 条）；产物头一行照它写 `gc_thread_yields_after_unlock=`。
pub(super) const GC_THREAD_YIELDS_AFTER_UNLOCK: bool = true;

/// 一次并发跑记下的（窗口外读）。
struct ConcurrentRunRecords {
    calls: Vec<ConcurrentCallRecord>,
    forced_fallback: ForcedFallbackReading,
    timings: Vec<UserChangeTimings>,
    the_opening_thread_was_running_at_the_first_call: bool,
    state: SessionAndDevices,
    write_generation_of_each_unit: Vec<u64>,
}

/// 一次并发跑的节奏：几次用户改动、隔多久一次、起不起 GC 线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcurrentRunPacing {
    user_changes: u64,
    foreground_interval: Duration,
    runs_the_compaction: bool,
}

/// 并发跑一次（登记第五节「两种跑法」）：前台线程照计划时刻发用户改动，发之前置上「有 fsync 在等」、拿到会话锁、发布返回之后撤下；
/// GC 线程循环拿会话锁调一次后台入口、放锁，这次调用一次整理发布都没发就在条件变量上等下一次用户改动发成或 1 毫秒，前台做完就停。
/// 会话与两块盘放在一把锁里，前台与 GC 不同时动它们。
fn concurrent_threads(
    session_and_devices_at_the_start: SessionAndDevices,
    lowest_live_data_inode: u64,
    write_generation_of_each_unit_at_the_start: Vec<u64>,
    first_write_time_seconds: u64,
    run_pacing: ConcurrentRunPacing,
    forced_fallback_of_the_run: ForcedFallbackOfTheRun,
) -> ConcurrentRunRecords {
    let session_and_devices = Mutex::new(session_and_devices_at_the_start);
    let fsync_is_waiting = AtomicBool::new(false);
    let foreground_is_done = AtomicBool::new(false);
    let user_changes_published = Mutex::new(0_u64);
    let user_change_published_condition = Condvar::new();
    // 前台的计划时刻、置标志、拿到锁、发布返回与 GC 调用的开始、返回、问前台都读这一个起点：跨度挂钟与第十条第 4 行拿它们互相比。
    let started = StartOnTheMonotonicRawClock::now();
    let (
        (calls, the_opening_thread_was_running_at_the_first_call, forced_fallback),
        timings,
        write_generation_of_each_unit,
    ) = std::thread::scope(|scope| {
        let foreground = scope.spawn(|| {
            let mut write_generation_of_each_unit = write_generation_of_each_unit_at_the_start;
            let mut timings: Vec<UserChangeTimings> = Vec::new();
            for change_number in 0..run_pacing.user_changes {
                let planned = run_pacing.foreground_interval
                    * u32::try_from(change_number).expect("用户改动次数装得进 u32");
                if let Some(ahead) = planned.checked_sub(started.elapsed()) {
                    std::thread::sleep(ahead);
                }
                fsync_is_waiting.store(true, Ordering::SeqCst);
                let flag_set = started.elapsed();
                let mut guard = session_and_devices.lock().expect("会话锁没在别处 panic 过");
                let lock_acquired = started.elapsed();
                let state = &mut *guard;
                publish_the_user_change(
                    &mut state.session,
                    &mut state.devices,
                    lowest_live_data_inode,
                    change_number,
                    &mut write_generation_of_each_unit,
                    first_write_time_seconds + change_number,
                );
                let publish_returned = started.elapsed();
                fsync_is_waiting.store(false, Ordering::SeqCst);
                drop(guard);
                *user_changes_published
                    .lock()
                    .expect("计数锁没在别处 panic 过") += 1;
                user_change_published_condition.notify_all();
                timings.push(UserChangeTimings {
                    planned,
                    flag_set,
                    lock_acquired,
                    publish_returned,
                });
            }
            foreground_is_done.store(true, Ordering::SeqCst);
            user_change_published_condition.notify_all();
            (timings, write_generation_of_each_unit)
        });
        let gc = run_pacing.runs_the_compaction.then(|| {
            scope.spawn(|| {
                // 强制选段后备（登记第十二节修订第 8 条）：点名在拿着会话锁、调后台入口之前做，点名用的挂钟另报（`forced_choice_wall_nanoseconds`）。
                let mut forced_fallback = forced_fallback_of_the_run;
                let caller = CallerOfTheConcurrentRun {
                    started,
                    fsync_is_waiting: &fsync_is_waiting,
                    queries_of_the_call: RefCell::new(Vec::new()),
                };
                let pacing = pacing_of_the_measuring_mount();
                let mut calls: Vec<ConcurrentCallRecord> = Vec::new();
                let mut the_opening_thread_was_running_at_the_first_call = false;
                while !foreground_is_done.load(Ordering::SeqCst) {
                    let published_before_the_call = *user_changes_published
                        .lock()
                        .expect("计数锁没在别处 panic 过");
                    if calls.is_empty() {
                        the_opening_thread_was_running_at_the_first_call =
                            the_gpu_opening_thread_is_still_running().unwrap_or(true);
                    }
                    let mut guard = session_and_devices.lock().expect("会话锁没在别处 panic 过");
                    let state = &mut *guard;
                    let empty_segments_before_the_call =
                        empty_segments_of_every_device(&state.session);
                    let intents_issued_before_the_call =
                        forced_fallback.before_the_call(&mut state.session, &state.devices);
                    let started_at = started.elapsed();
                    let call = state
                        .session
                        .run_compaction_in_the_background(&mut state.devices, pacing, &caller)
                        .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒");
                    let returned_at = started.elapsed();
                    forced_fallback.after_the_call(
                        &state.session,
                        intents_issued_before_the_call,
                        &call,
                    );
                    let empty_segments_after_the_call =
                        empty_segments_of_every_device(&state.session);
                    drop(guard);
                    // 放锁之后让一下（登记第十二节修订第 7 条）：标准库的锁不排队，刚放锁的这条线程马上再拿常常抢在等着的前台前面；
                    // 让一下不改登记的循环，只少记一截不是核心层造成的等待。产物头一行照同一个常量写 `gc_thread_yields_after_unlock=`。
                    if GC_THREAD_YIELDS_AFTER_UNLOCK {
                        std::thread::yield_now();
                    }
                    let mut record = record_of_the_call(
                        &call,
                        published_before_the_call,
                        None,
                        Duration::ZERO,
                        RejudgeTally::default(),
                    );
                    record.empty_segments_before_the_call = empty_segments_before_the_call;
                    record.empty_segments_after_the_call = empty_segments_after_the_call;
                    let published_nothing = the_call_published_nothing(&record);
                    calls.push(ConcurrentCallRecord {
                        call: record,
                        started_at,
                        returned_at,
                        fsync_queries: caller.queries_of_the_call.take(),
                    });
                    if published_nothing {
                        let published_count_guard = user_changes_published
                            .lock()
                            .expect("计数锁没在别处 panic 过");
                        let _still_the_same = user_change_published_condition
                            .wait_timeout_while(
                                published_count_guard,
                                CONCURRENT_GC_IDLE_WAIT,
                                |published| {
                                    *published == published_before_the_call
                                        && !foreground_is_done.load(Ordering::SeqCst)
                                },
                            )
                            .expect("计数锁没在别处 panic 过");
                    }
                }
                (
                    calls,
                    the_opening_thread_was_running_at_the_first_call,
                    forced_fallback.reading,
                )
            })
        });
        let gc_thread_records = gc
            .map(|handle| handle.join().expect("GC 线程没有 panic"))
            .unwrap_or_default();
        let (timings, write_generation_of_each_unit) =
            foreground.join().expect("前台线程没有 panic");
        (gc_thread_records, timings, write_generation_of_each_unit)
    });
    ConcurrentRunRecords {
        calls,
        forced_fallback,
        timings,
        the_opening_thread_was_running_at_the_first_call,
        state: session_and_devices
            .into_inner()
            .expect("会话锁没在别处 panic 过"),
        write_generation_of_each_unit,
    }
}

/// 第七节乙类最后一行（包含关系）：每次用户改动的延迟 ≥ 锁里发布用的时间。交回不成立（或读出负的）的改动数（作废第 1、2 条）。
fn user_changes_failing_the_inclusion_check(timings: &[UserChangeTimings]) -> u64 {
    let failing = timings
        .iter()
        .filter(|timings| {
            match (
                latency_of_the_user_change(**timings),
                publish_under_the_lock_of_the_user_change(**timings),
                wait_from_the_flag_to_the_lock(**timings),
            ) {
                (Some(latency), Some(publish_under_the_lock), Some(_wait)) => {
                    latency < publish_under_the_lock
                }
                (None, _, _) | (_, None, _) | (_, _, None) => true,
            }
        })
        .count();
    u64::try_from(failing).expect("改动数")
}

/// 并发跑一次的甲类锚点（第七节甲类第 1、2、5 行与第十条第 4 行）：每一项记账照设置走、条数不少于批数；GPU 臂分到活的卡与切片那几样；
/// 答了「在等」的调用都让了路；前台等锁没超过（最长一批 + 最长收尾）的两倍。
fn class_one_anchors_of_a_concurrent_run_hold(
    path_anchor: PathAnchor,
    compute: ComputeOfTheArm,
    gpu_work_anchors: GpuWorkAnchors,
    calls_not_yielding_after_answering_waiting: u64,
    the_foreground_waited_past_the_yield_bound: bool,
) -> bool {
    path_anchor.holds()
        && (!compute.uses_the_gpu() || gpu_work_anchors.hold())
        && calls_not_yielding_after_answering_waiting == 0
        && !the_foreground_waited_past_the_yield_bound
}

/// 一次并发跑的结果：`name=concurrent_run` 一行的数、锚点与作废原因。
pub(super) struct ConcurrentRunSummary {
    pub(super) arm: ConcurrentArm,
    pub(super) repeat: u32,
    gc_calls: u64,
    gc_calls_started: u64,
    pub(super) batches: u64,
    units_relocated: u64,
    latencies: Vec<f64>,
    publishes_under_the_lock: Vec<f64>,
    longest_wait_from_the_flag_to_the_lock: Duration,
    longest_batch: Duration,
    longest_tail: Duration,
    the_foreground_waited_past_the_yield_bound: bool,
    calls_not_yielding_after_answering_waiting: u64,
    waiting_answers: WaitingAnswersTrajectory,
    empty_segments_trajectory: Vec<EmptySegmentsTrajectoryOfADevice>,
    pub(super) forced_fallback: ForcedFallbackReading,
    user_changes_failing_the_inclusion_check: u64,
    pub(super) content_matches_the_expectation: bool,
    pub(super) host_other_cores: f64,
    path_anchor: PathAnchor,
    pub(super) gpu_work_anchors: GpuWorkAnchors,
    gpus_leaked: u64,
    stops: BTreeMap<String, u64>,
    pub(super) void_reason: Option<String>,
}

impl ConcurrentRunSummary {
    pub(super) fn class_one_anchors_hold(&self) -> bool {
        class_one_anchors_of_a_concurrent_run_hold(
            self.path_anchor,
            self.arm.compute_of_the_arm(),
            self.gpu_work_anchors,
            self.calls_not_yielding_after_answering_waiting,
            self.the_foreground_waited_past_the_yield_bound,
        )
    }

    pub(super) fn fields(
        &self,
        tier: UnitAreaTier,
        session_number: u64,
        foreground_interval: Duration,
    ) -> String {
        let stops: Vec<String> = self
            .stops
            .iter()
            .map(|(stop, count)| format!("{stop}:{count}"))
            .collect();
        format!(
            "tier={} arm={} repeat={} session={session_number} foreground_interval_milliseconds={} user_changes={} \
             gc_calls={} gc_calls_started={} batches={} units_relocated={} {} {} latencies_nanoseconds={} \
             publishes_under_the_lock_nanoseconds={} longest_wait_from_the_flag_to_the_lock_nanoseconds={} \
             longest_batch_upper_bound_nanoseconds={} longest_tail_upper_bound_nanoseconds={} \
             yield_bound_reading=upper_bound yield_bound_failure_clause={} \
             calls_not_yielding_after_answering_waiting={} waiting_answers_peak_in_one_call={} calls_answered_waiting={} \
             waiting_answers_in_the_last_call={} {} user_changes_failing_the_inclusion_check={} \
             content_matches_the_expectation={} host_other_cores={:.2} compute_records={} \
             compute_records_on_the_path_of_the_settings={} path_anchor_holds={} gpu_records={} \
             segments_given_to_a_card_other_than_the_lowest={} recuts_after_running_out_of_memory={} \
             segments_with_pieces_other_than_submissions={} uncertain_entries={} entries_judged_by_the_cpu_reference={} \
             batches_abandoned_after_a_compute_error={} class_one_anchors_hold={} gpus_leaked={} {} stops={} void={}",
            tier.name(),
            self.arm.name(),
            self.repeat,
            foreground_interval.as_millis(),
            self.latencies.len(),
            self.gc_calls,
            self.gc_calls_started,
            self.batches,
            self.units_relocated,
            distribution_fields("latency_nanoseconds", latency_distribution_of(&self.latencies)),
            distribution_fields(
                "publish_under_the_lock_nanoseconds",
                latency_distribution_of(&self.publishes_under_the_lock)
            ),
            joined_values(&self.latencies),
            joined_values(&self.publishes_under_the_lock),
            self.longest_wait_from_the_flag_to_the_lock.as_nanos(),
            self.longest_batch.as_nanos(),
            self.longest_tail.as_nanos(),
            yield_bound_failure_clause_reading(self.the_foreground_waited_past_the_yield_bound),
            self.calls_not_yielding_after_answering_waiting,
            self.waiting_answers.peak_in_one_call,
            self.waiting_answers.calls_answered_waiting,
            self.waiting_answers.in_the_last_call,
            empty_segments_fields(&self.empty_segments_trajectory),
            self.user_changes_failing_the_inclusion_check,
            self.content_matches_the_expectation,
            self.host_other_cores,
            self.path_anchor.compute_records,
            self.path_anchor.compute_records_on_the_path_of_the_settings,
            self.path_anchor.holds(),
            self.gpu_work_anchors.gpu_records,
            self.gpu_work_anchors.segments_given_to_a_card_other_than_the_lowest,
            self.gpu_work_anchors.recuts_after_running_out_of_memory,
            self.gpu_work_anchors.segments_with_pieces_other_than_submissions,
            self.gpu_work_anchors.uncertain_entries,
            self.gpu_work_anchors.entries_judged_by_the_cpu_reference,
            self.gpu_work_anchors.batches_abandoned_after_a_compute_error,
            self.class_one_anchors_hold(),
            self.gpus_leaked,
            self.forced_fallback.fields(),
            field_value(&stops.join(",")),
            field_value(self.void_reason.as_deref().unwrap_or("no")),
        )
    }
}

/// 一次并发跑的设定。
pub(super) struct ConcurrentRunSetting<'run> {
    pub(super) image: &'run StartingImage,
    pub(super) discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
    pub(super) memory_leaked: &'run GpuMemoryLeakedInThisProcess,
    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
    pub(super) vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
    pub(super) user_changes: u64,
    pub(super) foreground_interval: Duration,
}

/// 并发跑一次：拷池、开卡、挂载（批大小取主取样点）、两条线程并发、读内容、核锚点、记机器负载。开卡或挂载不成交回 `Err`（交主 agent）。
pub(super) fn run_concurrent_once(
    setting: &ConcurrentRunSetting<'_>,
    arm: ConcurrentArm,
    repeat: u32,
) -> Result<ConcurrentRunSummary, String> {
    let host_at_the_start = host_load_reading()?;
    let settings =
        background_compute_settings_of_the_arm(arm.compute_of_the_arm(), setting.discrete_gpus);
    let gpu = open_the_gpu_of_the_arm(
        arm.as_a_lockstep_arm(),
        &settings,
        setting.memory_leaked,
        setting.vulkan_instance,
    )?;
    let (session, devices, _recording) = measuring_mount(
        setting.image,
        settings,
        gpu.gpu_compute,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    let ConcurrentRunRecords {
        calls,
        forced_fallback,
        timings,
        the_opening_thread_was_running_at_the_first_call,
        state,
        write_generation_of_each_unit,
    } = concurrent_threads(
        SessionAndDevices { session, devices },
        setting.image.lowest_live_data_inode,
        setting.image.write_generation_of_each_unit.clone(),
        setting.image.next_write_time_seconds,
        ConcurrentRunPacing {
            user_changes: setting.user_changes,
            foreground_interval: setting.foreground_interval,
            runs_the_compaction: arm.runs_the_compaction(),
        },
        ForcedFallbackOfTheRun::of_the_image(setting.image),
    );
    let content_digest = read_content_digest(
        &state.devices,
        version_of_trees(&state.session),
        setting.image.lowest_live_data_inode,
        u64::try_from(write_generation_of_each_unit.len()).expect("P"),
    )?;
    let content_matches_the_expectation =
        content_digest == expected_content_digest(&write_generation_of_each_unit);
    drop(state);
    let gpus_leaked = match gpu
        .gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        Some(leaked) => {
            setting
                .memory_leaked
                .note_the_gpus_leaked_when_dropped(&leaked);
            u64::try_from(leaked.len()).expect("卡数")
        }
        None => 0,
    };
    let latencies: Vec<f64> = timings
        .iter()
        .filter_map(|timings| latency_of_the_user_change(*timings))
        .map(nanoseconds_as_float)
        .collect();
    let publishes_under_the_lock: Vec<f64> = timings
        .iter()
        .filter_map(|timings| publish_under_the_lock_of_the_user_change(*timings))
        .map(nanoseconds_as_float)
        .collect();
    let longest_wait_from_the_flag_to_the_lock = timings
        .iter()
        .filter_map(|timings| wait_from_the_flag_to_the_lock(*timings))
        .max()
        .unwrap_or(Duration::ZERO);
    let (longest_batch, longest_tail) = longest_batch_and_longest_tail_of_the_calls(&calls);
    let the_foreground_waited_past_the_yield_bound = arm.runs_the_compaction()
        && the_wait_exceeds_twice_the_longest_batch_and_tail(
            longest_wait_from_the_flag_to_the_lock,
            longest_batch,
            longest_tail,
        );
    let user_changes_failing_the_inclusion_check =
        user_changes_failing_the_inclusion_check(&timings);
    let calls_not_yielding_after_answering_waiting =
        calls_not_yielding_after_answering_waiting(&calls);
    let waiting_answers = waiting_answers_trajectory(&calls);
    let records: Vec<GcCallRecord> = calls.into_iter().map(|call| call.call).collect();
    let empty_segments_trajectory = empty_segments_trajectory_of_each_device(&records);
    let lowest_card = setting.discrete_gpus.first().map(|lowest_discrete_gpu| {
        gpu_device_identifier_of_the_pci_bus_address(&lowest_discrete_gpu.pci_bus_address)
    });
    let gpu_work_anchors = lowest_card
        .as_ref()
        .map_or_else(GpuWorkAnchors::default, |lowest_card| {
            gpu_work_anchors(&records, lowest_card)
        });
    let void_reason = if user_changes_failing_the_inclusion_check > 0 {
        Some(
            "a_latency_shorter_than_its_publish_under_the_lock_or_a_clock_went_backwards"
                .to_string(),
        )
    } else if the_opening_thread_was_running_at_the_first_call {
        Some("the_gpu_opening_thread_was_running_at_the_first_call".to_string())
    } else {
        None
    };
    let mut stops: BTreeMap<String, u64> = BTreeMap::new();
    for record in &records {
        *stops.entry(record.stop_name.clone()).or_insert(0) += 1;
    }
    let host_at_the_end = host_load_reading()?;
    Ok(ConcurrentRunSummary {
        arm,
        repeat,
        gc_calls: u64::try_from(records.len()).expect("调用数"),
        gc_calls_started: u64::try_from(records.iter().filter(|record| record.started).count())
            .expect("调用数"),
        batches: records.iter().map(|record| record.batches).sum(),
        units_relocated: records
            .iter()
            .flat_map(|record| record.units_relocated_of_each_batch.iter())
            .sum(),
        latencies,
        publishes_under_the_lock,
        longest_wait_from_the_flag_to_the_lock,
        longest_batch,
        longest_tail,
        the_foreground_waited_past_the_yield_bound,
        calls_not_yielding_after_answering_waiting,
        waiting_answers,
        empty_segments_trajectory,
        forced_fallback,
        user_changes_failing_the_inclusion_check,
        content_matches_the_expectation,
        host_other_cores: cores_used_outside_this_process(host_at_the_start, host_at_the_end),
        path_anchor: path_anchor(&records, arm.compute_of_the_arm()),
        gpu_work_anchors,
        gpus_leaked,
        stops,
        void_reason,
    })
}

/// 并发跑判定用的数：这一次跑刚量出的，或续跑时从产物的 `name=concurrent_run` 行读回的。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ConcurrentRunNumbers {
    pub(super) arm: ConcurrentArm,
    pub(super) repeat: u32,
    pub(super) latencies: Vec<f64>,
    pub(super) publishes_under_the_lock: Vec<f64>,
}

/// `joined_values` 写出的一串纳秒数读回来；空串是一个都没有。
fn values_of_a_joined_field(joined: &str) -> Option<Vec<f64>> {
    if joined.is_empty() {
        return Some(Vec::new());
    }
    joined
        .split(',')
        .map(|value| value.parse::<f64>().ok())
        .collect()
}

impl ConcurrentRunNumbers {
    pub(super) fn of_the_summary(summary: &ConcurrentRunSummary) -> ConcurrentRunNumbers {
        ConcurrentRunNumbers {
            arm: summary.arm,
            repeat: summary.repeat,
            latencies: summary.latencies.clone(),
            publishes_under_the_lock: summary.publishes_under_the_lock.clone(),
        }
    }

    pub(super) fn of_the_fields(fields: &BTreeMap<String, String>) -> Option<ConcurrentRunNumbers> {
        Some(ConcurrentRunNumbers {
            arm: ConcurrentArm::of_the_name(fields.get("arm")?)?,
            repeat: fields.get("repeat")?.parse().ok()?,
            latencies: values_of_a_joined_field(fields.get("latencies_nanoseconds")?)?,
            publishes_under_the_lock: values_of_a_joined_field(
                fields.get("publishes_under_the_lock_nanoseconds")?,
            )?,
        })
    }
}

/// 一条臂按重复次序排好的那几次跑。
pub(super) fn concurrent_runs_of_the_arm(
    numbers: &[ConcurrentRunNumbers],
    arm: ConcurrentArm,
) -> Vec<&ConcurrentRunNumbers> {
    let mut of_the_arm: Vec<&ConcurrentRunNumbers> =
        numbers.iter().filter(|run| run.arm == arm).collect();
    of_the_arm.sort_by_key(|run| run.repeat);
    of_the_arm
}

#[cfg(test)]
mod tests;
