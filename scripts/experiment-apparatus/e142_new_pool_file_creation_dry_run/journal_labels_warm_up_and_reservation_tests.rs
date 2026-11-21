use super::*;

/// 暖机（D16 已定项 8）：两次空发布，根落区域 1 与区域 2（分住两块盘），jsn 1、2 不点名任何单元，新池新建文件从 txg 3 起。
#[test]
fn warm_up_writes_two_empty_publishes_covering_both_devices() {
    let BuiltPool { recording, output, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
    let operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
    let writes = operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let fua_writes: Vec<&WriteRequest> = operations.iter().filter_map(|operation| match operation { RecordedOperation::Write(write) if write.is_fua() => Some(write), RecordedOperation::Write(_) | RecordedOperation::Barrier(_) => None }).collect();
    assert_eq!(writes, 10, "每次空发布 2 条记录 + 1 个根 + 2 个系统配置槽");
    assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier(_))).count(), 10, "每次空发布三道池屏障 × 2 盘，第二次开头那道与第一次发布尾那道连着、两块盘各并掉一步（登记 R19E-4 第 4 条 barrier_steps=10）");
    assert_eq!(fua_writes.len(), 2);
    let parameters = PoolParameters::settled_two_devices();
    assert_eq!(RING_REGION_DEVICES, [0, 1, 0], "根环区域归属第一版写死（D2 已定项 7，2026-09-14 用户定案）");
    assert_ne!(fua_writes[0].device, fua_writes[1].device, "两个暖机根要落在两块盘上");
    assert_eq!((parameters.region_devices[1], parameters.region_devices[2]), (fua_writes[0].device, fua_writes[1].device));
    assert_eq!(output.root.checkpoint_txg, CheckpointTxg(3));
    assert_eq!(output.record.counter, JournalCounter(3));
    assert_eq!(NEW_POOL_FILE_CREATION_TXG, WARM_UP_EMPTY_PUBLISHES + 1, "新池新建文件紧跟暖机之后");
    assert_eq!(output.record.instance, InstanceGeneration(1), "第一次可写挂载取的实例代号是 1（D23 已定项 16）");
    // D22 已定项 16：世代号 mkfs 1（两槽同写）、取号 2（槽 0）、w3 3（槽 1）、w6 4（槽 0）、t11 5（槽 1）。
    assert_eq!(system_configuration_write_for_publish(CheckpointTxg(1)), (3, 1));
    assert_eq!(system_configuration_write_for_publish(CheckpointTxg(2)), (4, 0));
    assert_eq!(system_configuration_write_for_publish(CheckpointTxg(3)), (5, 1));
    // D23 已定项 19 ②：环上计数器为 1 的那条反向链恒 0，之后每条罩前一条的整个头。
    let records = scan_journal(&recording.pool, unit_fsid(&FIXED_FSID));
    let chains: Vec<u32> = records.values().map(|record| record.back_chain).collect();
    assert_eq!(chains[0], 0, "jsn 1 的反向链恒 0");
    assert_ne!(chains[1], 0);
    assert_ne!(chains[2], 0);
    assert_eq!(chains[1], journal_back_chain(&records[&(InstanceGeneration(1), JournalCounter(1))].to_bytes()));
    assert_eq!(chains[2], journal_back_chain(&records[&(InstanceGeneration(1), JournalCounter(2))].to_bytes()));
    assert_eq!(records.values().filter(|record| record.transaction == TransactionNumber(0) && record.is_commit).count(), 2, "空发布事务号 0、提交标记 1（D23 已定项 19 ①）");
}

/// M156 的会红检查：主臂三条 journal 记录的角色名——第三条必须用写清单给的动态标签，退回旧编号
/// （分配记录树按位置寻址之前只有 8 个单元时代的 `t9`）就是 M156 描述的破坏。
#[test]
fn journal_header_record_labels_third_entry_uses_the_dynamic_write_list_label() {
    let labels = journal_header_record_labels("t13");
    assert_eq!(labels, [("w1".to_string(), 1u64), ("w4".to_string(), 2u64), ("t13".to_string(), 3u64)]);
}

