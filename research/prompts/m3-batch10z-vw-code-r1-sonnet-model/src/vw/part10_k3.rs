
// ====================== 本轮正推腿 K1-2 / K3：报里记的「还剩什么」逐形核；推完之后那一段与走空的分开形 ======================

fn vw_remainder_text(stuck: &StuckBehindTheWideProtection) -> String {
    let debug = format!("{stuck:?}");
    let start = debug.find("remainder_of_the_change: ").expect("有这一项");
    debug[start + "remainder_of_the_change: ".len()..].trim_end_matches(" }").to_string()
}

/// 每块盘的单元区里没挡的槽只留 `left` 个（注入级：位图上占着，照实现员 `pool_whose_unlink_…` 的做法）。
fn vw_leave_unblocked<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, left: u64) {
    for device_map in &mut pool.session.as_mut().expect("会话").allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= left {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 把 C 那一圈换成「第 k 次发布写的那一槽」上每块盘 per 个隔离着的槽（注入级，旧的 C 登记整个换掉，旧的隔离位留着）。
fn vw_replace_c_with<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, k: u64, per: u64) {
    let session = pool.session.as_mut().expect("会话");
    let txg = session.current.root().checkpoint_txg.0;
    let spr = session.allocator.root_ring_occupancy().expect("根环表").slots_per_region();
    let rs = singlefs_core::root_ring::target_for_publish(CheckpointTxg(txg + k), spr);
    let mut placements = Vec::new();
    for d in 0..session.allocator.devices.len() {
        let dev = session.allocator.devices[d].device;
        let start = session.allocator.devices[d].unit_area_start().slot().0 + 40 + 90 * (d as u64);
        let mut found = 0;
        let mut slot = start;
        while found < per && slot < start + 600 {
            if !session.allocator.devices[d].is_blocked_for_commit_generated(SlotNumber(slot)) {
                placements.push(singlefs_core::allocator::PlacementOnDevice { device: dev, slot: SlotNumber(slot), span: 1 });
                found += 1;
            }
            slot += 3;
        }
    }
    for placement in &placements {
        session.allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
    }
    let mut map = BTreeMap::new();
    map.insert(rs, placements);
    session.allocator.note_placements_named_by_unapplied_journal_records(map);
}

fn vw_queue_len<Device: DeviceOverASparseImage>(pool: &PoolUnderTest<Device>) -> usize {
    pool.session().current.version_of_trees().map_or(0, |v| v.deleted_inodes.len())
}

/// K1-2 记对了没有：四种改动 × 没挡的槽留 0..=70 个（被拒点从意图、走空、这次改动一路扫过去）；
/// 报「被宽护卡住」的，从 `{:?}` 读记着的「还剩什么」，与发出去的那个改动逐项比（种类、内容、写入时间、inode、个数）；
/// 并对照真推环、再发同一个改动的结局（走空的那一支比队列有没有走空）。
#[test]
#[ignore = "正推腿 K1-2，手动跑"]
fn vw_k12_remainder_recorded_matches_the_refused_change() {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let base = crashed.mounted_and_stuck_once_by_a_write_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let snap = AtkSnap::of(&base);
    let unit = singlefs_core::unit::data_unit_payload_capacity();
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let lefts: Vec<u64> = std::env::var("VW_LEFTS").map(|v| v.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or_else(|_| (0..=70).collect());
    let kinds = ["overwrite", "sequential", "inodes", "unlink", "overwrite_after_unlink_queue"];
    for &left in &lefts {
        for kind in kinds {
            let mut pool = snap.pool();
            // 先把 unlink 的意图发成（走空那一步留在队列里）：只给 overwrite_after_unlink_queue 这一形
            vw_leave_unblocked(&mut pool, left);
            let write_time = 700_000 + left * 10;
            pool.write_time_seconds = write_time - 1;
            let content: Vec<u8> = match kind {
                "overwrite" => (0..2000u32).map(|i| (i * 7 + left as u32) as u8).collect(),
                "sequential" => (0..(3 * unit) as u32).map(|i| (i * 13 + left as u32) as u8).collect(),
                _ => Vec::new(),
            };
            if kind == "overwrite_after_unlink_queue" {
                pool.write_time_seconds += 1;
                let wt = pool.write_time_seconds;
                let unlink = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: wt }, None);
                let queue = vw_queue_len(&pool);
                if unlink.is_ok() || queue == 0 {
                    *tally.entry(format!("{kind} left={left}: unlink did not leave a queue")).or_insert(0) += 0;
                    continue;
                }
            }
            let file = AtkFirstFile { content: &content, write_time_seconds: write_time };
            let change = match kind {
                "overwrite" | "overwrite_after_unlink_queue" => AtkUserChange::Overwrite(file),
                "sequential" => AtkUserChange::SequentialWrite(file),
                "inodes" => AtkUserChange::NewInodes { count: 3, write_time_seconds: write_time },
                _ => AtkUserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: write_time },
            };
            let outcome = pool.publish(change, None);
            let head = match &outcome {
                Ok(_) => { *tally.entry(format!("{kind}: admitted")).or_insert(0) += 1; continue; }
                Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => match &no_space.wide_protection_at_the_ceiling {
                    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_) => "stuck",
                    WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "notwhat",
                },
                Err(_) => "other",
            };
            if head != "stuck" { *tally.entry(format!("{kind}: {head}")).or_insert(0) += 1; continue; }
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let remainder = vw_remainder_text(stuck);
            let versions_published = match &outcome { Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(ns)) => ns.versions_published.len(), _ => 0 };
            // 期望：种类与内容
            let expected_change = match kind {
                "overwrite" | "overwrite_after_unlink_queue" => format!("Overwrite {{ content: {:?}, write_time_seconds: {write_time} }}", content),
                "sequential" => format!("SequentialWrite {{ content: {:?}, write_time_seconds: {write_time} }}", content),
                "inodes" => format!("NewInodes {{ count: 3, write_time_seconds: {write_time} }}"),
                _ => format!("Unlink {{ inode: {:?}, write_time_seconds: {write_time} }}", InodeNumber(FIRST_INODE_NUMBER)),
            };
            let expected_remainder = if kind == "unlink" && versions_published >= 1 {
                "WalkTheQueue".to_string()
            } else {
                format!("WalkTheQueueThenPublishTheChange({expected_change})")
            };
            let ok = remainder == expected_remainder;
            // 对照真走：要出路、再真推环
            let ways = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
            let reh = atk_rehearsal_head_from(&ways);
            let class = |text: &str| if text.starts_with("adm") { "adm" } else if text.starts_with("still") { "still" } else { "pushref" }.to_string();
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match &pushed {
                Err(_) => "pushref".to_string(),
                Ok(_) => {
                    if remainder == "WalkTheQueue" {
                        // 只剩走空：真走 = 再发一个小覆盖写（它之前先走空），看队列走空了没有
                        let small = AtkFirstFile { content: &[9u8; 40], write_time_seconds: write_time + 1 };
                        let _ = pool.publish(AtkUserChange::Overwrite(small), None);
                        if vw_queue_len(&pool) == 0 { "adm".to_string() } else { "still".to_string() }
                    } else {
                        let again = pool.publish(change, None);
                        if again.is_ok() { "adm".to_string() } else { "still".to_string() }
                    }
                }
            };
            let agree = class(&reh) == class(&truth);
            *tally.entry(format!("{kind}: stuck remainder_ok={ok} reh={} truth={truth} agree={agree} versions_published_before_report={}", class(&reh), versions_published.min(2))).or_insert(0) += 1;
            if !ok || !agree {
                println!("VW k12 BAD kind={kind} left={left} remainder_ok={ok} got={} expected={}", remainder.chars().take(100).collect::<String>(), expected_remainder.chars().take(100).collect::<String>());
            }
        }
    }
    for (key, count) in &tally { println!("VW k12 {count:4} × {key}"); }
}

