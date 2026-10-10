
// ---------- K2：两道判的先后（N1 变异把盘表那一道挪到版本那一道之前）在哪一形上看得出来 ----------

/// K1 主形（pre=4 fill=16）每个报卡住的格：被拒之后会话又动了（覆盖写一次发成，或照挂载时那一份盘表推环），再拿旧报、交次序对调的盘表要出路。
/// 文档（`WaysOutOfTheWideProtectionRefused` 两个成员）：先比版本、会话已动就拒成 SessionMoved；盘表那一种只在「会话还在被拒的那一版上」时报。
/// 打印这一次交回的是哪一种、数调用（一个盘都不读）；再换回挂载时那一份盘表要一次，看交回的种类。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k2_moved_session_and_reversed_table_which_refusal() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, 4, 16, 5).expect("录得下");
    let (probe, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
    let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
    let mounted = AtkSnap::of(&probe);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for grow in 13..=44usize {
        let mut pool = mounted.pool();
        let change = atk_seq_of(&mut pool, size_now + grow);
        let outcome = atk_apply(&mut pool, &change);
        let Some(stuck) = atk_take_stuck(outcome) else { continue };
        for moved_by in ["overwrite", "push"] {
            let mut moved = AtkSnap::of(&pool).pool();
            let note = if moved_by == "overwrite" {
                atk_kind(&moved.overwrite(OVERWRITE_BYTES))
            } else {
                atk_push_with_table(&mut moved, "mount")
            };
            let snap = AtkSnap::of(&moved);
            let reversed = atk_ask_with_table(&snap, None, "reversed", &stuck).expect("造得出");
            let restored = atk_ask(&snap.pool(), &stuck);
            let key = format!("{moved_by}:{note}: reversed={} restored={}", atk_head(reversed.head()), atk_head(restored.head()));
            *tally.entry(key).or_insert(0) += 1;
            println!("ATK k2order g={grow} moved_by={moved_by}:{note} reversed_table_ask={} {} restored_table_ask={}", reversed.head(), reversed.io(), restored.head());
        }
    }
    println!("ATK k2order totals {tally:?}");
}
