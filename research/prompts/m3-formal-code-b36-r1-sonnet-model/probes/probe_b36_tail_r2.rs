
// ===== 第 36 批 R2：「拿掉根槽写」两条流的新旧造法对照（只在自己副本里；拼在 probe_b36_tail.rs 之后）=====
// 旧造法 = 第 35 批对拍用例的 record_three_unit_file_applied_only_by_its_records_after_a_mount：整段历史（含挂载与覆盖写）在同一个进程里录完，
//   再从流里拿掉 txg 4 的根槽写（本文件 tail.rs 的 h3_flow 同形）。
// 新造法 = 第 36 批那一个函数：录到 txg 4 的根槽写之前停，从全落的崩溃镜像起新进程录可写挂载与实例 2 的覆盖写（改动全文 36-2 的 diff 逐行照搬，只把 checker-tier 的 RecordedFlow 换成三元组）。

fn r2_third_content() -> Vec<u8> {
    (0..2900usize).map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256")).collect()
}

fn r2_old_flow(tag: &str) -> (MemoryPool, Vec<RetainedWrite>) {
    let (base, writes, _segments) = h3_flow(tag);
    (base, writes)
}

/// 返回：基线、挂载之前那一段写（崩溃镜像 = 基线 + 这一段全落）、新进程那一段写（挂载 + 覆盖写）、覆盖写的输出。
fn r2_new_flow(tag: &str, rewrite_the_records_before_the_mount: &dyn Fn(&mut Vec<RetainedWrite>)) -> (MemoryPool, Vec<RetainedWrite>, Vec<RetainedWrite>, TransactionOutput) {
    let pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let operations = pool.retained_operations();
    let classifier = geometry();
    let operations_after_mkfs = &operations[pool.mkfs_operation_count..];
    let root_of_the_three_unit_publish = operations_after_mkfs
        .iter()
        .position(|retained| {
            classifier.classify(&retained.operation) == StepKind::RootRecordFua
                && retained.contents.as_ref().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
                })
        })
        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
    let (mut writes, _segments, _stream_indexes_before_the_crash) =
        writes_and_segments_with_stream_indexes(&operations_after_mkfs[..root_of_the_three_unit_publish], &classifier);
    rewrite_the_records_before_the_mount(&mut writes);
    let base = pool.memory_pool_after_mkfs();
    let stream_of_the_new_process = SharedStream::retaining_contents();
    let every_write_landed = vec![true; writes.len()];
    let mut devices = crash_state_devices(&base, &writes, &every_write_landed, &stream_of_the_new_process);
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("崩在 txg 4 根槽之前的镜像上新进程可写挂载：施加那三条记录、取号 2");
    assert_eq!(
        (mounted.output.chosen_root.checkpoint_txg, mounted.output.effective_root.checkpoint_txg, mounted.output.journal.prefix_applied),
        (CheckpointTxg(3), CheckpointTxg(4), 3),
        "择 (1, 3) 的根，施加 txg 4 的三条记录，现行版是 (1, 4)"
    );
    let current = mounted.current.into_file_version().expect("施加了 txg 4，现行那一版带文件");
    let overwrite = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(&mut writer, &mut mounted.allocator, &current, FirstFile { content: &r2_third_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 }, InstanceGeneration(2)).expect("实例 2 覆盖写")
    };
    let (writes_of_the_new_process, _segments, _indexes) =
        writes_and_segments_with_stream_indexes(&stream_of_the_new_process.retained_operations(), &classifier);
    (base, writes, writes_of_the_new_process, overwrite)
}

fn slot_label(slot: u64) -> &'static str {
    match slot { 0..=63 => "固定区(系统配置等)[0,64)", 64..=1023 => "根环[64,1024)", 1024..=50175 => "journal 环[1024,50176)", _ => "单元区[50176,…)" }
}

fn r2_describe(label: &str, image: &MemoryPool) {
    let configuration = singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
    let root = singlefs_core::recovery::choose_root(image, &configuration).expect("择到根");
    let chain = singlefs_core::recovery::instance_table_chain_of_root(image, &root).expect("实例表链");
    println!("PROBE-R2 {label} chosen_root=({},{}) instance_rows={:?}", root.instance.0, root.checkpoint_txg.0,
        chain.records.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect::<Vec<_>>());
    for device in 0..2u32 {
        for k in 0..2u64 {
            let bytes = read(image, device, k * 4096, 4096);
            match singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
                Ok(c) => println!("PROBE-R2 {label} sysconf dev{device} slot{k}: journal_tail={} journal_instance={}", c.quantities.journal_tail, c.quantities.journal_instance.0),
                Err(e) => println!("PROBE-R2 {label} sysconf dev{device} slot{k}: 解不出 {e:?}"),
            }
        }
    }
}

