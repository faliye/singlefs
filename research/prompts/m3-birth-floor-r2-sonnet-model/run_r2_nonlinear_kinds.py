#!/usr/bin/env python3
"""run_r2_nonlinear 里 sc_all 红的单元按种类拆：it（实例表）还是用户可见单元。深度由参数给。python3 run_r2_nonlinear_kinds.py <深度>"""
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
import r2_variants as r
import run_r2_nonlinear as nl   # 导入时已把 r.candidates 换成不滤被抛弃的根
from run_r2_deep import ALPH
from collections import Counter
from multiprocessing import Pool

def walk(args):
    alpha, prefix, depth = args
    h = m.start()
    for op in prefix:
        h = m.step(h, op, False, True)
    c = Counter(); imgs = Counter()
    def rec(h, d):
        a = r.analyse2(h)
        if a["sc_all"]:
            kinds = {b[0] for (b, _) in a["sc_all"]}
            imgs["仅 it" if kinds == {"it"} else "含非 it 单元"] += 1
            for (b, _) in a["sc_all"]:
                c[b[0]] += 1
        if d == depth:
            return
        for op in alpha:
            n = m.step(h, op, False, True)
            if n is not None:
                rec(n, d + 1)
    rec(h, len(prefix))
    return c, imgs

if __name__ == "__main__":
    depth = int(sys.argv[1]); alpha = ALPH["ROLLKX"]
    prefixes = [(a, b) for a in alpha for b in alpha if (lambda h: h is not None and m.step(h, b, False, True) is not None)(m.step(m.start(), a, False, True))]
    C = Counter(); I = Counter()
    with Pool(7) as p:
        for c, i in p.imap_unordered(walk, [(alpha, pf, depth) for pf in prefixes]):
            C.update(c); I.update(i)
    print(f"sc_all 红的镜像按单元种类：{dict(I)}；红的单元按种类 {dict(C)}（深度 {depth}）")