/// 已承诺预留（标签 6）的式子，绝对值取第 25 次登记（`research/prompts/e142-r25-prereg.md`）第七节 7.2 A-T1、A-T2，由
/// `anchors_r25.py` 独立算出，不取任何一次产物的输出（M221–M227、M238、M241 的会红检查落在这里）。
#[test]
fn committed_reservation_formula_reproduces_the_space_allocation_clause_anchors() {
    let main_point = committed_reservation_bytes(2, 0, 0, 1, 1, true);
    assert_eq!(main_point, 196_760, "A-T1：2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1)");
    assert_eq!(main_point.to_le_bytes(), [0x98, 0x00, 0x03, 0, 0, 0, 0, 0], "A-T1：小端 8 字节 9800030000000000");
    assert_eq!(tombstone_prepayment_bytes_per_live_object(), 41, "41 = ⌈2 × 16384 ÷ 815⌉（向下取整是 40）");
    assert_eq!(retirement_prepayment_bytes_per_inode_record(), 35, "A-T2：35 = ⌈32768 ÷ 959⌉（向下取整是 34）");
    // H：零根读 0（M226 读成 1 会红）；两项预付在边界上先取整（M222、M241）。
    assert_eq!(tombstone_tree_internal_levels(&NodePointer::empty_root()), 0, "零根与只有根时 H_墓碑 = 0");
    assert_eq!(retirement_tree_internal_levels(&NodePointer::empty_root()), 0, "零根与只有根时 H_退役 = 0");
    assert_eq!((2 * SLOT_BYTES) % TOMBSTONE_RECORDS_PER_CONTAINER, 168, "32768 = 40 × 815 + 168：除不尽，取整方向才有区别");
    assert_eq!((2 * SLOT_BYTES) % RETIREMENT_RECORDS_PER_CONTAINER, 162, "32768 = 34 × 959 + 162：除不尽，取整方向才有区别");
}

/// `layout/03`「退役容器」那一行：一个 32 KiB 容器去掉 136 字节的头，按 34 字节一条装 959 条（7.2 A-T2）。
#[test]
fn retirement_container_holds_959_records_of_34_bytes() {
    assert_eq!((2 * SLOT_BYTES - 136) / 34, RETIREMENT_RECORDS_PER_CONTAINER, "⌊(32768 − 136) ÷ 34⌋ = 959");
    assert_eq!(RETIREMENT_RECORDS_PER_CONTAINER, 959);
}

/// 7.2 A-T6 的判别值之一：副本 1 ⇒ 98 380。
#[test]
fn committed_reservation_with_one_replica_is_98380() {
    assert_eq!(committed_reservation_bytes(1, 0, 0, 1, 1, true), 98_380, "A-T6：副本 1");
}

/// 7.2 A-T6 的判别值之一：H_墓碑 = 1 ⇒ 229 528。
#[test]
fn committed_reservation_with_one_tombstone_tree_internal_level_is_229528() {
    assert_eq!(committed_reservation_bytes(2, 1, 0, 1, 1, true), 229_528, "A-T6：H_墓碑 1");
}

/// 7.2 A-T6 的判别值之一：H_退役 = 1 ⇒ 229 528（M239 去掉 H_退役 时在这一点上红，主点上与原式同值）。
#[test]
fn committed_reservation_with_one_retirement_tree_internal_level_is_229528() {
    assert_eq!(committed_reservation_bytes(2, 0, 1, 1, 1, true), 229_528, "A-T6：H_退役 1");
}

/// 7.2 A-T6 的判别值之一：n_live = n_记录 = 2 ⇒ 196 912。
#[test]
fn committed_reservation_with_two_live_objects_and_two_records_is_196912() {
    assert_eq!(committed_reservation_bytes(2, 0, 0, 2, 2, true), 196_912, "A-T6：n_live 2、n_记录 2");
}

/// 7.2 A-T6 的判别值之一：n_live = 1、n_记录 = 2 ⇒ 196 830（M240 把退役项挂到 n_live 上时在这一点上红，主点上与原式同值）。
#[test]
fn committed_reservation_with_one_live_object_and_two_records_is_196830() {
    assert_eq!(committed_reservation_bytes(2, 0, 0, 1, 2, true), 196_830, "A-T6：n_live 1、n_记录 2");
}

/// 7.2 A-T6 的判别值之一：树表里没有 `deleted_inodes` 条目的版本取 0，别的量取什么都是 0。
#[test]
fn committed_reservation_without_a_deleted_inodes_entry_is_zero() {
    assert_eq!(committed_reservation_bytes(2, 0, 0, 1, 1, false), 0, "A-T6：没有 deleted_inodes 条目");
    assert_eq!(committed_reservation_bytes(2, 5, 3, 7, 9, false), 0, "没有条目时别的五个量取什么都是 0");
}

/// 等价变异留档（登记第九节）：式子里 H_墓碑 与 H_退役 只相加，两者对调在所有输入上同值；
/// 在六个判别值的输入上逐个核一遍，对调之后相同。
#[test]
fn swapping_the_two_tree_internal_levels_leaves_the_committed_reservation_unchanged() {
    for (replicas, tombstone_levels, retirement_levels, live_objects, records, has_entry) in
        [(1u64, 0u64, 0u64, 1u64, 1u64, true), (2, 1, 0, 1, 1, true), (2, 0, 1, 1, 1, true), (2, 0, 0, 2, 2, true), (2, 0, 0, 1, 2, true), (2, 0, 0, 1, 1, false), (2, 3, 1, 4, 5, true)]
    {
        assert_eq!(
            committed_reservation_bytes(replicas, tombstone_levels, retirement_levels, live_objects, records, has_entry),
            committed_reservation_bytes(replicas, retirement_levels, tombstone_levels, live_objects, records, has_entry),
            "H_墓碑 与 H_退役 对调同值"
        );
    }
}

