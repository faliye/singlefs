#!/usr/bin/env python3
"""E169 第一、二段：按 (Q1 候选, Q2 臂, Q3 臂) 汇总不合法前缀与四个 Q3 量。只读产物。"""
from collections import defaultdict
from a_common import E169_ONE, E169_TWO, rows, short

CANDIDATES = ("jia_whole", "jia_interleaved", "yi_reserved_whole")


def one_segment(label, name):
    verdicts = [fields for _, fields in rows(name, "name=verdict")]
    print(f"\n== E169 {label}（{name}）==")
    by_reading = {"r2_covered": [], "r1_issued": []}
    for v in verdicts:
        by_reading[v["reading"]].append({k: x for k, x in v.items() if k != "reading"})
    print(f"核：r2 {len(by_reading['r2_covered'])} 行、r1 {len(by_reading['r1_issued'])} 行；去掉 reading 后逐行相同 = "
          f"{by_reading['r2_covered'] == by_reading['r1_issued']}")
    q2 = defaultdict(dict)
    q3 = defaultdict(dict)
    for v in verdicts:
        if v["reading"] != "r2_covered":
            continue
        quantity = v["quantity"]
        if quantity in ("Q2-1", "Q2-2"):
            q2[(v["q1"], v["q2"])][quantity] = int(v["legal_states"])
        elif quantity in ("Q3-1", "Q3-2", "Q3-4"):
            q3[(v["q1"], v["q2"], short(v["q3"]))][quantity] = int(v["legal_states"])
        elif quantity == "Q3-3":
            for arm in ("bing0", "bing1", "bing2", "bing3"):
                q3[(v["q1"], v["q2"], arm)]["Q3-3"] = int(v[arm])
    print("Q2（Q2-1 = N2a 号连续而内容属于两个半截事务的状态数；Q2-2 = N2b 即丙3 判坏数）按 (Q1 候选, Q2 臂)：")
    for key in sorted(q2):
        print("  ", key, q2[key])
    print("Q3 非零格（Q3-1 fsync 已返回而恢复丢了；Q3-2 V_dep；Q3-3 V_dangling；Q3-4 丙2 部分施加）：")
    nonzero = defaultdict(list)
    for key in sorted(q3):
        if any(q3[key].get(q, 0) for q in ("Q3-1", "Q3-2", "Q3-3", "Q3-4")):
            print("  ", key, q3[key])
            nonzero[key[2]].append(key[:2])
    print("按恢复臂汇总（只数三条候选追加臂 jia_whole / jia_interleaved / yi_reserved_whole 上有非零量的 (Q1, Q2) 格，Q3-3 单独列）：")
    for arm in ("bing0", "bing1", "bing2", "bing3"):
        cells = [c for c in nonzero.get(arm, []) if c[1] in CANDIDATES]
        only33 = [c for c in cells if all(q3[(c[0], c[1], arm)].get(q, 0) == 0 for q in ("Q3-1", "Q3-2", "Q3-4"))]
        print(f"   {arm}: 非零格 {len(cells)}，其中只有 Q3-3 非零 {len(only33)}")
        for cell in cells:
            print("      ", cell, q3[(cell[0], cell[1], arm)])


def main():
    one_segment("第一段", E169_ONE)
    one_segment("第二段", E169_TWO)


if __name__ == "__main__":
    main()
