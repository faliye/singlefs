#!/usr/bin/env python3
"""核 crates/mutations.tsv 里指定行的原文在源文件里恰好命中一次（mutate.sh 的锚点规则），并可落出变异后的源文件。
用法：anchor_check.py <树根> <行号起> <行号止> [--emit <输出目录>]"""
import sys, os
tree, lo, hi = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
emit = sys.argv[sys.argv.index("--emit") + 1] if "--emit" in sys.argv else None
lines = open(os.path.join(tree, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
for no in range(lo, hi + 1):
    parts = lines[no - 1].split("\t")
    name, path, frm, to = parts[0], parts[1], parts[2].replace("\\n", "\n"), parts[3].replace("\\n", "\n")
    src = open(os.path.join(tree, path), encoding="utf-8").read()
    n = src.count(frm)
    print(f"tsv:{no} 命中 {n} 次  {path}  {name[:60]}")
    if emit and n == 1:
        os.makedirs(emit, exist_ok=True)
        open(os.path.join(emit, f"row{no}.rs"), "w", encoding="utf-8").write(src.replace(frm, to))
