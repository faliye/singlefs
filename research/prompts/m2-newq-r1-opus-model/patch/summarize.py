#!/usr/bin/env python3
"""把扫描的 tsv 按 (故障, 臂) 汇总成各类结局的计数。"""
import csv, sys, collections
rows = []
for path in sys.argv[1:]:
    with open(path) as f:
        rows += list(csv.DictReader(f, delimiter="\t"))
key = lambda r: (r["fault"], r["arm"])
agg = collections.defaultdict(collections.Counter)
for r in rows:
    c = agg[key(r)]
    c["runs"] += 1
    steps = r["steps"]
    if "PANIC" in steps: c["panic"] += 1
    if "mount-err" in steps or r["final_mount"].startswith("mount-err"): c["mount_err_any"] += 1
    if r["final_mount"].startswith("mount-err"): c["final_mount_err"] += 1
    if "PlacementRefused" in steps: c["placement_refused"] += 1
    if "ReleaseCheckReadFailed" in steps: c["publish_failed_readerr"] += 1
    if "read-only" in steps: c["read_only"] += 1
    if "refused(not-writable)" in steps: c["later_steps_refused"] += 1
    if "err:" in steps: c["any_publish_err"] += 1
    if r["readback"] != "same": c["readback_not_same"] += 1
    if r["violated"] != "-": c["checker_red"] += 1
    if int(r["isolated_by_check"]) > 0: c["isolated_by_check>0"] += 1
    fma = eval(r["free_minus_allocatable"])
    if any(x != 0 for x in fma): c["free!=allocatable"] += 1
    iso = eval(r["isolated_per_device"])
    if len(set(iso)) > 1: c["isolation_asymmetric"] += 1
    if int(r["switches"]) > 0: c["switched"] += 1
cols = ["runs","panic","mount_err_any","final_mount_err","placement_refused","publish_failed_readerr","read_only","later_steps_refused","any_publish_err","readback_not_same","checker_red","isolated_by_check>0","free!=allocatable","isolation_asymmetric","switched"]
print("fault\tarm\t" + "\t".join(cols))
for k in sorted(agg):
    print(f"{k[0]}\t{k[1]}\t" + "\t".join(str(agg[k][c]) for c in cols))
