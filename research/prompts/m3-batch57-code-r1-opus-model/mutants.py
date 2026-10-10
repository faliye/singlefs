#!/usr/bin/env python3
"""攻方腿（m3-batch57-code-r1 opus）的变异：在仓副本上按名字做一处或几处原样替换（每个旧串在文件里恰好命中一次，否则退出 2）。
名字可以用 + 连几种（依次打）；`row<N>` 取副本 crates/mutations.tsv 里第 N 行（文件行号，从 1 数）那一条变异。
用法：mutants.py <副本根> <名字>；mutants.py --list；mutants.py --rows <副本根>（列出第 57 批不碰 .wgsl 的行：行号、cargo 参数、必须红的测试）。"""
import sys
from pathlib import Path

WALK = "crates/singlefs-checker/src/walk.rs"
REC = "crates/singlefs-core/src/recovery.rs"

MAPPING_COUNT_OF_THE_COLD_WALK = (
    "    mapping_entry_count_is_one_per_mapped_unit(\n"
    "        roots.mapping.leaf_entries_in_key_order.len(),\n"
    "        MappedUnitCounts {\n"
    "            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),\n"
    "            allocation_record_tree_nodes: roots.allocation.nodes.len(),\n"
    "            accounting_tree_nodes: roots.accounting.version.node_count(),\n"
    "            inode_leaf_containers: roots.inode.entries.len(),\n"
    "            data_units: roots.extent.data_pointers.len(),\n"
    "            deletion_tree_nodes,\n"
    "        },\n"
    "    )?;\n")
ALLOCATION_JUDGE_OF_THE_COLD_WALK = "    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n"

MUTANTS = {
    # K3：冷走读把「数映射条目」挪到「分配记录逐条判代与跨度」之前（第 57 批 H4 说分配记录排在两个数之前，用例只钉了记账条目数那一个）。
    "k3-cold-walk-mapping-count-first": [
        (REC, "    )?;\n" + MAPPING_COUNT_OF_THE_COLD_WALK + "\n    // inode 树：根", "    )?;\n\n    // inode 树：根"),
        (REC, ALLOCATION_JUDGE_OF_THE_COLD_WALK, MAPPING_COUNT_OF_THE_COLD_WALK + ALLOCATION_JUDGE_OF_THE_COLD_WALK),
    ],
    # K3：逐条判记账条目时也拒没登记的统计量标签（> inode 号水位 12）——H4 用例在正确次序下照绿吗、挪回之后还红吗。
    "k3-tag-judged": [(REC,
        "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {",
        "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 || entry.statistic > STATISTIC_INODE_WATERMARK {")],
    # K3：数记账条目时把没登记标签的行也不算（与台阶行同样不算）——H4 用例分辨次序靠的是这一行被算进去。
    "k3-count-skips-unregistered-tag": [(REC,
        "                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg",
        "                (entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg)\n                    || entry.statistic > STATISTIC_INODE_WATERMARK")],
    # K4：H1 新判只判叶序第一片（不是最右的其余各片不判），墓碑与谱系 / 退役两个判法各一处。
    "h1-first-only": [
        (WALK, "                if is_the_rightmost_under_the_knob {\n                    continue;\n                }",
               "                if is_the_rightmost_under_the_knob || position_in_leaf_order > 0 {\n                    continue;\n                }"),
        (WALK, "                if is_the_rightmost_tombstone_container_under_the_knob {\n                    continue;\n                }",
               "                if is_the_rightmost_tombstone_container_under_the_knob || position_in_leaf_order > 0 {\n                    continue;\n                }"),
    ],
    # K2：整镜像那条路上没装旋钮的那一棵不交容量判法（交空表，判法零片返回）；直接调判法的单测碰不到调用点。
    "k2-image-path-skips-product": [
        (WALK, "                *lineage_tree_identifier,\n                &containers\n                    .iter()\n",
               "                *lineage_tree_identifier,\n                &containers\n                    .iter()\n                    .filter(|_| lineage_capacity_knob.is_some())\n"),
        (WALK, "            LineageOrRetirementTree::Retirement,\n            retirement_tree.tree,\n            &containers\n                .iter()\n",
               "            LineageOrRetirementTree::Retirement,\n            retirement_tree.tree,\n            &containers\n                .iter()\n                .filter(|_| retirement_capacity_knob.is_some())\n"),
        (WALK, "            tombstone_tree.tree,\n            containers,\n            capacity_knob,\n            judgements,\n        );\n        judge_every_tombstone_of_a_version_is_well_formed(",
               "            tombstone_tree.tree,\n            if capacity_knob.is_some() { containers } else { &[] },\n            capacity_knob,\n            judgements,\n        );\n        judge_every_tombstone_of_a_version_is_well_formed("),
    ],
}


def tsv_rows(root):
    rows = []
    for number, line in enumerate((root / "crates/mutations.tsv").read_text().split("\n"), start=1):
        fields = line.split("\t")
        if line.startswith("第57批") and len(fields) == 6 and not fields[1].endswith(".wgsl"):
            rows.append((number, fields))
    return rows


def edits_of(root, name):
    if name.startswith("row"):
        number = int(name[3:])
        fields = dict(tsv_rows(root))[number]
        return [(fields[1], fields[2], fields[3])]
    return MUTANTS[name]


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(sorted(MUTANTS)))
        return 0
    if sys.argv[1] == "--rows":
        for number, fields in tsv_rows(Path(sys.argv[2])):
            print(f"{number}\t{fields[4]}\t{fields[5]}")
        return 0
    root = Path(sys.argv[1])
    for name in sys.argv[2].split("+"):
        for relative, old, new in edits_of(root, name):
            path = root / relative
            text = path.read_text()
            old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
            if text.count(old) != 1:
                print(f"{name}: {relative} 里旧串命中 {text.count(old)} 次", file=sys.stderr)
                return 2
            path.write_text(text.replace(old, new))
        print(f"applied {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
