use super::*;

#[test]
/// 29 次写、三道池屏障各按设备记两步、1 道 FUA（E142 第十九次跑登记 7.4 C11、D5）——
/// 12 个单元 × 2 盘 = 24（t1..t12）→ 屏障 → 记录 × 2 → 屏障 → 根槽 FUA → 系统配置槽 × 2 → 屏障（D16 已定项 7）。
fn transaction_issues_29_writes_3_pool_barriers_1_fua_in_the_settled_order() {
    let BuiltPool { recording, output, warm_up_operation_count, .. } = built_pool();
    let operations = &recording.operations[warm_up_operation_count..];
    let steps: Vec<&'static str> = operations.iter().map(|operation| RecordedStepKind::of(operation).tag()).collect();
    assert_eq!(steps.iter().filter(|tag| **tag != "barrier").count(), 29);
    assert_eq!(steps.iter().filter(|tag| **tag == "barrier").count(), 6);
    assert_eq!(steps.iter().filter(|tag| **tag == "root_record_fua").count(), 1);
    assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count(), 1, "FUA 只由步骤种类决定，只有根槽那一步是");
    assert_eq!(&steps[24..26], ["barrier", "barrier"]);
    assert_eq!(&steps[26..28], ["journal_record", "journal_record"]);
    assert_eq!(&steps[28..30], ["barrier", "barrier"]);
    assert_eq!(steps[30], "root_record_fua");
    assert_eq!(&steps[31..], ["system_configuration_slot", "system_configuration_slot", "barrier", "barrier"]);
    assert_eq!(output.record.named.len(), 12, "t1..t12（第七节 B6「named=12」）");
    assert_eq!(output.record.named.iter().map(|named| named.locations.len()).sum::<usize>(), 24, "第七节 B6「location_entries=24」");
    // D3 已定项 10 ⑤：t1 用户数据取不在开放聚簇段里的最低 32768 对齐空槽对；
    // t2..t12 从开放段 [50240, 50304) 按「树 ID 升序、树内先叶后根、映射树倒数第二、树表最末」bump（分配记录树内按
    // 「先叶后根、同层设备升序」），而 t3 是码 3 容器、要起点 32768 对齐 ⇒ 跳过 50241（第七节 B2）。
    let slots: Vec<u64> = output.units_by_slot.iter().map(|(slot, _, _)| slot.0).collect();
    assert_eq!(slots, vec![50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252]);
    assert_eq!(SLOT_INODE_LEAF % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "码 3 容器起点也 32768 对齐");
    assert!(!slots.contains(&SLOT_SKIPPED_BY_ALIGNMENT), "槽 50241 没人占");
    assert_eq!(SLOT_DATA_UNIT % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "数据单元起点 32768 对齐");
    assert!(std::hint::black_box(SLOT_DATA_UNIT < OPEN_CLUSTER_SEGMENT_START_SLOT), "用户数据不进开放聚簇段");
    assert_eq!(OPEN_CLUSTER_SEGMENT_START_SLOT % CLUSTER_SEGMENT_SLOTS, 0, "开放段 64 槽对齐");
    for slot in [
        SLOT_EXTENT_ROOT,
        SLOT_INODE_LEAF,
        SLOT_INODE_ROOT,
        SLOT_ALLOCATION_LEAF_DEVICE_0,
        SLOT_ALLOCATION_LEAF_DEVICE_1,
        SLOT_ALLOCATION_INTERNAL_DEVICE_0,
        SLOT_ALLOCATION_INTERNAL_DEVICE_1,
        SLOT_ALLOCATION_ROOT,
        SLOT_ACCOUNTING_ROOT,
        SLOT_MAPPING_ROOT,
        SLOT_TREE_TABLE_FIRST_PUBLISH,
    ] {
        assert!((OPEN_CLUSTER_SEGMENT_START_SLOT..OPEN_CLUSTER_SEGMENT_START_SLOT + CLUSTER_SEGMENT_SLOTS).contains(&slot), "提交内生块住开放聚簇段");
    }
}
