#!/usr/bin/env python3
"""把探针日志里的判定行收成汇总（排好序、不带耗时）。用法：summarize.py <日志…>（每份日志名形如 <阶段>-<臂>.log）。
每份日志：原样收 OPUS 开头的判定行、V2MODEL 行与 cargo 的 test result 行（去掉 finished in 那一截），前缀 `<日志名>\t`；
再按 (kind, vis, M1 结局) 数格、数「R 的单元被写掉」的格。"""
import collections
import pathlib
import re
import sys

lines = []
for path in map(pathlib.Path, sys.argv[1:]):
    name = path.name.removesuffix(".log")
    tally = collections.Counter()
    for line in path.read_text().splitlines():
        if line.startswith("test result:") or line.startswith("V2MODEL"):
            stripped = re.sub(r";? finished in [0-9.]+s", "", line)
            lines.append(f"{name}\t{stripped}")
            continue
        if not line.startswith("OPUS "):
            continue
        lines.append(f"{name}\t{line}")
        if line.startswith("OPUS CB "):
            fields = dict(re.findall(r"(\w+)=(\S+)", line))
            m1 = "Ok" if fields.get("M1") == "Ok" else fields.get("M1", "?")
            before = int(re.search(r"before_row_root:r_copies_overwritten=(\d+)", line).group(1)) if m1 == "Ok" else 0
            sweep = int(re.search(r"max_r_copies_overwritten_while_r_in_ring=(\d+)", line).group(1)) if m1 == "Ok" else 0
            hit = "overwritten" if before > 0 or sweep > 0 else "kept"
            tally[(fields["kind"], fields["vis"], m1, hit)] += 1
    for (kind, vis, m1, hit), count in tally.items():
        lines.append(f"{name}\tTALLY kind={kind} vis={vis} M1={m1} r_units={hit} cells={count}")
for line in sorted(lines):
    print(line)
