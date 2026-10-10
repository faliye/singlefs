
// ───────────── m3-formal-c655-c687-r1 攻方探针（只在攻方副本里，不进仓） ─────────────

/// 与 `build_pool` 同一串步骤（mkfs、取号、暖机、新池新建文件），盘是内存稀疏盘，不建镜像文件。
fn c687_probe_clean_image() -> MemoryPool {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content = common::file_content();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 同上，再接一次覆盖写（与 `image_after_the_overwrite` 同内容、同时间）：发布 B 是 txg 4，不是从 mkfs 水位建树的那一版。
fn c687_probe_image_after_overwrite() -> MemoryPool {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device| (DeviceIdentity(device), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content = common::file_content();
    let second: Vec<u8> = (0..4100usize).map(|index| u8::try_from((index * 7 + 3) % 253).expect("小")).collect();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
        let b = publish_overwrite(&mut pool, &mut allocator, &first, FirstFile { content: &second, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, instance).expect("覆盖写");
        assert_eq!(b.root.checkpoint_txg, CheckpointTxg(4));
    }
    MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

#[test]
fn c687_probe_duplicate_kinds_on_a_version_not_built_from_the_mkfs_watermark() {
    let clean = c687_probe_image_after_overwrite();
    c687_probe_report("after-overwrite-clean", &clean, &clean);
    let cases: Vec<(&str, Box<dyn Fn(&mut MemoryPool)>)> = vec![
        ("f1-after-overwrite-livelist-kind-to-extent-head-65537", Box::new(|image: &mut MemoryPool| {
            mutate_unit_of(image, &UNITS_AFTER_OVERWRITE, TREE_TABLE_AFTER_OVERWRITE, |bytes| {
                c687_probe_set_tree_table_entry_kind(bytes, 4, 1);
                set_u64(bytes, tree_table_entry_offset(4) + 116, 65537);
            }, true);
        })),
        ("f2-after-overwrite-deadlist-kind-to-allocation", Box::new(|image: &mut MemoryPool| {
            mutate_unit_of(image, &UNITS_AFTER_OVERWRITE, TREE_TABLE_AFTER_OVERWRITE, |bytes| c687_probe_set_tree_table_entry_kind(bytes, 6, 3), true);
        })),
    ];
    for (name, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        c687_probe_report(name, &image, &clean);
    }
}

fn c687_probe_report(name: &str, image: &MemoryPool, clean: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let clean_verdicts = check_pool_image(clean);
    let i55 = verdicts
        .iter()
        .find(|(identifier, _)| *identifier == FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK)
        .map(|(_, verdict)| verdict.clone());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(identifier, _)| *identifier)
        .collect();
    let partially: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::PartiallyJudged(_)))
        .map(|(identifier, _)| *identifier)
        .collect();
    let turned_not_applicable: Vec<&str> = verdicts
        .iter()
        .zip(&clean_verdicts)
        .filter(|((_, now), (_, before))| {
            matches!(now, InvariantVerdict::NotApplicable(_)) && !matches!(before, InvariantVerdict::NotApplicable(_))
        })
        .map(|((identifier, _), _)| *identifier)
        .collect();
    let recovery = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { .. } => "file".to_string(),
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{}", format!("{failure:?}").chars().take(90).collect::<String>()),
    };
    println!("name=c687-cell image={name} i55={i55:?} violated={violated:?} partially_judged={partially:?} turned_not_applicable={turned_not_applicable:?} core_recover={recovery}");
}

fn c687_probe_set_tree_table_entry_kind(bytes: &mut Vec<u8>, index: usize, kind: u16) {
    set_u16(bytes, tree_table_entry_offset(index) + 10, kind);
}

