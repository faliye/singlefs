
/// 攻方探针（m3-batch21-code-r1 攻方腿）：覆盖写 K 次、卸载落下 c 个写就崩、两块盘都在再可写挂载；
/// 一、挂载做成时：暖机几次、写行与暖机写进「卸载之前现行那一版换下的槽」几个（新用例那一判），
///     写进「卸载崩溃镜像上某块盘最新有效根引用、而现行那一版不再引用的槽」几个（按每块盘只读恢复那条根走树取槽，另一条路）；
/// 二、再挂落下 r 个写又崩、丢一块盘：卸载之前那块盘单独读得出的，剩下那块盘只读挂载整读读不出或读出的内容不是任何一版发过的，记坏态。
#[test]
#[ignore = "攻方探针，只在副本里点名跑"]
fn probe_unmount_crash_points_and_overwrite_counts() {
    let overwrite_counts: Vec<usize> = std::env::var("N5_K")
        .unwrap_or_else(|_| "0,1,2,3".to_string())
        .split(',')
        .map(|one| one.parse().expect("K"))
        .collect();
    let remount_crashes = std::env::var("N5_REMOUNT_CRASHES").map(|v| v == "1").unwrap_or(true);
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    for k in overwrite_counts {
        let mut pool =
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                device_width,
                176,
            );
        let mut refused = None;
        for overwrite_index in 0..k {
            if let Err(refusal) = pool.overwrite(OVERWRITE_BYTES) {
                refused = Some(format!("{overwrite_index}:{refusal:?}"));
                break;
            }
        }
        if let Some(refusal) = refused {
            println!("N5_K k={k} refused={refusal}");
            continue;
        }
        let current_txg = pool.session().current.root().checkpoint_txg.0;
        let released_before_the_unmount = slots_released_by_the_current_version(&pool);
        // 卸载一共几个写：不注错地数一遍。
        let unmount_writes_total = {
            let image = pool.image();
            let session = pool.session.as_ref().expect("会话");
            let plan = plan_failing_every_write_from(device_width, u64::MAX);
            let mut devices: Vec<_> = plain_devices_on(&image)
                .into_iter()
                .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
                .collect();
            let mut allocator = session.allocator.clone();
            let mut current = session.current.clone();
            unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger)
                .expect("不注错的卸载做成");
            plan.calls().writes
        };
        println!(
            "N5_K k={k} current_txg={current_txg} current_txg_mod3={} unmount_writes_total={unmount_writes_total} released_by_current={}",
            current_txg % 3,
            released_before_the_unmount.len()
        );
        for c in 1..unmount_writes_total {
            let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, c);
            // 每块盘单独恢复择到的根 txg（卸载崩溃镜像上）。
            let newest_on_each: Vec<Option<u64>> = [DeviceIdentity(1), DeviceIdentity(0)]
                .iter()
                .map(|lost| {
                    recover(&image_without(&after_the_unmount, *lost), JournalPolicy::Consult)
                        .effective_root
                        .map(|(_, txg)| txg.0)
                })
                .collect();
            let write_plan = plan_failing_every_write_from(device_width, u64::MAX);
            let mut remount_devices: Vec<_> = plain_devices_on(&after_the_unmount)
                .into_iter()
                .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, write_plan.clone())))
                .collect();
            let mounted = mount_writable_through(
                RemountEntry::WithDeletionBatchesOfOne,
                &pool,
                &mut remount_devices,
            );
            let mounted = match mounted {
                Ok(mounted) => mounted,
                Err(error) => {
                    println!("N5_STATE k={k} c={c} remount=refused:{error:?}");
                    continue;
                }
            };
            let remount_writes_total = write_plan.calls().writes;
            let taken_into_the_released: usize = std::iter::once(&mounted.output.row_publish)
                .chain(&mounted.output.warm_up_publishes)
                .filter_map(|version| version.version_of_trees())
                .flat_map(|version| {
                    version
                        .units
                        .iter()
                        .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
                        .collect::<Vec<_>>()
                })
                .filter(|slot| released_before_the_unmount.contains(slot))
                .count();
            let warm_ups = mounted.output.warm_up_publishes.len();
            drop(mounted);
            let mut bad_states = 0usize;
            let mut states = 0usize;
            let mut unreadable_before = 0usize;
            let mut first_bad = String::new();
            if remount_crashes {
                for r in 0..remount_writes_total {
                    for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
                        let survivor = lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
                            &pool,
                            device_width,
                            c,
                            RemountEntry::WithDeletionBatchesOfOne,
                            RootRingDuringTheRemount::ReadAsUsual,
                            r,
                            lost,
                        );
                        states += 1;
                        if survivor.before_the_unmount.is_err() {
                            unreadable_before += 1;
                            continue;
                        }
                        let bad = match &survivor.after_the_crashes {
                            Err(_) => true,
                            Ok(read) => !read.live_files.iter().all(|(_, bytes)| {
                                pool.content_of_each_root.values().any(|content| content == bytes)
                            }),
                        };
                        if bad {
                            bad_states += 1;
                            if first_bad.is_empty() {
                                first_bad = format!("r={r} lost={} after={:?}", lost.0, survivor.after_the_crashes.as_ref().map(|read| read.root_txg));
                            }
                        }
                    }
                }
            }
            println!(
                "N5_STATE k={k} c={c} newest_dev0={:?} newest_dev1={:?} warm_ups={warm_ups} remount_writes={remount_writes_total} taken_into_released_by_current={taken_into_the_released} remount_crash_states={states} unreadable_before={unreadable_before} bad_states={bad_states} first_bad={first_bad}",
                newest_on_each[0], newest_on_each[1]
            );
        }
    }
}
