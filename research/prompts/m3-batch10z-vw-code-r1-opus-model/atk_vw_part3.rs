
// ---------- K1-1：上一轮那几族形在新接口上逐格重扫，连同被拒之后、要出路之前用户做的事 ----------

/// 单元区 `unit_area_slots` 槽（384 槽那一档的参数与环，只把单元区放大或放窄）的每块盘字节数。
fn atk_device_bytes_with_unit_area(unit_area_slots: u64) -> u64 {
    u64::try_from(
        i128::from(device_bytes_of_the_384_slot_small_devices())
            + (i128::from(unit_area_slots) - 384) * i128::from(singlefs_format::SLOT_BYTES),
    )
    .expect("盘宽为正")
}

/// 攻方上一轮 K1 那一形：前缀覆盖写 `pre` 次，文件每次长 `step` 个单元、长到 `fill` 个，再录一次长 3 个单元的顺序写 R，
/// 崩在 R 第一条记录之后。交回（R 之前的盘面、崩了的盘面、每条根的内容、之后的写入时刻）。
fn atk_k1_record_on(
    unit_area_slots: u64,
    pre: u64,
    fill: usize,
    step: usize,
) -> Option<(MemoryPool, MemoryPool, BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>, u64)> {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        atk_device_bytes_with_unit_area(unit_area_slots),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..pre {
        pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
    }
    let mut units = 1usize;
    while units + step <= fill {
        match pool.sequential_write((units + step) * unit_bytes) {
            Ok(_) => units += step,
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
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&operations[operations_before_r..], &geometry);
    let first_record_end = writes.iter().position(|write| write.kind == StepKind::JournalRecord)? + 1;
    let mut crashed = base.clone();
    crashed.apply_writes(&writes[..first_record_end]);
    Some((base, crashed, pool.content_of_each_root.clone(), pool.write_time_seconds + 100))
}

/// 推环入口，盘表照 `table` 摆（`swapped`：两块盘的身份对调；`reversed`：次序倒过来、身份与盘不拆）。
fn atk_push_with_table<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, table: &str) -> String {
    let mut session = pool.session.take().expect("挂着的会话");
    match table {
        "swapped" => {
            let first = pool.devices[0].0;
            let second = pool.devices[1].0;
            pool.devices[0].0 = second;
            pool.devices[1].0 = first;
        }
        "reversed" => pool.devices.reverse(),
        _ => {}
    }
    let pushed = session.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut pool.devices);
    match table {
        "swapped" => {
            let first = pool.devices[0].0;
            let second = pool.devices[1].0;
            pool.devices[0].0 = second;
            pool.devices[1].0 = first;
        }
        "reversed" => pool.devices.reverse(),
        _ => {}
    }
    pool.session = Some(session);
    match pushed {
        Ok(done) => {
            for version in &done.versions_published {
                pool.note_root_of_the_current_content(&version.root);
            }
            format!("pushed({})", done.versions_published.len())
        }
        Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
    }
}

/// 被拒之后、要出路之前用户做的一件事（`action`），做完照那一份旧报要出路、再照出路二真走（推环、再发同一个改动）逐格比。
/// 能做的：none、same（原样再发一次）、bigger（再长 5 个单元的写）、inode、overwrite、push-swapped、push-reversed、
/// unmount-remount、frozen-overwrite（一次覆盖写第一个改盘调用报块设备错、冻结）、push-fail-1、push-fail-12（推环真发时第 1 / 12 个改盘调用报错）、
/// ask-then-same（先要一次出路、原样再发、拿第二份报要出路，比两份报）。
fn atk_after_action(after_report: &AtkSnap, stuck: &StuckBehindTheWideProtection, change: &AtkChange, action: &str) -> (String, Option<bool>) {
    atk_after_action_with_older(after_report, None, stuck, change, action)
}

