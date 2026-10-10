
// ====================== 攻方原型（m3-batch10z-pqrs-code-r1，opus 腿）======================
// 下面全是攻方腿自己的用例，名字都以 atk_ 开头；上面原样是冻结副本里同名 harness 测试文件，只借它的搭建函数。

use singlefs_core::mount::MountOutput;

/// 一份镜像上新起一个池（两块普通稀疏盘），可写挂载一次（空间准入判着）。
fn atk_mounted_pool_from(
    image: &MemoryPool,
    contents: &BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    write_time_seconds: u64,
) -> Result<(PoolUnderTest<SparseBlockDevice>, MountOutput), MountError> {
    let mut pool = PoolUnderTest {
        parameters: parameters_of_the_384_slot_small_devices(),
        device_bytes: device_bytes_of_the_384_slot_small_devices(),
        devices: plain_devices_on(image),
        session: None,
        write_time_seconds,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: contents.clone(),
    };
    let output = pool.crash_and_mount_writable()?;
    Ok((pool, output))
}

/// C 那一圈：记着落点的槽数、记着的落点覆盖的槽这一刻隔离着几个（两盘合计）。
fn atk_c_ring(allocator: &PoolAllocator) -> (usize, u64) {
    let Some(occupancy) = allocator.root_ring_occupancy() else {
        return (0, 0);
    };
    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
    let placements: Vec<PlacementOnDevice> = holding.values().flatten().copied().collect();
    let isolated: u64 = allocator
        .isolated_slots_covered_by(&placements)
        .iter()
        .map(|(_, isolated)| *isolated)
        .sum();
    (holding.len(), isolated)
}

/// 一次用户改动的结局，一行字。
fn atk_outcome(outcome: &Result<singlefs_core::mounted_session::UserChangePublished, UserChangeRefused>) -> String {
    match outcome {
        Ok(published) => format!("ok(raises={})", published.floor_raises.len()),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
            let stop = match &no_space.stop {
                FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling } => {
                    format!("ceiling(F={},C={})", floor.0, ceiling.0)
                }
                FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => "budget".to_string(),
                FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => "raise-refused".to_string(),
            };
            let last = match &no_space.last_refusal {
                PublishError::SpaceAdmissionRefused(refused) => {
                    let shorts: Vec<String> = refused
                        .short_devices
                        .iter()
                        .map(|short| format!("{}:{}/{}", short.device.0, short.available.0, short.demand.0))
                        .collect();
                    format!("admission[{}]", shorts.join(","))
                }
                PublishError::PlacementRefused { .. } => "placement".to_string(),
                other => format!("{other:?}").chars().take(60).collect(),
            };
            let field = match &no_space.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "NotWhat".to_string(),
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) => format!(
                    "Stuck(r_slot={}.{},r_txg={:?},n={},iso={:?})",
                    stuck.ring_slot_of_r.region,
                    stuck.ring_slot_of_r.slot,
                    stuck.checkpoint_txg_of_r.map(|txg| txg.0),
                    stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
                    stuck
                        .isolated_slots_on_each_device
                        .iter()
                        .map(|(_, isolated)| *isolated)
                        .collect::<Vec<_>>()
                ),
            };
            format!("nospace(stop={stop},last={last},field={field},raises={})", no_space.floor_raises.len())
        }
        Err(other) => format!("other({})", format!("{other:?}").chars().take(120).collect::<String>()),
    }
}

/// 推一次环，一行字；做成时把推出来的每条根记成现行内容。
fn atk_push<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>) -> String {
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    match pushed {
        Ok(pushed) => {
            for version in &pushed.versions_published {
                pool.note_root_of_the_current_content(&version.root);
            }
            format!("pushed({})", pushed.versions_published.len())
        }
        Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(failed)) => {
            for version in &failed.versions_published {
                pool.note_root_of_the_current_content(&version.root);
            }
            format!(
                "failed_after_rehearsal(at={}/{},landed={},cause={})",
                failed.failed_publish_in_the_push,
                failed.publishes_in_the_push,
                failed.versions_published.len(),
                format!("{:?}", failed.cause).chars().take(90).collect::<String>()
            )
        }
        Err(other) => format!("refused({})", format!("{other:?}").chars().take(140).collect::<String>()),
    }
}

