"""第四节透视：候选 × 卡 × 等法 × 门槛种类，另按输入形态、间隔、钟拆。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sys, collections
from e185lib import rows
path = sys.argv[1]
points = rows(path, "host_cpu_per_submission_point")
print("name=pivot_rows total=%d" % len(points))
for key in ("clock", "form", "gap_milliseconds", "wait", "threshold_kind", "candidate", "index"):
    print("name=pivot_values key=%s values=%s" % (key, ",".join(sorted({p[key] for p in points}))))
def table(filter_fn, label):
    c = collections.Counter()
    for p in points:
        if filter_fn(p):
            c[(p["candidate"], p["index"], p["judgement"])] += 1
    for cand in ("scan_checksum_only", "scan_checksum_and_choice", "release_check_only", "every_stage"):
        for idx in ("0", "1"):
            parts = {j: n for (cc, ii, j), n in c.items() if cc == cand and ii == idx}
            print("name=pivot label=%s candidate=%s card=%s %s" % (label, cand, idx, " ".join("%s=%d" % kv for kv in sorted(parts.items()))))
table(lambda p: p["clock"] == "thread", "thread_all_forms_all_gaps")
table(lambda p: p["clock"] == "process", "process_all_forms_all_gaps")
for form in sorted({p["form"] for p in points}):
    table(lambda p, f=form: p["clock"] == "thread" and p["form"] == f, "thread_form_" + form)
for gap in ("0", "50"):
    table(lambda p, g=gap: p["clock"] == "thread" and p["gap_milliseconds"] == g, "thread_gap_" + gap)
