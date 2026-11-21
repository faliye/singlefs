use super::*;

/// 5.2 γ/η（D18 已定项 2 射程、D8 已定项 14 第 401 行，唯一写法：写路径与读路径共用）：extent 上段叶
/// 第 k 片的 key 区间，第七节 B12 锚点的四个 inode 点（1、142、143、286）。
#[test]
fn extent_upper_leaf_positional_key_range_matches_the_frozen_anchors() {
    for (inode, expected_upper_leaf_index, expected_smallest_inode, expected_largest_inode) in
        [(1u64, 0u64, 0u64, 142u64), (142, 0, 0, 142), (143, 1, 143, 285), (286, 2, 286, 428)]
    {
        let upper_leaf_index = extent_upper_leaf_index(inode);
        assert_eq!(upper_leaf_index, expected_upper_leaf_index, "inode={inode}");
        let (smallest, largest) = extent_upper_leaf_positional_key_range(upper_leaf_index);
        let mut expected_smallest = [0u8; 24];
        expected_smallest[8..16].copy_from_slice(&expected_smallest_inode.to_le_bytes());
        let mut expected_largest = [0u8; 24];
        expected_largest[8..16].copy_from_slice(&expected_largest_inode.to_le_bytes());
        expected_largest[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(smallest, expected_smallest.to_vec(), "inode={inode}：smallest_key");
        assert_eq!(largest, expected_largest.to_vec(), "inode={inode}：largest_key（第三分量恒 2^64−1，第七节 A7）");
    }
}

// E142 第十五次跑步④：`compare_region`/`RegionComparison`（连同它们的 6 条单测、`snapshot_fields` 辅助函数）
// 被 Q142.1 的新比对器取代——新比对器按 (设备, 偏移, 长度) 配对，crates 导出恒发整段十六进制（窗口里最大
// 一次写 32768 字节，摆得出），不再需要「declared length 不等」「head_and_tail 抽样」这两种旧读法，
// 直接删掉旧函数与旧测试（`code-discipline.md`「用不到的代码删掉」），不留驻留的死代码。

// ───────── E142 第十五次跑步④：Q142.1 新比对器的单测（第九节 M108、M109 钉在这两条函数上） ─────────

fn sample_impl_line(device: u32, offset: u64, length: u64, sha256: &str) -> ParsedWriteLine {
    let line = format!("E7RESULT name=device_region_bytes step=0 device={device} offset={offset} length={length} kind=write sha256={sha256} hexadecimal=00");
    let fields = parse_result_line(&line);
    ParsedWriteLine {
        device: fields.get("device").and_then(|value| value.parse().ok()).unwrap_or(0),
        offset: fields.get("offset").and_then(|value| value.parse().ok()).unwrap_or(0),
        length: fields.get("length").and_then(|value| value.parse().ok()).unwrap_or(0),
        sha256: fields.get("sha256").cloned().unwrap_or_default(),
        hexadecimal: fields.get("hexadecimal").cloned(),
    }
}

#[test]
fn find_matching_impl_write_tells_two_devices_at_the_same_offset_apart() {
    // M108：按名字配对退回旧「量 5」的失败模式——两块设备在同一偏移各写不同内容，按名字配对分不清是哪一块。
    // 这里干脆没有名字字段，配对键就是 (设备, 偏移, 长度)；device=0 与 device=1 在同一偏移各有一条。
    let impl_writes = vec![
        sample_impl_line(0, 100, 10, "sha-for-device-0"),
        sample_impl_line(1, 100, 10, "sha-for-device-1"),
    ];
    assert_eq!(find_matching_impl_write(&impl_writes, 0, 100, 10), Some(0));
    assert_eq!(find_matching_impl_write(&impl_writes, 1, 100, 10), Some(1));
}

#[test]
fn find_matching_impl_write_refuses_a_length_mismatch_at_the_same_device_and_offset() {
    let impl_writes = vec![sample_impl_line(0, 100, 10, "sha")];
    assert_eq!(find_matching_impl_write(&impl_writes, 0, 100, 20), None, "长度不等就是配不上，不是「差不多就算」");
}

#[test]
fn find_matching_impl_write_reports_none_when_nothing_matches() {
    let impl_writes = vec![sample_impl_line(0, 100, 10, "sha")];
    assert_eq!(find_matching_impl_write(&impl_writes, 0, 999, 10), None);
}

#[test]
fn unmatched_crates_indices_lists_every_index_the_model_never_claimed() {
    // M109：比对器只从模型一侧遍历会漏掉「crates 多写了一次」——这条函数就是「配不上的两边都报」里 crates 那一半。
    assert_eq!(unmatched_crates_indices(&[true, false, false, true]), vec![1, 2]);
}

#[test]
fn unmatched_crates_indices_is_empty_when_every_impl_write_was_claimed() {
    assert_eq!(unmatched_crates_indices(&[true, true, true]), Vec::<usize>::new());
}

#[test]
fn compare_paired_write_reports_equal_when_sha256_matches() {
    let model_bytes = b"same bytes on both sides".to_vec();
    let sha = sha256_hex(&model_bytes);
    let impl_line = sample_impl_line(0, 0, model_bytes.len() as u64, &sha);
    assert_eq!(compare_paired_write(&model_bytes, &sha, &impl_line), (true, None, Some(0)));
}

#[test]
fn compare_paired_write_localizes_the_first_difference_when_sha256_differs() {
    let model_bytes = vec![1u8, 2, 3, 4];
    let impl_bytes = vec![1u8, 2, 9, 4];
    let mut impl_line = sample_impl_line(0, 0, 4, "different_from_model");
    impl_line.hexadecimal = Some(hex_bytes(&impl_bytes));
    assert_eq!(compare_paired_write(&model_bytes, &sha256_hex(&model_bytes), &impl_line), (false, Some(2), Some(1)));
}
