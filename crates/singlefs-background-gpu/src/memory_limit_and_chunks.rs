//! 显存上限与切批（D26（后台整理与放置回收） 已定项 13 第 3 条）：每张卡上本包占的缓冲（输入、输出、暂存一起）任何时刻不超过那张卡的上限，
//! 一批放不下就切成几次交，不给上限就不设限（只受这张卡的绑定上限与派发上限管）。
//!
//! 切法是纯函数（[`chunks_that_fit_on_the_gpu`]）；外壳每建一个缓冲先在这张卡的账上记下（[`GpuMemoryAccount::hold`]），
//! 记下之后超过上限就断言：切批算对了就走不到那一步，走到了是切批的 bug，不带着超限往下跑。
//!
//! 等一段的提交超过上限（这一批按出错交回）之后，GPU 上可能还在跑它：wgpu 留着被提交用到的缓冲直到提交做完，卡上的显存照旧占着。
//! 所以这一段的账不在交回出错时放，记成这张卡上还在跑的那次提交（[`SubmissionOfTheChunk`]、[`GpuMemoryAccount::settle_the_chunk_after_its_submission`]），
//! 等它的完成信号来了才放；下一次在这张卡上切段交活之前，先在上限之内等它做完
//! （[`GpuMemoryAccount::chunks_cut_once_the_submission_running_past_its_wait_completed`]），等不到就交回出错、这张卡上一段都不交，不越过上限去交。

use std::time::Duration;

use singlefs_core::background_compute::GpuMemoryInBytes;

use crate::background_gpu::GpuFailureCause;
use crate::waiting_for_the_gpu::{
    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
};

/// GPU 缓冲的字节数要按 4 字节对齐（wgpu 的 `COPY_BUFFER_ALIGNMENT`；着色器里输入按 `array<u32>` 读）。
pub const GPU_BUFFER_ALIGNMENT_IN_BYTES: u64 = 4;

/// 输入缓冲的字节数：输入字节按 4 字节向上取整；一个字节都没有时也占 4 字节（wgpu 不许绑定长度为 0 的缓冲）。
#[must_use]
pub fn input_buffer_bytes(input_bytes: u64) -> u64 {
    input_bytes
        .max(GPU_BUFFER_ALIGNMENT_IN_BYTES)
        .div_ceil(GPU_BUFFER_ALIGNMENT_IN_BYTES)
        * GPU_BUFFER_ALIGNMENT_IN_BYTES
}

/// 一段（这一次交给一张卡的那几条）在卡上占多少：输入缓冲之外，每一条占的固定字节（描述、两份核的输出与回读暂存），
/// 加上与条数无关的那几样（参数）。由每一段的核自己给（校验和核对那一段在 `checksum_kernels` 里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkFootprint {
    pub bytes_per_chunk_besides_the_entries: u64,
    pub bytes_per_entry_besides_its_input: u64,
    /// 每一条在它最大的那一个按条排的缓冲里占几字节：那个缓冲也要装得进这张卡的绑定上限。
    pub bytes_per_entry_in_the_largest_entry_buffer: u64,
}

impl ChunkFootprint {
    /// 这一段在卡上合占的字节数。
    #[must_use]
    pub fn bytes_held_by_a_chunk(&self, input_bytes: u64, entry_count: u64) -> u64 {
        input_buffer_bytes(input_bytes)
            + entry_count * self.bytes_per_entry_besides_its_input
            + self.bytes_per_chunk_besides_the_entries
    }
}

/// 一张卡上一段装得下多少：显存上限（不给是不设限）、单个缓冲能绑定的最大字节数、一次派发最多几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkBoundsOfAGpu {
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub largest_storage_binding_in_bytes: u64,
    pub largest_entry_count_per_dispatch: u64,
}

