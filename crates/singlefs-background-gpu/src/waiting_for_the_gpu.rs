//! 阻塞等 GPU 时的上限（等 GPU 结果设超时，超时算出错；D26（后台整理与放置回收） 已定项 13 第 4 条）：每一处阻塞等 GPU 的地方（挂载那一刻等 wgpu 的 future、
//! 等一次提交做完、等回读缓冲映射好）都在同一个上限之内等，上限由调用方开包时给（`BackgroundGpuSelection::background_gpu_wait_limit`）。
//! 超过上限就交回 [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]。
//!
//! 等超过上限的那一次提交在同一张卡上重交（[`outcome_resubmitted_after_timeouts`]）：放弃的那一批在同一张卡上重交，最多 3 次，都超时才报警；重交的单位只是超时的那一片。超时按批、按卡数（[`TimeoutsOfTheBatchOnOneGpu`]）：
//! 一批切成几片时各片的超时合起来算，交活之前等前一次超时的那次提交那一等（[`outcome_waited_for_again_after_timeouts`]）也算；
//! 重交之前先等前一次没做完的那次（照显存的账，等不到算又超时了一次）；任何一次做完了照常交回；
//! 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次超时就交回 [`GpuFailureCause::UnusableAfterResubmissions`]（这张卡用不了，带最后一次的原因），这一批放弃、不换 CPU，
//! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。别的出错不重交。
//!
//! 开卡（起设备、编管线、量代价）每一次至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（[`outcome_of_opening_a_gpu_within_the_limit`]）；
//! 一张卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才算它开不成（第 4 条，挂载那一刻点名的卡与挂着切换点到的新卡同一种：[`gpu_opened_with_the_reopenings`]）。
//!
//! 等法与钟、完成信号拆开：钟是 [`MonotonicClock`]（产品用这个进程的单调钟 [`SteadyClockOfThisProcess`]），完成信号是调用方给的
//! 「看一次做没做完、最多等这么久」的闭包。不要 GPU 的用例换上假钟与假完成信号，钉住「上限内完成照常、超过上限交回超时出错」。
//!
//! 调用方在停（运行库卸载、丢弃；第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）时，等一批的结果那几等不再等：
//! 期限带着调用方在停的标记（[`CallerStoppingSeenByTheWait`]），一次长等拆成至多 [`SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING`] 的短等，
//! 每一轮没等到就读一次标记，置了就交回 [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]；那一次提交照没等到它做完留在这张卡的账上，
//! 卸载与切换关卡时这张卡整张泄漏。放弃不是等超过上限：不重交、不算进一批在这张卡上的超时次数，也不判这张卡用不了。
//! 不读标记的期限（开卡那几等）照旧在上限之内一次等满；读标记而标记没置时，等的上限与超时的数法逐项不变，只是每一轮至多等一个短等间隔。

use std::time::{Duration, Instant};

use singlefs_core::background_compute::CallerIsStoppingTheBackgroundGpu;

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

/// 等一批的结果时每一轮至多等多久就醒来读一次调用方在停的标记（[`CallerStoppingSeenByTheWait::ReadBetweenShortWaits`]）：
/// 调用方置了标记之后，这一等至多再等这么久就放弃（运行库卸载、丢弃时等维护线程收尾，多等的就是它）。
/// 取 100 毫秒的理由：短等只多醒几次，不拖慢等到的那一刻——等提交用的是 wgpu 带限时的 `poll`，提交做完它当场返回，不等到这一轮的限时满；
/// 等回读映射收通道（`recv_timeout`），送到了同样当场醒。醒一次是一回 `poll` 或一回收通道，等满秒级到分钟级的 `background_gpu_wait_limit`
/// 也只醒上限除以 100 毫秒那么多次（推的，没量过醒一次的开销）；卸载因它多等的至多 100 毫秒（推的，没量过卸载的挂钟）。
pub const SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING: Duration = Duration::from_millis(100);

/// 一次等读不读调用方在停的标记（第 4 条「卸载时不等还在跑的提交」）。封闭集合。
#[derive(Clone, Debug)]
pub enum CallerStoppingSeenByTheWait {
    /// 不读，在上限之内一次等满：开卡那几等（挂载那一刻与切换时在会话锁外开卡，含开卡里量代价的那几次提交），
    /// 与还没收到调用方标记的卡（建把手之前；切换开卡线程上开出、还没收进来的卡）的每一等。
    NotReadByThisWait,
    /// 等一批的结果：每一轮至多等 [`SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING`]，醒来没等到就读一次这个标记，置了就放弃这一等。
    ReadBetweenShortWaits(CallerIsStoppingTheBackgroundGpu),
}

