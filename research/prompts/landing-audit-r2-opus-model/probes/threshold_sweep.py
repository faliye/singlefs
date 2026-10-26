import json, sys
agents = [json.loads(l) for l in open(sys.argv[1])]
total = sum(a["cost"] for a in agents)
NO_EFFECT = {"report-unreferenced", "written-overwritten", "written-unused", "report-used-by-later-sub", "stopped-abandoned", "cut-by-usage-limit"}
def bucket(a, landed_at, partial_at, minimum):
    b = a["bucket"]
    if not a["handed_back"] or a["type"].startswith("three-way"):
        return b
    if a["written"] >= minimum and a["written"] > 0:
        share = a["landed"] / a["written"]
        if share >= landed_at: return "written-landed"
        if share >= partial_at: return "written-partial"
        return "written-overwritten" if a["used_main"] else "written-unused"
    if a["used_main"]: return "report-used-by-main"
    return "report-used-by-later-sub" if a["used_any"] else "report-unreferenced"
# 先核：照原门槛重算与账里的桶逐个相同
mismatch = sum(1 for a in agents if bucket(a, 0.7, 0.3, 10) != a["bucket"])
print("原门槛重算与账里桶不同的个数", mismatch)
print("LANDED_AT PARTIAL_AT WRITTEN_MINIMUM | 落地个数 落地花费 | 部分个数 部分花费 | 没效果几桶个数 没效果花费")
for landed_at, partial_at, minimum in [(0.7,0.3,10),(0.9,0.3,10),(0.5,0.3,10),(0.7,0.5,10),(0.7,0.1,10),(0.9,0.7,10),(0.95,0.9,10),(0.7,0.3,1),(0.7,0.3,30),(0.9,0.7,1)]:
    counts = {}
    for a in agents:
        b = bucket(a, landed_at, partial_at, minimum)
        c = counts.setdefault(b, [0, 0.0]); c[0] += 1; c[1] += a["cost"]
    ne = [sum(counts.get(b, [0, 0])[i] for b in NO_EFFECT) for i in (0, 1)]
    wl, wp = counts.get("written-landed", [0, 0]), counts.get("written-partial", [0, 0])
    print(f"{landed_at:<9} {partial_at:<10} {minimum:<15} | {wl[0]:>6} {wl[1]/total:>7.1%} | {wp[0]:>6} {wp[1]/total:>7.1%} | {ne[0]:>8} {ne[1]/total:>7.1%}")
# 落地率分布（写了 ≥10 行、交回、非三方）
shares = sorted(a["landed"]/a["written"] for a in agents if a["handed_back"] and not a["type"].startswith("three-way") and a["written"] >= 10)
import bisect
print("写了≥10 行的交回个数", len(shares), "落地率 <0.3:", bisect.bisect_left(shares,0.3), " [0.3,0.7):", bisect.bisect_left(shares,0.7)-bisect.bisect_left(shares,0.3), " [0.7,0.9):", bisect.bisect_left(shares,0.9)-bisect.bisect_left(shares,0.7), " ≥0.9:", len(shares)-bisect.bisect_left(shares,0.9))
