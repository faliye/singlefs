//! 后台入口的一次唤醒里停下记成零次唤醒，可写挂载续做里停下，之后三次唤醒各数成一、二、三。

use super::*;

/// 在后台入口上：在 GPU 上签发意图之后注入甲段出错，建一个 inode 记下「该跑了」，
/// 后台入口判过起跑、那一批出错放弃、整理停下——这一次叫起照报、不数这一趟（叫起 0 次）；之后后台入口连着叫起三次（停着、阈值取默认 85% 不超过，
/// 都不起跑）报 1、2、3。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn a_stop_in_a_wake_of_the_background_entry_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-stopped-in-a-wake-of-the-background-entry",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) = gpu_session_with_the_intent_issued_on_the_gpu(
        &mut pool,
        region,
        &gpu_compute,
        &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
    );
    fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
        &gpu_compute_kept_by_the_test,
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
    let stopping_call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let BackgroundCompactionCall::Ran { run, .. } = &stopping_call else {
        panic!("用户改动之后记下了「该跑了」：起跑了（{stopping_call:?}）");
    };
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ),
        "甲段出错：这一批放弃、整理停下（{:?}）",
        run.outcome
    );
    let error = error_of_the_failing_unit_checksum_verification();
    assert_eq!(
        stopping_call.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 0, 0)),
        "出错停下的那一次叫起照报、不数这一趟"
    );
    let wakes_after_the_stop: Vec<Option<u64>> = (0..3)
        .map(|_wake| {
            session
                .run_compaction_in_the_background(
                    &mut devices,
                    pacing_with_the_timer_fallback_off(),
                    &CallerWithNoForegroundFsync,
                )
                .expect("后台入口")
                .alarm_of_the_compaction_stopped_after_a_compute_error()
                .map(|alarm| alarm.wakes_while_stopped)
        })
        .collect();
    pool.devices = Some(devices);
    assert_eq!(
        wakes_after_the_stop,
        vec![Some(1), Some(2), Some(3)],
        "下一次叫起才是第 1 次"
    );
}

/// 在可写挂载续做上：另一次挂载里在 GPU 上签发意图之后丢掉会话
/// （意图留在盘上），注入甲段出错，再挂选 `gpu`、经只供测试的入口装上同一组整理开关——挂载续做推那一条意图，头一批在 GPU 上出错放弃、整理停下，
/// 续做只单发一次删那一条意图的发布、挂上之后盘上没有活意图；续做不是叫起，不交报警。挂上之后显式入口连着叫起三次（停着、阈值取默认 85% 不超过）报 1、2、3，
/// 与在叫起里停下那一种（`a_stop_in_a_wake_of_the_background_entry_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three`：
/// 停下那一次叫起报 0、下一次才是 1）同一个数法：停下之后的头一次叫起是第 1 次。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn a_stop_in_the_writable_mount_continuation_is_followed_by_three_wakes_counting_one_two_three() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-stopped-in-the-writable-mount-continuation",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (session_of_the_earlier_mount, devices_of_the_earlier_mount) =
        gpu_session_with_the_intent_issued_on_the_gpu(
            &mut pool,
            region,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    drop(session_of_the_earlier_mount);
    pool.devices = Some(devices_of_the_earlier_mount);
    fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
        &gpu_compute_kept_by_the_test,
    );
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            &parameters(),
            &mut devices,
            admissible_gpu_settings_naming_both_cards(),
            Some(gpu_compute.clone()),
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            compaction_switches_of_the_test(region),
        )
        .expect("全合法的 gpu 设置连同实现、默认阈值：挂载照常做成（续做自己不让挂载失败）");
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    let continuation = output
        .compaction_continuation
        .run
        .as_ref()
        .expect("盘上有一条未完成的意图、删除队列是空的：续做推了它");
    assert!(
        matches!(
            continuation.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && continuation.batches_published == 0
            && continuation.versions_published.len() == 1
            && continuation.versions_published.iter().all(|dropped| {
                dropped.units_relocated_by_this_publish.is_empty()
                    && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            }),
        "续做的头一批在 GPU 上算甲段出错：这一批放弃、一个单元都没搬，单发一次删意图的发布，整理停下（{:?}）",
        continuation.outcome
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_none(),
        "挂上之后盘上与分配器这一侧都没有活意图"
    );
    assert_eq!(
        continuation.alarm_of_the_compaction_stopped_after_a_compute_error, None,
        "续做不是叫起：不交报警"
    );
    let error = error_of_the_failing_unit_checksum_verification();
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: error.clone(),
            batches_compensated_on_the_cpu: 0,
        }],
        "挂上之后整理停着、报出甲段那张卡的错，还没补跑过"
    );
    let alarms_of_the_next_three_wakes: Vec<
        Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    > = (0..3)
        .map(|_wake| {
            session
                .run_compaction(&mut devices)
                .expect("整理一趟")
                .alarm_of_the_compaction_stopped_after_a_compute_error
        })
        .collect();
    pool.devices = Some(devices);
    assert_eq!(
        alarms_of_the_next_three_wakes,
        vec![
            Some(alarm_of_the_stop(&error, 1, 0)),
            Some(alarm_of_the_stop(&error, 2, 0)),
            Some(alarm_of_the_stop(&error, 3, 0)),
        ],
        "停在续做里：停下之后的头一次叫起是第 1 次"
    );
}
