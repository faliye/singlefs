// ===== 攻方 m3-batch30-code-r1-opus 探针（只在副本，交回前删） =====
fn opus_fw_inode_blocks_violation(clean: &MemoryPool) -> MemoryPool {
    let mut v = clean.clone();
    mutate_unit(
        &mut v,
        INODE_LEAF,
        |b| {
            let o = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_BLOCKS_OFFSET;
            let x = get_u64(b, o);
            set_u64(b, o, x + 1);
        },
        true,
    );
    v
}

#[test]
fn opus_probe_fw_full_verdicts() {
    let clean = build_pool("opus-probe-inode-blocks").memory_pool();
    let with = opus_fw_inode_blocks_violation(&clean);
    eprintln!(
        "OPUSPROBE only-violation: {:?}",
        violated_invariant_set(&check_pool_image(&with))
    );
    for li in 0..2usize {
        let img = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            &with,
            TREE_KIND_INODE,
            li,
            &leave_the_entry_as_it_is,
        );
        eprintln!(
            "OPUSPROBE FW code2-copy-at-location {li}: {:?}",
            violated_invariant_set(&check_pool_image(&img))
        );
    }
    // 对照：干净池只做 H-V、不放违例
    for li in 0..2usize {
        let img = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            &clean,
            TREE_KIND_INODE,
            li,
            &leave_the_entry_as_it_is,
        );
        eprintln!(
            "OPUSPROBE HVonly code2-copy-at-location {li}: {:?}",
            violated_invariant_set(&check_pool_image(&img))
        );
    }
}