/// K1-1：盘被活数据占到边上、再崩在一次三条记录的顺序写半路（只有前几条记录落了，C 那一圈只有几个槽），挂上之后长文件被拒。
/// 对照臂是同一份盘面不带那几条记录（C 一槽都没有）。两臂各从同一份镜像起，逐个加大长出去的单元数，找第一个被拒的、看报的是哪一种；
/// 对照臂第一个被拒的那个长度，在 C 臂上推环之后再试一次。
fn atk_k1_fill_case(pre_overwrites: u64, fill_units: usize, step_units: usize) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..pre_overwrites {
        pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
    }
    let mut units = 1usize;
    while units + step_units <= fill_units {
        match pool.sequential_write((units + step_units) * unit_bytes) {
            Ok(_) => units += step_units,
            Err(_) => break,
        }
    }
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let r_outcome = pool.sequential_write((units + 3) * unit_bytes);
    let label = format!("pre={pre_overwrites} fill={fill_units} step={step_units} units={units}");
    match &r_outcome {
        Ok(published) if published.floor_raises.is_empty() => {}
        other => {
            println!("ATK k1 {label} r_not_recorded {}", atk_outcome(other));
            return;
        }
    }
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_index = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("R 写了根");
    let contents = pool.content_of_each_root.clone();
    let write_time_seconds = pool.write_time_seconds + 100;
    // 臂：对照（盘面停在 R 之前）与 C 臂（R 的写落到第 k 个为止，k 取每一条记录写之后、根之前）。
    let mut arms: Vec<(String, MemoryPool)> = vec![("ctrl".to_string(), base.clone())];
    for k in 1..root_index {
        if writes[k - 1].kind == StepKind::JournalRecord {
            let mut image = base.clone();
            image.apply_writes(&writes[..k]);
            arms.push((format!("crash@{k}/{root_index}"), image));
        }
    }
    let mut first_refused_in_the_control: Option<usize> = None;
    let c_arms_limit: usize = std::env::var("ATK_C_ARMS").map_or(2, |text| text.parse().expect("数"));
    let mut c_arms_run = 0usize;
    for (arm, image) in &arms {
        let Ok((probe, _)) = atk_mounted_pool_from(image, &contents, write_time_seconds) else {
            println!("ATK k1 {label} arm={arm} mount_refused");
            continue;
        };
        let (c_slots, c_isolated) = atk_c_ring(&probe.session().allocator);
        if arm != "ctrl" && c_slots == 0 {
            continue;
        }
        if arm != "ctrl" {
            c_arms_run += 1;
            if c_arms_run > c_arms_limit {
                continue;
            }
        }
        let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
        let mut first_refused = None;
        for grow in 1..=80usize {
            let (mut trial, _) =
                atk_mounted_pool_from(image, &contents, write_time_seconds).expect("同一份镜像再挂");
            let outcome = trial.sequential_write((size_now + grow) * unit_bytes);
            if outcome.is_err() {
                first_refused = Some((grow, atk_outcome(&outcome)));
                break;
            }
        }
        println!(
            "ATK k1 {label} arm={arm} size_now={size_now} c_slots={c_slots} c_iso={c_isolated} iso_all={} first_refused={first_refused:?}",
            isolated_slots_of(&probe.session().allocator)
        );
        if arm == "ctrl" {
            first_refused_in_the_control = first_refused.as_ref().map(|(grow, _)| *grow);
            continue;
        }
        let Some(grow) = first_refused_in_the_control else {
            continue;
        };
        // 对照臂被拒的那个长度：C 臂上先试（报什么），推环，再试同一个长度。
        let (mut trial, _) =
            atk_mounted_pool_from(image, &contents, write_time_seconds).expect("同一份镜像再挂");
        let before = trial.sequential_write((size_now + grow) * unit_bytes);
        let pushed = atk_push(&mut trial);
        let iso_after_push = isolated_slots_of(&trial.session().allocator);
        let after = trial.sequential_write((size_now + grow) * unit_bytes);
        let mut retries = Vec::new();
        for _retry in 0..4 {
            let again = trial.sequential_write((size_now + grow) * unit_bytes);
            let done = again.is_ok();
            retries.push(atk_outcome(&again));
            if done {
                break;
            }
        }
        let violations = violations_of_unit_reuse_on(&trial.image());
        println!(
            "ATK k1 {label} arm={arm} ctrl_grow={grow} before={} push={pushed} iso_after_push={iso_after_push} after={} retries={retries:?} viol={}",
            atk_outcome(&before),
            atk_outcome(&after),
            violations.len()
        );
        if let Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) = &before {
            if let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) = &no_space.wide_protection_at_the_ceiling {
                let atk_w = trial.session().ways_out_of_the_wide_protection(&trial.devices, stuck).expect("要出路");
                for (index, way) in atk_w.ways_out.iter().enumerate() {
                    println!("ATK k1 {label} arm={arm} way_out[{index}]={way}");
                }
            }
        }
        // C 臂自己第一个被拒的长度（宽护确实挡着的那一档）：推环，再试同一个长度至多五次。
        if let Some((own_grow, _)) = first_refused.as_ref() {
            let (mut own, _) =
                atk_mounted_pool_from(image, &contents, write_time_seconds).expect("同一份镜像再挂");
            let own_before = own.sequential_write((size_now + own_grow) * unit_bytes);
            let own_pushed = atk_push(&mut own);
            let mut own_after = Vec::new();
            for _attempt in 0..5 {
                let again = own.sequential_write((size_now + own_grow) * unit_bytes);
                let done = again.is_ok();
                own_after.push(atk_outcome(&again));
                if done {
                    break;
                }
            }
            println!(
                "ATK k1own {label} arm={arm} own_grow={own_grow} before={} push={own_pushed} after={own_after:?} viol={}",
                atk_outcome(&own_before),
                violations_of_unit_reuse_on(&own.image()).len()
            );
        }
        // 出路一（卸载重挂）在这一形上：C 臂挂上之后被拒、照出路一卸载再可写挂载，看 C 那一圈还在不在、C 臂第一个被拒的长度与对照臂被拒的长度各报什么。
        let (mut way_one, _) =
            atk_mounted_pool_from(image, &contents, write_time_seconds).expect("同一份镜像再挂");
        let first_refused_grow = first_refused.as_ref().map_or(grow, |(first, _)| *first);
        let refused_first = way_one.sequential_write((size_now + first_refused_grow) * unit_bytes);
        let mut session = way_one.session.take().expect("会话");
        let unmounted = unmount(
            &way_one.parameters,
            &mut way_one.devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        );
        let unmounted_text = match &unmounted {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                for version in &raised.publishes {
                    way_one.note_root_of_the_current_content(&version.root);
                }
                format!("ok({})", raised.publishes.len())
            }
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing".to_string(),
            Err(refusal) => format!("{refusal:?}").chars().take(100).collect(),
        };
        let remount = way_one.crash_and_mount_writable();
        let (c_slots_again, c_iso_again) = match &remount {
            Ok(_) => atk_c_ring(&way_one.session().allocator),
            Err(_) => (usize::MAX, 0),
        };
        let after_way_one_small = if remount.is_ok() {
            atk_outcome(&way_one.sequential_write((size_now + first_refused_grow) * unit_bytes))
        } else {
            format!("remount={:?}", remount.err()).chars().take(100).collect()
        };
        println!(
            "ATK k1 {label} arm={arm} way_one: refused_first={} unmount={unmounted_text} remount_c_slots={c_slots_again} remount_c_iso={c_iso_again} grow{first_refused_grow}_after={after_way_one_small} viol={}",
            atk_outcome(&refused_first),
            violations_of_unit_reuse_on(&way_one.image()).len()
        );
    }
}

