
// ---------- K1-1 新形：unlink 卡在意图之后的走空里（H）、推环做得成、推完走空放得下 ----------

/// 实现员 `pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push` 那一形，三个注入量放开：
/// C 那一圈整个换成现行那一版之后第 `k` 次发布写的那一槽、每块盘记着 `iso` 个隔离着的空闲槽（`k` = 0 不换），每块盘没挡的槽只留 `left` 个；
/// unlink 第一个文件。交回（卡住那一刻的池、那一份报、意图之后发成了几版）；没卡住交回 `Err(结局短名)`。
fn atk_h_form(base: &AtkSnap, k: u64, iso: u64, left: u64) -> Result<(PoolUnderTest<SparseBlockDevice>, StuckBehindTheWideProtection, usize, AtkChange), String> {
    let mut pool = base.pool();
    let ring_slot = target_for_publish(
        CheckpointTxg(pool.session().current.root().checkpoint_txg.0 + k),
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let free_on_each: Vec<u64> = allocator
            .devices
            .iter()
            .map(|device_map| {
                let start = device_map.unit_area_start().slot().0;
                (start..start + device_map.unit_area_slots()).filter(|slot| device_map.is_free(SlotNumber(*slot))).count() as u64
            })
            .collect();
        // k = 0：不换 C 那一圈（挂载自己算出来的那一圈原样），只压没挡的槽（实现员 `pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent` 那一造法）。
        if k > 0 {
            if free_on_each.iter().any(|free| *free < iso) {
                return Err(format!("free={free_on_each:?}<iso"));
            }
            let placements = isolate_free_slots(allocator, iso);
            allocator.note_placements_named_by_unapplied_journal_records(BTreeMap::from([(ring_slot, placements)]));
        }
        leave_only_unblocked_slots_on_each_device(allocator, left);
    }
    pool.write_time_seconds += 1;
    let unlink = AtkChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: pool.write_time_seconds };
    let outcome = atk_apply(&mut pool, &unlink);
    let published = atk_versions_published_of(&outcome);
    let Some(stuck) = atk_stuck_of(&outcome) else {
        return Err(format!("unlink={} published={published}", atk_kind(&outcome)));
    };
    let stuck = atk_take_stuck(outcome).expect("刚判过是被宽护卡住");
    Ok((pool, stuck, published, unlink))
}

/// 从一次结局里把那一份报拿出来（`atk_stuck_of` 只借）。
fn atk_take_stuck(outcome: Result<AtkPublished, UserChangeRefused>) -> Option<StuckBehindTheWideProtection> {
    match outcome {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => match no_space.wide_protection_at_the_ceiling {
            WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) => Some(*stuck),
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => None,
        },
        _ => None,
    }
}

/// K1-1：H 形三个注入量（k、iso、left）扫一遍。报卡住的：要出路（数调用），再照出路二真走三种后续——推环之后原样再发同一个 unlink
/// （出路二与文档说的「再发这次改动」）、推环之后建一个 inode、推环之后什么都不发再看现行那一版的队列与成员。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_h_walk_after_the_intent_admitted_after_the_push() {
    let ks: Vec<u64> = atk_env_list("ATK_KS", "1,2,4,8,16");
    let isos: Vec<u64> = atk_env_list("ATK_ISOS", "4,8,16,32,64");
    let lefts: Vec<u64> = atk_env_list("ATK_LEFTS", "0,5,10,20,30");
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let base = AtkSnap::of(&crashed.mounted_and_stuck_once_by_a_write_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY));
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for k in &ks {
        for iso in &isos {
            for left in &lefts {
                let label = format!("k={k} iso={iso} left={left}");
                let (pool, stuck, published, unlink) = match atk_h_form(&base, *k, *iso, *left) {
                    Ok(found) => found,
                    Err(why) => {
                        *tally.entry(format!("not-stuck:{}", atk_head(why.clone()))).or_insert(0) += 1;
                        println!("ATK k1h {label} {why}");
                        continue;
                    }
                };
                let where_stuck = if published >= 1 { "H" } else { "intent" };
                let ask = atk_ask(&pool, &stuck);
                let ways = match &ask.result { Ok(ways) => ways.ways_out.clone(), Err(_) => Vec::new() };
                let after = AtkSnap::of(&pool);
                let resend = atk_push_then(&after, &unlink);
                let inode = AtkChange::Inodes { count: 1, write_time_seconds: pool.write_time_seconds + 1 };
                let then_inode = atk_push_then(&after, &inode);
                let mut pushed_only = after.pool();
                let push_only = match push_the_root_ring_of(&mut pushed_only) {
                    Ok(done) => format!("pushed({})", done.versions_published.len()),
                    Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
                };
                let (member, queue) = match &pushed_only.session().current {
                    PoolVersion::WithFile(v) => ("WithFile", v.deleted_inodes.is_empty()),
                    PoolVersion::WithTreesAndNoFile(v) => ("WithTreesAndNoFile", v.deleted_inodes.is_empty()),
                    PoolVersion::WithoutFile(_) => ("WithoutFile", true),
                };
                let head = ask.head();
                let resend_admitted = resend.ends_with("->admitted");
                let mismatch = head == "adm" && !resend_admitted;
                *tally.entry(format!("stuck-{where_stuck}:{}:resend_ok={resend_admitted}", atk_head(head.clone()))).or_insert(0) += 1;
                println!(
                    "ATK k1h {label} stuck-{where_stuck} published={published} n_pub={} remainder={} ask={head} {} ways={} | real: resend_unlink={resend} then_inode={then_inode} push_only={push_only}:{member}:queue_empty={queue}{}",
                    stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
                    remainder_of_the_change_recorded_in(&stuck).chars().take(48).collect::<String>(),
                    ask.io(),
                    ways.len(),
                    if mismatch { " !ADM_BUT_RESEND_REFUSED" } else { "" }
                );
                if mismatch && tally.get("printed-way-two").is_none() {
                    tally.insert("printed-way-two".to_string(), 1);
                    println!("ATK k1h   way_two={}", ways.get(1).cloned().unwrap_or_default());
                    let mut copy = after.pool();
                    let _ = push_the_root_ring_of(&mut copy);
                    println!("ATK k1h   resend_detail={:?}", atk_apply(&mut copy, &unlink).err().map(|e| format!("{e:?}").chars().take(400).collect::<String>()));
                }
            }
        }
    }
    println!("ATK k1h totals {tally:?}");
}
