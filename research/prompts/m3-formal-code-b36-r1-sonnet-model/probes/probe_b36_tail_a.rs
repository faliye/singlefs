
// ===== 第 36 批 P1 补充探针（拼在 probe_b36_tail.rs 之后，用它的 status_char / text_of / short / names_with / erase_the_record_copy / recover_outcome）=====

fn flip_a_byte_of_the_first_named_unit_of_a_record(image: &mut MemoryPool, record_counter: u64, which_location: usize) {
    let record = read(image, 0, record_offset(record_counter, JOURNAL_RING_DEFAULT_BYTES).0, 4096);
    assert!(get_u32(&record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0, "记录 {record_counter} 有点名项");
    let location = JOURNAL_RECORD_HEADER_BYTES + 14 * which_location;
    let named_device = get_u32(&record, location);
    let mut named_slot_bytes = [0u8; 8];
    named_slot_bytes[..6].copy_from_slice(&record[location + 4..location + 10]);
    let named_slot = u64::from_le_bytes(named_slot_bytes);
    let mut unit = read(image, named_device, named_slot * SLOT, 16384);
    unit[1000] ^= 0xff;
    write(image, named_device, named_slot * SLOT, &unit);
}

fn next_mount_first_version_is_taken_by_the_checker(image: &MemoryPool, txg: u64) -> (bool, String) {
    let verdicts = check_pool_image(image);
    let v = &verdicts.iter().find(|(n, _)| *n == "I-5.4").unwrap().1;
    let detail = text_of(v);
    (matches!(v, InvariantVerdict::Violated(_)) && detail.starts_with(&format!("下一次挂载先施加的那一版（实例 1、txg {txg}）")), format!("{}:{}", status_char(v), short(&detail, 30)))
}

// ---------- P1b：B (1, 4)、C (1, 5) 各一条记录、两个根槽都读不出；B、C 两块盘上的记录拷贝落不落 × B、C 第一个点名单元哪一份被翻一个字节 ----------
#[test]
fn probe_p1b_grid_of_two_single_record_publishes_with_copies_and_named_units() {
    let built = image_with_two_single_record_publishes("probe-p1b");
    let mut base = built.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut base, &built.first_publish, 2);
    rewrite_journal_record_and_rechain_the_records_after_it(&mut base, built.first_publish.record.counter, |_kept| {});
    for txg in [4u64, 5] { tear_the_root_slot_written_for_txg(&mut base, txg); }
    let (counter_b, counter_c) = (built.first_publish.record.counter, built.second_publish.record.counter);
    let mut rows = 0u32;
    for copies_b in 0..4u32 { for copies_c in 0..4u32 { for named_b in 0..3usize { for named_c in 0..3usize {
        let mut image = base.clone();
        if named_b > 0 { flip_a_byte_of_the_first_named_unit_of_a_record(&mut image, counter_b, named_b - 1); }
        if named_c > 0 { flip_a_byte_of_the_first_named_unit_of_a_record(&mut image, counter_c, named_c - 1); }
        for device in 0..2u32 {
            if copies_b >> device & 1 == 0 { erase_the_record_copy(&mut image, counter_b, device); }
            if copies_c >> device & 1 == 0 { erase_the_record_copy(&mut image, counter_c, device); }
        }
        let (taken, d54) = next_mount_first_version_is_taken_by_the_checker(&image, 4);
        let verdicts = check_pool_image(&image);
        let i41 = text_of(&verdicts.iter().find(|(n, _)| *n == "I-4.1").unwrap().1);
        println!("PROBE-P1B copiesB={copies_b:02b} copiesC={copies_c:02b} namedB={named_b} namedC={named_c} checker_takes_B={taken} i54={d54} i41={} rec={}", short(&i41, 18), recover_outcome(&image));
        rows += 1;
    } } } }
    println!("PROBE-P1B rows={rows}");
}

// ---------- P1c：锚点认不定（所选根 A 那次发布带末条标志的记录两条）：B 的首条写成 (1, 3) 且带末条标志 ----------
#[test]
fn probe_p1c_anchor_undecided() {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root("probe-p1c");
    let mut image = crash.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 4, |_kept| {});
    let (taken, d54) = next_mount_first_version_is_taken_by_the_checker(&image, 4);
    println!("PROBE-P1C control_all_landed checker_takes_B={taken} i54={d54} rec={}", recover_outcome(&image));
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 4, |record| {
        record.checkpoint_txg = CheckpointTxg(3);
        record.place_in_publish = singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish;
    });
    let (taken, d54) = next_mount_first_version_is_taken_by_the_checker(&image, 4);
    let verdicts = check_pool_image(&image);
    let i41 = text_of(&verdicts.iter().find(|(n, _)| *n == "I-4.1").unwrap().1);
    println!("PROBE-P1C two_last_flags_at_root_txg checker_takes_B={taken} i54={d54} i41={} rec={} core={}", short(&i41, 40), recover_outcome(&image), core_outcome(&image));
}

// ---------- P1d：管理员回退之后（向前发布 D）再发一版 E，E 只在记录里：E 的记录拷贝 × E 的根槽落不落 ----------
fn pool_rolled_back_then_one_more_publish(tag: &str) -> (MemoryPool, TransactionOutput, TransactionOutput) {
    let (mut pool, first, mut third) = pool_through_the_third_version(tag);
    {
        let rollback_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(&rollback_parameters, devices, &mut pool.allocator, &mut third, RollbackTarget { instance: first.root.instance, checkpoint_txg: first.root.checkpoint_txg }).expect("挂着的时候回退到 A");
    }
    let rollback_publish = third.clone();
    let content: Vec<u8> = (0..4100usize).map(|index| u8::try_from((index * 5 + 9) % 253).expect("小于 256")).collect();
    let after = publish_overwrite_in_process(&mut pool, &rollback_publish, &content, FIXED_WRITE_TIME_SECONDS + 180, InstanceGeneration(2)).expect("回退之后再发一版 E");
    (pool.memory_pool(), rollback_publish, after)
}

#[test]
fn probe_p1d_after_a_rollback_publish_one_more_unrooted_publish() {
    let (image_all_landed, d, e) = pool_rolled_back_then_one_more_publish("probe-p1d");
    println!("PROBE-P1D rollback=(inst {}, txg {}, counter {}) e=(inst {}, txg {}, counter {})", d.root.instance.0, d.root.checkpoint_txg.0, d.record.counter, e.root.instance.0, e.root.checkpoint_txg.0, e.record.counter);
    let txg_e = e.root.checkpoint_txg.0;
    let counter_e = e.record.counter;
    let mut base = image_all_landed.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut base, &e, 2);
    for root_lands in [true, false] {
        for copies in 0..4u32 {
            let mut image = base.clone();
            if !root_lands { tear_the_root_slot_written_for_txg(&mut image, txg_e); }
            for device in 0..2u32 { if copies >> device & 1 == 0 { erase_the_record_copy(&mut image, counter_e, device); } }
            let verdicts = check_pool_image(&image);
            let v = &verdicts.iter().find(|(n, _)| *n == "I-5.4").unwrap().1;
            let detail = text_of(v);
            println!("PROBE-P1D root_lands={root_lands} copiesE={copies:02b} i54={}:{} red={:?} rec={}", status_char(v), short(&detail, 50), names_with(&verdicts, 'V'), recover_outcome(&image));
        }
    }
}