fn r2_diff(label: &str, a: &MemoryPool, b: &MemoryPool) -> usize {
    let mut differing = std::collections::BTreeMap::<&'static str, Vec<(u32, u64)>>::new();
    for device in 0..2u32 {
        for slot in 0..(50176u64 + 8192) {
            if read(a, device, slot * SLOT, SLOT as usize) != read(b, device, slot * SLOT, SLOT as usize) {
                differing.entry(slot_label(slot)).or_default().push((device, slot));
            }
        }
    }
    let total: usize = differing.values().map(Vec::len).sum();
    for (category, slots) in &differing { println!("PROBE-R2 {label} 不同的 16 KiB 槽 {category}: {} 个，前几个 (盘, 槽) = {:?}", slots.len(), &slots[..slots.len().min(6)]); }
    println!("PROBE-R2 {label} 不同的槽合计 {total}");
    total
}

#[test]
fn probe_r2_old_and_new_construction_of_the_applied_only_by_its_records_flow() {
    let (old_base, old_writes) = r2_old_flow("probe-r2-old");
    let mut old_final = old_base.clone();
    old_final.apply_writes(&old_writes);
    let (new_base, writes_before_the_mount, writes_of_the_new_process, overwrite) = r2_new_flow("probe-r2-new", &|_writes| {});
    let mut crash_image = new_base.clone();
    crash_image.apply_writes(&writes_before_the_mount);
    let mut new_final = crash_image.clone();
    new_final.apply_writes(&writes_of_the_new_process);
    println!("PROBE-R2 writes: old={} new_before_mount={} new_new_process={} overwrite=(inst {}, txg {})", old_writes.len(), writes_before_the_mount.len(), writes_of_the_new_process.len(), overwrite.root.instance.0, overwrite.root.checkpoint_txg.0);
    // B 的三条记录里的最大事务号：直接读记录字节（偏移 78，8 字节），不经 core 的挂载读法
    let mut max_transaction_of_b = 0u64;
    for write in writes_before_the_mount.iter().filter(|w| w.kind == StepKind::JournalRecord) {
        let bytes = write.bytes().expect("记录写带字节");
        if u32::from_le_bytes(bytes[16..20].try_into().unwrap()) == 1 && u64::from_le_bytes(bytes[26..34].try_into().unwrap()) == 4 {
            max_transaction_of_b = max_transaction_of_b.max(u64::from_le_bytes(bytes[78..86].try_into().unwrap()));
        }
    }
    println!("PROBE-R2 max_transaction_in_the_records_of_B(read_from_record_bytes)={max_transaction_of_b}");
    // 两条流里系统配置槽写的次序：(journal_tail, journal_instance)，按流的次序；旧造法里 B 的根槽写拿掉了、它之后的那次系统配置轮换还在
    let sysconf_sequence = |writes: &[RetainedWrite]| -> Vec<(u64, u32)> {
        writes.iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).filter_map(|w| w.bytes().and_then(|b| singlefs_core::system_configuration::SystemConfiguration::parse_slot(&b[..4096]).ok())).map(|c| (c.quantities.journal_tail, c.quantities.journal_instance.0)).collect()
    };
    let old_sequence = sysconf_sequence(&old_writes);
    let mut new_writes_all = writes_before_the_mount.clone();
    new_writes_all.extend(writes_of_the_new_process.iter().cloned());
    let new_sequence = sysconf_sequence(&new_writes_all);
    println!("PROBE-R2 sysconf_writes_in_stream_order old={old_sequence:?}");
    println!("PROBE-R2 sysconf_writes_in_stream_order new={new_sequence:?} (崩溃之前那一段 {} 个)", sysconf_sequence(&writes_before_the_mount).len());
    r2_describe("old_final", &old_final);
    r2_describe("new_final", &new_final);
    r2_describe("legal_crash_image_before_mount(new)", &crash_image);
    let _ = r2_diff("old_final_vs_new_final", &old_final, &new_final);
    // 末态判定：CPU 池级 checker 整套签名，新旧各一份；红的与部分没判的不变量名
    let (old_verdicts, new_verdicts) = (check_pool_image(&old_final), check_pool_image(&new_final));
    println!("PROBE-R2 final_state_red old={:?} new={:?}", names_with(&old_verdicts, 'V'), names_with(&new_verdicts, 'V'));
    println!("PROBE-R2 final_state_partial old={:?} new={:?}", names_with(&old_verdicts, 'P'), names_with(&new_verdicts, 'P'));
    let differing_verdicts: Vec<String> = old_verdicts.iter().zip(new_verdicts.iter()).filter(|((_, a), (_, b))| status_char(a) != status_char(b) || text_of(a) != text_of(b)).map(|((n, a), (_, b))| format!("{n}:{}->{} {}", status_char(a), status_char(b), short(&text_of(b), 60))).collect();
    println!("PROBE-R2 final_state_verdicts_that_differ={differing_verdicts:?}");
    // 新造法的崩溃镜像（挂载之前）与旧造法同一时刻的镜像：旧造法里那一刻 = 基线 + 旧流到 txg 4 根槽写之前的写
    println!("PROBE-R2 new_flow_effective_after_mount_recover={}", recover_outcome(&new_final));
}

