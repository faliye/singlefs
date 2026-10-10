#!/usr/bin/env python3
"""攻方腿（m3-batch56-code-r1 opus）的变异：在仓副本上按名字做一处或几处原样替换（每个旧串在文件里恰好命中一次，否则退出 2）。
用法：mutants.py <副本根> <变异名>；mutants.py --list。"""
import sys
from pathlib import Path

CT = "crates/singlefs-core/src/container_tree.rs"
WALK = "crates/singlefs-checker/src/walk.rs"
IT = "crates/singlefs-core/src/inode_tree.rs"
REC = "crates/singlefs-core/src/recovery.rs"
MOUNT = "crates/singlefs-core/src/mount.rs"

MUTANTS = {
    # K2-2：写侧早关一片（旋钮 K ≥ 2 时装 K − 1 条就开新片；旋钮 1 不变；产品容量 815 / 1359 / 959 少装一条）。
    "w-close-one-early": [(CT,
        "            .is_some_and(|open| open.records.len() < records_per_container);",
        "            .is_some_and(|open| open.records.len() < records_per_container.saturating_sub(1).max(1));")],
    # K6：产品那一支（没装旋钮：I-9.21、I-9.36）不判第 56 批 G2 三句（第一片是 1、相邻加一、每片至少一条），装了旋钮那一支照判。
    "c-product-skips-three": [
        (WALK, "            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,",
               "            knob_records_per_container.is_none() || first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,"),
        (WALK, "            left_container.checked_add(1) == Some(right_container);",
               "            knob_records_per_container.is_none() || left_container.checked_add(1) == Some(right_container);"),
        (WALK, "        let lineage_or_retirement_container_holds_a_record = record_count > 0;",
               "        let lineage_or_retirement_container_holds_a_record = knob_records_per_container.is_none() || record_count > 0;"),
        (WALK, "            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,",
               "            capacity_knob.is_none() || first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,"),
        (WALK, "            left.container.checked_add(1) == Some(right.container);",
               "            capacity_knob.is_none() || left.container.checked_add(1) == Some(right.container);"),
        (WALK, "        let tombstone_container_holds_a_record = record_count > 0;",
               "        let tombstone_container_holds_a_record = capacity_knob.is_none() || record_count > 0;"),
    ],
    # K2-2 的改法臂（攻方提的、被攻过零轮）：装了旋钮时也判「不是最右的那片装满旋钮值」（D18 已定项 24 第 3 条「满了才关」在旋钮下的那一半）。
    "fix-knob-full": [
        (WALK, "            Some(records_the_knob_allows) => {\n                judgements.judge(invariant, record_count <= records_the_knob_allows, || {",
               "            Some(records_the_knob_allows) => {\n                let is_the_rightmost = position_in_leaf_order + 1 == container_numbers_and_record_counts.len();\n                judgements.judge(invariant, is_the_rightmost || record_count == records_the_knob_allows, || {\n                    format!(\"{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是旋钮值 {records_the_knob_allows}（opus 改法臂）\")\n                });\n                judgements.judge(invariant, record_count <= records_the_knob_allows, || {"),
        (WALK, "                let knob_records = knob.records_per_container();\n                judgements.judge(invariant, record_count <= knob_records, || {",
               "                let knob_records = knob.records_per_container();\n                let is_the_rightmost = position_in_leaf_order + 1 == containers.len();\n                judgements.judge(invariant, is_the_rightmost || record_count == knob_records, || {\n                    format!(\"{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是旋钮值 {knob_records}（opus 改法臂）\", container.container)\n                });\n                judgements.judge(invariant, record_count <= knob_records, || {"),
    ],
    # K1 / K6：第 56 批 G1 两行变异（冷走读、重建各把数记账条目排回逐条判代与 seq 之前），核它们在第 56 批的用例上红。
    "g1-cold-walk-revert": [(REC,
        "    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;",
        "    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }")],
    "g1-rebuild-revert": [(REC,
        "    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;",
        "    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }")],
    # K6：装了旋钮时只一片容器不判（谱系、退役那一支）。
    "c-knob-needs-two": [(WALK,
        "    let judged_when_the_tree_has_at_least = match knob_records_per_container {\n        None => 2,\n        Some(_) => 1,\n    };",
        "    let judged_when_the_tree_has_at_least = match knob_records_per_container {\n        None => 2,\n        Some(_) => 2,\n    };")],
    # K6：「相邻加一」改成「相邻不减」（同号放过）——谱系、退役那一支。
    "c-lr-adjacent-nondecreasing": [(WALK,
        "            left_container.checked_add(1) == Some(right_container);",
        "            left_container <= right_container;")],
    # K4：借的下限 < 117 改成 ≤ 117。
    "k4-floor-inclusive": [(IT,
        "        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()\n        {\n            return;\n        }",
        "        if records_in_the_borrower > records_below_which_a_leaf_borrows_from_its_right_neighbour()\n        {\n            return;\n        }")],
    # K4：借完从借方（不是借出的那一片）起并。
    "k4-absorb-from-borrower": [(IT,
        "        borrower.is_changed_this_publish = true;\n        self.absorb_until_no_pair_fits(lender_position);",
        "        borrower.is_changed_this_publish = true;\n        self.absorb_until_no_pair_fits(borrower_position);")],
    # K4：删一条之后并过也照样借（等价候选：并完的幸存片恰好 233 条，不低于下限）。
    "k4-borrow-even-after-absorbing": [(IT,
        "        let absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);\n        if !absorbed_at_least_once {",
        "        let _absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);\n        if holding_position < tree.containers.len() {")],
    # K4：先查左邻那一对、再查右邻（M4 那一臂的方向）。
    "k4-left-first": [(IT,
        "            if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else {",
        "            if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else {")],
    # K5：回退取树 ID 水位改成只取没被抛弃的根（第 56 批变异行那一形）。
    "k5-treeid-not-abandoned": [(MOUNT,
        "    ring_roots\n        .iter()\n        .map(|root| root.tree_identifier_watermark)\n        .fold(tree_identifier_watermark_of_the_current_version, u64::max)",
        "    ring_roots_not_abandoned_by_crash_recovery(ring_roots, _current_table_not_consulted)\n        .iter()\n        .map(|root| root.tree_identifier_watermark)\n        .fold(tree_identifier_watermark_of_the_current_version, u64::max)")],
}


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(sorted(MUTANTS)))
        return 0
    root, name = Path(sys.argv[1]), sys.argv[2]
    for relative, old, new in MUTANTS[name]:
        path = root / relative
        text = path.read_text()
        old, new = old.replace('\\n', '\n'), new.replace('\\n', '\n')
        if text.count(old) != 1:
            print(f"{name}: {relative} 里旧串命中 {text.count(old)} 次", file=sys.stderr)
            return 2
        path.write_text(text.replace(old, new))
    print(f"applied {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
