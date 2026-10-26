"""候选 C 的乙、丙要的输入（映射表、分配记录）没进 c_rt：按同一格拟合的每字节部分补上传那一份，再判 C（推的：每条映射 60 字节、每条分配记录 20 字节，每批重扫所以映射每次提交都传，分配记录只在签发那一次传）。"""
import os, sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collections
from e185lib import rows, floats
path = sys.argv[1]
MAPPING_ENTRY_BYTES = 60
ALLOCATION_RECORD_BYTES = 20
fits = {(r["index"], r["form"], r["wait"], r["gap_milliseconds"]): r for r in rows(path, "fixed_and_per_byte_fit")}
for k, r in sorted(fits.items()):
    print("name=fit index=%s form=%s wait=%s gap=%s fixed_ms=%s per_mebibyte_ms=%s maximum_relative_residual=%s fit_not_linear=%s" % (k + (r["fixed_milliseconds"], r["per_mebibyte_milliseconds"], r["maximum_relative_residual"], r["fit_not_linear"])))
points = [p for p in rows(path, "host_cpu_per_submission_point") if p["clock"] == "thread" and p["candidate"] == "every_stage" and p["form"] != "today_single_kernel"]
c = collections.Counter()
for p in points:
    fit = fits[(p["index"], p["form"], p["wait"], p["gap_milliseconds"])]
    per_byte = float(fit["per_mebibyte_milliseconds"]) / 1048576
    pool = int(p["pool_data_units"])
    subs = len(p["submission_units"].split(","))
    extra = per_byte * (MAPPING_ENTRY_BYTES * pool + ALLOCATION_RECORD_BYTES * pool / subs)
    t = float(p["threshold_milliseconds"])
    per_round = ["not_losing" if v + extra < t else "losing" for v in floats(p["host_cpu_per_submission_milliseconds_per_round"])]
    j = per_round[0] if len(set(per_round)) == 1 else "unstable"
    c[(p["index"], p["pool_data_units"], p["judgement"], j)] += 1
    if p["judgement"] == "not_losing" and j != "not_losing":
        c[("flipped_extra_ms", p["index"], p["pool_data_units"], round(extra, 4))] += 1
for k, n in sorted(c.items(), key=str):
    print("name=c_with_the_inputs_of_scan_and_choice key=%s cells=%d" % (",".join(map(str, k)), n))
