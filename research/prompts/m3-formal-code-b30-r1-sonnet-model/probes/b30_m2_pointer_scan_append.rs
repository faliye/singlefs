
// ───────────── b30 正推腿 M2 探针：单元区里每条「像指针」的 86 / 88 字节，头部 extent 偏移与 MAC 第 0 字节各改 1，跑池级 checker ─────────────
// 找指针的办法：单元里任一处连着两条位置条目（设备身份 0 与 1、槽号相同、槽号 ≥ 单元区起点），前面 50 字节是指针头部。
// 结果每条指针一行 PTR：(extent 改 1 红不红 I-2.4, MAC 改 1 红不红 I-2.4)。
// (false, false)：这一条没人判（走读没跟随，或单元不可达）；(true, false)：只判 MAC 不判 extent 偏移（E1 那一类漏）；(false, true) 反过来。
#[derive(Debug)]
struct B30Pointer { unit_slot: u64, unit_class: u8, head_at: usize }

fn b30_candidate_pointers(image: &MemoryPool) -> Vec<B30Pointer> {
    let mut out = Vec::new();
    for (slot, length) in common_pointer_head_corruption::all_units_in_the_unit_area(image) {
        let bytes = common_pointer_head_corruption::unit_bytes_at(image, slot, length);
        let unit_class = bytes[6];
        for q in 50..bytes.len().saturating_sub(28) {
            let device_a = u32::from_le_bytes(bytes[q..q + 4].try_into().unwrap());
            let device_b = u32::from_le_bytes(bytes[q + 14..q + 18].try_into().unwrap());
            if device_a != 0 || device_b != 1 { continue; }
            if bytes[q + 4..q + 10] != bytes[q + 18..q + 24] { continue; }
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..6].copy_from_slice(&bytes[q + 4..q + 10]);
            if u64::from_le_bytes(slot_bytes) < singlefs_format::UNIT_AREA_START_SLOT { continue; }
            let head_at = q - 50;
            if bytes[head_at..head_at + 34].iter().any(|b| *b != 0) { continue; }
            out.push(B30Pointer { unit_slot: slot.0, unit_class, head_at });
        }
    }
    out
}

fn b30_i24_red_after(image: &MemoryPool, pointer: &B30Pointer, offset_in_head: usize, bytes_to_write: &[u8]) -> (bool, Vec<&'static str>) {
    let mut changed = image.clone();
    let length = if pointer.unit_class == 2 { common_pointer_head_corruption::node_bytes() } else { common_pointer_head_corruption::data_unit_bytes() };
    let at = pointer.head_at + offset_in_head;
    let bytes_to_write = bytes_to_write.to_vec();
    common_pointer_head_corruption::change_the_unit_at(&mut changed, singlefs_core::address::SlotNumber(pointer.unit_slot), length, move |unit| {
        unit[at..at + bytes_to_write.len()].copy_from_slice(&bytes_to_write);
    });
    let verdicts = check_pool_image(&changed);
    let violated = violated_invariants(&verdicts);
    (violated.contains(&"I-2.4"), violated)
}

fn b30_scan(tag: &str, image: &MemoryPool) {
    let clean = check_pool_image(image);
    println!("B30SCAN image={tag} clean_violated={:?}", violated_invariants(&clean));
    let pointers = b30_candidate_pointers(image);
    let (mut both, mut only_mac, mut only_extent, mut neither) = (0, 0, 0, 0);
    for pointer in &pointers {
        let (extent_red, extent_violated) = b30_i24_red_after(image, pointer, 32, &1u16.to_le_bytes());
        let (mac_red, _) = b30_i24_red_after(image, pointer, 0, &[1u8]);
        match (extent_red, mac_red) { (true, true) => both += 1, (false, true) => only_mac += 1, (true, false) => only_extent += 1, (false, false) => neither += 1 }
        if extent_red != mac_red || (!extent_red && !mac_red) {
            // 两格都不红的：把这个单元整个清零（两盘），看判定向量有没有任何变化；一点不变 = 没有任何走读读到它（死单元），不是「走了不判」
            let length = if pointer.unit_class == 2 { common_pointer_head_corruption::node_bytes() } else { common_pointer_head_corruption::data_unit_bytes() };
            let mut zeroed = image.clone();
            common_deletion_in_progress::zero_both_copies_of_the_unit_at(&mut zeroed, singlefs_core::address::SlotNumber(pointer.unit_slot), length);
            let after_zeroing = check_pool_image(&zeroed);
            let zero_changes_nothing = format!("{after_zeroing:?}") == format!("{clean:?}");
            let names_that_differ: Vec<&'static str> = clean.iter().zip(after_zeroing.iter()).filter(|(a, b)| format!("{a:?}") != format!("{b:?}")).map(|(a, _)| a.0).collect();
            println!("ZEROING image={tag} unit_slot={} head_at={} differing_invariants={names_that_differ:?}", pointer.unit_slot, pointer.head_at);
            println!("PTR image={tag} unit_slot={} class={} head_at={} extent_red={extent_red} mac_red={mac_red} violated_by_extent_edit={extent_violated:?} zeroing_the_unit_changes_no_verdict={zero_changes_nothing}", pointer.unit_slot, pointer.unit_class, pointer.head_at);
        }
    }
    println!("B30SCAN image={tag} pointers={} both_red={both} only_mac_red={only_mac} only_extent_red={only_extent} neither_red={neither}", pointers.len());
}

#[test]
fn b30_m2_pointer_scan() {
    let only = std::env::var("B30_ONLY").ok();
    let wanted = |tag: &str| only.as_deref().map_or(true, |o| o == tag);
    if wanted("overwrite") { b30_scan("overwrite", &image_after_the_overwrite("b30-scan-overwrite")); }
    if wanted("rollback") { b30_scan("rollback", &image_after_rolling_back_to_the_first_root("b30-scan-rollback").image); }
    if wanted("deletion-intent") { b30_scan("deletion-intent", &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("b30-scan-intent")); }
    if wanted("deletion-first-batch") { b30_scan("deletion-first-batch", &common_deletion_in_progress::image_after_the_first_batch_of_a_three_unit_file("b30-scan-batch")); }
    if wanted("three-unit-file") { b30_scan("three-unit-file", &common_pointer_head_corruption::image_after_a_sequential_write_of("b30-scan-three", 3)); }
    if wanted("two-page-instance-table") { b30_scan("two-page-instance-table", &common_pointer_head_corruption::image_whose_instance_table_has_two_pages()); }
    if wanted("normal-unmount") { b30_scan("normal-unmount", &image_after_the_normal_unmount("b30-scan-unmount")); }
    if wanted("new-inodes-300") { b30_scan("new-inodes-300", &common_deletion_in_progress::pool_with_new_inodes("b30-scan-inodes", 300).memory_pool()); }
}
