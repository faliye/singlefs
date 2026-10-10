
// ───── 攻方原型（m3-batch57-code-r1 opus，K3）：拷进仓副本这份用例文件末尾，只在副本上编；冻结副本与主工作区不动 ─────

/// 中央映射树单个叶根里删掉 key 最大的那一条：条目数减一、声明长度（偏移 8，= 条目数 × 条目宽）跟着减、头里最大 key 换成新的末条，
/// 空出来的那一条清零（之后由调用方重封）。
fn opus_drop_the_last_entry_of_a_mapping_leaf_root(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([node[header_end - 4], node[header_end - 3]]));
    assert!(entry_count >= 2, "映射叶根至少两条");
    let last = entry_range_in_node(node, entry_count - 1);
    let new_last = entry_range_in_node(node, entry_count - 2);
    let entry_width = last.end - last.start;
    node[last].fill(0);
    let new_count = u16::try_from(entry_count - 1).expect("小");
    node[header_end - 4..header_end - 2].copy_from_slice(&new_count.to_le_bytes());
    let declared = u16::try_from((entry_count - 1) * entry_width).expect("小");
    node[8..10].copy_from_slice(&declared.to_le_bytes());
    let new_last_key = node[new_last.start..new_last.start + key_width].to_vec();
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&new_last_key);
}

/// 最新那条根的中央映射树根（单个叶根）删掉 key 最大的那一条，重封、写回两份落点，根记录里映射树根那两条位置的校验和换成新值。
fn opus_drop_the_last_mapping_entry_of_the_newest_root(devices: &mut Devices) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    assert_eq!(parse_index_node(&mapping_root).expect("映射树根解得开").level, 0, "单个叶根");
    opus_drop_the_last_entry_of_a_mapping_leaf_root(&mut mapping_root);
    reseal_index_node(&mut mapping_root);
    let checksum = write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 台阶行那个池（txg 4）上罩这一版树表单元那个槽的两条分配记录代改成 5（第 57 批 H4 用例的 ② 那一处，不动记账树）。
fn opus_allocation_records_of_the_tree_table_slot_after_the_root(devices: &mut Devices) {
    let (_, root) = newest_root_in_the_ring(devices);
    let tree_table_slot = root.tree_table.locations[0].slot;
    let changed = rewrite_allocation_records_of_the_newest_root(
        devices,
        |record| record.slot == tree_table_slot,
        |record| record.generation = ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT,
    );
    assert_eq!(changed, 2, "两盘同槽各一条");
}

fn opus_outcomes(name: &str, devices: &mut Devices) {
    let recovered = recover(&*devices, JournalPolicy::Consult).outcome;
    let mounted = match mount_writable(&parameters(), devices) {
        Ok(mounted) => format!("mounted txg {:?}", mounted.output.effective_root.checkpoint_txg),
        Err(refusal) => format!("{refusal:?}"),
    };
    eprintln!("OPUS_K3 image={name} recover={recovered:?}");
    eprintln!("OPUS_K3 image={name} writable_mount={mounted}");
}

/// 记账树单个叶根里删掉 key 最小的那一条（池级「已分配字节」第一盘那一行）：其余条目前移一条、末尾清零、条目数减一、声明长度跟着减、
/// 头里最小 key 换成新的首条。标签与代都不碰：数条目只因少一行对不上（16 − 1 台阶行 − 1 = 14 ≠ 15），与标签登记不登记无关。
fn opus_drop_the_first_entry_of_an_accounting_leaf_root(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([node[header_end - 4], node[header_end - 3]]));
    let first = entry_range_in_node(node, 0);
    let last = entry_range_in_node(node, entry_count - 1);
    let entry_width = first.end - first.start;
    node.copy_within(first.end..last.end, first.start);
    node[last.end - entry_width..last.end].fill(0);
    let new_count = u16::try_from(entry_count - 1).expect("小");
    node[header_end - 4..header_end - 2].copy_from_slice(&new_count.to_le_bytes());
    let declared = u16::try_from((entry_count - 1) * entry_width).expect("小");
    node[8..10].copy_from_slice(&declared.to_le_bytes());
    let new_first_key = node[first.start..first.start + key_width].to_vec();
    node[52..52 + key_width].copy_from_slice(&new_first_key);
}

fn opus_drop_the_first_accounting_row_of_the_newest_root(devices: &mut Devices) {
    rewrite_the_root_node_of_a_tree_of_the_newest_root(devices, TREE_KIND_ACCOUNTING, |node| {
        opus_drop_the_first_entry_of_an_accounting_leaf_root(node);
    });
}

/// K3：五张镜像各交只读恢复（冷走读）与可写挂载（重建），打出两条路各报哪个成员：
/// ① 只删映射树根最后一条（映射条目数对不上）；② 只把分配记录代改到根之后；③ ① + ②；
/// ⑤ 只删记账树根第一行（记账条目数对不上，不碰标签）；⑥ ② + ⑤。
#[test]
fn opus_k3_which_member_each_reader_reports() {
    let fresh = pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row;
    let mut mapping_only = fresh();
    opus_drop_the_last_mapping_entry_of_the_newest_root(&mut mapping_only);
    opus_outcomes("mapping-count-off", &mut mapping_only);

    let mut allocation_only = fresh();
    opus_allocation_records_of_the_tree_table_slot_after_the_root(&mut allocation_only);
    opus_outcomes("allocation-generation-after-the-root", &mut allocation_only);

    let mut both = fresh();
    opus_allocation_records_of_the_tree_table_slot_after_the_root(&mut both);
    opus_drop_the_last_mapping_entry_of_the_newest_root(&mut both);
    opus_outcomes("allocation-generation-and-mapping-count", &mut both);

    let mut accounting_row_dropped = fresh();
    opus_drop_the_first_accounting_row_of_the_newest_root(&mut accounting_row_dropped);
    opus_outcomes("accounting-row-dropped", &mut accounting_row_dropped);

    let mut allocation_and_accounting = fresh();
    opus_drop_the_first_accounting_row_of_the_newest_root(&mut allocation_and_accounting);
    opus_allocation_records_of_the_tree_table_slot_after_the_root(&mut allocation_and_accounting);
    opus_outcomes("allocation-generation-and-accounting-row-dropped", &mut allocation_and_accounting);
}

/// K3 ④：第 57 批 H4 用例那张（台阶行标签 13 + 分配记录代），它的造法里那一句前提断言单独放一条，前提不成立时只这一条红。
#[test]
fn opus_k3_batch57_h4_image() {
    let mut h4 = pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
    opus_outcomes("batch57-h4-image", &mut h4);
}
