
// ===== 第 36 批 R3：I-5.5 的几种判定与理由文字 =====
fn i55_reason(label: &str, image: &MemoryPool) {
    let v = verdict(image, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK);
    println!("PROBE-R3 {label} I-5.5={} {}", status_char(&v), text_of(&v));
}

#[test]
fn probe_r3_first_data_unit_reason_text_by_form() {
    i55_reason("first_file_clean", &build_pool("probe-r3-clean").memory_pool());
    i55_reason("mkfs_only", &format_pool("probe-r3-mkfs").memory_pool());
    let mut image = build_pool("probe-r3-extent-root-zero").memory_pool();
    mutate_unit(&mut image, TREE_TABLE, |bytes| {
        for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
            let entry = tree_table_entry_offset(index);
            let kind = u16::from_le_bytes([bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET], bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]]);
            if kind == TREE_KIND_EXTENT { bytes[entry + 14..entry + 100].fill(0); }
        }
    }, true);
    i55_reason("extent_root_zeroed", &image);
    println!("PROBE-R3 constant={}", singlefs_checker::walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT);
}
