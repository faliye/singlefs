
/// 攻方探针（m4-close-batch2-code-r1，打中 A2）：活意图在 R 上、R 排他着，池子 R 之外的空槽被写满（用例在分配器里把 R 那段槽号之外的每个空槽标成已分配，
/// 代「意图活着期间前台的写都落在 R 之外、把 R 之外写满」），R 里还有空槽。整理推一趟：那一批取不到目的地被拒 ⇒ 暂停 ⇒ 单发一次只删意图的发布；
/// 删意图那一次重写固定点要取落点，而 R 排他一直留到它落盘（`drop_the_live_intent_on_a_stop` 的文档串），落点同样只能在 R 之外取 ⇒ 被拒，
/// 意图留在盘上、R 照旧排他。再推一趟同样被拒；用户改动同样被拒。对照臂（零轮的改法，只在这条探针上）：撤掉分配器这一侧的 R 排他、
/// 直接经发布路径发同一次只删意图的发布（它那一版里意图已不在，I-5.6 管不到它的落点）⇒ 发成，之后用户改动发成。
/// 断言写的是「打中」那一边：绿 = 产品路径删不掉意图、对照臂删得掉；红 = 没打中（看打出的观测行判是哪一格不成立）。
#[test]
fn attack_the_drop_on_a_pause_is_refused_while_only_the_region_has_free_slots() {
    let (mut pool, _fullest_segment) = pool_after_a_sequential_write_in_an_earlier_mount(
        "attack-drop-on-a-pause-refused-while-only-the-region-has-free-slots",
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
    for record in current_of(&session).allocation_records.iter().filter(|record| {
        record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
    }) {
        *data_units_by_segment
            .entry(record.slot.0 / 64 * 64)
            .or_default() += 1;
    }
    // R 取第一块盘上有数据单元、两块盘同槽号空槽最多的那一段（R 之外写满之后 R 里要有空槽，对照臂才有地方落）。
    let free_slot_pairs_in_the_segment = |segment: u64| -> usize {
        (segment..segment + 64)
            .filter(|slot| {
                session
                    .allocator
                    .devices
                    .iter()
                    .all(|device_map| device_map.is_free(SlotNumber(*slot)))
            })
            .count()
    };
    let (sparsest_segment, data_units_in_the_region) = data_units_by_segment
        .iter()
        .max_by_key(|(segment, _)| free_slot_pairs_in_the_segment(**segment))
        .map(|(segment, data_units)| (*segment, *data_units))
        .expect("写了数据单元：至少一段里有");
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(sparsest_segment),
    )
    .expect("按 64 取整的段起点");
    let (mut session, mut devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            session, devices, region, "在 CPU 上",
        );
    let region_slots = sparsest_segment..sparsest_segment + 64;
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
    let free_slots_in_the_region_of_each_device: Vec<usize> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            region_slots
                .clone()
                .filter(|slot| device_map.is_free(SlotNumber(*slot)))
                .count()
        })
        .collect();
    let free_slots_of_each_device: Vec<u64> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.free_slots())
        .collect();
    println!(
        "m4-close-batch2-attack: drop_refused_setup region_start_slot={} data_units_in_the_region={} slots_marked_outside_the_region={:?} free_slots_in_the_region={:?} free_slots_of_each_device={:?} region_excluded_from_placement={:?}",
        sparsest_segment,
        data_units_in_the_region,
        slots_marked_outside_the_region,
        free_slots_in_the_region_of_each_device,
        free_slots_of_each_device,
        session.allocator.region_excluded_from_placement()
    );
    let mut trips_with_the_intent_left_live = Vec::new();
    for trip in 0..2 {
        session.compaction.publishes_since_the_last_user_change = 0;
        let txg_before_the_trip = current_of(&session).root.checkpoint_txg;
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        let live_intent_on_disk =
            live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some();
        println!(
            "m4-close-batch2-attack: drop_refused_trip trip={} outcome={:?} versions_published={} txg_before={:?} txg_after={:?} live_intent_on_disk={} region_excluded_from_placement={:?}",
            trip,
            run.outcome,
            run.versions_published.len(),
            txg_before_the_trip,
            current_of(&session).root.checkpoint_txg,
            live_intent_on_disk,
            session.allocator.region_excluded_from_placement()
        );
        trips_with_the_intent_left_live.push(
            matches!(run.outcome, CompactionRunOutcome::Failed(_))
                && run.versions_published.is_empty()
                && live_intent_on_disk
                && session.allocator.region_excluded_from_placement() == Some(region),
        );
    }
    let user_change_before_the_contrast = session.publish_user_change(
        &mut devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
        },
    );
    let user_change_refused_before_the_contrast = user_change_before_the_contrast.is_err();
    println!(
        "m4-close-batch2-attack: drop_refused_user_change_with_the_region_excluded refused={} detail={}",
        user_change_refused_before_the_contrast,
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
    let mut contrast_user_change_published = false;
    if let Ok(dropped) = dropped_with_the_region_open {
        session.current = PoolVersion::of_a_version_of_trees(dropped);
        let user_change_after_the_contrast = session.publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        );
        contrast_user_change_published = user_change_after_the_contrast.is_ok();
        println!(
            "m4-close-batch2-attack: drop_contrast_user_change published={} detail={}",
            contrast_user_change_published,
            match &user_change_after_the_contrast {
                Ok(_) => String::from("published"),
                Err(refusal) => format!("{refusal:?}").chars().take(400).collect::<String>(),
            }
        );
    }
    pool.devices = Some(devices);
    assert!(
        free_slots_in_the_region_of_each_device
            .iter()
            .all(|free| *free > 0)
            && trips_with_the_intent_left_live.iter().all(|left| *left)
            && user_change_refused_before_the_contrast
            && contrast_drop_published
            && contrast_user_change_published,
        "打中 A2：R 里有空槽而 R 之外没有时，暂停删意图那一次取不到落点、意图与 R 排他一直留着，用户改动被拒；只删意图那一次放开 R 就发得成"
    );
}
