#!/usr/bin/env python3
"""c393grid 行按键对齐：每份副本每格的 可写挂载 / 隔离 / 计数 / 复用 / I-7.4。用法：summarize_c393.py <runs> <后缀> 副本...
输出一行一格：键 | 副本1 | 副本2 ...；最后按副本计「复用格数」「拒可写格数」「可写且不复用格数」。"""
import re, sys, collections, pathlib
runs = pathlib.Path(sys.argv[1]); suffix = sys.argv[2]; copies = sys.argv[3:]
def cell(line):
    if " writable=refused" in line:
        reason = re.search(r"error=NewerStateStillUnreadableAfterOneReread\((\w+)", line)
        return "refused(" + (reason.group(1) if reason else re.search(r"error=(\w+)", line).group(1)) + ")"
    iso = re.search(r"isolated=\[\(DeviceIdentity\(0\), (\d+)\), \(DeviceIdentity\(1\), (\d+)\)\]", line)
    unread = re.search(r"abandoned_roots_unreadable=(\d+)", line).group(1)
    if "reuse_txg=" in line:
        reuse = "REUSE@" + re.search(r"reuse_txg=(\d+)", line).group(1)
        which = re.search(r"overwritten=\[(.*?)\] I-7.4", line).group(1)
        which = ",".join(sorted(set(re.findall(r"\((\d+), \[", which))))
        reuse += f"(abandoned#{which})"
    else:
        reuse = "no_reuse"
    i74 = "I74red" if "I-7.4=Violated" in line else ("I74holds" if "I-7.4=Holds" in line else "?")
    return f"ok iso={iso.group(1)}/{iso.group(2)} unread={unread} {reuse} {i74}"
tables = {}
for copy in copies:
    rows = {}
    for line in (runs / f"{copy}-c393_candidates_grid{suffix}.log").read_text().splitlines():
        if line.startswith("name=c393grid start="):
            key = re.sub(r" fault_ranges=\d+.*", "", line[len("name=c393grid "):])
            rows[key] = cell(line)
    tables[copy] = rows
keys = sorted(tables[copies[0]])
for key in keys:
    short = key.replace("abandoned_roots_unreadable_account=", "acct_of=").replace("records_also_unreadable=", "rec=").replace("unreadability=", "").replace("read_back=", "").replace("account=", "").replace("start=c393grid-", "")
    print(short + " | " + " | ".join(tables[c].get(key, "MISSING") for c in copies))
for c in copies:
    counts = collections.Counter("REUSE" if "REUSE" in v else ("refused" if v.startswith("refused") else "ok_no_reuse") for v in tables[c].values())
    print(f"TOTAL {c}: cells={len(tables[c])} {dict(counts)}")
