
// ---------- K3：造一形把 F、H 推完之后那一段与 walkSkipped 分开（卡在走空里、预演里推环做得成） ----------

/// 每块盘没挡的槽标成已分配、只留 `leave` 个（注入级，与实现员 `pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent` 同一造法）。
fn atk_leave_unblocked(pool: &mut PoolUnderTest<SparseBlockDevice>, leave: u64) {
    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= leave {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// C 那一圈罩着、这一刻隔离着的槽，每块盘只留 `keep` 个隔离着（其余标成已分配，隔离位不动：推环解开隔离之后它们照旧占着，
/// 推环放不出它们；注入级）。交回每块盘标了几个。
fn atk_keep_only_isolated_by_c(pool: &mut PoolUnderTest<SparseBlockDevice>, keep: u64) -> Vec<u64> {
    let session = pool.session.as_mut().expect("挂着的会话");
    let placements: Vec<PlacementOnDevice> = session
        .allocator
        .root_ring_occupancy()
        .expect("根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_values()
        .flatten()
        .collect();
    let mut marked = Vec::new();
    for device_map in &mut session.allocator.devices {
        let mut kept = 0u64;
        let mut count = 0u64;
        let mut seen = std::collections::BTreeSet::new();
        let device = device_map.device;
        for placement in placements.iter().filter(|placement| placement.device == device) {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                if !seen.insert(slot) || !device_map.is_isolated(SlotNumber(slot)) {
                    continue;
                }
                if kept < keep {
                    kept += 1;
                } else {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                    count += 1;
                }
            }
        }
        marked.push(count);
    }
    marked
}

/// 照出路二真走、推完之后发 `then`（不必是被拒的那一个改动）：交回推环短名与再发结局短名。
fn atk_push_then(snap: &AtkSnap, then: &AtkChange) -> String {
    let mut pool = snap.pool();
    match push_the_root_ring_of(&mut pool) {
        Ok(done) => format!("pushed({})->{}", done.versions_published.len(), atk_kind(&atk_apply(&mut pool, then))),
        Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
    }
}

/// 搜：K1 那一形挂上之后长 13 个单元被拒一次，再覆盖写 `ow` 次（推环要推的次数随之变少），再把没挡的槽压到 `leave` 个，unlink 第一个文件：
/// 卡在意图之后的走空里（H）的，要出路、照出路二真走（推完再发同一个 unlink；推完建一个 inode）；再从那一刻发覆盖写与顺序写：
/// 卡在这次改动之前的走空里（F）的，要出路、照出路二真走。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k3_walk_stuck_with_a_push_that_goes_through() {
    let ows: Vec<usize> = atk_env_list("ATK_OWS", "0,2,4,6,8,10,12,14,16,18");
    let leaves: Vec<u64> = atk_env_list("ATK_LEAVES", "10,15,20,22,24,26,28,29,30,31,32,34,36,40,45,50,60");
    let grows: Vec<usize> = atk_env_list("ATK_F_GROWS", "1,5,13,20");
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let stuck_once = crashed.mounted_and_stuck_once_by_a_write_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for ow in &ows {
        let mut base = AtkSnap::of(&stuck_once).pool();
        let mut done = 0;
        for _ in 0..*ow {
            if base.overwrite(OVERWRITE_BYTES).is_err() {
                break;
            }
            done += 1;
        }
        if done < *ow {
            println!("ATK k3walk ow={ow} overwrites_refused_after={done}");
            continue;
        }
        // ATK_RESTICK=1：覆盖写之后再发长写直到报卡住（会话把 F 再抬到上限），推环要推的次数随之变少。
        if atk_env_or("ATK_RESTICK", 0u8) == 1 {
            let size_now = base.content_of_the_current_version.len().div_ceil(unit_bytes);
            let mut restuck = None;
            for grow in 1..=80usize {
                let mut trial = AtkSnap::of(&base).pool();
                let change = atk_seq_of(&mut trial, size_now + grow);
                let outcome = atk_apply(&mut trial, &change);
                if let Some(stuck) = atk_stuck_of(&outcome) {
                    restuck = Some((grow, stuck.empty_publishes_until_the_ring_slot_of_r_is_written));
                    base = trial;
                    break;
                }
            }
            println!("ATK k3walk ow={ow} restuck={restuck:?}");
            if restuck.is_none() {
                continue;
            }
        }
        let base = AtkSnap::of(&base);
        for leave in &leaves {
            let mut pool = base.pool();
            atk_leave_unblocked(&mut pool, *leave);
            let keep_iso: u64 = atk_env_or("ATK_KEEP_ISO", u64::MAX);
            if keep_iso != u64::MAX {
                atk_keep_only_isolated_by_c(&mut pool, keep_iso);
            }
            pool.write_time_seconds += 1;
            let unlink = AtkChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: pool.write_time_seconds };
            let outcome = atk_apply(&mut pool, &unlink);
            let published = atk_versions_published_of(&outcome);
            let label = format!("ow={ow} leave={leave}");
            let Some(stuck) = atk_stuck_of(&outcome) else {
                *tally.entry(format!("unlink:{}", atk_head(atk_kind(&outcome)))).or_insert(0) += 1;
                println!("ATK k3walk {label} unlink={} published={published}", atk_kind(&outcome));
                continue;
            };
            let where_stuck = if published >= 1 { "H" } else { "intent" };
            let after = AtkSnap::of(&pool);
            let ask = atk_ask(&after.pool(), stuck);
            let head = ask.head();
            *tally.entry(format!("unlink:stuck-{where_stuck}:{}", atk_head(head.clone()))).or_insert(0) += 1;
            let mut t = pool.write_time_seconds;
            t += 1;
            let inode = AtkChange::Inodes { count: 1, write_time_seconds: t };
            let resend = atk_push_then(&after, &unlink);
            let then_inode = atk_push_then(&after, &inode);
            println!(
                "ATK k3walk {label} unlink=stuck-{where_stuck} n_pub={} remainder={} ask={head} {} | real: resend_unlink={resend} then_inode={then_inode}",
                stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
                remainder_of_the_change_recorded_in(stuck).chars().take(60).collect::<String>(),
                ask.io()
            );
            if where_stuck != "H" {
                continue;
            }
            let size_now = after.content_now.len().div_ceil(unit_bytes);
            let mut f_changes: Vec<(String, AtkChange)> = Vec::new();
            for grow in &grows {
                t += 1;
                f_changes.push((format!("seq(+{grow})"), AtkChange::Sequential { content: content_of((size_now + grow) * unit_bytes, t), write_time_seconds: t }));
            }
            t += 1;
            f_changes.push(("overwrite".to_string(), AtkChange::Overwrite { content: content_of(OVERWRITE_BYTES, t), write_time_seconds: t }));
            for (name, change) in &f_changes {
                let mut f_pool = after.pool();
                let f_outcome = atk_apply(&mut f_pool, change);
                let f_published = atk_versions_published_of(&f_outcome);
                let Some(f_stuck) = atk_stuck_of(&f_outcome) else {
                    *tally.entry(format!("F:{}", atk_head(atk_kind(&f_outcome)))).or_insert(0) += 1;
                    println!("ATK k3walk {label} F {name} first={} published={f_published}", atk_kind(&f_outcome));
                    continue;
                };
                let f_after = AtkSnap::of(&f_pool);
                let (line, ok) = atk_follow_way_two(&f_after, f_stuck, change);
                let f_where = if f_published == 0 { "walk-before" } else { "after-walk" };
                *tally.entry(format!("F:stuck-{f_where}:{}:ok={ok}", atk_head(line.split(' ').next().unwrap_or("").to_string()))).or_insert(0) += 1;
                println!(
                    "ATK k3walk {label} F {name} stuck-{f_where} published={f_published} remainder={} {line}",
                    remainder_of_the_change_recorded_in(f_stuck).chars().take(70).collect::<String>()
                );
            }
        }
    }
    println!("ATK k3walk totals {tally:?}");
}
