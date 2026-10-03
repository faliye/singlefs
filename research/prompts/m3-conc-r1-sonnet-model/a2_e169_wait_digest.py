#!/usr/bin/env python3
"""E169 第三段（fsync 等待模型）：按 (Q1 候选, Q2 臂) 汇总 ρ = W_max(64) ÷ W_max(8) 的范围与「长 / 中间 / 有界」条数；
再按 Q1 候选比较 W_max(64) 在等速剖面、r=1、两个代价取样点（P1 设备主导、P2 CPU 主导）上的绝对值，看哪个 Q1 候选的等待最低。
抽象单位里的离散事件模型（E169 页「它答不了的」），不是本机计时。"""
from collections import defaultdict
from a_common import E169_THREE, rows


def main():
    verdicts = [f for _, f in rows(E169_THREE, "name=wait_verdict")]
    print(f"wait_verdict 行数 = {len(verdicts)}（含 r1、r2 两种读法）；以下只取 reading=r2_covered")
    groups = defaultdict(list)
    for v in verdicts:
        if v["reading"] == "r2_covered" and v["quantity"] == "Q2-3":
            groups[(v["q1"], v["q2"])].append(v)
    print("\nQ2-3：按 (Q1 候选, Q2 臂) 的 ρ 范围与判定条数（每组 = 2 剖面 × 2 取样点 × r∈{1,2} = 8 行）")
    for key in sorted(groups):
        rhos = [float(v["rho"]) for v in groups[key]]
        judgements = defaultdict(int)
        for v in groups[key]:
            judgements[v["judgement"]] += 1
        print(f"  {key[0]:32s} {key[1]:18s} 行数 {len(rhos)}  rho {min(rhos):.4f}–{max(rhos):.4f}  {dict(judgements)}")
    for point in ("p1_device_dominated", "p2_cpu_dominated"):
        print(f"\n取样点 {point}（uniform、r=1）上 W_max(k=8)、W_max(k=64)，只列甲整段与乙整段，按 Q1 候选：")
        for key in sorted(groups):
            if key[1] not in ("jia_whole", "yi_reserved_whole"):
                continue
            for v in groups[key]:
                if v["profile"] == "uniform" and v["point"] == point and v["r"] == "1":
                    print(f"  {key[0]:32s} {key[1]:18s} W_max(8)={float(v['w_max_k8']):8.3f}  "
                          f"W_max(64)={float(v['w_max_k64']):8.3f}  rho={float(v['rho']):.4f}")
    bounded = sum(1 for g in groups.values() for v in g if v["judgement"] == "bounded")
    grows = sum(1 for g in groups.values() for v in g if v["judgement"] == "grows")
    print(f"\n核：r2 读法 Q2-3 共 {sum(len(g) for g in groups.values())} 行，bounded {bounded}、grows {grows}")
    count = defaultdict(int)
    for _, p in rows(E169_THREE, "name=wait_pair"):
        if p["reading"] == "r2_covered":
            count[p["clause"]] += 1
    print("wait_pair（r2）按 clause：", dict(count))


if __name__ == "__main__":
    main()