#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_fill_sweep() {
    let fills: Vec<usize> = std::env::var("ATK_FILLS")
        .unwrap_or_else(|_| "120,180,240".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let pre: u64 = std::env::var("ATK_PRE").map_or(4, |text| text.parse().expect("数"));
    let step: usize = std::env::var("ATK_STEP").map_or(20, |text| text.parse().expect("数"));
    for fill in fills {
        atk_k1_fill_case(pre, fill, step);
    }
}

/// 一个长度在 C 臂上的三样：先试报什么、推环之后至多试三次放没放、照出路一卸载重挂之后放没放。
fn atk_k1_ways_out_at(
    image: &MemoryPool,
    contents: &BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    write_time_seconds: u64,
    size_now: usize,
    grow: usize,
) -> (String, String, String) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let (mut pool, _) = atk_mounted_pool_from(image, contents, write_time_seconds).expect("再挂");
    let before = pool.sequential_write((size_now + grow) * unit_bytes);
    let field = match &before {
        Ok(_) => "admitted".to_string(),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
            match &no_space.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "NotWhat".to_string(),
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(s) => { let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, s).expect("要出路"); format!("Stuck:{}:{}", if w.ways_out.len() == 2 { "A" } else { "R" }, { let d = format!("{:?}", w.pushing_the_root_ring); if d.contains("stop: PublishesPerAdmission") { "budget" } else if d.contains("stop: FloorAlreadyAtTheCeiling") { "ceil" } else if d.contains("stop: FloorRaiseRefusedForSpace") { "space" } else if d.starts_with("PushRefused") { "pushrefused" } else if d.starts_with("ChangeAdmitted") { "adm" } else { "other" } }) }
            }
        }
        Err(_) => "other".to_string(),
    };
    if before.is_ok() {
        return (field, "-".to_string(), "-".to_string());
    }
    let pushed = atk_push(&mut pool);
    let mut push_then = format!("fail({pushed},iso_after={})", isolated_slots_of(&pool.session().allocator));
    let attempts: usize = std::env::var("ATK_PUSH_ATTEMPTS").map_or(3, |text| text.parse().expect("数"));
    for attempt in 1..=attempts {
        if pool.sequential_write((size_now + grow) * unit_bytes).is_ok() {
            push_then = format!("ok@{attempt}");
            break;
        }
    }
    let (mut way_one, _) = atk_mounted_pool_from(image, contents, write_time_seconds).expect("再挂");
    let _refused = way_one.sequential_write((size_now + grow) * unit_bytes);
    let mut session = way_one.session.take().expect("会话");
    let unmounted = unmount(
        &way_one.parameters,
        &mut way_one.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let way_one_then = match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            for version in &raised.publishes {
                way_one.note_root_of_the_current_content(&version.root);
            }
            match way_one.crash_and_mount_writable() {
                Ok(_) => {
                    if way_one.sequential_write((size_now + grow) * unit_bytes).is_ok() {
                        "ok".to_string()
                    } else {
                        format!("fail(c_slots={})", atk_c_ring(&way_one.session().allocator).0)
                    }
                }
                Err(refusal) => format!("remount_refused({})", format!("{refusal:?}").chars().take(60).collect::<String>()),
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing".to_string(),
        Err(refusal) => format!("unmount_refused({})", format!("{refusal:?}").chars().take(60).collect::<String>()),
    };
    (field, push_then, way_one_then)
}

