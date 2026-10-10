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
        let ask = atk_ask_with_table(after_report, older, table, stuck).expect("盘表那一份造得出");
        let restored = atk_ask(&after_report.pool(), stuck).head();
        let (line, ok) = atk_compare(&ask, &mut pool, change);
        return (format!("act={action}: restored_ask={restored} {line}"), ok);
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
