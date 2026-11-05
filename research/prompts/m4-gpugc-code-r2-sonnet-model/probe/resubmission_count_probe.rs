//! 辩方腿探针（只读调用冻结副本里公开的 `waiting_for_the_gpu`）：重交那一圈里「等前一次超时的提交又超时」时，
//! 记账里的 `resubmissions()` 与真正交出去的重交次数是否一致。只打印，不断言（不改任何产品代码）。
use std::time::Duration;

use singlefs_background_gpu::background_gpu::GpuFailureCause;
use singlefs_background_gpu::waiting_for_the_gpu::{
    outcome_resubmitted_after_timeouts, TimeoutsOfTheBatchOnOneGpu,
};

const LIMIT: Duration = Duration::from_secs(10);

struct ScriptedGpu {
    submissions: Vec<Result<u32, GpuFailureCause>>,
    waits: Vec<Result<(), GpuFailureCause>>,
    submissions_made: usize,
    waits_made: usize,
}

fn this_submission_timed_out() -> GpuFailureCause {
    GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit: LIMIT }
}

fn earlier_submission_still_running() -> GpuFailureCause {
    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit: LIMIT }
}

fn run(
    name: &str,
    submissions: Vec<Result<u32, GpuFailureCause>>,
    waits: Vec<Result<(), GpuFailureCause>>,
) {
    let mut gpu = ScriptedGpu {
        submissions,
        waits,
        submissions_made: 0,
        waits_made: 0,
    };
    let mut timeouts_of_the_batch = TimeoutsOfTheBatchOnOneGpu::none_yet();
    let outcome = outcome_resubmitted_after_timeouts(
        &mut gpu,
        &mut timeouts_of_the_batch,
        |gpu: &mut ScriptedGpu| {
            let outcome = gpu.waits.get(gpu.waits_made).cloned().unwrap_or(Ok(()));
            gpu.waits_made += 1;
            outcome
        },
        |gpu: &mut ScriptedGpu| {
            let outcome = gpu
                .submissions
                .get(gpu.submissions_made)
                .cloned()
                .unwrap_or(Ok(0));
            gpu.submissions_made += 1;
            outcome
        },
    );
    println!(
        "PROBE name={name} outcome_is_ok={} submissions_made={} waits_made={} timeouts_recorded={} resubmissions_recorded={} resubmissions_really_made={}",
        outcome.is_ok(),
        gpu.submissions_made,
        gpu.waits_made,
        timeouts_of_the_batch.timeouts(),
        timeouts_of_the_batch.resubmissions(),
        gpu.submissions_made.saturating_sub(1),
    );
}

#[test]
fn the_wait_for_the_timed_out_submission_timing_out_once_before_the_resubmission() {
    run(
        "wait_times_out_once_then_resubmission_completes",
        vec![Err(this_submission_timed_out()), Ok(0)],
        vec![Err(earlier_submission_still_running()), Ok(())],
    );
}

#[test]
fn the_wait_for_the_timed_out_submission_timing_out_three_times_after_one_timeout() {
    run(
        "wait_times_out_three_times_no_resubmission_ever_made",
        vec![Err(this_submission_timed_out())],
        vec![
            Err(earlier_submission_still_running()),
            Err(earlier_submission_still_running()),
            Err(earlier_submission_still_running()),
        ],
    );
}

#[test]
fn the_plain_case_for_comparison_three_timeouts_then_completion() {
    run(
        "three_submissions_time_out_fourth_completes",
        vec![
            Err(this_submission_timed_out()),
            Err(this_submission_timed_out()),
            Err(this_submission_timed_out()),
            Ok(0),
        ],
        vec![Ok(()), Ok(()), Ok(())],
    );
}