/// 同上；`older` 是被拒之前某一刻的盘面（`table-stale`：要出路与照出路走都交第二块盘的那一份旧盘面）。
/// 另几件改的是要出路与照出路走交进来的盘表（两边交同一份）：table-reversed（次序倒过来、身份与盘不拆）、table-swapped（两块盘的身份对调）、
/// table-one（只交第一块盘）、table-stale。
fn atk_after_action_with_older(
    after_report: &AtkSnap,
    older: Option<&MemoryPool>,
    stuck: &StuckBehindTheWideProtection,
    change: &AtkChange,
    action: &str,
) -> (String, Option<bool>) {
    let mut pool = after_report.pool();
    let mut note = String::new();
    if let Some(table) = action.strip_prefix("table-") {
        match table {
            "reversed" => pool.devices.reverse(),
            "swapped" => {
                let first = pool.devices[0].0;
                pool.devices[0].0 = pool.devices[1].0;
                pool.devices[1].0 = first;
            }
            "one" => {
                pool.devices.truncate(1);
            }
            "stale" => {
                let Some(older) = older else {
                    return (format!("act={action}: no_older_image"), None);
                };
                let stale = plain_devices_on(older).remove(1);
                pool.devices[1] = stale;
            }
            other => panic!("没有这种盘表：{other}"),
        }
        let ask = atk_ask_direct(&pool, stuck);
        let (line, ok) = atk_compare(&ask, &mut pool, change);
        return (format!("act={action}: {line}"), ok);
    }
    // inode-then-same-K / overwrite-then-same-K：先做 K 次小动作（建 inode / 覆盖写 2999 字节），再原样发同一个改动；
    // 那一次报卡住的，拿那一份新报要出路、照出路二真走（上一轮「放开小动作」那一族在新接口上的样子）。
    for (prefix, small) in [("inode-then-same-", "inode"), ("overwrite-then-same-", "overwrite")] {
        if let Some(k) = action.strip_prefix(prefix).map(|n| n.parse::<usize>().expect("数")) {
            let mut refused = 0;
            for _ in 0..k {
                let outcome = if small == "inode" { pool.create_one_inode() } else { pool.overwrite(OVERWRITE_BYTES) };
                if outcome.is_err() {
                    refused += 1;
                }
            }
            let again = atk_apply(&mut pool, change);
            let Some(stuck_again) = atk_stuck_of(&again) else {
                return (format!("act={action}:small_refused={refused} again={}", atk_kind(&again)), None);
            };
            let old_ask = atk_ask(&pool, stuck);
            let ask = atk_ask(&pool, stuck_again);
            let (line, ok) = atk_compare(&ask, &mut pool, change);
            return (format!("act={action}:small_refused={refused} again=stuck old_report_ask={} {line}", old_ask.head()), ok);
        }
    }
    if let Some(failing) = action.strip_prefix("push-fail-").map(|n| n.parse::<u64>().expect("数")) {
        let (mut failing_pool, _calls) = pool_failing_one_changing_call_copied_from(&pool, Some(failing));
        let txg_before = failing_pool.session().current.root().checkpoint_txg;
        note = match push_the_root_ring_of(&mut failing_pool) {
            Ok(done) => format!("pushed({})", done.versions_published.len()),
            Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(failed)) => {
                format!("push_failed_at({}/{})", failed.failed_publish_in_the_push, failed.publishes_in_the_push)
            }
            Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
        };
        let moved = failing_pool.session().current.root().checkpoint_txg.0 - txg_before.0;
        let frozen = failing_pool.session().allocator.frozen_publish().is_some();
        let ask = atk_ask_direct(&failing_pool, stuck);
        let (line, ok) = atk_compare(&ask, &mut failing_pool, change);
        return (format!("act={action}:{note} txg_moved={moved} frozen={frozen} {line}"), ok);
    }
    match action {
        "none" => {}
        "same" => note = atk_kind(&atk_apply(&mut pool, change)),
        "bigger" => {
            let units = change.content_after().map_or(1, |c| c.len().div_ceil(singlefs_core::unit::data_unit_payload_capacity())) + 5;
            let bigger = atk_seq_of(&mut pool, units);
            note = atk_kind(&atk_apply(&mut pool, &bigger));
        }
        "inode" => note = atk_kind(&pool.create_one_inode()),
        "overwrite" => note = atk_kind(&pool.overwrite(OVERWRITE_BYTES)),
        "push-swapped" => note = atk_push_with_table(&mut pool, "swapped"),
        "push-reversed" => note = atk_push_with_table(&mut pool, "reversed"),
        "unmount-remount" => {
            let mut session = pool.session.take().expect("会话");
            let unmounted = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            note = format!("unmount={} ", match &unmounted { Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)) => "raised".to_string(), Ok(_) => "nothing".to_string(), Err(error) => atk_head(format!("{error:?}")) });
            match pool.crash_and_mount_writable() {
                Ok(output) => note.push_str(&format!("remount=ok(instance {})", output.instance.0)),
                Err(error) => note.push_str(&format!("remount=refused[{}]", atk_head(format!("{error:?}")))),
            }
        }
        "frozen-overwrite" => {
            let failing = 1;
            let (mut failing_pool, _calls) = pool_failing_one_changing_call_copied_from(&pool, Some(failing));
            note = if action == "frozen-overwrite" {
                atk_kind(&failing_pool.overwrite(OVERWRITE_BYTES))
            } else {
                match push_the_root_ring_of(&mut failing_pool) {
                    Ok(done) => format!("pushed({})", done.versions_published.len()),
                    Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
                }
            };
            let frozen = failing_pool.session().allocator.frozen_publish().is_some();
            if !frozen {
                // 这件事一个改盘调用都没发（覆盖写在任何写之前就被拒）：注入的那一次报错还没用掉，之后的真推会撞上它——
                // 那是装置的故障留到了后面，不是这件事造成的状态。换回不带故障的盘接着比。
                let mut plain = AtkSnap::of(&failing_pool).pool();
                let ask = atk_ask(&plain, stuck);
                let (line, ok) = atk_compare(&ask, &mut plain, change);
                return (format!("act={action}:{note} frozen=false(fault_unused) {line}"), ok);
            }
            let ask = atk_ask_direct(&failing_pool, stuck);
            let (line, ok) = atk_compare(&ask, &mut failing_pool, change);
            return (format!("act={action}:{note} frozen={frozen} {line}"), ok);
        }
        "ask-then-same" => {
            let first_ask = atk_ask(&pool, stuck);
            let again = atk_apply(&mut pool, change);
            note = atk_kind(&again);
            if let Some(stuck_again) = atk_stuck_of(&again) {
                let second_ask = atk_ask(&pool, stuck_again);
                let old_ask = atk_ask(&pool, stuck);
                let same = format!("{:?}", first_ask.result) == format!("{:?}", second_ask.result)
                    && format!("{:?}", old_ask.result) == format!("{:?}", second_ask.result);
                note.push_str(&format!(" asks_same={same}"));
            }
        }
        other => panic!("没有这件事：{other}"),
    }
    let ask = atk_ask(&pool, stuck);
    let (line, ok) = atk_compare(&ask, &mut pool, change);
    (format!("act={action}:{note} {line}"), ok)
}

