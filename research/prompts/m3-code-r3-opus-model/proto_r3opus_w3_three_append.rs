
// ─── m3-code-r3 攻方（Opus）原型：W3 三盘副本上的扫描（只在 work3：mkfs 放开到 3 块盘、起点按 R3_THREE_DEVICES / R3_LAYOUT 建池） ───

fn r3opus_env_list3(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().parse().expect("逗号分隔的数"))
        .collect()
}

/// P 的 txg 取 R3_W3_TXGS，P 的根住的那块盘切在根槽 FUA 之前（R3_W3_CUT=after 时切在之后），经四个入口（回退两种目标）各走一次；
/// 入口之后跑一次池级 checker，之后这组盘的拷贝可写挂载一次、再跑一次 checker。
#[test]
fn r3opus_w3_three_device_sweep() {
    let cut = if std::env::var("R3_W3_CUT").is_ok_and(|value| value == "after") {
        WhereThePublishIsCut::AfterItsRootSlotWrite
    } else {
        WhereThePublishIsCut::BeforeItsRootSlotWrite
    };
    let actions = [
        (EntryAfterTheMount::SessionPublish, RollbackTargetOfTheEntry::FirstFileVersion),
        (EntryAfterTheMount::NormalUnmount, RollbackTargetOfTheEntry::FirstFileVersion),
        (EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling, RollbackTargetOfTheEntry::FirstFileVersion),
        (EntryAfterTheMount::AdministratorRollback, RollbackTargetOfTheEntry::FirstFileVersion),
        (EntryAfterTheMount::AdministratorRollback, RollbackTargetOfTheEntry::VersionBeforeTheCurrent),
    ];
    for txg in r3opus_env_list3("R3_W3_TXGS", "6,7,8,25,26,27") {
        for (entry, target) in actions {
            let started = std::time::Instant::now();
            let RootDeviceOfThePublishTakenMidPublish {
                mut pool,
                mut devices,
                root_device_of_the_publish,
                ring_slot_of_the_publish_root,
            } = root_device_of_the_publish_taken_mid_publish(cut, CheckpointTxg(txg));
            let images_before = images_of(&devices);
            let refusal = go_through_rolling_back_to(entry, target, &mut pool, &mut devices);
            let unchanged = images_of(&devices) == images_before;
            let after = MemoryPool {
                devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
                device_size_in_bytes: pool.device_bytes,
            };
            let checker = checker_violations_on(&after);
            let mut copies = copies_of(&devices);
            let remount = mount_writable(&pool.parameters, &mut copies)
                .map(|mounted| (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0))
                .map_err(|error| format!("{error:?}").chars().take(120).collect::<String>());
            let after_remount = MemoryPool {
                devices: copies.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
                device_size_in_bytes: pool.device_bytes,
            };
            let checker_after_remount = if remount.is_ok() { checker_violations_on(&after_remount).len() } else { 0 };
            let outcome: String = describe(&refusal).chars().take(220).collect();
            println!(
                "W3SWEEP devices={} layout={:?} cut={cut:?} txg={txg} region={} slot={} root_dev={} entry={entry:?} target={target:?} outcome={outcome} unchanged={unchanged} checker={} first={:?} remount={remount:?} checker_after_remount={checker_after_remount} ms={}",
                devices.len(),
                pool.parameters.region_devices.map(|device| device.0),
                ring_slot_of_the_publish_root.region,
                ring_slot_of_the_publish_root.slot,
                root_device_of_the_publish.0,
                checker.len(),
                checker.first().map(|line| line.chars().take(100).collect::<String>()),
                started.elapsed().as_millis()
            );
        }
    }
}
