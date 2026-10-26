"""候选 C 跨全部旋钮都不赔的点是哪几个；把 P = 8192 的乙段换成同 f、K 的 P = 1024 乙段（推的：区域扫描改按身份查之后乙不再随 P 涨）再判一遍。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows, floats
path = sys.argv[1]
points = [p for p in rows(path, "host_cpu_per_submission_point") if p["clock"] == "thread" and p["form"] != "today_single_kernel"]
thr = {(r["fraction"], r["pool_data_units"], r["batch_units"], r["candidate"]): float(r["threshold_milliseconds"]) for r in rows(path, "candidate_threshold")}
g = collections.defaultdict(set)
for p in points:
    if p["candidate"] == "every_stage":
        g[(p["fraction"], p["pool_data_units"], p["batch_units"], p["index"])].add(p["judgement"])
for k, js in sorted(g.items()):
    if js == {"not_losing"}:
        print("name=c_not_losing_at_every_knob fraction=%s pool_data_units=%s batch_units=%s card=%s" % k)
# 换乙：T_C' = T_B(P) + (T_C − T_B)(P = 1024)，门槛种类 machine_scaled 时乘 ρ
ratio = float(rows(path, "machine_ratio")[0]["ratio"])
c = collections.Counter()
for p in points:
    if p["candidate"] != "every_stage" or p["pool_data_units"] != "8192":
        continue
    f, K = p["fraction"], p["batch_units"]
    tb = thr[(f, "8192", K, "scan_checksum_and_choice")]
    second_small = thr[(f, "1024", K, "every_stage")] - thr[(f, "1024", K, "scan_checksum_and_choice")]
    t_new = tb + second_small
    if p["threshold_kind"] == "machine_scaled":
        t_new *= ratio
    per_round = ["not_losing" if v < t_new else "losing" for v in floats(p["host_cpu_per_submission_milliseconds_per_round"])]
    j = per_round[0] if len(set(per_round)) == 1 else "unstable"
    c[(p["index"], p["judgement"], j)] += 1
for (idx, old, new), n in sorted(c.items()):
    print("name=identity_index_pushed card=%s pool_data_units=8192 judgement_as_measured=%s judgement_with_the_scan_increment_of_1024=%s cells=%d" % (idx, old, new, n))
