"""同一格五轮的 c_rt 随轮次走：每格第 5 轮 ÷ 第 1 轮、五轮与轮号的秩相关；同一格前后校准比（作废谓词）的范围。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, statistics
from e185lib import rows
path = sys.argv[1]
cells = collections.defaultdict(dict); cal = collections.defaultdict(list)
for r in rows(path, "submission_cell"):
    if r["invalid_reason"] != "none": continue
    if r["form"] == "today_single_kernel": continue
    k = (r["index"], r["form"], r["wait"], r["gap_milliseconds"], r["units"])
    cells[k][int(r["round"])] = (float(r["thread_cpu_median_nanoseconds"]), float(r["floor_thread_cpu_median_nanoseconds"]))
    cal[r["index"]].append(float(r["calibration_ratio_before"])); cal[r["index"]].append(float(r["calibration_ratio_after"]))
def rank(v):
    s = sorted(range(len(v)), key=lambda i: v[i]); out = [0]*len(v)
    for pos, i in enumerate(s): out[i] = pos
    return out
for idx in ("0", "1"):
    ratios = []; rhos = []; floor_ratios = []; spread = []
    for k, d in cells.items():
        if k[0] != idx or len(d) != 5: continue
        v = [d[r][0] for r in range(1, 6)]; fl = [d[r][1] for r in range(1, 6)]
        ratios.append(v[4] / v[0]); floor_ratios.append(fl[4] / fl[0]); spread.append(max(v) / min(v))
        rv = rank(v); rr = list(range(5)); n = 5
        rhos.append(1 - 6 * sum((a - b) ** 2 for a, b in zip(rv, rr)) / (n * (n * n - 1)))
    print("name=round_drift card=%s cells=%d round5_over_round1_median=%.3f round5_below_round1=%d floor_round5_over_round1_median=%.3f max_over_min_of_five_rounds_median=%.3f max_over_min_maximum=%.3f spearman_with_round_median=%.2f spearman_at_most_minus_0_7=%d calibration_ratio_min=%.4f calibration_ratio_max=%.4f" % (
        idx, len(ratios), statistics.median(ratios), sum(1 for x in ratios if x < 1), statistics.median(floor_ratios), statistics.median(spread), max(spread), statistics.median(rhos), sum(1 for x in rhos if x <= -0.7), min(cal[idx]), max(cal[idx])))
