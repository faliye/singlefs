#!/usr/bin/env python3
"""K1 路径 3：给判定库写一份「上一趟这张表每一行都判成抓到」的记录（字段照 judge_row 写的那一份），之后用真的 plan 子命令问每一行此刻复用不复用。
键是这一份仓（--script-repo）的判法此刻照 --root 这棵 crates 树算出来的。库目录由环境变量 GATE_MUTATION_RECORDS_HOME 给。
用法：k1_seed.py --script-repo <判法所在仓> --root <crates 树根>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import k1_common

parser = argparse.ArgumentParser()
parser.add_argument("--script-repo", required=True)
parser.add_argument("--root", required=True)
arguments = parser.parse_args()
module = k1_common.load_rows_module(arguments.script_repo)
rows, memory_max, timeout_seconds = k1_common.table_rows(module, arguments.root)
key_context, components, library = k1_common.key_context_for(module, arguments.root, rows, memory_max, timeout_seconds)
seeded = 0
for row in rows:
    comp = components[row.line_number]
    if not comp.row_key:
        continue
    library.write("records", comp.row_key, dict(
        kind="mutation", row_key=comp.row_key, judging=comp.judging, verdict=module.REUSABLE_VERDICT["mutation"], text="K1 探针造的记录：上一趟判成抓到",
        exit_code=101, judged_date="2026-10-24", how="k1_seed.py", test_full_name=comp.test_full_name or "", test_closure=comp.test_closure or "",
        mutated=",".join(module.mutation_units.unit_id(reference) for reference in comp.mutated or []), toolchain=comp.toolchain or "", dependency=comp.dependency or ""))
    seeded += 1
print(f"name=k1_seed script_repo={os.path.basename(arguments.script_repo)} root={os.path.basename(arguments.root)} rows={len(rows)} seeded={seeded} "
      f"judging={comp.judging[:16]} break={os.environ.get('GATE_MUTATION_BREAK', '')}")
