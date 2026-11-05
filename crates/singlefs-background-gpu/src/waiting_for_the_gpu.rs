//! 阻塞等 GPU 时的上限（用户 2026-10-26 定等 GPU 结果「设超时，超时算出错」，记在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 第 40 批交回那一行；D26（后台整理与放置回收） 已定项 13 第 4 条）：每一处阻塞等 GPU 的地方（挂载那一刻等 wgpu 的 future、
//! 等一次提交做完、等回读缓冲映射好）都在同一个上限之内等，上限由调用方开包时给（`BackgroundGpuSelection::background_gpu_wait_limit`）。
//! 超过上限就交回 [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]。
//!
//! 等超过上限的那一次提交在同一张卡上重交（[`outcome_resubmitted_after_timeouts`]；用户 2026-10-28 定
//! 「重试是把那一批重新交一次」：放弃的那一批在同一张卡上重交，最多 3 次，都超时才报警，原话在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 末尾「用户定案（弹窗两次）」那一行；用户 2026-10-30 定重交的单位照旧只是超时的那一片）。超时按批、按卡数（[`TimeoutsOfTheBatchOnOneGpu`]）：
//! 一批切成几片时各片的超时合起来算，交活之前等前一次超时的那次提交那一等（[`outcome_waited_for_again_after_timeouts`]）也算；
//! 重交之前先等前一次没做完的那次（照显存的账，等不到算又超时了一次）；任何一次做完了照常交回；
//! 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次超时就交回 [`GpuFailureCause::UnusableAfterResubmissions`]（这张卡用不了，带最后一次的原因），这一批放弃、不换 CPU，
//! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。别的出错不重交。
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

/// 一批里一张卡等超过上限之后在同一张卡上至多重交几次（用户 2026-10-28 定「最多 3 次，都超时才报警」）：这一批在这张卡上第
/// 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限就报这张卡用不了。
pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;

/// 是不是等超过上限的那两种：这一次提交自己等超过上限，或交活之前、重交之前等前一次那一次又等满了上限。只有这两种重交、算进一批的超时次数。
fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. } => true,
        GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. } => false,
    }
}

/// 一批里一张卡上到此刻等超过上限了几次、重交了几次（重交只数真交出去的：重交之前那一等又等满了上限、没交的那一回只算超时，
/// 重交那一次建缓冲显存不足、没交出去的那一回不算）（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
/// 一批切成几片、几段时各片的超时合起来算，交活之前等前一次超时的那次提交那一等也算）。一批开头每张卡各一份，从 0 起。
/// 外壳的「一批」是三段各自交给 GPU 的那一次（`BackgroundGpu` 的三段方法各调一次是一批）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeoutsOfTheBatchOnOneGpu {
    timeouts: u32,
    resubmissions: u32,
}

impl TimeoutsOfTheBatchOnOneGpu {
    /// 一批开头：一次都还没超时。
    #[must_use]
    pub fn none_yet() -> Self {
        Self {
            timeouts: 0,
            resubmissions: 0,
        }
    }

    #[must_use]
    pub fn timeouts(&self) -> u32 {
        self.timeouts
    }

    #[must_use]
    pub fn resubmissions(&self) -> u32 {
        self.resubmissions
    }

    /// 记下一次等超过上限的（`cause` 是等超过上限的那两种之一）：这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次就交回这张卡用不了。
    ///
    /// # Errors
    /// 超时次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] ⇒ [`GpuFailureCause::UnusableAfterResubmissions`]，带这一次的原因。
    fn note_a_wait_past_the_limit(
        &mut self,
        cause: GpuFailureCause,
    ) -> Result<(), GpuFailureCause> {
        self.timeouts += 1;
        if self.timeouts > RESUBMISSIONS_AFTER_A_TIMEOUT {
            // 报这一批这张卡真交出去的重交次数（判决 m4-gpugc-code-r3 第三节第 6 条）：重交之前那一等等满了上限、
            // 重交那一次建缓冲显存不足没交出去的那几回不算，所以它可以比 RESUBMISSIONS_AFTER_A_TIMEOUT 少。
            return Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: self.resubmissions,
                last_cause: Box::new(cause),
            });
        }
        Ok(())
    }
}

