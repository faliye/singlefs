#!/usr/bin/env python3
"""K1 路径 2：把 k1_direct.py 各变体的 JSONL 与基线（base-base.jsonl）逐行比，按字段数出几行不同。用法：k1_compare.py <目录> <标签>…"""
import json
import os
import sys

directory, labels = sys.argv[1], sys.argv[2:]
def load(label):
    return {json.loads(line)["line"]: json.loads(line) for line in open(os.path.join(directory, label + ".jsonl"), encoding="utf-8")}
base = load("base-base")
fields = ["resolve", "mutated", "row_key", "baseline_key", "coverage_id", "test_closure", "executed_functions_now", "executed_functions_same_as_recorded", "set_current", "reuse_problem", "reuse_problem_realistic"]
print("label\t" + "\t".join(fields))
for label in labels:
    current = load(label)
    counts = [sum(1 for line in base if base[line].get(field) != current[line].get(field)) for field in fields]
    print(f"{label}\t" + "\t".join(str(count) for count in counts))
print(f"name=k1_compare rows={len(base)} set_current_true_in_base={sum(1 for r in base.values() if r.get('set_current'))}")
for label in labels:
    current = load(label)
    flipped = [line for line in base if base[line].get("set_current") != current[line].get("set_current")]
    reuse = sum(1 for line in current if current[line].get("reuse_problem") is None)
    reuse_realistic = sum(1 for line in current if current[line].get("reuse_problem_realistic") is None)
    # 真实流程（baseline_check）：先按基线键在库里取记录，取得到才问 baseline_reuse_problem；库里的记录是 base 那一趟写的，所以「取得到」= 基线键与 base 的相同
    real_flow = sum(1 for line in current if current[line].get("baseline_key") and current[line]["baseline_key"] == base[line]["baseline_key"]
                    and current[line].get("reuse_problem_realistic") is None)
    print(f"label={label} set_current_true={sum(1 for r in current.values() if r.get('set_current'))} baseline_reused_by_coverage_only={reuse} "
          f"baseline_reused_given_record={reuse_realistic} baseline_reused_real_flow={real_flow} set_current_flipped_rows={len(flipped)}")
