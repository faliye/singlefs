
// ---------- P1：「环里读不到末条、退回先到的那一条」那条退路：H3 流（B 只由三条记录施加）最终镜像上抹掉 B 的末条（计数器 6）两份 ----------
fn final_image_of(base: &MemoryPool, writes: &[RetainedWrite]) -> MemoryPool {
    let mut image = base.clone();
    image.apply_writes(writes);
    image
}

#[test]
fn probe_p1_fallback_when_the_last_record_of_an_applied_only_version_is_not_readable() {
    let (base, writes, _segments) = h3_flow("probe-p1-fallback");
    let legal = final_image_of(&base, &writes);
    let mut legal_without_last = legal.clone();
    for device in 0..2u32 { erase_the_record_copy(&mut legal_without_last, 6, device); }
    let v = check_pool_image(&legal);
    println!("PROBE-P1F legal_final red={:?} partial={:?}", names_with(&v, 'V'), names_with(&v, 'P'));
    let v = check_pool_image(&legal_without_last);
    println!("PROBE-P1F legal_final_without_record6 red={:?} partial={:?}", names_with(&v, 'V'), names_with(&v, 'P'));
    // 第一条写成 A 的新根段（写侧违反已定项 15）：末条在时取末条（B 自己的树），末条不在时退回先到的第一条（A 的树）
    let mut stale = writes.clone();
    let root_of_a: Vec<u8> = stale.iter().filter(|w| w.kind == StepKind::RootRecordFua).filter_map(RetainedWrite::bytes).find(|b| u32::from_le_bytes(b[24..28].try_into().unwrap()) == 1 && u64::from_le_bytes(b[28..36].try_into().unwrap()) == 3).expect("A 的根").to_vec();
    let record_indexes: Vec<usize> = stale.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord && w.bytes().is_some_and(|b| u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1 && u64::from_le_bytes(b[26..34].try_into().unwrap()) == 4)).map(|(i, _)| i).collect();
    let counter_of = |w: &RetainedWrite| { let b = w.bytes().unwrap(); let mut c = [0u8; 8]; c[..6].copy_from_slice(&b[20..26]); u64::from_le_bytes(c) };
    let fields: [(usize, usize, usize); 7] = [(0, 36, 86), (86, 256, 86), (172, 122, 8), (180, 130, 8), (188, 170, 86), (274, 342, 86), (360, 20, 4)];
    for index in &record_indexes {
        if counter_of(&stale[*index]) != 4 { continue; }
        let WrittenContents::Bytes(bytes) = &mut stale[*index].contents else { panic!() };
        for (in_segment, in_root, length) in fields { bytes[99 + in_segment..99 + in_segment + length].copy_from_slice(&root_of_a[in_root..in_root + length]); }
        let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
    }
    rechain_record_writes(&mut stale, &record_indexes);
    let stale_final = final_image_of(&base, &stale);
    let v = check_pool_image(&stale_final);
    println!("PROBE-P1F stale_first_final red={:?} partial={:?}", names_with(&v, 'V'), names_with(&v, 'P'));
    let mut stale_without_last = stale_final.clone();
    for device in 0..2u32 { erase_the_record_copy(&mut stale_without_last, 6, device); }
    let v = check_pool_image(&stale_without_last);
    println!("PROBE-P1F stale_first_final_without_record6 red={:?} partial={:?}", names_with(&v, 'V'), names_with(&v, 'P'));
    println!("PROBE-P1F core_outcome(stale_first_final)={}", core_outcome(&stale_final));
    println!("PROBE-P1F core_outcome(stale_first_final_without_record6)={}", core_outcome(&stale_without_last));
}
