
// ---------- K1-3（本轮正推腿加）：别的会话（同一对实例代号与 txg）的报，列出路二时文字里的数是谁的 ----------

/// 一格 (pre, fill) 的池：录 R、崩、可写挂载。交回（崩了的盘面、各根内容、之后的写入时刻、挂上的池）。
fn sonnet_foreign_cell(pre: u64, fill: usize) -> Option<(MemoryPool, BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>, u64)> {
    let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, pre, fill, 5)?;
    Some((crashed, contents, write_time_seconds))
}

fn sonnet_num_after(text: &str, before: &str, after: &str) -> Option<u64> {
    let start = text.find(before)? + before.len();
    let rest = &text[start..];
    rest[..rest.find(after)?].trim().parse().ok()
}

/// 报来自池 A（格 (pre_a, fill_a)，长 grow 个单元被拒），拿去问池 B（别的格，覆盖写推到 A 被拒那一版的 txg）：
/// 同一对 (实例代号, txg) 时 B 照自己的状态预演；比文字里的「发 N 次」「根环槽」是报里的还是 B 自己的，预演类别对 B 真走的结局。
#[test]
#[ignore = "正推腿 K1-3，手动跑"]
fn sonnet_k13_foreign_report_matrix() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let cells: Vec<(u64, usize)> = vec![(2, 6), (2, 11), (2, 16), (4, 6), (4, 11), (4, 16), (8, 6), (8, 11), (8, 16)];
    let (mut pairs, mut same_pairs, mut listed_two) = (0, 0, 0);
    for &(pre_a, fill_a) in &cells {
        let Some((crashed_a, contents_a, time_a)) = sonnet_foreign_cell(pre_a, fill_a) else { continue };
        let (mounted_a, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed_a, &contents_a, time_a).expect("挂");
        let size_now = mounted_a.content_of_the_current_version.len().div_ceil(unit_bytes);
        let snap_a = AtkSnap::of(&mounted_a);
        for grow in 8..=48usize {
            let mut pool_a = snap_a.pool();
            let change = atk_seq_of(&mut pool_a, size_now + grow);
            let outcome = atk_apply(&mut pool_a, &change);
            let Some(stuck) = atk_stuck_of(&outcome) else { continue };
            let refused_on = *pool_a.session().current.root();
            let own = atk_ask(&pool_a, stuck);
            let own_two = match &own.result { Ok(w) if w.ways_out.len() == 2 => Some(w.ways_out[1].clone()), _ => None };
            let own_one = match &own.result { Ok(w) => Some(w.ways_out[0].clone()), _ => None };
            let r_slot_text = format!("根环槽 {}.{}", stuck.ring_slot_of_r.region, stuck.ring_slot_of_r.slot);
            for &(pre_b, fill_b) in &cells {
                if (pre_b, fill_b) == (pre_a, fill_a) { continue; }
                let Some((crashed_b, contents_b, time_b)) = sonnet_foreign_cell(pre_b, fill_b) else { continue };
                let Ok((mut pool_b, _)) = atk_mounted_pool_from(&parameters, device_bytes, &crashed_b, &contents_b, time_b) else { continue };
                pairs += 1;
                let reached = atk_advance_to_txg(&mut pool_b, refused_on.checkpoint_txg);
                let root_b = *pool_b.session().current.root();
                if !(reached && root_b.instance == refused_on.instance && root_b.checkpoint_txg == refused_on.checkpoint_txg) { continue; }
                same_pairs += 1;
                let ask = atk_ask(&pool_b, stuck);
                let mut copy = AtkSnap::of(&pool_b).pool();
                let (cmp_line, cmp_ok) = atk_compare(&ask, &mut copy, &change);
                let way_one_same_as_the_reports_own = match &ask.result { Ok(w) => format!("{}", Some(w.ways_out[0].clone()) == own_one), _ => "-".to_string() };
                let (text_n, text_slot_is_report, text_n_is_report) = match &ask.result {
                    Ok(w) if w.ways_out.len() == 2 => {
                        listed_two += 1;
                        let t = &w.ways_out[1];
                        let n = sonnet_num_after(t, "从现行那一版起发 ", " 次");
                        (format!("{n:?}"), t.contains(&r_slot_text), n == Some(stuck.empty_publishes_until_the_ring_slot_of_r_is_written))
                    }
                    _ => ("-".to_string(), false, false),
                };
                println!(
                    "SON k13x A=({pre_a},{fill_a},g={grow}) report_n={} report_slot={r_slot_text} own_two={} | B=({pre_b},{fill_b}) pair={}.{} ask={} two_text_n={text_n} two_text_slot_is_the_reports={text_slot_is_report} two_text_n_is_the_reports={text_n_is_report} way_one_same_as_the_reports_own={way_one_same_as_the_reports_own} cmp_ok={cmp_ok:?} real: {}",
                    stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
                    own_two.is_some(),
                    root_b.instance.0, root_b.checkpoint_txg.0,
                    ask.head(),
                    cmp_line.split(" push=").nth(1).unwrap_or("-").chars().take(80).collect::<String>()
                );
            }
        }
    }
    println!("SON k13x totals pairs={pairs} same_pairs={same_pairs} listed_two={listed_two}");
}

