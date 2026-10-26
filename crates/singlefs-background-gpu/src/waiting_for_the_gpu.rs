//! 阻塞等 GPU 时的上限（用户 2026-10-26 定等 GPU 结果「设超时，超时算出错」，记在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 第 40 批交回那一行；D26（后台整理与放置回收） 已定项 13 第 4 条）：每一处阻塞等 GPU 的地方（挂载那一刻等 wgpu 的 future、
//! 等一次提交做完、等回读缓冲映射好）都在同一个上限之内等，上限由调用方开包时给（`BackgroundGpuSelection::background_gpu_wait_limit`）。
//! 超过上限就交回 [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]：与别的 GPU 出错一样，这一批放弃、不重试、不换 CPU，
//! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。
//!
//! 等法与钟、完成信号拆开：钟是 [`MonotonicClock`]（产品用这个进程的单调钟 [`SteadyClockOfThisProcess`]），完成信号是调用方给的
//! 「看一次做没做完、最多等这么久」的闭包。不要 GPU 的用例换上假钟与假完成信号，钉住「上限内完成照常、超过上限交回超时出错」。

use std::time::{Duration, Instant};

use crate::background_gpu::GpuFailureCause;

/// 单调钟：读数只增不减，读的是从某个固定起点起过了多久。产品用 [`SteadyClockOfThisProcess`]；用例换成假钟。
pub trait MonotonicClock {
    fn time_since_the_origin(&self) -> Duration;
}

/// 这个进程的单调钟（`std::time::Instant`），起点是造它的那一刻。
#[derive(Clone, Copy, Debug)]
pub struct SteadyClockOfThisProcess {
    origin: Instant,
}

impl SteadyClockOfThisProcess {
    #[must_use]
    pub fn starting_now() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl MonotonicClock for SteadyClockOfThisProcess {
    fn time_since_the_origin(&self) -> Duration {
        self.origin.elapsed()
    }
}

/// 一次等的期限：从哪一刻起等、最多等多久。一次提交的几处等（等提交做完、等两份回读映射好）共用一个期限，合起来不超过上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuWaitDeadline {
    started_at: Duration,
    wait_limit: Duration,
}

impl GpuWaitDeadline {
    /// 从钟此刻的读数起等，最多等 `wait_limit`。
    #[must_use]
    pub fn starting_now(wait_limit: Duration, clock: &impl MonotonicClock) -> Self {
        Self {
            started_at: clock.time_since_the_origin(),
            wait_limit,
        }
    }

    #[must_use]
    pub fn wait_limit(&self) -> Duration {
        self.wait_limit
    }

    /// 离期限还剩多久；已经等满上限（剩 0 或钟走过了期限）交回 `None`。
    #[must_use]
    pub fn time_left_before_the_deadline(&self, clock: &impl MonotonicClock) -> Option<Duration> {
        let waited = clock
            .time_since_the_origin()
            .saturating_sub(self.started_at);
        self.wait_limit
            .checked_sub(waited)
            .filter(|time_left| !time_left.is_zero())
    }
}

/// 看一次做没做完的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionCheck<Output> {
    Completed(Output),
    /// 这一次没等到（最多等了交给它的那么久，或提前醒了）：还在期限内就再看一次。
    NotYetComplete,
    /// 等的那一样自己报了错（不是超时）：照原样交回。
    Failed(GpuFailureCause),
}

