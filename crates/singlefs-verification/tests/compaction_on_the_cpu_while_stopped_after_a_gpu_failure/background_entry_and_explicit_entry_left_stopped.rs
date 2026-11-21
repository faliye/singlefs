//! 后台入口停着时写的与显式入口停着时写的逐项相同；暂停时丢意图取 R 里的落点；一趟起头就停着时先报停下、不判批大小、不续做删除队列。

use super::*;

/// 停着删意图走的入口。封闭集合。
#[derive(Clone, Copy)]
enum EntryLeftStopped {
    /// 显式入口：一趟开头判出停着那一段。
    Explicit,
    /// 后台入口：判出停着而有活意图起跑的那一趟。
    Background,
}

impl EntryLeftStopped {
    fn name(self) -> &'static str {
        match self {
            EntryLeftStopped::Explicit => "explicit",
            EntryLeftStopped::Background => "background",
        }
    }
}

/// 停着删意图的两处入口写的一样：同一段历史（停着、补跑签下的意图活着、预算还剩）造两份池，
/// 一份走显式入口（一趟开头判出停着那一段），一份走后台入口（判出停着而有活意图起跑的那一趟），那一次调用录下的写逐步相同、发成的那一版相同。
/// 两处都只经 `drop_the_live_intent_on_a_stop` 单发一次只删意图的发布，同暂停删意图流里那一步
/// （`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs` 的 `CompactionIntentDroppedOnAStop`）。
#[test]
fn the_background_entry_left_stopped_writes_exactly_what_the_explicit_entry_left_stopped_writes() {
    let mut operations_and_version_of_each_entry = Vec::new();
    for entry in [EntryLeftStopped::Explicit, EntryLeftStopped::Background] {
        let (pool, _region, mut session, mut devices) =
            session_left_stopped_with_the_intent_a_compensating_run_issued(&format!(
                "compaction-left-stopped-drop-through-the-{}-entry",
                entry.name()
            ));
        session.compaction.publishes_since_the_last_user_change =
            BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
        let start = pool.stream.operation_count();
        let run = match entry {
            EntryLeftStopped::Explicit => session.run_compaction(&mut devices).expect("整理一趟"),
            EntryLeftStopped::Background => match session
                .run_compaction_in_the_background(
                    &mut devices,
                    pacing_with_the_timer_fallback_off(),
                    &CallerWithNoForegroundFsync,
                )
                .expect("后台入口")
            {
                BackgroundCompactionCall::Ran { run, .. } => *run,
                not_started @ BackgroundCompactionCall::NotStarted { .. } => {
                    panic!("停着而有活意图：后台入口起跑：{not_started:?}")
                }
            },
        };
        let [dropped] = <[TransactionOutput; 1]>::try_from(run.versions_published)
            .expect("停着那一趟恰好发一次只删意图的发布");
        assert!(
            dropped.units_relocated_by_this_publish.is_empty()
                && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
            "{}：只删意图、不搬单元",
            entry.name()
        );
        let operations = pool.stream.retained_operations().split_off(start);
        drop(devices);
        operations_and_version_of_each_entry.push((operations, dropped));
    }
    let (background_operations, background_dropped) = operations_and_version_of_each_entry
        .pop()
        .expect("后台入口那一份");
    let (explicit_operations, explicit_dropped) = operations_and_version_of_each_entry
        .pop()
        .expect("显式入口那一份");
    assert!(
        !explicit_operations.is_empty() && explicit_operations == background_operations,
        "两处入口那一次调用录下的写逐步相同（{} 步对 {} 步）",
        explicit_operations.len(),
        background_operations.len()
    );
    assert_eq!(
        explicit_dropped, background_dropped,
        "两处入口发成的删意图那一版相同"
    );
}

