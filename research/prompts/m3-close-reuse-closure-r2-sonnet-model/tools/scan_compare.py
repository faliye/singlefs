#!/usr/bin/env python3
"""scan_compare.py <现状输出> <名>=<输出>…：scan_entries.sh 的输出逐入口比，列闭包变了的入口（rc、单元数、拒算数、摘要任一不同）。"""
import re
import sys


def load(path):
    rows = {}
    for line in open(path, encoding="utf-8"):
        found = re.match(r"rc=(\d+) units=(\d+) refused=(\d+) digest=(\w+) (.*)", line.strip())
        rows[found.group(5)] = found.groups()[:4]
    return rows


current = load(sys.argv[1])
for argument in sys.argv[2:]:
    name, path = argument.split("=", 1)
    other = load(path)
    changed = [entry for entry in current if current[entry] != other[entry]]
    refused_new = [entry for entry in changed if int(other[entry][2]) > int(current[entry][2])]
    print(f"SCAN {name}: 入口 {len(current)} 个，闭包变了 {len(changed)} 个，其中拒算变多的 {len(refused_new)} 个")
    for entry in changed:
        print(f"  {entry} rc {current[entry][0]}→{other[entry][0]} 单元 {current[entry][1]}→{other[entry][1]} 拒算 {current[entry][2]}→{other[entry][2]}")