#[test]
fn c687_probe_cells_on_bad_images() {
    let clean = c687_probe_clean_image();
    c687_probe_report("clean", &clean, &clean);
    let data_unit_crc = crc32_castagnoli(&read(&clean, 0, DATA_UNIT * SLOT, 32768));
    let cases: Vec<(&str, Box<dyn Fn(&mut MemoryPool)>)> = vec![
        ("a1-tree-table-zeroed-both-devices", Box::new(|image: &mut MemoryPool| {
            for device in DEVICES { write(image, device, TREE_TABLE * SLOT, &vec![0u8; 16384]); }
        })),
        ("a2-tree-table-entry-width-narrowed", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| narrow_the_entry_width_to_the_key_width(bytes), true);
        })),
        ("b1-livelist-entry-kind-to-extent", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| c687_probe_set_tree_table_entry_kind(bytes, 4, 1), true);
        })),
        ("b2-livelist-entry-kind-to-extent-head-65537", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| {
                c687_probe_set_tree_table_entry_kind(bytes, 4, 1);
                set_u64(bytes, tree_table_entry_offset(4) + 116, 65537);
            }, true);
        })),
        ("b3-logged-ops-entry-kind-to-extent-head-65537", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| {
                c687_probe_set_tree_table_entry_kind(bytes, 10, 1);
                set_u64(bytes, tree_table_entry_offset(10) + 116, 65537);
            }, true);
        })),
        ("c1-extent-root-zeroed-both-devices", Box::new(|image: &mut MemoryPool| {
            for device in DEVICES { write(image, device, EXTENT_ROOT * SLOT, &vec![0u8; 16384]); }
        })),
        ("d1-data-pointer-second-location-device-1-to-0", Box::new(move |image: &mut MemoryPool| {
            mutate_unit(image, EXTENT_ROOT, |bytes| {
                assert!(replace_all(bytes, &location_pattern(1, DATA_UNIT, data_unit_crc), &location_pattern(0, DATA_UNIT, data_unit_crc)));
            }, true);
        })),
        ("d2-data-pointer-second-location-device-1-to-7", Box::new(move |image: &mut MemoryPool| {
            mutate_unit(image, EXTENT_ROOT, |bytes| {
                assert!(replace_all(bytes, &location_pattern(1, DATA_UNIT, data_unit_crc), &location_pattern(7, DATA_UNIT, data_unit_crc)));
            }, true);
        })),
        ("e1-allocation-entry-kind-to-sparse-side-table", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| c687_probe_set_tree_table_entry_kind(bytes, 2, 7), true);
        })),
        ("e2-deadlist-entry-kind-to-allocation", Box::new(|image: &mut MemoryPool| {
            mutate_unit(image, TREE_TABLE, |bytes| c687_probe_set_tree_table_entry_kind(bytes, 6, 3), true);
        })),
        ("e3-allocation-root-zeroed-both-devices", Box::new(|image: &mut MemoryPool| {
            for device in DEVICES { write(image, device, 50249 * SLOT, &vec![0u8; 16384]); }
        })),
    ];
    for (name, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        c687_probe_report(name, &image, &clean);
    }
}

/// 根环里 checkpoint_txg 为 `txg` 的那条根槽（盘, 偏移）：照 `root_slots()` 逐槽读，magic 对上、偏移 28 的 txg 相等的第一个。
fn c655_probe_root_slot_of_txg(image: &MemoryPool, txg: u64) -> (u32, u64) {
    root_slots()
        .into_iter()
        .find(|(device, offset)| {
            let bytes = read(image, *device, *offset, 512);
            &bytes[..4] == b"SFSR" && get_u64(&bytes, 28) == txg
        })
        .expect("根环里有这一版的根")
}

/// 改 txg 4（B）那条根槽里 [from, to) 那段字节、重算整槽校验和，再把每条根抄进它那几条记录的新根段（I-8.10 照旧成立）。
fn c655_probe_rewrite_root_bytes(image: &mut MemoryPool, change: impl Fn(&mut Vec<u8>)) {
    let (device, offset) = c655_probe_root_slot_of_txg(image, 4);
    let mut bytes = read(image, device, offset, 512);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(image, device, offset, &bytes);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// Y1：一个坏的挂载把表外字段写坏，checker 与 core 的只读恢复各看不看得出（根记录偏移：实例表 170、中央映射 256、分配记录树 342、算法 428、nonce 429、MAC 441）。
#[test]
fn c655_probe_root_fields_outside_the_table_written_wrong_on_a_file_version() {
    let clean = c687_probe_image_after_overwrite();
    let (device_a, offset_a) = c655_probe_root_slot_of_txg(&clean, 3);
    let root_a = read(&clean, device_a, offset_a, 512);
    let cases: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>)> = vec![
        ("m1-B-mapping-root-is-A-mapping-root", Box::new(move |bytes: &mut Vec<u8>| bytes[256..342].copy_from_slice(&root_a[256..342]))),
        ("m2-B-mapping-root-zeroed", Box::new(|bytes: &mut Vec<u8>| bytes[256..342].fill(0))),
        ("m3-B-allocation-record-tree-root-is-its-tree-table-pointer", Box::new(|bytes: &mut Vec<u8>| { let tree_table = bytes[36..122].to_vec(); bytes[342..428].copy_from_slice(&tree_table); })),
        ("m4-B-algorithm-1", Box::new(|bytes: &mut Vec<u8>| bytes[428] = 1)),
        ("m5-B-nonce-byte-5a", Box::new(|bytes: &mut Vec<u8>| bytes[430] = 0x5a)),
        ("m6-B-mac-byte-5a", Box::new(|bytes: &mut Vec<u8>| bytes[450] = 0x5a)),
    ];
    c687_probe_report("after-overwrite-clean", &clean, &clean);
    for (name, change) in cases {
        let mut image = clean.clone();
        c655_probe_rewrite_root_bytes(&mut image, |bytes| change(bytes));
        c687_probe_report(name, &image, &clean);
        let report = recover(&image, JournalPolicy::Consult);
        let same_content = match (&report.outcome, &recover(&clean, JournalPolicy::Consult).outcome) {
            (RecoveryOutcome::FileRead { content: a, .. }, RecoveryOutcome::FileRead { content: b, .. }) => a == b,
            _ => false,
        };
        println!("name=c655-root-field image={name} recovered_root={:?} posix_same_as_clean={same_content}", report.effective_root);
    }
}

