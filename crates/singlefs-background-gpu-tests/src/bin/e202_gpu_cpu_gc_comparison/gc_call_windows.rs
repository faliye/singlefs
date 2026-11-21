//! 一次 GC 调用的计时窗口：挂钟、进程与 GC 线程的 CPU、调用方与量挂载的节奏。

use super::*;

// ---- 一次 GC 调用的计时窗口（登记第一节「GC 挂钟」「进程口径」「GC 线程口径」，第五节「计时与计数怎么做」）----

/// 窗口一端的一次读数：单调钟、进程 CPU 钟、GC 线程自己的 CPU 钟、前台线程与主线程各自的 CPU 钟，
/// 与这一端读到的本进程每条线程的 schedstat（线程号 → CPU；拿两端比，认出窗口里退出的线程）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CallWindowReading {
    pub(super) wall: Duration,
    pub(super) process_cpu: Duration,
    pub(super) gc_thread_cpu: Duration,
    pub(super) foreground_thread_cpu: Duration,
    pub(super) main_thread_cpu: Duration,
    cpu_by_task: BTreeMap<u64, Duration>,
}

/// 窗口两端的读数从哪来：产品是本进程的钟，测试换成假的。
pub(super) trait CallWindowClocks {
    fn read(&self, end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed>;
}

/// 读的是窗口的哪一端：读 `/proc` 那几个系统调用的开销放在窗口外（开窗那一端先读 `/proc` 再读钟，关窗那一端先读钟再读 `/proc`）；
/// 两样钟里挂钟在外、GC 线程的 CPU 钟在里（开窗先读挂钟再读 CPU 钟，关窗先读 CPU 钟再读挂钟），挂钟的窗口罩住 CPU 钟的窗口，
/// 包含自检「挂钟 ≥ GC 线程 CPU」比的才是同一段时间（读 CPU 钟那一次系统调用的开销落在挂钟里，不落在挂钟外）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EndOfTheWindow {
    Opening,
    Closing,
}

/// GC 线程上读的真钟：挂钟从 `started` 起算（`CLOCK_MONOTONIC_RAW`）；GC 线程自己的 CPU 读线程 CPU 钟，进程口径是它加上本进程别的线程各自的 schedstat，
/// 前台线程与主线程按它们各自交来的线程号读（登记第一节「进程口径」：窗口里进程 CPU 减前台线程与主线程的增量）。
pub(super) struct ClocksOfTheGcThread {
    pub(super) started: StartOnTheMonotonicRawClock,
    /// 读不到 GC 线程自己的线程号时是 `None`：这次跑每个窗口都读不成，整次作废。
    pub(super) gc_thread_task: Option<u64>,
    pub(super) foreground_thread_task: u64,
    pub(super) main_thread_task: u64,
}

impl CallWindowClocks for ClocksOfTheGcThread {
    fn read(&self, end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed> {
        let gc_thread_task = self.gc_thread_task.ok_or(ClockReadFailed {
            reading: "GC 线程的线程号",
        })?;
        let (wall, gc_thread_cpu, cpu_by_task) = match end {
            EndOfTheWindow::Opening => {
                let cpu_by_task = cpu_of_every_task_of_this_process()?;
                let wall = self.started.elapsed();
                (wall, host_cpu_time_of_this_thread(), cpu_by_task)
            }
            EndOfTheWindow::Closing => {
                let gc_thread_cpu = host_cpu_time_of_this_thread();
                let wall = self.started.elapsed();
                (wall, gc_thread_cpu, cpu_of_every_task_of_this_process()?)
            }
        };
        Ok(CallWindowReading {
            wall,
            process_cpu: gc_thread_cpu
                + cpu_by_task
                    .iter()
                    .filter(|(task, _cpu)| **task != gc_thread_task)
                    .map(|(_task, cpu)| *cpu)
                    .sum::<Duration>(),
            gc_thread_cpu,
            foreground_thread_cpu: *cpu_by_task.get(&self.foreground_thread_task).ok_or(
                ClockReadFailed {
                    reading: "前台线程的 schedstat",
                },
            )?,
            main_thread_cpu: *cpu_by_task
                .get(&self.main_thread_task)
                .ok_or(ClockReadFailed {
                    reading: "主线程的 schedstat",
                })?,
            cpu_by_task,
        })
    }
}

/// 窗口两端各读一次的每条线程的 schedstat 里，有没有开窗时在、关窗时不在的线程（窗口里退出了），或关窗时比开窗时少的线程
/// （线程号让窗口里新起的线程重用了）。有就是进程口径读不全：退出的线程到开窗为止的 CPU 在关窗那一端读不到，两端之和相减少掉它，
/// 少得多了进程口径比 GC 线程口径还小（登记第十二节修订第 1 条：这一窗照第十一节作废第 1 条作废）。
fn any_task_read_at_the_opening_is_missing_or_behind_at_the_closing(
    opening: &BTreeMap<u64, Duration>,
    closing: &BTreeMap<u64, Duration>,
) -> bool {
    opening.iter().any(|(task, opening_cpu)| {
        closing
            .get(task)
            .is_none_or(|closing_cpu| closing_cpu < opening_cpu)
    })
}

/// 一次 GC 调用的窗口：挂钟、进程口径（进程 CPU 减前台线程与主线程的增量）、GC 线程口径与两条减掉的增量。
/// 任一增量是负的、或窗口里有线程退出的为 `None`（作废第 1 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallWindow {
    pub(super) wall: Duration,
    pub(super) process_cpu: Duration,
    pub(super) gc_thread_cpu: Duration,
    pub(super) foreground_thread_cpu: Duration,
    pub(super) main_thread_cpu: Duration,
}

