#!/usr/bin/env python3
"""K1 路径 2：只算 python，不起 cargo。直接调某一份仓里的 UnitKeys.resolve_test / mutated_units / executed_units 和 executed_set_is_current / baseline_reuse_problem。
记录阶段（--record，只在 base 上跑）：每个覆盖号造一份「合成的 llvm-cov export」（这条测试闭包里每个 fn 单元一个函数，第一段代码区域的行取 fn 关键字所在行）
  并照 judge_coverage 的写法造一份覆盖记录，存进 --store。
检查阶段（每个变体都跑）：同一份 export 喂这个变体的 executed_units，和同一份覆盖记录问 executed_set_is_current、baseline_reuse_problem。
每行打一行 JSON 到 --out。用法：
  k1_direct.py --script-repo <判法所在仓> --root <crates 树根> --store <存档> --out <结果> [--record] [--rows N]
需要环境变量 GATE_MUTATION_RECORDS_HOME（判定库目录，这里只读不写）。"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import k1_common


def region_line(root, reference):
    first, last = (int(part) for part in reference.lines.split("-"))
    with open(os.path.join(root, reference.file), encoding="utf-8") as handle:
        source = handle.read().split("\n")
    short = reference.name.split("::")[-1]
    for number in range(first, last + 1):
        if f"fn {short}" in source[number - 1]:
            return number
    return first


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--script-repo", required=True)
    parser.add_argument("--root", required=True)
    parser.add_argument("--store", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--record", action="store_true")
    parser.add_argument("--rows", type=int, default=50)
    parser.add_argument("--stride", type=int, default=100, help="每隔几行取一行（变异表按行号间隔取样，覆盖不同的 crate 与测试目标）")
    arguments = parser.parse_args()
    module = k1_common.load_rows_module(arguments.script_repo)
    units = module.mutation_units
    rows, memory_max, timeout_seconds = k1_common.table_rows(module, arguments.root)
    rows = rows[::arguments.stride][:arguments.rows]
    key_context, components, _library = k1_common.key_context_for(module, arguments.root, rows, memory_max, timeout_seconds)
    keys = key_context.keys
    store = {} if arguments.record else json.load(open(arguments.store, encoding="utf-8"))
    environment = dict(os.environ)
    with open(arguments.out, "w", encoding="utf-8") as out:
        for row in rows:
            comp = components[row.line_number]
            test, refusal = keys.resolve_test(row.arguments, row.expected)
            mutated, mutated_refusal = keys.mutated_units(row.path, row.old)
            record = dict(line=row.line_number, row_key=comp.row_key, baseline_key=comp.baseline_key, coverage_id=comp.coverage_id, test_closure=comp.test_closure,
                          resolve=(f"拒算：{refusal}" if refusal else f"{test.target}::{test.full_name}::{test.unit.fingerprint[:12]}"),
                          mutated=(f"拒算：{mutated_refusal}" if mutated_refusal else [f"{item.kind}:{item.fingerprint[:12]}" for item in mutated]))
            cid = comp.coverage_id
            if arguments.record and cid and cid not in store and comp.test_closure in key_context.closures:
                closure = key_context.closures[comp.test_closure]
                functions = []
                for reference in list(closure.members) + [units.reference_of(test.unit)]:
                    if reference.kind == "fn" and reference.file.endswith(".rs"):
                        line = region_line(arguments.root, reference)
                        functions.append(dict(count=1, regions=[[line, 1, line, 2, 1, 0, 0, 0]], filenames=[os.path.join(arguments.root, reference.file)]))
                export = dict(data=[dict(functions=functions)])
                executed, unplaced = keys.executed_units(export, arguments.root)
                members = {(r.kind, r.file, r.name, r.fingerprint): r for r in executed + [m for m in closure.members if m.kind != "fn"]}
                coverage = dict(test=list(units.reference_of(test.unit)), executed=sorted(list(r) for r in members.values()),
                                executed_functions=len(executed), unplaced_functions=unplaced, complete=unplaced == 0 and bool(executed))
                store[cid] = dict(export=export, coverage=coverage, test_closure=comp.test_closure)
            if cid in store:
                again, unplaced = keys.executed_units(store[cid]["export"], arguments.root)
                stored_functions = sorted(tuple(item[:4]) + (item[4],) for item in store[cid]["coverage"]["executed"] if item[0] == "fn")
                now_functions = sorted((r.kind, r.file, r.name, r.lines, r.fingerprint) for r in again)
                record["executed_functions_same_as_recorded"] = [tuple(x) for x in stored_functions] == now_functions
                record["executed_functions_now"] = len(again)
                record["unplaced_now"] = unplaced
                record["set_current"] = module.executed_set_is_current(keys, store[cid]["coverage"])
                # 只走覆盖那一支（基线记录里不记闭包号）与真实的写法（基线记录里记着记下它那一刻的点名测试闭包号，覆盖不对时再比闭包号）各问一次
                record["reuse_problem"] = module.baseline_reuse_problem({"verdict": "ok", "test_closure": ""}, comp, keys, store[cid]["coverage"], "snapshot", environment)
                record["reuse_problem_realistic"] = module.baseline_reuse_problem({"verdict": "ok", "test_closure": store[cid]["test_closure"]}, comp, keys,
                                                                                  store[cid]["coverage"], "snapshot", environment)
            out.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
    if arguments.record:
        with open(arguments.store, "w", encoding="utf-8") as handle:
            json.dump(store, handle, ensure_ascii=False, sort_keys=True)
    print(f"name=k1_direct script_repo={os.path.basename(arguments.script_repo)} root={os.path.basename(arguments.root)} rows={len(rows)} stored={len(store)}")


main()
