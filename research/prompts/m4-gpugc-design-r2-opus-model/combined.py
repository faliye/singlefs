"""候选 C 在 CPU 模式两处改进叠加（CRC 用 SSE4.2 指令、P = 8192 的乙段取同 f、K 的 P = 1024 值）之后，按间隔分开逐格再判；乙、丙输入的上传不加（对 C 有利的一侧）。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections, re
from e185lib import rows, floats
path, speed_file = sys.argv[1], sys.argv[2]
speed = {m.group(1): float(m.group(2)) for m in (re.search(r"implementation=(\S+).*thread_cpu_median_nanoseconds=(\S+)", l) for l in open(speed_file)) if m}
factor = speed["software_eight_tables"] / speed["sse42_instruction"]
thr = {(r["fraction"], r["pool_data_units"], r["batch_units"], r["candidate"]): float(r["threshold_milliseconds"]) for r in rows(path, "candidate_threshold")}
ratio = float(rows(path, "machine_ratio")[0]["ratio"])
c = collections.Counter()
for p in rows(path, "host_cpu_per_submission_point"):
    if p["clock"] != "thread" or p["form"] == "today_single_kernel" or p["candidate"] != "every_stage": continue
    f, P, K = p["fraction"], p["pool_data_units"], p["batch_units"]
    a = thr[(f, P, K, "scan_checksum_only")]; b = thr[(f, P, K, "scan_checksum_and_choice")]
    second = thr[(f, "1024", K, "every_stage")] - thr[(f, "1024", K, "scan_checksum_and_choice")]
    t = a / factor + (b - a) + second
    if p["threshold_kind"] == "machine_scaled": t *= ratio
    js = ["not_losing" if v < t else "losing" for v in floats(p["host_cpu_per_submission_milliseconds_per_round"])]
    j = js[0] if len(set(js)) == 1 else "unstable"
    c[(p["index"], p["gap_milliseconds"], j)] += 1
for k, n in sorted(c.items()):
    print("name=every_stage_after_both_cpu_improvements card=%s gap_milliseconds=%s judgement=%s cells=%d" % (k + (n,)))
