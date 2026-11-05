#!/usr/bin/env python3
"""Hierarchical decomposition of probe windows (own tool). Usage: decompose.py LOG [phase-filter]"""
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from parse_windows import parse

def ms(row, name):
    n, s, a = row[name]
    return s * 1000.0

def main(path, phase=None):
    rows = parse(path)
    for r in rows:
        if phase and phase not in r["phase"]:
            continue
        n = r["publish_user_change_total"][0]
        if n == 0:
            continue
        T = ms(r, "publish_user_change_total") / n
        def c(name):
            return ms(r, name) / n
        O = c("publish_the_change_once")
        J = c("judge_before_any_read_or_write")
        A = c("publish_admitted_total")
        S = c("settle_allocation_record_tree")
        # level 1: T children
        l1 = c("checks_before_the_publish") + O + c("clone_into_versions_published") + c("relocate_kept_for_snapshots") + c("replace_current_dropping_old") + c("discard_after_the_change")
        l2 = c("allocator_clone_before_the_publish") + J + A + c("persist_the_publish")
        l3a = c("settle_allocator_clone") + c("settle_plan_resolve") + c("settle_placements_released")  # inside S? 
        l3b = c("allocation_records_sort_and_assert") + c("build_allocation_record_tree") + c("carried_unit_clone")
        print(f"{r['phase'][:6]} steps={r['steps_done']:5d} n={n:3d} U={r['units_in_the_version']:5d} MiB={r['unit_bytes_in_the_version']/2**20:6.1f} R={r['allocation_records_in_the_version']:6d} "
              f"T={T:7.2f} | T-l1={T-l1:6.2f}({(T-l1)/T*100:4.1f}%) O-l2={O-l2:6.2f}({(O-l2)/T*100:4.1f}%) J-S={J-S:5.2f} S-l3a={S-l3a:5.2f}({(S-l3a)/T*100:4.1f}%) A-l3b={A-l3b:6.2f}({(A-l3b)/T*100:4.1f}%) "
              f"win={r['window_seconds']*1000/n:7.2f} out={(r['window_seconds']*1000/n)-T:6.2f}")

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
