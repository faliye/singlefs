
// ─── m3-code-r3 攻方（Opus）原型：W2 两个新错实现（RU、RGC）的后果。接在 entries 用例文件原样之后，名字都带 r3opus ───

fn r3opus_env_list(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().parse().expect("逗号分隔的数"))
        .collect()
}

fn r3opus_wrapped_with(
    devices: SparseDevices,
    pool: &PoolUnderTest<SparseBlockDevice>,
    ring_slot: RootRingSlot,
    fault: InjectedFault,
    occurrence: FaultOccurrence,
) -> (
    Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>,
    SharedFaultPlan,
) {
    let plan = SharedFaultPlan::armed(
        DEVICE_WIDTH.fixed_geometry(),
        FaultSchedule {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
                named_slots: NamedRootRingSlots::NONE.with(ring_slot),
                region_devices: pool.parameters.region_devices,
                fixed_structure_slot_spacing: pool.parameters.geometry.fixed_structure_slot_spacing,
            }),
            counting: FaultCounting::AcrossThePool,
            occurrence,
        },
    );
    let wrapped = devices
        .into_iter()
        .map(|(identity, device)| (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone())))
        .collect();
    (wrapped, plan)
}

fn r3opus_short(text: String) -> String {
    text.chars().take(260).collect()
}

/// 用户动作那一维：四个入口（回退两种目标）加可写挂载。
#[derive(Clone, Copy, Debug)]
enum R3opusAction {
    Entry(EntryAfterTheMount, RollbackTargetOfTheEntry),
    WritableMount,
}

fn r3opus_actions() -> Vec<R3opusAction> {
    vec![
        R3opusAction::Entry(EntryAfterTheMount::SessionPublish, RollbackTargetOfTheEntry::FirstFileVersion),
        R3opusAction::Entry(EntryAfterTheMount::NormalUnmount, RollbackTargetOfTheEntry::FirstFileVersion),
        R3opusAction::Entry(EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling, RollbackTargetOfTheEntry::FirstFileVersion),
        R3opusAction::Entry(EntryAfterTheMount::AdministratorRollback, RollbackTargetOfTheEntry::FirstFileVersion),
        R3opusAction::Entry(EntryAfterTheMount::AdministratorRollback, RollbackTargetOfTheEntry::VersionBeforeTheCurrent),
        R3opusAction::WritableMount,
    ]
}

/// RU 的后果：P（txg 取 R3_TXGS）的根住的那块盘切在根槽 FUA 之后（落后、单元都在、根槽里是 P），整池第 n 次读 P 的根槽时
/// 读回的字节翻一位（只这一次），n 从 1 扫到 R3_NMAX 或那一槽读不到第 n 次为止；每个用户动作各走一次。
#[test]
fn r3opus_ru_one_corrupted_read_of_the_publish_root_slot() {
    let n_max = r3opus_env_list("R3_NMAX", "12")[0];
    for txg in r3opus_env_list("R3_TXGS", "6,7,8") {
        for action in r3opus_actions() {
            for n in 1..=n_max {
                let RootDeviceOfThePublishTakenMidPublish {
                    mut pool,
                    devices,
                    ring_slot_of_the_publish_root,
                    root_device_of_the_publish,
                } = root_device_of_the_publish_taken_mid_publish(
                    WhereThePublishIsCut::AfterItsRootSlotWrite,
                    CheckpointTxg(txg),
                );
                let fault = InjectedFault::ReadReturnsCorruptedBytes {
                    flipped_bit: singlefs_harness::fault_injection::FlippedBit { byte_index: 100, bit_index: 0 },
                };
                let started = std::time::Instant::now();
                let (outcome, fired, after) = match action {
                    R3opusAction::Entry(entry, target) => {
                        let (mut wrapped, plan) = r3opus_wrapped_with(
                            devices,
                            &pool,
                            ring_slot_of_the_publish_root,
                            fault,
                            FaultOccurrence::TheNthMatchingCall(n),
                        );
                        let refusal = go_through_rolling_back_to(entry, target, &mut pool, &mut wrapped);
                        let after = MemoryPool {
                            devices: wrapped
                                .iter()
                                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                                .collect(),
                            device_size_in_bytes: pool.device_bytes,
                        };
                        (r3opus_short(describe(&refusal)), plan.fired_count(), after)
                    }
                    R3opusAction::WritableMount => {
                        let (mut wrapped, plan) = r3opus_wrapped_with(
                            copies_of(&devices),
                            &pool,
                            ring_slot_of_the_publish_root,
                            fault,
                            FaultOccurrence::TheNthMatchingCall(n),
                        );
                        let outcome = match mount_writable(&pool.parameters, &mut wrapped) {
                            Ok(_mounted) => "做成了".to_string(),
                            Err(error) => r3opus_short(format!("{error:?}")),
                        };
                        let after = MemoryPool {
                            devices: wrapped
                                .iter()
                                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                                .collect(),
                            device_size_in_bytes: pool.device_bytes,
                        };
                        (outcome, plan.fired_count(), after)
                    }
                };
                let checker = checker_violations_on(&after);
                println!(
                    "R3RU txg={txg} root_dev={} action={action:?} n={n} fired={fired} outcome={outcome} checker={} ms={}",
                    root_device_of_the_publish.0,
                    checker.len(),
                    started.elapsed().as_millis()
                );
                if fired == 0 {
                    break;
                }
            }
        }
    }
}

/// RGC 的后果：P（txg 取 R3_RGC_TXGS，默认区域 2 的 8、26、29、32；26–32 根环转过一圈）的根住的那块盘切在根槽 FUA 之前，
/// 每个入口（回退两种目标）各走一次；之后这组盘的拷贝可写挂载一次。入口之后与再挂之后各跑一次池级 checker。
#[test]
fn r3opus_rgc_region_two_through_every_entry() {
    for txg in r3opus_env_list("R3_RGC_TXGS", "8,26,29,32") {
        for action in r3opus_actions() {
            let R3opusAction::Entry(entry, target) = action else { continue };
            let RootDeviceOfThePublishTakenMidPublish {
                mut pool,
                mut devices,
                root_device_of_the_publish,
                ring_slot_of_the_publish_root,
            } = root_device_of_the_publish_taken_mid_publish(
                WhereThePublishIsCut::BeforeItsRootSlotWrite,
                CheckpointTxg(txg),
            );
            let images_before = images_of(&devices);
            let refusal = go_through_rolling_back_to(entry, target, &mut pool, &mut devices);
            let unchanged = images_of(&devices) == images_before;
            let after = MemoryPool {
                devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
                device_size_in_bytes: pool.device_bytes,
            };
            let checker = checker_violations_on(&after);
            let remount = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
            println!(
                "R3RGC txg={txg} region={} slot={} root_dev={} action={action:?} outcome={} unchanged={unchanged} checker={} first_violation={:?} remount={}",
                ring_slot_of_the_publish_root.region,
                ring_slot_of_the_publish_root.slot,
                root_device_of_the_publish.0,
                r3opus_short(describe(&refusal)),
                checker.len(),
                checker.first().map(|line| line.chars().take(120).collect::<String>()),
                r3opus_short(format!("{remount:?}"))
            );
        }
    }
}
