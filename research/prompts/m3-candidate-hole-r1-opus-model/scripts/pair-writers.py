#!/usr/bin/env python3
"""用法：pair-writers.py <草稿 out 目录> <前缀>。同一组参数的正确写者（-ok）与坏写者（-m4a）逐步配对：取 skip=rows i31="e1w" 那一组的
NOTE（洞表、盘 0 的已分配 / 遍历 / 上沿），列出坏写者盘面上洞表里有「存在过却走不到」、而 e1w 判成立的那几步，
并给出同一步正确写者的已分配（坏写者少记了多少 = 正确写者已分配 − 坏写者已分配；两边遍历也一起列）。"""
import glob
import re
import sys

out, prefix = sys.argv[1], sys.argv[2]


def steps(path):
    found = {}
    for line in open(path, encoding="utf-8"):
        m = re.match(r'  NOTE (.*) skip=rows i31="e1w" HOLE_TABLE .*non_present=(\[.*\])', line)
        if m:
            found.setdefault(m.group(1), {})["unreachable"] = "ExistedUnreachable" in m.group(2)
        m = re.match(r'  NOTE (.*) skip=rows i31="e1w" I31 device=0 allocated=Some\((\d+)\) walked=(\d+) .*proto=(.*)$', line)
        if m:
            entry = found.setdefault(m.group(1), {})
            entry["allocated"], entry["walked"] = int(m.group(2)), int(m.group(3))
            up = re.search(r"upper: (\d+)", m.group(4))
            entry["upper"] = int(up.group(1)) if up else entry["walked"]
    return found


total = missed = 0
for bad in sorted(glob.glob(f"{out}/{prefix}*-m4a.out")):
    ok = bad.replace("-m4a.out", "-ok.out")
    b, o = steps(bad), steps(ok)
    for tag, e in b.items():
        if not e.get("unreachable") or "allocated" not in e:
            continue
        total += 1
        o_e = o.get(tag, {})
        green = e["walked"] <= e["allocated"] <= e["upper"]
        short = o_e.get("allocated", 0) - e["allocated"]
        if green and short > 0:
            missed += 1
            print(f"MISS {bad.rsplit('/', 1)[1]} {tag}: m4a allocated={e['allocated']} walked={e['walked']} upper={e['upper']} | ok allocated={o_e.get('allocated')} walked={o_e.get('walked')} | short={short}")
print(f"m4a_unreachable_states={total} e1w_green_and_accounting_below_the_correct_writer={missed}")