impl CallerStoppingSeenByTheWait {
    /// 这一轮看一次至多等多久：读标记的不超过短等间隔，不读的等到期限。
    fn longest_wait_of_one_check(&self, time_left: Duration) -> Duration {
        match self {
            CallerStoppingSeenByTheWait::NotReadByThisWait => time_left,
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(_caller_is_stopping) => {
                time_left.min(SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING)
            }
        }
    }

    /// 这一等此刻放不放弃：读标记的照标记答，不读的恒不放弃。
    fn gives_up_the_wait_now(&self) -> bool {
        match self {
            CallerStoppingSeenByTheWait::NotReadByThisWait => false,
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping) => {
                caller_is_stopping.is_the_caller_stopping()
            }
        }
    }
}

/// 一次等的期限：从哪一刻起等、最多等多久，与读不读调用方在停的标记。一次提交的几处等（等提交做完、等两份回读映射好）共用一个期限，合起来不超过上限。
#[derive(Clone, Debug)]
pub struct GpuWaitDeadline {
    started_at: Duration,
    wait_limit: Duration,
    caller_stopping: CallerStoppingSeenByTheWait,
}

impl GpuWaitDeadline {
    /// 从钟此刻的读数起等，最多等 `wait_limit`，不读调用方在停的标记（开卡那几等用它）。
    #[must_use]
    pub fn starting_now(wait_limit: Duration, clock: &impl MonotonicClock) -> Self {
        Self::starting_now_seeing_the_caller_stopping(
            wait_limit,
            clock,
            CallerStoppingSeenByTheWait::NotReadByThisWait,
        )
    }

    /// 同 [`Self::starting_now`]，读不读调用方在停的标记照 `caller_stopping`（等一批的结果那几等从卡的账上取，
    /// `gpus_of_this_host::GpuBudgetUnderTheSettings::deadline_of_a_wait_of_the_batch_starting_now`）。
    #[must_use]
    pub fn starting_now_seeing_the_caller_stopping(
        wait_limit: Duration,
        clock: &impl MonotonicClock,
        caller_stopping: CallerStoppingSeenByTheWait,
    ) -> Self {
        Self {
            started_at: clock.time_since_the_origin(),
            wait_limit,
            caller_stopping,
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

/// 在期限之内反复看 `check_once_waiting_at_most`（参数是这一次最多等多久：离期限还剩的那么久，期限读调用方在停的标记时至多
/// [`SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING`]），直到做完、报错、等满上限，或调用方在停。
/// 每一轮先看还剩多久：剩 0 就交回超时，不再看；所以一次都还没看就已经过了期限的（上限是 0）也交回超时。
/// 每一轮看完没等到，期限读标记的就读一次：置了就交回放弃，不再看（醒来才读：标记置上之前已在等的那一轮照等完它那一个短等）。
///
/// 循环的出口只有四个：做完、报错、等满上限、调用方在停。迭代次数没有静态上界：每一轮要么等到了、要么至少等了交给它的那么久或提前醒了，
/// 钟单调往前走，期限总会到。
///
/// # Errors
/// 等满上限 ⇒ [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]（带上限本身）；期限读标记而调用方在停 ⇒
/// [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]；`check_once_waiting_at_most` 交回的出错照原样交回。
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
        match check_once_waiting_at_most(
            deadline
                .caller_stopping
                .longest_wait_of_one_check(time_left),
        ) {
            CompletionCheck::Completed(output) => return Ok(output),
            CompletionCheck::NotYetComplete => {
                if deadline.caller_stopping.gives_up_the_wait_now() {
                    return Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping);
                }
            }
            CompletionCheck::Failed(cause) => return Err(cause),
        }
    }
}

/// 一批里一张卡等超过上限之后在同一张卡上至多重交几次：这一批在这张卡上第
/// 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限就报这张卡用不了。
pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;

/// 是不是等超过上限的那三种：这一次提交自己等超过上限，交活之前、重交之前等前一次那一次又等满了上限，或建缓冲出错之后冲掉待写的那一次空提交等满了上限。
/// 只有这三种重交、算进一批的超时次数。调用方在停而放弃的那一等不是：不重交、不数（第 4 条「卸载时不等还在跑的提交」）。
fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. } => true,
        GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        | GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
        | GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => false,
    }
}

/// 一批里一张卡上到此刻等超过上限了几次、重交了几次（重交只数真交出去的：重交之前那一等又等满了上限、没交的那一回只算超时，
/// 重交那一次建缓冲显存不足、或之后冲掉待写的那一次空提交等满了上限而没交出去的那一回不算）（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
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
            // 报这一批这张卡真交出去的重交次数：重交之前那一等等满了上限、
            // 重交那一次建缓冲显存不足没交出去的那几回不算，所以它可以比 RESUBMISSIONS_AFTER_A_TIMEOUT 少。
            return Err(GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: self.resubmissions,
                last_cause: Box::new(cause),
            });
        }
        Ok(())
    }
}

