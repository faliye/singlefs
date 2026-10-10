#!/usr/bin/env python3
"""攻方腿（m3-batch58-code-r1 opus）的变异：在仓副本上按名字做一处或几处原样替换（每个旧串在文件里恰好命中一次，否则退出 2）。
名字可以用 + 连几种（依次打）；`row<N>` 取副本 crates/mutations.tsv 里第 N 行（文件行号，从 1 数）那一条变异。
用法：mutants.py <副本根> <名字>；mutants.py --list；mutants.py --rows <副本根>（列出第 58 批不碰 .wgsl 的行：行号、cargo 参数、必须红的测试）。"""
import sys
from pathlib import Path

WALK = "crates/singlefs-checker/src/walk.rs"
REC = "crates/singlefs-core/src/recovery.rs"

TOMBSTONE_RIGHTMOST = ("                let is_the_rightmost_tombstone_container_under_the_knob =\n"
                       "                    position_in_leaf_order + 1 == containers.len();")
LINEAGE_RIGHTMOST = ("                let is_the_rightmost_under_the_knob =\n"
                     "                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();")
REBUILD_ALLOCATION_JUDGE = "    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n"
REBUILD_ACCOUNTING_COUNT = ("    accounting_entry_count_is_three_plus_six_per_device(\n"
                            "        &accounting_tree.leaf_entries_in_key_order,\n")
REBUILD_ORDER_COMMENT = ("    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n")
REBUILD_MAPPING_START = ("    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。\n"
                         "    let mapping_tree = central_mapping_root.into_tree()?;\n"
                         "    mapping_entry_count_is_one_per_mapped_unit(\n"
                         "        mapping_tree.leaf_entries_in_key_order.len(),\n")


def knob_skip(rightmost_line, length):
    """把「是最右那片才跳过」改成别的跳过条件（length 是那一行里容器表的表达式）。"""
    head = rightmost_line.rsplit("\n", 1)[0]
    return lambda condition: (WALK, rightmost_line, head + "\n                    " + condition.format(n=length) + ";")


TOMB = knob_skip(TOMBSTONE_RIGHTMOST, "containers.len()")
LINE = knob_skip(LINEAGE_RIGHTMOST, "container_numbers_and_record_counts.len()")

MUTANTS = {
    # K1：旋钮支只判倒数第二片（墓碑与谱系 / 退役两个判法各一处，与 GPU 扫描内核那一个共用函数里同形的改坏对应）。
    "w-second-to-last": [TOMB("position_in_leaf_order + 2 != {n}"), LINE("position_in_leaf_order + 2 != {n}")],
    # K1（替身会红的自证）：旋钮支只判叶序第一片（第 57 批攻方 h1-first-only，第 58 批变异表 GPU 那一行的 CPU 同形）。
    "w-first-only": [TOMB("position_in_leaf_order + 1 == {n} || position_in_leaf_order > 0"),
                     LINE("position_in_leaf_order + 1 == {n} || position_in_leaf_order > 0")],
    # K1：旋钮支只判最右那片之前的两片（倒数第三、倒数第二；更靠左的不判）。
    "w-last-two": [TOMB("position_in_leaf_order + 1 == {n} || position_in_leaf_order + 3 < {n}"),
                   LINE("position_in_leaf_order + 1 == {n} || position_in_leaf_order + 3 < {n}")],
    # K3：重建（可写挂载）把分配记录逐条判挪到记账条目宽与逐条判之后、数记账条目之前（冷走读不动）。
    "r-rebuild-alloc-after-each-entry": [
        (REC, REBUILD_ALLOCATION_JUDGE, ""),
        (REC, REBUILD_ACCOUNTING_COUNT, REBUILD_ALLOCATION_JUDGE + REBUILD_ACCOUNTING_COUNT),
    ],
    # K3：重建里读完记账树就先数映射条目、再判分配记录，之后才是记账条目宽、逐条判、数记账条目（冷走读不动）。
    "r-rebuild-mapping-first": "special",
    # K3（上一轮的两种读者改法，次序不动）：逐条判记账条目时拒没登记的统计量标签；数记账条目时不算没登记标签的行。
    "r-tag-judged": [(REC,
        "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {",
        "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 || entry.statistic > STATISTIC_INODE_WATERMARK {")],
    "r-count-skips-unregistered-tag": [(REC,
        "                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg",
        "                (entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg)\n                    || entry.statistic > STATISTIC_INODE_WATERMARK")],
}


def rebuild_mapping_first(text):
    assert text.count(REBUILD_MAPPING_START) == 1 and text.count(REBUILD_ORDER_COMMENT) == 1 and text.count(REBUILD_ALLOCATION_JUDGE) == 1
    start = text.index(REBUILD_MAPPING_START)
    end = text.index("\n    )?;\n", start) + len("\n    )?;\n")
    block = text[start:end]
    text = text[:start] + text[end:]
    text = text.replace(REBUILD_ALLOCATION_JUDGE, "")
    return text.replace(REBUILD_ORDER_COMMENT, block + REBUILD_ALLOCATION_JUDGE + REBUILD_ORDER_COMMENT)


def tsv_rows(root):
    rows = []
    for number, line in enumerate((root / "crates/mutations.tsv").read_text().split("\n"), start=1):
        fields = line.split("\t")
        if line.startswith("第58批") and len(fields) == 6 and not fields[1].endswith(".wgsl"):
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
        if name == "r-rebuild-mapping-first":
            path = root / REC
            path.write_text(rebuild_mapping_first(path.read_text()))
            print(f"applied {name}")
            continue
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
