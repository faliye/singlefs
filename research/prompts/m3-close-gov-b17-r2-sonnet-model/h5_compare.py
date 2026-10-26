#!/usr/bin/env python3
"""H5：把第一轮正推腿模型 out/ 里的喂命令结果，与今天主工作区上重喂的结果按「格号 + 退出码」逐格比（不比排版和行号）。
用法：python3 -I h5_compare.py <第一轮正推模型目录> <今天重喂的输出目录>
feed.txt 比整行（classify 层，不带路径与行号）；hook_feed.txt 只比 格号、exit、want。"""
import os, re, sys

old_dir, new_dir = sys.argv[1], sys.argv[2]


def cells(path):
    found = {}
    for line in open(path, encoding="utf-8").read().split("\n"):
        match = re.match(r"^(h\d+)\texit=(\d+)\twant=(\d+)\t", line)
        if match:
            found[match.group(1)] = (int(match.group(2)), int(match.group(3)))
    return found


old, new = cells(os.path.join(old_dir, "hook_feed.txt")), cells(os.path.join(new_dir, "hook_feed.txt"))
print(f"hook_feed cells old={len(old)} new={len(new)}")
changed = [(cell, old[cell], new.get(cell)) for cell in sorted(old) if old[cell][0] != new.get(cell, (None,))[0]]
for cell, was, now in changed:
    print(f"hook_feed {cell}: exit {was[0]} -> {now[0]}（want={was[1]}）")
print(f"hook_feed exit_changed_cells={len(changed)}")
old_feed = open(os.path.join(old_dir, "feed.txt"), encoding="utf-8").read().split("\n")
new_feed = open(os.path.join(new_dir, "feed.txt"), encoding="utf-8").read().split("\n")
print(f"feed lines old={len(old_feed)} new={len(new_feed)} identical={old_feed == new_feed}")
