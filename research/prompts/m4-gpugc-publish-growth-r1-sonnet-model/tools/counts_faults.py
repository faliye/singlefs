#!/usr/bin/env python3
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from parse_windows import parse
D = "/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/"
rows = parse(D + sys.argv[1])
prev_f = {}
print("P phase steps | per user change: rec_leaf rounds make_room release_find record_for first_idx | single-call us: rec_leaf make_room release_find record_for | faults/user_change | deep ms/MiB | tick sys/(usr+sys) in window | carried count/user_change, us/call | settle_allocator_clone us/call | first_entry_index_build us/call | device_read count, us/call | device_write count, us/call")
pf = 0; pu = 0; ps_ = 0
for r in rows:
    n = r["publish_user_change_total"][0]
    def cnt(k): return r[k][0] / n
    def us(k):
        c, s, a = r[k]
        return s * 1e6 / c if c else 0
    faults = r["minor_faults_total"]
    uticks = r["user_ticks_total"]; sticks = r["system_ticks_total"]
    # first window of a phase has the previous window as baseline only inside same phase/P sequence
    df = faults - pf; du = uticks - pu; ds = sticks - ps_
    pf, pu, ps_ = faults, uticks, sticks
    mib = r["unit_bytes_in_the_version"] / 2**20
    deep = (r["carried_unit_clone"][1] + r["clone_into_versions_published"][1] + r["replace_current_dropping_old"][1]) * 1000 / n
    print(f"{r['live_data_units']:5d} {r['phase'][:5]} {r['steps_done']:5d} | {cnt('records_of_each_leaf'):5.1f} {cnt('settle_allocator_clone'):4.2f} {cnt('allocator_make_room_for_record'):6.1f} {cnt('allocator_release_find'):5.1f} {cnt('allocator_record_for'):5.1f} {cnt('first_entry_index_build'):5.1f} | {us('records_of_each_leaf'):7.1f} {us('allocator_make_room_for_record'):6.2f} {us('allocator_release_find'):6.2f} {us('allocator_record_for'):6.2f} | {df/n:8.0f} | {deep/mib:.4f} | {ds/(du+ds) if du+ds else 0:.2f}  R={r['allocation_records_in_the_version']} MiB={mib:.1f} | {cnt('carried_unit_clone'):7.1f} {us('carried_unit_clone'):5.2f} | {us('settle_allocator_clone'):7.2f} | {us('first_entry_index_build'):6.2f} | {cnt('device_read_at'):6.1f} {us('device_read_at'):5.2f} | {cnt('device_write_at'):6.1f} {us('device_write_at'):5.2f}")
