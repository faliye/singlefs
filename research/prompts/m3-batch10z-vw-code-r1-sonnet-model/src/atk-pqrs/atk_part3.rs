
// ---------- K2-2：C 有多槽（Q 与 R）、环里另有被抛弃根时推环 ----------

/// 录一段推环的写流，逐个前缀（每个写之后，不再挂）判 I-7.4、I-4.8；另报每条根落之前那一态与之后那一态里 `keys` 每条根在不在环里。
fn atk_push_recorded_and_judged<Device: DeviceOverASparseImage>(
    label: &str,
    pool: &mut PoolUnderTest<Device>,
    stream: &SharedStream,
    keys: &[(&str, (InstanceGeneration, CheckpointTxg))],
) -> String {
    let image_before = pool.image();
    let iso_before = isolated_slots_of(&pool.session().allocator);
    let operations_before = stream.operation_count();
    let pushed = atk_push(pool);
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let mut violating_prefixes = Vec::new();
    let mut leave_order: Vec<String> = Vec::new();
    let mut in_ring_before: Vec<bool> = keys.iter().map(|(_, key)| root_is_in_the_ring(&image_before, *key)).collect();
    for k in 0..=writes.len() {
        let mut image = image_before.clone();
        image.apply_writes(&writes[..k]);
        let violations = violations_of_unit_reuse_on(&image);
        if !violations.is_empty() {
            violating_prefixes.push(format!("k{k}:{}", violations[0].chars().take(140).collect::<String>()));
        }
        if k > 0 && writes[k - 1].kind == StepKind::RootRecordFua {
            for (index, (name, key)) in keys.iter().enumerate() {
                let now = root_is_in_the_ring(&image, *key);
                if in_ring_before[index] && !now {
                    leave_order.push(format!("{name}@root{}", writes[..k].iter().filter(|write| write.kind == StepKind::RootRecordFua).count()));
                }
                in_ring_before[index] = now;
            }
        }
    }
    let allocator = &pool.session().allocator;
    format!(
        "ATK {label} push={pushed} writes={} iso_before={iso_before} iso_after={} c_after={:?} abandoned_after={:?} leave_order={leave_order:?} violating_prefixes={} first={:?}",
        writes.len(),
        isolated_slots_of(allocator),
        atk_c_ring(allocator),
        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator).len(),
        violating_prefixes.len(),
        violating_prefixes.first()
    )
}

/// 之后覆盖写 n 次、每次判 I-7.4 / I-4.8，交回（被拒次数, 违例次数）。
fn atk_overwrites_and_judge<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, n: usize) -> (usize, usize) {
    let mut refused = 0;
    let mut violated = 0;
    for _ in 0..n {
        if pool.overwrite(OVERWRITE_BYTES).is_err() {
            refused += 1;
        }
        if !violations_of_unit_reuse_on(&pool.image()).is_empty() {
            violated += 1;
        }
    }
    (refused, violated)
}

