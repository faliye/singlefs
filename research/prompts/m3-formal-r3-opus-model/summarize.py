#!/usr/bin/env python3
"""m3-formal-r3 攻方腿：把 runs-attack/*.out 的 E7RESULT 行按 (运行, 域) 合计。
每个 (域, N) 的 name=attack 行逐键相加；J0–J4 取 name=verdict domain=… 行（不带 set=）的字段。
用法：summarize.py <runs 目录> [键 …]；不给键时打 J1–J4 与全部 atk_ 键里不全为 0 的。"""
import re
import sys
from collections import defaultdict
from pathlib import Path

runs = Path(sys.argv[1])
wanted = sys.argv[2:]
for out in sorted(runs.glob("*.out")):
    sums = defaultdict(lambda: defaultdict(int))
    order = []
    for line in out.read_text().splitlines():
        if not line.startswith("E7RESULT "):
            continue
        fields = dict(item.split("=", 1) for item in line.split()[1:] if "=" in item)
        domain = fields.get("domain")
        if domain is None:
            continue
        if domain not in order:
            order.append(domain)
        if fields.get("name") == "attack":
            for key, value in fields.items():
                if key.startswith("atk_"):
                    sums[domain][key] += int(value)
        elif fields.get("name") == "verdict" and "set" not in fields:
            for key in ("states", "j1_violations", "j2_violations", "j3_violations", "j4_violations",
                        "j6_premise_holds_violations", "applied_ge1", "failed_states",
                        "as_whole_table_violations", "as_consecutive_gap_continues_violations",
                        "as_consecutive_gap_stops_violations", "a_structure_violations"):
                if key in fields:
                    sums[domain][key] = int(fields[key])
    for domain in order:
        values = sums[domain]
        keys = wanted or [k for k in values if (not k.startswith("atk_")) or values[k] != 0]
        shown = " ".join(f"{k}={values[k]}" for k in keys if k in values)
        print(f"{out.stem}\t{domain}\t{shown}")
