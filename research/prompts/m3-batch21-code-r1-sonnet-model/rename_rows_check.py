#!/usr/bin/env python3
# N4：6 条对拍用例改名连带 11 行变异替换——把前几批 diff 材料里带旧名的变异行逐行改名后，与冻结副本变异表里的新行比（原文、替换文、参数、点名用例逐字相同）。
# 用法：SINGLEFS_REPO=… FROZEN_TREE=… python3 rename_rows_check.py
import os
repo = os.environ["SINGLEFS_REPO"]; tree = os.environ.get("FROZEN_TREE", "/tmp/claude-1000/m3-batch21-code-r1/tree")
new = {}
for l in open(f"{tree}/crates/mutations.tsv", encoding="utf-8").read().split("\n"):
    c = l.split("\t")
    if len(c) == 6:
        new[tuple(c[:4])] = c
ok = bad = 0
for f in ("_m3-batch18-code-r1-diff.md", "_m3-batch20-code-r1-diff.md"):
    for l in open(f"{repo}/research/prompts/{f}", encoding="utf-8").read().split("\n"):
        c = l.split("\t")
        if len(c) == 6 and "the_gpu_checker_agrees_with_the_cpu" in l:
            r = lambda s: s.replace("the_gpu_checker_agrees_with_the_cpu", "the_gpu_judge_agrees_with_the_cpu")
            n = new.get(tuple(c[:4]))
            good = bool(n) and n[4] == r(c[4]) and n[5] == r(c[5])
            ok += good; bad += not good
            print(("OK  " if good else "DIFF"), f[:22], c[0][:50])
leftover = sum(open(os.path.join(d, x), encoding="utf-8", errors="ignore").read().count("the_gpu_checker_agrees")
               for d, _s, fs in os.walk(f"{tree}/crates") for x in fs if x.endswith((".rs", ".tsv", ".wgsl")))
print(f"改名行 OK={ok} DIFF={bad}；冻结副本 crates/ 里旧名残留 {leftover} 处")
