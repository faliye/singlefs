use e7_index_bench::usize_from_u64;
use super::*;

/// 系统配置 489（D22 已定项 9 + 已定项 15，含自举头与几何段的四个字段、末尾的
/// 回退下界 F 8 字节）：每个字段按**绝对偏移**读一遍，以及「整槽校验和罩整个 4096 槽」——
/// 改 489 之后那 3607 字节补齐里的任何一个字节，解析都要拒绝。
#[test]
fn system_configuration_is_489_bytes_and_the_slot_checksum_covers_all_4096() {
    let BuiltPool { recording, .. } = built_pool();
    let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
    assert_eq!(slot.len(), 4096, "系统配置槽宽是格式常量 4096（D22 已定项 2，2026-09-14 三方论证后）");
    assert!(SystemConfiguration::parse_slot(&slot).is_some());
    let read_u64 = |offset: usize| u64::from_le_bytes(slot[offset..offset + 8].try_into().expect("切了 8 字节"));
    let read_u32 = |offset: usize| u32::from_le_bytes(slot[offset..offset + 4].try_into().expect("切了 4 字节"));
    // 自举头的两个字段：写入者身份 20（偏移 118）与校验和算法标识 1（偏移 138）。
    assert_eq!(&slot[118..118 + WRITER_IDENTITY_NAME.len()], WRITER_IDENTITY_NAME, "实现标识 ASCII");
    assert!(slot[118 + WRITER_IDENTITY_NAME.len()..134].iter().all(|byte| *byte == 0), "实现标识零补齐到 16");
    assert_eq!(read_u32(134), 1, "写入者版本（I-1.5、D17 债 3）");
    assert_eq!(slot[138], 1, "校验和算法标识 = 1 CRC-32C（I-2.2、C309）");
    // 几何段的两个字段：mkfs 时的 physical_block_size（偏移 321）与扩展点声明值 N（325）。
    assert_eq!(read_u32(317), 512, "mkfs 时的 physical_block_size（D13 已定项 5 的比对输入）");
    assert_eq!(read_u32(321), 0, "扩展点声明值 N（D21 已定项 8，第一版 0）");
    assert_eq!(read_u64(333), 805_306_368, "journal 环长 768 MiB（D23 已定项 19 ③）");
    assert_eq!(read_u32(345), 65_536, "在飞记录数上限（D23 已定项 18）");
    assert_eq!(read_u64(349), 268_435_456, "journal 最坏占用 = 在飞上限 × 4096（I-8.1）");
    assert_eq!(&slot[379..391], &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], "根环逐区域设备身份写死 0 / 1 / 0（D2 已定项 7）");
    assert_eq!(slot[391], 4, "w_max（D2 已定项 18）");
    assert_eq!(slot[392], 4, "组大小 g（D2 已定项 18）");
    assert_eq!(read_u64(417), 50_176, "单元区起始槽号（D3 已定项 10 ④）");
    assert_eq!(read_u32(425), 512, "mkfs 时的 io_min（D2 已定项 19）");
    assert_eq!(read_u32(429), 4096, "固定结构槽距（D2 已定项 19）");
    assert!(slot[445..469].iter().all(|byte| *byte == 0), "整理三条水位 24 字节恒 0（D22 已定项 15）");
    // 回退下界 F：住 [481, 489)，新池新建文件里没有抬 F，恒 0（D16 已定项 1）。
    assert_eq!(SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET, 481);
    assert_eq!(read_u64(SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET), 0, "新池新建文件里没有抬 F");
    assert!(slot[usize_from_u64(SYSTEM_CONFIGURATION_BYTES)..].iter().all(|byte| *byte == 0), "489 之后的 3607 字节补齐恒 0");
    let mut padded = slot.clone();
    padded[usize_from_u64(SYSTEM_CONFIGURATION_BYTES)] ^= 0x01;
    assert!(SystemConfiguration::parse_slot(&padded).is_none(), "整槽校验和罩到补齐区（D18 已定项 17）");
    // 算法标识写成未登记的码：这个槽不认（I-2.2）。
    let mut wrong_algorithm = slot.clone();
    wrong_algorithm[138] = 2;
    let digest = wide_checksum_with_field_zeroed(&wrong_algorithm, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    wrong_algorithm[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    assert!(SystemConfiguration::parse_slot(&wrong_algorithm).is_none(), "校验和算法标识不是 1 就不认这个槽");
}

#[test]
fn unknown_incompat_bit_or_missing_ssd_line_bit_refuses_to_mount_but_compat_bits_do_not() {
    let parameters = PoolParameters::settled_two_devices();
    let (recording, _) = mkfs(&parameters);
    let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[0]), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
    // 布局身份在位 1，位 0 空出（D15 已定项 4）；位 2 是 inode 记录带截断字段，mkfs 起就置上。
    assert_eq!(slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET], 0x06, "mkfs 起 incompat 位 1 与位 2 都置 1（判决 m4-trunc-r2 第六节）");
    assert!(slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 1..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0), "其余 95 字节全 0");
    assert!(SystemConfiguration::parse_slot(&slot).is_some(), "原样（byte0 = 0x06）可挂");
    let reseal = |mut bytes: Vec<u8>, offset: usize, value: u8| {
        bytes[offset] = value;
        let digest = wide_checksum_with_field_zeroed(&bytes, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    };
    assert_eq!(SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 6);
    // 拒挂四档：0x02（只有位 1、没有位 2）、0x00（没有布局身份）、0x04（只有位 2、没有布局身份）、0x0e（位 1、位 2 加没登记的位 3）。
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x02)).is_none(), "只带位 1、不带位 2：拒绝");
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x00)).is_none(), "没有布局身份：拒绝");
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x04)).is_none(), "只有位 2、没有布局身份：拒绝");
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x0e)).is_none(), "位 1、位 2 加没登记的位 3：不认识的位拒");
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES - 1, 0x80)).is_none(), "incompat 位 255 没登记：不认识不许挂");
    assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat_ro 位不认识只读挂，读者照样解析");
    assert!(SystemConfiguration::parse_slot(&reseal(slot, SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 2 * FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat 位不认识随便");
}

/// 第八节 G8（A24）：回退下界 F 从 0 换成一个可辨识的模式值，差异字节必须恰好落在
/// [155, 159)（整槽校验和）与 [481, 489)（F 自己），解析要把值原样读回、不是从别的偏移读、不是大端。
#[test]
fn system_configuration_rollback_floor_round_trips_at_absolute_offset_481() {
    let parameters = PoolParameters::settled_two_devices();
    let baseline = SystemConfiguration {
        fsid: parameters.fsid,
        this_device: DeviceIdentity(0),
        device_count: 2,
        slot_generation: 5,
        region_devices: parameters.region_devices,
        journal_tail: 3,
        journal_instance: InstanceGeneration(1),
        rollback_floor: CheckpointTxg(0),
    };
    let modified = SystemConfiguration {
        fsid: parameters.fsid,
        this_device: DeviceIdentity(0),
        device_count: 2,
        slot_generation: 5,
        region_devices: parameters.region_devices,
        journal_tail: 3,
        journal_instance: InstanceGeneration(1),
        rollback_floor: CheckpointTxg(0x0807_0605_0403_0201),
    };
    let baseline_bytes = baseline.to_slot();
    let modified_bytes = modified.to_slot();
    assert_eq!(
        &modified_bytes[SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET..SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET + 8],
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        "F 8 字节小端写在 [481, 489)"
    );
    let differing_offsets: Vec<usize> = (0..baseline_bytes.len()).filter(|&index| baseline_bytes[index] != modified_bytes[index]).collect();
    let expected_offsets: Vec<usize> = (155..159).chain(481..489).collect();
    assert_eq!(differing_offsets, expected_offsets, "F 从 0 换成非 0，只应该动 [155, 159) 的校验和与 [481, 489) 的 F 自己");
    let parsed = SystemConfiguration::parse_slot(&modified_bytes).expect("重算过校验和的槽要能解析");
    assert_eq!(parsed.rollback_floor, CheckpointTxg(0x0807_0605_0403_0201), "F 原样读回");
}

/// 第八节 G9（A27）：根记录 flags 位 0 是卸载记号，其余位第一版恒 0、非 0 拒收；卸载记号能原样解析回来。
#[test]
fn root_record_unmount_marker_round_trips_and_other_flag_bits_are_rejected() {
    let parameters = PoolParameters::settled_two_devices();
    let (_, genesis) = mkfs(&parameters);
    let baseline = genesis.root;
    let mut marked = baseline;
    marked.unmount_marker = true;
    let baseline_bytes = baseline.to_slot();
    let marked_bytes = marked.to_slot();
    assert_eq!(&baseline_bytes[20..24], &[0, 0, 0, 0], "mkfs 的根不带卸载记号");
    assert_eq!(&marked_bytes[20..24], &[1, 0, 0, 0], "卸载记号写进 flags 位 0（小端）");
    let differing_offsets: Vec<usize> = (0..baseline_bytes.len()).filter(|&index| baseline_bytes[index] != marked_bytes[index]).collect();
    assert!(differing_offsets.contains(&20), "flags 第一字节要变");
    assert!(differing_offsets.iter().all(|offset| *offset == 20 || (138..142).contains(offset)), "只应该动 flags 与自证校验和");
    let parsed = RootRecord::parse_slot(&marked_bytes, &parameters.fsid).expect("重算过校验和的根槽要能解析");
    assert!(parsed.unmount_marker, "卸载记号原样解析回来");
    let mut other_bit = marked_bytes.clone();
    other_bit[20..24].copy_from_slice(&0x0000_0002u32.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(&other_bit, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET);
    other_bit[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    assert!(RootRecord::parse_slot(&other_bit, &parameters.fsid).is_none(), "flags 位 1 非 0：拒收（D22 已定项 17，其余位第一版恒 0）");
}

/// 根记录 457（D22 已定项 7 的字段表，中央映射树根指针之后是算法类型 1 + nonce 12 + MAC 16；
/// C512（树表 0 条的一版上被换下的实例表记在哪没有条款）：映射树根指针与那三段之间是分配记录树根指针 86）：
/// 自证校验和罩整个 512 槽（D18 已定项 17），末尾 29 字节第一版全 0，分配记录树根指针这个装置的场景里也恒 0。
#[test]
fn root_record_is_457_bytes_and_carries_the_mapping_tree_root() {
    let BuiltPool { output, .. } = built_pool();
    let slot = output.root.to_slot();
    assert_eq!(slot.len(), usize_from_u64(PHYSICAL_BLOCK_BYTES));
    assert_eq!(RootRecord::parse_slot(&slot, &FIXED_FSID), Some(output.root));
    assert!(slot[usize_from_u64(ROOT_RECORD_BYTES)..].iter().all(|byte| *byte == 0), "457 之后的 55 字节补齐恒 0");
    let mut padded = slot.clone();
    padded[usize_from_u64(ROOT_RECORD_BYTES)] ^= 0x01;
    assert!(RootRecord::parse_slot(&padded, &FIXED_FSID).is_none(), "自证校验和罩到补齐区");
    // 分配记录树根指针这 86 字节这个装置的场景里恒 0（C512：只有「树表 0 条、写过行」的一版写非零，这个装置不建那一格）。
    assert!(slot[342..428].iter().all(|byte| *byte == 0), "分配记录树根指针这个装置的场景里恒 0");
    // 算法类型 / nonce / MAC 这 29 字节是第一版的留位，全 0；映射树根指针在它们之前隔着分配记录树根指针。
    assert!(slot[428..457].iter().all(|byte| *byte == 0), "算法类型 1 + nonce 12 + MAC 16 第一版全 0");
    // 7.2 B10：根记录里 flags 20、水位 122（4 + 16 + 4 + 4 + 8 + 86）、自证校验和 138、实例表单元指针 [170, 256)、中央映射树根指针 [256, 342)、
    // 分配记录树根指针 [342, 428)（4 + 16 + 4 + 4 + 8 + 86 + 8 + 8 + 32 = 170）。
    assert_eq!(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES, 170, "实例表单元指针起点");
    assert_eq!(&slot[122..130], &65549u64.to_le_bytes(), "水位在 122：新池新建文件的根写 65549");
    let mut instance_table_pointer_bytes = ByteReader::at(&slot, 170);
    assert_eq!(NodePointer::read_from(&mut instance_table_pointer_bytes), output.root.instance_table);
    let mut mapping_pointer_at_256 = ByteReader::at(&slot, 256);
    assert_eq!(NodePointer::read_from(&mut mapping_pointer_at_256), output.root.mapping_root);
    let mut mapping_pointer_bytes = ByteReader::at(&slot, usize_from_u64(ROOT_RECORD_BYTES - 2 * NODE_POINTER_BYTES - 1 - 12 - 16));
    assert_eq!(NodePointer::read_from(&mut mapping_pointer_bytes), output.root.mapping_root);
    assert_eq!(output.root.mapping_root.locations[0].slot, SlotNumber(SLOT_MAPPING_ROOT));
    let mut allocation_record_tree_root_bytes = ByteReader::at(&slot, usize_from_u64(ROOT_RECORD_BYTES - NODE_POINTER_BYTES - 1 - 12 - 16));
    assert_eq!(NodePointer::read_from(&mut allocation_record_tree_root_bytes), output.root.allocation_record_tree_root);
}

/// 7.2 B11 的落点绝对值（独立算出的字节偏移）：t1 在 50180 × 16384、区域 1 与 2 的槽 0、t14（区域 0 槽 1）、journal 环里计数器 1、2、3。
/// 字段表（`layout_field_rows`）里写的是这些字面量，这条把装置自己的落点函数钉在同一批数上。
#[test]
fn placements_match_the_independently_computed_byte_offsets() {
    assert_eq!(SlotNumber(50180).device_offset().0, 822_149_120);
    assert_eq!(ring_region_offset(1).0, 4_194_304);
    assert_eq!(ring_region_offset(2).0, 7_340_032);
    assert_eq!(ring_slot_offset(0, 1).0, 1_052_672);
    assert_eq!(ring_slot_offset(1, 0).0, 4_194_304, "w2：区域 1 的槽 0");
    assert_eq!(ring_slot_offset(2, 0).0, 7_340_032, "w5：区域 2 的槽 0");
    assert_eq!(journal_record_offset(JournalCounter(1)).0, 16_777_216);
    assert_eq!(journal_record_offset(JournalCounter(2)).0, 16_781_312);
    assert_eq!(journal_record_offset(JournalCounter(3)).0, 16_785_408, "t13：16 MiB + 8192");
    assert_eq!(ring_target_for_publish(CheckpointTxg(1)), (1, 0), "w2 写区域 1 的槽 0");
    assert_eq!(ring_target_for_publish(CheckpointTxg(2)), (2, 0), "w5 写区域 2 的槽 0");
    assert_eq!(ring_target_for_publish(CheckpointTxg(3)), (0, 1), "t14 写区域 0 的槽 1");
    let BuiltPool { output, .. } = built_pool();
    assert!(output.record.named.len() <= 64, "7.1 乙 C4：t13 点名 12 项，装得下一条记录的 64 项");
    assert_eq!(output.record.named.len(), 12);
}
