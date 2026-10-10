#!/usr/bin/env python3
"""攻方提的改法 RV（只在攻方副本上量过、被攻过零轮）：I-5.4 按版本那一层把「由记录施加出来的那几版」与「下一次挂载先施加的那一版」
也并进来——它们的树表指针与树表 0 条那一版的分配记录树根取记录新根段里的，判法与候选根同一套。

用法：patch_fix_rv.py <副本根>。每处旧串必须恰好命中一次，否则退 3、一个字节不写。
"""
import pathlib
import sys

target = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker/src/walk.rs"
text = target.read_text()
R = []
R.append(("""    candidate_indexes: &[usize],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();""",
"""    candidate_indexes: &[usize],
    versions_built_from_records: &[(String, u64, Vec<u8>)],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();"""))
R.append(("""    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;""",
"""    let versions: Vec<(String, u64, Vec<u8>, Vec<u8>)> = candidate_indexes
        .iter()
        .map(|index| {
            let root = &roots[*index].2;
            (
                format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                root.checkpoint_txg,
                root.record_bytes[36..122].to_vec(),
                root.record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT].to_vec(),
            )
        })
        .chain(versions_built_from_records.iter().map(|(name, txg, segment)| {
            (
                name.clone(),
                *txg,
                segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER].to_vec(),
                segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER].to_vec(),
            )
        }))
        .collect();
    for (version_name, version_txg, tree_table_pointer_bytes, allocation_root_bytes) in &versions {
        let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;"""))
R.append(("""        let mut allocation_roots = vec![parse_node_pointer(
            &root.record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
        )];
        let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);""",
"""        let mut allocation_roots = vec![parse_node_pointer(allocation_root_bytes)];
        let tree_table_pointer = parse_node_pointer(tree_table_pointer_bytes);"""))
R.append(("""                versions_whose_tree_table_does_not_read_back.push(format!(
                    "根（实例 {}、txg {}）",
                    root.instance, root.checkpoint_txg
                ));""",
"""                versions_whose_tree_table_does_not_read_back.push(version_name.clone());"""))
R.append(("""                                "根（实例 {}、txg {}）的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
                                root.instance,
                                root.checkpoint_txg,
""",
"""                                "{}的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
                                version_name,
"""))
R.append(("""            judge_allocation_record_ranges(&records, root.checkpoint_txg, judgements);""",
"""            judge_allocation_record_ranges(&records, *version_txg, judgements);"""))
R.append(("""            versions_where_the_root_record_allocation_tree_root_is_zero.push((
                format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),""",
"""            versions_where_the_root_record_allocation_tree_root_is_zero.push((
                version_name.clone(),"""))
R.append(("""            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(
                "根（实例 {}、txg {}）",
                root.instance, root.checkpoint_txg
            ));""",
"""            versions_whose_root_record_allocation_record_tree_does_not_read_back
                .push(version_name.clone());"""))
R.append(("""    judge_allocation_records_disjoint(
        reader,
        &roots,
        &candidate_indexes,
        &mut index_node_cache,""",
"""    let mut versions_built_from_records_for_the_disjointness: Vec<(String, u64, Vec<u8>)> =
        versions_applied_only_by_records
            .iter()
            .map(|version| {
                (
                    format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    version.checkpoint_txg,
                    version.new_root_segment.clone(),
                )
            })
            .collect();
    versions_built_from_records_for_the_disjointness.extend(
        new_root_segment_of_the_version_the_next_mount_applies_first(
            &journal_records_by_device,
            &roots,
            newest_index,
        ),
    );
    judge_allocation_records_disjoint(
        reader,
        &roots,
        &candidate_indexes,
        &versions_built_from_records_for_the_disjointness,
        &mut index_node_cache,"""))
R.append(("""fn tree_table_pointer_of_the_version_the_next_mount_applies_first(""",
"""/// 改法 RV：同 [`tree_table_pointer_of_the_version_the_next_mount_applies_first`]，交回整段新根段与它的名字、txg。
fn new_root_segment_of_the_version_the_next_mount_applies_first(
    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
) -> Option<(String, u64, Vec<u8>)> {
    let newest_root = &roots[newest_index].2;
    let next_txg = newest_root.checkpoint_txg.checked_add(1)?;
    if roots.iter().any(|(_, _, root)| root.instance == newest_root.instance && root.checkpoint_txg == next_txg) {
        return None;
    }
    let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
    records_as_the_recovery_reads_them
        .iter()
        .find(|((_, counter), record)| {
            record.commit_marker == CommitMarker::Present
                && record.instance == newest_root.instance
                && record.checkpoint_txg == next_txg
                && record_can_enter_a_replay_prefix(&records_as_the_recovery_reads_them, *counter, record)
        })
        .map(|(_, record)| {
            (
                format!("下一次挂载先施加的那一版（实例 {}、txg {}）", record.instance, record.checkpoint_txg),
                record.checkpoint_txg,
                record.new_root_segment.clone(),
            )
        })
}

fn tree_table_pointer_of_the_version_the_next_mount_applies_first("""))
for old, new in R:
    hits = text.count(old)
    if hits != 1:
        print(f"旧串命中 {hits} 次：{old[:80]!r}", file=sys.stderr)
        sys.exit(3)
    text = text.replace(old, new)
target.write_text(text)
print(f"patched {target} ({len(R)} 处)")
