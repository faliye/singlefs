
// ---------- P4：extent 树根指针写零的镜像与干净镜像逐条比 ----------
#[test]
fn probe_p4_what_else_changes_when_the_extent_root_pointer_is_zeroed() {
    let clean = build_pool("probe-p4b-clean").memory_pool();
    let mut image = clean.clone();
    mutate_unit(&mut image, TREE_TABLE, |bytes| {
        for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
            let entry = tree_table_entry_offset(index);
            let kind = u16::from_le_bytes([bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET], bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]]);
            if kind == TREE_KIND_EXTENT { bytes[entry + 14..entry + 100].fill(0); }
        }
    }, true);
    let (a, b) = (check_pool_image(&clean), check_pool_image(&image));
    for ((name, va), (_, vb)) in a.iter().zip(b.iter()) {
        if status_char(va) != status_char(vb) || (status_char(va) != 'H' && text_of(va) != text_of(vb)) {
            println!("PROBE-P4B {name} clean={} zeroed={} | {}", status_char(va), status_char(vb), short(&text_of(vb), 100));
        }
    }
    println!("PROBE-P4B done");
}
