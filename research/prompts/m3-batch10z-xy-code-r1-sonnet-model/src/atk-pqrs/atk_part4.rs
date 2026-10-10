
// ---------- K3-1：被抛弃根那一圈只隔离分配位没置着的槽，回收时补隔离有没有漏路 ----------

/// 一步用户动作（字母表 o 覆盖写、s 长两个单元、n 建 inode、f 抬 F 到上限（上限不高于 F 时 F 不变）、u 正常卸载再可写挂载）。
fn atk_k3_step(
    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    fault: &ReadFaultOnChosenSlots,
    fault_after_mount: bool,
    action: char,
) -> String {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    match action {
        'o' => atk_outcome(&pool.overwrite(OVERWRITE_BYTES)),
        's' => {
            let length = pool.content_of_the_current_version.len().div_ceil(unit_bytes).max(1) + 2;
            atk_outcome(&pool.sequential_write(length * unit_bytes))
        }
        'n' => atk_outcome(&pool.create_one_inode()),
        'f' => {
            let ceiling = {
                let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置");
                let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
                    .expect("实例表")
                    .records;
                rollback_floor_ceiling(
                    &pool.devices,
                    &system_configuration,
                    pool.rollback_floor(),
                    &table,
                    &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&pool.session().allocator),
                )
            };
            let new_floor = match ceiling {
                Ok(ceiling) => ceiling.max(pool.rollback_floor()),
                Err(_) => pool.rollback_floor(),
            };
            let mut session = pool.session.take().expect("会话");
            let raised = {
                let PoolVersion::WithFile(current) = &mut session.current else {
                    pool.session = Some(session);
                    return "f_no_file".to_string();
                };
                raise_rollback_floor(&pool.parameters, &mut pool.devices, &mut session.allocator, current, new_floor, ShadowLedger::On)
            };
            pool.session = Some(session);
            match raised {
                Ok(raised) => {
                    for version in &raised.publishes {
                        pool.note_root_of_the_current_content(&version.root);
                    }
                    format!("f_ok(F={})", new_floor.0)
                }
                Err(refusal) => format!("f_refused({})", format!("{refusal:?}").chars().take(50).collect::<String>()),
            }
        }
        'u' => {
            let mut session = pool.session.take().expect("会话");
            let unmounted = unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
            let text = match unmounted {
                Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                    for version in &raised.publishes {
                        pool.note_root_of_the_current_content(&version.root);
                    }
                    "u_ok".to_string()
                }
                Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "u_nothing".to_string(),
                Err(refusal) => {
                    pool.session = Some(session);
                    return format!("u_refused({})", format!("{refusal:?}").chars().take(50).collect::<String>());
                }
            };
            fault.is_active.set(false);
            let remount = pool.crash_and_mount_writable();
            fault.is_active.set(fault_after_mount);
            match remount {
                Ok(_) => text,
                Err(refusal) => format!("{text}_remount_refused({})", format!("{refusal:?}").chars().take(50).collect::<String>()),
            }
        }
        other => panic!("没有这个动作 {other}"),
    }
}