impl ChunkBoundsOfAGpu {
    /// 这张卡上已经有 `bytes_already_held_on_the_gpu` 字节被本包别的段占着时，再切一段能用的界：显存上限扣掉已占的（第 3 条：全部后台任务
    /// 在那张卡上合占的显存不超过上限，三段同时在一张卡上时一起算），绑定上限与派发上限不变。不给上限的照旧不设限；已占的不比上限少时一字节都不剩。
    #[must_use]
    pub fn with_the_room_left_after(self, bytes_already_held_on_the_gpu: u64) -> Self {
        Self {
            memory_limit: self.memory_limit.map(|memory_limit| {
                GpuMemoryInBytes(memory_limit.0.saturating_sub(bytes_already_held_on_the_gpu))
            }),
            largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
        }
    }
}

/// 一条单独就装不进一段：条款只写了「一批放不下就切成几次交」，单个条目放不下怎么办没写，第一版不支持。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryLargerThanAChunkOnTheGpu {
    pub entry_position: usize,
    pub input_bytes: u64,
}

/// 把一批（每条的输入字节数按批里的次序给）切成相连的几段，每段在卡上合占的字节不超过上限、输入与按条排的缓冲都装得进绑定上限、
/// 条数不超过一次派发的上限。从前往后贪心：能装就装进当前这一段，装不下就另起一段。空批交回空。
///
/// # Errors
/// 某一条单独成一段也装不下（[`EntryLargerThanAChunkOnTheGpu`]），一段都不交。
pub fn chunks_that_fit_on_the_gpu(
    entry_input_lengths_in_bytes: &[u64],
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
) -> Result<Vec<std::ops::Range<usize>>, EntryLargerThanAChunkOnTheGpu> {
    let fits = |input_bytes: u64, entry_count: u64| -> bool {
        let within_the_memory_limit = bounds.memory_limit.is_none_or(|memory_limit| {
            footprint.bytes_held_by_a_chunk(input_bytes, entry_count) <= memory_limit.0
        });
        within_the_memory_limit
            && input_buffer_bytes(input_bytes) <= bounds.largest_storage_binding_in_bytes
            && entry_count * footprint.bytes_per_entry_in_the_largest_entry_buffer
                <= bounds.largest_storage_binding_in_bytes
            && entry_count <= bounds.largest_entry_count_per_dispatch
    };
    let mut chunks = Vec::new();
    let mut chunk_start = 0;
    let mut chunk_input_bytes: u64 = 0;
    for (entry_position, input_bytes) in entry_input_lengths_in_bytes.iter().copied().enumerate() {
        let entry_count_with_this_entry =
            u64::try_from(entry_position - chunk_start + 1).expect("条数装得进 u64");
        if fits(chunk_input_bytes + input_bytes, entry_count_with_this_entry) {
            chunk_input_bytes += input_bytes;
            continue;
        }
        if !fits(input_bytes, 1) {
            return Err(EntryLargerThanAChunkOnTheGpu {
                entry_position,
                input_bytes,
            });
        }
        chunks.push(chunk_start..entry_position);
        chunk_start = entry_position;
        chunk_input_bytes = input_bytes;
    }
    if chunk_start < entry_input_lengths_in_bytes.len() {
        chunks.push(chunk_start..entry_input_lengths_in_bytes.len());
    }
    Ok(chunks)
}

/// 一段的提交等完之后，它的缓冲在卡上还占不占显存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionOfTheChunk<Submission> {
    /// 等到它在 GPU 上做完了：这一段的缓冲随它丢掉，显存还回去。
    CompletedOnTheGpu,
    /// 没等到它做完（等超过上限，或等的时候出错）：GPU 上可能还在跑它，这一段的缓冲在卡上照旧占着显存。
    MayStillBeRunningOnTheGpu(Submission),
}

impl<Submission> SubmissionOfTheChunk<Submission> {
    /// 按等这次提交做完的结局判：等到了才算做完；超时与别的出错都当它可能还在跑（等的那一样报了错，GPU 上的提交不一定停了）。
    #[must_use]
    pub fn after_waiting_for_its_completion(
        submission: Submission,
        waited_for_its_completion: &Result<(), GpuFailureCause>,
    ) -> Self {
        match waited_for_its_completion {
            Ok(()) => Self::CompletedOnTheGpu,
            Err(_cause_of_the_failed_wait) => Self::MayStillBeRunningOnTheGpu(submission),
        }
    }
}

