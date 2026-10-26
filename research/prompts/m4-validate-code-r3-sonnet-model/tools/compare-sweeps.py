#!/usr/bin/env python3
"""把「已打甲」与「撤掉甲」两份扫描输出逐串比：字节哈希同不同、四处读者收不收、池级 checker 红的集合。用法：compare-sweeps.py <打甲输出> <撤甲输出>"""
import re, sys

def load(path):
    rows = {}
    for line in open(path, encoding="utf-8"):
        if not line.startswith("SWEEP "):
            continue
        name = line.strip().split("name=", 1)[1]
        key = name[:2]
        rows[key] = line.strip()
    return rows

def parse(line):
    if "会话里panic" in line:
        return {"panic": True}
    h = re.search(r"hash=(\w+)", line).group(1)
    readers = re.search(r"冷走读=(.*?) \| 冷恢复=(.*?) \| 可写重挂=(.*?) \| 挂载态=(.*?) \| checker红=(.*)$", line)
    refusal = re.search(r"会话拒=(\[.*?\]) hash", line)
    return {"hash": h, "readers": readers.groups(), "session_refusal": refusal.group(1) if refusal else ""}

def accepted(readers):
    cold_walk, cold_recovery, remount, mounted, _ = readers
    return [("拒" not in x.split("{")[0]) or "NoSuchInode" in x for x in (cold_walk, cold_recovery, remount, mounted)]

new = load(sys.argv[1]); old = load(sys.argv[2])
rows_new_refused = rows_old_refused = rows_bytes_differ = 0
for key in sorted(new):
    n = parse(new[key]); o = parse(old[key]) if key in old else None
    if n.get("panic") or (o and o.get("panic")):
        print(key, "PANIC", n.get("panic"), o and o.get("panic")); continue
    n_ok = all(accepted(n["readers"])) and not n["session_refusal"].strip("[]")
    o_ok = all(accepted(o["readers"])) and not o["session_refusal"].strip("[]")
    differ = n["hash"] != o["hash"]
    rows_bytes_differ += differ
    rows_new_refused += (not n_ok)
    rows_old_refused += (not o_ok)
    print(f"{key} bytes={'differ' if differ else 'same'} 打甲读者全收={n_ok} 撤甲读者全收={o_ok} 打甲checker红={n['readers'][4]} 撤甲checker红={o['readers'][4]}")
print(f"合计 串数={len(new)} 字节不同={rows_bytes_differ} 打甲有读者拒={rows_new_refused} 撤甲有读者拒={rows_old_refused}")
