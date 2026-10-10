#!/usr/bin/env python3
"""攻方腿（m3-batch52-54-code-r1 opus）的变异与改法：每一种在一份仓副本上做几处定点替换（旧串在文件里恰好一次，否则退出 3）。

用法：mutants.py list                  列出名字
      mutants.py apply <名字> <副本根>  在副本上改
      mutants.py tests <名字>           打印这一种要跑的命令（一行一条：cargo 参数 \t 过滤）
"""
import sys
from pathlib import Path

CT = "crates/singlefs-core/src/container_tree.rs"
IT = "crates/singlefs-core/src/inode_tree.rs"
MT = "crates/singlefs-core/src/mount.rs"
RC = "crates/singlefs-core/src/recovery.rs"
WK = "crates/singlefs-checker/src/walk.rs"

K2_NUMBERS = [
    "--release -p singlefs-harness --test opus_probe_k2_k4\topus_probe_k2_container_numbers_under_the_knobs",
    "--release -p singlefs-core --lib\t_tree::",
]
K4_TESTS = [
    "--release -p singlefs-core --lib\tinode_tree::",
    "--release -p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference\t",
]
K1_TESTS = [
    "--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking\topus_probe",
    "--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking\t--skip opus_probe",
]

EAGER_OLD = """        let rewritten_position = containers.len() - 1;
        if rewritten_containers.last() != Some(&rewritten_position) {
            rewritten_containers.push(rewritten_position);
        }
"""
EAGER_NEW = EAGER_OLD + """        if containers.last().is_some_and(|open| open.records.len() == records_per_container) {
            let next_container_number = containers.last().map_or(1, |rightmost| rightmost.identity.container + 1);
            containers.push(RecordContainer {
                identity: PackedIdentity { birth_tree: tree, record_type, container: next_container_number, container_birth: checkpoint_txg },
                records: Vec::new(),
            });
            shape = shape.after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node());
            rewritten_containers.push(containers.len() - 1);
        }
"""

WIDTH_REBUILD = """    every_accounting_entry_is_as_wide_as_its_field_table(
        &accounting_tree.leaf_entries_in_key_order,
    )?;
"""
WIDTH_WALK = """    every_accounting_entry_is_as_wide_as_its_field_table(
        &roots.accounting.leaf_entries_in_key_order,
    )?;
"""
B53_NEW = """    let inode_number_watermark = inode_number_watermark_of_the_rollback(
        &ring_roots,
        &current_table,
        current.inode_number_watermark(),
        |ring_roots_not_abandoned| {
            highest_inode_number_watermark_in_the_ring(
                devices,
                unit_area_start,
                ring_roots_not_abandoned,
                &current.record,
            )
        },
    );
"""
B53_OLD = """    let inode_number_watermark = highest_inode_number_watermark_in_the_ring(
        devices,
        unit_area_start,
        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),
        &current.record,
    )
    .map_or(current.inode_number_watermark(), |ring| {
        ring.max(current.inode_number_watermark())
    });
"""

VARIANTS = {
    # K2.2：写侧改坏容器号，看池级 checker（I-9.36 / I-9.37 / I-9.21 / I-9.22）红不红
    "w-gap2": ([(CT, ".map_or(1, |rightmost| rightmost.identity.container + 1);", ".map_or(1, |rightmost| rightmost.identity.container + 2);")], K2_NUMBERS),
    "w-start2": ([(CT, ".map_or(1, |rightmost| rightmost.identity.container + 1);", ".map_or(2, |rightmost| rightmost.identity.container + 1);")], K2_NUMBERS),
    "w-same": ([(CT, ".map_or(1, |rightmost| rightmost.identity.container + 1);", ".map_or(1, |rightmost| rightmost.identity.container);")], K2_NUMBERS),
    "w-eager": ([(CT, EAGER_OLD, EAGER_NEW)], K2_NUMBERS),
    # K6：I-9.36 / I-9.37 的「容器号沿叶序严格递增」那一半恒真，整份坏镜像用例与容器树长高用例红不红
    "c-incr-true": ([(WK, "judgements.judge(invariant, left_container < right_container, || {", "judgements.judge(invariant, true, || {")],
                    ["--release -p singlefs-harness --test checker_known_bad_images\t",
                     "--release -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full\t"]),
    # K4：合并与借的几种改坏，core 单测与第 53 批参照用例红不红
    "k4-no-absorb-after-borrow": ([(IT, "        self.absorb_until_no_pair_fits(lender_position);\n", "")], K4_TESTS),
    "k4-floor-116": ([(IT, 'usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")', 'usize::try_from(INODE_LEAF_RECORDS / 2).expect("117")')], K4_TESTS),
    "k4-borrow-ceil": ([(IT, ".map_or(0, |difference| difference / 2);", ".map_or(0, |difference| difference.div_ceil(2));")], K4_TESTS),
    "k4-drop-start-right": ([(IT, "tree.absorb_until_no_pair_fits(holding_position - 1);", "tree.absorb_until_no_pair_fits(holding_position);")], K4_TESTS),
    "k4-no-chain": ([(IT, "            absorbed_at_least_once = true;\n        }\n    }", "            return true;\n        }\n    }")], K4_TESTS),
    "k4-no-borrow": ([(IT, "            tree.borrow_from_the_right_neighbour_if_below_the_floor(holding_position);", "            let _ = holding_position;")], K4_TESTS),
    # K3：树 ID 水位改成只取没被抛弃的根（条款说照旧取全部），回退那几份用例红不红；第 53 批那一段改回内联，摘要比
    "k3-treeid-not-abandoned": ([(MT, "    let tree_identifier_watermark = ring_roots\n        .iter()", "    let tree_identifier_watermark = ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table)\n        .iter()")],
                                ["--release -p singlefs-harness --test rollback_by_a_forward_publish\t--include-ignored",
                                 "--release -p singlefs-harness --test tree_identifier_watermark_after_the_first_file_root_is_abandoned\t",
                                 "--release -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over\t"]),
    "k3-revert-b53": ([(MT, B53_NEW, B53_OLD)], ["--release -p singlefs-harness --test opus_probe_k2_k4\topus_probe_k3_rollback_history_digest"]),
    # K1：第 52 批两处条目宽之判拿掉（改之前的样子）；攻方改法：逐条判代与 seq 提到数条目之前
    "k1-revert-b52": ([(RC, WIDTH_REBUILD, ""), (RC, WIDTH_WALK, "")], K1_TESTS),
    "k1-fix-judge-entries-before-count": ([
        (RC, WIDTH_REBUILD, "    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(bytes, root)?;\n    }\n"),
        (RC, WIDTH_WALK, "    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n"),
    ], K1_TESTS + ["--release -p singlefs-harness --test writable_mount_of_a_formatted_pool\t"]),
}


def main() -> int:
    if len(sys.argv) >= 2 and sys.argv[1] == "list":
        print("\n".join(VARIANTS))
        return 0
    if len(sys.argv) == 4 and sys.argv[1] == "apply":
        edits, _ = VARIANTS[sys.argv[2]]
        root = Path(sys.argv[3])
        for relative, old, new in edits:
            path = root / relative
            text = path.read_text()
            if text.count(old) != 1:
                print(f"{sys.argv[2]}：{relative} 里旧串出现 {text.count(old)} 次，不是 1 次", file=sys.stderr)
                return 3
            path.write_text(text.replace(old, new))
        return 0
    if len(sys.argv) == 3 and sys.argv[1] == "tests":
        print("\n".join(VARIANTS[sys.argv[2]][1]))
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