/// 等超过上限之后还在 GPU 上跑的那次提交，与它那一段在账上占着的字节。
#[derive(Debug)]
struct SubmissionRunningPastItsWait<Submission> {
    submission: Submission,
    bytes_held_by_its_chunk: u64,
}

/// 一张卡上切不出这一批分到它的几条（这张卡上一段都不交）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChunksNotCutOnTheGpu {
    /// 这张卡上前一次等超过上限的那次提交，这一次在上限之内又没等到它做完（或等的时候出错）：它的缓冲还占着卡上的显存，
    /// 不越过上限去交（第 3、4 条）。
    EarlierSubmissionStillRunning(GpuFailureCause),
    /// 一条单独就装不进一段。
    EntryLargerThanAChunk(EntryLargerThanAChunkOnTheGpu),
}

/// 一张卡上本包此刻占着的缓冲字节数与上限，连同等超过上限之后还在 GPU 上跑的那次提交。外壳建缓冲之前记、那一段的提交做完之后放。
#[derive(Debug)]
pub struct GpuMemoryAccount<Submission> {
    memory_limit: Option<GpuMemoryInBytes>,
    /// 合占的字节数，含还在跑的那次提交那一段。
    bytes_held: u64,
    /// 至多一次：它没做完之前这张卡不再交活（[`Self::hold`] 断言）。
    submission_running_past_its_wait: Option<SubmissionRunningPastItsWait<Submission>>,
}

impl<Submission> GpuMemoryAccount<Submission> {
    #[must_use]
    pub fn with_nothing_held(memory_limit: Option<GpuMemoryInBytes>) -> Self {
        Self {
            memory_limit,
            bytes_held: 0,
            submission_running_past_its_wait: None,
        }
    }

    #[must_use]
    pub fn bytes_held(&self) -> u64 {
        self.bytes_held
    }

    #[must_use]
    pub fn memory_limit(&self) -> Option<GpuMemoryInBytes> {
        self.memory_limit
    }

    /// 换上切换给的上限（第 1、3 条：从下一批起照它，外壳在两批之间调）。账不清：还占着的字节、等超过上限还在跑的那次提交照旧记着，
    /// 下一批交活之前先等它做完、放掉它的账，再按新上限扣掉还占着的切。
    pub fn use_the_memory_limit_from_the_next_batch(
        &mut self,
        memory_limit: Option<GpuMemoryInBytes>,
    ) {
        self.memory_limit = memory_limit;
    }

    /// 记下要建的那个缓冲。
    ///
    /// # Panics
    /// 这张卡上还有一次等超过上限、没做完的提交：交活之前先等它（[`Self::chunks_cut_once_the_submission_running_past_its_wait_completed`]），
    /// 走到这里是外壳跳过了那一步。
    /// 记下之后超过上限：切批（[`chunks_that_fit_on_the_gpu`]）按同一个上限切过，超了是切批与建缓冲对不上的 bug。
    pub fn hold(&mut self, bytes: u64) {
        assert!(
            self.submission_running_past_its_wait.is_none(),
            "这张卡上还有一次等超过上限、没做完的提交：交下一段之前先等它做完"
        );
        let bytes_held_afterwards = self.bytes_held + bytes;
        if let Some(memory_limit) = self.memory_limit {
            assert!(
                bytes_held_afterwards <= memory_limit.0,
                "本包在这张卡上占的缓冲 {bytes_held_afterwards} 字节超过了上限 {} 字节：切批按同一个上限切过，这里超了是 bug",
                memory_limit.0
            );
        }
        self.bytes_held = bytes_held_afterwards;
    }

    /// 放掉丢掉的那几个缓冲。
    ///
    /// # Panics
    /// 放的比记的多。
    pub fn release(&mut self, bytes: u64) {
        assert!(bytes <= self.bytes_held, "放掉的缓冲比记下的多");
        self.bytes_held -= bytes;
    }

