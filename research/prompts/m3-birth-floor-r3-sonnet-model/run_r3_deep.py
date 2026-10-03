#!/usr/bin/env python3
"""python3 run_r3_deep.py <字母表> <深度> [进程数]：深枚举，每个历史前缀一张镜像，数 r3_variants 各写法 × 各缺口处置红的镜像数。"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v3 as m
import r3_variants as r
from multiprocessing import Pool

ALPH = {
    "ROLL": [("W",), ("N",)] + [("R", s) for s in (("o", 0), ("o", 1), ("n", 1), ("n", 2))] + [("F", x) for x in range(4, 10)] + [("L", k) for k in range(4)],
    "ROLLKX": [("W",), ("N",), ("K",), ("X",)] + [("R", s) for s in (("o", 0), ("n", 1))] + [("F", x) for x in range(4, 9)] + [("L", k) for k in range(3)],
}
KEYS = [f"{v}|{g}" for v in r.VARS for g in r.GAPS]

def walk(args):
    alpha, prefix, depth = args
    h = m.start()
    for op in prefix:
        h = m.step(h, op, False, True)
        if h is None:
            return None
    stats = {k: 0 for k in KEYS}
    for k in ("states", "pairs", "skipped_pairs", "cells", "cells_in_skipped", "images_with_gap"):
        stats[k] = 0
    ex = {}
    def rec(h, d):
        stats["states"] += 1
        a = r.analyse3(h)
        stats["pairs"] += a["_pairs"]; stats["skipped_pairs"] += a["_skipped_pairs"]
        stats["cells"] += a["_cells"]; stats["cells_in_skipped"] += a["_cells_in_skipped"]
        if a["_skipped_pairs"]:
            stats["images_with_gap"] += 1
        for k in KEYS:
            if a[k]:
                stats[k] += 1
                ex.setdefault(k, (h.ops, a[k][:1]))
        if d == depth:
            return
        for op in alpha:
            n = m.step(h, op, False, True)
            if n is not None:
                rec(n, d + 1)
    rec(h, len(prefix))
    return stats, ex

if __name__ == "__main__":
    name, depth = sys.argv[1], int(sys.argv[2])
    procs = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    alpha = ALPH[name]
    prefixes = []
    for a in alpha:
        h = m.step(m.start(), a, False, True)
        if h is None:
            continue
        for b in alpha:
            if m.step(h, b, False, True) is not None:
                prefixes.append((a, b))
    t = time.time()
    tot = {k: 0 for k in KEYS}
    for k in ("states", "pairs", "skipped_pairs", "cells", "cells_in_skipped", "images_with_gap"):
        tot[k] = 0
    ex = {}
    with Pool(procs) as p:
        for res in p.imap_unordered(walk, [(alpha, pf, depth) for pf in prefixes], chunksize=1):
            if res is None:
                continue
            s, e = res
            for k in tot:
                tot[k] += s[k]
            for k, v in e.items():
                ex.setdefault(k, v)
    print(f"[r3 深枚举] 字母表 {name}（{len(alpha)} 个操作）深度 {depth}：前缀 {len(prefixes)} 个；镜像 {tot['states']}")
    print(f"  相邻对 {tot['pairs']}，gap=all 跳过的对 {tot['skipped_pairs']}（含缺口的镜像 {tot['images_with_gap']}），新单元 {tot['cells']}，其中在被跳过的对里 {tot['cells_in_skipped']}")
    for k in KEYS:
        print(f"  {k:14s} 红的镜像 {tot[k]}")
    for k in KEYS:
        if k in ex:
            print(f"  首例 {k}: {ex[k]}")
    print(f"用时 {time.time() - t:.0f} 秒")