/// K3-1 扫：崩溃恢复抛弃了 A 的那份盘面（两种：原样、或先在实例 4 里把 F 抬到 A 的 txg），可写挂载之后故障三种（不开、A 的根槽读报错、
/// A 的账读报错），随机动作串（种子放开）；每一步之后：A 还在根环里时它挂载那一刻记着的落点一槽都不空闲、池级 checker 不红 I-7.4 / I-4.8、
/// 没有一个槽同时算进已分配与第九项。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k3_sweep() {
    let seeds: u64 = std::env::var("ATK_SEEDS").map_or(20, |text| text.parse().expect("数"));
    let seed_base: u64 = std::env::var("ATK_SEED_BASE").map_or(0, |text| text.parse().expect("数"));
    let length: usize = std::env::var("ATK_LEN").map_or(16, |text| text.parse().expect("数"));
    let alphabet: Vec<char> = std::env::var("ATK_ALPHABET").unwrap_or_else(|_| "oosnfu".to_string()).chars().collect();
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let key_of_a = (built.abandoned_root.instance, built.abandoned_root.checkpoint_txg);
    let images = [
        ("plain", built.image.clone()),
        ("floor_at_a", image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built)),
    ];
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    for (image_name, image) in &images {
        for fault_name in ["none", "root_slot", "account"] {
            for seed in seed_base..seed_base + seeds {
                let fault = match fault_name {
                    "root_slot" => ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters),
                    "account" => ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root),
                    _ => ReadFaultOnChosenSlots { ranges: Rc::new(Vec::new()), is_active: Rc::new(Cell::new(false)) },
                };
                let mut devices = devices_with_the_read_fault_on(image, &fault);
                let Ok(mounted) = mount_writable_with_space_admission(&built.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) else {
                    *totals.entry(format!("{image_name}/{fault_name} mount_refused")).or_insert(0) += 1;
                    continue;
                };
                let mut recorded_for_a = mounted
                    .allocator
                    .root_ring_occupancy()
                    .and_then(|occupancy| occupancy.placements_recorded_for_the_abandoned_root(key_of_a.0, key_of_a.1))
                    .unwrap_or_default();
                let mut pool = pool_under_test_after_the_mount(&built.parameters, built.device_bytes, devices, mounted, built.write_time_seconds + 3000);
                fault.is_active.set(fault_name != "none");
                // 简单的线性同余，种子定动作串。
                let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let mut actions = String::new();
                let mut problems = Vec::new();
                for step in 0..length {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let action = alphabet[usize::try_from((state >> 33) % u64::try_from(alphabet.len()).expect("数")).expect("数")];
                    actions.push(action);
                    let outcome = atk_k3_step(&mut pool, &fault, fault_name != "none", action);
                    if outcome.starts_with("u_ok") && !outcome.contains("remount_refused") {
                        recorded_for_a = pool
                            .session()
                            .allocator
                            .root_ring_occupancy()
                            .and_then(|occupancy| occupancy.placements_recorded_for_the_abandoned_root(key_of_a.0, key_of_a.1))
                            .unwrap_or(recorded_for_a);
                    }
                    if pool.session.is_none() {
                        problems.push(format!("step{step}:{action}:no_session:{outcome}"));
                        break;
                    }
                    let image_now = pool.image();
                    let a_in_ring = root_is_in_the_ring(&image_now, key_of_a);
                    let allocator = &pool.session().allocator;
                    let free_of_a = if a_in_ring { free_slots_covered_by(allocator, &recorded_for_a) } else { Vec::new() };
                    let double = slots_counted_both_as_allocated_and_as_isolated(allocator);
                    let violations = violations_of_unit_reuse_on(&image_now);
                    if !free_of_a.is_empty() {
                        problems.push(format!("step{step}:{action}:LEAK free_of_a={:?} outcome={outcome}", &free_of_a[..free_of_a.len().min(4)]));
                    }
                    if double > 0 {
                        problems.push(format!("step{step}:{action}:double={double}"));
                    }
                    if let Some(first) = violations.first() {
                        problems.push(format!("step{step}:{action}:VIOL {}", first.chars().take(160).collect::<String>()));
                    }
                    if outcome.contains("refused") && !outcome.starts_with("nospace") {
                        *totals.entry(format!("{image_name}/{fault_name} refused_{}", outcome.chars().take(24).collect::<String>())).or_insert(0) += 1;
                    }
                }
                let key = format!("{image_name}/{fault_name} {}", if problems.is_empty() { "clean" } else { "PROBLEM" });
                *totals.entry(key).or_insert(0) += 1;
                if !problems.is_empty() {
                    println!("ATK k3 {image_name}/{fault_name} seed={seed} actions={actions} problems={:?}", &problems[..problems.len().min(3)]);
                }
            }
        }
    }
    println!("ATK k3 totals={totals:?}");
}