/// K1-1 / K1-2 扫：前缀覆盖写次数、文件长到几个单元（每步长 5 个）都放开；R 是长 3 个单元的顺序写，崩在它第一条记录之后。
/// 每一形里：对照臂第一个被拒的长度 g_ctrl、C 臂第一个被拒的长度 g_c；g 从 g_c 扫到 g_ctrl + 2，每个 g 报 C 臂的三样。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_ways_out_sweep() {
    let pres: Vec<u64> = std::env::var("ATK_PRES")
        .unwrap_or_else(|_| "2,4,8".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let fills: Vec<usize> = std::env::var("ATK_FILLS")
        .unwrap_or_else(|_| "6,11,16,21".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    for pre in &pres {
        for fill in &fills {
            let stream = SharedStream::retaining_contents();
            let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
                parameters_of_the_384_slot_small_devices(),
                device_bytes_of_the_384_slot_small_devices(),
                |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            );
            for _ in 0..*pre {
                pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
            }
            let mut units = 1usize;
            while units + 5 <= *fill {
                match pool.sequential_write((units + 5) * unit_bytes) {
                    Ok(_) => units += 5,
                    Err(_) => break,
                }
            }
            let base = pool.image();
            let operations_before_r = stream.operation_count();
            let label = format!("pre={pre} fill={fill} units={units}");
            match pool.sequential_write((units + 3) * unit_bytes) {
                Ok(published) if published.floor_raises.is_empty() => {}
                other => {
                    println!("ATK k1w {label} r_not_recorded {}", atk_outcome(&other));
                    continue;
                }
            }
            let operations = stream.retained_operations();
            let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
                &operations[operations_before_r..],
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            let root_index = writes
                .iter()
                .position(|write| write.kind == StepKind::RootRecordFua)
                .expect("R 写了根");
            let first_record_end = writes
                .iter()
                .enumerate()
                .filter(|(_, write)| write.kind == StepKind::JournalRecord)
                .map(|(index, _)| index + 1)
                .next()
                .expect("R 写了记录");
            let contents = pool.content_of_each_root.clone();
            let write_time_seconds = pool.write_time_seconds + 100;
            let mut crashed = base.clone();
            crashed.apply_writes(&writes[..first_record_end]);
            let first_refused_of = |image: &MemoryPool| -> (usize, Option<usize>) {
                let (probe, _) = atk_mounted_pool_from(image, &contents, write_time_seconds).expect("挂");
                let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
                for grow in 1..=80usize {
                    let (mut trial, _) = atk_mounted_pool_from(image, &contents, write_time_seconds).expect("挂");
                    if trial.sequential_write((size_now + grow) * unit_bytes).is_err() {
                        return (size_now, Some(grow));
                    }
                }
                (size_now, None)
            };
            let (size_now, g_ctrl) = first_refused_of(&base);
            let (_, g_c) = first_refused_of(&crashed);
            let (c_slots, c_iso) = {
                let (probe, _) = atk_mounted_pool_from(&crashed, &contents, write_time_seconds).expect("挂");
                atk_c_ring(&probe.session().allocator)
            };
            println!("ATK k1w {label} crash_after_write={first_record_end}/{root_index} size_now={size_now} c_slots={c_slots} c_iso={c_iso} g_ctrl={g_ctrl:?} g_c={g_c:?}");
            let (Some(g_ctrl), Some(g_c)) = (g_ctrl, g_c) else {
                continue;
            };
            for grow in g_c..=(g_ctrl + 2) {
                let (field, push_then, way_one_then) =
                    atk_k1_ways_out_at(&crashed, &contents, write_time_seconds, size_now, grow);
                println!("ATK k1w {label} g={grow} ctrl={} field={field} push_then={push_then} way_one_then={way_one_then}", if grow >= g_ctrl { "refused" } else { "admitted" });
            }
        }
    }
}

