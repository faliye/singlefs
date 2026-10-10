
fn pool_image_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

/// 第 txg 条根（每个自证过的根槽）的树表指针 86 字节写零、整槽重封。返回改过几个槽。
fn zero_the_tree_table_pointer_of_the_roots_with(devices: &mut Devices, txg: CheckpointTxg) -> usize {
    let parameters = parameters();
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).unwrap();
    let roots = self_verified_roots_of(devices, txg);
    for (ring_slot, _root) in &roots {
        let device = parameters.region_devices[usize::try_from(ring_slot.region).unwrap()];
        let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        let dev = &mut devices.iter_mut().find(|(identity, _)| *identity == device).unwrap().1;
        let mut slot = dev.image.read(offset, root_slot_bytes);
        for b in &mut slot[TREE_TABLE_POINTER_IN_THE_ROOT_SLOT..TREE_TABLE_POINTER_IN_THE_ROOT_SLOT + 86] { *b = 0; }
        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        dev.image.write(offset, &slot);
    }
    roots.len()
}

fn report(label: &str, mut devices: Devices) {
    let image = pool_image_of(&devices);
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    let partial: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::PartiallyJudged(_))).map(|(n, _)| *n).collect();
    let i2_5 = verdicts.iter().find(|(n, _)| *n == "I-2.5").map(|(_, v)| format!("{v:?}")).unwrap();
    println!("PROBE {label} checker_violated={violated:?}");
    println!("PROBE {label} checker_partially_judged={partial:?}");
    println!("PROBE {label} I-2.5={}", i2_5.chars().take(60).collect::<String>());
    let recovered = recover(&devices, JournalPolicy::Consult);
    let outcome = format!("{:?}", recovered.outcome);
    println!("PROBE {label} recover.outcome={}", outcome.chars().take(200).collect::<String>());
    println!("PROBE {label} recover.effective_root={:?} prefix_applied={}", recovered.effective_root, recovered.journal.prefix_applied);
    match mount_writable(&parameters(), &mut devices) {
        Ok(m) => println!("PROBE {label} mount_writable=Ok chosen_txg={:?} effective_txg={:?}", m.output.chosen_root.checkpoint_txg, m.output.effective_root.checkpoint_txg),
        Err(e) => println!("PROBE {label} mount_writable=Err {:?}", format!("{e:?}").chars().take(200).collect::<String>()),
    }
}

#[test]
fn probe_clean() { report("clean", pool_after_the_first_file()); }

#[test]
fn probe_newest_root_tree_table_pointer_all_zero() {
    let mut devices = pool_after_the_first_file();
    println!("PROBE newest zeroed_slots={}", zero_the_tree_table_pointer_of_the_roots_with(&mut devices, FIRST_FILE_TXG));
    report("newest(txg3)", devices);
}

#[test]
fn probe_older_root_tree_table_pointer_all_zero() {
    let mut devices = pool_after_the_first_file();
    println!("PROBE older zeroed_slots={}", zero_the_tree_table_pointer_of_the_roots_with(&mut devices, LAST_WARM_UP_TXG));
    report("older(txg2)", devices);
}

fn pool_after_a_first_file_of(content: Vec<u8>) -> Devices {
    let parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE)))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator: PoolAllocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号 1");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("暖机写了两条根"),
        FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS }, instance, &warmed.last_record_bytes).expect("第一个文件");
    devices
}

#[test]
fn probe_i55_verdicts_by_form() {
    for (label, content) in [("zero-length-first-file", Vec::new()), ("3000-byte-first-file", first_file_content())] {
        let devices = pool_after_a_first_file_of(content);
        let image = pool_image_of(&devices);
        let verdicts = check_pool_image(&image);
        let v = verdicts.iter().find(|(n, _)| *n == "I-5.5").map(|(_, v)| format!("{v:?}")).unwrap();
        println!("PROBE-I55 {label} I-5.5={}", v.chars().take(160).collect::<String>());
    }
}

/// 第 txg 条根的回退下界 F（根槽偏移 130 起 8 字节）改成 floor、整槽重封。
fn set_the_rollback_floor_of_the_roots_with(devices: &mut Devices, txg: CheckpointTxg, floor: u64) -> usize {
    let parameters = parameters();
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).unwrap();
    let roots = self_verified_roots_of(devices, txg);
    for (ring_slot, _root) in &roots {
        let device = parameters.region_devices[usize::try_from(ring_slot.region).unwrap()];
        let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        let dev = &mut devices.iter_mut().find(|(identity, _)| *identity == device).unwrap().1;
        let mut slot = dev.image.read(offset, root_slot_bytes);
        slot[130..138].copy_from_slice(&floor.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        dev.image.write(offset, &slot);
    }
    roots.len()
}

fn brief(label: &str, devices: &Devices) {
    let image = pool_image_of(devices);
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    let partial: Vec<(&str, String)> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::PartiallyJudged(_))).map(|(n, v)| (*n, format!("{v:?}").chars().take(150).collect::<String>())).collect();
    println!("PROBE-FLOOR {label} violated={violated:?}");
    println!("PROBE-FLOOR {label} partially_judged={partial:?}");
}

#[test]
fn probe_a_root_below_the_floor_with_an_all_zero_tree_table_pointer_is_not_a_candidate() {
    let mut devices = pool_after_the_first_file();
    println!("PROBE-FLOOR floor_set_slots={}", set_the_rollback_floor_of_the_roots_with(&mut devices, FIRST_FILE_TXG, 3));
    brief("F=3 on newest(txg3), roots txg1..2 below floor", &devices);
    println!("PROBE-FLOOR zeroed_slots={}", zero_the_tree_table_pointer_of_the_roots_with(&mut devices, LAST_WARM_UP_TXG));
    brief("F=3 and txg2 root tree-table pointer all-zero (txg2 is below the floor)", &devices);
}
