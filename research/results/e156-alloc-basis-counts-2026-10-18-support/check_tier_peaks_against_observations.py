#!/usr/bin/env python3
# E156 第 7 次重跑的核对（只读产物）：把 name=r7_tier_peaks 行里的每档峰值，用 name=q1_observation 的原始观测点重算一遍。
# 不读 q1_trajectory，也不读装置源码；只在仓根下跑：python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_tier_peaks_against_observations.py
import collections
import re

PRODUCT = "research/results/e156-alloc-basis-counts-2026-10-18.out"

peak = collections.defaultdict(lambda: {"real": 0, "every": 0})
for line in open(PRODUCT):
    if not line.startswith("E7RESULT name=q1_observation "):
        continue
    fields = dict(re.findall(r"(\w+)=(\S+)", line))
    key = (fields["family"], fields["s"], fields["rho"], fields["k"], fields["position"])
    peak[key]["real"] = max(peak[key]["real"], int(fields["q1a_real"]))
    peak[key]["every"] = max(peak[key]["every"], int(fields["q1a_every"]))

# 每个统计量档序列里的一档对应哪一格：R7a 与 R7c 的前位置（Hh 或 Hc），R7b 的后位置 Hh，R7d 的 Hs 前与后
checked = 0
differing = 0
for line in open(PRODUCT):
    if not line.startswith("E7RESULT name=r7_tier_peaks "):
        continue
    fields = dict(re.findall(r"(\w+)=(\S+)", line))
    quantity = fields["quantity"]
    timing = "real" if fields["timing"] == "real" else "every"
    peaks = [int(x) for x in fields["peaks"].strip("[]").split(",")]
    tiers = fields["tiers"].strip("[]").split(",")
    families = fields["families"].strip("[]").split(",")
    recomputed = []
    for tier, family in zip(tiers, families):
        if quantity in ("r7a", "r7c"):
            key = (family, fields["s"], fields["rho"], tier, "front")
        elif quantity == "r7b":
            key = (family, fields["s"], fields["rho"], tier, "back")
        elif quantity == "r7d_front":
            key = (family, fields["s"], fields["rho"], tier, "front")
        elif quantity == "r7d_back":
            key = (family, fields["s"], fields["rho"], tier, "back")
        else:
            raise SystemExit(f"不认识的统计量 {quantity}")
        recomputed.append(peak[key][timing] if key in peak else None)
    checked += 1
    if recomputed != peaks:
        differing += 1
        print("不同", quantity, fields["s"], fields["rho"], fields["timing"], peaks, recomputed)
print(f"r7_tier_peaks 行 {checked} 条，用原始观测点重算后峰值序列不同的 {differing} 条；有观测点的格 {len(peak)} 个")
