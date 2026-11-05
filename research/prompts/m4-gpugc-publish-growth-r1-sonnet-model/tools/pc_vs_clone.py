#!/usr/bin/env python3
"""Control cell (extra Vec<PublishedUnit> clone + drop outside the core) vs clone_into_versions_published (whole TransactionOutput clone inside the core), same run 7623."""
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from parse_windows import parse
D = "/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/"
print("P phase steps | unitMiB | pc ms/MiB | clone_into_versions ms/MiB | ratio clone/pc | faults(pages/MiB) in window")
prev = 0
for r in parse(D + "task-7623.log"):
    n = r["publish_user_change_total"][0]
    pcn, pcs, pca = r["positive_control_extra_units_clone"]
    cn, cs, ca = r["clone_into_versions_published"]
    mib = r["unit_bytes_in_the_version"] / 2**20
    df = r["minor_faults_total"] - prev; prev = r["minor_faults_total"]
    if pcn == 0:
        continue
    pc = pcs * 1000 / pcn / (pca / pcn / 2**20)
    cl = cs * 1000 / cn / mib
    print(f"{r['live_data_units']:5d} {r['phase'][:5]} {r['steps_done']:5d} | {mib:6.1f} | {pc:.4f} | {cl:.4f} | {cl/pc:5.2f} | {df/(pcn+n)/ (mib):6.1f}")