/// 一次交的结局说不说明那一次真交到了卡上：建缓冲出错（显存不足 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，或之后冲掉待写的那一次空提交等满了上限
/// [`GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit`]）是在交之前判出的、那一段没交；
/// 别的出错都是交了之后才判出的（等提交、回读、读回的判定字），做完了的当然交了。只给交一段那一次的结局用（`submit_once` 交回的）。
/// 调用方在停而放弃的那一等算交到了：放弃的多半是等交出去的那一次提交（冲掉待写那一等放弃的分不出来，一并算交到了；放弃之后不再重交，
/// 这个数只进这一批的记账）。
fn the_attempt_reached_the_gpu<Outcome>(attempt: &Result<Outcome, GpuFailureCause>) -> bool {
    match attempt {
        Ok(_outcome) => true,
        Err(
            GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
            | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. },
        ) => false,
        Err(
            GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
            | GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
            | GpuFailureCause::OpeningThreadNotStarted { .. }
            | GpuFailureCause::OpeningPanicked { .. }
            | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
            | GpuFailureCause::OpeningCalledOff
            | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
            | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
            | GpuFailureCause::WaitForTheSubmissionFailed { .. }
            | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
            | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
            | GpuFailureCause::ReadbackMappingFailed { .. }
            | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
            | GpuFailureCause::UnusableAfterResubmissions { .. }
            | GpuFailureCause::NotOpenedAfterTheReopenings { .. },
        ) => true,
    }
}

/// 在一张卡（`gpu`）上交一次提交（`submit_once`），等超过上限就在同一张卡上重交；超时记进这一批这张卡的那一份（`timeouts_of_the_batch`），
/// 这一批在这张卡上超时的次数越过 [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 就不再重交。每次重交之前先等前一次等超过上限、还没做完的那次
/// （`wait_for_the_timed_out_submission`，照显存的账在等的上限之内等），等不到算这一次重交又超时了一次、不交。任何一次做完了交回它的结果。
/// 不是等超过上限的出错当场交回，不重交；调用方在停而放弃的那一等（交这一次之后那一等，或重交之前等前一次那一等）同样当场交回，不再重交。
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
        // 「重交了几次」只数真交出去的：重交之前那一等又等满了上限的那一回没交，
        // 重交那一次建缓冲显存不足的那一回也没交，所以交完再看它交没交到卡上才数。
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

/// 挂载那一刻开一张卡（起设备、编各段两份核的管线、量一次提交的代价）最多等这么久：
/// 到点就当那张卡没开成（[`outcome_of_opening_a_gpu_within_the_limit`]）。与挂着时的等的上限
/// （`background_gpu_wait_limit`，管每一次等 wgpu 的 future 与提交）分开：编管线（`create_shader_module`、`create_compute_pipeline`）是同步调用，
/// 那个上限管不到它。
pub const WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT: Duration = Duration::from_secs(5 * 60);

/// 在另一条线程上开一张卡（`open`），这条线程至多等 `opening_limit`：开成或开不成照原样交回；到点还没交回就交回
/// [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]（带 `opening_limit`），不再等它（挂载等开卡成功或到点才往下走，不做回调）。
/// 到点之后那条线程照旧跑着、不停它（编管线是驱动里的同步调用，停不下来）：它开出来的东西没人收，它跑完时就地丢掉。
/// 产品开卡（`background_gpu::started_gpu_with_its_cost_measured`）交回的东西丢掉时不等 GPU：开成的那张卡量代价的提交都已做完，照常析构；
/// 量代价出错而账上还记着在跑的提交的那张卡，在 `open` 里已经泄漏、这一次占着的字节已记进这个进程的泄漏账，交回的只是那个错。
///
/// # Errors
/// `open` 交回的出错；到点 ⇒ `WaitForTheGpuExceededTheLimit`；起不了那条线程 ⇒ [`GpuFailureCause::OpeningThreadNotStarted`]。
///
/// # Panics
/// `open` panic 了：在这条线程上照原样再 panic（与不隔一条线程时相同；挂载与切换共用的开一张卡那一步接住它，
/// `gpus_of_this_host::gpu_opened_under_its_memory_limit`）。
pub fn outcome_of_opening_a_gpu_within_the_limit<Opened: Send + 'static>(
    opening_limit: Duration,
    open: impl FnOnce() -> Result<Opened, GpuFailureCause> + Send + 'static,
) -> Result<Opened, GpuFailureCause> {
    outcome_of_opening_a_gpu_within_what_is_left_of_the_limit(opening_limit, opening_limit, open)
}

