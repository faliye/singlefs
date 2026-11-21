//! 补跑那一趟不再超过阈值时：下一批起停下、单发一次只删意图的发布丢掉补跑签下的意图；显式入口与后台入口停着时都丢、之后一个写都不发，排空被拒的那一次在切换之后不再续做。

use super::*;

/// 停着时整理照常推：一批读完设置、开算之前的钩子里数这一趟的第几批（从 1 数），数到 `batch_numbered` 那一批时调只供测试的开关，
/// 从那一批之后每一批开头都当占用不超过阈值判。
fn judge_the_batches_after_the_numbered_one_not_above_the_threshold(
    batch_numbered: u64,
    batches_seen: &Cell<u64>,
    background_compute: &mut BackgroundComputeOfThisMount,
) {
    batches_seen.set(batches_seen.get() + 1);
    if batches_seen.get() == batch_numbered {
        background_compute.judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests();
    }
}

/// 停着、阈值 0%（出错停下时删了意图）：一趟开头超过阈值，头一批在 CPU 上补跑、在同一个 R 上从头签发，第二批补跑、搬了单元
/// （两批都照开关排他 R）；只供测试的开关在第二批的钩子里让之后每一批都当不超过阈值判，第三批开头就不开：单发一次只删意图的发布丢掉
/// 补跑签下的那一条，停在「出错停着」上。只有补跑那两批的记账，没开的那一批没有；盘上与分配器这一侧都没有活意图；
/// 补跑的批数是 2，整理照旧停着、报的还是那个错。
#[test]
fn a_compensating_run_stops_at_the_next_batch_once_it_is_not_above_the_threshold_and_drops_the_intent_it_issued(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensation-ends-at-the-next-batch-of-the-run",
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
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let batches_seen = Cell::new(0);
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut |background_compute| {
                judge_the_batches_after_the_numbered_one_not_above_the_threshold(
                    2,
                    &batches_seen,
                    background_compute,
                );
            },
        )
        .expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 1
            && run.versions_published.len() == 3,
        "签发与一批补跑发成，第三批开头不超过阈值、不开，删意图之后停在此前出错停着上：{:?}，发了 {} 版",
        run.outcome,
        run.versions_published.len()
    );
    let [issued, relocated, dropped] = run.versions_published.as_slice() else {
        panic!("发了三版：签发、一批、删意图");
    };
    assert!(
        [issued, relocated].iter().all(|version| {
            live_compaction_intent_of_the_entries(&version.logged_operations)
                .is_some_and(|(_, intent)| intent.region == region)
        }) && !relocated.units_relocated_by_this_publish.is_empty(),
        "补跑那两批在同一个 R 上从头签发、搬了单元"
    );
    assert!(
        dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
        "没开的那一批开头单发一次只删意图的发布：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        matches!(
            run.background_compute_of_each_batch.as_slice(),
            [
                BackgroundComputeOfOneBatch {
                    path_taken: BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    ),
                    compute_error_that_abandoned_the_batch: None,
                    ..
                },
                BackgroundComputeOfOneBatch {
                    path_taken: BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    ),
                    compute_error_that_abandoned_the_batch: None,
                    ..
                }
            ]
        ),
        "只有补跑那两批的记账，没开的那一批没有：{:?}",
        run.background_compute_of_each_batch
    );
    assert!(
        session.allocator.compaction().live_intent().is_none() && current_of(&session) == dropped,
        "分配器这一侧意图连同 R 排他一起撤了，会话的现行那一版就是删意图那一版"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 2,
        }],
        "整理照旧停着、报的还是那个错，补跑了两批"
    );
}

