#!/usr/bin/env python3
"""旁注：覆盖记录里闭包带进来的 wgsl 单元（种类 wgsl_compute 等）此刻能不能在 current_fingerprints 里找回。
用 k1_direct.py --record 存下的覆盖记录（--store），逐条数：记录里 wgsl_* 成员几个、其中此刻找不回指纹的几个、整条 executed_set_is_current 是真是假。
用法：k1_wgsl_probe.py --script-repo <判法所在仓> --root <crates 树根> --store <存档>"""
import argparse
import collections
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import k1_common

parser = argparse.ArgumentParser()
parser.add_argument("--script-repo", required=True)
parser.add_argument("--root", required=True)
parser.add_argument("--store", required=True)
arguments = parser.parse_args()
module = k1_common.load_rows_module(arguments.script_repo)
keys = module.mutation_units.UnitKeys(arguments.root)
store = json.load(open(arguments.store, encoding="utf-8"))
with_wgsl = without_wgsl = 0
current_with = current_without = 0
missing_by_kind = collections.Counter()
for cid, entry in sorted(store.items()):
    references = [module.mutation_units.UnitReference(*item) for item in entry["coverage"]["executed"]]
    found = keys.current_fingerprints(references)
    missing = [r for r in references if r.fingerprint not in found.get((r.kind, r.file, r.name), set())]
    has_wgsl = any(r.kind.startswith("wgsl") for r in references)
    is_current = module.executed_set_is_current(keys, entry["coverage"])
    if has_wgsl:
        with_wgsl += 1
        current_with += is_current
        missing_by_kind.update(r.kind for r in missing)
    else:
        without_wgsl += 1
        current_without += is_current
print(f"name=k1_wgsl_probe coverage_records={len(store)} with_wgsl_members={with_wgsl} (executed_set_is_current 真 {current_with}) "
      f"without_wgsl_members={without_wgsl} (executed_set_is_current 真 {current_without}) missing_by_kind_in_wgsl_records={dict(sorted(missing_by_kind.items()))}")