/// K1 那一形的盘面：交回（R 之前的盘面、崩在 R 第一条记录之后的盘面、每条根的内容、之后的写入时刻、P 的单元数）。
fn atk_k1_record(
    pre: u64,
    fill: usize,
) -> Option<(MemoryPool, MemoryPool, BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>, u64, usize)> {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..pre {
        pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
    }
    let mut units = 1usize;
    while units + 5 <= fill {
        match pool.sequential_write((units + 5) * unit_bytes) {
            Ok(_) => units += 5,
            Err(_) => break,
        }
    }
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    match pool.sequential_write((units + 3) * unit_bytes) {
        Ok(published) if published.floor_raises.is_empty() => {}
        _ => return None,
    }
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_record_end = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
        .map(|(index, _)| index + 1)
        .next()?;
    let mut crashed = base.clone();
    crashed.apply_writes(&writes[..first_record_end]);
    Some((base, crashed, pool.content_of_each_root.clone(), pool.write_time_seconds + 100, units))
}

fn atk_state(pool: &PoolUnderTest<SparseBlockDevice>) -> String {
    let allocator = &pool.session().allocator;
    let unblocked: Vec<u64> = allocator.devices.iter().map(|map| map.unblocked_slots()).collect();
    let ceiling = {
        let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置");
        let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
            .expect("实例表")
            .records;
        rollback_floor_ceiling(
            &pool.devices,
            &system_configuration,
            pool.rollback_floor(),
            &table,
            &std::collections::BTreeSet::new(),
        )
        .map(|txg| txg.0)
    };
    format!(
        "txg={} F={} ceiling={ceiling:?} iso={} unblocked={unblocked:?} c={:?}",
        pool.session().current.root().checkpoint_txg.0,
        pool.rollback_floor().0,
        isolated_slots_of(allocator),
        atk_c_ring(allocator)
    )
}

