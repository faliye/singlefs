#!/usr/bin/env python3
"""Positive control check (own): compare 7603 (plain) and 7623 (control) window by window."""
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from parse_windows import parse
D = "/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/"
plain = parse(D + "task-7603.log")
ctrl = parse(D + "task-7623.log")
key = lambda r: (r["live_data_units"], r["phase"], r["steps_done"])
pm = {key(r): r for r in plain}
print("P phase steps | pc_calls pc_ms/call pc_MiB/call | extra_out_ms/call (ctrl_out - plain_out) | extra/pc | carried ms/MiB ctrl plain | unit MiB")
for r in ctrl:
    p = pm.get(key(r))
    if not p:
        continue
    n = r["publish_user_change_total"][0]
    pcn, pcs, pca = r["positive_control_extra_units_clone"]
    if pcn == 0:
        continue
    out_c = (r["window_seconds"] - r["publish_user_change_total"][1] - r["apparatus_drop_of_the_published"][1])
    out_p = (p["window_seconds"] - p["publish_user_change_total"][1] - p["apparatus_drop_of_the_published"][1])
    extra = (out_c - out_p) * 1000 / pcn
    pc_ms = pcs * 1000 / pcn
    cn, cs, ca = r["carried_unit_clone"]
    pn, ps, pa = p["carried_unit_clone"]
    mib = r["unit_bytes_in_the_version"] / 2**20
    print(f"{r['live_data_units']:5d} {r['phase'][:5]} {r['steps_done']:5d} | {pcn:4d} {pc_ms:8.3f} {pca/pcn/2**20:7.2f} | {extra:8.3f} | {extra/pc_ms if pc_ms else 0:5.2f} | {cs*1000/(ca/2**20):.4f} {ps*1000/(pa/2**20):.4f} | {mib:6.1f} | T plain={p['publish_user_change_total'][1]*1000/p['publish_user_change_total'][0]:.2f} ctrl={r['publish_user_change_total'][1]*1000/n:.2f}")