/// 流 1（「第一条记录带上一版的根」）的挂载前改法：B 的第一条记录（计数器 4，两块盘）新根段七项换成 A 那条根的，重封、后两条重链（同 tail5 的 h3 探针）。
fn r2_write_the_root_of_a_into_the_first_record_of_b(writes: &mut Vec<RetainedWrite>) {
    let root_of_a: Vec<u8> = writes.iter().filter(|w| w.kind == StepKind::RootRecordFua).filter_map(RetainedWrite::bytes).find(|b| u32::from_le_bytes(b[24..28].try_into().unwrap()) == 1 && u64::from_le_bytes(b[28..36].try_into().unwrap()) == 3).expect("A 的根").to_vec();
    let record_indexes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord && w.bytes().is_some_and(|b| u32::from_le_bytes(b[16..20].try_into().unwrap()) == 1 && u64::from_le_bytes(b[26..34].try_into().unwrap()) == 4)).map(|(i, _)| i).collect();
    let counter_of = |w: &RetainedWrite| { let b = w.bytes().unwrap(); let mut c = [0u8; 8]; c[..6].copy_from_slice(&b[20..26]); u64::from_le_bytes(c) };
    let fields: [(usize, usize, usize); 7] = [(0, 36, 86), (86, 256, 86), (172, 122, 8), (180, 130, 8), (188, 170, 86), (274, 342, 86), (360, 20, 4)];
    for index in &record_indexes {
        if counter_of(&writes[*index]) != 4 { continue; }
        let WrittenContents::Bytes(bytes) = &mut writes[*index].contents else { panic!() };
        for (in_segment, in_root, length) in fields { bytes[99 + in_segment..99 + in_segment + length].copy_from_slice(&root_of_a[in_root..in_root + length]); }
        let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
    }
    rechain_record_writes(writes, &record_indexes);
}

/// 流 2（「树表指针全零」）：挂载之后改记录，在镜像上把 B 的三条记录（计数器 4、5、6，两块盘）新根段里的树表指针写零、重封、同盘 4 → 5 → 6 重链
/// （新造法里那一段在流尾、不动别的记录；这里直接在「整段落盘」的末态上做，与那一段全落的状态同一份字节）。
fn r2_zero_the_tree_table_pointer_in_the_records_of_b(image: &mut MemoryPool) {
    for device in 0..2u32 {
        let mut previous_chain: Option<u32> = None;
        for counter in [4u64, 5, 6] {
            let offset = singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
            let mut bytes = read(image, device, offset, 4096);
            bytes[99..99 + 86].fill(0);
            if let Some(chain) = previous_chain { bytes[91..95].copy_from_slice(&chain.to_le_bytes()); }
            let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
            bytes[46..78].copy_from_slice(&digest);
            write(image, device, offset, &bytes);
            let mut zeroed: Vec<u8> = bytes[..487].to_vec();
            zeroed[46..78].fill(0);
            previous_chain = Some(crc32_castagnoli(&zeroed));
        }
    }
}

#[test]
fn probe_r2_the_two_flows_by_name() {
    // 流 1：第一条记录带上一版的根；新造法：挂载读到的就是这三条记录（末条是 B 自己的根段）
    let (base, before, after, overwrite) = r2_new_flow("probe-r2-flow1", &r2_write_the_root_of_a_into_the_first_record_of_b);
    let mut crash = base.clone(); crash.apply_writes(&before);
    let mut fin = crash.clone(); fin.apply_writes(&after);
    println!("PROBE-R2F1 crash_image_recover={} final_recover={} overwrite_txg={}", recover_outcome(&crash), recover_outcome(&fin), overwrite.root.checkpoint_txg.0);
    let v = check_pool_image(&fin);
    println!("PROBE-R2F1 final_red={:?} final_partial={:?}", names_with(&v, 'V'), names_with(&v, 'P'));
    r2_describe("flow1_final", &fin);
    // 流 2：树表指针全零
    let (base2, before2, after2, _o) = r2_new_flow("probe-r2-flow2", &|_w| {});
    let mut crash2 = base2.clone(); crash2.apply_writes(&before2);
    let mut fin2 = crash2.clone(); fin2.apply_writes(&after2);
    let v_clean = check_pool_image(&fin2);
    println!("PROBE-R2F2 final_before_the_appended_segment red={:?}", names_with(&v_clean, 'V'));
    let mut zeroed = fin2.clone();
    r2_zero_the_tree_table_pointer_in_the_records_of_b(&mut zeroed);
    let v_zero = check_pool_image(&zeroed);
    println!("PROBE-R2F2 final_after_the_appended_segment red={:?}", names_with(&v_zero, 'V'));
    for (name, verdict) in &v_zero { if matches!(verdict, InvariantVerdict::Violated(_)) { println!("PROBE-R2F2 {name}: {}", short(&text_of(verdict), 110)); } }
    // 同一份字节在「挂载之前」（崩溃镜像上改同样三条记录）上：B 是下一次挂载先施加的那一版，不是由记录施加出来的
    let mut zeroed_before = crash2.clone();
    r2_zero_the_tree_table_pointer_in_the_records_of_b(&mut zeroed_before);
    let v_before = check_pool_image(&zeroed_before);
    println!("PROBE-R2F2 same_rewrite_on_the_crash_image_before_the_mount red={:?} recover={}", names_with(&v_before, 'V'), recover_outcome(&zeroed_before));
}