/// 一次交的结局说不说明那一次真交到了卡上：建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）是在交之前判出的、那一段没交；
/// 别的出错都是交了之后才判出的（等提交、回读、读回的判定字），做完了的当然交了。只给交一段那一次的结局用（`submit_once` 交回的）。
fn the_attempt_reached_the_gpu<Outcome>(attempt: &Result<Outcome, GpuFailureCause>) -> bool {
    match attempt {
        Ok(_outcome) => true,
        Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }) => false,
        Err(
            GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
            | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
            | GpuFailureCause::WaitForTheSubmissionFailed { .. }
            | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
            | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
            | GpuFailureCause::ReadbackMappingFailed { .. }
            | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
            | GpuFailureCause::UnusableAfterResubmissions { .. },
        ) => true,
    }
}

/// 在一张卡（`gpu`）上交一次提交（`submit_once`），等超过上限就在同一张卡上重交；超时记进这一批这张卡的那一份（`timeouts_of_the_batch`），
/// 这一批在这张卡上超时的次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就不再重交。每次重交之前先等前一次等超过上限、还没做完的那次
/// （`wait_for_the_timed_out_submission`，照显存的账在等的上限之内等），等不到算这一次重交又超时了一次、不交。任何一次做完了交回它的结果。
/// 不是等超过上限的出错当场交回，不重交。
///
/// 循环每一轮要么交回、要么这一批这张卡的超时次数加一；次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就交回，所以至多 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 轮。
///
/// # Errors
/// 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限 ⇒ [`GpuFailureCause::UnusableAfterResubmissions`]，带最后一次的原因；
/// 别的出错照原样交回。
pub fn outcome_resubmitted_after_timeouts<Gpu, Outcome>(
    gpu: &mut Gpu,
    timeouts_of_the_batch: &mut TimeoutsOfTheBatchOnOneGpu,
    mut wait_for_the_timed_out_submission: impl FnMut(&mut Gpu) -> Result<(), GpuFailureCause>,
    mut submit_once: impl FnMut(&mut Gpu) -> Result<Outcome, GpuFailureCause>,
) -> Result<Outcome, GpuFailureCause> {
    let mut attempt = submit_once(gpu);
    loop {
        match attempt {
            Ok(outcome) => return Ok(outcome),
            Err(cause) if waited_past_the_limit(&cause) => {
                timeouts_of_the_batch.note_a_wait_past_the_limit(cause)?;
            }
            Err(cause) => return Err(cause),
        }
        // 「重交了几次」只数真交出去的：重交之前那一等又等满了上限的那一回没交（判决 m4-gpugc-code-r2 第三节第 2 条），
        // 重交那一次建缓冲显存不足的那一回也没交（判决 m4-gpugc-code-r3 第三节第 1 条），所以交完再看它交没交到卡上才数。
        attempt = match wait_for_the_timed_out_submission(gpu) {
            Ok(()) => {
                let resubmitted = submit_once(gpu);
                if the_attempt_reached_the_gpu(&resubmitted) {
                    timeouts_of_the_batch.resubmissions += 1;
                }
                resubmitted
            }
            Err(cause) => Err(cause),
        };
    }
}

