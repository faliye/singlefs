#!/usr/bin/env python3
"""K1 路径 2（整张表）：只算 python，对变异表每一行问这一份仓的 UnitKeys.resolve_test 和 mutated_units，一行一行打 JSON 到 --out；不算闭包、不碰判定库。
用法：k1_resolve_all.py --script-repo <判法所在仓> --root <crates 树根> --out <结果>"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import k1_common

parser = argparse.ArgumentParser()
parser.add_argument("--script-repo", required=True)
parser.add_argument("--root", required=True)
parser.add_argument("--out", required=True)
arguments = parser.parse_args()
module = k1_common.load_rows_module(arguments.script_repo)
rows, _memory_max, _timeout = k1_common.table_rows(module, arguments.root)
keys = module.mutation_units.UnitKeys(arguments.root)
with open(arguments.out, "w", encoding="utf-8") as out:
    for row in rows:
        test, refusal = keys.resolve_test(row.arguments, row.expected)
        mutated, mutated_refusal = keys.mutated_units(row.path, row.old)
        out.write(json.dumps(dict(
            line=row.line_number,
            resolve=(f"拒算：{refusal}" if refusal else f"{test.target}::{test.full_name}::{test.unit.fingerprint[:12]}"),
            mutated=(f"拒算：{mutated_refusal}" if mutated_refusal else [f"{item.kind}:{item.file}:{item.name}:{item.fingerprint[:12]}" for item in mutated])),
            ensure_ascii=False, sort_keys=True) + "\n")
print(f"name=k1_resolve_all script_repo={os.path.basename(arguments.script_repo)} root={os.path.basename(arguments.root)} rows={len(rows)}")
