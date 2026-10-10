#!/usr/bin/env python3
"""冻结副本里 crates/mutations.tsv 指向几份被动过的源文件的每一行，原文在源文件里命中几次（mutate.sh 要求恰好一次）。"""
import collections, sys
tree = sys.argv[1].rstrip("/") + "/"
rows = open(tree + "crates/mutations.tsv", encoding="utf-8").read().split("\n")
targets = ["crates/singlefs-checker-tier/src/crash_injection.rs", "crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl",
           "crates/singlefs-checker/src/image.rs", "crates/singlefs-harness/src/model.rs", "crates/singlefs-core/src/mount.rs"]
sources = {p: open(tree + p, encoding="utf-8").read() for p in targets}
count = {p: collections.Counter() for p in targets}
bad = []
for no, line in enumerate(rows, 1):
    part = line.split("\t")
    if len(part) < 4 or part[1] not in sources:
        continue
    n = sources[part[1]].count(part[2].replace("\\n", "\n"))
    count[part[1]][n] += 1
    if n != 1:
        bad.append((no, n))
for p in targets:
    print(p.split("/")[-1], "命中次数分布", dict(count[p]))
print("命中次数不是 1 的行：", bad)