/// 两端读数之差（第九节变异「进程口径只读 GC 线程钟」改的是 `process_cpu` 那一算）。
fn call_window_between(before: CallWindowReading, after: CallWindowReading) -> Option<CallWindow> {
    if any_task_read_at_the_opening_is_missing_or_behind_at_the_closing(
        &before.cpu_by_task,
        &after.cpu_by_task,
    ) {
        return None;
    }
    let foreground_thread_cpu = after
        .foreground_thread_cpu
        .checked_sub(before.foreground_thread_cpu)?;
    let main_thread_cpu = after.main_thread_cpu.checked_sub(before.main_thread_cpu)?;
    let process_cpu = after
        .process_cpu
        .checked_sub(before.process_cpu)?
        .checked_sub(foreground_thread_cpu)?
        .checked_sub(main_thread_cpu)?;
    Some(CallWindow {
        wall: after.wall.checked_sub(before.wall)?,
        process_cpu,
        gc_thread_cpu: after.gc_thread_cpu.checked_sub(before.gc_thread_cpu)?,
        foreground_thread_cpu,
        main_thread_cpu,
    })
}

/// 窗口量出的一次调用：调用交回的、窗口（钟读错、增量为负、里层计时读不到都是 `None`）、窗口之内多判那一段的里层计时与再判结果。
pub(super) struct MeasuredCall<Outcome> {
    pub(super) outcome: Outcome,
    pub(super) window: Option<CallWindow>,
    pub(super) inner_rejudge_cpu: Duration,
    pub(super) rejudge_tally: RejudgeTally,
}

/// 量一次 GC 调用：窗口一端读钟 → 调用 → 窗口之内的多判（CPU 多判臂；别的臂什么都不做）→ 窗口另一端读钟
/// （第九节变异「CPU 多判放到计时窗口外」改的是这里的次序）。
pub(super) fn measure_one_gc_call<State, Outcome>(
    clocks: &impl CallWindowClocks,
    state: &mut State,
    call: impl FnOnce(&mut State) -> Outcome,
    rejudge_inside_the_window: impl FnOnce(&mut State, &Outcome) -> (Option<Duration>, RejudgeTally),
) -> MeasuredCall<Outcome> {
    let before = clocks.read(EndOfTheWindow::Opening);
    let outcome = call(state);
    let (inner_rejudge_cpu, rejudge_tally) = rejudge_inside_the_window(state, &outcome);
    let after = clocks.read(EndOfTheWindow::Closing);
    let window = match (before, after, inner_rejudge_cpu) {
        (Ok(before), Ok(after), Some(_inner)) => call_window_between(before, after),
        (Err(_), _, _) | (_, Err(_), _) | (_, _, None) => None,
    };
    MeasuredCall {
        outcome,
        window,
        inner_rejudge_cpu: inner_rejudge_cpu.unwrap_or(Duration::ZERO),
        rejudge_tally,
    }
}

/// 交替跑里调用方的假钟与前台：每次调用前进 1 毫秒（定时兜底每次都到点），前台恒答没有 fsync 在等（登记第五节「交替跑」）。
pub(super) struct CallerOfTheLockstepRun {
    pub(super) reading: Cell<Duration>,
}

impl CallerOfTheLockstepRun {
    pub(super) fn starting_at_zero() -> CallerOfTheLockstepRun {
        CallerOfTheLockstepRun {
            reading: Cell::new(Duration::ZERO),
        }
    }

    /// 下一次调用之前把钟往前拨一步（第九节变异「交替跑的假钟不前进」改的是这里）。
    pub(super) fn advance_before_the_next_call(&self) {
        self.reading
            .set(self.reading.get() + LOCKSTEP_FAKE_CLOCK_STEP_PER_CALL);
    }
}

impl CallerOfTheBackgroundCompaction for CallerOfTheLockstepRun {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.reading.get())
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 后台入口的节奏：定时兜底每 1 毫秒、写带宽不限（登记第五节「量挂载」）。
pub(super) fn pacing_of_the_measuring_mount() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(TIMER_FALLBACK_MILLISECONDS).expect("1 毫秒不是 0"),
        ),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

/// 量挂载装上的整理只供测试的开关：批大小 `batch_units`，起跑强制开着，其余照产品（同产品 GPU 包测试共用模块的写法）。
pub(super) fn compaction_switches_of_the_measuring_mount(batch_units: usize) -> CompactionSwitches {
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
        ),
        forced_region,
        run_forcing: CompactionRunForcing::ForcedToStartForTests,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

#[cfg(test)]
mod tests;
