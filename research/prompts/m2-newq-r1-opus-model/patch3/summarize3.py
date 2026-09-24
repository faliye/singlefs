#!/usr/bin/env python3
"""第三、四版扫描的汇总：按 (故障, 臂, 文件前缀) 数各类结局。列名见 print 那一行。"""
import csv, sys, collections, os
agg = collections.defaultdict(collections.Counter)
for path in sys.argv[1:]:
    tail = os.path.basename(path).split('-')[0]
    with open(path) as f:
        for r in csv.DictReader(f, delimiter="\t"):
            c = agg[(r["fault"], r["arm"], tail)]
            c["runs"] += 1
            steps = r["steps"]
            if "PANIC" in steps: c["panic"] += 1
            if r["final_mount"].startswith("mount-err"): c["final_mount_err"] += 1
            if "mount-err" in steps: c["mount_err_in_steps"] += 1
            if "BlockDevice" in steps: c["publish_write_err"] += 1
            if "ReleaseCheckReadFailed" in steps: c["publish_failed_readerr"] += 1
            if "ReleaseTarget" in steps: c["release_target_err"] += 1
            if "read-only" in steps: c["read_only"] += 1
            if r["readback"] != "same": c["readback_not_same"] += 1
            if r["violated"] != "-": c["checker_red"] += 1
            if "I-3.1" in r["violated"]: c["I-3.1"] += 1
            if r["reuse_steps"] != "[]": c["foreign_unit_on_target"] += 1
            if int(r["isolated_by_check"]) > 0: c["isolated_by_check>0"] += 1
            if any(x != 0 for x in eval(r["free_minus_allocatable"])): c["free!=allocatable"] += 1
            if int(r["rebuild_mapping_mismatch"]) > 0: c["rebuild_saw_mapping_mismatch"] += 1
            if int(r["writes_refused"]) > 0: c["writes_refused>0"] += 1
            if "record-kept-allocated" in r["check_events"]: c["kept_allocated"] += 1
cols = ["runs","panic","final_mount_err","mount_err_in_steps","publish_write_err","publish_failed_readerr","release_target_err","read_only","readback_not_same","checker_red","I-3.1","foreign_unit_on_target","isolated_by_check>0","free!=allocatable","rebuild_saw_mapping_mismatch","writes_refused>0","kept_allocated"]
print("fault\tarm\tset\t" + "\t".join(cols))
for k in sorted(agg):
    print(f"{k[0]}\t{k[1]}\t{k[2]}\t" + "\t".join(str(agg[k][c]) for c in cols))