/// 同 [`outcome_of_opening_a_gpu_within_the_limit`]，这一次开卡的期限 `opening_limit` 已经用掉了一段（同一张卡上排队等别的挂载那一次开完，
/// `background_gpu::opening_of_a_card_within_the_limit`）：这条线程只再等剩下的 `time_left_of_the_limit`，到点报的仍是整个期限 `opening_limit`。
///
/// # Errors
/// 同 [`outcome_of_opening_a_gpu_within_the_limit`]。
///
/// # Panics
/// 同 [`outcome_of_opening_a_gpu_within_the_limit`]。
pub fn outcome_of_opening_a_gpu_within_what_is_left_of_the_limit<Opened: Send + 'static>(
    opening_limit: Duration,
    time_left_of_the_limit: Duration,
    open: impl FnOnce() -> Result<Opened, GpuFailureCause> + Send + 'static,
) -> Result<Opened, GpuFailureCause> {
    let (outcome_sender, outcome_receiver) = std::sync::mpsc::channel();
    let opening_thread = std::thread::Builder::new()
        .name("singlefs-opening-a-gpu".to_string())
        .spawn(move || {
            // 送不出去只有一种：等它的那一头到点不等了、已经丢了，这次开卡的结局没人要。
            let _nobody_waits_for_this_opening_any_more = outcome_sender.send(open());
        })
        .map_err(|error| GpuFailureCause::OpeningThreadNotStarted {
            reason: format!("起不了开卡的那条线程：{error}"),
        })?;
    match outcome_receiver.recv_timeout(time_left_of_the_limit) {
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

/// 一张卡开不成之后隔多久重开（第 4 条「间隔 1 分钟重开」，挂载那一刻点名的卡与挂着切换点到的新卡同一个数）。
pub const INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED: Duration = Duration::from_secs(60);

/// 一张卡开不成之后至多重开几次（第 4 条「最多重开 3 次」）：头一次开不成之后再开这么多次，都不成才算这张卡开不成。
pub const REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST: u32 = 3;

/// 一张卡开不成时怎么重开：头一次开不成就等 `interval` 再开，至多再开 `reopenings_at_most` 次，都不成才算这张卡开不成
/// （[`gpu_opened_with_the_reopenings`]）。产品是 [`Self::OF_THE_PRODUCT`]；用例换上间隔为 0 的，不真等 1 分钟。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReopeningsOfAGpuNotOpened {
    pub interval: Duration,
    pub reopenings_at_most: u32,
}

impl ReopeningsOfAGpuNotOpened {
    /// 产品的：间隔 [`INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED`]，至多重开 [`REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST`] 次。
    pub const OF_THE_PRODUCT: Self = Self {
        interval: INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED,
        reopenings_at_most: REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST,
    };

    /// 一张卡至多开几次：头一次加至多重开的次数（第 4 条「头一次开卡不算重开，一张卡至多开 4 次」）。挂着切换时一张卡在一次挂载里连着开不成到这个数（开成一次清零），
    /// 再被点到就不再开（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。
    #[must_use]
    pub const fn openings_at_most(self) -> u32 {
        self.reopenings_at_most.saturating_add(1)
    }

    /// 连同重开，一张卡至多要等多久才有结局：每一次开卡至多 `each_opening`，开 1 + 重开次数 次，中间隔重开次数个间隔；超出 `Duration` 取它的最大值。
    #[must_use]
    pub const fn longest_wait_of_one_gpu(self, each_opening: Duration) -> Duration {
        each_opening
            .saturating_mul(self.reopenings_at_most.saturating_add(1))
            .saturating_add(self.interval.saturating_mul(self.reopenings_at_most))
    }
}

/// 开一张卡（`open_once` 开一次，交回开出来的或这一次开不成的原因），开不成就照 `reopenings` 重开：每次重开之前把间隔交给
/// `wait_before_a_reopening` 等（产品是 `std::thread::sleep`），开成了就交回、不再开。
///
/// # Errors
/// 开了 1 + `reopenings.reopenings_at_most` 次都没开成：每一次的原因，按开的次序（头一次的在前）。
pub fn gpu_opened_with_the_reopenings<Opened>(
    reopenings: ReopeningsOfAGpuNotOpened,
    mut wait_before_a_reopening: impl FnMut(Duration),
    mut open_once: impl FnMut() -> Result<Opened, GpuFailureCause>,
) -> Result<Opened, Vec<GpuFailureCause>> {
    let mut causes_of_each_opening = Vec::new();
    for reopenings_so_far in 0..=reopenings.reopenings_at_most {
        if reopenings_so_far > 0 {
            wait_before_a_reopening(reopenings.interval);
        }
        match open_once() {
            Ok(opened) => return Ok(opened),
            Err(cause) => causes_of_each_opening.push(cause),
        }
    }
    Err(causes_of_each_opening)
}

/// 用例要一个置得上的调用方在停的标记：核心层只让把手造它、置它（`singlefs_core::background_compute::BackgroundGpuComputeHandle`），
/// 这里照核心层建把手的路造一份只收标记的替身实现，交回替身收到的那一份标记与置它的那一份把手。本包几个模块的用例共用。
#[cfg(test)]
pub(crate) mod caller_stopping_for_tests {
    use std::sync::{Arc, Mutex};

    use singlefs_core::allocator::AllocationRecord;
    use singlefs_core::background_compute::{
        AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
        BackgroundGpuComputeHandle, CallerIsStoppingTheBackgroundGpu, ClusterSegmentEntryKey,
        GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
        RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    };
    use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
    use singlefs_core::logged_operations::CompactionRegion;
    use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};

    /// 只记下建把手时交来的调用方在停的标记的替身（三段与换设置都不该被调）。
    struct ImplementationKeepingTheCallerStoppingFlag {
        flag_received: Arc<Mutex<Option<CallerIsStoppingTheBackgroundGpu>>>,
    }

    impl GpuInventory for ImplementationKeepingTheCallerStoppingFlag {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            Vec::new()
        }
    }

    impl BackgroundGpuCompute for ImplementationKeepingTheCallerStoppingFlag {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            panic!("只收标记的替身不算一批");
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            panic!("只收标记的替身不算一批");
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            panic!("只收标记的替身不算一批");
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {
            panic!("只收标记的替身不换设置");
        }

        fn give_up_waiting_for_the_cards_once_the_caller_is_stopping(
            &mut self,
            caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
        ) {
            *self
                .flag_received
                .lock()
                .expect("记标记的格子只在这里写，写的时候不 panic") = Some(caller_is_stopping);
        }
    }

    /// 一份还没置的调用方在停的标记，与置它的那一份把手（`tell_the_implementation_the_caller_is_stopping`）。
    pub(crate) fn caller_stopping_flag_and_the_handle_setting_it(
    ) -> (CallerIsStoppingTheBackgroundGpu, BackgroundGpuComputeHandle) {
        let flag_received = Arc::new(Mutex::new(None));
        let handle = BackgroundGpuComputeHandle::of_an_implementation(
            ImplementationKeepingTheCallerStoppingFlag {
                flag_received: Arc::clone(&flag_received),
            },
        );
        let caller_is_stopping = flag_received
            .lock()
            .expect("记标记的格子只在建把手时写过一次")
            .clone()
            .expect("核心层建把手时把调用方在停的标记交给实现");
        assert!(
            !caller_is_stopping.is_the_caller_stopping(),
            "刚建的把手还没说调用方在停"
        );
        (caller_is_stopping, handle)
    }
}

