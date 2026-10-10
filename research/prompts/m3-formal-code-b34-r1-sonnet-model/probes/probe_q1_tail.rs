
// ===== 正推腿探针（只在自己的副本里）：Q1 =====
fn probe_summary(label: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    let partial: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::PartiallyJudged(_))).map(|(n, _)| *n).collect();
    let not_applicable: Vec<String> = verdicts.iter().filter(|(n, v)| ["I-5.4","I-5.5","I-9.5","I-9.24","I-9.18","I-9.28","I-3.10"].contains(n) && matches!(v, InvariantVerdict::NotApplicable(_))).map(|(n, v)| format!("{n}:{v:?}")).collect();
    let i54 = verdicts.iter().find(|(n, _)| *n == "I-5.4").map(|(_, v)| format!("{v:?}")).unwrap();
    println!("PROBE {label} violated={violated:?}");
    println!("PROBE {label} partially_judged={partial:?}");
    println!("PROBE {label} not_applicable_among_watched={not_applicable:?}");
    println!("PROBE {label} I-5.4={}", i54.chars().take(260).collect::<String>());
}

#[test]
fn probe_q1_next_mount_version_with_all_zero_tree_table_pointer_in_the_new_root_segment() {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root("probe-q1-next-mount");
    probe_summary("q1.clean", &clean);
    let mut image = clean.clone();
    let mut touched = 0;
    for device in DEVICES {
        let records = journal_records_on_device(&image, device);
        let (counter, _, _) = *records.last().expect("有记录");
        let offset = singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
        let mut record = read(&image, device, offset, 4096);
        for b in &mut record[99..99 + 86] { *b = 0; }
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(&mut image, device, offset, &record);
        touched += 1;
    }
    println!("PROBE q1 records_zeroed={touched}");
    probe_summary("q1.zeroed_new_root_segment_tree_table_pointer(next-mount version)", &image);
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    match mount_writable(&parameters(), &mut devices) {
        Ok(m) => println!("PROBE q1 mount_writable=Ok chosen={:?} effective={:?} prefix={}", m.output.chosen_root.checkpoint_txg, m.output.effective_root.checkpoint_txg, m.output.journal.prefix_applied),
        Err(e) => println!("PROBE q1 mount_writable=Err {}", format!("{e:?}").chars().take(220).collect::<String>()),
    }
}

#[test]
fn probe_q1_applied_only_by_records_version_with_all_zero_tree_table_pointer() {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record("probe-q1-applied-only");
    probe_summary("q1b.clean", &built.image);
    let mut image = built.image.clone();
    let mut touched = 0;
    for device in DEVICES {
        for (counter, instance, _) in journal_records_on_device(&image, device) {
            let offset = singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
            let mut record = read(&image, device, offset, 4096);
            let txg = get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET);
            if instance == 1 && txg == 5 {
                for b in &mut record[99..99 + 86] { *b = 0; }
                let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
                record[46..78].copy_from_slice(&digest);
                write(&mut image, device, offset, &record);
                touched += 1;
            }
        }
    }
    println!("PROBE q1b records_zeroed={touched}");
    probe_summary("q1b.zeroed(applied-only version (1,5))", &image);
}

#[test]
fn probe_prefix_states_of_legal_publishes_never_redden_or_partially_judge_i54() {
    // 合法历史：新池新建文件（A）后连发三次覆盖写 B、C、D，全部写操作按先后施加前缀（含两盘各一份的记录、根槽 FUA 之前的每一步）。
    let mut pool = build_pool("probe-prefix-legal");
    let first_op_count = pool.retained_operations().len();
    let mut previous = pool.output.clone();
    for (n, extra) in [60u64, 120, 180].iter().enumerate() {
        let content: Vec<u8> = (0..(4100usize + 700 * n)).map(|i| u8::try_from((i * 11 + n) % 251).unwrap()).collect();
        previous = publish_overwrite_in_process(&mut pool, &previous, &content, FIXED_WRITE_TIME_SECONDS + extra, InstanceGeneration(1)).expect("覆盖写");
    }
    let operations = pool.retained_operations();
    let total = operations.len();
    let mut tallies = std::collections::BTreeMap::<String, u64>::new();
    let mut states = 0u64;
    let mut any_violation_states = 0u64;
    for k in first_op_count..=total {
        let mut image = pool.memory_pool_after_mkfs();
        image.apply(&operations[pool.mkfs_operation_count..k]);
        let verdicts = check_pool_image(&image);
        states += 1;
        let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
        if !violated.is_empty() { any_violation_states += 1; println!("PROBE-PREFIX k={k} violated={violated:?}"); }
        let v = verdicts.iter().find(|(n, _)| *n == "I-5.4").map(|(_, v)| match v {
            InvariantVerdict::Holds => "Holds".to_string(),
            InvariantVerdict::Violated(_) => "Violated".to_string(),
            InvariantVerdict::PartiallyJudged(_) => "PartiallyJudged".to_string(),
            InvariantVerdict::NotApplicable(_) => "NotApplicable".to_string(),
        }).unwrap();
        *tallies.entry(v).or_default() += 1;
    }
    println!("PROBE-PREFIX states={states} states_with_any_violation={any_violation_states} i54_tally={tallies:?} first_op_count={first_op_count} total_ops={total}");
}

#[test]
fn probe_i54_tally_over_the_prefix_states_of_the_new_pool_file_creation() {
    // mkfs 之后到第一个文件那次发布写完：每个写操作前缀一个状态，数 I-5.4 的判定（对照 RV 开 / 关）。
    let pool = build_pool("probe-prefix-new-pool");
    let operations = pool.retained_operations();
    let total = operations.len();
    let mut tallies = std::collections::BTreeMap::<String, u64>::new();
    let mut first_state_of_each = std::collections::BTreeMap::<String, usize>::new();
    for k in pool.mkfs_operation_count..=total {
        let mut image = pool.memory_pool_after_mkfs();
        image.apply(&operations[pool.mkfs_operation_count..k]);
        let verdicts = check_pool_image(&image);
        let v = verdicts.iter().find(|(n, _)| *n == "I-5.4").map(|(_, v)| match v {
            InvariantVerdict::Holds => "Holds".to_string(),
            InvariantVerdict::Violated(_) => "Violated".to_string(),
            InvariantVerdict::PartiallyJudged(_) => "PartiallyJudged".to_string(),
            InvariantVerdict::NotApplicable(_) => "NotApplicable".to_string(),
        }).unwrap();
        first_state_of_each.entry(v.clone()).or_insert(k);
        *tallies.entry(v).or_default() += 1;
    }
    println!("PROBE-TALLY states={} mkfs_ops={} total_ops={} i54_tally={tallies:?} first_k={first_state_of_each:?}", total - pool.mkfs_operation_count + 1, pool.mkfs_operation_count, total);
}
