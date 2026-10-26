"""主 agent 正文第四节那一句「每种等法 14–38 格」的口径：候选 C、B 按卡 × 等法 × 门槛种类数判定，含与不含今天的写法（form=today_single_kernel，登记说它不当第 1 行的读数）各数一遍。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows
points = rows(sys.argv[1], "host_cpu_per_submission_point")
for include_today in (False, True):
    c = collections.Counter()
    for p in points:
        if p["clock"] != "thread": continue
        if not include_today and p["form"] == "today_single_kernel": continue
        c[(p["candidate"], p["index"], p["wait"], p["threshold_kind"], p["judgement"])] += 1
    for cand in ("every_stage", "scan_checksum_and_choice"):
        for idx in "01":
            for w in ("blocking", "sleep_100_microseconds", "sleep_1_millisecond"):
                for t in ("attack_ledger", "machine_scaled"):
                    d = {j: n for (a, b, ww, tt, j), n in c.items() if (a, b, ww, tt) == (cand, idx, w, t)}
                    print("name=pivot_wait include_today=%s candidate=%s card=%s wait=%s threshold_kind=%s %s" % (include_today, cand, idx, w, t, " ".join("%s=%d" % kv for kv in sorted(d.items()))))