#[cfg(test)]
mod tests {
    use super::caller_stopping_for_tests::caller_stopping_flag_and_the_handle_setting_it;
    use super::*;
    use std::cell::Cell;

    use crate::checksum_kernels::{
        checksum_outcomes_of_the_units_cut_into_pieces, ChecksumKernelOutcome, UnitChecksumToVerify,
    };
    use crate::memory_limit_and_chunks::{InputLengthOnTheHostInBytes, PieceOfAnEntry};
    use crate::two_kernel_reconciliation::{
        ConsecutiveUncertaintyCounts, SameEntryKeyTwiceInOneBatch, TwoKernelOutcomesOfAnEntry,
    };

    /// 用例里一批交不成的样子：等 GPU 那一样报的错原样，或同一个条目键出现两次。
    #[derive(Debug, PartialEq, Eq)]
    enum BatchFailureInTheTest {
        WaitingForTheGpu(GpuFailureCause),
        SameEntryKeyTwiceInOneBatch(SameEntryKeyTwiceInOneBatch),
    }

    impl From<SameEntryKeyTwiceInOneBatch> for BatchFailureInTheTest {
        fn from(repeated: SameEntryKeyTwiceInOneBatch) -> Self {
            Self::SameEntryKeyTwiceInOneBatch(repeated)
        }
    }

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

    /// 短等间隔是 100 毫秒（取值的理由在常量那一处）。
    #[test]
    fn the_short_wait_between_readings_of_the_caller_stopping_is_one_hundred_milliseconds() {
        assert_eq!(
            SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING,
            Duration::from_millis(100)
        );
    }