/// n_live 只数 nlink ≥ 1 的记录（M225 把每条记录都算上会红）：合成 nlink = 0、1、2 的记录字节。
#[test]
fn live_object_count_skips_inode_records_whose_link_count_is_zero() {
    let record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: CheckpointTxg(3), size: 3000, change_count: 3 };
    let linked = record.to_bytes();
    assert_eq!(inode_record_link_count(&linked), 1, "装置写的记录 nlink 恒为 1（偏移 36）");
    let mut unlinked = record.to_bytes();
    unlinked[INODE_RECORD_LINK_COUNT_OFFSET..INODE_RECORD_LINK_COUNT_OFFSET + 4].copy_from_slice(&0u32.to_le_bytes());
    let mut linked_twice = record.to_bytes();
    linked_twice[INODE_RECORD_LINK_COUNT_OFFSET..INODE_RECORD_LINK_COUNT_OFFSET + 4].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(live_object_count_of_inode_records(&[unlinked.clone()]), 0, "nlink = 0 的记录不算活对象");
    assert_eq!(live_object_count_of_inode_records(&[linked.clone(), unlinked]), 1);
    assert_eq!(live_object_count_of_inode_records(&[linked, linked_twice]), 2, "nlink = 2 也只算一个对象");
    assert_eq!(live_object_count_of_inode_records(&[]), 0);
}

/// n_记录 数全部记录，含 nlink = 0 的（D3 已定项 2 的退役项）：合成 nlink = 0、1 的记录字节。
#[test]
fn inode_record_count_includes_records_whose_link_count_is_zero() {
    let record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: CheckpointTxg(3), size: 3000, change_count: 3 };
    let linked = record.to_bytes();
    let mut unlinked = record.to_bytes();
    unlinked[INODE_RECORD_LINK_COUNT_OFFSET..INODE_RECORD_LINK_COUNT_OFFSET + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(record_count_of_inode_records(&[unlinked.clone()]), 1, "nlink = 0 的记录也算进 n_记录");
    assert_eq!(record_count_of_inode_records(&[linked, unlinked]), 2);
    assert_eq!(record_count_of_inode_records(&[]), 0);
}

/// 记账树单元 t10 里各行的字节偏移与标签（登记 7.2 A4，由 D8 已定项 11 与 `layout/01` 第五节独立推出）、A6 前三个区域的落点。
#[test]
fn accounting_unit_carries_the_committed_reservation_at_the_registered_byte_offsets() {
    let BuiltPool { output, .. } = built_pool();
    let (slot, unit_kind, unit) = &output.units_by_slot[9];
    assert_eq!((slot.0, *unit_kind), (50250, TransactionUnit::AccountingTree), "t10");
    assert_eq!(unit.len() as u64, NODE_BYTES);
    let label_at = |offset: usize| u16::from_le_bytes(unit[offset..offset + 2].try_into().expect("切了 2 字节"));
    let value_at = |offset: usize| u64::from_le_bytes(unit[offset..offset + 8].try_into().expect("切了 8 字节"));
    // 第 r 行条目起点 159 + 34 (r − 1)；key 22（标签 2 + 树 8 + 设备 4 + 代 8），value 紧接 key。
    assert_eq!((label_at(227), value_at(249)), (2, 3_472_605_184), "第 3 行：设备 0 的空闲字节，value [249, 257)");
    assert_eq!((label_at(261), value_at(283)), (2, 3_472_605_184), "第 4 行：设备 1 的空闲字节，value [283, 291)");
    assert_eq!((label_at(465), value_at(487)), (6, 196_760), "第 10 行：已承诺预留，标签在 465，value [487, 495)");
    assert_eq!(&unit[487..495], &[0x98, 0x00, 0x03, 0, 0, 0, 0, 0], "A-T1：盘上小端 9800030000000000");
    assert_eq!((label_at(567), value_at(589)), (11, 3310), "第 13 行：设备 0 的全空聚簇段数，value [589, 597)");
    assert_eq!((label_at(601), value_at(623)), (11, 3310), "第 14 行：设备 1 的全空聚簇段数，value [623, 631)");
    // A6：t10、t11、t12 的（设备内偏移，长度）；t13、t14 的落点。
    let offsets: Vec<u64> = output.units_by_slot[9..=11].iter().map(|(window_slot, _, _)| layout_slot_offset(window_slot.0)).collect();
    assert_eq!(offsets, vec![823_296_000, 823_312_384, 823_328_768]);
    assert_eq!(journal_record_offset(JournalCounter(3)).0, 16_785_408, "t13");
    assert_eq!(ring_slot_offset(0, 1).0, 1_052_672, "t14");
}
