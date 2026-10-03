
// ─── m3-code-r2 攻方原型（Opus）：Y3 别的入口上「根在别的盘」「根环转过一圈」；Y2 别的入口上的重读一次 ───
// 这一段接在 entries 用例文件的一份拷贝后面（复用它的造法与 go_through），只在攻方副本里编、跑；只打印，不断言。

fn r2_behind_devices(refusal: &Option<EntryRefusal>) -> Option<&Vec<DeviceBehindTheCurrentVersion>> {
    match refusal {
        Some(EntryRefusal::Session(
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            },
        ))) => Some(devices),
        _ => None,
    }
}

fn r2_short(text: String) -> String {
    text.chars().take(200).collect()
}

fn r2_label(refusal: &Option<EntryRefusal>) -> String {
    if refusal.is_none() {
        return "ok".to_string();
    }
    if let Some(devices) = r2_behind_devices(refusal) {
        return devices
            .iter()
            .map(|behind| match &behind.lacking {
                VersionLackingOnADeviceBehindIt::UnitsMissing(units) => {
                    format!("behind-units({:?},{})", behind.device, units.len())
                }
                VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(root) => format!(
                    "behind-root({:?},{:?},{:?})",
                    behind.device, root.first_reading, root.reread
                ),
            })
            .collect::<Vec<_>>()
            .join("+");
    }
    format!("other:{}", r2_short(describe(refusal)))
}

fn r2_go_through<D: BlockDevice>(
    entry: EntryAfterTheMount,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, D)>,
) -> Option<EntryRefusal> {
    let parameters = pool.parameters.clone();
    let (content, write_time_seconds) = overwrite_after(pool, 0);
    let session = pool.session.as_mut().expect("这一刻有可写会话");
    match entry {
        EntryAfterTheMount::SessionPublish => session
            .publish_user_change(
                devices,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            )
            .err()
            .map(EntryRefusal::Session),
        EntryAfterTheMount::NormalUnmount => unmount(
            &parameters,
            devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        )
        .err()
        .map(EntryRefusal::Mount),
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling => {
            let MountedSession {
                allocator,
                current,
                shadow_ledger,
                ..
            } = session;
            raise_rollback_floor_to_the_admission_ceiling(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                *shadow_ledger,
            )
            .err()
            .map(EntryRefusal::Mount)
        }
        EntryAfterTheMount::AdministratorRollback => {
            let MountedSession {
                allocator, current, ..
            } = session;
            // 回退目标是用户定的：默认第一个文件那一版（txg 3）；R2_ROLLBACK_PREV=1 时取现行那一版的前一版（一直在根环里）。
            let target = if std::env::var("R2_ROLLBACK_PREV").is_ok() {
                let current_root = file_version_of(current).root;
                RollbackTarget {
                    instance: current_root.instance,
                    checkpoint_txg: CheckpointTxg(current_root.checkpoint_txg.0 - 1),
                }
            } else {
                FIRST_FILE_VERSION
            };
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                target,
            )
            .err()
            .map(EntryRefusal::Rollback)
        }
    }
}

fn r2_env_list(name: &str, default: &[u64]) -> Vec<u64> {
    match std::env::var(name) {
        Ok(text) if text.contains("..") => {
            let (low, high) = text.split_once("..").expect("a..b");
            (low.parse::<u64>().expect("数")..=high.parse::<u64>().expect("数")).collect()
        }
        Ok(text) => text
            .split(',')
            .map(|item| item.trim().parse::<u64>().expect("数"))
            .collect(),
        Err(_) => default.to_vec(),
    }
}

fn r2_env_entries() -> Vec<EntryAfterTheMount> {
    match std::env::var("R2_ENTRIES") {
        Ok(text) => text
            .split(',')
            .map(|name| match name.trim() {
                "publish" => EntryAfterTheMount::SessionPublish,
                "unmount" => EntryAfterTheMount::NormalUnmount,
                "raise" => EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
                "rollback" => EntryAfterTheMount::AdministratorRollback,
                other => panic!("不认得入口 {other}"),
            })
            .collect(),
        Err(_) => EVERY_ENTRY.to_vec(),
    }
}

