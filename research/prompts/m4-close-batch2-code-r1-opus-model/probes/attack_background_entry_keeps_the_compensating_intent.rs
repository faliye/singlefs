
/// 攻方探针（m4-close-batch2-code-r1，打中 A1）：整理因 GPU 出错停着，补跑那一趟签下一条意图之后读数降到不超过阈值
/// （这里用只供测试的「之后都当不超过」开关代用户删文件，同文件 `a_compensating_compaction_stays_stopped_from_the_next_batch_once_a_user_deletion_brings_the_reading_down`
/// 用真删文件造同一个读数），产品入口（后台入口）连叫三次、中间夹一次用户改动再叫一次：每次都在判起跑那一刻判出停着、不起跑，
/// 补跑签下的那条意图一直留在盘上、R 一直排他。对照：显式入口（没有产品调用方）同一刻一趟开头就单发一次只删意图的发布。
/// 断言写的是「打中」那一边：这条用例绿 = 后台入口停着时不删意图（与「停下都删意图」的定案与文档串对不上）；红 = 没打中。
#[test]
fn attack_the_background_entry_keeps_the_compensating_intent_and_its_region_exclusion_while_stopped(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "attack-background-entry-keeps-the-compensating-intent",
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
    let intent_after_the_compensating_run =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .map(|(issued_txg, intent)| (issued_txg, intent.region));
    println!(
        "m4-close-batch2-attack: compensating_run outcome={:?} versions_published={} batches_on_the_cpu={} live_intent_on_disk={:?}",
        compensating.outcome,
        compensating.versions_published.len(),
        compensating.background_compute_of_each_batch.len(),
        intent_after_the_compensating_run
    );
    assert!(
        intent_after_the_compensating_run.is_some_and(|(_, intent_region)| intent_region == region),
        "前提：补跑那一趟在 R 上签下意图、意图活着（{:?}）",
        compensating.outcome
    );
    let mut background_calls_not_started_while_stopped = Vec::new();
    for _call in 0..3 {
        let call = session
            .run_compaction_in_the_background(
                &mut devices,
                pacing_with_the_timer_fallback_off(),
                &CallerWithNoForegroundFsync,
            )
            .expect("后台入口");
        background_calls_not_started_while_stopped.push(matches!(
            call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ));
    }
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("停着时用户照常建一个 inode");
    let call_after_the_user_change = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let not_started_after_the_user_change = matches!(
        call_after_the_user_change,
        BackgroundCompactionCall::NotStarted {
            reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
            ..
        }
    );
    let live_intent_on_disk_after_the_background_calls =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .map(|(issued_txg, intent)| (issued_txg, intent.region));
    let region_excluded_after_the_background_calls =
        session.allocator.region_excluded_from_placement();
    println!(
        "m4-close-batch2-attack: background_entry_while_stopped not_started_while_stopped={:?} not_started_after_a_user_change={} live_intent_on_disk={:?} region_excluded_from_placement={:?}",
        background_calls_not_started_while_stopped,
        not_started_after_the_user_change,
        live_intent_on_disk_after_the_background_calls,
        region_excluded_after_the_background_calls
    );
    let explicit = session.run_compaction(&mut devices).expect("整理一趟");
    let live_intent_on_disk_after_the_explicit_entry =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_some();
    println!(
        "m4-close-batch2-attack: explicit_entry_while_stopped outcome={:?} versions_published={} live_intent_on_disk={} region_excluded_from_placement={:?}",
        explicit.outcome,
        explicit.versions_published.len(),
        live_intent_on_disk_after_the_explicit_entry,
        session.allocator.region_excluded_from_placement()
    );
    pool.devices = Some(devices);
    assert!(
        background_calls_not_started_while_stopped
            .iter()
            .all(|not_started| *not_started)
            && not_started_after_the_user_change
            && live_intent_on_disk_after_the_background_calls == intent_after_the_compensating_run
            && region_excluded_after_the_background_calls == Some(region),
        "打中 A1：后台入口停着时一个写都不发，补跑签下的意图留在盘上、R 照旧排他"
    );
}
