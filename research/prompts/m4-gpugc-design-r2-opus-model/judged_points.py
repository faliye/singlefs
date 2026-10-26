"""照登记 J4（随旋钮翻面的点第 1 行不当已判）逐点数：每个 (f, P, K) × 候选 × 卡，跨输入形态、等法、间隔、门槛机器的判定集合。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows
path = sys.argv[1]
points = [p for p in rows(path, "host_cpu_per_submission_point") if p["clock"] == "thread" and p["form"] != "today_single_kernel"]
def gather(sel):
    g = collections.defaultdict(set)
    for p in points:
        if sel(p):
            g[(p["fraction"], p["pool_data_units"], p["batch_units"], p["candidate"], p["index"])].add(p["judgement"])
    return g
for label, sel in (("all_knobs", lambda p: True),
                   ("gap_50_only", lambda p: p["gap_milliseconds"] == "50"),
                   ("gap_0_only", lambda p: p["gap_milliseconds"] == "0"),
                   ("shared_gap_0_machine_scaled", lambda p: p["gap_milliseconds"] == "0" and p["form"] == "shared" and p["threshold_kind"] == "machine_scaled")):
    g = gather(sel)
    c = collections.Counter()
    for (f, P, K, cand, idx), js in g.items():
        verdict = next(iter(js)) if len(js) == 1 else "flips_with_a_knob"
        c[(cand, idx, verdict)] += 1
    for cand in ("scan_checksum_only", "scan_checksum_and_choice", "release_check_only", "every_stage"):
        for idx in ("0", "1"):
            d = {v: n for (a, b, v), n in c.items() if a == cand and b == idx}
            print("name=judged_points selection=%s candidate=%s card=%s points=%d %s" % (label, cand, idx, sum(d.values()), " ".join("%s=%d" % kv for kv in sorted(d.items()))))
