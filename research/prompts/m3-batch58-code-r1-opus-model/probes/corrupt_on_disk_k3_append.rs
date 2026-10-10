
// ─── 攻方原型（m3-batch58-code-r1 opus）K3：接在副本里这份用例文件末尾，用它自己的造法函数，只打印、不断言 ───


fn opus_b58_clone(devices: &Devices) -> Devices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE);
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 一张镜像交给只读恢复（冷走读）与可写挂载各报什么（可写挂载在副本上起，镜像先拷一份，原镜像不动）。
fn opus_b58_k3_report(name: &str, devices: &Devices) {
    let recovered = match recover(devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead root={root:?} content_bytes={}", content.len()),
        other => format!("{other:?}"),
    };
    let mut copy = opus_b58_clone(devices);
    let mounted = match mount_writable(&parameters(), &mut copy) {
        Ok(mounted) => format!(
            "mounted instance={:?} txg={:?}",
            mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg
        ),
        Err(refusal) => format!("refused {refusal:?}"),
    };
    eprintln!("OPUS_K3 image={name} recover={recovered}");
    eprintln!("OPUS_K3 image={name} writable={mounted}");
}

fn opus_b58_drop_the_smallest_accounting_row(devices: &mut Devices) {
    rewrite_the_root_node_of_a_tree_of_the_newest_root(devices, TREE_KIND_ACCOUNTING, |node| {
        let dropped = AccountingEntry::parse(&drop_one_entry_of_an_index_leaf_root(node, 0)).expect("解得开");
        eprintln!("OPUS_K3 j2-dropped-row {dropped:?}");
    });
}

fn opus_b58_move_this_versions_watermark_row_after_the_root(devices: &mut Devices) {
    rewrite_the_root_node_of_a_tree_of_the_newest_root(devices, TREE_KIND_ACCOUNTING, |node| {
        rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
            node,
            |entry| entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW,
            |row| row.generation = WATERMARK_ROW_GENERATION_AFTER_THE_ROOT,
        );
    });
}

#[test]
fn opus_k3_b58_images_on_both_paths() {
    let base = pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    let (_, root) = newest_root_in_the_ring(&base);
    eprintln!("OPUS_K3 base-root txg={:?}", root.checkpoint_txg);
    opus_b58_k3_report("base", &base);

    let mut only_allocation = opus_b58_clone(&base);
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut only_allocation);
    opus_b58_k3_report("only-allocation-after-root", &only_allocation);

    let mut only_accounting_row_dropped = opus_b58_clone(&base);
    opus_b58_drop_the_smallest_accounting_row(&mut only_accounting_row_dropped);
    opus_b58_k3_report("only-smallest-accounting-row-dropped", &only_accounting_row_dropped);

    let mut only_mapping_dropped = opus_b58_clone(&base);
    drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(&mut only_mapping_dropped);
    opus_b58_k3_report("only-largest-mapping-entry-dropped", &only_mapping_dropped);

    let mut only_watermark_row = opus_b58_clone(&base);
    opus_b58_move_this_versions_watermark_row_after_the_root(&mut only_watermark_row);
    opus_b58_k3_report("only-this-versions-watermark-row-after-root", &only_watermark_row);

    opus_b58_k3_report(
        "j2-accounting-count-off-and-allocation",
        &pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off(),
    );
    opus_b58_k3_report(
        "j1a-mapping-count-off-and-allocation",
        &pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off(),
    );
    opus_b58_k3_report(
        "j1b-watermark-row-and-allocation",
        &pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root(),
    );
}
