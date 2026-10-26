"""machine_scaled 门槛用的 ρ 取的是段开头的校准；段末校准快了 12%（drifted=true）。卡 1 在段的后半量，换成段末的 ρ 再判 machine_scaled 那几行。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows, floats
path = sys.argv[1]
mr = rows(path, "machine_ratio")[0]
start = float(mr["local_baseline_nanoseconds"]); peer = float(mr["peer_baseline_nanoseconds"])
end = float([r for r in rows(path, "calibration_baseline") if r["position"] == "end"][0]["median_nanoseconds"])
rho_start, rho_end = start / peer, end / peer
print("name=machine_ratio_at_both_ends rho_start=%.6f rho_end=%.6f" % (rho_start, rho_end))
c = collections.Counter()
for p in rows(path, "host_cpu_per_submission_point"):
    if p["clock"] != "thread" or p["form"] == "today_single_kernel" or p["threshold_kind"] != "machine_scaled": continue
    t = float(p["threshold_milliseconds"]) * rho_end / rho_start
    js = ["not_losing" if v < t else "losing" for v in floats(p["host_cpu_per_submission_milliseconds_per_round"])]
    j = js[0] if len(set(js)) == 1 else "unstable"
    old = p["judgement"].replace("_lower_bound", "")
    c[(p["candidate"], p["index"], old, j)] += 1
for k, n in sorted(c.items()):
    if k[2] != k[3]:
        print("name=machine_scaled_with_rho_end candidate=%s card=%s judgement_with_rho_start=%s judgement_with_rho_end=%s cells=%d" % (k + (n,)))
