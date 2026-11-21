use super::*;

/// R2-2：校准作废之后头两次读 1.30、1.25，第三、四次 1.02、1.03，等到第四次（40 秒）才回去重量；假钟与假校准源，不真睡。
/// 别的原因（挂载失败）当场重量：不睡、不校准。
#[test]
fn waiting_retries_only_after_two_normal_calibrations_in_a_row() {
    let mut ledger = WaitingLedger {
        total_seconds: 0,
        longest_seconds: 0,
        waits: 0,
    };
    let mut ratios = [1.30, 1.25, 1.02, 1.03].into_iter();
    let mut calibrate = || ratios.next();
    let mut slept: Vec<Duration> = Vec::new();
    let mut sleep = |duration: Duration| slept.push(duration);
    let wait = wait_before_the_retry(
        CALIBRATION_RATIO_INVALID_REASON,
        &mut ledger,
        &mut calibrate,
        &mut sleep,
    )
    .expect("校准比作废的要先等常态");
    assert_eq!(wait.checks, 4);
    assert_eq!(wait.waited_seconds, 40);
    assert_eq!(wait.outcome, NormalWaitOutcome::BackToNormal);
    assert_eq!(slept, vec![Duration::from_secs(10); 4]);
    assert_eq!(
        ledger,
        WaitingLedger {
            total_seconds: 40,
            longest_seconds: 40,
            waits: 1,
        }
    );
    let mut calibration_not_expected =
        || -> Option<f64> { panic!("当场重量的原因不做校准") };
    let mut sleep_not_expected = |_duration: Duration| panic!("当场重量的原因不睡");
    assert!(wait_before_the_retry(
        "mount_failed",
        &mut ledger,
        &mut calibration_not_expected,
        &mut sleep_not_expected,
    )
    .is_none());
    assert!(is_retried_only_after_the_machine_is_back_to_normal(
        "unaccounted_time_above_ten_percent_of_the_wall"
    ));
}

/// R2-2 的上限：校准一直读 1.30，头一次等满 300 秒照样重量；一段累计到 600 秒就停这一段、退 5。
#[test]
fn waiting_stops_the_segment_after_six_hundred_seconds_in_total_with_exit_code_five() {
    let mut ledger = WaitingLedger {
        total_seconds: 0,
        longest_seconds: 0,
        waits: 0,
    };
    let mut calibrate = || Some(1.30);
    let mut sleep = |_duration: Duration| {};
    let wait_reaching_the_one_wait_cap = wait_before_the_retry(
        CALIBRATION_RATIO_INVALID_REASON,
        &mut ledger,
        &mut calibrate,
        &mut sleep,
    )
    .expect("要等");
    assert_eq!(
        wait_reaching_the_one_wait_cap.outcome,
        NormalWaitOutcome::OneWaitCapReached
    );
    assert_eq!(wait_reaching_the_one_wait_cap.checks, 30);
    assert_eq!(wait_reaching_the_one_wait_cap.waited_seconds, 300);
    assert_eq!(
        wait_reaching_the_one_wait_cap
            .outcome
            .exit_code_stopping_the_segment(),
        None
    );
    let wait_reaching_the_segment_cap = wait_before_the_retry(
        CALIBRATION_RATIO_INVALID_REASON,
        &mut ledger,
        &mut calibrate,
        &mut sleep,
    )
    .expect("要等");
    assert_eq!(
        wait_reaching_the_segment_cap.outcome,
        NormalWaitOutcome::SegmentWaitCapReached
    );
    assert_eq!(ledger.total_seconds, 600);
    assert_eq!(
        wait_reaching_the_segment_cap
            .outcome
            .exit_code_stopping_the_segment(),
        Some(5)
    );
}
