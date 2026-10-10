
// ---------- K1-1：实现员三形、两槽 C、被抛弃根 A 与 C 同在（上一轮那几族），在新接口上逐格重扫 ----------

fn atk_actions_from_env() -> Vec<String> {
    std::env::var("ATK_ACTIONS").unwrap_or_else(|_| "none".to_string()).split(',').map(str::to_string).collect()
}

/// 在一个卡住的池上扫写的长度：每个 g 发一次长到 g 个单元的顺序写（`total_units` 为真时 g 是整份长度，否则是现在长度加 g），
/// 报「被宽护卡住」的对每件事做完照旧报要出路、照出路二真走逐格比。交回（报卡住的格数, 对不上的格数, moved 的格数）。
fn atk_scan_lengths_and_follow(label: &str, snap: &AtkSnap, grows: &[usize], total_units: bool) -> (usize, usize, usize) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let size_now = snap.content_now.len().div_ceil(unit_bytes);
    let actions = atk_actions_from_env();
    let (mut stuck_cells, mut disagree, mut moved) = (0, 0, 0);
    for grow in grows {
        let units = if total_units { *grow } else { size_now + grow };
        let mut pool = snap.pool();
        let change = atk_seq_of(&mut pool, units);
        let outcome = atk_apply(&mut pool, &change);
        let Some(stuck) = atk_stuck_of(&outcome) else {
            println!("ATK {label} g={grow} first={}", atk_kind(&outcome));
            continue;
        };
        stuck_cells += 1;
        let after_report = AtkSnap::of(&pool);
        for action in &actions {
            let (line, ok) = atk_after_action_with_older(&after_report, Some(&snap.image), stuck, &change, action);
            match ok {
                Some(true) => {}
                Some(false) => disagree += 1,
                None => moved += 1,
            }
            println!("ATK {label} g={grow} n_pub={} {line}", stuck.empty_publishes_until_the_ring_slot_of_r_is_written);
        }
    }
    (stuck_cells, disagree, moved)
}

/// 实现员三形：卡住的那次覆盖写自己的报照出路二走；再在卡住那一刻起写长到 g 个单元（放开）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_implementer_forms_ways_out_vs_real() {
    let grows: Vec<usize> = atk_env_list("ATK_GROWS", "2,5,10,20,30,40,50,60,70,80");
    let forms = [
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 20 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation { overwrites_before: 40 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion { overwrites_before: 160 },
    ];
    let actions = atk_actions_from_env();
    let (mut stuck_cells, mut disagree, mut moved) = (0, 0, 0);
    for form in forms {
        let PoolStuckBehindTheWideProtection { pool, stuck, .. } = pool_stuck_behind_the_wide_protection(form);
        let snap = AtkSnap::of(&pool);
        let own = AtkChange::Overwrite {
            content: content_of(OVERWRITE_BYTES, pool.write_time_seconds),
            write_time_seconds: pool.write_time_seconds,
        };
        stuck_cells += 1;
        for action in &actions {
            let (line, ok) = atk_after_action_with_older(&snap, None, &stuck, &own, action);
            match ok {
                Some(true) => {}
                Some(false) => disagree += 1,
                None => moved += 1,
            }
            println!("ATK k1impl form={form:?} own_change {line}");
        }
        let (cells, bad, mv) = atk_scan_lengths_and_follow(&format!("k1impl form={form:?}"), &snap, &grows, true);
        stuck_cells += cells;
        disagree += bad;
        moved += mv;
    }
    println!("ATK k1impl totals stuck={stuck_cells} disagree={disagree} moved={moved} actions={}", actions.join(","));
}

/// 两槽 C（Q 顺序写、R 建 inode，故障挂上就撤）：挂上之后先覆盖写 `pre` 次（放开），再扫写的长度。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_two_c_slots_ways_out_vs_real() {
    let pre_writes_list: Vec<usize> = atk_env_list("ATK_PRE_WRITES", "0,1,3,6,10");
    let grows: Vec<usize> = (1..=90).collect();
    let (mut stuck_cells, mut disagree, mut moved) = (0, 0, 0);
    for overwrites_before in [1u64, 3, 8] {
        for pre_writes in &pre_writes_list {
            let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(overwrites_before);
            let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
            let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
            let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
            let Ok(mounted) = mount_writable_with_space_admission(&recorded.pool.parameters, &mut faulted_devices, SpaceAdmission::JudgedByTheFormula) else {
                println!("ATK k1two o={overwrites_before} pre={pre_writes} mount_refused");
                continue;
            };
            let image_after_the_mount = image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
            let (output, session) = MountedSession::of_the_mount(mounted);
            let content_of_p = recorded.pool.content_of_each_root.get(&recorded.key_of_p).cloned().expect("P");
            let mut contents = recorded.pool.content_of_each_root.clone();
            for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
                let root = version.root();
                contents.insert((root.instance, root.checkpoint_txg), content_of_p.clone());
            }
            let mut pool = PoolUnderTest {
                parameters: recorded.pool.parameters.clone(),
                device_bytes: recorded.pool.device_bytes,
                devices: plain_devices_on(&image_after_the_mount),
                session: Some(session),
                write_time_seconds: recorded.pool.write_time_seconds + 10_000,
                content_of_the_current_version: content_of_p,
                content_of_each_root: contents,
            };
            for _ in 0..*pre_writes {
                let _ = pool.overwrite(OVERWRITE_BYTES);
            }
            let label = format!("k1two o={overwrites_before} pre={pre_writes}");
            let (cells, bad, mv) = atk_scan_lengths_and_follow(&label, &AtkSnap::of(&pool), &grows, false);
            stuck_cells += cells;
            disagree += bad;
            moved += mv;
        }
    }
    println!("ATK k1two totals stuck={stuck_cells} disagree={disagree} moved={moved}");
}

/// 环里另有一条被抛弃根 A，同一个实例里再录一次建 inode 得 R、崩在 R 的根落盘之后轮换之前，挂载那一刻 R 写的全部单元读回全零（挂上就撤）：
/// C 那一圈与被抛弃根 A 同时在。覆盖写 `k` 次（放开）之后扫写的长度。
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
