
// ---------- K1-2：报里记的「还剩什么」是不是被拒那一刻真剩下的那一份（四种改动） ----------

/// 报里记着的「这次改动还剩什么」应当长的样子（`{:?}`，内容、写入时刻、inode、个数逐字）。
fn atk_expected_remainder(change: &AtkChange, walk_only: bool) -> String {
    if walk_only {
        return "remainder_of_the_change: WalkTheQueue }".to_string();
    }
    let inner = match change {
        AtkChange::Sequential { content, write_time_seconds } => format!("SequentialWrite {{ content: {content:?}, write_time_seconds: {write_time_seconds} }}"),
        AtkChange::Overwrite { content, write_time_seconds } => format!("Overwrite {{ content: {content:?}, write_time_seconds: {write_time_seconds} }}"),
        AtkChange::Unlink { inode, write_time_seconds } => format!("Unlink {{ inode: {inode:?}, write_time_seconds: {write_time_seconds} }}"),
        AtkChange::Inodes { count, write_time_seconds } => format!("NewInodes {{ count: {count}, write_time_seconds: {write_time_seconds} }}"),
    };
    format!("remainder_of_the_change: WalkTheQueueThenPublishTheChange({inner}) }}")
}

/// 一个卡住的池上发 `change`：报卡住的，比报里记着的「还剩什么」（`walk_only`：意图已发成、只剩走空），要出路、照出路二真走。
fn atk_kind_cell(label: &str, snap: &AtkSnap, change: &AtkChange, tally: &mut BTreeMap<String, usize>) {
    let mut pool = snap.pool();
    let outcome = atk_apply(&mut pool, change);
    let published = atk_versions_published_of(&outcome);
    let Some(stuck) = atk_stuck_of(&outcome) else {
        println!("ATK k1kinds {label} first={} published={published}", atk_kind(&outcome));
        *tally.entry(format!("first={}", atk_head(atk_kind(&outcome)))).or_insert(0) += 1;
        return;
    };
    let recorded = remainder_of_the_change_recorded_in(stuck);
    let walk_only = matches!(change, AtkChange::Unlink { .. }) && published >= 1;
    let remainder_ok = recorded == atk_expected_remainder(change, walk_only);
    let after_report = AtkSnap::of(&pool);
    let (line, ok) = atk_follow_way_two(&after_report, stuck, change);
    let key = format!("stuck walk_only={walk_only} remainder_ok={remainder_ok} follow_ok={ok}");
    *tally.entry(key).or_insert(0) += 1;
    println!("ATK k1kinds {label} published={published} remainder_ok={remainder_ok} {line}");
}

/// K1-2：四种改动各在卡住的池上发一次（覆盖写几种长度、顺序写、建几个 inode、unlink 第一个文件），报卡住的比报里记着的那一份与照出路走。
/// 池：K1 主形（pre=4 fill=16）挂上之后被拒一次；实现员三形卡住那一刻。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_remainder_of_each_kind_of_change() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let mut snaps: Vec<(String, AtkSnap)> = Vec::new();
    {
        let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, 4, 16, 5).expect("录得下");
        let (mut pool, _) = atk_mounted_pool_from(&parameters_of_the_384_slot_small_devices(), device_bytes_of_the_384_slot_small_devices(), &crashed, &contents, write_time_seconds).expect("挂");
        let size_now = pool.content_of_the_current_version.len().div_ceil(unit_bytes);
        let first = atk_seq_of(&mut pool, size_now + 13);
        let outcome = atk_apply(&mut pool, &first);
        assert!(atk_stuck_of(&outcome).is_some(), "主形第一次被拒报卡住");
        snaps.push(("k1main".to_string(), AtkSnap::of(&pool)));
    }
    for form in [
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 20 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation { overwrites_before: 40 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion { overwrites_before: 160 },
    ] {
        let PoolStuckBehindTheWideProtection { pool, .. } = pool_stuck_behind_the_wide_protection(form);
        snaps.push((format!("{form:?}"), AtkSnap::of(&pool)));
    }
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for (name, snap) in &snaps {
        let size_now = snap.content_now.len().div_ceil(unit_bytes);
        let mut changes: Vec<(String, AtkChange)> = Vec::new();
        let mut time = snap.write_time_seconds;
        let mut next_time = || {
            time += 1;
            time
        };
        for bytes in [1usize, 2999, unit_bytes / 2, unit_bytes] {
            let t = next_time();
            changes.push((format!("overwrite({bytes})"), AtkChange::Overwrite { content: content_of(bytes, t), write_time_seconds: t }));
        }
        for grow in [1usize, 5, 13, 20, 40] {
            let t = next_time();
            changes.push((format!("seq(+{grow})"), AtkChange::Sequential { content: content_of((size_now + grow) * unit_bytes, t), write_time_seconds: t }));
        }
        for count in [1u64, 2, 8, 32, 128, 512] {
            let t = next_time();
            changes.push((format!("inodes({count})"), AtkChange::Inodes { count, write_time_seconds: t }));
        }
        let t = next_time();
        changes.push(("unlink(first)".to_string(), AtkChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: t }));
        for (label, change) in &changes {
            atk_kind_cell(&format!("{name} {label}"), snap, change, &mut tally);
        }
    }
    println!("ATK k1kinds totals {tally:?}");
}