    /// 第 4 条「卸载时不等还在跑的提交」：一次本来要等满 600 秒上限的等（提交一直没做完），等到第 101 秒调用方置了在停的标记——
    /// 置标记那一轮醒来就放弃、交回放弃那一种，比置标记那一刻至多晚一个短等间隔；每一轮至多等一个短等间隔。
    #[test]
    fn a_wait_reading_the_caller_stopping_gives_up_within_one_short_wait_after_the_flag_is_set() {
        let (caller_is_stopping, handle_setting_the_flag) =
            caller_stopping_flag_and_the_handle_setting_it();
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::from_secs(100)),
        };
        let deadline = GpuWaitDeadline::starting_now_seeing_the_caller_stopping(
            Duration::from_secs(600),
            &clock,
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping),
        );
        let flag_set_at: Cell<Option<Duration>> = Cell::new(None);
        let longest_wait_handed_to_a_check = Cell::new(Duration::ZERO);
        let waited = wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
            longest_wait_handed_to_a_check.set(longest_wait_handed_to_a_check.get().max(time_left));
            clock
                .now
                .set(clock.now.get() + time_left.max(Duration::from_nanos(1)));
            if flag_set_at.get().is_none() && clock.now.get() >= Duration::from_secs(101) {
                handle_setting_the_flag.tell_the_implementation_the_caller_is_stopping();
                flag_set_at.set(Some(clock.now.get()));
            }
            CompletionCheck::<u32>::NotYetComplete
        });
        assert_eq!(
            waited,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping),
            "调用方在停：放弃这一等，不等满上限、不报超时"
        );
        let moment_the_flag_was_set = flag_set_at.get().expect("钟走过第 101 秒的那一轮置了标记");
        assert_eq!(
            clock.now.get(),
            moment_the_flag_was_set,
            "置标记那一轮醒来就放弃，不再看下一轮"
        );
        assert!(
            moment_the_flag_was_set
                < Duration::from_secs(101) + SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING,
            "放弃比第 101 秒至多晚一个短等间隔（放弃在 {moment_the_flag_was_set:?}）"
        );
        assert_eq!(
            longest_wait_handed_to_a_check.get(),
            SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING,
            "读标记的等每一轮至多等一个短等间隔"
        );
    }

    /// 同一条在真钟上：上限 30 秒的等（提交一直没做完、每一轮睡满交给它的那么久），另一条线程 50 毫秒后置了在停的标记——
    /// 这一等 5 秒之内就交回放弃（量级是一个短等间隔），不等满 30 秒。
    #[test]
    fn a_wait_reading_the_caller_stopping_on_the_steady_clock_returns_soon_after_another_thread_sets_the_flag(
    ) {
        let (caller_is_stopping, handle_setting_the_flag) =
            caller_stopping_flag_and_the_handle_setting_it();
        let clock = SteadyClockOfThisProcess::starting_now();
        let deadline = GpuWaitDeadline::starting_now_seeing_the_caller_stopping(
            Duration::from_secs(30),
            &clock,
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping),
        );
        let started_at = Instant::now();
        let waited = std::thread::scope(|scope| {
            let _thread_setting_the_flag_joined_at_the_end_of_the_scope = scope.spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                handle_setting_the_flag.tell_the_implementation_the_caller_is_stopping();
            });
            wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
                std::thread::sleep(time_left);
                CompletionCheck::<u32>::NotYetComplete
            })
        });
        let waited_for = started_at.elapsed();
        assert_eq!(
            waited,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)
        );
        assert!(
            waited_for < Duration::from_secs(5),
            "置标记之后一个短等间隔的量级就交回，不等满 30 秒（等了 {waited_for:?}）"
        );
    }

    /// 没置标记时等的上限与超时照旧：读标记的等，提交要到第 7 秒才做完、上限 5 秒——照旧在第 5 秒交回超时（带上限本身），
    /// 只是每一轮至多等一个短等间隔、多看了几次。
    #[test]
    fn a_wait_reading_the_caller_stopping_not_yet_set_times_out_at_the_wait_limit_as_before() {
        let (caller_is_stopping, _handle_never_setting_the_flag) =
            caller_stopping_flag_and_the_handle_setting_it();
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::from_secs(100)),
        };
        let deadline = GpuWaitDeadline::starting_now_seeing_the_caller_stopping(
            Duration::from_secs(5),
            &clock,
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping),
        );
        let checks_made = Cell::new(0);
        assert_eq!(
            wait_for_completion_before_the_deadline(
                &deadline,
                &clock,
                completion_at(&clock, Duration::from_secs(107), &checks_made),
            ),
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(5)
            }),
            "没置标记：照旧等满 5 秒交回超时"
        );
        assert_eq!(clock.now.get(), Duration::from_secs(105), "等到期限为止");
        assert_eq!(checks_made.get(), 50, "每一轮至多等 100 毫秒：5 秒看 50 次");
    }

    /// 不读标记的等（开卡那几等）不拆短等：头一次看就把离期限还剩的整段交给它。
    #[test]
    fn a_wait_not_reading_the_caller_stopping_hands_the_whole_time_left_to_one_check() {
        let clock = ClockMovedByTheTest {
            now: Cell::new(Duration::from_secs(100)),
        };
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(300), &clock);
        let waits_handed_to_the_checks = std::cell::RefCell::new(Vec::new());
        let waited = wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
            waits_handed_to_the_checks.borrow_mut().push(time_left);
            clock.now.set(clock.now.get() + time_left);
            CompletionCheck::<u32>::NotYetComplete
        });
        assert_eq!(
            waited,
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(300)
            })
        );
        assert_eq!(
            waits_handed_to_the_checks.into_inner(),
            vec![Duration::from_secs(300)],
            "一次等满上限"
        );
    }

    /// 超时那一批走的是与 GPU 出错同一条路：交回出错，连续不确定计数一条都不动，CPU 参照一次都不调。
    #[test]
    fn the_batch_whose_wait_times_out_leaves_the_consecutive_uncertainty_counts_unchanged() {
        let mut uncertainty: ConsecutiveUncertaintyCounts<u32> =
            ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let disagreeing_round = uncertainty.reconcile_unless_the_batch_failed(
            Ok::<_, BatchFailureInTheTest>(vec![TwoKernelOutcomesOfAnEntry {
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
        })
        .map_err(BatchFailureInTheTest::WaitingForTheGpu);
        let timed_out_round = uncertainty
            .reconcile_unless_the_batch_failed(timed_out_outcomes, |_entry_position| {
                panic!("超时那一批不交 CPU 判")
            });
        assert_eq!(
            timed_out_round,
            Err(BatchFailureInTheTest::WaitingForTheGpu(
                GpuFailureCause::WaitForTheGpuExceededTheLimit {
                    wait_limit: Duration::from_secs(2)
                }
            )),
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
                    input_bytes: InputLengthOnTheHostInBytes(0)..InputLengthOnTheHostInBytes(8),
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
    /// 这一批在这张卡上第 4 次超时就报这张卡用不了，第 4 片不再重交。
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

    /// 一批 1 片，超时 3 次、第 4 次交做完——这一批照常交回，不报警。
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

    /// 放弃的那一批在同一张卡上重交，最多 3 次：头两次等超过上限、第三次做完——照常交回第三次的结果，
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

    /// 「都超时才报警」：四次都等超过上限，恰好重交 3 次，报这张卡用不了、带最后一次的原因。
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

    /// 头一次交超时，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
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
    /// 报警里带的重交次数也是 0（报真交出去的次数，不报常数 3）。
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

    /// 头一次交等超过上限，重交之前那一等等到了，重交那一次在建缓冲时显存不足、一次都没交出去——
    /// 「重交了几次」记 0（只数真交到卡上的），超时 1 次，建缓冲显存不足照原样交回（外壳照它重切）。
    #[test]
    fn the_resubmission_short_of_memory_for_its_buffers_is_not_counted_as_a_resubmission() {
        let short_of_memory = GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: singlefs_core::background_compute::GpuMemoryInBytes(2000),
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

    /// 头一次交等超过上限，重交之前那一等等到了，重交那一次建缓冲出错、冲掉待写的那一次空提交等满了上限（这一段没交到卡上），
    /// 再一回等到、重交做完——超时 2 次（冲掉待写那一回算一次），「重交了几次」只记真交到卡上的那 1 次。
    #[test]
    fn the_flush_of_the_pending_writes_timing_out_counts_as_a_timeout_and_not_as_a_resubmission() {
        let flush_timed_out =
            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
                wait_limit: WAIT_LIMIT_OF_THE_RESUBMISSIONS,
            };
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![
                Err(this_submission_timed_out()),
                Err(flush_timed_out),
                Ok(9),
            ],
            vec![Ok(()), Ok(())],
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(
            outcome,
            Ok(9),
            "冲掉待写等不到照重交的规矩往下走，第三回做完"
        );
        assert_eq!(
            (
                gpu.submissions_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (3, 2, 1),
            "交了 3 回，超时 2 次，冲掉待写等不到的那一回没交到卡上：重交记 1"
        );
    }

    /// 编管线那一步卡住的替身：卡 `stuck_for` 那么久之后才交回开成了（`OPENED_AFTER_BEING_STUCK`）。
    const OPENED_AFTER_BEING_STUCK: u32 = 7;

    /// 挂载时开卡加上限、到点就当那张卡没开成：编管线那一步卡住 20 秒，
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

    /// 开卡的上限是 5 分钟。
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

    /// 第 4 条「卸载时不等还在跑的提交」：交了一次、等它时调用方在停而放弃——当场交回放弃，不重交、不等前一次，
    /// 不算进这一批这张卡的超时次数（不朝「这张卡用不了」数）。
    #[test]
    fn giving_up_the_wait_of_a_submission_for_the_caller_stopping_is_not_resubmitted_nor_counted_as_a_timeout(
    ) {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)],
            Vec::new(),
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(
            outcome,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)
        );
        assert_eq!(
            (
                gpu.submissions_made,
                gpu.waits_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (1, 0, 0, 0),
            "只交了那一次：不重交、不等、超时 0 次"
        );
    }

    /// 头一次交等超过上限，重交之前等前一次那一等调用方在停而放弃——当场交回放弃，不再重交；超时只记头一次那 1 次。
    #[test]
    fn giving_up_the_wait_for_the_timed_out_submission_for_the_caller_stopping_does_not_resubmit() {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            vec![Err(this_submission_timed_out())],
            vec![Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)],
        );
        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
        assert_eq!(
            outcome,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)
        );
        assert_eq!(
            (
                gpu.submissions_made,
                gpu.waits_made,
                timeouts_of_the_batch.timeouts(),
                timeouts_of_the_batch.resubmissions()
            ),
            (1, 1, 1, 0),
            "交了头一次、等了一回就放弃：不重交"
        );
    }

    /// 交活之前等前一次超时的那次提交时调用方在停而放弃：当场交回放弃，不再等下一回、不算一次超时。
    #[test]
    fn giving_up_the_wait_before_handing_over_for_the_caller_stopping_is_not_counted_as_a_timeout()
    {
        let mut gpu = GpuWithScriptedSubmissions::scripted(
            Vec::new(),
            vec![
                Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping),
                Err(earlier_submission_still_running()),
            ],
        );
        let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
        assert_eq!(
            outcome_waited_for_again_after_timeouts(
                &mut gpu,
                &mut timeouts_of_the_batch,
                GpuWithScriptedSubmissions::wait_for_the_timed_out_submission,
            ),
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping)
        );
        assert_eq!(
            (gpu.waits_made, timeouts_of_the_batch.timeouts()),
            (1, 0),
            "等了一回就放弃，不算超时"
        );
    }

    /// 用例里第 `opening` 次开卡（从 1 数）开不成的原因。
    fn cause_of_the_opening_in_the_test(opening: u32) -> GpuFailureCause {
        GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: format!("用例造的：第 {opening} 次起不了设备"),
        }
    }

    /// 照脚本开卡（`outcomes_in_order` 按次序交回，脚本用完就开不成），重开的规矩是间隔 7 秒、至多重开 3 次；交回结局、开了几次与每次重开之前交去等的间隔。
    fn opening_with_the_reopenings_in_the_test(
        outcomes_in_order: Vec<Result<u32, GpuFailureCause>>,
    ) -> (Result<u32, Vec<GpuFailureCause>>, u32, Vec<Duration>) {
        let mut outcomes = outcomes_in_order.into_iter();
        let mut openings = 0_u32;
        let mut waits = Vec::new();
        let outcome = gpu_opened_with_the_reopenings(
            ReopeningsOfAGpuNotOpened {
                interval: Duration::from_secs(7),
                reopenings_at_most: 3,
            },
            |interval| waits.push(interval),
            || {
                openings += 1;
                outcomes
                    .next()
                    .unwrap_or_else(|| Err(cause_of_the_opening_in_the_test(openings)))
            },
        );
        (outcome, openings, waits)
    }

    /// 开不成两次、第三次开成：交回第三次开出来的，开了 3 次，两次重开之前各等一个间隔。
    #[test]
    fn the_gpu_not_opened_twice_and_opened_the_third_time_is_handed_back_after_two_intervals() {
        let (outcome, openings, waits) = opening_with_the_reopenings_in_the_test(vec![
            Err(cause_of_the_opening_in_the_test(1)),
            Err(cause_of_the_opening_in_the_test(2)),
            Ok(42),
        ]);
        assert_eq!(outcome, Ok(42), "第三次开成：照常交回");
        assert_eq!(openings, 3, "开成了就不再开");
        assert_eq!(
            waits,
            vec![Duration::from_secs(7); 2],
            "两次重开之前各等一个间隔"
        );
    }

    /// 头一次与 3 次重开都开不成：交回每一次的原因（按开的次序），一共开 4 次、等 3 个间隔，之后不再开。
    #[test]
    fn the_gpu_never_opened_is_reopened_three_times_and_every_cause_is_handed_back_in_order() {
        let (outcome, openings, waits) = opening_with_the_reopenings_in_the_test(Vec::new());
        assert_eq!(
            outcome,
            Err((1..=4).map(cause_of_the_opening_in_the_test).collect()),
            "每一次开不成的原因都交回，头一次的在前"
        );
        assert_eq!(openings, 4, "头一次加 3 次重开");
        assert_eq!(waits, vec![Duration::from_secs(7); 3]);
    }

    /// 产品的重开：间隔 1 分钟、至多重开 3 次；连同重开一张卡至多等 4 次开卡上限加 3 个间隔（5 分钟的上限下是 23 分钟）。
    #[test]
    fn the_product_reopens_a_gpu_not_opened_three_times_one_minute_apart() {
        assert_eq!(
            ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
            ReopeningsOfAGpuNotOpened {
                interval: Duration::from_secs(60),
                reopenings_at_most: 3,
            }
        );
        assert_eq!(
            ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT
                .longest_wait_of_one_gpu(WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT),
            Duration::from_secs(60) * 23
        );
    }
}