/// 停着、阈值 0%（出错停下时删了意图）：先补跑一趟（预算 1：补跑那一批在同一个 R 上从头签发、照开关排他 R；钩子里让之后每一批都当
/// 不超过阈值判），下一趟开头就判出停着而不补跑：一批都不开，单发一次只删意图的发布丢掉补跑签下的那一条，停在「出错停着」上；
/// 没有一批的记账，盘上与分配器这一侧都没有活意图。
#[test]
fn a_run_left_stopped_at_its_start_drops_the_intent_a_compensating_run_issued() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-left-stopped-at-the-start-drops-the-intent",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
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
    assert!(
        matches!(
            compensating.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && compensating.versions_published.len() == 1,
        "补跑那一趟只发了签发那一次、推到预算用完：{:?}",
        compensating.outcome
    );
    assert_eq!(
        session.allocator.compaction().live_intent().map(|live| (
            live.intent.region,
            live.is_the_region_excluded_from_placement
        )),
        Some((region, true)),
        "补跑那一批在同一个 R 上从头签发，照开关排他 R"
    );
    let txg_before_the_run = current_of(&session).root.checkpoint_txg;
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let root_ring_slots_used_before_the_run = session.compaction.observations.root_ring_slots_used;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        (
            session.compaction.publishes_since_the_last_user_change,
            session.compaction.observations.root_ring_slots_used
        ),
        (
            BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN + 1,
            root_ring_slots_used_before_the_run + 1
        ),
        "删意图那一次是整理的一次发布：从 `B_gc` 里扣一次、用掉一个根槽"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "一趟开头就判出停着而不补跑：一批都不开：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "一趟开头恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none() && current_of(&session) == dropped,
        "分配器这一侧意图连同 R 排他一起撤了，会话的现行那一版就是删意图那一版"
    );
}

/// 后台入口判出停着而现行那一版有活意图（D26（后台整理与放置回收） 已定项 13 第 4 条）：
/// 上一次用户改动给的那一份 `B_gc` 还剩，后台入口起跑一趟，只单发一次删意图的发布（同显式入口一趟开头判出停着那一段），
/// 盘上与分配器这一侧意图连同 R 排他都撤了；之后再调不起跑，盘上逐字节不变。
/// 判别力：`crates/mutations.tsv`「后台入口停着不删」那一行。
#[test]
fn the_background_entry_left_stopped_drops_the_intent_a_compensating_run_issued_and_then_writes_nothing(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-background-entry-left-stopped-drops-the-intent",
        );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let txg_before_the_call = current_of(&session).root.checkpoint_txg;
    let intents_dropped_before_the_call = session.compaction.observations.intents_dropped;
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而现行那一版有活意图：后台入口起跑一趟删它：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.deletion_steps_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "那一趟开头判出停着：一批都不开、删除续做不走：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "那一趟恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_call.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(&session) == dropped,
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图，会话的现行那一版就是它"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none()
            && session.compaction.observations.intents_dropped
                == intents_dropped_before_the_call + 1,
        "分配器这一侧意图连同 R 排他一起撤了，记一次丢意图"
    );
    let after_the_drop = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call_after_the_drop = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            call_after_the_drop,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "删完之后照旧停着、不起跑：{call_after_the_drop:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        after_the_drop,
        "之后再调一个写都不发：系统配置槽、根环、录制流的步数逐字节不变"
    );
}

/// 停着而有活意图、预算扣完（D26（后台整理与放置回收） 已定项 13 第 4、5 条）：补跑那一趟用完上一次用户改动给的那一份 `B_gc`，后台入口连叫三次都不起跑、
/// 一个写都不发，意图与 R 排他留着（预算照后台入口别的起跑给）；用户改动发成、给了新的一份之后，后台入口那一次只单发一次删意图的发布，
/// 盘上与分配器这一侧意图连同 R 排他都撤了。
#[test]
fn the_background_entry_left_stopped_with_the_budget_spent_drops_the_intent_after_the_next_user_change(
) {
    let (mut pool, region, mut session, mut devices) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-background-entry-left-stopped-with-the-budget-spent",
        );
    let before_the_calls = disk_snapshot(&pool.memory_pool(), &pool.stream);
    for _call in 0..3 {
        let call = session
            .run_compaction_in_the_background(
                &mut devices,
                pacing_with_the_timer_fallback_off(),
                &CallerWithNoForegroundFsync,
            )
            .expect("后台入口");
        assert!(
            matches!(
                call,
                BackgroundCompactionCall::NotStarted {
                    reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                    ..
                }
            ),
            "那一份 `B_gc` 扣完、前台写得进：不起跑：{call:?}"
        );
    }
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_calls,
        "预算扣完时一个写都不发"
    );
    assert_eq!(
        session.allocator.region_excluded_from_placement(),
        Some(region),
        "意图与 R 排他留到下一次用户改动之后"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("停着时用户照常建一个 inode");
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("用户改动给了新的一份 `B_gc`：后台入口起跑一趟删意图：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图"
    );
    assert!(
        run.versions_published.len() == 1
            && run.batches_published == 0
            && matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
                )
            ),
        "只单发一次删意图、停在停着上：{:?}，发了 {} 次",
        run.outcome,
        run.versions_published.len()
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_none()
            && session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "盘上与分配器这一侧意图连同 R 排他都撤了"
    );
}