/// 只有 R 里有空槽时暂停删意图（D26（后台整理与放置回收） 已定项 2）：签发之后在复用窗口置 0 下推几批，把 R 里搬空的槽当场回收成空槽，回收到每块盘
/// R 里至少 [`FREE_SLOTS_IN_THE_REGION_WANTED`] 个；再在分配器里把 R 之外的每一个空槽标成已分配（不写盘）。整理推一趟：那一批在 R 之外取不到落点、
/// 被空间拒、暂停，单发一次只删意图的发布——它取落点时放开 R，在 R 里落得下，发成；盘上与分配器这一侧意图连同 R 排他都撤了。
/// 判别力：`crates/mutations.tsv`「删意图照旧排除 R」那一行。
#[test]
fn the_drop_on_a_pause_takes_its_placements_in_the_region_once_only_the_region_has_free_slots() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-drop-on-a-pause-places-in-the-region",
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let (mut session, mut devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            session,
            devices,
            region,
            "在 CPU 上",
        );
    let region_slots = region.start_slot().0..region.end_slot().0;
    let free_slots_in_the_region_of_each_device = |allocator: &PoolAllocator| -> Vec<usize> {
        allocator
            .devices
            .iter()
            .map(|device_map| {
                region_slots
                    .clone()
                    .filter(|slot| device_map.is_free(SlotNumber(*slot)))
                    .count()
            })
            .collect()
    };
    session
        .allocator
        .set_reuse_window(ReuseWindow::ForcedToZero);
    for _batch_index in 0..BATCHES_FREEING_THE_REGION_AT_MOST {
        if free_slots_in_the_region_of_each_device(&session.allocator)
            .iter()
            .all(|free| *free >= FREE_SLOTS_IN_THE_REGION_WANTED)
        {
            break;
        }
        let freeing = run_one_compaction_publish(&mut session, &mut devices);
        assert!(
            freeing.batches_published == 1
                && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                    .is_some_and(|(_, intent)| intent.region == region),
            "前提：回收 R 里空槽的那几批各搬一批、意图照抄：{:?}",
            freeing.outcome
        );
    }
    session
        .allocator
        .set_reuse_window(ReuseWindow::GatedByTheRollbackFloor);
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let end = start + device_map.unit_area_slots();
        let mut slot = start;
        while slot < end {
            if region_slots.contains(&slot) || !device_map.is_free(SlotNumber(slot)) {
                slot += 1;
                continue;
            }
            let run_start = slot;
            while slot < end
                && !region_slots.contains(&slot)
                && device_map.is_free(SlotNumber(slot))
            {
                slot += 1;
            }
            device_map.mark_allocated(SlotNumber(run_start), slot - run_start);
        }
    }
    let free_slots_outside_the_region: Vec<usize> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            let start = device_map.unit_area_start().slot().0;
            (start..start + device_map.unit_area_slots())
                .filter(|slot| {
                    !region_slots.contains(slot) && device_map.is_free(SlotNumber(*slot))
                })
                .count()
        })
        .collect();
    assert!(
        free_slots_in_the_region_of_each_device(&session.allocator)
            .iter()
            .all(|free| *free >= FREE_SLOTS_IN_THE_REGION_WANTED)
            && free_slots_outside_the_region.iter().all(|free| *free == 0)
            && session.allocator.region_excluded_from_placement() == Some(region),
        "前提：R 里每块盘至少 {FREE_SLOTS_IN_THE_REGION_WANTED} 个空槽、R 之外一个都没有，意图活着、R 排他着（R 之外 {free_slots_outside_the_region:?}）"
    );
    session.compaction.publishes_since_the_last_user_change = 0;
    let txg_before_the_run = current_of(&session).root.checkpoint_txg;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PausedShortOfSpace)
        ),
        "那一批在 R 之外取不到落点：暂停，删意图那一次发成：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "暂停那一趟恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(&session) == dropped,
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        dropped.allocation_records.iter().any(|record| {
            record.device == region.device()
                && !record.is_released
                && record.generation == dropped.root.checkpoint_txg
                && region.contains_slot(record.slot)
        }),
        "删意图那一次自己的新落点落在 R 里（R 之外一个空槽都没有）"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "分配器这一侧意图连同 R 排他一起撤了"
    );
}

/// 停着、不超过阈值（取默认 85%），整理的批大小用产品那一种，删除队列非空（停着时会话 unlink 第一个文件、再挂着回退到 unlink 意图那一版，
/// T4-1 那一条路）：一趟开头那一判排在删除续做之前，这一趟报的是出错停着；删除续做没走，队列照旧带着第一个文件；
/// 一个写都不发，盘上逐字节不变。
#[test]
fn a_run_left_stopped_at_its_start_reports_the_stop_before_judging_the_batch_size_or_continuing_the_deletion_queue(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-left-stopped-at-the-start-before-the-batch-size",
    );
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
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = session.allocator.compaction().switches;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionSwitches::PRODUCT.batch_size,
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
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
        ) && run.versions_published.is_empty()
            && run.background_compute_of_each_batch.is_empty(),
        "报的是整理先前出错停着、一个写都不发：{:?}",
        run.outcome
    );
    assert_eq!(
        current_of(&session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "停着时删除续做不走：队列照旧带着第一个文件"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变"
    );
}
