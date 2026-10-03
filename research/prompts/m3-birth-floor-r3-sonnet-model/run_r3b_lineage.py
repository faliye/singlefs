#!/usr/bin/env python3
"""python3 run_r3_lineage.py <字母表> <深度> [进程数]"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v3 as m
import r3b_lineage as L
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

def bad(n, length, seed):
    alpha = m.ALPHABETS["API"] + [("W",)] * 4 + [a for a in m.ALPHABETS["API"] if a[0] in "RF"]
    hs = m.sample_histories(alpha, n, length, seed)
    tot = {}
    for h in hs:
        cand = sorted(m.candidates(h), key=lambda x: x[0])
        rs = [m.refs(x) for x in cand]
        recs = L.records_of(h)
        for j in range(1, len(cand)):
            R, P = cand[j], cand[j - 1]
            if R[0] != P[0] + 1:
                continue
            seen = set().union(*rs[:j])
            fresh = [b for b in (rs[j] - seen) if b[1] == R[0]]
            if not fresh:
                continue
            fx = {b for b in fresh if b[0] == "fx"}
            ci = {b for b in (m.subtree(R[2]) if R[2] is not None else ()) if b in fresh}
            cx = {b for b in (m.subtree(R[3]) if R[3] is not None else ()) if b in fresh}
            modes = []
            if R[1] == "R":
                told = [t for (trb, t) in recs if trb == R[0]]
                if told and fx:
                    modes.append(("FX_in_rollback_root", fx, (), told[0]))
            else:
                for mode, units, kinds in (("M2both", ci | cx, ("I", "X")), ("M2x", cx, ("X",)), ("M2i", ci, ("I",))):
                    if units:
                        modes.append((mode, units, kinds, None))
            for mode, units, kinds, told in modes:
                for k in ((0,) if told is not None else (1, 2, 3)):
                    if told is not None:
                        bo = {b: told for b in units}   # 固定点单元的诞生代号改成 T_old
                        po = {}
                    else:
                        if R[0] - k < 0:
                            continue
                        bo = {b: b[1] - k for b in units}
                        po = {(R[0], kd): R[0] - k for kd in kinds}
                    a = L.analyse_lin(h, bo=bo, po=po)
                    row = tot.setdefault((mode, k), {"n": 0, **{x: 0 for x in L.LV}})
                    row["n"] += 1
                    for x in L.LV:
                        row[x] += int(any(jj == j for (_, jj) in a[x]))
    print(f"[r3 谱系写法·坏镜像] {len(hs)} 段随机合法历史（API 字母表，无故障），种子 {seed}；只在 R.txg = R⁻.txg + 1 的对上盖")
    keys = [x for x in L.LV if not x.endswith("_all")]
    print("  模式 k 镜像数 | " + " ".join(keys))
    for (mode, k), row in sorted(tot.items()):
        print(f"  {mode:20s} k={k} 镜像数 {row['n']:6d} | " + " ".join(f"{row[x]:6d}" for x in keys))

if __name__ == "__main__":
    if sys.argv[1] == "bad":
        bad(int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]))
        sys.exit(0)
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
