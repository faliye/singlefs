"""unstable 的几种读法：5 轮里不赔的轮数分布、多数读法、中位读法、只看后几轮。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, statistics
from e185lib import rows, floats
path = sys.argv[1]
points = [p for p in rows(path, "host_cpu_per_submission_point") if p["clock"] == "thread" and p["form"] != "today_single_kernel"]
def wins(p):
    return [j.startswith("not_losing") for j in p["judgement_per_round"].split(",")]
# 1 候选 × 卡：5 轮里不赔的轮数分布
dist = collections.Counter()
for p in points:
    dist[(p["candidate"], p["index"], sum(wins(p)))] += 1
for cand in ("scan_checksum_only", "scan_checksum_and_choice", "release_check_only", "every_stage"):
    for idx in ("0", "1"):
        print("name=rounds_not_losing candidate=%s card=%s %s" % (cand, idx, " ".join("k%d=%d" % (k, dist[(cand, idx, k)]) for k in range(6))))
# 2 多数读法（≥3 轮不赔）与中位读法（5 轮 c_rt 的中位 < 门槛）
for cand in ("scan_checksum_only", "scan_checksum_and_choice", "release_check_only", "every_stage"):
    for idx in ("0", "1"):
        sel = [p for p in points if p["candidate"] == cand and p["index"] == idx]
        major = sum(1 for p in sel if sum(wins(p)) >= 3)
        med = sum(1 for p in sel if statistics.median(floats(p["host_cpu_per_submission_milliseconds_per_round"])) < float(p["threshold_milliseconds"]))
        best = sum(1 for p in sel if min(floats(p["host_cpu_per_submission_milliseconds_per_round"])) < float(p["threshold_milliseconds"]))
        last = sum(1 for p in sel if floats(p["host_cpu_per_submission_milliseconds_per_round"])[-1] < float(p["threshold_milliseconds"]))
        print("name=reading candidate=%s card=%s cells=%d majority_not_losing=%d median_not_losing=%d any_round_not_losing=%d last_round_not_losing=%d" % (cand, idx, len(sel), major, med, best, last))
# 3 轮次走势：每格 5 轮 c_rt，第 5 轮 ÷ 第 1 轮
for idx in ("0", "1"):
    for wait in ("blocking", "sleep_100_microseconds", "sleep_1_millisecond"):
        sel = [p for p in points if p["index"] == idx and p["wait"] == wait and p["candidate"] == "scan_checksum_only" and p["threshold_kind"] == "attack_ledger"]
        per_round = list(zip(*[floats(p["host_cpu_per_submission_milliseconds_per_round"]) for p in sel]))
        meds = [statistics.median([v for v in r if v is not None]) for r in per_round]
        print("name=round_trend card=%s wait=%s points=%d median_c_rt_by_round_ms=%s" % (idx, wait, len(sel), ",".join("%.4f" % m for m in meds)))
