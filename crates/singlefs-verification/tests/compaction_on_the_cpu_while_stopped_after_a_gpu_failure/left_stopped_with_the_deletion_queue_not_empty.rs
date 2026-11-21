//! 停着、意图活着、删除队列不空时丢意图：在删除续做之前丢掉、把队列原样带过去。

use super::*;

/// 停着、不超过阈值（取默认 85%），出错停下那一刻删了意图之后删除队列带回第一个文件（会话 unlink、再挂着回退到 unlink 意图那一版），
/// 再在现行那一版上签下一条意图（经发布入口直接签：会话推整理会先走删除续做），分配器这一侧照会话的做法排他 R。交回池、R、会话与盘：
/// 整理停着而不补跑，现行那一版有活意图、删除队列不空，上一次用户改动给的那一份 `B_gc` 还剩。
fn session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
    tag: &str,
) -> (
    BuiltPool,
    CompactionRegion,
    MountedSession,
    Vec<(DeviceIdentity, Recorded)>,
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(tag);
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    bring_the_deletion_queue_back_by_rolling_back_an_unlink(&mut session, &mut devices);
    let previous = current_of(&session).clone();
    let unit_area = {
        let device_map = &session.allocator.devices[0];
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area,
        ),
    };
    session.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let issued = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(
            &mut writer,
            &mut session.allocator,
            &previous,
            &[],
            CompactionIntentStep::Issue(intent),
            session.instance,
        )
        .expect("在删除队列不空的那一版上签发")
    };
    session.current = PoolVersion::of_a_version_of_trees(issued);
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, logged)| logged == intent)
            && current_of(&session).deleted_inodes == vec![InodeNumber(FIRST_INODE_NUMBER)]
            && session.allocator.region_excluded_from_placement() == Some(region),
        "前提：签发那一版有活意图、照抄删除队列（带着第一个文件），分配器这一侧排他 R"
    );
    assert!(
        session
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "前提：整理照旧停着"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    (pool, region, session, devices)
}

/// 停着删意图、删除队列不空时那一次发布的样子：恰好一版、不搬单元、logged_ops 里没有整理意图、照抄删除队列（带着第一个文件），删除续做一步都没走；
/// 会话的现行那一版就是它，分配器这一侧意图连同 R 排他都撤了。`entry` 写进断言消息。
fn assert_the_stopped_drop_carried_the_deletion_queue(
    session: &MountedSession,
    run: &CompactionRun,
    txg_before_the_call: u64,
    entry: &str,
) {
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.deletion_steps_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "{entry}：停着，删意图排在删除续做之前、续做一步都不走、一批都不开：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "{entry}：恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_call + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && dropped.deleted_inodes == vec![InodeNumber(FIRST_INODE_NUMBER)]
            && current_of(session) == dropped,
        "{entry}：紧接着的就是删意图那一版，不搬单元、logged_ops 里没有整理意图，删除队列照抄（带着第一个文件），会话的现行那一版就是它"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "{entry}：分配器这一侧意图连同 R 排他都撤了"
    );
}

/// 显式入口一趟开头判出停着而现行那一版有活意图、删除队列不空（D26（后台整理与放置回收） 已定项 13 第 4 条）：删意图排在删除续做之前，
/// 单发的那一版照抄删除队列，续做等解除之后。与暂停删意图流
/// （`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs`）不同处：暂停那一趟删除续做排在批之前、先走空队列，
/// 删意图那一版之前队列已空；这里停着时续做不走，删意图那一版带着没走的队列落盘。
/// 判别力：`crates/mutations.tsv`「一趟开头停着、删除队列不空时不删意图」那一行。
#[test]
fn a_run_left_stopped_with_the_deletion_queue_not_empty_drops_the_intent_before_the_deletion_continuation_and_carries_the_queue(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
            "compaction-left-stopped-drops-the-intent-carrying-the-deletion-queue",
        );
    let txg_before_the_run = current_of(&session).root.checkpoint_txg.0;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_the_stopped_drop_carried_the_deletion_queue(
        &session,
        &run,
        txg_before_the_run,
        "显式入口",
    );
}

/// 同一形在后台入口：判出停着而有活意图起跑的那一趟（[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]）排在
/// 「删除队列不空就起跑只走队列」那一判之前，单发的删意图那一版照抄删除队列。
/// 判别力：`crates/mutations.tsv`「后台入口停着、删除队列不空时不起跑删意图」那一行。
#[test]
fn the_background_entry_left_stopped_with_the_deletion_queue_not_empty_drops_the_intent_and_carries_the_queue(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
            "compaction-background-entry-left-stopped-drops-the-intent-carrying-the-deletion-queue",
        );
    let txg_before_the_call = current_of(&session).root.checkpoint_txg.0;
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而现行那一版有活意图：后台入口起跑一趟删它：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图，不是删除队列不空"
    );
    assert_the_stopped_drop_carried_the_deletion_queue(
        &session,
        &run,
        txg_before_the_call,
        "后台入口",
    );
}
