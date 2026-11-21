use e7_index_bench::usize_from_u64;
use super::*;

/// M87 的会红检查：`explain_journal_record_offset` 对偏移 87/91/95 的标签要按 D23 已定项 4
/// 今天的字段表（本次发布内序号/反向链/载荷校验和），不是退回头 307 布局的旧标签；
/// 第二十一次跑再钉新根段后三项与挪位之后的 fsid、MAC（7.2 B6：287、373、459、463、471，M220 的会红检查落在这里）。
#[test]
fn explain_journal_record_offset_labels_match_the_487_byte_field_table() {
    let (_, label) = explain_journal_record_offset(&[], 87);
    assert!(label.contains("ordinal_within_publish"), "偏移 87 是本次发布内序号，标签是「{label}」");
    let (_, label_at_91) = explain_journal_record_offset(&[], 91);
    assert!(label_at_91.contains("back_chain"), "偏移 91 是反向链，标签是「{label_at_91}」");
    let (_, label_at_95) = explain_journal_record_offset(&[], 95);
    assert!(label_at_95.contains("载荷校验和"), "偏移 95 是载荷校验和，标签是「{label_at_95}」");
    for (offset, expected) in [
        (271u64, "new_tree_identifier_watermark"),
        (279, "new_rollback_floor"),
        (287, "new_instance_table"),
        (372, "new_instance_table"),
        (373, "new_allocation_record_tree_root"),
        (458, "new_allocation_record_tree_root"),
        (459, "new_unmount_marker"),
        (462, "new_unmount_marker"),
        (463, "fsid"),
        (470, "fsid"),
        (471, "MAC"),
        (486, "MAC"),
    ] {
        let (explained, offset_label) = explain_journal_record_offset(&[], offset);
        assert!(explained, "偏移 {offset} 要归得了因");
        assert!(offset_label.contains(expected), "偏移 {offset} 的标签应含「{expected}」，是「{offset_label}」");
    }
    // 点名项数组从头末 487 起：偏移 487 是第 0 项（无单元表时标 `?`）的内偏移 0。
    let (_, label_at_487) = explain_journal_record_offset(&[], 487);
    assert!(label_at_487.contains("点名项 0"), "偏移 487 是第 0 个点名项，标签是「{label_at_487}」");
    let (_, label_at_543) = explain_journal_record_offset(&[], 487 + 56);
    assert!(label_at_543.contains("点名项 1"), "偏移 543 是第 1 个点名项，标签是「{label_at_543}」");
    // 实例表指针里两条位置条目的单元校验和：指针内偏移 60 与 74 ⇒ 记录内 347、361；分配记录树根指针同理 433、447。
    assert!(explain_journal_record_offset(&[], 347).1.contains("第 0 条位置条目单元校验和"));
    assert!(explain_journal_record_offset(&[], 361).1.contains("第 1 条位置条目单元校验和"));
    assert!(explain_journal_record_offset(&[], 433).1.contains("第 0 条位置条目单元校验和"));
    assert!(explain_journal_record_offset(&[], 447).1.contains("第 1 条位置条目单元校验和"));
}

#[test]
fn classify_offset_finds_the_inode_leaf_change_count_field_and_the_root_record_checksum() {
    let BuiltPool { output, .. } = built_pool();
    let parameters = PoolParameters::settled_two_devices();
    let (region, slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let root_device = parameters.region_devices[usize_from_u64(region)].0;
    let root_offset = ring_slot_offset(region, slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    let leaf_base = SlotNumber(SLOT_INODE_LEAF).device_offset().0;
    assert_eq!(classify_offset(&catalog, 0, leaf_base + 224), Some(("inode_leaf", 224)), "记录区 136 起 + 记录内偏移 88 = 224");
    assert_eq!(classify_offset(&catalog, 0, leaf_base), Some(("inode_leaf", 0)));
    assert_eq!(classify_offset(&catalog, root_device, root_offset + 138), Some(("root_record", 138)), "根记录自证校验和的结构内偏移");
    assert_eq!(classify_offset(&catalog, 0, 999_999_999_999), None, "远处的偏移落不进任何候选结构");
}

#[test]
fn explain_unit_offset_finds_the_change_count_field_and_the_checksum_regions_but_not_the_common_prefix() {
    let record_start = PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES;
    let (explained, reason) = explain_unit_offset(TransactionUnit::InodeLeaf, record_start + 88);
    assert!(explained);
    assert!(reason.contains("改动计数"), "reason={reason}");
    assert!(explain_unit_offset(TransactionUnit::InodeLeaf, 15).0, "偏移 15 落在头校验和 [10,42)");
    assert!(explain_unit_offset(TransactionUnit::InodeLeaf, 89).0, "偏移 89 是码 3 载荷 CRC");
    assert!(!explain_unit_offset(TransactionUnit::InodeLeaf, 5).0, "偏移 5 是共同前缀，不在头校验和/载荷CRC/记录区里，量 3 该记 gap");
}

#[test]
fn explain_inode_root_offset_finds_the_two_location_checksums_that_point_at_the_inode_leaf() {
    let (explained_0, reason_0) = explain_unit_offset(TransactionUnit::InodeRoot, 225);
    let (explained_1, reason_1) = explain_unit_offset(TransactionUnit::InodeRoot, 239);
    assert!(explained_0 && reason_0.contains("位置条目单元校验和"), "reason_0={reason_0}");
    assert!(explained_1 && reason_1.contains("位置条目单元校验和"), "reason_1={reason_1}");
}

#[test]
fn change_count_bullets_are_all_covered_only_when_all_six_offsets_are_hit() {
    let mut bullets = ChangeCountBullets::default();
    assert!(!bullets.all_covered());
    bullets.record("inode_leaf", 136 + 88, 1);
    bullets.record("inode_leaf", 15, 4);
    bullets.record("inode_root", 225, 4);
    bullets.record("inode_root", 92, 4);
    bullets.record("tree_table", 50, 4);
    assert!(!bullets.all_covered(), "根记录那一格还没打到，六处不能全过");
    bullets.record("root_record", 150, 4);
    assert!(bullets.all_covered());
}
