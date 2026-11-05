#!/usr/bin/env python3
import math
# construction_seconds from research/results/e202-gpu-cpu-gc-comparison-2026-11-05/*.out and probe logs
each = {1024: 27.324, 2048: 184.946, 4096: 950.446}
shared = {1024: 35.790, 2048: 188.649, 4096: 646.824}
probe = {512: 11.033, 1024: 32.838, 2048: 145.311}
TARGET = 36608
def slope(a, b, d):
    return math.log(d[b] / d[a]) / math.log(b / a)
def loglog_fit(d):
    xs = [math.log(p) for p in d]; ys = [math.log(t) for t in d.values()]
    n = len(xs); mx = sum(xs)/n; my = sum(ys)/n
    b = sum((x-mx)*(y-my) for x, y in zip(xs, ys)) / sum((x-mx)**2 for x in xs)
    a = my - b*mx
    return b, a
def hours(s): return s / 3600
print("target/4096 =", TARGET/4096, " octaves beyond 4096 =", math.log2(TARGET/4096), " octaves sampled 1024..4096 = 2")
for name, d in (("each", each), ("shared", shared)):
    ks = sorted(d)
    s1 = slope(ks[0], ks[1], d); s2 = slope(ks[1], ks[2], d)
    b, a = loglog_fit(d)
    print(f"[{name}] slope 1024->2048={s1:.3f} 2048->4096={s2:.3f} LSfit slope={b:.3f}")
    last = d[4096]
    for label, e in (("top-octave slope", s2), ("LS-fit slope", b), ("1024->2048 slope", s1), ("pure P^2", 2.0)):
        t = last * (TARGET/4096) ** e
        print(f"   anchored at P=4096, exponent {e:.3f} ({label}): {t:12.0f} s = {hours(t):7.1f} h")
    t = math.exp(a) * TARGET ** b
    print(f"   LS-fit curve itself at 36608: {t:12.0f} s = {hours(t):7.1f} h")
print("probe-run (with probe overhead) slopes: 512->1024", slope(512,1024,probe), " 1024->2048", slope(1024,2048,probe))
# spread at same P
print("P=1024 spread", max(27.324,35.790,32.838)/min(27.324,35.790,32.838))
print("P=2048 spread", max(184.946,188.649,145.311,149.637)/min(184.946,188.649,145.311,149.637))
print("P=4096 spread", 950.446/646.824)
# publishes and units scaling for structural model
for p, pub, u in ((1024,1670,1715),(2048,3302,3386),(4096,6566,6724)):
    print(p, "publishes/P", pub/p, "U_final/P", u/p)
