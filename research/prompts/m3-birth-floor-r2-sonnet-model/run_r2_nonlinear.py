#!/usr/bin/env python3
"""候选集不是线性链时（被抛弃的根没被滤掉，例如最新根的实例表读不出、一行都没有，walk.rs 里「最新根的实例表读不出时一行都没有、不滤」那种）
第一次出现版（α 的下界半，sc/scp）与相邻差集版（β，sc_all/scp_all）各红多少。被抛弃的根留在候选集里、其余照旧。
python3 run_r2_nonlinear.py <深度> [进程数]"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
import r2_variants as r
from run_r2_deep import ALPH
from multiprocessing import Pool

def unfiltered(h):
    cur = m.cur_root(h)
    return [x for x in h.roots[-m.RING:] if ((x[0] >= h.floor and x[0] not in h.lost) or x is cur)]
r.candidates = unfiltered
KEYS = ("plain", "plain_all", "sc", "sc_all", "scp", "scp_all")

def walk(args):
    alpha, prefix, depth = args
    h = m.start()
    for op in prefix:
        h = m.step(h, op, False, True)
        if h is None:
            return None
    st = {k: 0 for k in KEYS}
    st["states"] = 0
    st["有被抛弃的根留在候选集"] = 0
    ex = {}
    def rec(h, d):
        st["states"] += 1
        if h.ab:
            st["有被抛弃的根留在候选集"] += 1
        a = r.analyse2(h)
        for k in KEYS:
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
    depth = int(sys.argv[1])
    procs = int(sys.argv[2]) if len(sys.argv) > 2 else 7
    alpha = ALPH["ROLLKX"]
    prefixes = [(a, b) for a in alpha for b in alpha if (lambda h: h is not None and m.step(h, b, False, True) is not None)(m.step(m.start(), a, False, True))]
    tot = {k: 0 for k in KEYS}
    tot["states"] = 0
    tot["有被抛弃的根留在候选集"] = 0
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
    print(f"[候选集不滤被抛弃的根] 字母表 ROLLKX 深度 {depth}：镜像 {tot['states']}，其中被抛弃的根留在候选集里的 {tot['有被抛弃的根留在候选集']}")
    for k in KEYS:
        print(f"  {k:10s} 红的镜像 {tot[k]}" + (f"  首例 {ex[k]}" if k in ex else ""))
    print(f"用时 {time.time() - t:.0f} 秒")
