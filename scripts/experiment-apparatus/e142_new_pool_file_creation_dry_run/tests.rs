use super::*;

#[test]
fn widths_equal_the_byte_table_and_the_root_record_is_457() {
    // 每一行左边是字节表零到七写死的绝对值，右边是装置自己算出来的——不是几个常量互相比。
    assert_eq!(ROOT_RECORD_BYTES, 457);
    assert_eq!(ROOT_CHECKSUM_OFFSET, 138, "自证校验和在根记录里的偏移");
    assert_eq!(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 3 * NODE_POINTER_BYTES + 1 + 12 + 16, ROOT_RECORD_BYTES);
    assert_eq!(POINTER_HEAD_BYTES, 50);
    assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 4 + 4, NODE_POINTER_BYTES);
    assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 10, DATA_POINTER_BYTES);
    assert_eq!(NODE_POINTER_BYTES, 86);
    assert_eq!(DATA_POINTER_BYTES, 88);
    assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET, 46);
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES, 78);
    assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES, JOURNAL_HEADER_TEN_FIELD_BYTES);
    // 事务号 8 + 提交标记 1 + 本次发布内序号 4（已定项 4）+ 反向链 4 + 载荷校验和 4 + 新根段 364 + fsid 8 + MAC 16 = 409；78 + 409 = 487
    // （第二十一次跑登记 7.2 B5）。
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + 4 + JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8 + 16, JOURNAL_HEADER_BYTES);
    assert_eq!(JOURNAL_HEADER_BYTES, 487);
    assert_eq!(JOURNAL_HEADER_BYTES - JOURNAL_HEADER_TEN_FIELD_BYTES, 409, "十个字段之后的增量");
    assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 364);
    assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES, 3609, "4096 − 487（7.2 B7）");
    assert_eq!((JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES, 64, "4096 的记录装 64 个点名项（D23 已定项 17）");
    assert_eq!((JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) % JOURNAL_NAMED_ENTRY_BYTES, 25, "3609 除 56 余 25");
    assert_eq!(PHYSICAL_BLOCK_BYTES - JOURNAL_HEADER_BYTES, 25, "512 扇区上头之后还余 25 字节（D23 已定项 4，7.2 B8）");
    // A2（第七节）：字段起点逐个钉住，记录标志 7、事务号 78、提交标记 86、本次发布内序号 87、反向链 91、
    // 载荷校验和 95、新根段 99、fsid 463、MAC 471、头末 487（D23 已定项 4）。
    // 新根段内（D23 已定项 15 的七项次序，7.2 B6）：树表 99、映射 185、水位 271、F 279、实例表指针 287、分配记录树根指针 373、卸载记号 459、段末 463。
    assert_eq!(99 + NODE_POINTER_BYTES, 185, "映射树根指针起点");
    assert_eq!(185 + NODE_POINTER_BYTES, 271, "树 ID 水位起点");
    assert_eq!(271 + 8, 279, "回退下界 F 起点");
    assert_eq!(279 + 8, 287, "实例表单元指针起点");
    assert_eq!(287 + NODE_POINTER_BYTES, 373, "分配记录树根指针起点");
    assert_eq!(373 + NODE_POINTER_BYTES, 459, "卸载记号起点");
    assert_eq!(459 + 4, 463, "新根段末 = fsid 起点");
    assert_eq!(99 + JOURNAL_NEW_ROOT_SEGMENT_BYTES, 463, "fsid 起点");
    assert_eq!(463 + 8, 471, "MAC 起点");
    assert_eq!(471 + 16, JOURNAL_HEADER_BYTES, "头末");
    assert_eq!(JOURNAL_RECORD_FLAGS_OFFSET, 7);
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES, 78, "事务号起点");
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8, 86, "提交标记起点");
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1, 87, "本次发布内序号起点");
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4, 91, "反向链起点");
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4, 95, "载荷校验和起点");
    assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + 4, 99, "新根段起点");
    assert_eq!(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET, 155);
    assert_eq!(SYSTEM_CONFIGURATION_FSID_OFFSET, 102);
    assert_eq!(SYSTEM_CONFIGURATION_BYTES, 489, "2026-09-26 用户定案加回退下界 F 8 字节，481 → 489");
    assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES, 4096, "系统配置槽宽是格式常量（D22 已定项 2，2026-09-14 三方论证后按主 agent 推荐值）");
    assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES, SYSTEM_CONFIGURATION_BYTES + 3607, "489 的系统配置在 4096 槽里余 3607");
    assert_eq!(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1] - SYSTEM_CONFIGURATION_SLOT_OFFSETS[0], SYSTEM_CONFIGURATION_SLOT_BYTES, "槽距与槽宽同值 ⇒ 两个槽首尾相接、不重叠");
    assert_eq!(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64 + 8 + 4, SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET as u64, "journal 实例代号之后紧跟回退下界 F");
    assert_eq!(SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET as u64 + 8, SYSTEM_CONFIGURATION_BYTES, "回退下界 F 8 字节是字段表末行（D22 已定项 9）");
    assert_eq!(MAPPING_KEY_BYTES, 27);
    assert_eq!(MAPPING_KEY_BYTES + 2 * LOC_ENTRY, MAPPING_ENTRY_BYTES);
    assert_eq!(MAPPING_ENTRY_BYTES, 55, "映射条目一律 55（D19 已定项 10，2026-09-14 用户定案）");
    assert_eq!(MAPPING_KEY_NODE_SIGNIFICANT_BYTES + 2, MAPPING_KEY_BYTES, "码 2 / 码 3 的 key 末尾补两个零");
    assert_eq!(ALLOCATION_KEY_BYTES as u64 + 10, ALLOCATION_RECORD_BYTES);
    assert_eq!(24 + DATA_POINTER_BYTES, EXTENT_LEAF_RECORD_BYTES);
    assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112);
    assert_eq!(8 + 26 + NODE_POINTER_BYTES, INODE_INTERNAL_ENTRY);
    assert_eq!(INODE_INTERNAL_ENTRY, 120);
    assert_eq!(TREE_TABLE_ENTRY_BYTES, 200);
    assert_eq!(8 + 2 + 2 + 2 + NODE_POINTER_BYTES + 8 + 8 + 8 + 76, TREE_TABLE_ENTRY_BYTES);
    assert_eq!(NONCE_MAC_RESERVED_BYTES, 29, "nonce 12 + MAC 16 + 算法类型 1（D18 已定项 16，2026-09-14 用户定案）");
    assert_eq!(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES, 134, "码 1 载荷从 134 起");
    assert_eq!(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES, 136, "码 3 记录区从 136 起");
    assert_eq!((DATA_UNIT_BYTES - (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES)) / JOURNAL_NAMED_ENTRY_BYTES, 582, "按点名项宽 56 算的码 3 容量（不是墓碑宽：第一版墓碑 40 字节、容量 815，I-1.7 与 D18 已定项 24）");
    assert_eq!((DATA_UNIT_BYTES - (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES)) / INSTANCE_ROW_BYTES, 370, "实例表一片 370 条");
    // D23 已定项 18 / 19 ③：环 768 MiB ⇒ 196608 个 4 KiB 槽，在飞上限 = 槽数 ÷ 3 = 65536；单元区起于槽 50176。
    assert_eq!(JOURNAL_RING_BYTES, 805_306_368);
    assert_eq!(JOURNAL_RING_SLOTS, 196_608);
    assert_eq!(JOURNAL_IN_FLIGHT_RECORD_LIMIT, 65_536);
    // 钉绝对值：合计对账只对宽度不对取值，取值写错抓不到。
    assert_eq!(JOURNAL_WORST_CASE_BYTES, 268_435_456);
    assert_eq!(JOURNAL_WORST_CASE_BYTES, JOURNAL_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES);
    assert_eq!(UNIT_AREA_START_SLOT, 50_176);
    // D23 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」：768 MiB 的环要 3 GiB 以上的盘，装置取 4 GiB。
    assert_eq!(DEVICE_BYTES, 4_294_967_296);
    assert!(std::hint::black_box(JOURNAL_RING_BYTES * 4 <= DEVICE_BYTES), "环 ≤ 设备容量 ÷ 4");
    assert_eq!(UNIT_AREA_SLOTS, 211_968, "4 GiB 镜像 262144 槽减去固定结构占的 50176 槽");
    assert_eq!(UNIT_AREA_SLOTS / CLUSTER_SEGMENT_SLOTS, 3312, "单元区整 3312 个 64 槽聚簇段");
}

