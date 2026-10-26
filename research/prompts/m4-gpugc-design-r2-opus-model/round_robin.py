"""多卡按窗口轮转（正推推荐）：一条意图的提交轮流交卡 0、卡 1，一个点的 c_rt 取两张卡同一格同一轮的平均；与只用卡 0 比，逐格判。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows, floats
path = sys.argv[1]
pts = collections.defaultdict(dict)
for p in rows(path, "host_cpu_per_submission_point"):
    if p["clock"] != "thread" or p["form"] == "today_single_kernel": continue
    k = (p["fraction"], p["pool_data_units"], p["batch_units"], p["candidate"], p["form"], p["wait"], p["gap_milliseconds"], p["threshold_kind"])
    pts[k][p["index"]] = p
c = collections.Counter()
for k, d in pts.items():
    if "0" not in d or "1" not in d: continue
    t = float(d["0"]["threshold_milliseconds"])
    v0 = floats(d["0"]["host_cpu_per_submission_milliseconds_per_round"]); v1 = floats(d["1"]["host_cpu_per_submission_milliseconds_per_round"])
    js = ["not_losing" if (a + b) / 2 < t else "losing" for a, b in zip(v0, v1)]
    j = js[0] if len(set(js)) == 1 else "unstable"
    c[(k[3], d["0"]["judgement"].replace("_lower_bound", ""), j)] += 1
for (cand, alone, rr), n in sorted(c.items()):
    print("name=round_robin_two_cards candidate=%s judgement_card0_alone=%s judgement_round_robin=%s cells=%d" % (cand, alone, rr, n))
