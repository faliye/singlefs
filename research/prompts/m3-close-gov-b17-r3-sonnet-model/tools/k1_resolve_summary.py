#!/usr/bin/env python3
"""K1 路径 2（整张表）：k1_resolve_all.py 各变体的 JSONL 与 base 逐行比。用法：k1_resolve_summary.py <目录> base <变体>…"""
import json
import os
import sys

directory, labels = sys.argv[1], sys.argv[2:]
def load(label):
    return {json.loads(line)["line"]: json.loads(line) for line in open(os.path.join(directory, label + ".jsonl"), encoding="utf-8")}
base = load("base")
print(f"base: rows={len(base)} resolve_refused={sum(1 for r in base.values() if r['resolve'].startswith('拒算'))} mutated_refused={sum(1 for r in base.values() if isinstance(r['mutated'], str))}")
for label in labels[1:]:
    current = load(label)
    resolve = [line for line in base if base[line]["resolve"] != current[line]["resolve"]]
    mutated = [line for line in base if base[line]["mutated"] != current[line]["mutated"]]
    newly_refused = [line for line in resolve if current[line]["resolve"].startswith("拒算") and not base[line]["resolve"].startswith("拒算")]
    print(f"{label}: resolve_changed={len(resolve)} (其中新拒算 {len(newly_refused)}) mutated_changed={len(mutated)}")
