

// ───── b30 攻方腿 M2 探针（接在 pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs
// 的副本后面）：同一份坏盘（走读跟随的指针头部算法类型 / 压缩算法码 / 压后长度不是第一版的取值，MAC / nonce 与 extent 偏移仍 0），
// core 冷启动恢复拒；池级 checker 报什么 ─────
fn b30_opus_report(label: &str, image: &singlefs_harness::memory_pool::MemoryPool) {
    let verdicts = singlefs_checker::walk::check_pool_image(image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, singlefs_checker::image::InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    let i24 = verdicts.iter().find(|(name, _)| *name == "I-2.4").map(|(_, verdict)| format!("{verdict:?}"));
    println!(
        "B30ALG {label} checker_violated={violated:?} I-2.4={i24:?} core_recovery={:?}",
        recovery_failure(image)
    );
}

#[test]
fn b30_opus_pool_checker_on_encryption_and_compression_fields() {
    for change in [
        PointerHeadFieldChange::AlgorithmTypeToOne,
        PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
        PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
        PointerHeadFieldChange::FirstMacByteToOne,
    ] {
        // 上段叶条目内联的数据指针
        let mut image = build_pool("b30-opus-alg-inline").memory_pool();
        let ExtentUpperLeafTarget::InlineDataUnit(data_pointer) = first_upper_leaf_entry(&image).target else {
            panic!("内联");
        };
        let extent_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
        change_a_pointer_head_field_inside_the_unit(
            &mut image,
            the_slot_of(&extent_root),
            node_bytes(),
            &data_pointer_bytes(&data_pointer),
            change,
        );
        b30_opus_report(&format!("inline_data_pointer {change:?}"), &image);
        // 树表里 inode 树那条条目的根指针（树表单元里）
        let mut image = build_pool("b30-opus-alg-tree-table").memory_pool();
        let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
        let tree_table = newest_root(&image).tree_table;
        change_a_pointer_head_field_inside_the_unit(
            &mut image,
            the_slot_of(&tree_table),
            node_bytes(),
            &node_pointer_bytes(&inode_root),
            change,
        );
        b30_opus_report(&format!("tree_table_inode_root {change:?}"), &image);
    }
}
