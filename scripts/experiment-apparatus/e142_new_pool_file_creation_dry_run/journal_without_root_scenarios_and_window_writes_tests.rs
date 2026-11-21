use e7_index_bench::usize_from_u64;
use super::*;

/// 第八节 G4 用：`model_window_writes_for_parameters` 取的窗口（暖机最后一次写之后到发布结束）
/// 在一盘几何上恰 13 次写——这是「一次发布调用自己发出的写数」，与暖机走没走过无关，
/// 与下一条测试（跳过暖机、mkfs 之后直接发布）数到的同一个 13 是同一个量的两种取法（第七节 B10）。
/// M126 的会红检查落在这里：改了窗口起点会把这个数改成 12 或漏最后一次写；这条函数原来没有
/// 任何单测覆盖（`model_window_writes_for_parameters` 是这次跑从 `run_variant_window_writes`
/// 改来专供 G4 用的，第十五次跑冻结的单测钉的是旧函数，没有跟着搬），第十六次跑在这里补上（真盲区）。
#[test]
fn model_window_writes_for_parameters_has_thirteen_writes_on_the_one_device_geometry() {
    let parameters = PoolParameters::control_one_device_no_barriers();
    let writes = model_window_writes_for_parameters(&parameters, &sample_file());
    assert_eq!(writes.len(), 13, "一盘几何：暖机最后一次写之后到发布结束共 13 次写");
}

/// P6（A8，E142 第十四次跑第五节 5.2）：读者读了标志与序号——十六格预期逐句照条款推出来。
/// S*：t9 之前（含 t9 两份）全部持久，t10 根槽与 t11 系统配置槽都没持久。
#[test]
fn journal_without_root_reader_outcomes_match_the_clause_table() {
    let flag_set_parameters = PoolParameters::settled_two_devices();
    let flag_cleared_parameters = flag_set_parameters.clone().with_last_of_publish_flag(0x00);
    let file_bytes = sample_file();

    let flag_set_row = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, None, false);
    assert_eq!(flag_set_row.primary_prefix_applied, 1, "读者甲：F 上 t9 施加 ⇒ 根推进到 (1,3)");
    assert_eq!(flag_set_row.clause_prefix_applied, 1, "锚 w4（带标志）⇒ 接 t9；t9 带标志、提交标记 1 ⇒ 施加，根推进到 (1,3)");
    assert!(!flag_set_row.clause_fatal);

    let flag_cleared_row = run_journal_without_root_scenario(&flag_cleared_parameters, &file_bytes, None, false);
    assert_eq!(flag_cleared_row.primary_prefix_applied, 1, "读者甲不看标志，H 上 t9 照样施加 ⇒ (1,3)");
    assert_eq!(flag_cleared_row.clause_prefix_applied, 0, "w4 不带标志 ⇒ 锚点读不出；t9 也不带标志 ⇒ 这次发布走不到末条，不施加，根停在 (1,2)");
    assert!(!flag_cleared_row.clause_fatal);

    let row_flag_03 = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, Some(journal_without_root_mutate_flag_0x03), false);
    assert_eq!(row_flag_03.primary_prefix_applied, 1);
    assert_eq!(row_flag_03.primary_readable, 3);
    assert_eq!(row_flag_03.clause_prefix_applied, 0, "位 0 之外有位为 1 ⇒ 当损坏（已定项 4），根停在 (1,2)");
    assert_eq!(row_flag_03.clause_readable, 2, "t9 当损坏，读得出的只剩 w1、w4");

    let row_ordinal_0 = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, Some(journal_without_root_mutate_ordinal_0), false);
    assert_eq!(row_ordinal_0.primary_prefix_applied, 1);
    assert_eq!(row_ordinal_0.primary_readable, 3);
    assert_eq!(row_ordinal_0.clause_prefix_applied, 0, "序号 0 ⇒ 当损坏（已定项 4），根停在 (1,2)");
    assert_eq!(row_ordinal_0.clause_readable, 2);

    let row_flag_00 = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, Some(journal_without_root_mutate_flag_0x00), false);
    assert_eq!(row_flag_00.primary_prefix_applied, 1);
    assert_eq!(row_flag_00.clause_prefix_applied, 0, "锚 w4 ⇒ 接 t9；t9 不带标志 ⇒ 不施加（已定项 17），根停在 (1,2)");

    let row_ordinal_2 = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, Some(journal_without_root_mutate_ordinal_2), false);
    assert_eq!(row_ordinal_2.primary_prefix_applied, 1);
    assert_eq!(row_ordinal_2.clause_prefix_applied, 0, "锚点 w4 读得出、t9 是下一次发布的首条而序号不是 1 ⇒ 断在 t9（已定项 4），根停在 (1,2)");

    let row_trailing = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, None, true);
    // 读者甲不看「带标志的那条之后还有没有记录」：t9（counter 3）施加之后，链继续接上新加的 counter 4 那条也施加 ⇒ 2 条。
    assert_eq!(row_trailing.primary_prefix_applied, 2, "读者甲逐条施加，t9 与新加的那条都施加，根仍推进到 txg 3");
    assert_eq!(row_trailing.clause_prefix_applied, 0, "t9 带标志而其后还有同 (1,3) 的记录 ⇒ 断在 t9、这次发布不施加（已定项 4），根停在 (1,2)");

    let row_txg_2 = run_journal_without_root_scenario(&flag_set_parameters, &file_bytes, Some(journal_without_root_mutate_txg_2), false);
    assert_eq!(row_txg_2.primary_prefix_applied, 0, "读者甲锚到 w4，t9 的 (1,2) 不在水位之上 ⇒ 不施加，根停在 (1,2)");
    assert!(row_txg_2.clause_fatal, "同一 (实例, txg) 两条带末条标志 ⇒ 盘坏了、停下（已定项 14「这一版的失败处置」）");
}

