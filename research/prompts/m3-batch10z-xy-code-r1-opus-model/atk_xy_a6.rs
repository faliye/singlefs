#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_ways_out_vs_real_sweep() {
    let pres: Vec<u64> = atk_env_list("ATK_PRES", "2,4,8");
    let fills: Vec<usize> = atk_env_list("ATK_FILLS", "6,11,16,21");
    let extra: usize = atk_env_or("ATK_EXTRA", 8);
    let actions: Vec<String> = std::env::var("ATK_ACTIONS").unwrap_or_else(|_| "none".to_string()).split(',').map(str::to_string).collect();
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let area: u64 = atk_env_or("ATK_AREA", 384);
    let device_bytes = atk_device_bytes_with_unit_area(area);
    let (mut cells, mut stuck_cells) = (0usize, 0usize);
    let mut tally: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for pre in &pres {
        for fill in &fills {
            let label = format!("pre={pre} fill={fill}");
            let Some((base, crashed, contents, write_time_seconds)) = atk_k1_record_on(area, *pre, *fill, 5) else {
                println!("ATK k1 {label} r_not_recorded");
                continue;
            };
            let first_refused_of = |image: &MemoryPool| -> (usize, Option<usize>) {
                let (probe, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
                for grow in 1..=90usize {
                    let (mut trial, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                    if trial.sequential_write((size_now + grow) * unit_bytes).is_err() {
                        return (size_now, Some(grow));
                    }
                }
                (size_now, None)
            };
            let (size_now, g_ctrl) = first_refused_of(&base);
            let (_, g_c) = first_refused_of(&crashed);
            println!("ATK k1 {label} size_now={size_now} g_ctrl={g_ctrl:?} g_c={g_c:?}");
            let (Some(g_ctrl), Some(g_c)) = (g_ctrl, g_c) else { continue };
            let (mounted, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
            let mounted = AtkSnap::of(&mounted);
            for grow in g_c..=(g_ctrl + extra) {
                cells += 1;
                let mut pool = mounted.pool();
                let change = atk_seq_of(&mut pool, size_now + grow);
                let outcome = atk_apply(&mut pool, &change);
                let Some(stuck) = atk_stuck_of(&outcome) else {
                    println!("ATK k1 {label} g={grow} first={}", atk_kind(&outcome));
                    continue;
                };
                stuck_cells += 1;
                let after_report = AtkSnap::of(&pool);
                for action in &actions {
                    let (line, ok) = atk_after_action_with_older(&after_report, Some(&mounted.image), stuck, &change, action);
                    let entry = tally.entry(action.clone()).or_insert((0, 0, 0));
                    match ok {
                        Some(true) => entry.0 += 1,
                        Some(false) => entry.1 += 1,
                        None => entry.2 += 1,
                    }
                    println!("ATK k1 {label} g={grow} n_pub={} {line}", stuck.empty_publishes_until_the_ring_slot_of_r_is_written);
                }
            }
        }
    }
    for (action, (agree, mismatch, moved)) in &tally {
        println!("ATK k1 totals area={area} action={action} cells={cells} stuck={stuck_cells} agree={agree} mismatch={mismatch} moved_or_unasked={moved}");
    }
}