#[test]
fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
    let BuiltPool { output, .. } = built_pool();
    assert_eq!(output.allocation_records.len(), 28, "每盘 14 条：m1 m2 与 t1–t12（deadlist 无节点、不占落点，第七节 B4）");
    assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count(), 2, "mkfs 的 m1 分配代 0；m2 被 A 换下、释放代 3");
    assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(3)).count(), 26, "A 的十二个落点各两盘 24 条，加 m2 那两条（已释放、释放代 3）");
    assert_eq!(output.allocation_records.iter().filter(|record| record.is_released).map(|record| record.slot.0).collect::<Vec<_>>(), vec![50178, 50178], "只有 m2 已释放");
    assert_eq!(output.allocation_records[0].key_bytes().len(), 10);
    let slots_per_device: Vec<u64> = output.allocation_records.iter().filter(|record| record.device == DeviceIdentity(0)).map(|record| record.slot.0).collect();
    assert_eq!(slots_per_device, vec![50176, 50178, 50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252]);
    // D5 已定项 8：两盘时 **15 行**，seq 一律 1、代一律 3——这一项不随分配记录树的节点数变。
    assert_eq!(output.accounting_entries.len(), 15);
    assert!(output.accounting_entries.iter().all(|entry| entry.sequence == 1 && entry.generation == CheckpointTxg(3)));
    let value_of = |statistic: u16| output.accounting_entries.iter().find(|entry| entry.statistic == statistic).expect("统计量").value;
    let rows_of = |statistic: u16| output.accounting_entries.iter().filter(|entry| entry.statistic == statistic).count();
    assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES), 278_528, "每盘 17 槽 × 16384（第七节 B14「allocated_bytes」）");
    assert_eq!(value_of(STATISTIC_FREE_BYTES), 3_472_605_184, "(211968 − 17) × 16384（第七节 B14「free_bytes」）");
    assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES) + value_of(STATISTIC_FREE_BYTES), UNIT_AREA_SLOTS * SLOT_BYTES);
    assert_eq!(value_of(STATISTIC_EMPTY_CLUSTER_SEGMENTS), 3310, "3312 个 64 槽段里两个被占");
    assert_eq!(value_of(STATISTIC_FRAGMENTATION_RUNS), 4, "空闲 run：[50179]、[50182, 50239]、[50241]、[50253, 单元区末]（第七节 B14）");
    assert_eq!(value_of(STATISTIC_INODE_WATERMARK), 2);
    // 准入不等式要的四项 day-1 各写一行：不可回收、待删占用两项 0（读不到行是坏账，读到 0 是真的 0），
    // 已承诺预留照 D3（空间分配） 已定项 2 的式子（第 25 次登记第七节 7.2 A-T1：2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1) = 196 760），
    // defer 待释放 = 被 A 换下的 mkfs 树表 1 槽。
    for statistic in [STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_PENDING_DELETE_BYTES] {
        assert_eq!(value_of(statistic), 0);
    }
    assert_eq!(value_of(STATISTIC_COMMITTED_RESERVATION_BYTES), 196_760, "A-T1：副本数 2 × ((2 + H_墓碑 0 + 1 + 2 + H_退役 0 + 1) × 16384 + 41 × n_live 1 + 35 × n_记录 1)");
    assert_eq!(value_of(STATISTIC_DEFER_QUEUE_BYTES), SLOT_BYTES, "mkfs 那片第 0 版树表单元进 defer 队列");
    // 哪几项带设备维、哪几项是池级的：逐项数行数**并逐项核 key 里的设备段**——
    // 只数行数抓不到「两行都写成池级」（那也是两行），这一格是第七次跑的变异 M55 逼出来的。
    let devices_of = |statistic: u16| -> Vec<u32> {
        output.accounting_entries.iter().filter(|entry| entry.statistic == statistic).map(|entry| entry.device.0).collect()
    };
    for statistic in [STATISTIC_ALLOCATED_BYTES, STATISTIC_FREE_BYTES, STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_EMPTY_CLUSTER_SEGMENTS] {
        assert_eq!(rows_of(statistic), 2, "带设备维的统计量每盘一行");
        assert_eq!(devices_of(statistic), vec![0, 1], "带设备维的统计量，key 的设备段写真实设备号");
    }
    for statistic in [STATISTIC_PENDING_DELETE_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_INODE_WATERMARK] {
        assert_eq!(rows_of(statistic), 1, "池级统计量一行");
        assert_eq!(devices_of(statistic), vec![STATISTIC_NO_DEVICE_DIMENSION], "池级统计量的设备段取保留值");
    }
    let inode_watermark_row = output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK).expect("水位");
    assert_eq!(inode_watermark_row.tree, TreeIdentifier(TREE_IDENTIFIER_INODE), "第 12 项带树维：树 ID 就是可写头的 inode 树");
    assert_eq!(inode_watermark_row.device, DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION), "不带设备维的设备段取 0xFFFF_FFFF");
    assert_eq!(output.root.tree_identifier_watermark, 65549, "最后一棵退役树之后水位 65549（7.2 A-T4）");
    assert_eq!(output.root.rollback_floor, CheckpointTxg(0));
    assert_eq!(output.root.mapping_root.head.birth_tree, TreeIdentifier(TREE_IDENTIFIER_MAPPING), "中央映射树的根住根记录");
}
