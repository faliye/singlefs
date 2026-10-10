
// ===== 正推腿探针（只在自己的副本里，拼在 checker_known_bad_images.rs 整份之后）：第 35 批 =====
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, RetainedWrite, WrittenContents};

fn status_char(v: &InvariantVerdict) -> char {
    match v {
        InvariantVerdict::Holds => 'H',
        InvariantVerdict::Violated(_) => 'V',
        InvariantVerdict::PartiallyJudged(_) => 'P',
        InvariantVerdict::NotApplicable(_) => 'N',
    }
}

fn text_of(v: &InvariantVerdict) -> String {
    match v {
        InvariantVerdict::Holds => String::new(),
        InvariantVerdict::Violated(t) | InvariantVerdict::PartiallyJudged(t) => t.clone(),
        InvariantVerdict::NotApplicable(t) => (*t).to_string(),
    }
}

/// 一份镜像的整套判定：每条一个字符（H 成立 / V 违例 / P 部分没判 / N 不适用）+ 全部说明文字的哈希（SipHash 固定键，两次跑可比）。
fn full_signature(image: &MemoryPool) -> String {
    use std::hash::{Hash, Hasher};
    let verdicts = check_pool_image(image);
    let chars: String = verdicts.iter().map(|(_, v)| status_char(v)).collect();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (name, v) in &verdicts {
        name.hash(&mut hasher);
        text_of(v).hash(&mut hasher);
    }
    format!("{chars} {:016x}", hasher.finish())
}

fn names_with(verdicts: &[(&'static str, InvariantVerdict)], wanted: char) -> Vec<&'static str> {
    verdicts.iter().filter(|(_, v)| status_char(v) == wanted).map(|(n, _)| *n).collect()
}

fn short(text: &str, n: usize) -> String { text.chars().take(n).collect() }

/// 崩溃状态：基线之上，前面各段全落、这一段落一个子集（段长不超过 `subset_limit` 逐个子集，更长的只取空、逐个前缀、全）。
fn crash_states(base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], from_segment: usize, subset_limit: usize) -> Vec<(String, MemoryPool)> {
    let mut states = Vec::new();
    let mut landed = base.clone();
    for (s, segment) in segments.iter().enumerate() {
        if s >= from_segment {
            if segment.len() <= subset_limit {
                for mask in 0..(1u64 << segment.len()) {
                    let mut image = landed.clone();
                    for (bit, index) in segment.iter().enumerate() {
                        if mask >> bit & 1 == 1 { image.apply_writes(std::slice::from_ref(&writes[*index])); }
                    }
                    states.push((format!("seg{s}-mask{mask}"), image));
                }
            } else {
                for prefix in 0..=segment.len() {
                    let mut image = landed.clone();
                    for index in &segment[..prefix] { image.apply_writes(std::slice::from_ref(&writes[*index])); }
                    states.push((format!("seg{s}-prefix{prefix}of{}", segment.len()), image));
                }
            }
        }
        for index in segment { landed.apply_writes(std::slice::from_ref(&writes[*index])); }
    }
    states.push(("final".to_string(), landed));
    states
}

/// 把一个槽里的一条 journal 记录（计数器 counter，盘 device）抹成零：这份拷贝没落盘。
fn erase_the_record_copy(image: &mut MemoryPool, counter: u64, device: u32) {
    let offset = singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
    write(image, device, offset, &vec![0u8; 4096]);
}

fn core_outcome(image: &MemoryPool) -> String {
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    match mount_writable(&parameters(), &mut devices) {
        Ok(m) => format!("ok(chosen={},eff={},prefix={})", m.output.chosen_root.checkpoint_txg.0, m.output.effective_root.checkpoint_txg.0, m.output.journal.prefix_applied),
        Err(e) => format!("refused({})", short(&format!("{e:?}"), 90)),
    }
}

// ---------- P1：三条记录的发布 B（计数器 4、5、6，两块盘各一份），根槽没落；记录拷贝落的子集 × 四种改法 ----------
#[test]
fn probe_p1_subsets_of_the_record_copies_of_a_three_record_publish() {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root("probe-p1-subsets");
    let clean = crash.image.clone();
    let mut overlap = clean.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut overlap, &crash.publish, 6);
    // propagate 改了记录头、没重链：照 s3 用例的做法先从计数器 4 起恒等重链一遍（改一个字段为自己），后面两条的反向链才接得上
    rewrite_journal_record_and_rechain_the_records_after_it(&mut overlap, 4, |_record| {});
    let mut stale_first = overlap.clone();
    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut stale_first, &crash);
    // 第三种：旧根段写进末条（计数器 6）而不是首条
    let mut stale_last = overlap.clone();
    {
        let stale = &crash.previous.root;
        rewrite_journal_record_and_rechain_the_records_after_it(&mut stale_last, 6, |record| {
            record.new_tree_table = stale.tree_table;
            record.new_mapping_root = stale.mapping_root;
            record.new_tree_identifier_watermark = stale.tree_identifier_watermark;
            record.new_rollback_floor = stale.rollback_floor;
            record.new_instance_table = stale.instance_table;
            record.new_allocation_record_tree_root = stale.allocation_record_tree_root;
            record.new_unmount_marker = stale.unmount_marker;
        });
    }
    let variants: [(&str, &MemoryPool); 4] = [("clean", &clean), ("overlap", &overlap), ("overlap+stale_first", &stale_first), ("overlap+stale_last", &stale_last)];
    for mask in 0..64u32 {
        for (name, base) in variants {
            let mut image = base.clone();
            for rec in 0..3u32 {
                for device in 0..2u32 {
                    if mask >> (rec * 2 + device) & 1 == 0 { erase_the_record_copy(&mut image, 4 + u64::from(rec), device); }
                }
            }
            let verdicts = check_pool_image(&image);
            println!("PROBE-P1 mask={mask:06b} variant={name} red={:?} i54={} i310={} core={}", names_with(&verdicts, 'V'),
                status_char(&verdicts.iter().find(|(n, _)| *n == "I-5.4").unwrap().1),
                status_char(&verdicts.iter().find(|(n, _)| *n == "I-3.10").unwrap().1),
                core_outcome(&image));
        }
    }
}