/// 攒批的会话经放锁的后台入口（至多一次发布）判出停着而有活意图、起跑删意图那一趟（D26（后台整理与放置回收） 已定项 13 第 4 条），
/// 排空待发那一次写报 I/O 错被拒（待发并进冻结着的那一次发布）：这一趟不记成没完。重发冻结的那一次（一次用户改动，记下「该跑了」）、
/// 一次通过校验的切换解除停下之后，下一次调用照普通整理判起跑，触发是用户改动之后的「该跑了」，不顶着停着删意图那一趟的触发名推普通整理。
/// 判别力：`crates/mutations.tsv`「排空被拒时停着删意图那一趟照样记成没完」那一行。
#[test]
fn the_stopped_drop_refused_at_the_drain_is_not_continued_under_its_trigger_once_a_switch_lifts_the_stop(
) {
    let (mut pool, _region, mut session, mut devices_of_the_stopped_session) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-stopped-drop-refused-at-the-drain-is-judged-again-after-the-switch",
        );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    session_with_pending_changes
        .accept(
            &mut devices_of_the_stopped_session,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
            CallerMonotonicClockReading(Duration::ZERO),
        )
        .expect("建一个 inode 收进待发");
    assert_eq!(
        session_with_pending_changes.pending_changes_accepted(),
        1,
        "前提：收进待发、两个阈值都没到：没发布"
    );
    let mut devices_failing_every_write: Vec<(DeviceIdentity, DeviceFailingEveryWrite)> =
        devices_of_the_stopped_session
            .into_iter()
            .map(|(identity, device)| (identity, DeviceFailingEveryWrite { inner: device }))
            .collect();
    let drained = session_with_pending_changes
        .run_compaction_in_the_background_for_at_most_one_publish(
            &mut devices_failing_every_write,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = devices_failing_every_write
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    assert!(
        matches!(
            drained,
            Err(BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused { .. })
        ),
        "前提：停着而有活意图，起跑删意图那一趟之前排空待发、那一次写报错被拒：{drained:?}"
    );
    assert!(
        !session_with_pending_changes
            .session_for_tests_only()
            .compaction
            .has_a_background_trip_under_way(),
        "停着删意图那一趟排空被拒：不记成没完的一趟"
    );
    assert!(
        session_with_pending_changes
            .resend_the_frozen_publish(&mut devices)
            .expect("盘照常写得进：重发成了"),
        "前提：排空那一次冻结着，重发成了"
    );
    assert_eq!(
        session_with_pending_changes
            .session_for_tests_only()
            .compaction
            .start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "前提：重发的是一次用户改动，起跑判据为真，记下「该跑了」"
    );
    session_with_pending_changes
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu 的产品设置：切换收下");
    assert!(
        session_with_pending_changes
            .session_for_tests_only()
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "前提：停着时收到通过校验的切换请求，当场解除停下"
    );
    let after_the_switch = session_with_pending_changes
        .run_compaction_in_the_background_for_at_most_one_publish(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, .. } = &after_the_switch.call else {
        panic!(
            "解除停下、记着「该跑了」：后台入口起跑：{:?}",
            after_the_switch.call
        );
    };
    assert_eq!(
        *trigger,
        BackgroundCompactionTrigger::DueAfterAUserChange,
        "解除停下之后照普通整理判起跑：触发是用户改动之后的「该跑了」，不是停着删意图那一趟"
    );
}
