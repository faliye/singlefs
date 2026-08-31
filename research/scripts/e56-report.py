#!/usr/bin/env python3
"""E56 汇总：每 (mode, cache, nleaf, arm, eps) 取种子中位数与极差。

用法：python3 research/scripts/e56-report.py <跑批输出>
"""
import sys, re, statistics as st
from collections import defaultdict

rows = []
key = None
for line in open(sys.argv[1]):
    m = re.match(r"=== mode=(\w+) seed=(\d+) cache=(\d+) nleaf=(\d+) ===", line)
    if m:
        key = dict(mode=m.group(1), seed=int(m.group(2)), cache=int(m.group(3)), nleaf=int(m.group(4)))
        continue
    if not line.startswith("E7RESULT ") or key is None:
        continue
    kv = dict(p.split("=", 1) for p in line.split()[1:] if "=" in p)
    kv.update(key)
    rows.append(kv)

def num(x):
    try: return float(x)
    except: return None

upd = defaultdict(list); qry = defaultdict(list)
for r in rows:
    if "eps" not in r: continue
    k = (r["mode"], r["cache"], r["nleaf"], r.get("arm"), int(r["eps"]))
    if r.get("name") == "update": upd[k].append(r)
    elif r.get("name") == "query": qry[k].append(r)

def med(rs, f):
    v = [num(r[f]) for r in rs if f in r and num(r[f]) is not None]
    return st.median(v) if v else None
def spread(rs, f):
    v = [num(r[f]) for r in rs if f in r and num(r[f]) is not None]
    if not v or st.median(v) == 0: return 0.0
    return (max(v)-min(v))/st.median(v)
def fmt(x, p=4):
    return "—" if x is None else f"{x:.{p}f}"

for g in sorted({(k[0], k[1], k[2]) for k in upd}):
    base = med(upd.get((g[0], g[1], g[2], "logstruct_wb", 0), []), "io_per_op")
    print(f"\n## mode={g[0]} cache={g[1]} nleaf={g[2]}   基线(现方向)={fmt(base,6)}")
    print("arm/eps | F | B | H | io/op | 极差 | 排空后 | 残留% | vs基线 | vs基线(排空) | 热查 | 冷查 | n")
    ks = [k for k in upd if (k[0],k[1],k[2])==g]
    for k in sorted(ks, key=lambda k:(k[3] != "sorted_bplus", k[3] != "logstruct_wb", k[4])):
        rs = upd[k]; qs = qry.get(k, [])
        io = med(rs,"io_per_op"); dr = med(rs,"io_per_op_drained")
        lbl = k[3] if k[4] == 0 else f"eps={k[4]/1000:.3f}"
        rf = med(rs,'residual_frac')
        print(" | ".join([lbl,
            fmt(med(rs,'fanout'),0), fmt(med(rs,'buf'),0), fmt(med(rs,'height'),0),
            fmt(io,6), f"{spread(rs,'io_per_op')*100:.2f}%",
            fmt(dr,6), "—" if rf is None else f"{rf*100:.2f}",
            fmt(base/io,2) if base and io else "—",
            fmt(base/dr,2) if (base and dr) else "—",
            fmt(med(qs,'hot_reads_per_query')), fmt(med(qs,'cold_reads_per_query')),
            str(len(rs))]))