// ---------- P1 / P6：合法历史上的逐状态签名（FL 开 / 关各跑一遍，比输出）----------
#[test]
fn probe_h2_states_of_a_legal_three_record_publish() {
    let mut pool = build_pool("probe-h2");
    let through_a = pool.retained_operations().len();
    let previous = pool.output.clone();
    let content = content_needing_data_units(3);
    {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(&mut writer, &mut pool.allocator, &previous, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(1)).expect("顺序写三个数据单元");
    }
    let ops = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&ops[pool.mkfs_operation_count..through_a]);
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&ops[through_a..], &geometry());
    let states = crash_states(&base, &writes, &segments, 0, 8);
    println!("PROBE-H2 states={} segments={:?}", states.len(), segments.iter().map(Vec::len).collect::<Vec<_>>());
    let mut red_states = 0;
    for (label, image) in &states {
        let sig = full_signature(image);
        if sig.contains('V') { red_states += 1; }
        println!("SIG h2 {label} {sig}");
    }
    println!("PROBE-H2 states_with_a_violation={red_states}");
}

fn h3_flow(tag: &str) -> (MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let mut reopened = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut reopened).expect("重开取号 2");
    pool.devices = Some(reopened);
    pool.allocator = mounted.allocator;
    let current = mounted.current.into_file_version().expect("三个数据单元那一版之后重开，现行那一版带文件");
    let content: Vec<u8> = (0..2900usize).map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256")).collect();
    publish_overwrite_in_process(&mut pool, &current, &content, FIXED_WRITE_TIME_SECONDS + 120, InstanceGeneration(2)).expect("实例 2 覆盖写");
    let ops = pool.retained_operations();
    let (mut writes, mut segments, _) = writes_and_segments_with_stream_indexes(&ops[pool.mkfs_operation_count..], &geometry());
    let root_of_b = writes.iter().position(|w| w.kind == StepKind::RootRecordFua && w.bytes().is_some_and(|b| u32::from_le_bytes(b[24..28].try_into().unwrap()) == 1 && u64::from_le_bytes(b[28..36].try_into().unwrap()) == 4)).expect("B 的根槽写");
    writes.remove(root_of_b);
    for segment in &mut segments {
        segment.retain(|i| *i != root_of_b);
        for i in segment.iter_mut() { if *i > root_of_b { *i -= 1; } }
    }
    segments.retain(|s| !s.is_empty());
    (pool.memory_pool_after_mkfs(), writes, segments)
}

#[test]
fn probe_h3_states_after_a_mount_over_a_version_applied_only_by_its_three_records() {
    let (base, writes, segments) = h3_flow("probe-h3");
    let first_record_segment = segments.iter().position(|s| s.iter().any(|i| writes[*i].kind == StepKind::JournalRecord && writes[*i].bytes().is_some_and(|b| u64::from_le_bytes(b[26..34].try_into().unwrap()) == 4))).expect("B 的记录段");
    let states = crash_states(&base, &writes, &segments, first_record_segment, 6);
    println!("PROBE-H3 states={} first_record_segment={first_record_segment} segments={:?}", states.len(), segments.iter().map(Vec::len).collect::<Vec<_>>());
    let mut red_states = 0;
    for (label, image) in &states {
        let sig = full_signature(image);
        if sig.contains('V') { red_states += 1; println!("PROBE-H3-RED {label} red={:?}", names_with(&check_pool_image(image), 'V')); }
        println!("SIG h3 {label} {sig}");
    }
    println!("PROBE-H3 states_with_a_violation={red_states}");
}

/// H3 流上把 B 的三条记录（两块盘）里树表指针写零、重封重链：挂载之后 B 是由记录施加出来的那一版（对拍用例 3 的 CPU 一侧）。
#[test]
fn probe_h3_all_zero_tree_table_pointer_in_the_records_of_b() {
    let (base, mut writes, segments) = h3_flow("probe-h3-zero");
    let record_indexes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord && w.bytes().is_some_and(|b| u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1 && u64::from_le_bytes(b[26..34].try_into().unwrap()) == 4)).map(|(i, _)| i).collect();
    println!("PROBE-H3Z record_writes_of_b={}", record_indexes.len());
    for index in &record_indexes {
        let WrittenContents::Bytes(bytes) = &mut writes[*index].contents else { panic!() };
        for b in &mut bytes[99..99 + 86] { *b = 0; }
        let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
    }
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
    let states = crash_states(&base, &writes, &segments, 0, 6);
    let mut union_red = std::collections::BTreeSet::new();
    for (label, image) in &states {
        let verdicts = check_pool_image(image);
        for n in names_with(&verdicts, 'V') { union_red.insert(n); }
        println!("SIG h3z {label} {}", full_signature(image));
    }
    println!("PROBE-H3Z states={} union_of_red_invariants_over_all_states={union_red:?}", states.len());
}
