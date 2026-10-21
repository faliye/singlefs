#!/usr/bin/env python3
"""m3-close-c693-r3 辩方腿：把 accounting_planner_per_statistic_arm_node_cost 的输出（R3PLANNER 行）按判据归并，并与第二轮攻方
同类装置的输出（R2PLANNER 行）在重叠的格上逐臂比。纯整数比较，只读两份输出文件。
用法：planner_claims.py <本腿 planner.out> <第二轮攻方 planner-carry-cost.out>
自证（会红）：重叠的格（r_tree_dim = 1 且台阶行条数落在攻方扫过的 {1,3,500,2000}）三条老臂的 first / steady_max 必须逐格相等；
  弄坏开关 PLANNER_CLAIMS_BREAK=shift_carry 把本腿 Carry 的 steady_max 加 1，自证必须红（退 3）。
"""
import os, re, sys
from collections import Counter

MINE, THEIRS = sys.argv[1], sys.argv[2]
BREAK = os.environ.get("PLANNER_CLAIMS_BREAK", "")
LEAF = 477


def parse(path, tag):
    out = []
    for line in open(path):
        if not line.startswith(tag) or "capacity" in line:
            continue
        d = dict(tok.split("=", 1) for tok in line.split()[1:] if "=" in tok and ":" not in tok)
        arms = {}
        for m in re.finditer(r"(\w+):first=(\d+),steady_max=(\d+)(?:,rows=(\d+))?", line):
            arms[m.group(1)] = (int(m.group(2)), int(m.group(3)) + (1 if BREAK == "shift_carry" and tag == "R3PLANNER" and m.group(1) == "Carry" else 0),
                                int(m.group(4)) if m.group(4) else None)
        d["arms"] = arms
        out.append(d)
    return out


mine, theirs = parse(MINE, "R3PLANNER"), parse(THEIRS, "R2PLANNER")
ok = True

# 自证：重叠格逐臂相等
index = {(d["devices"], d["known_steps"], d["unknown_heads"], d["unknown_steps"]): d for d in mine if d["r_tree_dim"] == "1"}
checked = bad = 0
for t in theirs:
    key = (t["devices"], t["known_steps"], t["unknown_heads"], t["unknown_steps"])
    m = index.get(key)
    if m is None:
        continue
    for arm in ("Carry", "NoCarry", "KeepWithRoot"):
        checked += 1
        if m["arms"][arm][:2] != t["arms"][arm][:2]:
            bad += 1
print(f"E7RESULT name=selftest_overlap_with_r2_opus cells_x_arms={checked} mismatches={bad}")
ok &= bad == 0 and checked > 0

# 判据 A：记账树装得进一片叶（rows_in_r <= 477）时，四条臂每次发布重写的节点数全相等
a_total = a_equal = 0
for d in mine:
    if int(d["rows_in_r"]) <= LEAF:
        a_total += 1
        if len({v[1] for v in d["arms"].values()}) == 1:
            a_equal += 1
print(f"E7RESULT name=claim_a_single_leaf lines={a_total} all_four_arms_equal={a_equal}")

# 判据 B：装不进一片叶时 Carry 比 NoCarry 多重写 0..h_new 个节点（r_tree_dim = 1）；r_tree_dim = 2 时 Carry 对按统计量、NoCarry 的差
dist = {}
for d in mine:
    if int(d["rows_in_r"]) <= LEAF:
        continue
    h = int(d["unknown_heads"])
    key = (d["r_tree_dim"], d["second"])
    dc = d["arms"]["Carry"][1] - d["arms"]["NoCarry"][1]
    ds = d["arms"]["Carry"][1] - d["arms"]["PerStatistic"][1]
    sn = d["arms"]["PerStatistic"][1] - d["arms"]["NoCarry"][1]
    dist.setdefault(key, Counter())[(dc, ds, sn, h)] += 1
for key in sorted(dist):
    print(f"E7RESULT name=claim_b_multi_leaf r_tree_dim={key[0]} second={key[1]} lines={sum(dist[key].values())} "
          f"carry_minus_nocarry_minus_per_statistic_minus_nocarry_per_statistic_minus_nocarry_by_h_new="
          + ";".join(f"({a},{b},{c})@h{h}:{n}" for (a, b, c, h), n in sorted(dist[key].items())))

# 门槛：每个（设备数, 已知台阶, 头数）下，Carry 比 NoCarry 多重写节点的最小台阶行条数（r_tree_dim = 1）
firsts = {}
for d in mine:
    if d["r_tree_dim"] != "1":
        continue
    k = (d["devices"], d["known_steps"], d["unknown_heads"])
    extra = d["arms"]["Carry"][1] - d["arms"]["NoCarry"][1]
    if extra > 0:
        s = int(d["unknown_steps"])
        firsts[k] = min(firsts.get(k, 10**9), s)
for k in sorted(firsts):
    print(f"E7RESULT name=threshold_unknown_steps_scanned devices={k[0]} known_steps={k[1]} unknown_heads={k[2]} smallest_scanned_step_count_where_carry_writes_more_nodes={firsts[k]}")
never = sorted({(d["devices"], d["known_steps"], d["unknown_heads"]) for d in mine if d["r_tree_dim"] == "1"} - set(firsts))
print(f"E7RESULT name=never_more_nodes_in_scan groups={never}")
# 反例搜：rows_in_r <= 477 却有臂不等
print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
