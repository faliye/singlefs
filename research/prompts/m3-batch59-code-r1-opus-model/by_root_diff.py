#!/usr/bin/env python3
"""比两份替身输出（基线与一个改坏）的 OPUS_M1 by-root 行：每条流列出旋钮那一条违例态数不同的那几个根 txg、差多少个状态。
用法：by_root_diff.py <基线.txt> <改坏.txt>。"""
import re
import sys


def load(path):
    flows = {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("OPUS_M1 by-root"):
            flow = re.search(r"flow=(\S+)", line).group(1)
            flows[flow] = {int(txg): tuple(map(int, counts.split("/"))) for txg, counts in (cell.split(":") for cell in line.split()[4:])}
    return flows


base, mutant = load(sys.argv[1]), load(sys.argv[2])
for flow in sorted(base):
    differing = [txg for txg in base[flow] if base[flow][txg] != mutant[flow][txg]]
    states = sum(base[flow][txg][1] - mutant[flow][txg][1] for txg in differing)
    span = f"{differing[0]}..={differing[-1]}" if differing else "-"
    print(f"{flow} differing_roots={len(differing)} root_txgs={span} knob_violated_states_lost={states}")
