"""不挪出调用线程、只在调用线程上用 GPU：一次提交的挂钟（wall_median）按卡、等法、间隔、提交大小取 5 轮中位，给前台每次越线多等多少。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, statistics
from e185lib import rows
path = sys.argv[1]
d = collections.defaultdict(list)
for r in rows(path, "submission_cell"):
    if r["invalid_reason"] != "none" or r["form"] == "today_single_kernel": continue
    d[(r["index"], r["form"], r["wait"], r["gap_milliseconds"], int(r["units"]))].append(float(r["wall_median_nanoseconds"]) / 1e6)
for k in sorted(d):
    if k[4] in (4, 8, 16, 24, 32):
        v = d[k]
        print("name=submission_wall card=%s form=%s wait=%s gap_milliseconds=%s units=%d rounds=%d wall_ms_median=%.3f wall_ms_max=%.3f" % (k + (len(v), statistics.median(v), max(v))))