/// 在期限之内反复看 `check_once_waiting_at_most`（参数是这一次最多等多久，即离期限还剩的那么久），直到做完、报错或等满上限。
/// 每一轮先看还剩多久：剩 0 就交回超时，不再看；所以一次都还没看就已经过了期限的（上限是 0）也交回超时。
///
/// 循环的出口只有三个：做完、报错、等满上限。迭代次数没有静态上界：每一轮要么等到了、要么至少等了交给它的那么久或提前醒了，
/// 钟单调往前走，期限总会到。
///
/// # Errors
/// 等满上限 ⇒ [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]（带上限本身）；`check_once_waiting_at_most` 交回的出错照原样交回。
pub fn wait_for_completion_before_the_deadline<Output>(
    deadline: &GpuWaitDeadline,
    clock: &impl MonotonicClock,
    mut check_once_waiting_at_most: impl FnMut(Duration) -> CompletionCheck<Output>,
) -> Result<Output, GpuFailureCause> {
    loop {
        let Some(time_left) = deadline.time_left_before_the_deadline(clock) else {
            return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: deadline.wait_limit,
            });
        };
        match check_once_waiting_at_most(time_left) {
            CompletionCheck::Completed(output) => return Ok(output),
            CompletionCheck::NotYetComplete => {}
            CompletionCheck::Failed(cause) => return Err(cause),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use crate::two_kernel_reconciliation::{
        ConsecutiveUncertaintyCounts, TwoKernelOutcomesOfAnEntry,
    };

    /// 假钟：只在完成信号让它走时才走。
    struct ClockMovedByTheTest {
        now: Cell<Duration>,
    }

    impl MonotonicClock for ClockMovedByTheTest {
        fn time_since_the_origin(&self) -> Duration {
            self.now.get()
        }
    }

    /// 假完成信号：每看一次假钟走一秒、最多走到期限（这一次等了一秒没等到），钟走到 `completes_at` 那一刻算做完、交回 42。
    /// 交给它的最多等多久是 0 时（被测的等法不该这样交）也让钟走 1 纳秒：等法写错时用例照样走得完、在断言上红，不在原地转圈。
    fn completion_at<'clock>(
        clock: &'clock ClockMovedByTheTest,
        completes_at: Duration,
        checks_made: &'clock Cell<u32>,
    ) -> impl FnMut(Duration) -> CompletionCheck<u32> + 'clock {
        move |time_left| {
            checks_made.set(checks_made.get() + 1);
            let waited_this_check = Duration::from_secs(1)
                .min(time_left)
                .max(Duration::from_nanos(1));
            clock.now.set(clock.now.get() + waited_this_check);
            if clock.now.get() >= completes_at {
                CompletionCheck::Completed(42)
            } else {
                CompletionCheck::NotYetComplete
            }
        }
    }

    #[test]
    fn a_completion_within_the_wait_limit_returns_its_output() {
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::from_secs(100)),
        };
        let checks_made = Cell::new(0);
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(5), &clock);
        assert_eq!(
            wait_for_completion_before_the_deadline(
                &deadline,
                &clock,
                completion_at(&clock, Duration::from_secs(103), &checks_made),
            ),
            Ok(42),
            "等了 3 秒做完，上限 5 秒：照常交回"
        );
        assert_eq!(checks_made.get(), 3, "做完就不再看");
    }

    #[test]
    fn a_completion_later_than_the_wait_limit_is_a_timeout_failure() {
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::from_secs(100)),
        };
        let checks_made = Cell::new(0);
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(5), &clock);
        assert_eq!(
            wait_for_completion_before_the_deadline(
                &deadline,
                &clock,
                completion_at(&clock, Duration::from_secs(107), &checks_made),
            ),
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(5)
            }),
            "要 7 秒才做完，上限 5 秒：交回超时出错"
        );
        assert_eq!(checks_made.get(), 5, "等满 5 秒就不再看");
        assert_eq!(
            clock.now.get(),
            Duration::from_secs(105),
            "每次最多等到期限为止"
        );
    }

    #[test]
    fn a_zero_wait_limit_times_out_before_checking() {
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::ZERO),
        };
        let checks_made = Cell::new(0);
        let deadline = GpuWaitDeadline::starting_now(Duration::ZERO, &clock);
        assert_eq!(
            wait_for_completion_before_the_deadline(
                &deadline,
                &clock,
                completion_at(&clock, Duration::ZERO, &checks_made),
            ),
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::ZERO
            })
        );
        assert_eq!(checks_made.get(), 0);
    }

    #[test]
    fn a_failure_reported_by_the_wait_is_returned_as_it_is() {
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::ZERO),
        };
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(5), &clock);
        let failure = GpuFailureCause::WaitForTheSubmissionFailed {
            reason: "测试给的".to_string(),
        };
        assert_eq!(
            wait_for_completion_before_the_deadline(&deadline, &clock, |_time_left| {
                CompletionCheck::<u32>::Failed(failure.clone())
            }),
            Err(failure)
        );
    }

    /// 超时那一批走的是与 GPU 出错同一条路：交回出错，连续不确定计数一条都不动，CPU 参照一次都不调。
    #[test]
    fn a_batch_whose_wait_times_out_leaves_the_consecutive_uncertainty_counts_unchanged() {
        let mut uncertainty: ConsecutiveUncertaintyCounts<u32> =
            ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let disagreeing_round = uncertainty.reconcile_unless_the_batch_failed(
            Ok::<_, GpuFailureCause>(vec![TwoKernelOutcomesOfAnEntry {
                entry_key: 1,
                outcome_by_one_kernel: 7_u32,
                outcome_by_the_independently_written_kernel: 8,
            }]),
            |_entry_position| panic!("只不确定一轮时不交 CPU 判"),
        );
        assert!(disagreeing_round.is_ok());
        assert_eq!(uncertainty.consecutive_uncertain_rounds_of(&1), 1);

        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::ZERO),
        };
        let checks_made = Cell::new(0);
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(2), &clock);
        let timed_out_outcomes = wait_for_completion_before_the_deadline(
            &deadline,
            &clock,
            completion_at(&clock, Duration::from_secs(9), &checks_made),
        )
        .map(|_completed| {
            vec![TwoKernelOutcomesOfAnEntry {
                entry_key: 1,
                outcome_by_one_kernel: 7_u32,
                outcome_by_the_independently_written_kernel: 8,
            }]
        });
        let timed_out_round = uncertainty
            .reconcile_unless_the_batch_failed(timed_out_outcomes, |_entry_position| {
                panic!("超时那一批不交 CPU 判")
            });
        assert_eq!(
            timed_out_round,
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(2)
            }),
            "超时算出错"
        );
        assert_eq!(
            uncertainty.consecutive_uncertain_rounds_of(&1),
            1,
            "超时那一批放弃了，不算一轮：计数不加也不清零"
        );
    }
}
