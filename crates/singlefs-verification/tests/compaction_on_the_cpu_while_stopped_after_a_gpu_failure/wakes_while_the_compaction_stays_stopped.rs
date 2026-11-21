//! 用户删除把读数降下来之后补跑从下一批起停下；停着的每一次唤醒都交回报警，直到切换解除停下。

use super::*;

/// 「补跑途中降到不超过，下一批起照旧停着」，不经只供测试的补跑开关：停着、阈值 3%，第一块盘（R 那块）的占用抬过 3%；
/// 复用窗口压成 0（只供测试，释放那一刻就回收，用户删文件当场让已分配降下来）。先补跑两趟，各推一次（预算 1）：头一趟在同一个 R 上从头签发；
/// 签发那一版回收了 defer 队列里待回收的槽、第一块盘可能落回 3% 以内，用例在已标的那一段前面再标几槽、抬回刚好越线，第二趟补跑一批。
/// 再由用户真删第一个文件——读数降到不超过 3%；下一趟开头就照旧停着：一批都不开，只单发一次删掉补跑签下的意图的发布，补跑的批数照旧是 2。
/// 补跑过一批就一直补跑下去（攻方锁存那一种改坏）这里就红。
#[test]
fn a_compensating_compaction_stays_stopped_from_the_next_batch_once_a_user_deletion_brings_the_reading_down(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensation-ended-by-a-user-deletion",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    session
        .allocator
        .set_reuse_window(ReuseWindow::ForcedToZero);
    let slots_marked_at_the_tail = allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(0),
        threshold.allocated_percent_of_the_capacity,
    );
    assert!(
        session
            .gc_cpu_compensation_reading()
            .is_the_allocated_share_above_the_threshold(),
        "第一块盘越线：停着时补跑"
    );
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            issuing.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && issuing.batches_published == 0
            && issuing.versions_published.len() == 1
            && session
                .allocator
                .compaction()
                .live_intent()
                .is_some_and(|live| live.intent.region == region),
        "补跑的头一趟在同一个 R 上从头签发：{:?}，发了 {} 版",
        issuing.outcome,
        issuing.versions_published.len()
    );
    let reading_after_the_issue = session.gc_cpu_compensation_reading();
    let slots_marked_after_the_issue =
        allocate_slots_below_the_marked_tail_until_just_above_percent(
            &mut session,
            DeviceIdentity(0),
            threshold.allocated_percent_of_the_capacity,
            slots_marked_at_the_tail,
        );
    assert!(
        session
            .gc_cpu_compensation_reading()
            .is_the_allocated_share_above_the_threshold(),
        "签发之后第一块盘抬回刚好越线（签发之后的读数 {reading_after_the_issue:?}，又标了 {slots_marked_after_the_issue} 槽）"
    );
    let compensating = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            compensating.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && compensating.batches_published == 1,
        "补跑的第二趟发了一批：{:?}",
        compensating.outcome
    );
    assert_eq!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .iter()
            .map(|stopped| stopped.batches_compensated_on_the_cpu)
            .collect::<Vec<u64>>(),
        vec![2],
        "补跑了签发那一次与一批"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("停着时用户照常删第一个文件");
    let reading_after_the_deletion = session.gc_cpu_compensation_reading();
    assert!(
        !reading_after_the_deletion.is_the_allocated_share_above_the_threshold(),
        "删了文件、当场回收：读数降到不超过 3%（{reading_after_the_deletion:?}）"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.background_compute_of_each_batch.is_empty()
            && run.versions_published.len() == 1
            && run.versions_published.iter().all(|dropped| {
                dropped.units_relocated_by_this_publish.is_empty()
                    && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            }),
        "降到不超过之后下一批起照旧停着：一批都不开，只单发一次删意图的发布：{:?}，发了 {} 版",
        run.outcome,
        run.versions_published.len()
    );
    assert!(
        session.allocator.compaction().live_intent().is_none(),
        "补跑签下的那一条连同 R 排他一起撤了"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 2,
        }],
        "整理照旧停着、报的还是那个错，补跑的批数照旧是 2"
    );
}

/// 三次失败后每次后台整理被叫起都报、不管后台有没有 gpu 可用，经返回值里必须处理的报警交回：
/// 出错停下（那一趟照报、不数，报 0）之后连着叫起三次——显式入口一趟在 CPU 上补跑一批（预算 1：从头签发那一次；
/// 阈值 0%，这一批的钩子里调只供测试的开关，之后都当不超过阈值）、显式入口一趟照旧停着（补跑那一趟扣完了上一次用户改动给的那一份 `B_gc`，
/// 显式入口每一趟的预算照旧至少 1：一趟开头单发一次只删意图的发布丢掉补跑签下的意图）、后台入口一次照旧停着——三次都交回报警，逐字段：
/// 甲段那张卡、停下的那一个错、叫起 1、2、3 次、补跑 1 批；
/// 一次通过校验的切换（选 `cpu`）解除停下之后，显式入口再叫起不交报警。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn every_wake_while_the_compaction_stays_stopped_after_a_gpu_failure_returns_an_alarm_until_a_switch_lifts_the_stop(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-on-every-wake-while-stopped",
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
    session.compaction.publishes_since_the_last_user_change = BUDGET_GIVEN_BY_A_USER_CHANGE - 1;
    let compensating = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut judge_the_next_batches_not_above_the_threshold,
        )
        .expect("整理一趟");
    assert_eq!(
        compensating.background_compute_of_each_batch.len(),
        1,
        "超过阈值 0%：在 CPU 上补跑了一批（{:?}）",
        compensating.outcome
    );
    assert_eq!(
        compensating.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(alarm_of_the_stop(&error, 1, 1)),
        "补跑了一批的那一次叫起照报"
    );
    let left_stopped = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            left_stopped.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && left_stopped.batches_published == 0
            && left_stopped.background_compute_of_each_batch.is_empty(),
        "当作不超过阈值：照旧停着、一批都不开（{:?}）",
        left_stopped.outcome
    );
    let [dropped] = left_stopped.versions_published.as_slice() else {
        panic!(
            "显式入口每一趟的预算至少 1：补跑那一趟扣完了上一次用户改动给的那一份，这一趟开头照样恰好发一次只删意图的发布，实际发了 {} 次",
            left_stopped.versions_published.len()
        );
    };
    assert!(
        dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && session.allocator.compaction().live_intent().is_none(),
        "删的是补跑签下的那一条：不搬单元、logged_ops 里没有整理意图，分配器这一侧意图连同 R 排他一起撤了"
    );
    assert_eq!(
        left_stopped.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(alarm_of_the_stop(&error, 2, 1)),
        "显式入口照旧停着的那一次叫起照报"
    );
    let background_call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    assert!(
        matches!(
            background_call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "后台入口不起跑：{background_call:?}"
    );
    assert_eq!(
        background_call.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 3, 1)),
        "后台入口没起跑的那一次叫起照报"
    );
    session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下，解除停下");
    let after_the_switch = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        after_the_switch.alarm_of_the_compaction_stopped_after_a_compute_error, None,
        "切换解除停下之后再叫起：不报警（{:?}）",
        after_the_switch.outcome
    );
}