// ---------- K1-3：只比实例代号与 txg：别的池、同一对数 ----------

/// 把一个池用覆盖写推到 txg `target`（覆盖写一次 txg 加一；推过头或被拒交 false）。
fn atk_advance_to_txg(pool: &mut PoolUnderTest<SparseBlockDevice>, target: CheckpointTxg) -> bool {
    for _ in 0..64 {
        let now = pool.session().current.root().checkpoint_txg;
        if now == target {
            return true;
        }
        if now.0 > target.0 || pool.overwrite(OVERWRITE_BYTES).is_err() {
            return false;
        }
    }
    false
}

/// K1-3：别的池（同一份 mkfs 参数、同一个实例代号），推到被拒那一版的 txg，拿第一个池的报去要出路。
/// 文档注释说「这一份报不是这个会话的」也拒成 SessionMoved（`mounted_session.rs` 的 `WaysOutOfTheWideProtectionRefused`）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_ways_out_of_another_pools_report() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let cells: Vec<(u64, usize, usize)> = vec![(4, 16, 13), (4, 16, 20), (2, 16, 13), (8, 11, 20)];
    for (pre, fill, grow) in cells {
        let (base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, pre, fill, 5).expect("录得下");
        let (mut first_pool, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
        let size_now = first_pool.content_of_the_current_version.len().div_ceil(unit_bytes);
        let change = atk_seq_of(&mut first_pool, size_now + grow);
        let outcome = atk_apply(&mut first_pool, &change);
        let Some(stuck) = atk_stuck_of(&outcome) else {
            println!("ATK k1other pre={pre} fill={fill} g={grow} first={}", atk_kind(&outcome));
            continue;
        };
        let refused_on = *first_pool.session().current.root();
        let own = atk_ask(&first_pool, stuck);
        let own_ways = match &own.result {
            Ok(ways) => ways.ways_out.clone(),
            Err(_) => Vec::new(),
        };
        // 别的池：同一个 K1 前缀没录 R 的盘面（没有 C 那一圈）；K1 另一格崩了的盘面。
        let mut others: Vec<(String, PoolUnderTest<SparseBlockDevice>)> = Vec::new();
        others.push(("same-prefix-without-r".to_string(), atk_mounted_pool_from(&parameters, device_bytes, &base, &contents, write_time_seconds).expect("挂").0));
        if let Some((_b, other_crashed, other_contents, other_time)) = atk_k1_record_on(384, if pre == 4 { 2 } else { 4 }, 16, 5) {
            others.push(("another-k1-cell".to_string(), atk_mounted_pool_from(&parameters, device_bytes, &other_crashed, &other_contents, other_time).expect("挂").0));
        }
        for (name, mut other) in others {
            let reached = atk_advance_to_txg(&mut other, refused_on.checkpoint_txg);
            let other_root = *other.session().current.root();
            let same_pair = other_root.instance == refused_on.instance && other_root.checkpoint_txg == refused_on.checkpoint_txg;
            let ask = atk_ask(&other, stuck);
            let ways_text = match &ask.result {
                Ok(ways) => format!("ways={} same_text_as_own={}", ways.ways_out.len(), ways.ways_out == own_ways),
                Err(_) => "-".to_string(),
            };
            let real_push = {
                let mut copy = AtkSnap::of(&other).pool();
                atk_real_follow(&mut copy, &change).0
            };
            println!(
                "ATK k1other pre={pre} fill={fill} g={grow} own={} other={name} reached={reached} same_pair={same_pair} pair={}.{} ask={} {} {ways_text} real_push_on_other={real_push}",
                own.head(), other_root.instance.0, other_root.checkpoint_txg.0, ask.head(), ask.io()
            );
            if let Ok(ways) = &ask.result {
                println!("ATK k1other   way_one_on_other={}", ways.ways_out[0].chars().take(160).collect::<String>());
            }
        }
    }
}
