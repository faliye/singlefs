"""CPU 模式的 CRC 换成本机量到的 SSE4.2 指令速度之后（软件八表 ÷ 指令的倍数由 crc/crc_speed.out 给），各候选门槛里 CRC 那一部分按倍数缩，逐格再判。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, re
from e185lib import rows, floats
path, speed_file = sys.argv[1], sys.argv[2]
speed = {m.group(1): float(m.group(2)) for m in (re.search(r"implementation=(\S+).*thread_cpu_median_nanoseconds=(\S+)", l) for l in open(speed_file)) if m}
factor = speed["software_eight_tables"] / speed["sse42_instruction"]
print("name=hardware_crc_factor software_over_instruction=%.4f" % factor)
thr = {(r["fraction"], r["pool_data_units"], r["batch_units"], r["candidate"]): float(r["threshold_milliseconds"]) for r in rows(path, "candidate_threshold")}
ratio = float(rows(path, "machine_ratio")[0]["ratio"])
def new_threshold(f, P, K, cand):
    a = thr[(f, P, K, "scan_checksum_only")]
    if cand == "scan_checksum_only": return a / factor
    if cand == "release_check_only": return thr[(f, P, K, cand)] / factor
    return thr[(f, P, K, cand)] - a + a / factor
c = collections.Counter()
for p in rows(path, "host_cpu_per_submission_point"):
    if p["clock"] != "thread" or p["form"] == "today_single_kernel": continue
    t = new_threshold(p["fraction"], p["pool_data_units"], p["batch_units"], p["candidate"])
    if p["threshold_kind"] == "machine_scaled": t *= ratio
    js = ["not_losing" if v < t else "losing" for v in floats(p["host_cpu_per_submission_milliseconds_per_round"])]
    j = js[0] if len(set(js)) == 1 else "unstable"
    c[(p["candidate"], p["index"], j)] += 1
for cand in ("scan_checksum_only", "scan_checksum_and_choice", "release_check_only", "every_stage"):
    for idx in ("0", "1"):
        d = {j: n for (a, b, j), n in c.items() if a == cand and b == idx}
        print("name=judgement_with_the_instruction_crc candidate=%s card=%s %s" % (cand, idx, " ".join("%s=%d" % kv for kv in sorted(d.items()))))
