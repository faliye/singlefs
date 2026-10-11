#!/usr/bin/env python3
"""攻方腿（m3-batch60-61-code-r1 opus）的变异：在仓副本上按名字做一处原样替换（旧串在文件里恰好命中一次，否则退出 2）。
名字可以用 + 连几种（依次打）；`tsv<N>` 取副本 crates/mutations.tsv 第 N 行那一条。用法：mutants.py <副本根> <名字>；mutants.py --list。"""
import sys
from pathlib import Path

AMP = "crates/singlefs-checker-tier/src/crash_amplification.rs"
STORE = "crates/singlefs-checker-tier/src/verdict_store.rs"

MUTANTS = {
    # 流水线导入那一段排键时一行核对行都不点（两种核对代码都漏）：被判用例与第 61 批变异表都不看核对行
    "mv": [(AMP, "                for verifier_digest in &verifier_digests {",
            "                for verifier_digest in verifier_digests.iter().skip(verifier_digests.len()) {")],
    # 流水线导入那一趟的 imported_blocks 只数写下的、不数本库已有且相同的（含义回到「写了几块」）
    "mi-pipe": [(AMP, "            imported_blocks +=\n                tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;",
                 "            imported_blocks +=\n                tally.conclusion_blocks_written + 0 * tally.conclusion_blocks_already_identical;")],
    # 给定状态表那一路的 imported_blocks 同上
    "mi-given": [(AMP, "                imported_blocks +=\n                    tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;",
                  "                imported_blocks +=\n                    tally.conclusion_blocks_written + 0 * tally.conclusion_blocks_already_identical;")],
    # 不是变异，是 resume 阶段的量法：按计划导入的批长从 64 MiB 改成 256 KiB，小库上也攒出十几批（批的边界与写法不动）
    "rb": [(STORE, "const PLANNED_IMPORT_BATCH_BYTES: usize = 64 << 20;", "const PLANNED_IMPORT_BATCH_BYTES: usize = 256 << 10;")],
    # 流水线导入那一段一条路径都不点（推的等价变异：本机录入时已登了同一份路径）
    "mp": [(AMP, "                .map(|point| point.path_number.clone())\n                .collect(),\n            verifier_blocks: Vec::new(),",
            "                .map(|point| point.path_number.clone())\n                .filter(|_| false)\n                .collect(),\n            verifier_blocks: Vec::new(),")],
}


def main():
    if sys.argv[1:] == ["--list"]:
        print("\n".join(sorted(MUTANTS)))
        return 0
    root, names = Path(sys.argv[1]), sys.argv[2].split("+")
    for name in names:
        if name.startswith("tsv"):
            # tsv<N>：副本 crates/mutations.tsv 第 N 行（文件行号，从 1 数）那一条变异；原文与替换文里的 \n 是换行
            columns = (root / "crates/mutations.tsv").read_text().splitlines()[int(name[3:]) - 1].split("\t")
            edits = [(columns[1], columns[2].replace("\\n", "\n"), columns[3].replace("\\n", "\n"))]
        else:
            edits = MUTANTS[name]
        for relative, old, new in edits:
            path = root / relative
            text = path.read_text()
            if text.count(old) != 1:
                print(f"变异 {name}：{relative} 里旧串命中 {text.count(old)} 次", file=sys.stderr)
                return 2
            path.write_text(text.replace(old, new))
            print(f"变异 {name}：{relative} 已替换")
    return 0


sys.exit(main())