/// 同一份崩了的盘面可写挂载两次（两条会话，各拿一份拷贝）：第一条会话报被宽护卡住，拿它的报问第二条会话。
/// 两次挂载取到同一个实例代号、同一个 txg，报在第二条会话上不拦；两条会话状态相同，结局相同。
#[test]
#[ignore = "正推腿 K1-3，手动跑"]
fn sonnet_k13_two_mounts_of_the_same_crashed_image() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    for (pre, fill) in [(2u64, 11usize), (4, 16), (8, 11)] {
        let Some((_base, crashed, contents, time)) = atk_k1_record_on(384, pre, fill, 5) else { continue };
        let (first, out1) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, time).expect("挂");
        let (second, out2) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, time).expect("挂");
        let size_now = first.content_of_the_current_version.len().div_ceil(unit_bytes);
        let (r1, r2) = (*first.session().current.root(), *second.session().current.root());
        println!("SON fork pre={pre} fill={fill} mount1 instance={} root={}.{} mount2 instance={} root={}.{}", out1.instance.0, r1.instance.0, r1.checkpoint_txg.0, out2.instance.0, r2.instance.0, r2.checkpoint_txg.0);
        let snap = AtkSnap::of(&first);
        for grow in 10..=40usize {
            let mut pool = snap.pool();
            let change = atk_seq_of(&mut pool, size_now + grow);
            let outcome = atk_apply(&mut pool, &change);
            let Some(stuck) = atk_stuck_of(&outcome) else { continue };
            let own = atk_ask(&pool, stuck);
            let mut other = AtkSnap::of(&second).pool();
            let ask = atk_ask(&other, stuck);
            let (line, ok) = atk_compare(&ask, &mut other, &change);
            println!("SON fork pre={pre} fill={fill} g={grow} own={} other={} same_ways={} cmp_ok={ok:?}", own.head(), ask.head(), format!("{:?}", own.result.as_ref().map(|w| w.ways_out.clone()).ok()) == format!("{:?}", ask.result.as_ref().map(|w| w.ways_out.clone()).ok()));
            let _ = line;
        }
    }
}

// ---------- K1-2（本轮正推腿加）：推环入口交次序对调的盘表：写落在哪块盘、之后怎么判、再挂载看到什么 ----------

fn sonnet_sector_diff(before: &MemoryPool, after: &MemoryPool) -> Vec<(u32, usize, usize)> {
    // 每块盘（按身份）：(身份, 新写的扇区数, 内容变了的扇区数)
    before
        .devices
        .iter()
        .map(|(identity, device_before)| {
            let device_after = &after.devices[identity];
            let (mut added, mut changed) = (0, 0);
            for (sector, bytes) in device_after.written_sectors() {
                match device_before.written_sectors().get(sector) {
                    None => added += 1,
                    Some(old) if old != bytes => changed += 1,
                    Some(_) => {}
                }
            }
            (identity.0, added, changed)
        })
        .collect()
}

