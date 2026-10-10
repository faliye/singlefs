
// ---------- P2：树表指针全零的几种形态，整套判定对照 ----------
fn matrix_row(label: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let red = names_with(&verdicts, 'V');
    let partial = names_with(&verdicts, 'P');
    let watched = ["I-5.4", "I-5.5", "I-9.5", "I-9.18", "I-9.28", "I-3.10", "I-4.1", "I-3.1", "I-3.5", "I-2.5"];
    let na: Vec<String> = verdicts.iter().filter(|(n, v)| watched.contains(n) && matches!(v, InvariantVerdict::NotApplicable(_))).map(|(n, v)| format!("{n}:{}", short(&text_of(v), 24))).collect();
    println!("PROBE-P2 {label} red={red:?} partial={partial:?} na_watched={na:?}");
    for (n, v) in &verdicts {
        if ["I-2.1", "I-4.1", "I-7.2", "I-7.4", "I-4.8", "I-3.1"].contains(n) && matches!(v, InvariantVerdict::Violated(_)) {
            println!("PROBE-P2-DETAIL {label} {n}: {}", short(&text_of(v), 150));
        }
    }
}

#[test]
fn probe_p2_matrix_of_the_all_zero_tree_table_pointer_forms() {
    matrix_row("F0.clean(first file, rows written)", &image_after_the_first_file_on_the_version_whose_rows_were_written("probe-p2-clean").0);
    for txg in [5u64, 4, 3] {
        let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(&format!("probe-p2-root-{txg}"), CheckpointTxg(txg));
        matrix_row(&format!("F1.root_ring_root_txg{txg}"), &image);
    }
    // 由记录施加出来的那一版
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record("probe-p2-applied-only");
    matrix_row("F2.clean(version applied only by its record)", &built.image);
    let mut image = built.image.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(&mut image, built.version_applied_only_by_its_record.record.counter);
    matrix_row("F2.applied_only_by_records_all_zero", &image);
    // 下一次挂载先施加的那一版
    let clean = crash_image_with_the_overwrite_record_but_not_its_root("probe-p2-next");
    matrix_row("F3.clean(crash before root of B)", &clean);
    let mut image = clean.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(&mut image, COUNTER_OF_THE_OVERWRITE_RECORD);
    matrix_row("F3.next_mount_version_all_zero", &image);
    println!("PROBE-P2 F3.core_outcome={}", core_outcome(&image));
    // 被抛弃的根
    let (mut image, _, _, abandoned_txg) = image_after_a_recovery_abandoned_the_third_root("probe-p2-abandoned");
    matrix_row("F4.clean(abandoned third root)", &image);
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(abandoned_txg), |bytes| { bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0); });
    matrix_row("F4.abandoned_root_all_zero", &image);
}

// ---------- P4：I-5.5 的几种判定 ----------
fn i55(label: &str, image: &MemoryPool) {
    let v = verdict(image, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK);
    println!("PROBE-P4 {label} I-5.5={}{}", status_char(&v), match &v { InvariantVerdict::Holds => String::new(), other => format!(" {}", short(&text_of(other), 110)) });
}

#[test]
fn probe_p4_first_data_unit_verdicts_by_form() {
    i55("first_file_clean", &build_pool("probe-p4-clean").memory_pool());
    i55("after_overwrite(two versions)", &image_after_the_overwrite("probe-p4-overwrite"));
    i55("mkfs_only", &format_pool("probe-p4-mkfs").memory_pool());
    let mut image = build_pool("probe-p4-extent-root-zero").memory_pool();
    mutate_unit(&mut image, TREE_TABLE, |bytes| {
        for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
            let entry = tree_table_entry_offset(index);
            let kind = u16::from_le_bytes([bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET], bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]]);
            if kind == TREE_KIND_EXTENT { bytes[entry + 14..entry + 100].fill(0); }
        }
    }, true);
    i55("extent_root_zeroed", &image);
    let verdicts = check_pool_image(&image);
    println!("PROBE-P4 extent_root_zeroed red={:?} partial={:?}", names_with(&verdicts, 'V'), names_with(&verdicts, 'P'));
    // 数据单元那一条 extent 记录的数据指针整条清零（extent 树根仍在、读得出，但里面没有数据指针）：「一条数据指针都没有」那一支
    let mut image2 = build_pool("probe-p4-data-pointer-zero").memory_pool();
    println!("PROBE-P4 (data-pointer-zero variant skipped: extent leaf rewrite helper not in scope)");
    let _ = &mut image2;
}

// ---------- P6：新池新建文件那条流，I-5.4 与 I-3.10 逐状态评估集 ----------
#[test]
fn probe_p6_new_pool_stream_i54_and_i310_evaluated_sets() {
    let pool = build_pool("probe-p6-new-pool");
    let ops = pool.retained_operations();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&ops[pool.mkfs_operation_count..], &geometry());
    println!("PROBE-P6 segments={:?}", segments.iter().map(Vec::len).collect::<Vec<_>>());
    let root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("根槽写");
    let base = pool.memory_pool_after_mkfs();
    // 不展开 24 个单元写的那一段（只取空与全，段里的子集不影响这两条的评估集：单元写之后才有记录）
    let states = crash_states(&base, &writes, &segments, 0, 8);
    let mut tally: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut disagreements = 0u64;
    for (label, image) in &states {
        let verdicts = check_pool_image(image);
        let evaluated = |name: &str| { let c = status_char(&verdicts.iter().find(|(n, _)| *n == name).unwrap().1); c == 'H' || c == 'V' };
        let (a, b) = (evaluated("I-5.4"), evaluated("I-3.10"));
        if a != b { disagreements += 1; println!("PROBE-P6-DISAGREE {label} i54={a} i310={b}"); }
        let key = format!("i54={}/i310={}", status_char(&verdicts.iter().find(|(n, _)| *n == "I-5.4").unwrap().1), status_char(&verdicts.iter().find(|(n, _)| *n == "I-3.10").unwrap().1));
        *tally.entry(key).or_default() += 1;
    }
    println!("PROBE-P6 states={} root_write_index={root_index} disagreements={disagreements} tally={tally:?}", states.len());
}
