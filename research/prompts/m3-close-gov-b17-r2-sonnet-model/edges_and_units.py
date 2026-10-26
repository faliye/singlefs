#!/usr/bin/env python3
"""H1：EXCLUDED_PROGRAM_EDGES 今天登记了几条边；crates 变异判法单元清单里出自 NOT_JUDGING_FILES 两份的有几个。
用法：python3 -I edges_and_units.py <仓根> <judging-units --units 的输出文件>"""
import re, sys

repo, units_path = sys.argv[1], sys.argv[2]
text = open(f"{repo}/scripts/gate/lib_function_units.py", encoding="utf-8").read()
start = text.index("EXCLUDED_PROGRAM_EDGES = {")
block = text[start:text.index("\n}\n", start)]
edges = re.findall(r'^    \("([^"]+)", "([^"]+)", "([^"]+)"\):', block, re.M)
print(f"excluded_program_edges={len(edges)}")
for source, unit_name, target in edges:
    print(f"  {source} :: {unit_name} -> {target}")
rows = [line.split("\t") for line in open(units_path, encoding="utf-8").read().split("\n")[1:] if line]
excluded = ("scripts/crash/lib_verdict_library.py:", "scripts/gate/lib_function_units.py:")
leaked = [row for row in rows if any(row[2].startswith(prefix) for prefix in excluded)]
print(f"crates_judging_units total={len(rows)} from_not_judging_files={len(leaked)}")
