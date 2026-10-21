#!/usr/bin/env python3
# E156 第 7 次重跑的核对（登记 A22，只读产物与锚点脚本输出）：每格每列（实、每）的 Q1a 峰值须不低于锚点脚本的树表下界 q1a_tt_peak。
# 锚点输入是同目录 anchors_e156_r7_hdong_window.out（脚本 anchors_e156_r7_hdong_window.py 的汇总输出）。
# 只在仓根下跑：python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_peaks_against_anchor_lower_bounds.py
import collections
import re

PRODUCT = "research/results/e156-alloc-basis-counts-2026-10-18.out"
ANCHOR_SUMMARY = "research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.out"

lower_bound = {}
for line in open(ANCHOR_SUMMARY):
    fields = dict(re.findall(r"(\S+?)=(\S+)", line))
    if "family" not in fields:
        continue
    match = re.match(r"(Hh|Hc|Hs)-(front|back)-[kmn](\d+)$", fields["family"])
    family, position, hole_count = match.group(1), match.group(2), match.group(3)
    slots_per_region = re.match(r"S=(\d+)", line).group(1)
    lower_bound[(family, slots_per_region, hole_count, position)] = int(fields["q1a_tt_peak"])

peak = collections.defaultdict(int)
for line in open(PRODUCT):
    if line.startswith("E7RESULT name=q1_observation "):
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        if fields["family"] in ("Hh", "Hc", "Hs"):
            for column in ("q1a_real", "q1a_every"):
                key = (fields["family"], fields["s"], fields["rho"], fields["k"], fields["position"], column)
                peak[key] = max(peak[key], int(fields[column]))

checked = 0
at_or_above = 0
violations = []
for (family, slots_per_region, rho, hole_count, position, column), value in sorted(peak.items()):
    if position == "-":
        continue
    checked += 1
    bound = lower_bound.get((family, slots_per_region, hole_count, position))
    if bound is None:
        violations.append(("锚点里没有这一格", family, slots_per_region, rho, hole_count, position))
    elif value >= bound:
        at_or_above += 1
    else:
        violations.append((family, slots_per_region, rho, hole_count, position, column, value, bound))
print(f"格 × 列 {checked} 个，峰值不低于锚点树表下界的 {at_or_above} 个，低于的 {len(violations)} 个 {violations}")