/// K2-2 一：丙并中间那一形（Q 顺序写、R 建 inode，故障 cfgq,qroot,qlast,racct），挂上之后先覆盖写 `pre_writes` 次（放开），再推环。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k2_two_c_slots_push() {
    let pre_writes_list: Vec<usize> = std::env::var("ATK_PRE_WRITES")
        .unwrap_or_else(|_| "0,1,3,6,10".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    for overwrites_before in [1u64, 3, 8] {
        for pre_writes in &pre_writes_list {
            let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(overwrites_before);
            let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
            let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
            let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
            let Ok(mounted) = mount_writable_with_space_admission(&recorded.pool.parameters, &mut faulted_devices, SpaceAdmission::JudgedByTheFormula) else {
                println!("ATK k2two o={overwrites_before} pre={pre_writes} mount_refused");
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
            let stream = SharedStream::retaining_contents();
            let mut pool = PoolUnderTest {
                parameters: recorded.pool.parameters.clone(),
                device_bytes: recorded.pool.device_bytes,
                devices: plain_devices_on(&image_after_the_mount)
                    .into_iter()
                    .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
                    .collect(),
                session: Some(session),
                write_time_seconds: recorded.pool.write_time_seconds + 10_000,
                content_of_the_current_version: content_of_the_selected_version,
                content_of_each_root,
            };
            let c_at_mount = atk_c_ring(&pool.session().allocator);
            let mut pre_refused = 0;
            for _ in 0..*pre_writes {
                if pool.overwrite(OVERWRITE_BYTES).is_err() {
                    pre_refused += 1;
                }
            }
            let c_before_push = atk_c_ring(&pool.session().allocator);
            let label = format!("k2two o={overwrites_before} pre={pre_writes} c_at_mount={c_at_mount:?} c_before_push={c_before_push:?} pre_refused={pre_refused}");
            let line = atk_push_recorded_and_judged(&label, &mut pool, &stream, &[("Q", recorded.key_of_q), ("R", recorded.key_of_r)]);
            let (refused, violated) = atk_overwrites_and_judge(&mut pool, 5);
            println!("{line} after5(refused={refused},violated={violated})");
        }
    }
}

/// K2-2 二：环里另有一条被抛弃根 A（崩溃恢复抛弃的，根环表记着它的落点），同一个实例里再录一次建 inode 得 R、崩在 R 的根落盘之后轮换之前，
/// 可写挂载那一刻 R 写的全部单元读回全零（挂上就撤）：C 那一圈与被抛弃根 A 同时在。覆盖写 `k` 次（放开）之后推环。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k2_abandoned_root_and_c_push() {
    let ks: Vec<u64> = std::env::var("ATK_KS")
        .unwrap_or_else(|_| "0,1,2,4,6".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let key_of_a = (built.abandoned_root.instance, built.abandoned_root.checkpoint_txg);
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
        let abandoned_at_mount = ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).len();
        for _ in 0..k {
            pool.overwrite(OVERWRITE_BYTES).expect("实例 4 覆盖写");
        }
        let previous_root = *pool.session().current.root();
        let previous_content = pool.content_of_the_current_version.clone();
        let base = pool.image();
        let operations_before_r = stream.operation_count();
        let r = pool.create_one_inode();
        let Ok(published) = r else {
            println!("ATK k2ab k={k} r_refused");
            continue;
        };
        if !published.floor_raises.is_empty() {
            println!("ATK k2ab k={k} r_pushed_raises");
            continue;
        }
        let root = *pool.session().current.root();
        let operations = stream.retained_operations();
        let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
            &operations[operations_before_r..],
            &fixed_geometry_of_the_384_slot_small_devices(),
        );
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
                println!("ATK k2ab k={k} mount_refused({})", format!("{refusal:?}").chars().take(80).collect::<String>());
                continue;
            }
        };
        lift.lift();
        let image_after_the_mount = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
        let mut plain_pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
        let a_in_ring = root_is_in_the_ring(&plain_pool.image(), key_of_a);
        let abandoned_now = ring_slots_recorded_as_holding_an_abandoned_root_by(&plain_pool.session().allocator).len();
        let c_now = atk_c_ring(&plain_pool.session().allocator);
        let iso_all = isolated_slots_of(&plain_pool.session().allocator);
        // 被宽护卡住那一报（若卡住）：先试一次覆盖写。
        let first = plain_pool.overwrite(OVERWRITE_BYTES);
        let first_text = atk_outcome(&first);
        // 推环：换成录写流的盘。
        let snapshot = AtkStuckSnapshot::of(&plain_pool);
        let (mut recording, push_stream) = snapshot.recording_pool();
        let label = format!(
            "k2ab k={k} abandoned_at_mount={abandoned_at_mount} abandoned_now={abandoned_now} a_in_ring={a_in_ring} c={c_now:?} iso_all={iso_all} first={first_text}"
        );
        let line = atk_push_recorded_and_judged(&label, &mut recording, &push_stream, &[("A", key_of_a), ("R", recorded.key)]);
        let (refused, violated) = atk_overwrites_and_judge(&mut recording, 8);
        println!("{line} after8(refused={refused},violated={violated})");
    }
}
