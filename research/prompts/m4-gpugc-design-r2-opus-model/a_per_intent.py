"""候选 A 若照乙-1（一条意图窗口开头交一次 m 个单元）：c_rt(m) 对 A 一条意图省下的（saved_per_intent），逐格逐轮判；merging_shape 只对 C 的门槛判过，这里对 A 补一遍。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows
path = sys.argv[1]
ratio = float(rows(path, "machine_ratio")[0]["ratio"])
saved = {(r["fraction"], r["pool_data_units"], r["batch_units"], r["candidate"]): float(r["saved_per_intent_milliseconds"]) for r in rows(path, "candidate_threshold")}
cell = {}
for r in rows(path, "submission_cell"):
    if r["invalid_reason"] != "none":
        continue
    cell[(r["index"], r["form"], r["wait"], r["gap_milliseconds"], r["units"], r["round"])] = float(r["thread_cpu_median_nanoseconds"]) / 1e6
c = collections.Counter()
for (f, P, K, cand), s in saved.items():
    if cand != "scan_checksum_only":
        continue
    m = "8" if f == "0.25" else "24"
    for idx in ("0", "1"):
        for form in ("shared", "split"):
            for wait in ("blocking", "sleep_100_microseconds", "sleep_1_millisecond"):
                for gap in ("0", "50"):
                    for kind, scale in (("attack_ledger", 1.0), ("machine_scaled", ratio)):
                        vals = [cell.get((idx, form, wait, gap, m, str(r))) for r in range(1, 6)]
                        if None in vals:
                            continue
                        js = ["not_losing" if v < s * scale else "losing" for v in vals]
                        j = js[0] if len(set(js)) == 1 else "unstable"
                        c[(idx, gap, j)] += 1
for k, n in sorted(c.items()):
    print("name=scan_checksum_only_one_submission_per_intent card=%s gap_milliseconds=%s judgement=%s cells=%d" % (k + (n,)))
