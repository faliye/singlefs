use e7_index_bench::usize_from_u64;
use super::*;

#[test]
fn mkfs_seeds_three_generation_zero_roots_readable_from_all_regions() {
    let parameters = PoolParameters::settled_two_devices();
    let (recording, genesis) = mkfs(&parameters);
    let system_configuration = choose_system_configuration(&recording.pool).expect("系统配置");
    let mut readable = 0;
    for region in 0..RING_REGIONS {
        let bytes = recording.pool.read(system_configuration.region_devices[usize_from_u64(region)], ring_slot_offset(region, 0), 512);
        if RootRecord::parse_slot(&bytes, &FIXED_FSID) == Some(genesis.root) {
            readable += 1;
        }
    }
    assert_eq!(readable, 3);
    assert_eq!(ring_region_offset(2).0, 7 << 20);
    // C512：mkfs 第 0 代写全零（这里直接查 mkfs 自己构造的根，不是靠 to_slot/parse_slot 往返——
    // 往返只证明「写的值读得回来」，证不了写的值就是全零，两者是不同的命题）。
    assert!(genesis.root.allocation_record_tree_root.is_empty_root(), "mkfs 第 0 代的分配记录树根指针全零");
    // D23 已定项 16：mkfs 写实例代号 0，单元写序 (0, 0)；树 ID 水位就是第一个要发的树 ID。
    assert_eq!(genesis.root.instance, InstanceGeneration(0));
    assert_eq!(genesis.root.tree_identifier_watermark, 65536, "mkfs 时水位 65536（D8 已定项 11，登记 7.2 B3）");
    assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH, 65549, "十三棵树（最后一棵退役树）⇒ 发布后水位 65549（D8 已定项 8 ②、已定项 11；登记 7.2 A-T4）");
    assert_eq!(genesis.root.tree_identifier_watermark + 13, TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH, "第一次发布发十三个号（7.2 A-T4：65549 − 65536 = 13）");
    assert_eq!(parse_packed_unit(&genesis.instance_table_unit).expect("实例表").write_order, WriteOrder { instance: InstanceGeneration(0), transaction: TransactionNumber(0) });
    // D22 已定项 16：mkfs 把每盘两个槽都种上世代号 1。
    for device_index in 0..2u32 {
        for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            let slot = recording.pool.read(DeviceIdentity(device_index), DeviceOffset(slot_offset), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
            let parsed_system_configuration = SystemConfiguration::parse_slot(&slot).expect("mkfs 的系统配置槽");
            assert_eq!(parsed_system_configuration.slot_generation, 1);
            assert_eq!(parsed_system_configuration.journal_instance, InstanceGeneration(0));
        }
    }
}

/// mkfs 出来的池带着每盘字节数；崩溃态镜像读出同一个数——分配记录树的读路径按它定根的格数。
#[test]
fn pools_and_crash_images_report_the_device_bytes_of_the_parameters() {
    for parameters in [PoolParameters::settled_two_devices(), p8g_parameters()] {
        let (recording, _) = mkfs(&parameters);
        assert_eq!(recording.pool.device_bytes(), parameters.device_bytes);
        let image = CrashImage { base: &recording.pool, writes: &[], persisted: Vec::new() };
        assert_eq!(image.device_bytes(), parameters.device_bytes);
    }
}