/// K1-1：攻方上一轮 9 形（前缀覆盖写 × 文件长到几个单元）× 长出去的单元数 g；报「被宽护卡住」的，对 `ATK_ACTIONS` 里每件事
/// 做完照旧报要出路、照出路二真走逐格比。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_ways_out_vs_real_sweep() {
    let pres: Vec<u64> = atk_env_list("ATK_PRES", "2,4,8");
    let fills: Vec<usize> = atk_env_list("ATK_FILLS", "6,11,16,21");
    let extra: usize = atk_env_or("ATK_EXTRA", 8);
    let actions: Vec<String> = std::env::var("ATK_ACTIONS").unwrap_or_else(|_| "none".to_string()).split(',').map(str::to_string).collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let area: u64 = atk_env_or("ATK_AREA", 384);
    let device_bytes = atk_device_bytes_with_unit_area(area);
    let (mut cells, mut stuck_cells) = (0usize, 0usize);
    let mut tally: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for pre in &pres {
        for fill in &fills {
            let label = format!("pre={pre} fill={fill}");
            let Some((base, crashed, contents, write_time_seconds)) = atk_k1_record_on(area, *pre, *fill, 5) else {
                println!("ATK k1 {label} r_not_recorded");
                continue;
            };
            let first_refused_of = |image: &MemoryPool| -> (usize, Option<usize>) {
                let (probe, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
                for grow in 1..=90usize {
                    let (mut trial, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                    if trial.sequential_write((size_now + grow) * unit_bytes).is_err() {
                        return (size_now, Some(grow));
                    }
                }
                (size_now, None)
            };
            let (size_now, g_ctrl) = first_refused_of(&base);
            let (_, g_c) = first_refused_of(&crashed);
            println!("ATK k1 {label} size_now={size_now} g_ctrl={g_ctrl:?} g_c={g_c:?}");
            let (Some(g_ctrl), Some(g_c)) = (g_ctrl, g_c) else { continue };
            let (mounted, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
            let mounted = AtkSnap::of(&mounted);
            for grow in g_c..=(g_ctrl + extra) {
                cells += 1;
                let mut pool = mounted.pool();
                let change = atk_seq_of(&mut pool, size_now + grow);
                let outcome = atk_apply(&mut pool, &change);
                let Some(stuck) = atk_stuck_of(&outcome) else {
                    println!("ATK k1 {label} g={grow} first={}", atk_kind(&outcome));
                    continue;
                };
                stuck_cells += 1;
                let after_report = AtkSnap::of(&pool);
                for action in &actions {
                    let (line, ok) = atk_after_action_with_older(&after_report, Some(&mounted.image), stuck, &change, action);
                    let entry = tally.entry(action.clone()).or_insert((0, 0, 0));
                    match ok {
                        Some(true) => entry.0 += 1,
                        Some(false) => entry.1 += 1,
                        None => entry.2 += 1,
                    }
                    println!("ATK k1 {label} g={grow} n_pub={} {line}", stuck.empty_publishes_until_the_ring_slot_of_r_is_written);
                }
            }
        }
    }
    for (action, (agree, mismatch, moved)) in &tally {
        println!("ATK k1 totals area={area} action={action} cells={cells} stuck={stuck_cells} agree={agree} mismatch={mismatch} moved_or_unasked={moved}");
    }
}
