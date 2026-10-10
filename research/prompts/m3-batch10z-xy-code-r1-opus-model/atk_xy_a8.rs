#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_abandoned_root_and_c_ways_out_vs_real() {
    let ks: Vec<u64> = atk_env_list("ATK_KS", "0,1,2,4,6");
    let grows: Vec<usize> = (1..=90).collect();
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let (mut stuck_cells, mut disagree, mut moved) = (0, 0, 0);
    for k in ks {
        let stream = SharedStream::retaining_contents();
        let mut pool = PoolUnderTest {
            parameters: built.parameters.clone(),
            device_bytes: built.device_bytes,
            devices: plain_devices_on(&built.image)
                .into_iter()
                .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
                .collect(),
            session: None,
            write_time_seconds: built.write_time_seconds + 1000,
            content_of_the_current_version: Vec::new(),
            content_of_each_root: BTreeMap::new(),
        };
        pool.crash_and_mount_writable().expect("实例 4 可写挂载");
        for _ in 0..k {
            pool.overwrite(OVERWRITE_BYTES).expect("实例 4 覆盖写");
        }
        let previous_root = *pool.session().current.root();
        let previous_content = pool.content_of_the_current_version.clone();
        let base = pool.image();
        let operations_before_r = stream.operation_count();
        let Ok(published) = pool.create_one_inode() else {
            println!("ATK k1ab k={k} r_refused");
            continue;
        };
        if !published.floor_raises.is_empty() {
            println!("ATK k1ab k={k} r_pushed_raises");
            continue;
        }
        let root = *pool.session().current.root();
        let operations = stream.retained_operations();
        let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&operations[operations_before_r..], &fixed_geometry_of_the_384_slot_small_devices());
        let root_index = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("根");
        let recorded = RecordedPublish {
            content: pool.content_of_the_current_version.clone(),
            pool,
            base,
            writes,
            root_index,
            key: (root.instance, root.checkpoint_txg),
            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
            previous_content,
        };
        let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
        let lift = ReadBackAllZeroLift::default();
        let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
        let mounted = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => mounted,
            Err(refusal) => {
                println!("ATK k1ab k={k} mount_refused({})", atk_head(format!("{refusal:?}")));
                continue;
            }
        };
        lift.lift();
        let image_after_the_mount = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
        let plain_pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
        let allocator = &plain_pool.session().allocator;
        let label = format!("k1ab k={k} abandoned={} iso_all={}", ring_slots_recorded_as_holding_an_abandoned_root_by(allocator).len(), isolated_slots_of(allocator));
        let (cells, bad, mv) = atk_scan_lengths_and_follow(&label, &AtkSnap::of(&plain_pool), &grows, false);
        stuck_cells += cells;
        disagree += bad;
        moved += mv;
    }
    println!("ATK k1ab totals stuck={stuck_cells} disagree={disagree} moved={moved}");
}
