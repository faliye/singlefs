"""候选 C 赢在哪一段：T_A、T_B、T_C 拆成甲、丙、乙；同一格 A、B、C 的 c_rt 是否同一组数。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, statistics
from e185lib import rows, floats
path = sys.argv[1]
thr = {}
for r in rows(path, "candidate_threshold"):
    thr[(r["fraction"], r["pool_data_units"], r["batch_units"], r["candidate"])] = (float(r["saved_per_intent_milliseconds"]), int(r["submissions_per_intent"]), float(r["threshold_milliseconds"]))
for key in sorted({k[:3] for k in thr}, key=lambda k: (float(k[0]), int(k[1]), int(k[2]))):
    a = thr[key + ("scan_checksum_only",)]; b = thr[key + ("scan_checksum_and_choice",)]; c = thr[key + ("every_stage",)]; d = thr[key + ("release_check_only",)]
    first = a[2]; third = b[2] - a[2]; second = c[2] - b[2]
    print("name=threshold_split fraction=%s pool_data_units=%s batch_units=%s T_A=%.6f T_B=%.6f T_C=%.6f T_D=%.6f checksum_part=%.6f choice_part=%.6f scan_increment_part=%.6f scan_increment_share_of_T_C=%.4f submissions=%d" % (key + (a[2], b[2], c[2], d[2], first, third, second, second / c[2], a[1])))
points = [p for p in rows(path, "host_cpu_per_submission_point") if p["clock"] == "thread" and p["form"] != "today_single_kernel"]
by = collections.defaultdict(dict)
for p in points:
    k = (p["fraction"], p["pool_data_units"], p["batch_units"], p["index"], p["form"], p["wait"], p["gap_milliseconds"], p["threshold_kind"])
    by[k][p["candidate"]] = p
same = differ = 0
for k, d in by.items():
    vals = {c: d[c]["host_cpu_per_submission_milliseconds_per_round"] for c in ("scan_checksum_only", "scan_checksum_and_choice", "every_stage") if c in d}
    if len(set(vals.values())) == 1: same += 1
    else: differ += 1
print("name=same_c_rt_for_a_b_c groups=%d identical=%d different=%d" % (len(by), same, differ))
# C 不赔的格里，c_rt（5 轮中位）落在哪一段：< T_A、[T_A, T_B)、[T_B, T_C)
bucket = collections.Counter()
for k, d in by.items():
    c = d.get("every_stage")
    if c is None or c["judgement"] != "not_losing": continue
    med = statistics.median(floats(c["host_cpu_per_submission_milliseconds_per_round"]))
    ta = float(d["scan_checksum_only"]["threshold_milliseconds"]); tb = float(d["scan_checksum_and_choice"]["threshold_milliseconds"])
    zone = "below_T_A" if med < ta else ("below_T_B" if med < tb else "only_below_T_C")
    bucket[(k[3], zone)] += 1
for (idx, zone), n in sorted(bucket.items()):
    print("name=c_not_losing_cells_by_zone card=%s zone=%s cells=%d" % (idx, zone, n))
