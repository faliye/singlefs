
// ---------- K1-3：别的会话的报这两个数恰好相同时，出路文字里哪些数是报里的（照实写的那一段文档逐句对） ----------

/// 出路二那句话里的数：（R 那一槽, 要推的次数, 放出来的合计）；没有出路二交 None。
fn atk_way_two_numbers(ways: &[String]) -> Option<(String, String, String)> {
    let two = ways.get(1)?;
    let slot = two.split("根环槽 ").nth(1)?.split('）').next()?.to_string();
    let publishes = two.split("从现行那一版起发 ").nth(1)?.split(' ').next()?.to_string();
    let released = two.split("隔离的槽放出来 ").nth(1)?.split(' ').next()?.to_string();
    Some((slot, publishes, released))
}

/// K1-3：报的池 A（K1 那一形 `ATK_A`=pre:fill:g，挂上之后先覆盖写 j 次再长 g 个单元被拒）；别的池 B（K1 别的格崩了的盘面挂上，
/// 覆盖写推到 A 被拒那一版的 txg，实例代号都是挂载取的 2）。拿 A 的报去 B 要出路：比出路一、出路二的文字与数（R 那一槽、要推的次数、放出来的槽数）
/// 各是谁的：A 自己要出路的、B 自己报卡住（B 发同一个改动）之后要出路的、照 B 的状态真推环推了几次。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k13_ways_out_on_another_session_with_the_same_pair() {
    let a_cell: Vec<usize> = atk_env_list("ATK_A", "4,16,13");
    let (a_pre, a_fill, a_grow) = (a_cell[0] as u64, a_cell[1], a_cell[2]);
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let (_a_base, a_crashed, a_contents, a_time) = atk_k1_record_on(384, a_pre, a_fill, 5).expect("录得下");
    let (a_mounted, _) = atk_mounted_pool_from(&parameters, device_bytes, &a_crashed, &a_contents, a_time).expect("挂");
    let a_mounted = AtkSnap::of(&a_mounted);
    let mut a_variants = Vec::new();
    for j in 0..16usize {
        let mut a = a_mounted.pool();
        if (0..j).any(|_| a.overwrite(OVERWRITE_BYTES).is_err()) {
            break;
        }
        let size_now = a.content_of_the_current_version.len().div_ceil(unit_bytes);
        let change = atk_seq_of(&mut a, size_now + a_grow);
        let outcome = atk_apply(&mut a, &change);
        if let Some(stuck) = atk_take_stuck(outcome) {
            let refused_on = *a.session().current.root();
            a_variants.push((j, refused_on, stuck, change, AtkSnap::of(&a)));
        }
    }
    println!("ATK k13 A=pre{a_pre}:fill{a_fill}:g{a_grow} stuck_variants={:?}", a_variants.iter().map(|(j, r, ..)| (*j, r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>());
    for b_pre in [2u64, 4, 8] {
        for b_fill in [6usize, 11, 16, 21] {
            if (b_pre, b_fill) == (a_pre, a_fill) {
                continue;
            }
            let Some((_b_base, b_crashed, b_contents, b_time)) = atk_k1_record_on(384, b_pre, b_fill, 5) else { continue };
            let Ok((b_probe, _)) = atk_mounted_pool_from(&parameters, device_bytes, &b_crashed, &b_contents, b_time) else { continue };
            let b_txg = b_probe.session().current.root().checkpoint_txg;
            let b_instance = b_probe.session().current.root().instance;
            let Some((j, refused_on, stuck, change, a_after)) = a_variants.iter().find(|(_, r, ..)| r.checkpoint_txg.0 >= b_txg.0 && r.instance == b_instance) else {
                println!("ATK k13 B=pre{b_pre}:fill{b_fill} txg={} instance={} no_A_variant_at_or_after", b_txg.0, b_instance.0);
                continue;
            };
            let mut b = AtkSnap::of(&b_probe).pool();
            let reached = atk_advance_to_txg(&mut b, refused_on.checkpoint_txg);
            let pair = *b.session().current.root();
            let same_pair = pair.instance == refused_on.instance && pair.checkpoint_txg == refused_on.checkpoint_txg;
            let a_ask = atk_ask(&a_after.pool(), stuck);
            let b_ask = atk_ask(&b, stuck);
            let a_ways = match &a_ask.result { Ok(w) => w.ways_out.clone(), Err(_) => Vec::new() };
            let b_ways = match &b_ask.result { Ok(w) => w.ways_out.clone(), Err(_) => Vec::new() };
            // B 自己报：B 发同一个改动（A 的那一份内容）。
            let mut b_own = AtkSnap::of(&b).pool();
            let b_outcome = atk_apply(&mut b_own, change);
            let b_own_ways = match atk_stuck_of(&b_outcome) {
                Some(b_stuck) => match atk_ask(&b_own, b_stuck).result { Ok(w) => w.ways_out, Err(_) => Vec::new() },
                None => vec![format!("B 自己发：{}", atk_kind(&b_outcome))],
            };
            let mut b_push = AtkSnap::of(&b).pool();
            let b_real = match push_the_root_ring_of(&mut b_push) {
                Ok(done) => format!("pushed({}) ring_slots_written={:?}", done.versions_published.len(), done.ring_slots_written.iter().map(|s| format!("{}.{}", s.region, s.slot)).collect::<Vec<_>>()),
                Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
            };
            println!(
                "ATK k13 B=pre{b_pre}:fill{b_fill} A_j={j} reached={reached} same_pair={same_pair} pair={}.{} ask_on_B={} {} | A_own={:?} on_B_with_A_report={:?} B_own={:?} B_real_push={b_real}",
                pair.instance.0, pair.checkpoint_txg.0, b_ask.head(), b_ask.io(),
                atk_way_two_numbers(&a_ways), atk_way_two_numbers(&b_ways), atk_way_two_numbers(&b_own_ways)
            );
            println!("ATK k13   way_one: A_own_eq_on_B={} on_B={}", a_ways.first() == b_ways.first(), b_ways.first().map(|w| w.chars().take(90).collect::<String>()).unwrap_or_default());
            println!("ATK k13   way_one: B_own={}", b_own_ways.first().map(|w| w.chars().take(90).collect::<String>()).unwrap_or_default());
        }
    }
}