    /// 一段的提交等完之后结它的账：做完了就放掉这一段；没等到它做完的，账照旧记着，记成这张卡上还在跑的那次提交，
    /// 等它的完成信号来了才放（第 3 条：任何时刻都不许比上限多）。
    ///
    /// # Panics
    /// 放的比记的多；或这张卡上已经记着一次还在跑的提交（交这一段之前 [`Self::hold`] 断言过没有）。
    pub fn settle_the_chunk_after_its_submission(
        &mut self,
        bytes_held_by_the_chunk: u64,
        submission_of_the_chunk: SubmissionOfTheChunk<Submission>,
    ) {
        match submission_of_the_chunk {
            SubmissionOfTheChunk::CompletedOnTheGpu => self.release(bytes_held_by_the_chunk),
            SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(submission) => {
                assert!(
                    bytes_held_by_the_chunk <= self.bytes_held,
                    "还在跑的那一段比记下的多"
                );
                assert!(
                    self.submission_running_past_its_wait.is_none(),
                    "这张卡上已经记着一次还在跑的提交：交这一段之前没有等它"
                );
                self.submission_running_past_its_wait = Some(SubmissionRunningPastItsWait {
                    submission,
                    bytes_held_by_its_chunk: bytes_held_by_the_chunk,
                });
            }
        }
    }

    /// 这张卡上那次等超过上限、还在跑的提交，在 `deadline` 之内等它做完（`check_once_waiting_at_most` 看一次它做没做完、最多等交给它的那么久）；
    /// 做完放掉它那一段的账。没有这样的提交就当场交回。
    ///
    /// # Errors
    /// 等满上限还没做完 ⇒ [`GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`]；看的时候报的出错照原样交回。两种都不放账。
    fn wait_for_the_submission_running_past_its_wait(
        &mut self,
        deadline: &GpuWaitDeadline,
        clock: &impl MonotonicClock,
        mut check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
    ) -> Result<(), GpuFailureCause> {
        let Some(running) = self.submission_running_past_its_wait.as_ref() else {
            return Ok(());
        };
        let waited = wait_for_completion_before_the_deadline(deadline, clock, |time_left| {
            check_once_waiting_at_most(&running.submission, time_left)
        });
        if let Err(cause) = waited {
            return Err(failure_of_waiting_for_the_earlier_submission(cause));
        }
        let completed = self
            .submission_running_past_its_wait
            .take()
            .expect("上面刚看过这张卡上记着一次还在跑的提交");
        self.release(completed.bytes_held_by_its_chunk);
        Ok(())
    }

    /// 这张卡分到的几条（按批里的次序给每条的输入字节数）切段，在交这几段之前调（第 3、4 条）：分到了活而这张卡上还有一次等超过上限、
    /// 没做完的提交，先在 `deadline` 之内等它做完、放掉它的账，再按上限扣掉此刻还占着的切（[`chunks_that_fit_on_the_gpu`]）；
    /// 没分到活的不等（这一批不在这张卡上交活）。
    ///
    /// # Errors
    /// 等不到它做完 ⇒ [`ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning`]，一段都不切、账照旧记着；
    /// 一条单独装不下 ⇒ [`ChunksNotCutOnTheGpu::EntryLargerThanAChunk`]。
    pub fn chunks_cut_once_the_submission_running_past_its_wait_completed(
        &mut self,
        deadline: &GpuWaitDeadline,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
        entry_input_lengths_in_bytes: &[u64],
        footprint: ChunkFootprint,
        bounds: ChunkBoundsOfAGpu,
    ) -> Result<Vec<std::ops::Range<usize>>, ChunksNotCutOnTheGpu> {
        if entry_input_lengths_in_bytes.is_empty() {
            return Ok(Vec::new());
        }
        self.wait_for_the_submission_running_past_its_wait(
            deadline,
            clock,
            check_once_waiting_at_most,
        )
        .map_err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning)?;
        chunks_that_fit_on_the_gpu(
            entry_input_lengths_in_bytes,
            footprint,
            bounds.with_the_room_left_after(self.bytes_held),
        )
        .map_err(ChunksNotCutOnTheGpu::EntryLargerThanAChunk)
    }
}

