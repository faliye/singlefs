#!/usr/bin/env python3
"""python3 run_r3_lineage.py <字母表> <深度> [进程数]"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v3 as m
import r3_lineage as L
from run_r3_deep import ALPH
from multiprocessing import Pool

def walk(args):
    alpha, prefix, depth = args
    h = m.start()
    for op in prefix:
        h = m.step(h, op, False, True)
        if h is None:
            return None
    st = {k: 0 for k in L.LV}
    st["states"] = 0
    ex = {}
    def rec(h, d):
        st["states"] += 1
        a = L.analyse_lin(h)
        for k in L.LV:
            if a[k]:
                st[k] += 1
                ex.setdefault(k, (h.ops, a[k][:1]))
        if d == depth:
            return
        for op in alpha:
            n = m.step(h, op, False, True)
            if n is not None:
                rec(n, d + 1)
    rec(h, len(prefix))
    return st, ex

if __name__ == "__main__":
    name, depth = sys.argv[1], int(sys.argv[2])
    procs = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    alpha = ALPH[name]
    prefixes = [(a, b) for a in alpha for b in alpha if (lambda h: h is not None and m.step(h, b, False, True) is not None)(m.step(m.start(), a, False, True))]
    tot = {k: 0 for k in L.LV}
    tot["states"] = 0
    ex = {}
    t = time.time()
    with Pool(procs) as p:
        for res in p.imap_unordered(walk, [(alpha, pf, depth) for pf in prefixes], chunksize=1):
            if res is None:
                continue
            s, e = res
            for k in tot:
                tot[k] += s[k]
            for k, v in e.items():
                ex.setdefault(k, v)
    print(f"[r3 谱系写法·合法镜像] 字母表 {name} 深度 {depth}：镜像 {tot['states']}")
    for k in L.LV:
        print(f"  {k:18s} 红的镜像 {tot[k]}" + (f"  首例 {ex[k]}" if k in ex else ""))
    print(f"用时 {time.time() - t:.0f} 秒")