/// M88 的会红检查：H / F 相减的每一段都归到 w1 / w4 / 本次事务自己那条记录（今天 12 个单元 ⇒ t13）之一，
/// 一段都不落进 unregistered（`journal_record_name_for_offset` 漏掉暖机记录时这条会红）。
#[test]
fn journal_header_flag_diff_segments_all_map_to_a_known_record() {
    let flag_set_parameters = PoolParameters::settled_two_devices();
    let flag_cleared_parameters = flag_set_parameters.clone().with_last_of_publish_flag(0x00);
    let (flag_cleared_image, _) = run_full_pipeline(&flag_cleared_parameters, NEW_POOL_FILE_CREATION_TXG, &sample_file());
    let (flag_set_image, flag_set_output) = run_full_pipeline(&flag_set_parameters, NEW_POOL_FILE_CREATION_TXG, &sample_file());
    let journal_label = format!("t{}", flag_set_output.units_by_slot.len() + 1);
    assert_eq!(journal_label, "t13", "今天写清单 12 个单元（第七节 B23），本次事务自己的记录是 t13");
    let differences = diff_pools(&flag_cleared_image, &flag_set_image);
    assert!(!differences.is_empty(), "P1/P2 的旋钮读到了才谈得上这条检查");
    for segment in &differences {
        let (record_name, _) = journal_record_name_for_offset(segment.offset, &journal_label).expect("H/F 的差异只可能落在 w1/w4/本次事务那条记录的头里");
        assert!(["w1".to_string(), "w4".to_string(), journal_label.clone()].contains(&record_name), "段落在 {record_name}，不是 w1/w4/{journal_label} 之一");
    }
}

#[test]
fn structure_sets_differing_by_one_are_reported_unstable() {
    let left: std::collections::BTreeSet<&str> = ["inode_leaf", "inode_root"].into_iter().collect();
    let right: std::collections::BTreeSet<&str> = ["inode_leaf", "inode_root", "mapping_root"].into_iter().collect();
    assert!(!structure_sets_match(&left, &right), "量 8 判据是「相等」，不是「相差 ≤ 1 段」（V6 判别力自证）");
}

#[test]
fn run_full_pipeline_writes_the_requested_change_count_into_the_inode_leaf_record() {
    // V1 变异（改动计数字面值改回 1）锚在 run_full_pipeline 内部的转发行：main() 自己那个独立调用点没有单测直接盖到，
    // M_A / M_B / G1 / G2 都经这个函数构造，锚在这里等价地测住了「装置真的按参数写出对应的 change_count」。
    for &value in &[1u64, NEW_POOL_FILE_CREATION_TXG, 4u64] {
        let (image, output) = run_full_pipeline(&PoolParameters::settled_two_devices(), value, &sample_file());
        let leaf = output.units_by_slot.iter().find(|(_, unit, _)| *unit == TransactionUnit::InodeLeaf).expect("t3 是 inode 叶");
        let parsed = parse_packed_unit(&leaf.2).expect("inode 叶自检要过");
        assert_eq!(InodeRecord::parse(&parsed.records[0]).expect("记录三段恒零要过").change_count, value);
        let record_field_offset = usize_from_u64(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) + 88;
        let sector = image.read(DeviceIdentity(0), leaf.0.device_offset(), usize_from_u64(PHYSICAL_BLOCK_BYTES));
        assert_eq!(&sector[record_field_offset..record_field_offset + 8], value.to_le_bytes().as_slice(), "盘上的字节要与请求的 change_count 一致");
    }
}

#[test]
fn run_full_pipeline_matches_the_manually_assembled_pool_for_the_same_change_count() {
    let BuiltPool { recording, .. } = built_pool();
    let (image, _) = run_full_pipeline(&PoolParameters::settled_two_devices(), NEW_POOL_FILE_CREATION_TXG, &sample_file());
    assert_eq!(recording.pool.devices.len(), image.devices.len());
    for device_index in 0..image.devices.len() {
        assert_eq!(recording.pool.devices[device_index].sectors, image.devices[device_index].sectors, "同参数同口径应逐字节相同");
    }
}
