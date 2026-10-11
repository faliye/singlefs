#!/usr/bin/env python3
"""攻方腿（m3-batch59-code-r1 opus）的变异：在仓副本上按名字做一处或几处原样替换（每个旧串在文件里恰好命中一次，否则退出 2）。
名字可以用 + 连几种（依次打）；`row<N>` 取副本 crates/mutations.tsv 里第 N 行（文件行号，从 1 数）那一条变异（只认不碰 .wgsl 的行）。
用法：mutants.py <副本根> <名字>；mutants.py --list。"""
import sys
from pathlib import Path

WALK = "crates/singlefs-checker/src/walk.rs"
REC = "crates/singlefs-core/src/recovery.rs"

TOMBSTONE_RIGHTMOST = ("                let is_the_rightmost_tombstone_container_under_the_knob =\n"
                       "                    position_in_leaf_order + 1 == containers.len();")
LINEAGE_RIGHTMOST = ("                let is_the_rightmost_under_the_knob =\n"
                     "                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();")
EACH_ENTRY = "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {"


def knob_skip(rightmost_line, length):
    """把「是最右那片才跳过」改成别的跳过条件（length 是那一行里容器表的表达式）。"""
    head = rightmost_line.rsplit("\n", 1)[0]
    return lambda condition: (WALK, rightmost_line, head + "\n                    " + condition.format(n=length) + ";")


TOMB = knob_skip(TOMBSTONE_RIGHTMOST, "containers.len()")
LINE = knob_skip(LINEAGE_RIGHTMOST, "container_numbers_and_record_counts.len()")


def last_k(k):
    """旋钮支只判最右之前的 k 片（位置 n−1−k … n−2），更靠左的不判；k = 1 就是只判倒数第二片。"""
    condition = "position_in_leaf_order + 1 == {n} || position_in_leaf_order + " + str(k + 1) + " < {n}"
    return [TOMB(condition), LINE(condition)]


MUTANTS = {
    # 旋钮支只判倒数第二片（GPU 扫描内核 crates/mutations.tsv 第 3862–3864 行那一形的 CPU 同形；两个 CPU 判法各一处）。
    "w-second-to-last": [TOMB("position_in_leaf_order + 2 != {n}"), LINE("position_in_leaf_order + 2 != {n}")],
    # 替身会红的自证：旋钮支只判叶序第一片（第 58 批变异表 GPU 那一行的 CPU 同形）。
    "w-first-only": [TOMB("position_in_leaf_order + 1 == {n} || position_in_leaf_order > 0"),
                     LINE("position_in_leaf_order + 1 == {n} || position_in_leaf_order > 0")],
    # 旋钮支只判最右之前 k 片（k = 2 与第 59 批变异表第 3865–3867 行同形）。
    "w-last-two": last_k(2),
    "w-last-three": last_k(3),
    "w-last-four": last_k(4),
    # 上一轮（第 57 批）的两种读者改法，次序不动：逐条判记账条目时拒没登记的统计量标签；数记账条目时不算没登记标签的行。
    "r-tag-judged": [(REC, EACH_ENTRY,
        "    if entry.generation > root.checkpoint_txg || entry.sequence == 0 || entry.statistic > STATISTIC_INODE_WATERMARK {")],
    "r-count-skips-unregistered-tag": [(REC,
        "                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg",
        "                (entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg)\n                    || entry.statistic > STATISTIC_INODE_WATERMARK")],
    # M3：读者逐条判 I-3.13（记账行代段等于这一版的 txg，inode 号水位的台阶行不判）。冷走读与重建共用这一个逐条判，排在条目宽之后、数条目之前；
    # 报一个自己的说明，看得出红在哪一判。
    "r-each-entry-judges-i313": [(REC, EACH_ENTRY,
        "    if entry.generation != root.checkpoint_txg\n"
        "        && !(entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root.checkpoint_txg)\n"
        "        && entry.generation <= root.checkpoint_txg\n"
        "        && entry.sequence != 0\n"
        "    {\n"
        "        return Err(RecoveryFailure::InvariantViolated {\n"
        "            invariant: \"I-3.13\",\n"
        "            detail: \"记账行代段不等于这一版的 txg（inode 号水位的台阶行之外）\",\n"
        "        });\n"
        "    }\n" + EACH_ENTRY)],
}


def tsv_row(root, number):
    line = (root / "crates/mutations.tsv").read_text().split("\n")[number - 1]
    fields = line.split("\t")
    if len(fields) != 6 or fields[1].endswith(".wgsl"):
        raise SystemExit(f"第 {number} 行不是不碰 .wgsl 的六段变异行")
    return [(fields[1], fields[2], fields[3])]


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(sorted(MUTANTS)))
        return 0
    root = Path(sys.argv[1])
    for name in sys.argv[2].split("+"):
        edits = tsv_row(root, int(name[3:])) if name.startswith("row") else MUTANTS[name]
        for relative, old, new in edits:
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
