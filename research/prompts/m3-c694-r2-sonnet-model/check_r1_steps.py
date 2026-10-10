#!/usr/bin/env python3
"""check_r1_steps.py [逐步表]：复核第一轮攻方腿报告里「36 个不拒的起点里 33 个有 R 读不出的步、第一次都在第 1 步」。
默认读 research/prompts/m3-c694-r1-opus-model/out/h1-grid-steps.txt（相对仓根；SINGLEFS_REPO 设了就取它）。不写任何文件。"""
import collections
import os
import re
import sys

root = os.environ.get("SINGLEFS_REPO", os.getcwd())
path = sys.argv[1] if len(sys.argv) > 1 else os.path.join(root, "research/prompts/m3-c694-r1-opus-model/out/h1-grid-steps.txt")
current = None
starts = collections.OrderedDict()
refused = set()
first_failed_step = {}
for line in open(path, encoding="utf-8"):
    header = re.match(r"== (W=\S+ k=\d+) arm=(\S+)", line)
    if header:
        current = (header.group(1), header.group(2))
        starts[current] = None
        continue
    step = re.match(r"\s+step (\d+): (.*)", line)
    if step and current and current[1] == "today":
        index, rest = int(step.group(1)), step.group(2)
        if "um=Err" in rest:
            refused.add(current)
        if "FAILED(MappingStillUnreadable" in rest and current not in first_failed_step:
            first_failed_step[current] = index
today = [key for key in starts if key[1] == "today"]
not_refused = [key for key in today if key not in refused]
failed = [key for key in not_refused if key in first_failed_step]
print(f"today_starts={len(today)} refused_starts={len(refused)} not_refused={len(not_refused)} "
      f"not_refused_with_a_step_where_R_is_unreadable={len(failed)}")
print("first_step_where_R_is_unreadable_histogram=" + str(dict(collections.Counter(first_failed_step[key] for key in failed))))
