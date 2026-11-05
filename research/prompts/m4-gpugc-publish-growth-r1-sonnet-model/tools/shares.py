#!/usr/bin/env python3
"""Shares and prices from probe windows (own tool). Usage: shares.py LOG"""
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from parse_windows import parse

def main(path):
    rows = parse(path)
    print("P phase steps | T_ms win_ms | deep3_in_T(carried+clone+replace) share_of_T | deep4(+drop) share_of_window | clone_cell/carried_cell | unprobed: A_res O_res J-S  (share of T) | carried ms/MiB(own bytes) | pages_refaulted/unitMiB")
    prev_faults = 0
    for r in rows:
        n = r["publish_user_change_total"][0]
        c = lambda k: r[k][1] * 1000 / n
        T = c("publish_user_change_total")
        win = r["window_seconds"] * 1000 / n
        carried, clone, replace, drop = c("carried_unit_clone"), c("clone_into_versions_published"), c("replace_current_dropping_old"), c("apparatus_drop_of_the_published")
        O, J, A, S = c("publish_the_change_once"), c("judge_before_any_read_or_write"), c("publish_admitted_total"), c("settle_allocation_record_tree")
        O_res = O - (c("allocator_clone_before_the_publish") + J + A + c("persist_the_publish"))
        A_res = A - (c("allocation_records_sort_and_assert") + c("build_allocation_record_tree") + carried)
        JS = J - S
        cn, cs, ca = r["carried_unit_clone"]
        price = (cs * 1000) / (ca / 2**20) if ca else 0
        faults = r["minor_faults_total"]
        dfault = (faults - prev_faults) / n
        prev_faults = faults
        mib = r["unit_bytes_in_the_version"] / 2**20
        print(f"{r['live_data_units']:5d} {r['phase'][:5]} {r['steps_done']:5d} | {T:7.2f} {win:7.2f} | {carried+clone+replace:6.2f} {100*(carried+clone+replace)/T:5.1f}% | {carried+clone+replace+drop:6.2f} {100*(carried+clone+replace+drop)/win:5.1f}% | {clone/carried if carried else 0:5.2f} | {A_res:5.2f} {O_res:5.2f} {JS:5.2f} ({100*(A_res+O_res+JS)/T:4.1f}%) | {price:.4f} | {dfault*4096/2**20/mib:5.2f}")

if __name__ == "__main__":
    main(sys.argv[1])
