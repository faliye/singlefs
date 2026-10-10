
// ─── 攻方原型（m3-batch52-54-code-r1 opus，K1）：台阶行的代挪到根之后 ───

/// 记账叶根（单个叶根）里每条条目按 (统计量, 树, 盘, 代) 解开、`change` 改完之后按同一个 key 重排，头里的最小 / 最大 key 换成首末两条的 key。
fn opus_probe_rewrite_accounting_leaf_entries(node: &mut [u8], change: impl FnOnce(&mut Vec<Vec<u8>>)) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([node[header_end - 4], node[header_end - 3]]));
    let ranges: Vec<std::ops::Range<usize>> =
        (0..entry_count).map(|index| entry_range_in_node(node, index)).collect();
    let mut entries: Vec<Vec<u8>> = ranges.iter().map(|range| node[range.clone()].to_vec()).collect();
    change(&mut entries);
    assert_eq!(entries.len(), entry_count, "条数不变");
    let sort_key = |entry: &Vec<u8>| {
        (
            u16::from_le_bytes([entry[0], entry[1]]),
            u64::from_le_bytes(entry[2..10].try_into().expect("8")),
            u32::from_le_bytes(entry[10..14].try_into().expect("4")),
            u64::from_le_bytes(entry[14..22].try_into().expect("8")),
        )
    };
    entries.sort_by_key(sort_key);
    for (range, entry) in ranges.iter().zip(&entries) {
        node[range.clone()].copy_from_slice(entry);
    }
    let first = entries.first().expect("至少一条")[..key_width].to_vec();
    let last = entries.last().expect("至少一条")[..key_width].to_vec();
    node[52..52 + key_width].copy_from_slice(&first);
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&last);
}

fn opus_probe_entry_field_u64(entry: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(entry[offset..offset + 8].try_into().expect("8"))
}

/// txg 4 那一版的记账叶根里，满足 `pick` 的那一条（恰好一条）交给 `change` 改。
fn opus_probe_pool_with_one_accounting_entry_changed(
    pick: impl Fn(&[u8]) -> bool,
    change: impl Fn(&mut Vec<u8>),
) -> Devices {
    let mut devices = pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(&mut devices, TREE_KIND_ACCOUNTING, |node| {
        opus_probe_rewrite_accounting_leaf_entries(node, |entries| {
            let picked: Vec<usize> =
                (0..entries.len()).filter(|index| pick(&entries[*index])).collect();
            assert_eq!(picked.len(), 1, "恰好一条");
            change(&mut entries[picked[0]]);
        });
    });
    devices
}

fn opus_probe_is_the_step_row(entry: &[u8]) -> bool {
    u16::from_le_bytes([entry[0], entry[1]]) == STATISTIC_INODE_WATERMARK
        && opus_probe_entry_field_u64(entry, 14) < 4
}

fn opus_probe_is_the_current_watermark_row(entry: &[u8]) -> bool {
    u16::from_le_bytes([entry[0], entry[1]]) == STATISTIC_INODE_WATERMARK
        && opus_probe_entry_field_u64(entry, 14) == 4
}

fn opus_probe_report(name: &str, devices: &mut Devices) {
    let report = recover(devices, JournalPolicy::Consult);
    let refusal = match mount_writable(&parameters(), devices) {
        Ok(mounted) => format!(
            "收下（实例 {:?}、txg {:?}）",
            mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg
        ),
        Err(refusal) => format!("{refusal:?}"),
    };
    let outcome = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead {{ root: {root:?}, content_bytes: {} }}", content.len()),
        other => format!("{other:?}"),
    };
    eprintln!("OPUS_K1 case={name} recover={outcome}");
    eprintln!("OPUS_K1 case={name} writable_mount={refusal}");
}

#[test]
fn opus_probe_k1_members_of_the_accounting_faults_around_the_step_row() {
    // ① 对照：不改。
    let mut clean = pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    opus_probe_report("clean", &mut clean);
    // ② 台阶行（代 3）的代改成 5（> 根的 txg 4），按 key 重排到最后。
    let mut step_after_root = opus_probe_pool_with_one_accounting_entry_changed(opus_probe_is_the_step_row, |entry| {
        entry[14..22].copy_from_slice(&5u64.to_le_bytes());
    });
    opus_probe_report("step_row_generation_5_after_root_4", &mut step_after_root);
    // ③ 台阶行 seq 改成 0。
    let mut step_seq_zero = opus_probe_pool_with_one_accounting_entry_changed(opus_probe_is_the_step_row, |entry| {
        entry[30..34].copy_from_slice(&0u32.to_le_bytes());
    });
    opus_probe_report("step_row_sequence_0", &mut step_seq_zero);
    // ④ 这一版自己那一行水位（代 4）的代改成 5。
    let mut current_after_root = opus_probe_pool_with_one_accounting_entry_changed(opus_probe_is_the_current_watermark_row, |entry| {
        entry[14..22].copy_from_slice(&5u64.to_le_bytes());
    });
    opus_probe_report("current_watermark_row_generation_5", &mut current_after_root);
    // ⑤ 池级那一行统计量 4（不带盘维）的代改成 5。
    let mut pool_row_after_root = opus_probe_pool_with_one_accounting_entry_changed(
        |entry| u16::from_le_bytes([entry[0], entry[1]]) == 4 && u32::from_le_bytes(entry[10..14].try_into().expect("4")) == 0xFFFF_FFFF,
        |entry| entry[14..22].copy_from_slice(&5u64.to_le_bytes()),
    );
    opus_probe_report("statistic_4_pool_row_generation_5", &mut pool_row_after_root);
    // ⑥ 这一版自己那一行水位的代改成 2（< 3，成了第二级台阶，这一版自己那一行没了）。
    let mut current_turned_step = opus_probe_pool_with_one_accounting_entry_changed(opus_probe_is_the_current_watermark_row, |entry| {
        entry[14..22].copy_from_slice(&2u64.to_le_bytes());
    });
    opus_probe_report("current_watermark_row_generation_2", &mut current_turned_step);
}