#[test]
#[ignore = "正推腿 K1-2，手动跑"]
fn sonnet_k12_push_entry_with_the_two_devices_reversed() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let cells: Vec<(u64, usize, usize)> = vec![(4, 16, 13), (4, 16, 20), (2, 16, 17), (8, 11, 40)];
    for (pre, fill, grow) in cells {
        let (_base, crashed, contents, time) = atk_k1_record_on(384, pre, fill, 5).expect("录得下");
        let (mounted, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, time).expect("挂");
        let size_now = mounted.content_of_the_current_version.len().div_ceil(unit_bytes);
        let snap = AtkSnap::of(&mounted);
        let mut stuck_pool = snap.pool();
        let change = atk_seq_of(&mut stuck_pool, size_now + grow);
        let outcome = atk_apply(&mut stuck_pool, &change);
        if atk_stuck_of(&outcome).is_none() {
            println!("SON k12 cell=({pre},{fill},g={grow}) first={}", atk_kind(&outcome));
            continue;
        }
        let after_report = AtkSnap::of(&stuck_pool);
        let mut correct_image: Option<MemoryPool> = None;
        for variant in ["correct", "reversed"] {
            let mut pool = after_report.pool();
            let image_before = pool.image();
            let txg_before = pool.session().current.root().checkpoint_txg.0;
            let iso_before: u64 = isolated_slots_on_each_device_of(&pool.session().allocator).iter().map(|(_, n)| *n).sum();
            if variant == "reversed" {
                pool.devices.reverse();
            }
            let pushed = push_the_root_ring_of(&mut pool);
            let push_text = match &pushed {
                Ok(done) => format!("pushed({})", done.versions_published.len()),
                Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
            };
            if variant == "reversed" {
                pool.devices.reverse();
            }
            let image_after = pool.image();
            let txg_after = pool.session().current.root().checkpoint_txg.0;
            let iso_after: u64 = isolated_slots_on_each_device_of(&pool.session().allocator).iter().map(|(_, n)| *n).sum();
            let diff = sonnet_sector_diff(&image_before, &image_after);
            let same_image_as_the_correct_order_push = match &correct_image { Some(correct) => format!("{}", *correct == image_after), None => "-".to_string() };
            if variant == "correct" { correct_image = Some(image_after.clone()); }
            // 之后：表摆对了，再发被拒的那个改动
            let mut resend = AtkSnap::of(&pool).pool();
            let again = atk_kind(&atk_apply(&mut resend, &change));
            // 再挂载：从盘面（表摆对）
            let mut remount = AtkSnap::of(&pool).pool();
            let remounted = match remount.crash_and_mount_writable() {
                Ok(output) => format!(
                    "remount=ok chosen_root={}.{} effective_root={}.{} isolated={:?} abandoned_unreadable={} warm={}",
                    output.chosen_root.instance.0, output.chosen_root.checkpoint_txg.0, output.effective_root.instance.0, output.effective_root.checkpoint_txg.0,
                    output.isolated_slots_per_device.iter().map(|(_, n)| *n).collect::<Vec<_>>(), output.abandoned_roots_unreadable, output.warm_up_publishes.len()
                ),
                Err(error) => format!("remount=refused[{}]", atk_head(format!("{error:?}"))),
            };
            println!(
                "SON k12 cell=({pre},{fill},g={grow}) table={variant} push={push_text} txg {txg_before}->{txg_after} iso_sum {iso_before}->{iso_after} sector_diff(id,added,changed)={diff:?} resend_with_right_table={again} image_equals_the_correct_order_push={same_image_as_the_correct_order_push} {remounted}"
            );
        }
    }
}