/// 等前一次超时的那次提交又等满了上限：报成它自己的那一种，与这一次交的段自己等超时分得开；别的出错照原样。
fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuFailureCause {
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
            GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit }
        }
        reported_by_the_wait @ (GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)) => reported_by_the_wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// 每条另占 40 字节、每段另占 16 字节、按条排的最大缓冲每条 16 字节：校验和核对那一段的量级，数是测试自己定的。
    const FOOTPRINT_UNDER_TEST: ChunkFootprint = ChunkFootprint {
        bytes_per_chunk_besides_the_entries: 16,
        bytes_per_entry_besides_its_input: 40,
        bytes_per_entry_in_the_largest_entry_buffer: 16,
    };

    fn bounds(memory_limit_in_bytes: Option<u64>) -> ChunkBoundsOfAGpu {
        ChunkBoundsOfAGpu {
            memory_limit: memory_limit_in_bytes.map(GpuMemoryInBytes),
            largest_storage_binding_in_bytes: 1 << 30,
            largest_entry_count_per_dispatch: 1 << 20,
        }
    }

    /// 用例里交给 GPU 的一次提交：只是个编号。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SubmissionNamedByTheTest(u32);

    /// 等超时的那一次提交。
    const TIMED_OUT_SUBMISSION: SubmissionNamedByTheTest = SubmissionNamedByTheTest(1);
    /// 等的上限：用例自己定的数。
    const WAIT_LIMIT_UNDER_TEST: Duration = Duration::from_secs(5);
    /// 这张卡的显存上限：用例自己定的数。
    const MEMORY_LIMIT_UNDER_TEST_IN_BYTES: u64 = 1000;
    /// 等超时的那一段占的字节。
    const BYTES_HELD_BY_THE_TIMED_OUT_CHUNK: u64 = 600;

    /// 假钟：只在假完成信号让它走时才走。
    struct ClockMovedByTheCompletionSignal {
        now: Cell<Duration>,
    }

    impl MonotonicClock for ClockMovedByTheCompletionSignal {
        fn time_since_the_origin(&self) -> Duration {
            self.now.get()
        }
    }

    /// 假完成信号：`submission` 在钟走到 `completes_at` 那一刻在 GPU 上做完。每看一次钟走一秒（最多走到期限，至少走 1 纳秒：
    /// 被测的写错时用例照样走得完、在断言上红）；看的不是这一次提交就断言。
    fn completion_signal_of(
        clock: &ClockMovedByTheCompletionSignal,
        submission: SubmissionNamedByTheTest,
        completes_at: Duration,
    ) -> impl FnMut(&SubmissionNamedByTheTest, Duration) -> CompletionCheck<()> + '_ {
        move |submission_checked, time_left| {
            assert_eq!(*submission_checked, submission, "等的是超时的那一次提交");
            let waited_this_check = Duration::from_secs(1)
                .min(time_left)
                .max(Duration::from_nanos(1));
            clock.now.set(clock.now.get() + waited_this_check);
            if clock.now.get() >= completes_at {
                CompletionCheck::Completed(())
            } else {
                CompletionCheck::NotYetComplete
            }
        }
    }

    /// 照外壳交一段的次序走一遍：钟在 100 秒，记下 600 字节的一段、交、在 5 秒上限之内等它做完；那次提交到 `completes_at`（晚于 105 秒）
    /// 才在 GPU 上做完，等满上限交回超时出错；再照等的结局结这一段的账。交回账与钟（停在 105 秒）。
    fn account_after_a_chunk_whose_wait_timed_out(
        completes_at: Duration,
    ) -> (
        GpuMemoryAccount<SubmissionNamedByTheTest>,
        ClockMovedByTheCompletionSignal,
    ) {
        assert!(completes_at > Duration::from_secs(105), "要等超时");
        let mut account = GpuMemoryAccount::with_nothing_held(Some(GpuMemoryInBytes(
            MEMORY_LIMIT_UNDER_TEST_IN_BYTES,
        )));
        let clock = ClockMovedByTheCompletionSignal {
            now: Cell::new(Duration::from_secs(100)),
        };
        account.hold(BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        let waited = {
            let mut completion_signal =
                completion_signal_of(&clock, TIMED_OUT_SUBMISSION, completes_at);
            wait_for_completion_before_the_deadline(
                &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
                &clock,
                |time_left| completion_signal(&TIMED_OUT_SUBMISSION, time_left),
            )
        };
        assert_eq!(
            waited,
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: WAIT_LIMIT_UNDER_TEST
            }),
            "等满 5 秒还没做完：这一批交回超时出错"
        );
        let submission_of_the_chunk =
            SubmissionOfTheChunk::after_waiting_for_its_completion(TIMED_OUT_SUBMISSION, &waited);
        assert_eq!(
            submission_of_the_chunk,
            SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(TIMED_OUT_SUBMISSION),
            "没等到做完的提交可能还在 GPU 上跑"
        );
        account.settle_the_chunk_after_its_submission(
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            submission_of_the_chunk,
        );
        assert_eq!(clock.now.get(), Duration::from_secs(105));
        (account, clock)
    }

    /// 第 3 条在等超时之后：一段的提交等超过上限、这一批按出错交回之后，GPU 上可能还在跑它，这一段的缓冲照旧占着卡上的显存。
    /// 账上照旧记着它，这期间切批扣掉它；完成信号来了才放。
    #[test]
    fn a_chunk_whose_wait_timed_out_stays_held_until_its_submission_completes_on_the_gpu() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "超时之后、完成之前：这一段在账上照旧占着 600 字节"
        );
        let lengths = [100_u64, 100, 100, 100, 100];
        let bounds_of_the_gpu = bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES));
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths,
                FOOTPRINT_UNDER_TEST,
                bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
            ),
            Ok(vec![0..2, 2..4, 4..5]),
            "这期间切批只用剩下的 400 字节；超时就放账的话整批一段 716 字节，与卡上还占着的 600 合 1316"
        );
        // 下一批在 105 秒来了，那次提交在 107 秒做完：切段之前等到了它、放掉它的账，再按整个上限切
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(107)),
            &lengths,
            FOOTPRINT_UNDER_TEST,
            bounds_of_the_gpu,
        );
        assert_eq!(
            chunks,
            Ok(std::iter::once(0..5).collect::<Vec<_>>()),
            "那次提交做完、账放了：五条 716 字节一段"
        );
        assert_eq!(
            clock.now.get(),
            Duration::from_secs(107),
            "等到它做完那一刻"
        );
        assert_eq!(account.bytes_held(), 0, "完成信号来了才放");
    }

    /// 第 3、4 条：前一次等超时的那次提交还没在 GPU 上做完时，下一批不在这张卡上交活：切段之前在上限之内等它，等满上限还没做完就交回出错，
    /// 一段都不切，它的账照旧记着。
    #[test]
    fn a_gpu_whose_timed_out_submission_is_still_running_gets_no_chunk_of_the_next_batch() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(200));
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(200)),
            &[100, 100, 100, 100, 100],
            FOOTPRINT_UNDER_TEST,
            bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
        );
        assert_eq!(
            chunks,
            Err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: WAIT_LIMIT_UNDER_TEST
                }
            )),
            "那次提交要到 200 秒才做完，下一批在 105 秒来、等满 5 秒：这张卡上一段都不交，交回出错"
        );
        assert_eq!(clock.now.get(), Duration::from_secs(110), "等满 5 秒");
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "它的账照旧记着"
        );
        assert!(
            account.submission_running_past_its_wait.is_some(),
            "它仍记成还在跑的那次提交"
        );
    }

    /// 外壳要是跳过了等那一步就交下一段：记账那一刻断言，不带着可能超限的显存往下交。
    #[test]
    #[should_panic(expected = "交下一段之前先等它做完")]
    fn holding_a_chunk_while_a_timed_out_submission_is_still_running_panics() {
        let (mut account, _clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        account.hold(100);
    }

    /// 这一批没分到这张卡的活：不在这张卡上交活，就不等它前一次超时的那次提交；一段不切，账照旧记着。
    #[test]
    fn a_gpu_given_no_entries_of_the_next_batch_does_not_wait_for_its_timed_out_submission() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            |_submission, _time_left| panic!("没分到活的卡不该等"),
            &[],
            FOOTPRINT_UNDER_TEST,
            bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
        );
        assert_eq!(chunks, Ok(Vec::new()));
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "没等，账照旧记着"
        );
    }

    #[test]
    fn input_buffers_round_up_to_whole_words_and_never_have_zero_bytes() {
        assert_eq!(input_buffer_bytes(0), 4);
        assert_eq!(input_buffer_bytes(1), 4);
        assert_eq!(input_buffer_bytes(4), 4);
        assert_eq!(input_buffer_bytes(5), 8);
        assert_eq!(input_buffer_bytes(4097), 4100);
    }

    #[test]
    fn without_a_memory_limit_a_batch_is_one_chunk() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[4096, 4096, 7, 0], FOOTPRINT_UNDER_TEST, bounds(None)),
            Ok(std::iter::once(0..4).collect::<Vec<_>>()),
            "不给上限就不设限：整批一段"
        );
    }

    #[test]
    fn a_batch_that_does_not_fit_the_memory_limit_is_cut_into_chunks_that_each_fit() {
        let lengths = [100_u64, 100, 100, 100, 100];
        // 两条一段：输入 200 字节、条目 80、参数 16，合 296；三条要 444
        let memory_limit_in_bytes = 300;
        let chunks = chunks_that_fit_on_the_gpu(
            &lengths,
            FOOTPRINT_UNDER_TEST,
            bounds(Some(memory_limit_in_bytes)),
        )
        .expect("单条 156 字节装得下");
        assert_eq!(chunks, vec![0..2, 2..4, 4..5], "两条一段，最后剩一条");
        for chunk in &chunks {
            let input_bytes: u64 = lengths[chunk.clone()].iter().sum();
            let held = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                input_bytes,
                u64::try_from(chunk.len()).expect("装得进 u64"),
            );
            assert!(
                held <= memory_limit_in_bytes,
                "每一段合占 {held} 字节不超过上限 {memory_limit_in_bytes}"
            );
        }
    }

    #[test]
    fn a_chunk_exactly_at_the_memory_limit_is_kept_whole() {
        // 两条 100 字节：200 + 80 + 16 = 296，正好等于上限也算装得下
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[100, 100], FOOTPRINT_UNDER_TEST, bounds(Some(296))),
            Ok(std::iter::once(0..2).collect::<Vec<_>>())
        );
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[100, 100], FOOTPRINT_UNDER_TEST, bounds(Some(295))),
            Ok(vec![0..1, 1..2]),
            "少一个字节就切开"
        );
    }

    #[test]
    fn the_dispatch_and_binding_limits_also_cut_chunks() {
        let dispatch_bound = ChunkBoundsOfAGpu {
            memory_limit: None,
            largest_storage_binding_in_bytes: 1 << 30,
            largest_entry_count_per_dispatch: 3,
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[1; 7], FOOTPRINT_UNDER_TEST, dispatch_bound),
            Ok(vec![0..3, 3..6, 6..7]),
            "一次派发最多 3 条"
        );
        let binding_bound = ChunkBoundsOfAGpu {
            memory_limit: None,
            largest_storage_binding_in_bytes: 8,
            largest_entry_count_per_dispatch: 1 << 20,
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[4, 4, 4], FOOTPRINT_UNDER_TEST, binding_bound),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 0,
                input_bytes: 4
            }),
            "一条在按条排的缓冲里就占 16 字节，超过 8 字节的绑定上限"
        );
    }

    /// 两段同时在一张卡上（第 3 条：全部后台任务在那张卡上合占的显存不超过上限）：另一段先占着 600 字节，这一段再切时只用剩下的 400 字节，
    /// 两段合占任何时刻都不超过上限 1000。
    #[test]
    fn two_segments_held_on_one_gpu_together_stay_within_its_memory_limit() {
        let memory_limit_in_bytes = 1000;
        let bounds_of_the_gpu = bounds(Some(memory_limit_in_bytes));
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(memory_limit_in_bytes),
        ));
        // 先占着的那一段：每段另占 200 字节、每条另占 50 字节的四条 50 字节，合 200 + 200 + 200 = 600
        let footprint_of_the_other_segment = ChunkFootprint {
            bytes_per_chunk_besides_the_entries: 200,
            bytes_per_entry_besides_its_input: 50,
            bytes_per_entry_in_the_largest_entry_buffer: 50,
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &[50, 50, 50, 50],
                footprint_of_the_other_segment,
                bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
            ),
            Ok(std::iter::once(0..4).collect::<Vec<_>>())
        );
        let held_by_the_other_segment =
            footprint_of_the_other_segment.bytes_held_by_a_chunk(200, 4);
        assert_eq!(held_by_the_other_segment, 600);
        account.hold(held_by_the_other_segment);

        let lengths = [100_u64, 100, 100, 100, 100];
        assert_eq!(
            chunks_that_fit_on_the_gpu(&lengths, FOOTPRINT_UNDER_TEST, bounds_of_the_gpu),
            Ok(std::iter::once(0..5).collect::<Vec<_>>()),
            "单看这一段，五条合 716 字节装得进 1000：不扣另一段占的就整批一段，两段合占 1316 字节"
        );
        let chunks = chunks_that_fit_on_the_gpu(
            &lengths,
            FOOTPRINT_UNDER_TEST,
            bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
        )
        .expect("单条 156 字节装得进剩下的 400 字节");
        assert_eq!(
            chunks,
            vec![0..2, 2..4, 4..5],
            "只用剩下的 400 字节：两条 296 一段，三条 436 装不下"
        );
        for chunk in &chunks {
            let input_bytes: u64 = lengths[chunk.clone()].iter().sum();
            let held_by_this_chunk = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                input_bytes,
                u64::try_from(chunk.len()).expect("装得进 u64"),
            );
            account.hold(held_by_this_chunk);
            assert!(
                account.bytes_held() <= memory_limit_in_bytes,
                "两段合占 {} 字节不超过上限 {memory_limit_in_bytes}",
                account.bytes_held()
            );
            account.release(held_by_this_chunk);
        }
        assert_eq!(
            account.bytes_held(),
            held_by_the_other_segment,
            "这一段交完放掉，另一段的照旧占着"
        );
    }

    #[test]
    fn the_room_left_after_other_segments_without_a_memory_limit_is_still_unlimited() {
        assert_eq!(
            bounds(None).with_the_room_left_after(1 << 40),
            bounds(None),
            "不给上限就不设限，别的段占多少都一样"
        );
        assert_eq!(
            bounds(Some(100)).with_the_room_left_after(150).memory_limit,
            Some(GpuMemoryInBytes(0)),
            "已占的比上限多：一字节都不剩"
        );
    }

    #[test]
    fn an_entry_that_alone_exceeds_the_memory_limit_refuses_the_batch() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[10, 4096, 10], FOOTPRINT_UNDER_TEST, bounds(Some(1024))),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 1,
                input_bytes: 4096
            })
        );
    }

    #[test]
    fn an_empty_batch_has_no_chunk() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(&[], FOOTPRINT_UNDER_TEST, bounds(Some(64))),
            Ok(Vec::new())
        );
    }

    #[test]
    fn holding_up_to_the_memory_limit_is_allowed_and_releasing_gives_it_back() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(100),
        ));
        account.hold(60);
        account.hold(40);
        assert_eq!(account.bytes_held(), 100, "正好到上限");
        account.release(100);
        account.hold(100);
        assert_eq!(account.bytes_held(), 100, "放掉之后又能占满");
    }

    #[test]
    #[should_panic(expected = "超过了上限")]
    fn holding_past_the_memory_limit_panics() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(100),
        ));
        account.hold(60);
        account.hold(41);
    }

    #[test]
    fn without_a_memory_limit_holding_is_never_refused() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(None);
        account.hold(u64::MAX / 2);
        assert_eq!(account.bytes_held(), u64::MAX / 2);
    }
}