/// 交活之前在一张卡上先等前一次等超过上限、还没做完的那次提交（`wait_once`，在等的上限之内等一回、交回此刻剩下的地方）：
/// 等满上限没做完就记进这一批这张卡的超时次数、再等一回，次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就不再等。
///
/// 循环每一轮要么交回、要么这一批这张卡的超时次数加一，所以至多 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 轮。
///
/// # Errors
/// 同 [`outcome_resubmitted_after_timeouts`]。
pub fn outcome_waited_for_again_after_timeouts<Gpu, Outcome>(
    gpu: &mut Gpu,
    timeouts_of_the_batch: &mut TimeoutsOfTheBatchOnOneGpu,
    mut wait_once: impl FnMut(&mut Gpu) -> Result<Outcome, GpuFailureCause>,
) -> Result<Outcome, GpuFailureCause> {
    loop {
        match wait_once(gpu) {
            Ok(outcome) => return Ok(outcome),
            Err(cause) if waited_past_the_limit(&cause) => {
                timeouts_of_the_batch.note_a_wait_past_the_limit(cause)?;
            }
            Err(cause) => return Err(cause),
        }
    }
}

/// 挂载那一刻开一张卡（起设备、编各段两份核的管线、量一次提交的代价）最多等这么久（用户 2026-11-02 定「gpu 挂载要什么时间上限？ 五分钟？」，
/// 判决 m4-gpugc-code-r3 五之一）：到点就当那张卡没开成（[`outcome_of_opening_a_gpu_within_the_limit`]）。与挂着时的等的上限
/// （`background_gpu_wait_limit`，管每一次等 wgpu 的 future 与提交）分开：编管线（`create_shader_module`、`create_compute_pipeline`）是同步调用，
/// 那个上限管不到它。
pub const WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT: Duration = Duration::from_secs(5 * 60);

