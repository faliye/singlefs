
// ---------- P5：根记录四条指针头部逐字节：core 择根 vs 对 GPU 择根谓词（head_ok && extent_offset_zero）的逐字节仿写 ----------
#[test]
fn probe_p5_core_root_choice_against_the_gpu_rule_byte_by_byte() {
    let pool = build_pool("probe-p5");
    let operations = pool.retained_operations();
    let (writes, _segments, _) = writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("发布有一条根槽写");
    let base = pool.memory_pool_after_mkfs();
    let WrittenContents::Bytes(publish_root) = &writes[judged_root_index].contents else { panic!() };
    let publish_txg = u64::from_le_bytes(publish_root[28..36].try_into().unwrap());
    let choose_txg = |writes: &[RetainedWrite]| -> u64 {
        let mut image = base.clone();
        image.apply_writes(writes);
        let configuration = singlefs_core::recovery::choose_system_configuration(&image).expect("系统配置");
        singlefs_core::recovery::choose_root(&image, &configuration).expect("择得到根").checkpoint_txg.0
    };
    assert_eq!(choose_txg(&writes), publish_txg, "不改时 core 择到发布的根");
    let pointers = [("tree_table", 36usize), ("instance_table", 170), ("mapping_root", 256), ("allocation_root", 342)];
    let (mut agree, mut disagree, mut core_rejects, mut gpu_rejects) = (0u32, 0u32, 0u32, 0u32);
    let mut disagreements = Vec::new();
    for (name, pointer_offset) in pointers {
        let all_zero_originally = publish_root[pointer_offset..pointer_offset + 86].iter().all(|b| *b == 0);
        for position in 0..40usize {
            let mut rewritten = publish_root.clone();
            rewritten[pointer_offset + position] ^= 1;
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&rewritten, rewritten.len(), 138);
            rewritten[138..170].copy_from_slice(&digest);
            let mut changed = writes.clone();
            changed[judged_root_index].contents = WrittenContents::Bytes(rewritten.clone());
            let core_takes_publish_root = choose_txg(&changed) == publish_txg;
            // 对 GPU 谓词的逐字节仿写：四条指针各自头部 34 字节（head_ok 的 0..32 + extent 偏移 32..34）全 0 才择
            let gpu_rule_takes_publish_root = pointers.iter().all(|(_, o)| rewritten[*o..*o + 34].iter().all(|b| *b == 0));
            if core_takes_publish_root == gpu_rule_takes_publish_root { agree += 1; } else { disagree += 1; disagreements.push((name, position, core_takes_publish_root)); }
            if !core_takes_publish_root { core_rejects += 1; }
            if !gpu_rule_takes_publish_root { gpu_rejects += 1; }
            if position == 0 || position == 28 || position == 29 || position == 30 || position == 32 || position == 34 { println!("PROBE-P5 {name} byte {position}: orig_zero_pointer={all_zero_originally} core_takes_publish_root={core_takes_publish_root} gpu_rule_takes_publish_root={gpu_rule_takes_publish_root}"); }
        }
    }
    println!("PROBE-P5 total={} agree={agree} disagree={disagree} core_rejects={core_rejects} gpu_rule_rejects={gpu_rejects} disagreements={disagreements:?}", agree + disagree);
}
