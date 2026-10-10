
// ====================== 正推腿（m3-batch10z-tu-code-r1，sonnet 腿）追加：两槽 C、C 与被抛弃根重叠时的「被宽护卡住」报告 ======================
// 起点是攻方上一轮 atk_k2_two_c_slots_push / atk_k2_abandoned_root_and_c_push 的搭建；把「覆盖写」换成「顺序写长 grow 个单元」，被宽护卡住时
// 比 ways_out（出路二列不列、次数、槽数、放出来几个）与真推环（推几次、隔离降几个）、推完再试同一个写的结局。

fn sonnet_judge_stuck(label: &str, pool: &mut PoolUnderTest<SparseBlockDevice>, size_now: usize, grow: usize) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let c_before = atk_c_ring(&pool.session().allocator);
    let ab_before = ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).len();
    let first = pool.sequential_write((size_now + grow) * unit_bytes);
    let stuck = match &first {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(ns)) => match &ns.wide_protection_at_the_ceiling {
            WideProtectionAtTheCeiling::StuckBehindTheWideProtection(s) => { let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, s).expect("要出路"); Some((
                w.ways_out.clone(),
                s.empty_publishes_until_the_ring_slot_of_r_is_written,
                s.checkpoint_txg_of_r.map(|t| t.0),
                s.isolated_slots_on_each_device.iter().map(|(_, n)| *n).sum::<u64>(),
                format!("{:?}", w.pushing_the_root_ring).chars().take(40).collect::<String>(),
            )) },
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => None,
        },
        _ => None,
    };
    let Some((ways, n, r_txg, iso, reh)) = stuck else {
        println!("SON {label} grow={grow} first={} c_before={c_before:?} ab_before={ab_before}", atk_outcome(&first));
        return;
    };
    let iso_all_before = isolated_slots_of(&pool.session().allocator);
    let per_dev_before: Vec<u64> = pool.session().allocator.devices.iter().map(|m| m.isolated_slots()).collect();
    let pushed = atk_push(pool);
    let per_dev_after: Vec<u64> = pool.session().allocator.devices.iter().map(|m| m.isolated_slots()).collect();
    let real_released: u64 = per_dev_before.iter().zip(per_dev_after.iter()).map(|(b, a)| b.saturating_sub(*a)).sum();
    let mut retry = Vec::new();
    for _ in 0..2 {
        let again = pool.sequential_write((size_now + grow) * unit_bytes);
        let done = again.is_ok();
        retry.push(if done { "ok".to_string() } else { "refused".to_string() });
        if done { break; }
    }
    let text = if ways.len() == 2 { ways[1].clone() } else { String::new() };
    let num = |before: &str, after: &str| -> String {
        text.find(before).map(|i| &text[i + before.len()..]).and_then(|r| r.find(after).map(|j| r[..j].trim().to_string())).unwrap_or("-".into())
    };
    println!(
        "SON {label} grow={grow} STUCK listed={} reh={reh} n_field={n} r_txg={r_txg:?} iso_field={iso} c_before={c_before:?} ab_before={ab_before} iso_all_before={iso_all_before} push={pushed} real_released={real_released} text_times={} text_slots={} text_released={} retry={retry:?}",
        ways.len() == 2, num("从现行那一版起发 ", " 次"), num("接下来轮到的 ", " 槽"), num("隔离的槽放出来 ", " 个")
    );
}

#[test]
#[ignore = "正推腿原型，手动跑"]
fn son_two_c_stuck_sweep() {
    let grows: Vec<usize> = std::env::var("SON_GROWS").unwrap_or_else(|_| "2,10,20,30,40,50,60".to_string()).split(',').map(|t| t.parse().expect("数")).collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    for overwrites_before in [1u64, 3, 8] {
        let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(overwrites_before);
        let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
        let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
        for grow in &grows {
            let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
            let Ok(mounted) = mount_writable_with_space_admission(&recorded.pool.parameters, &mut faulted_devices, SpaceAdmission::JudgedByTheFormula) else {
                println!("SON two o={overwrites_before} mount_refused");
                continue;
            };
            let image_after_the_mount = image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
            let (output, session) = MountedSession::of_the_mount(mounted);
            let content_of_the_selected_version = recorded.pool.content_of_each_root.get(&recorded.key_of_p).cloned().expect("P");
            let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
            for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
                let root = version.root();
                content_of_each_root.insert((root.instance, root.checkpoint_txg), content_of_the_selected_version.clone());
            }
            let mut pool = PoolUnderTest {
                parameters: recorded.pool.parameters.clone(),
                device_bytes: recorded.pool.device_bytes,
                devices: plain_devices_on(&image_after_the_mount),
                session: Some(session),
                write_time_seconds: recorded.pool.write_time_seconds + 10_000,
                content_of_the_current_version: content_of_the_selected_version,
                content_of_each_root,
            };
            let size_now = pool.content_of_the_current_version.len().div_ceil(unit_bytes);
            sonnet_judge_stuck(&format!("two o={overwrites_before}"), &mut pool, size_now, *grow);
        }
    }
}

#[test]
#[ignore = "正推腿原型，手动跑"]
fn son_abandoned_and_c_stuck_sweep() {
    let ks: Vec<u64> = std::env::var("SON_KS").unwrap_or_else(|_| "0,2,6".to_string()).split(',').map(|t| t.parse().expect("数")).collect();
    let grows: Vec<usize> = std::env::var("SON_GROWS").unwrap_or_else(|_| "2,10,20,30,40,50,60".to_string()).split(',').map(|t| t.parse().expect("数")).collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    for k in ks {
        let stream = SharedStream::retaining_contents();
        let mut pool = PoolUnderTest {
            parameters: built.parameters.clone(),
            device_bytes: built.device_bytes,
            devices: plain_devices_on(&built.image).into_iter().map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))).collect(),
            session: None,
            write_time_seconds: built.write_time_seconds + 1000,
            content_of_the_current_version: Vec::new(),
            content_of_each_root: BTreeMap::new(),
        };
        pool.crash_and_mount_writable().expect("实例 4 可写挂载");
        for _ in 0..k { pool.overwrite(OVERWRITE_BYTES).expect("实例 4 覆盖写"); }
        let previous_root = *pool.session().current.root();
        let previous_content = pool.content_of_the_current_version.clone();
        let base = pool.image();
        let operations_before_r = stream.operation_count();
        let Ok(published) = pool.create_one_inode() else { println!("SON ab k={k} r_refused"); continue; };
        if !published.floor_raises.is_empty() { println!("SON ab k={k} r_pushed_raises"); continue; }
        let root = *pool.session().current.root();
        let operations = stream.retained_operations();
        let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&operations[operations_before_r..], &fixed_geometry_of_the_384_slot_small_devices());
        let root_index = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("根");
        let recorded = RecordedPublish {
            content: pool.content_of_the_current_version.clone(),
            pool, base, writes, root_index,
            key: (root.instance, root.checkpoint_txg),
            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
            previous_content,
        };
        let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
        for grow in &grows {
            let lift = ReadBackAllZeroLift::default();
            let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
            let Ok(mounted) = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) else { println!("SON ab k={k} mount_refused"); continue; };
            lift.lift();
            let image_after_the_mount = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
            let mut plain_pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
            let size_now = plain_pool.content_of_the_current_version.len().div_ceil(unit_bytes);
            sonnet_judge_stuck(&format!("ab k={k}"), &mut plain_pool, size_now, *grow);
        }
    }
}
