#!/usr/bin/env python3
"""深枚举：受限字母表上穷举到深度 D（含任意位置、任意多次丢根槽 L、抛弃最新根 X、可写挂载 K、建 inode N、回退、抬 F），
每个历史前缀一张镜像，数各写法红的镜像数、首例。python3 run_r2_deep.py <字母表名> <深度> [进程数] > out
合法镜像上任一写法红 ⇒ 它在这个模型上出局（对应第一轮判决跑前判据）。"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
import r2_variants as r
from multiprocessing import Pool

ALPH = {
    # 只带回退相关的最小集：覆盖写、建 inode、三种回退目标、抬 F、丢根槽
    "ROLL": [("W",), ("N",)] + [("R", s) for s in (("o", 0), ("o", 1), ("n", 1), ("n", 2))] + [("F", x) for x in range(4, 10)] + [("L", k) for k in range(4)],
    # 再加挂载 K 与抛弃最新根 X
    "ROLLKX": [("W",), ("N",), ("K",), ("X",)] + [("R", s) for s in (("o", 0), ("n", 1))] + [("F", x) for x in range(4, 9)] + [("L", k) for k in range(3)],
}

def walk(args):
    alpha, prefix, depth = args
    h = m.start()
    for op in prefix:
        h = m.step(h, op, False, True)
        if h is None:
            return None
    stats = {k: 0 for k in r.VARS}
    stats["states"] = 0
    ex = {}
    def rec(h, d):
        stats["states"] += 1
        a = r.analyse2(h)
        for k in r.VARS:
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
    tot = {k: 0 for k in r.VARS}
    tot["states"] = 0
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
    print(f"[深枚举] 字母表 {name}（{len(alpha)} 个操作）深度 {depth}：前缀 {len(prefixes)} 个；检查的镜像（每个历史前缀一张）{tot['states']}")
    for k in r.VARS:
        print(f"  {k:14s} 红的镜像 {tot[k]}" + (f"  首例 {ex[k]}" if k in ex else ""))
    print(f"用时 {time.time() - t:.0f} 秒")