fn r2_image_of_sparse(devices: &SparseDevices, device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

/// 起完入口之后（不论拒没拒）拿这组盘的拷贝可写挂载一次：交回结局与挂上之后池级 checker 的违例条数。
fn r2_remount(pool: &PoolUnderTest<SparseBlockDevice>, devices: &SparseDevices) -> (String, usize) {
    let mut copies = copies_of(devices);
    let outcome = match mount_writable(&pool.parameters, &mut copies) {
        Ok(mounted) => format!(
            "Ok({:?},{:?})",
            mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg
        ),
        Err(error) => format!("Err:{}", r2_short(format!("{error:?}"))),
    };
    let violations = checker_violations_on(&r2_image_of_sparse(&copies, pool.device_bytes)).len();
    (outcome, violations)
}

/// Y3：P 的 txg 扫 `R2_TXGS`（默认 6..=32：区域 0/1/2、盘 0/1、根环转过一圈），切在根槽 FUA 之前 / 之后，四个入口各走一次。
/// 每格打印入口的结局、盘变没变、入口之后池级 checker、之后再可写挂载一次的结局与 checker。FLAG：切在根槽之前却没拒成缺根那一半、
/// 或盘变了；切在根槽之后却被拒、或 checker 红；再挂之后 checker 红。
#[test]
fn r2opus_y3_entries_sweep_root_elsewhere_and_lap() {
    let txgs = r2_env_list("R2_TXGS", &(6..=32).collect::<Vec<u64>>());
    let entries = r2_env_entries();
    let mut cases = 0u64;
    let mut flags = 0u64;
    for txg in txgs {
        let cuts: Vec<WhereThePublishIsCut> = match std::env::var("R2_CUTS").as_deref() {
            Ok("before") => vec![WhereThePublishIsCut::BeforeItsRootSlotWrite],
            Ok("after") => vec![WhereThePublishIsCut::AfterItsRootSlotWrite],
            _ => vec![
                WhereThePublishIsCut::BeforeItsRootSlotWrite,
                WhereThePublishIsCut::AfterItsRootSlotWrite,
            ],
        };
        for cut in cuts {
            for &entry in &entries {
                let started = std::time::Instant::now();
                let RootDeviceOfThePublishTakenMidPublish {
                    mut pool,
                    mut devices,
                    root_device_of_the_publish,
                    ring_slot_of_the_publish_root,
                } = root_device_of_the_publish_taken_mid_publish(cut, CheckpointTxg(txg));
                let images_before = images_of(&devices);
                let refusal = r2_go_through(entry, &mut pool, &mut devices);
                let label = r2_label(&refusal);
                let unchanged = images_of(&devices) == images_before;
                let violations = checker_violations_on(&r2_image_of_sparse(&devices, pool.device_bytes));
                if !violations.is_empty() {
                    println!("Y3-VIOLATIONS txg={txg} entry={entry:?} {:?}", violations.iter().take(3).map(|v| r2_short(v.clone())).collect::<Vec<_>>());
                }
                let checker = violations.len();
                let (remount, checker_after_remount) = r2_remount(&pool, &devices);
                let flag = match cut {
                    WhereThePublishIsCut::BeforeItsRootSlotWrite => {
                        !label.starts_with("behind-root") || !unchanged
                    }
                    WhereThePublishIsCut::AfterItsRootSlotWrite => {
                        refusal.is_some() || checker != 0
                    }
                } || checker_after_remount != 0;
                cases += 1;
                if flag {
                    flags += 1;
                }
                println!(
                    "Y3 txg={txg} region={} slot={} root_dev={:?} cut={cut:?} entry={entry:?} outcome={label} unchanged={unchanged} checker={checker} remount={remount} checker_after_remount={checker_after_remount} ms={}{}",
                    ring_slot_of_the_publish_root.region,
                    ring_slot_of_the_publish_root.slot,
                    root_device_of_the_publish,
                    started.elapsed().as_millis(),
                    if flag { " FLAG" } else { "" }
                );
            }
        }
    }
    println!("Y3-SUMMARY cases={cases} flags={flags}");
}

/// Y2：P 的根住的那块盘切在根槽 FUA 之后（落后、单元都在、根槽里是 P），经四个入口，P 的根槽上第 n 次读坏一次（`once`）
/// 或从第 n 次起每次都坏（`from`），n 扫 1..=`R2_MAX_N`。打印每格的结局与真读坏的次数；n 超过那一槽被读的次数时（一次都没坏）停。
#[test]
fn r2opus_y2_reread_sweep_over_every_entry() {
    let txgs = r2_env_list("R2_REREAD_TXGS", &[6, 7, 8, 27]);
    let max_n = r2_env_list("R2_MAX_N", &[12])[0];
    let entries = r2_env_entries();
    for txg in txgs {
        for &entry in &entries {
            for two_or_more in [false, true] {
                for n in 1..=max_n {
                    let RootDeviceOfThePublishTakenMidPublish {
                        mut pool,
                        devices,
                        ring_slot_of_the_publish_root,
                        ..
                    } = root_device_of_the_publish_taken_mid_publish(
                        WhereThePublishIsCut::AfterItsRootSlotWrite,
                        CheckpointTxg(txg),
                    );
                    let occurrence = if two_or_more {
                        FaultOccurrence::EveryMatchingCallFromTheNthOnward(n)
                    } else {
                        FaultOccurrence::TheNthMatchingCall(n)
                    };
                    let (mut wrapped, plan) = with_reads_of_the_publish_root_slot_failing(
                        devices,
                        &pool,
                        ring_slot_of_the_publish_root,
                        occurrence,
                    );
                    let refusal = r2_go_through(entry, &mut pool, &mut wrapped);
                    let fired = plan.fired_count();
                    let label = r2_label(&refusal);
                    let after = MemoryPool {
                        devices: wrapped
                            .iter()
                            .map(|(identity, device)| {
                                (*identity, device.wrapped_device().image.clone())
                            })
                            .collect(),
                        device_size_in_bytes: pool.device_bytes,
                    };
                    let checker = checker_violations_on(&after).len();
                    println!(
                        "Y2R txg={txg} entry={entry:?} occ={} n={n} fired={fired} writes={} outcome={label} checker={checker}",
                        if two_or_more { "from" } else { "once" },
                        plan.calls().writes,
                    );
                    if fired == 0 {
                        break;
                    }
                }
            }
        }
    }
}

/// Y2：P 的根槽恰好连着坏两次（第 n、n+1 次），之后读得出：两层注入盘，里层坏第 n 次、外层坏第 n+1 次（外层拦下的那一次不进里层）。
/// 原样代码：缺根那一半读一次、重读一次都坏就拒；读三次的错实现在第三次读到 P、放过。n 扫 1..=`R2_MAX_N`。
#[test]
fn r2opus_y2_exactly_two_failures_in_a_row() {
    let txgs = r2_env_list("R2_REREAD_TXGS", &[6, 7, 8, 27]);
    let max_n = r2_env_list("R2_MAX_N", &[12])[0];
    let entries = r2_env_entries();
    for txg in txgs {
        for &entry in &entries {
            for n in 1..=max_n {
                let RootDeviceOfThePublishTakenMidPublish {
                    mut pool,
                    devices,
                    ring_slot_of_the_publish_root,
                    ..
                } = root_device_of_the_publish_taken_mid_publish(
                    WhereThePublishIsCut::AfterItsRootSlotWrite,
                    CheckpointTxg(txg),
                );
                let (inner, inner_plan) = with_reads_of_the_publish_root_slot_failing(
                    devices,
                    &pool,
                    ring_slot_of_the_publish_root,
                    FaultOccurrence::TheNthMatchingCall(n),
                );
                let outer_plan = SharedFaultPlan::armed(
                    DEVICE_WIDTH.fixed_geometry(),
                    FaultSchedule {
                        fault: InjectedFault::ReadFails,
                        device: FaultDeviceSelector::EveryDevice,
                        placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
                            named_slots: NamedRootRingSlots::NONE.with(ring_slot_of_the_publish_root),
                            region_devices: pool.parameters.region_devices,
                            fixed_structure_slot_spacing: pool
                                .parameters
                                .geometry
                                .fixed_structure_slot_spacing,
                        }),
                        counting: FaultCounting::AcrossThePool,
                        occurrence: FaultOccurrence::TheNthMatchingCall(n + 1),
                    },
                );
                let mut wrapped: Vec<(
                    DeviceIdentity,
                    FaultInjectingBlockDevice<FaultInjectingBlockDevice<SparseBlockDevice>>,
                )> = inner
                    .into_iter()
                    .map(|(identity, device)| {
                        (
                            identity,
                            FaultInjectingBlockDevice::new(identity, device, outer_plan.clone()),
                        )
                    })
                    .collect();
                let refusal = r2_go_through(entry, &mut pool, &mut wrapped);
                let fired = inner_plan.fired_count() + outer_plan.fired_count();
                println!(
                    "Y2T txg={txg} entry={entry:?} n={n} fired={fired} writes={} outcome={}",
                    outer_plan.calls().writes,
                    r2_label(&refusal)
                );
                if fired < 2 {
                    break;
                }
            }
        }
    }
}
