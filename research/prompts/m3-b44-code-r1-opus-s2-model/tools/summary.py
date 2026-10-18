#!/usr/bin/env python3
"""从 aggregate.py 的输出数每段历史、每组读故障、每个交盘次序的状态：做成 / 拒、判错（R 走得完而放过并写掉）、R 已没而拒、凭据为真。
用法：summary.py <agg.txt>"""
import re
import sys
from collections import defaultdict

FIELD = re.compile(r"(\w+)=(\S+)")
totals = defaultdict(lambda: defaultdict(int))
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("name=k1_cell "):
        continue
    fields = dict(FIELD.findall(line))
    key = (fields["history"], fields["faults"], fields["order"])
    states = int(fields["states"])
    row = totals[key]
    row["states"] += states
    row["mounted" if fields["outcome"].startswith("mounted") else "refused"] += states
    if fields["wrong"] == "true":
        row["wrong"] += states
    if fields["refused_with_r_gone"] == "true":
        row["refused_with_r_gone"] += states
    if fields["clause_credential"] == "true":
        row["credential"] += states
    if fields["outcome"].startswith("mounted") and ":new=0:" not in fields["outcome"]:
        row["mounted_newly_overwrote"] += states
for key in sorted(totals):
    row = totals[key]
    print("name=k1_summary history={} faults={} order={} ".format(*key) + " ".join(
        f"{name}={row[name]}" for name in ["states", "mounted", "refused", "credential", "wrong", "refused_with_r_gone", "mounted_newly_overwrote"]))
