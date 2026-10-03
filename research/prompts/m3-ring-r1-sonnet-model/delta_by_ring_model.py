#!/usr/bin/env python3
"""正推腿的第二条路：按环与树表寿命的定义重算一遍 E156 的 q1a（树表多扣个数）与 h_缺，跟产物里 name=anchor_dump 的行逐行比，
再把同一段历史里的洞补上（崩溃那次的发布当成根槽落了盘的发布）看多扣还剩多少。

用法：python3 delta_by_ring_model.py <E156 产物 .out>
定义（只取 kb / 产物里写明的，不读装置源码里的计算）：
- 环：txg % (3S) 是环里的位置，每次有根的发布盖掉那个位置；mkfs 的第 0 代根种在位置 0（以及三个区域的槽 0，这里只看它会不会晚于 3S 才被盖，取位置 0、S、2S）。
- 树表：暖机的 txg 1、2 是零单元发布（不换树表）；其余每次发布 t 诞生树表 t、换下上一张（释放代 t）。
- 门槛 = 环里最旧有效根的 txg（F 取 0）；扣住 = 释放代 > 门槛（或还没释放）；被引用 = 环里有根 r 满足 诞生 <= r < 释放（未释放的一直被引用）。
- 多扣 = 扣住的张数 - 被引用的张数。h_缺 = [环里最旧根, 当前 txg] 里环里没有 txg 等于它的根的个数。
- 一个洞 k=1：前 = 第一个洞落 txg 5；后 = 落 3S+3+1；洞之后一次写行（洞+1），暖机到两块盘都覆盖为止（区域 txg%3 归盘 0/1/0），之后都是普通发布，跑到洞 + 3S + 2。
"""
import re
import sys

DEVICE_OF_REGION = [0, 1, 0]


def series(S, hole, patched, end_extra=2):
    L = 3 * S
    roles = {t: "setup" for t in range(0, 4)}
    t = 3
    while t + 1 < hole:
        t += 1
        roles[t] = "work"
    t += 1
    roles[t] = "crash"
    t += 1
    roles[t] = "row"
    covered = {DEVICE_OF_REGION[t % 3]}
    while len(covered) < 2:
        t += 1
        roles[t] = "warm"
        covered.add(DEVICE_OF_REGION[t % 3])
    end = hole + L + end_extra
    for u in range(t + 1, end + 1):
        roles[u] = "work"
    ring = {0: 0, S: 0, 2 * S: 0}
    tables = {0: None}
    live = 0
    rows = []
    for txg in range(0, end + 1):
        if txg > 0 and txg not in (1, 2):
            tables[live] = txg
            tables[txg] = None
            live = txg
        if roles[txg] != "crash" or patched:
            ring[txg % L] = txg
        if roles[txg] == "crash" or txg < 3:
            continue
        roots = sorted(set(ring.values()))
        oldest = roots[0]
        floor = oldest
        withheld = {a for a, r in tables.items() if r is None or r > floor}
        referenced = {a for a, r in tables.items()
                      if r is None or any(a <= x < r for x in roots)}
        missing = sum(1 for c in range(oldest, txg + 1) if c not in roots)
        rows.append((txg, floor, missing, len(withheld - referenced)))
    return rows


def main():
    text = open(sys.argv[1], encoding="utf-8").read().splitlines()
    dump = {}
    for line in text:
        m = re.match(r"E7RESULT name=anchor_dump S=(\d+) k=(\d+) pos=(\S+) fits=\S+ txg=(\d+) floor=(\d+) h_que=(\d+) h_dong=\d+ q1a_tree_table_lower_bound=(\d+)", line)
        if m:
            S, k, position, txg, floor, h_que, q1a = m.groups()
            dump.setdefault((int(S), int(k), position), {})[int(txg)] = (int(floor), int(h_que), int(q1a))
    for S in (4, 8, 16):
        for position, hole in (("front", 5), ("back", 3 * S + 3 + 1)):
            unpatched = series(S, hole, patched=False)
            patched = series(S, hole, patched=True)
            ref = dump[(S, 1, position)]
            compared = 0
            mismatched = 0
            for txg, floor, missing, delta in unpatched:
                if txg in ref:
                    compared += 1
                    if ref[txg] != (floor, missing, delta):
                        mismatched += 1
            peak = max(d for *_, d in unpatched)
            peak_patched = max(d for *_, d in patched)
            missing_patched = max(m for _, _, m, _ in patched)
            print(f"S={S:2d} {position:5s} 洞 txg={hole:3d}: 与 anchor_dump 逐行比 {compared} 行、不符 {mismatched} 行；"
                  f"没补：树表多扣峰值 {peak}、h_缺峰值 {max(m for _, _, m, _ in unpatched)}；补了：树表多扣峰值 {peak_patched}、h_缺峰值 {missing_patched}")


if __name__ == "__main__":
    main()