/// K1-2 那一形逐步看：推环之前、推环之后、推环之后每一次重试（F、上限、隔离、没挡的槽）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_push_trace() {
    let pre: u64 = std::env::var("ATK_PRE").map_or(4, |text| text.parse().expect("数"));
    let fill: usize = std::env::var("ATK_FILL").map_or(16, |text| text.parse().expect("数"));
    let grow: usize = std::env::var("ATK_GROW").map_or(20, |text| text.parse().expect("数"));
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let (_base, crashed, contents, write_time_seconds, units) = atk_k1_record(pre, fill).expect("录得下");
    let (mut pool, _) = atk_mounted_pool_from(&crashed, &contents, write_time_seconds).expect("挂");
    println!("ATK trace mounted {}", atk_state(&pool));
    let before = pool.sequential_write((units + grow) * unit_bytes);
    println!("ATK trace before {} | {}", atk_outcome(&before), atk_state(&pool));
    let pushed = atk_push(&mut pool);
    println!("ATK trace push={pushed} | {}", atk_state(&pool));
    for attempt in 1..=6 {
        let again = pool.sequential_write((units + grow) * unit_bytes);
        println!("ATK trace attempt{attempt} {} | {}", atk_outcome(&again), atk_state(&pool));
        if again.is_ok() {
            break;
        }
    }
    let small = pool.overwrite(OVERWRITE_BYTES);
    println!("ATK trace overwrite_after {} | {} viol={}", atk_outcome(&small), atk_state(&pool), violations_of_unit_reuse_on(&pool.image()).len());
}

/// K1-2 推环之后用户先做 k 次小覆盖写（2999 字节）、再试被拒的那个长度：k 从 0 扫到 8，看第几次起放得下。
/// 推环之前同样先做 k 次小覆盖写再试（不推环那一臂），对照。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_small_writes_after_push_sweep() {
    let pre: u64 = std::env::var("ATK_PRE").map_or(4, |text| text.parse().expect("数"));
    let fill: usize = std::env::var("ATK_FILL").map_or(16, |text| text.parse().expect("数"));
    let grows: Vec<usize> = std::env::var("ATK_GROWS")
        .unwrap_or_else(|_| "20,28,35".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let (_base, crashed, contents, write_time_seconds, units) = atk_k1_record(pre, fill).expect("录得下");
    let small_action_is_inode = std::env::var("ATK_SMALL").map_or(false, |text| text == "inode");
    let max_small: usize = std::env::var("ATK_MAX_SMALL").map_or(8, |text| text.parse().expect("数"));
    for grow in grows {
        for pushes in [false, true] {
            let mut cells = Vec::new();
            for small_writes in 0..=max_small {
                let (mut pool, _) = atk_mounted_pool_from(&crashed, &contents, write_time_seconds).expect("挂");
                let _first = pool.sequential_write((units + grow) * unit_bytes);
                if pushes {
                    let _ = atk_push(&mut pool);
                }
                let mut small_refused = 0;
                for _ in 0..small_writes {
                    let small = if small_action_is_inode { pool.create_one_inode() } else { pool.overwrite(OVERWRITE_BYTES) };
                    if small.is_err() {
                        small_refused += 1;
                    }
                }
                // 覆盖写把文件改成 2999 字节：再试时长回原来那个长度（P 的单元数加 grow）。
                let again = pool.sequential_write((units + grow) * unit_bytes);
                cells.push(format!(
                    "k{small_writes}:{}{}",
                    if again.is_ok() { "ok" } else { "no" },
                    if small_refused > 0 { format!("(small_refused={small_refused})") } else { String::new() }
                ));
            }
            println!("ATK k1small small={} pre={pre} fill={fill} grow={grow} push={pushes} {}", if small_action_is_inode { "inode" } else { "overwrite" }, cells.join(" "));
        }
    }
}

