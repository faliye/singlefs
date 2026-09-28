#!/usr/bin/env python3
"""X5 ① 扫描日志的分格（m2-closeout-code-r3 云端攻方腿）：按入口、注入种类、结局成员、冻不冻结分格计数，
再逐段判红：盘面 checker、崩了再可写挂载、之后 checker、读回内容、读故障报错之前已写、写 / 屏障故障报错之后还写、崩溃态枚举。
用法：analyze_x5.py <日志…>"""
import re
import sys
from collections import Counter

cells = Counter()
red = Counter()
red_lines = []
enum_totals = Counter()
runs = 0
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("SWEEP seed=") or " fault=" not in line:
            continue
        runs += 1
        entry = re.search(r"entry=(\S+)", line).group(1)
        fault = re.search(r"fault=(None|Some\(\((\w+), (\d+)\)\))", line)
        kind = "none" if fault.group(1) == "None" else fault.group(2).lower()
        outcome = re.search(r"outcome=(Ok|Err \w+(?:\(\w+)?)", line).group(1)
        frozen = re.search(r"frozen=(\w+)", line).group(1)
        cells[(entry, kind, outcome, frozen)] += 1
        checker = int(re.search(r" checker=(\d+)", line).group(1))
        remount = re.search(r"remount=(\S+)", line).group(1)
        after = int(re.search(r"checker_after_remount=(\d+)", line).group(1))
        content_ok = re.search(r"content_ok=(\w+)", line).group(1) == "true"
        writes_before = int(re.search(r"writes_before=(\d+)", line).group(1))
        writes_after = int(re.search(r"writes_after=(\d+)", line).group(1))
        resend = re.search(r"resend=(\S+)", line).group(1)
        reasons = []
        if checker:
            reasons.append("checker")
        if remount != "Ok":
            reasons.append("remount")
        if after:
            reasons.append("checker_after_remount")
        if not content_ok:
            reasons.append("content")
        if kind == "read" and outcome.startswith("Err") and writes_before > 0:
            reasons.append("refused_after_writes")
        if kind in ("write", "barrier") and outcome.startswith("Err") and writes_after > 0 and frozen == "false":
            reasons.append("wrote_after_the_failure_without_freezing")
        if frozen == "true" and resend != "Ok":
            reasons.append("resend")
        enum = re.search(r"enum=(\S+)", line).group(1)
        if enum.startswith("states="):
            states = int(re.search(r"enum=states=(\d+)", line).group(1))
            enum_totals[(entry, kind)] += states
            for field in ("oracle", "root_without_record", "missing_unit", "checker_red_states"):
                value = int(re.search(field + r"=(\d+)", line).group(1))
                if value:
                    reasons.append("enum_" + field)
        for reason in reasons:
            red[(entry, kind, reason)] += 1
        if reasons:
            red_lines.append(line.strip()[:400])
print(f"RUNS {runs}")
for key, count in sorted(cells.items()):
    print("CELL", key, count)
for key, count in sorted(red.items()):
    print("RED", key, count)
for key, count in sorted(enum_totals.items()):
    print("ENUM_STATES", key, count)
print("ENUM_STATES_TOTAL", sum(enum_totals.values()))
for line in red_lines[:40]:
    print("REDLINE", line)
