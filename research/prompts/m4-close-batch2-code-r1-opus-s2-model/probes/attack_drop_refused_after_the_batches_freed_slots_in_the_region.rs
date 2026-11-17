
/// 攻方探针（m4-close-batch2-code-r1，A2，承接那一条腿改了造前提的法子）：R 取数据单元最多的那一段（同文件别的用例在 CPU 上签发过的那一段），
/// 签发之后在只供测试的复用窗口 `ReuseWindow::ForcedToZero` 下推几批，把 R 里的单元搬到 R 之外、搬空的槽当场回收成空槽
/// （代「抬 F 之后 R 里搬空的槽回收」），回收到每块盘 R 里至少 [`FREE_SLOTS_IN_THE_REGION_WANTED`] 个空槽就停、复用窗口放回产品那一档；
/// 再在分配器里把 R 那段槽号之外的每一个空槽标成已分配（代「意图活着期间前台的写都落在 R 之外、把 R 之外写满」）。
/// 整理推一趟：那一批取不到落点被空间拒 ⇒ 暂停 ⇒ 单发一次只删意图的发布；删意图那一次重写固定点要取落点，R 排他一直留到它落盘，
/// 落点只能在 R 之外取 ⇒ 被拒，意图留在盘上、R 照旧排他。再推一趟同样被拒。对照臂（零轮的改法，只在这条探针上）：撤掉分配器这一侧的 R 排他、
/// 直接经发布路径发同一次只删意图的发布 ⇒ 发成。用户改动两行只观测：它们是前台写，合取 7 与式子那一判照判，R 之外标满之后两臂都可能被拒。
/// 断言分两道：前提（意图活在 R 上、R 排他、R 里每块盘有空槽、R 之外每块盘没有）；打中（两趟都停在空间拒那一类的失败上、一版不发、
/// 意图与 R 排他都在、丢意图数不变，对照臂删意图发成）。绿 = 打中；红看是哪一道、看打出的观测行判哪一格不成立。
#[test]
fn attack_the_drop_on_a_pause_is_refused_once_only_the_region_has_free_slots() {
    /// 签发之后几批把 R 里搬空、回收出的空槽（每块盘）至少这么多：R 之外写满之后对照臂的删意图要在 R 里落得下。
    const FREE_SLOTS_IN_THE_REGION_WANTED: usize = 32;
    /// 回收 R 里空槽那几批的迭代上界：一批至多搬 [`UNITS_PER_BATCH`] 个两槽单元，32 槽至多 8 批；R 里 29 个数据单元，搬不空。
    const BATCHES_FREEING_THE_REGION_AT_MOST: usize = 16;
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "attack-drop-on-a-pause-refused-once-only-the-region-has-free-slots",
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let (mut session, mut devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            session, devices, region, "在 CPU 上",
        );
    let region_slots = region.start_slot().0..region.end_slot().0;
    let free_slots_in_the_region_of_each_device = |session: &MountedSession| -> Vec<usize> {
        session
            .allocator
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
    let free_slots_in_the_region_at_the_issue = free_slots_in_the_region_of_each_device(&session);
    session
        .allocator
        .set_reuse_window(ReuseWindow::ForcedToZero);
    let mut batches_freeing_the_region = 0_usize;
    let mut outcome_of_the_last_freeing_run = String::from("no_run");
    while batches_freeing_the_region < BATCHES_FREEING_THE_REGION_AT_MOST
        && free_slots_in_the_region_of_each_device(&session)
            .iter()
            .any(|free| *free < FREE_SLOTS_IN_THE_REGION_WANTED)
    {
        let freeing = run_one_compaction_publish(&mut session, &mut devices);
        outcome_of_the_last_freeing_run = format!("{:?}", freeing.outcome);
        let still_live_on_the_region =
            live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some_and(|(_, intent)| intent.region == region);
        if freeing.batches_published != 1 || !still_live_on_the_region {
            break;
        }
        batches_freeing_the_region += 1;
    }
    session
        .allocator
        .set_reuse_window(ReuseWindow::GatedByTheRollbackFloor);
    let mut slots_marked_outside_the_region: Vec<u64> = Vec::new();
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let end = start + device_map.unit_area_slots();
        let mut marked = 0_u64;
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
            marked += slot - run_start;
        }
        slots_marked_outside_the_region.push(marked);
    }
    let free_slots_in_the_region = free_slots_in_the_region_of_each_device(&session);
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
    let free_slots_of_each_device: Vec<u64> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.free_slots())
        .collect();
    let data_units_in_the_region = current_of(&session)
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && record.span_slots == 2
                && region.contains_slot(record.slot)
        })
        .count();
    let live_intent_on_the_region_at_the_setup =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region);
    println!(
        "m4-close-batch2-attack: drop_refused_setup region_start_slot={} data_units_in_the_region={} batches_freeing_the_region={} outcome_of_the_last_freeing_run={} free_slots_in_the_region_at_the_issue={:?} slots_marked_outside_the_region={:?} free_slots_in_the_region={:?} free_slots_outside_the_region={:?} free_slots_of_each_device={:?} live_intent_on_the_region={} region_excluded_from_placement={:?} reuse_window={}",
        region.start_slot().0,
        data_units_in_the_region,
        batches_freeing_the_region,
        outcome_of_the_last_freeing_run,
        free_slots_in_the_region_at_the_issue,
        slots_marked_outside_the_region,
        free_slots_in_the_region,
        free_slots_outside_the_region,
        free_slots_of_each_device,
        live_intent_on_the_region_at_the_setup,
        session.allocator.region_excluded_from_placement(),
        session.allocator.reuse_window().name()
    );
    let precondition_holds = live_intent_on_the_region_at_the_setup
        && session.allocator.region_excluded_from_placement() == Some(region)
        && free_slots_in_the_region.iter().all(|free| *free > 0)
        && free_slots_outside_the_region.iter().all(|free| *free == 0);
    let mut trips_with_the_intent_left_live = Vec::new();
    for trip in 0..2 {
        session.compaction.publishes_since_the_last_user_change = 0;
        let txg_before_the_trip = current_of(&session).root.checkpoint_txg;
        let intents_dropped_before_the_trip = session.compaction.observations.intents_dropped;
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        let live_intent_on_disk =
            live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some();
        // 一批的拒绝是空间那一类才走暂停删意图；删意图那一次再被拒，交回的就是它的拒绝。所以停在空间那一类的失败上 ⇒ 删意图那一次试过、被空间拒。
        let refused_short_of_space = match &run.outcome {
            CompactionRunOutcome::Failed(
                singlefs_core::mounted_session::CompactionRunFailure::Batch(
                    singlefs_core::transaction::RelocationBatchRefused::Publish(cause),
                ),
            ) => singlefs_core::mounted_session::refusal_is_short_of_space(cause),
            _ => false,
        };
        let intents_dropped_after_the_trip = session.compaction.observations.intents_dropped;
        println!(
            "m4-close-batch2-attack: drop_refused_trip trip={} outcome={:?} refused_short_of_space={} versions_published={} txg_before={:?} txg_after={:?} live_intent_on_disk={} region_excluded_from_placement={:?} intents_dropped_before={} intents_dropped_after={}",
            trip,
            run.outcome,
            refused_short_of_space,
            run.versions_published.len(),
            txg_before_the_trip,
            current_of(&session).root.checkpoint_txg,
            live_intent_on_disk,
            session.allocator.region_excluded_from_placement(),
            intents_dropped_before_the_trip,
            intents_dropped_after_the_trip
        );
        trips_with_the_intent_left_live.push(
            matches!(run.outcome, CompactionRunOutcome::Failed(_))
                && refused_short_of_space
                && run.versions_published.is_empty()
                && live_intent_on_disk
                && session.allocator.region_excluded_from_placement() == Some(region)
                && intents_dropped_after_the_trip == intents_dropped_before_the_trip,
        );
    }
    let user_change_before_the_contrast = session.publish_user_change(
        &mut devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
        },
    );
    println!(
        "m4-close-batch2-attack: drop_refused_user_change_with_the_region_excluded refused={} detail={}",
        user_change_before_the_contrast.is_err(),
        match &user_change_before_the_contrast {
            Ok(_) => String::from("published"),
            Err(refusal) => format!("{refusal:?}").chars().take(400).collect::<String>(),
        }
    );
    let withdrawn = session.allocator.compaction_mut().withdraw_the_live_intent();
    let before_the_drop = current_of(&session).clone();
    let publish_parameters = parameters();
    let dropped_with_the_region_open = {
        let mut writer = singlefs_core::transaction::PoolWriter::new(
            &publish_parameters,
            devices.as_mut_slice(),
        );
        singlefs_core::transaction::publish_compaction_step(
            &mut writer,
            &mut session.allocator,
            &before_the_drop,
            &[],
            singlefs_core::transaction::CompactionIntentStep::Drop,
            session.instance,
        )
    };
    let contrast_drop_published = dropped_with_the_region_open.is_ok();
    println!(
        "m4-close-batch2-attack: drop_contrast_with_the_region_open withdrawn={:?} published={} detail={}",
        withdrawn.map(|intent| intent.region),
        contrast_drop_published,
        match &dropped_with_the_region_open {
            Ok(dropped) => format!(
                "txg={:?} live_intent_on_disk={} rewritten_slots_in_the_region={}",
                dropped.root.checkpoint_txg,
                live_compaction_intent_of_the_entries(&dropped.logged_operations).is_some(),
                dropped
                    .units
                    .iter()
                    .filter(|unit| dropped.rewritten.contains(&unit.identity)
                        && region_slots.contains(&unit.slot.0))
                    .count()
            ),
            Err(refusal) => format!("{refusal:?}").chars().take(400).collect::<String>(),
        }
    );
    if let Ok(dropped) = dropped_with_the_region_open {
        session.current = PoolVersion::of_a_version_of_trees(dropped);
        let user_change_after_the_contrast = session.publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        );
        println!(
            "m4-close-batch2-attack: drop_contrast_user_change published={} detail={}",
            user_change_after_the_contrast.is_ok(),
            match &user_change_after_the_contrast {
                Ok(_) => String::from("published"),
                Err(refusal) => format!("{refusal:?}").chars().take(400).collect::<String>(),
            }
        );
    }
    pool.devices = Some(devices);
    assert!(
        precondition_holds,
        "前提不成立（看 drop_refused_setup 那一行）：要意图活在 R 上、R 排他、R 里每块盘有空槽、R 之外每块盘没有空槽"
    );
    assert!(
        trips_with_the_intent_left_live.iter().all(|left| *left) && contrast_drop_published,
        "打中 A2：R 里有空槽而 R 之外没有时，暂停删意图那一次被空间拒、意图与 R 排他一直留着；只删意图那一次放开 R 就发得成"
    );
}