/// K1-2 在实现员自己的三形上：卡住之后用户要写的是长到 g 个单元的顺序写（不是覆盖写一次），推环之后至多试三次放没放、
/// 卸载重挂之后放没放。g 放开扫。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_implementer_forms_sweep() {
    let grows: Vec<usize> = std::env::var("ATK_GROWS")
        .unwrap_or_else(|_| "2,5,10,20,30,40,60,80".to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let forms = [
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 20 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation { overwrites_before: 40 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion { overwrites_before: 160 },
    ];
    for form in forms {
        for grow in &grows {
            let PoolStuckBehindTheWideProtection { mut pool, stuck, .. } = pool_stuck_behind_the_wide_protection(form);
            let iso: u64 = stuck.isolated_slots_on_each_device.iter().map(|(_, isolated)| *isolated).sum();
            let first = pool.sequential_write(grow * unit_bytes);
            let field = match &first {
                Ok(_) => "admitted".to_string(),
                Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => match &no_space.wide_protection_at_the_ceiling {
                    WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "NotWhat".to_string(),
                    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(s) => { let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, s).expect("要出路"); format!("Stuck:{}", if w.ways_out.len() == 2 { "A" } else { "R" }) }
                },
                Err(_) => "other".to_string(),
            };
            let image = pool.image();
            let contents = pool.content_of_each_root.clone();
            let t = pool.write_time_seconds;
            let mut push_then = "-".to_string();
            if first.is_err() {
                let pushed = atk_push(&mut pool);
                push_then = format!("fail({pushed})");
                for attempt in 1..=3 {
                    if pool.sequential_write(grow * unit_bytes).is_ok() {
                        push_then = format!("ok@{attempt}");
                        break;
                    }
                }
            }
            // 卸载重挂那一臂：从被拒那一刻的盘面另起一份（会话丢了：等于崩了再挂，不是正常卸载；正常卸载那一臂在下面）。
            let _ = (&image, &contents, t);
            let PoolStuckBehindTheWideProtection { mut pool, .. } = pool_stuck_behind_the_wide_protection(form);
            let _refused = pool.sequential_write(grow * unit_bytes);
            let mut session = pool.session.take().expect("会话");
            let unmounted = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            let way_one = match unmounted {
                Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                    for version in &raised.publishes {
                        pool.note_root_of_the_current_content(&version.root);
                    }
                    match pool.crash_and_mount_writable() {
                        Ok(_) => if pool.sequential_write(grow * unit_bytes).is_ok() { "ok".to_string() } else { "fail".to_string() },
                        Err(refusal) => format!("remount_refused({})", format!("{refusal:?}").chars().take(50).collect::<String>()),
                    }
                }
                Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing".to_string(),
                Err(refusal) => format!("unmount_refused({})", format!("{refusal:?}").chars().take(50).collect::<String>()),
            };
            println!("ATK k1impl form={form:?} grow={grow} stuck_iso={iso} n={} first={field} push_then={push_then} way_one={way_one}", stuck.empty_publishes_until_the_ring_slot_of_r_is_written);
        }
    }
}
