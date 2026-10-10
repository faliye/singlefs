
/// 同一块盘上、一组记录写（下标）按计数器升序，后一条的反向链按前一条的记录头重算、重封。
fn rechain_record_writes(writes: &mut [RetainedWrite], record_indexes: &[usize]) {
    let mut by_counter: Vec<(u64, usize)> = record_indexes.iter().map(|i| { let b = writes[*i].bytes().unwrap(); let mut c = [0u8; 8]; c[..6].copy_from_slice(&b[20..26]); (u64::from_le_bytes(c), *i) }).collect();
    by_counter.sort();
    for device_filter in [0u32, 1u32] {
        let chain: Vec<usize> = by_counter.iter().filter(|(_, i)| writes[*i].device.0 == device_filter).map(|(_, i)| *i).collect();
        for pair in chain.windows(2) {
            let mut zeroed: Vec<u8> = writes[pair[0]].bytes().unwrap()[..487].to_vec();
            zeroed[46..78].fill(0);
            let chain_value = crc32_castagnoli(&zeroed);
            let WrittenContents::Bytes(bytes) = &mut writes[pair[1]].contents else { panic!() };
            bytes[91..95].copy_from_slice(&chain_value.to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
            bytes[46..78].copy_from_slice(&digest);
        }
    }
}

/// 对拍用例 2 的 CPU 一侧：H3 流上 B 的第一条记录（计数器 4，两块盘）新根段七段换成 A 那条根（实例 1、txg 3）的，重封、重链。
#[test]
fn probe_h3_stale_first_record_of_b() {
    let (base, mut writes, segments) = h3_flow("probe-h3-stale-first");
    let root_of_a: Vec<u8> = writes.iter().filter(|w| w.kind == StepKind::RootRecordFua).filter_map(RetainedWrite::bytes).find(|b| u32::from_le_bytes(b[24..28].try_into().unwrap()) == 1 && u64::from_le_bytes(b[28..36].try_into().unwrap()) == 3).expect("A 的根").to_vec();
    let record_indexes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord && w.bytes().is_some_and(|b| u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1 && u64::from_le_bytes(b[26..34].try_into().unwrap()) == 4)).map(|(i, _)| i).collect();
    let counter_of = |w: &RetainedWrite| { let b = w.bytes().unwrap(); let mut c = [0u8; 8]; c[..6].copy_from_slice(&b[20..26]); u64::from_le_bytes(c) };
    let first_counter = record_indexes.iter().map(|i| counter_of(&writes[*i])).min().unwrap();
    let fields: [(usize, usize, usize); 7] = [(0, 36, 86), (86, 256, 86), (172, 122, 8), (180, 130, 8), (188, 170, 86), (274, 342, 86), (360, 20, 4)];
    for index in &record_indexes {
        if counter_of(&writes[*index]) != first_counter { continue; }
        let WrittenContents::Bytes(bytes) = &mut writes[*index].contents else { panic!() };
        for (in_segment, in_root, length) in fields { bytes[99 + in_segment..99 + in_segment + length].copy_from_slice(&root_of_a[in_root..in_root + length]); }
        let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
    }
    rechain_record_writes(&mut writes, &record_indexes);
    let states = crash_states(&base, &writes, &segments, 0, 6);
    let mut union_red = std::collections::BTreeSet::new();
    for (label, image) in &states {
        let verdicts = check_pool_image(image);
        for n in names_with(&verdicts, 'V') { union_red.insert(n); }
        println!("SIG h3s {label} {}", full_signature(image));
    }
    println!("PROBE-H3S states={} union_of_red_invariants_over_all_states={union_red:?}", states.len());
}