/// K3：推完之后那一段（F、H）与走空（walkSkipped）能不能分开：C 换成第 k 次发布写的那一槽上每块盘 per 个槽（推环只要 k 次空发布），
/// 没挡的槽留 left 个；unlink（意图之后的走空被拒）与它之后再覆盖写（先走空被拒）；对每个报「被宽护卡住」的：要出路交什么（adm / still / pushref），
/// 与真推环、再走一次的结局（走空：队列走空没有；覆盖写：再发放得下没有）。
#[test]
#[ignore = "正推腿 K3，手动跑"]
fn vw_k3_walk_after_the_push_sweep() {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let base = crashed.mounted_and_stuck_once_by_a_write_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let snap = AtkSnap::of(&base);
    let list = |name: &str, default: &str| -> Vec<u64> { std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').map(|x| x.parse().unwrap()).collect() };
    let ks = list("VW_KS", "1,2,3");
    let pers = list("VW_PERS", "1,2,4,8");
    let lefts = list("VW_LEFTS", "0,10,15,20,25,26,27,28,29,30,31,32,33,34,36,38,40,44,48,56,64");
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut firsts: BTreeMap<String, String> = BTreeMap::new();
    for &k in &ks { for &per in &pers { for &left in &lefts {
        for variant in ["unlink_walk_after_intent", "overwrite_walk_before"] {
            let mut pool = snap.pool();
            vw_replace_c_with(&mut pool, k, per);
            vw_leave_unblocked(&mut pool, left);
            pool.write_time_seconds += 1;
            let wt = pool.write_time_seconds;
            let unlink = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: wt }, None);
            let (outcome, change_kind) = if variant == "unlink_walk_after_intent" {
                (unlink, "unlink")
            } else {
                if unlink.is_ok() || vw_queue_len(&pool) == 0 { *tally.entry(format!("{variant}: no queue left")).or_insert(0) += 1; continue; }
                pool.write_time_seconds += 1;
                let content = vec![5u8; 3000];
                let file = AtkFirstFile { content: &content, write_time_seconds: pool.write_time_seconds };
                (pool.publish(AtkUserChange::Overwrite(file), None), "overwrite")
            };
            let (stuck_present, vp) = match &outcome {
                Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(ns)) => (matches!(&ns.wide_protection_at_the_ceiling, WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_)), ns.versions_published.len()),
                _ => (false, 0),
            };
            if !stuck_present {
                let what = match &outcome { Ok(_) => "admitted".to_string(), Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(ns)) => format!("notwhat({})", format!("{:?}", ns.stop).chars().take(40).collect::<String>()), Err(e) => format!("other:{}", format!("{e:?}").chars().take(30).collect::<String>()) };
                *tally.entry(format!("{variant}: {what}")).or_insert(0) += 1;
                continue;
            }
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let remainder = vw_remainder_text(stuck);
            let n = format!("{:?}", stuck).split("empty_publishes_until_the_ring_slot_of_r_is_written: ").nth(1).and_then(|r| r.split(',').next().map(String::from)).unwrap_or_default();
            let ways = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
            let reh = atk_rehearsal_head_from(&ways);
            let class = |text: &str| if text.starts_with("adm") { "adm" } else if text.starts_with("still") { "still" } else { "pushref" }.to_string();
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match &pushed {
                Err(_) => "pushref".to_string(),
                Ok(_) => {
                    if remainder == "WalkTheQueue" {
                        let small = AtkFirstFile { content: &[9u8; 40], write_time_seconds: wt + 50 };
                        let _ = pool.publish(AtkUserChange::Overwrite(small), None);
                        if vw_queue_len(&pool) == 0 { "adm".to_string() } else { "still".to_string() }
                    } else {
                        let content = vec![5u8; 3000];
                        let file = AtkFirstFile { content: &content, write_time_seconds: wt + 1 };
                        let again = pool.publish(AtkUserChange::Overwrite(file), None);
                        if again.is_ok() { "adm".to_string() } else { "still".to_string() }
                    }
                }
            };
            let key = format!("{variant}: stuck(change={change_kind},versions_before_report={vp}, remainder={}) reh={} truth={truth} agree={}", remainder.chars().take(32).collect::<String>(), class(&reh), class(&reh) == class(&truth));
            if std::env::var("VW_CELLS").is_ok() { println!("VW k3cell {variant} k={k} per={per} left={left} n={n} versions_before_report={vp} remainder={} reh={reh} truth={truth}", remainder.chars().take(32).collect::<String>()); }
            firsts.entry(key.clone()).or_insert(format!("k={k} per={per} left={left} n={n}"));
            *tally.entry(key).or_insert(0) += 1;
        }
    } } }
    for (key, count) in &tally { println!("VW k3 {count:4} × {key}   [first {}]", firsts.get(key).cloned().unwrap_or_default()); }
}
