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