/// 在另一条线程上开一张卡（`open`），这条线程至多等 `opening_limit`：开成或开不成照原样交回；到点还没交回就交回
/// [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]（带 `opening_limit`），不再等它（挂载等开卡成功或到点才往下走，不做回调）。
/// 到点之后那条线程照旧跑着、不停它（编管线是驱动里的同步调用，停不下来）：它开出来的东西没人收，它跑完时就地丢掉。
///
/// # Errors
/// `open` 交回的出错；到点 ⇒ `WaitForTheGpuExceededTheLimit`；起不了那条线程 ⇒ [`GpuFailureCause::DeviceOrPipelineDidNotStart`]。
///
/// # Panics
/// `open` panic 了：在这条线程上照原样再 panic（与不隔一条线程时相同）。
pub fn outcome_of_opening_a_gpu_within_the_limit<Opened: Send + 'static>(
    opening_limit: Duration,
    open: impl FnOnce() -> Result<Opened, GpuFailureCause> + Send + 'static,
) -> Result<Opened, GpuFailureCause> {
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let opening_thread = std::thread::Builder::new()
        .name("singlefs-opening-a-gpu".to_string())
        .spawn(move || {
            // 送不出去只有一种：等它的那一头到点不等了、已经丢了，这次开卡的结局没人要。
            let _nobody_waits_for_this_opening_any_more = outcome_sender.send(open());
        })
        .map_err(|error| GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: format!("起不了开卡的那条线程：{error}"),
        })?;
    match outcome_receiver.recv_timeout(opening_limit) {
        Ok(outcome_of_the_opening) => outcome_of_the_opening,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: opening_limit,
            })
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            std::panic::resume_unwind(opening_thread.join().expect_err(
                "开卡的线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还在等）",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use crate::checksum_kernels::{
        checksum_outcomes_of_the_units_cut_into_pieces, ChecksumKernelOutcome, UnitChecksumToVerify,
    };
    use crate::memory_limit_and_chunks::PieceOfAnEntry;
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
    fn the_completion_within_the_wait_limit_returns_its_output() {
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
    fn the_completion_later_than_the_wait_limit_is_a_timeout_failure() {
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
    fn the_zero_wait_limit_times_out_before_checking() {
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
    fn the_failure_reported_by_the_wait_is_returned_as_it_is() {
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
    fn the_batch_whose_wait_times_out_leaves_the_consecutive_uncertainty_counts_unchanged() {
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

    /// 一张卡的替身：按次序交回每一次提交的结局与每一次重交之前等前一次的结局，记下各交了几次、等了几次。
    struct GpuWithScriptedSubmissions {
        submission_outcomes: Vec<Result<u32, GpuFailureCause>>,
        wait_outcomes: Vec<Result<(), GpuFailureCause>>,
        submissions_made: usize,
        waits_made: usize,
    }

    impl GpuWithScriptedSubmissions {
        fn scripted(
            submission_outcomes: Vec<Result<u32, GpuFailureCause>>,
            wait_outcomes: Vec<Result<(), GpuFailureCause>>,
        ) -> Self {
            Self {
                submission_outcomes,
                wait_outcomes,
                submissions_made: 0,
                waits_made: 0,
            }
        }

        /// 照脚本交回这一次提交的结局；脚本用完之后都做完（交 [`SUBMISSION_OUTCOME_AFTER_THE_SCRIPT`]）：被测的重交多交了几次时，
        /// 用例照样走得完、红在断言上，不在脚本的下标上越界。
        fn submit_once(&mut self) -> Result<u32, GpuFailureCause> {
            let outcome = self
                .submission_outcomes
                .get(self.submissions_made)
                .cloned()
                .unwrap_or(Ok(SUBMISSION_OUTCOME_AFTER_THE_SCRIPT));
            self.submissions_made += 1;
            outcome
        }

        /// 照脚本交回这一回等前一次提交的结局；脚本用完之后都等到。
        fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
            let outcome = self
                .wait_outcomes
                .get(self.waits_made)
                .cloned()
                .unwrap_or(Ok(()));
            self.waits_made += 1;
            outcome
        }

        fn outcome_with_resubmissions(&mut self) -> Result<u32, GpuFailureCause> {
            outcome_resubmitted_after_timeouts(
                self,
                &mut TimeoutsOfTheBatchOnOneGpu::none_yet(),
                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
                GpuWithScriptedSubmissions::submit_once,
            )
        }
    }

    /// 一批 `pieces` 片（每片单独一段、各一条 8 字节的单元）照外壳那样逐片交：同一张卡、同一批共用一份超时次数，每片各自套重交。
    /// 交回这一批的结局与这一批这张卡的超时次数。
    fn batch_of_pieces_through_the_resubmissions(
        gpu: &mut GpuWithScriptedSubmissions,
        pieces: usize,
    ) -> (
        Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause>,
        TimeoutsOfTheBatchOnOneGpu,
    ) {
        let unit_bytes: Vec<Vec<u8>> = (0..pieces)
            .map(|piece| vec![u8::try_from(piece).expect("小数"); 8])
            .collect();
        let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, bytes)| UnitChecksumToVerify {
                entry_key,
                unit_bytes: bytes,
                expected_crc32c: 0,
            })
            .collect();
        let chunks_of_pieces: Vec<Vec<PieceOfAnEntry>> = (0..pieces)
            .map(|entry_position| {
                vec![PieceOfAnEntry {
                    entry_position,
                    input_bytes: 0..8,
                }]
            })
            .collect();
        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
        let outcome = ChecksumKernelOutcome {
            computed_crc32c: 0,
            matches_the_expected_crc32c: true,
        };
        let batch = checksum_outcomes_of_the_units_cut_into_pieces(
            &units,
            &chunks_of_pieces,
            |piece_units| {
                let piece_count = piece_units.len();
                outcome_resubmitted_after_timeouts(
                    &mut *gpu,
                    &mut timeouts_of_the_batch,
                    GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
                    |scripted| {
                        scripted
                            .submit_once()
                            .map(|_outcome| vec![(outcome, outcome); piece_count])
                    },
                )
            },
        );
        (batch, timeouts_of_the_batch)
    }

    /// 第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批数：一批切成 4 片、每片头一次交都等超过上限、重交一次就做完——
    /// 这一批在这张卡上第 4 次超时就报这张卡用不了，第 4 片不再重交（攻方探针 T1-1 那一种，改前按片各重交 3 次、照常交回）。
    #[test]
    fn the_batch_of_four_pieces_each_timing_out_once_reports_the_gpu_unusable_at_the_fourth_timeout(
    ) {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
                Ok(0),
                Err(this_submission_timed_out()),
            ],
            vec![Ok(()), Ok(()), Ok(())],
        );
        let (batch, timeouts_of_the_batch) = batch_of_pieces_through_the_resubmissions(&mut gpu, 4);
        assert_eq!(
            batch,
            Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 3,
                last_cause: Box::new(this_submission_timed_out()),
            }),
            "四片各超时一次：这一批第 4 次超时报警"
        );
        assert_eq!(
            (
                gpu.submissions_made,
                gpu.waits_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (7, 3, 4, 3),
            "交了 7 次、重交 3 次，第 4 次超时之后不再重交"
        );
    }

    /// 同一定案：一批 1 片，超时 3 次、第 4 次交做完——这一批照常交回，不报警。
    #[test]
    fn the_batch_of_one_piece_timing_out_three_times_then_completing_is_not_reported() {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
                Ok(0),
            ],
            vec![Ok(()), Ok(()), Ok(())],
        );
        let (batch, timeouts_of_the_batch) = batch_of_pieces_through_the_resubmissions(&mut gpu, 1);
        assert!(batch.is_ok(), "超时 3 次之后做完：照常交回（{batch:?}）");
        assert_eq!(
            (
                gpu.submissions_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (4, 3, 3)
        );
    }

    /// 交活之前先等前一次超时的那次提交那一等也算进这一批这张卡的超时次数：等了两回没等到、第三回等到，再交一片超时两次——第 4 次超时报警。
    #[test]
    fn waits_for_the_earlier_submission_before_handing_over_count_toward_the_timeouts_of_the_batch()
    {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
            ],
            vec![
                Err(earlier_submission_still_running()),
                Err(earlier_submission_still_running()),
                Ok(()),
                Ok(()),
            ],
        );
        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
        assert_eq!(
            outcome_waited_for_again_after_timeouts(
                &mut gpu,
                &mut timeouts_of_the_batch,
                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
            ),
            Ok(()),
            "交活之前等了三回，第三回等到"
        );
        assert_eq!(timeouts_of_the_batch.timeouts(), 2);
        assert_eq!(
            outcome_resubmitted_after_timeouts(
                &mut gpu,
                &mut timeouts_of_the_batch,
                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
                GpuWithScriptedSubmissions::submit_once,
            ),
            Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 1,
                last_cause: Box::new(this_submission_timed_out()),
            }),
            "交活之前那两回加这一片的两回：第 4 次超时报警，真交出去的重交只有 1 次"
        );
        assert_eq!((gpu.submissions_made, gpu.waits_made), (2, 4));
    }

    const WAIT_LIMIT_OF_THE_RESUBMISSIONS: Duration = Duration::from_secs(10);

    /// 脚本用完之后每次提交都做完、交回的值。
    const SUBMISSION_OUTCOME_AFTER_THE_SCRIPT: u32 = 0;

    fn this_submission_timed_out() -> GpuFailureCause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit {
            wait_limit: WAIT_LIMIT_OF_THE_RESUBMISSIONS,
        }
    }

    fn earlier_submission_still_running() -> GpuFailureCause {
        GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
            wait_limit: WAIT_LIMIT_OF_THE_RESUBMISSIONS,
        }
    }

    /// 用户 2026-10-28 定「放弃的那一批在同一张卡上重交，最多 3 次」：头两次等超过上限、第三次做完——照常交回第三次的结果，
    /// 交了 3 次、每次重交之前各等了前一次一回。
    #[test]
    fn two_timeouts_then_a_completion_return_the_outcome_of_the_third_submission() {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
                Ok(42),
            ],
            vec![Ok(()), Ok(())],
        );
        assert_eq!(
            gpu.outcome_with_resubmissions(),
            Ok(42),
            "第三次做完：照常交回"
        );
        assert_eq!(
            (gpu.submissions_made, gpu.waits_made),
            (3, 2),
            "交 3 次，重交两次、每次之前等一回"
        );
    }

    /// 同一定案「都超时才报警」：四次都等超过上限，恰好重交 3 次，报这张卡用不了、带最后一次的原因。
    #[test]
    fn four_timeouts_report_the_gpu_unusable_after_exactly_three_resubmissions() {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
                Err(this_submission_timed_out()),
            ],
            vec![Ok(()), Ok(()), Ok(())],
        );
        assert_eq!(
            gpu.outcome_with_resubmissions(),
            Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 3,
                last_cause: Box::new(this_submission_timed_out()),
            }),
            "四次都超时：这张卡用不了"
        );
        assert_eq!(
            (gpu.submissions_made, gpu.waits_made),
            (4, 3),
            "原来那一次加恰好重交 3 次"
        );
    }

    /// 重交之前等前一次没做完的那次，等不到算这一次重交失败、不交：两回等不到、第三回等到了再交、做完。
    #[test]
    fn the_resubmission_whose_wait_for_the_earlier_submission_times_out_counts_as_failed_without_submitting(
    ) {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![Err(this_submission_timed_out()), Ok(7)],
            vec![
                Err(earlier_submission_still_running()),
                Err(earlier_submission_still_running()),
                Ok(()),
            ],
        );
        assert_eq!(gpu.outcome_with_resubmissions(), Ok(7));
        assert_eq!(
            (gpu.submissions_made, gpu.waits_made),
            (2, 3),
            "两回等不到都没交，第三回等到了才交"
        );
    }

    /// 在一张脚本卡上交一次（套重交），交回结局与这一批这张卡的超时次数、重交次数。
    fn outcome_and_timeouts_of_one_submission(
        gpu: &mut GpuWithScriptedSubmissions,
    ) -> (Result<u32, GpuFailureCause>, TimeoutsOfTheBatchOnOneGpu) {
        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
        let outcome = outcome_resubmitted_after_timeouts(
            gpu,
            &mut timeouts_of_the_batch,
            GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
            GpuWithScriptedSubmissions::submit_once,
        );
        (outcome, timeouts_of_the_batch)
    }

    /// 判决 m4-gpugc-code-r2 第三节第 2 条：头一次交超时，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
    /// 超时 3 次，真交出去的重交 1 次，「重交了几次」记 1。
    #[test]
    fn two_waits_timing_out_before_the_resubmission_count_one_resubmission_actually_made() {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![Err(this_submission_timed_out()), Ok(5)],
            vec![
                Err(earlier_submission_still_running()),
                Err(earlier_submission_still_running()),
                Ok(()),
            ],
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(outcome, Ok(5), "第 3 回等到、重交做完：照常交回");
        assert_eq!(
            (
                gpu.submissions_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (2, 3, 1),
            "交了 2 次，超时 3 次，真交出去的重交 1 次"
        );
    }

    /// 同一条：头一次交超时，之后那一等连着超时 3 回、一次都没重交——第 4 次超时照旧报这张卡用不了，「重交了几次」记 0，
    /// 报警里带的重交次数也是 0（判决 m4-gpugc-code-r3 第三节第 6 条：报真交出去的次数，不报常数 3）。
    #[test]
    fn three_waits_timing_out_before_any_resubmission_count_no_resubmission_and_report_the_gpu_unusable(
    ) {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![Err(this_submission_timed_out())],
            vec![
                Err(earlier_submission_still_running()),
                Err(earlier_submission_still_running()),
                Err(earlier_submission_still_running()),
            ],
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(
            outcome,
            Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 0,
                last_cause: Box::new(earlier_submission_still_running()),
            }),
            "第 4 次超时：这张卡用不了，一次都没真重交"
        );
        assert_eq!(
            (
                gpu.submissions_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (1, 4, 0),
            "只交了头一次，超时 4 次，一次都没真重交"
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 1 条：头一次交等超过上限，重交之前那一等等到了，重交那一次在建缓冲时显存不足、一次都没交出去——
    /// 「重交了几次」记 0（只数真交到卡上的），超时 1 次，建缓冲显存不足照原样交回（外壳照它重切）。
    #[test]
    fn the_resubmission_short_of_memory_for_its_buffers_is_not_counted_as_a_resubmission() {
        let short_of_memory = GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: 2000,
            reason: "用例造的：重交那一次卡上空闲的不够".to_string(),
        };
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(short_of_memory.clone()),
            ],
            vec![Ok(())],
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(outcome, Err(short_of_memory), "建缓冲显存不足照原样交回");
        assert_eq!(
            (
                gpu.submissions_made,
                gpu.waits_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (2, 1, 1, 0),
            "交了两回、等了一回、超时 1 次，重交那一回没交到卡上：重交记 0"
        );
    }

    /// 编管线那一步卡住的替身：卡 `stuck_for` 那么久之后才交回开成了（`OPENED_AFTER_BEING_STUCK`）。
    const OPENED_AFTER_BEING_STUCK: u32 = 7;

    /// 用户 2026-11-02 定挂载时开卡加上限、到点就当那张卡没开成（判决 m4-gpugc-code-r3 五之一）：编管线那一步卡住 20 秒，
    /// 开卡的上限 200 毫秒——到点交回等超过上限（带开卡的上限），挂载那一头不跟着卡住（10 秒之内就交回）。
    #[test]
    fn opening_a_gpu_stuck_past_the_opening_limit_is_reported_as_not_opened_without_waiting_for_it()
    {
        let opening_limit = Duration::from_millis(200);
        let stuck_for = Duration::from_secs(20);
        let started_at = Instant::now();
        let opened = outcome_of_opening_a_gpu_within_the_limit(opening_limit, move || {
            std::thread::sleep(stuck_for);
            Ok(OPENED_AFTER_BEING_STUCK)
        });
        let waited = started_at.elapsed();
        assert_eq!(
            opened,
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: opening_limit
            }),
            "到点还没开成：当那张卡没开成"
        );
        assert!(
            waited < Duration::from_secs(10),
            "到点就往下走，不等卡住的那一步（等了 {waited:?}）"
        );
    }

    /// 同一条的阳性对照：开卡在上限之内开成或开不成，照原样交回。
    #[test]
    fn opening_a_gpu_within_the_opening_limit_returns_what_the_opening_returned() {
        assert_eq!(
            outcome_of_opening_a_gpu_within_the_limit(Duration::from_secs(60), || Ok(42_u32)),
            Ok(42),
            "开成了：交回开出来的"
        );
        let did_not_start = GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: "用例造的：编不了管线".to_string(),
        };
        let did_not_start_in_the_opening = did_not_start.clone();
        assert_eq!(
            outcome_of_opening_a_gpu_within_the_limit(Duration::from_secs(60), move || {
                Err::<u32, GpuFailureCause>(did_not_start_in_the_opening)
            }),
            Err(did_not_start),
            "开不成：交回它的原因"
        );
    }

    /// 开卡的上限是 5 分钟（用户 2026-11-02 定）。
    #[test]
    fn the_wait_limit_of_opening_a_gpu_at_the_mount_is_five_minutes() {
        assert_eq!(
            WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
            Duration::from_secs(60) * 5
        );
    }

    /// 不是等超过上限的出错不重交：当场照原样交回。
    #[test]
    fn the_device_lost_failure_other_than_a_timeout_is_not_resubmitted() {
        let device_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
            reason: "设备丢了（用例造的）".to_string(),
        };
        let mut gpu =
            GpuWithScriptedSubmissions::scripted(vec![Err(device_lost.clone())], Vec::new());
        assert_eq!(gpu.outcome_with_resubmissions(), Err(device_lost));
        assert_eq!((gpu.submissions_made, gpu.waits_made), (1, 0));
    }
}
