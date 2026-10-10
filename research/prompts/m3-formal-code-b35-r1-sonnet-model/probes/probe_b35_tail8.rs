
// ---------- P3：对拍用例 4（新池新建文件那次发布的根与它的记录里树表指针全零）的 CPU 一侧 ----------
#[test]
fn probe_flow4_new_pool_publish_with_the_tree_table_pointer_of_the_root_and_its_records_all_zero() {
    let pool = build_pool("probe-flow4");
    let ops = pool.retained_operations();
    let (mut writes, segments, _) = writes_and_segments_with_stream_indexes(&ops[pool.mkfs_operation_count..], &geometry());
    let root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let (instance, txg) = { let b = writes[root_index].bytes().unwrap(); (u32::from_le_bytes(b[24..28].try_into().unwrap()), u64::from_le_bytes(b[28..36].try_into().unwrap())) };
    let record_indexes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord && w.bytes().is_some_and(|b| u32::from_le_bytes(b[16..20].try_into().unwrap()) == instance && u64::from_le_bytes(b[26..34].try_into().unwrap()) == txg)).map(|(i, _)| i).collect();
    for index in &record_indexes {
        let WrittenContents::Bytes(bytes) = &mut writes[*index].contents else { panic!() };
        for b in &mut bytes[99..99 + 86] { *b = 0; }
        let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
    }
    {
        let WrittenContents::Bytes(bytes) = &mut writes[root_index].contents else { panic!() };
        for b in &mut bytes[36..36 + 86] { *b = 0; }
        let digest = wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
        bytes[138..170].copy_from_slice(&digest);
    }
    let states = crash_states(&pool.memory_pool_after_mkfs(), &writes, &segments, 0, 8);
    let order = singlefs_checker::image::IMPLEMENTED_INVARIANTS;
    let mut by_red: std::collections::BTreeMap<Vec<&str>, u64> = std::collections::BTreeMap::new();
    for (_, image) in &states {
        let verdicts = check_pool_image(image);
        let red: Vec<&str> = order.iter().copied().filter(|n| verdicts.iter().any(|(m, v)| m == n && status_char(v) == 'V')).collect();
        *by_red.entry(red).or_default() += 1;
    }
    println!("PROBE-FLOW4 records_of_the_publish={} states={} by_red_set={by_red:?}", record_indexes.len(), states.len());
}
