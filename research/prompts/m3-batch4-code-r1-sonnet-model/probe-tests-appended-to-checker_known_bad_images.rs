
// ===== 草稿目录里的探针（正推腿，不进仓）=====
fn probe_report_core_side(image: &MemoryPool, label: &str) {
    let mounted = singlefs_core::mounted_read::mount_read_only(image);
    match mounted {
        Err(error) => println!("PROBE {label} core mount_read_only = Err({error:?})"),
        Ok(mounted) => {
            let read = mounted
                .mounted
                .open_file(image, singlefs_core::address::InodeNumber(1))
                .map(|file| {
                    file.read_at(image, singlefs_core::address::FileOffsetInBytes(0), 1)
                        .map(|output| output.bytes.len())
                });
            println!("PROBE {label} core mount_read_only = Ok, open+read = {read:?}");
        }
    }
}

#[test]
fn zz_probe_extent_offset_on_the_extent_data_pointer() {
    let clean = build_pool("known-bad-pointer-journal-and-generation").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            let field = EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET - 8 - 2;
            assert_eq!(bytes[field..field + 2], [0, 0], "extent 偏移本来是 0");
            bytes[field] = 1;
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    println!(
        "PROBE extent-data-pointer checker violated = {:?}",
        violated_invariants(&verdicts)
    );
    probe_report_core_side(&clean, "clean");
    probe_report_core_side(&image, "extent-data-pointer-offset-1");
}

#[test]
fn zz_probe_safety_factor_two_with_consistent_fields() {
    let clean = build_pool("known-bad-ring-geometry-and-root-slot-alignment").memory_pool();
    let mut image = clean.clone();
    for (device, offset) in every_system_configuration_slot(&image) {
        mutate_system_configuration_slot(&mut image, device, offset, |bytes| {
            let ring_records = JOURNAL_RING_BYTES_OF_THE_POOL / 4096;
            let limit = ring_records / 2;
            set_u32(bytes, SYSTEM_CONFIGURATION_SAFETY_FACTOR_OFFSET, 2);
            set_u32(
                bytes,
                SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET,
                u32::try_from(limit).expect("装得进"),
            );
            set_u64(
                bytes,
                SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
                limit * 4096,
            );
        });
    }
    let verdicts = check_pool_image(&image);
    println!(
        "PROBE F=2 checker violated = {:?}; I-8.1 = {:?}; I-7.13 = {:?}",
        violated_invariants(&verdicts),
        verdict(&image, JOURNAL_RING_GEOMETRY_FIELDS_AGREE),
        verdict(&image, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS)
    );
    probe_report_core_side(&clean, "clean");
    probe_report_core_side(&image, "F=2-consistent");
}

#[test]
fn zz_probe_newest_root_only_with_extent_offset_one() {
    let clean = build_pool("known-bad-pointer-journal-and-generation").memory_pool();
    let mut image = clean.clone();
    let mut newest: Option<(u64, u32, u64)> = None;
    for (device, offset) in root_slots() {
        let bytes = read(&image, device, offset, 512);
        if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
            continue;
        }
        let txg = get_u64(&bytes, 28);
        if newest.map_or(true, |(best, _, _)| txg > best) {
            newest = Some((txg, device, offset));
        }
    }
    let (txg, device, offset) = newest.expect("有根");
    let mut bytes = read(&image, device, offset, 512);
    let field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
    bytes[field] = 1;
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(&mut image, device, offset, &bytes);
    let verdicts = check_pool_image(&image);
    println!(
        "PROBE newest-root-only (txg {txg}, device {device}) checker violated = {:?}; I-7.2 = {:?}",
        violated_invariants(&verdicts),
        verdict(&image, "I-7.2")
    );
    let mounted = singlefs_core::mounted_read::mount_read_only(&image);
    match mounted {
        Err(error) => println!("PROBE newest-root-only core mount_read_only = Err({error:?})"),
        Ok(mounted) => println!(
            "PROBE newest-root-only core mount_read_only = Ok, chosen_root txg {:?}, effective_root txg {:?}",
            mounted.chosen_root.checkpoint_txg, mounted.effective_root.checkpoint_txg
        ),
    }
    let clean_mounted = singlefs_core::mounted_read::mount_read_only(&clean).expect("干净镜像挂得上");
    println!(
        "PROBE clean core chosen_root txg {:?}, effective_root txg {:?}",
        clean_mounted.chosen_root.checkpoint_txg, clean_mounted.effective_root.checkpoint_txg
    );
}
