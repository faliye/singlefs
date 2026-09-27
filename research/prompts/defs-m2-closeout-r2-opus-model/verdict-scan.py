#!/usr/bin/env python3
"""defs-m2-closeout-r2 云端攻方 F5：照执行员第 4c 步改后的字面扫 research/results/ 里全部判决行（grep -n 'name=verdict'）。
每个字段分三类打一行：
  NAMED-FALSE   取值是 false 或 not_run（字面要点名）；
  NAMED-COUNT   名字带 violation / mismatch / ambiguous / fail、取值不是 0（字面要点名）——含取值是 true、zero 这类不是计数的；
  UNNAMED       名字不带那四个词、取值是非 0 整数（字面不要求点名）。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r2-opus-model/verdict-scan.py"""
import glob, os, re, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
SUBSTRINGS = ("violation", "mismatch", "ambiguous", "fail")
counts = {"NAMED-FALSE": 0, "NAMED-COUNT": 0, "UNNAMED": 0}
rows = []
for path in sorted(glob.glob(os.path.join(ROOT, "research/results/*"))):
    if not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            if "name=verdict" not in line:
                continue
            for field, value in re.findall(r"(?<![^\s])([A-Za-z_][A-Za-z0-9_]*)=(\S+)", line):
                if field == "name":
                    continue
                if value in ("false", "not_run"):
                    kind = "NAMED-FALSE"
                elif any(word in field for word in SUBSTRINGS):
                    kind = "NAMED-COUNT" if value != "0" else None
                elif re.fullmatch(r"-?\d+", value) and value != "0":
                    kind = "UNNAMED"
                else:
                    kind = None
                if kind:
                    counts[kind] += 1
                    rows.append((kind, f"{os.path.relpath(path, ROOT)}:{number}", field, value))
for kind in ("UNNAMED", "NAMED-COUNT"):
    seen = {}
    for row in rows:
        if row[0] == kind:
            seen.setdefault((row[2], row[3]), []).append(row[1])
    for (field, value), places in sorted(seen.items()):
        print(f"{kind}\t{field}={value}\t{len(places)} 处\t例 {places[-1]}")
print("合计 " + "、".join(f"{kind} {number}" for kind, number in counts.items()))
