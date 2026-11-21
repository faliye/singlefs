//! 会话有待落的改动时后台入口的报警：不起跑、补跑一批、排空待落改动被拒（每次写都报错的盘）这三种都交回报警。

use super::*;

/// 攒批的会话（`MountedSessionWithPendingChanges`）的后台入口同样每次叫起都报：停着、阈值取默认 85%（不超过），
/// 没起跑的那一次交回报警——甲段那张卡、停下的那一个错、叫起 1 次（停下那一趟不数）、补跑 0 批。
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_while_it_does_not_start(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-not-started",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (session, mut devices, error) = gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
        &mut pool,
        region,
        &gpu_compute_kept_by_the_test,
        &gpu_compute,
        &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
    );
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    let after_the_pending_changes = session_with_pending_changes
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            after_the_pending_changes.call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "停着而不超过阈值：不起跑（{:?}）",
        after_the_pending_changes.call
    );
    assert_eq!(
        after_the_pending_changes
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 0)),
        "没起跑的那一次叫起照报"
    );
}

/// 同上，阈值 0%（超过）、用户改动之后记下「该跑了」：攒批的会话的后台入口起跑，从头签发那一次与那一批都在 CPU 上补跑，起跑了的那一次也交回报警
/// （叫起 1 次、补跑 2 批）。
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_when_it_runs_a_compensating_batch(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-compensating",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    let after_the_pending_changes = session_with_pending_changes
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { run, .. } = &after_the_pending_changes.call else {
        panic!(
            "停着而超过阈值：起跑了（{:?}）",
            after_the_pending_changes.call
        );
    };
    assert!(
        run.batches_published == 1 && run.background_compute_of_each_batch.len() == 2,
        "用户改动之后那一趟推至多一批：出错停下时删了意图，先从头签发、再一批，各一份记账（{:?}）",
        run.outcome
    );
    assert_eq!(
        after_the_pending_changes
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 2)),
        "起跑了、补跑了签发那一次与一批的那一次叫起照报"
    );
}

/// 判决 m4-gpugc-code-r2 第三节第 3 条：攒批的会话的后台入口判过起跑、排空待发被拒的那一次也是一次叫起（不是入口在任何读写之前就拒的那一种），
/// 照数一次叫起、拒绝里带着这一次的报警。停着、阈值 0%（超过）：出错停下的那一趟报 0；后台入口一次没起跑（没有「该跑了」、定时兜底关着）报 1；
/// 建一个 inode 记下「该跑了」，攒批的会话收下一次建 inode 进待发，后台入口判过起跑、排空那一次写报 I/O 错被拒——拒绝里的报警：
/// 甲段那张卡、停下的那一个错、叫起接着数到 2、补跑 0 批（那一趟整理没跑）。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_when_draining_the_pending_changes_is_refused(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-drain-refused",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    let not_started = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    assert!(
        matches!(
            not_started,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::NothingDueAndTheTimerFallbackOff,
                ..
            }
        ),
        "没有「该跑了」、定时兜底关着：不起跑（{not_started:?}）"
    );
    assert_eq!(
        not_started.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 0)),
        "停下之后头一次叫起报 1"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    session_with_pending_changes
        .accept(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
            CallerMonotonicClockReading(Duration::ZERO),
        )
        .expect("建一个 inode 收进待发");
    assert_eq!(
        session_with_pending_changes.pending_changes_accepted(),
        1,
        "收进待发、两个阈值都没到：没发布"
    );
    let mut devices_failing_every_write: Vec<(DeviceIdentity, DeviceFailingEveryWrite)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, DeviceFailingEveryWrite { inner: device }))
        .collect();
    let drained = session_with_pending_changes.run_compaction_in_the_background(
        &mut devices_failing_every_write,
        pacing_with_the_timer_fallback_off(),
        &CallerWithNoForegroundFsync,
    );
    pool.devices = Some(
        devices_failing_every_write
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    match drained {
        Err(
            BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused {
                refusal: _,
                alarm_of_the_compaction_stopped_after_a_compute_error,
            },
        ) => assert_eq!(
            alarm_of_the_compaction_stopped_after_a_compute_error,
            Some(alarm_of_the_stop(&error, 2, 0)),
            "排空被拒的那一次叫起照数、照报：接着数到 2"
        ),
        Err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused(refused)) => {
            panic!("后台入口自己在任何读写之前拒了：{refused:?}")
        }
        Ok(after_the_pending_changes) => {
            panic!("排空那一次写报错应当被拒：{after_the_pending_changes:?}")
        }
    }
}
