
/// K3（攻方变异 K 那一支：预演里推环自己被拒）：被拒之后用户接着建 inode / 覆盖写直到它们也被拒（至多 `ATK_MAX_SMALL` 次），
/// 每一步之后再发那次长写，报「被宽护卡住」时预演是哪一种（找自然历史里预演推环被拒的那一形），照它走一遍。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_fill_up_with_small_actions_then_report() {
    let area: u64 = std::env::var("ATK_AREA").map_or(384, |text| text.parse().expect("数"));
    let max_small: usize = std::env::var("ATK_MAX_SMALL").map_or(120, |text| text.parse().expect("数"));
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut disagree = 0usize;
    for (pre, fill, grow) in atk_cells_from_env() {
        let Some((_base, crashed, contents, write_time_seconds)) = atk_k1_record_on(area, pre, fill, 5) else { continue };
        let (mounted, _) = atk_mounted_pool_from(&parameters_of_the_384_slot_small_devices(), atk_device_bytes_with_unit_area(area), &crashed, &contents, write_time_seconds).expect("挂");
        let size_now = mounted.content_of_the_current_version.len().div_ceil(unit_bytes);
        for action in ["inode", "overwrite"] {
            let mut pool = AtkSnap::of(&mounted).pool();
            let mut line = Vec::new();
            for step in 0..=max_small {
                if step > 0 {
                    let small = if action == "inode" { pool.create_one_inode() } else { pool.overwrite(OVERWRITE_BYTES) };
                    if small.is_err() {
                        line.push(format!("s{step}:small_refused({})", atk_kind(&small).chars().take(30).collect::<String>()));
                        break;
                    }
                }
                let snap = AtkSnap::of(&pool);
                let mut trial = snap.pool();
                let write = atk_write_of(&mut trial, size_now + grow);
                let outcome = atk_apply(&mut trial, &write);
                let head = match atk_stuck_of(&outcome) {
                    Some(stuck) => {
                        let head = atk_rehearsal_head(stuck);
                        let (_, ok) = atk_follow_way_two(&AtkSnap::of(&trial), stuck, &write);
                        if !ok {
                            disagree += 1;
                        }
                        format!("{head}{}", if ok { "" } else { "!DISAGREE" })
                    }
                    None => atk_kind(&outcome).chars().take(30).collect(),
                };
                *kinds.entry(head.clone()).or_insert(0) += 1;
                if step % 10 == 0 || head.starts_with("pushref") || head.contains("DISAGREE") {
                    line.push(format!("s{step}:{head}"));
                }
            }
            println!("ATK k1fill area={area} pre={pre} fill={fill} g={grow} action={action} {}", line.join(" "));
        }
    }
    println!("ATK k1fill totals area={area} kinds={kinds:?} disagree={disagree}");
}