/// Y1：坏的挂载把系统配置的运行量写坏（最新那一槽、两块盘一起改，整槽校验和重算）：checker 看不看得出、之后的可写挂载怎么走。
#[test]
fn c655_probe_system_configuration_runtime_quantities_written_wrong() {
    let clean = c687_probe_image_after_overwrite();
    let newest_slot_offset = |image: &MemoryPool| -> u64 {
        let generation = |offset: u64| get_u64(&read(image, 0, offset, 4096), 147);
        if generation(0) >= generation(4096) { 0 } else { 4096 }
    };
    let cases: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>)> = vec![
        ("q1-tail-zero", Box::new(|bytes: &mut Vec<u8>| set_u64(bytes, 469, 0))),
        ("q2-tail-plus-100", Box::new(|bytes: &mut Vec<u8>| { let tail = get_u64(bytes, 469); set_u64(bytes, 469, tail + 100); })),
        ("q3-journal-instance-9", Box::new(|bytes: &mut Vec<u8>| set_u32(bytes, 477, 9))),
        ("q4-slot-generation-plus-10", Box::new(|bytes: &mut Vec<u8>| { let generation = get_u64(bytes, 147); set_u64(bytes, 147, generation + 10); })),
        ("q5-slot-generation-minus-1", Box::new(|bytes: &mut Vec<u8>| { let generation = get_u64(bytes, 147); set_u64(bytes, 147, generation - 1); })),
    ];
    for (name, change) in cases {
        let mut image = clean.clone();
        let offset = newest_slot_offset(&image);
        for device in DEVICES {
            mutate_system_configuration_slot(&mut image, device, offset, |bytes| change(bytes));
        }
        c687_probe_report(name, &image, &clean);
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
                device.image = sectors.clone();
                (*identity, device)
            })
            .collect();
        let mounted = match mount_writable(&parameters(), &mut devices) {
            Ok(mounted) => format!("mounted instance {:?}", mounted.output.instance),
            Err(error) => format!("refused {}", format!("{error:?}").chars().take(120).collect::<String>()),
        };
        println!("name=c655-sc-field image={name} slot_offset={offset} writable_mount={mounted}");
    }
}

/// 同上，改的是 A（txg 3，覆盖写之后不是最新那一版）的根槽：挂载写的那几版在「挂载加一次发布」之后也都不是最新。
#[test]
fn c655_probe_root_fields_written_wrong_on_a_version_that_is_not_the_newest() {
    let clean = c687_probe_image_after_overwrite();
    let (device_b, offset_b) = c655_probe_root_slot_of_txg(&clean, 4);
    let root_b = read(&clean, device_b, offset_b, 512);
    let rewrite_a = |image: &mut MemoryPool, change: &dyn Fn(&mut Vec<u8>)| {
        let (device, offset) = c655_probe_root_slot_of_txg(image, 3);
        let mut bytes = read(image, device, offset, 512);
        change(&mut bytes);
        let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
        bytes[138..170].copy_from_slice(&digest);
        write(image, device, offset, &bytes);
        copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    };
    let cases: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>)> = vec![
        ("m7-A-mapping-root-zeroed", Box::new(|bytes: &mut Vec<u8>| bytes[256..342].fill(0))),
        ("m8-A-mapping-root-is-B-mapping-root", Box::new(move |bytes: &mut Vec<u8>| bytes[256..342].copy_from_slice(&root_b[256..342]))),
        ("m9-A-allocation-record-tree-root-is-its-tree-table-pointer", Box::new(|bytes: &mut Vec<u8>| { let tree_table = bytes[36..122].to_vec(); bytes[342..428].copy_from_slice(&tree_table); })),
    ];
    for (name, change) in cases {
        let mut image = clean.clone();
        rewrite_a(&mut image, &*change);
        c687_probe_report(name, &image, &clean);
    }
}

/// 树表里去掉一条（第 `index` 条之后的往前挪、末尾 200 字节清零、条目数与声明长度各减一条），重封。
fn c687_probe_remove_tree_table_entry(bytes: &mut Vec<u8>, index: usize) {
    let key_width = usize::from(bytes[51]);
    let count_offset = 82 + 2 * key_width;
    let count = usize::from(u16::from_le_bytes([bytes[count_offset], bytes[count_offset + 1]]));
    let start = tree_table_entry_offset(index);
    let end = tree_table_entry_offset(count);
    bytes.copy_within(start + 200..end, start);
    bytes[end - 200..end].fill(0);
    set_u16(bytes, count_offset, u16::try_from(count - 1).expect("条数"));
    let declared = u16::from_le_bytes([bytes[8], bytes[9]]);
    set_u16(bytes, 8, declared - 200);
}

#[test]
fn c687_probe_tree_table_without_the_allocation_entry() {
    let clean = c687_probe_clean_image();
    for (name, index) in [("e4-allocation-entry-removed", 2usize), ("e5-extent-entry-removed", 0usize)] {
        let mut image = clean.clone();
        mutate_unit(&mut image, TREE_TABLE, |bytes| c687_probe_remove_tree_table_entry(bytes, index), true);
        c687_probe_report(name, &image, &clean);
    }
}
