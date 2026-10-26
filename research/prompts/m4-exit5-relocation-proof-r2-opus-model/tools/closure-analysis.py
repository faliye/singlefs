#!/usr/bin/env python3
"""闭包清单（lib_function_units.py closure 的输出）上数 Y3、Y4 用的几样，一行一个 name= 行。"""
import collections, re, sys
work = sys.argv[1]
def fns(name):
    rows = []
    for line in open(f"{work}/closure-{name}.txt"):
        kind, full, place, _ = line.rstrip("\n").split("\t")
        if kind == "fn":
            rows.append((full, place))
    return rows
def units(name):
    return {tuple(l.rstrip("\n").split("\t")[:2]) for l in open(f"{work}/closure-{name}.txt")}
publish, rollback, judge, recover = (fns(n) for n in ("publish", "roll_back_by_a_forward_publish", "judge_crash_image", "recover"))
both = {full: place for full, place in publish + rollback}
core_both = {f: p for f, p in both.items() if f.startswith("singlefs_core::")}
def lines_of(place):
    a, b = place.rsplit(":", 1)[1].split("-")
    return int(b) - int(a) + 1
print(f"name=closure-union entries=publish_relocation_batch+roll_back_by_a_forward_publish fn={len(both)} fn_lines={sum(lines_of(p) for p in both.values())} core_fn={len(core_both)}")
in_impl = [f for f in core_both if "<impl" in f]
print(f"name=closure-union-in-impl core_fn_inside_impl_or_trait={len(in_impl)} of={len(core_both)}")
last = collections.Counter(f.rsplit("::", 1)[1] for f in both)
dup_names = {n: c for n, c in last.items() if c > 1}
print(f"name=closure-union-name-collisions distinct_names_shared_by_two_or_more_fns={len(dup_names)} fns_carrying_a_shared_name={sum(dup_names.values())} top={sorted(dup_names.items(), key=lambda x: -x[1])[:6]}")
judge_set = {f for f, _ in judge}
recover_set = {f for f, _ in recover}
inter = [f for f in core_both if f in judge_set]
print(f"name=closure-overlap core_fn_in_union_and_in_judge_crash_image={len(inter)} of={len(core_both)} judge_fn_total={len(judge_set)} judge_core_fn={sum(1 for f in judge_set if f.startswith('singlefs_core::'))}")
rec_outside = [f for f in recover_set if f.startswith("singlefs_core::") and f not in both]
print(f"name=closure-recover-outside-union core_fn_in_recover_not_in_union={len(rec_outside)} recover_core_fn={sum(1 for f in recover_set if f.startswith('singlefs_core::'))}")
switches = ["ReuseWindow", "SpaceAdmission", "MountSpaceAdmission", "DeletionSwitches", "CompactionBatchSize", "CompactionRunForcing", "ShadowLedger", "JournalPolicy", "InstanceTableSweepConfiguration", "UnlinkBatchSize"]
u = units("publish") | units("roll_back_by_a_forward_publish")
present = sorted({s for s in switches for kind, full in u if kind in ("enum", "struct") and full.rsplit("::", 1)[1] == s})
print(f"name=closure-union-test-only-switch-types present={present}")
for f in ("singlefs_core::allocator::<impl PoolAllocator>::set_reuse_window", "singlefs_core::allocator::<impl PoolAllocator>::release_leaving_the_record_allocated_on", "singlefs_core::allocator::<impl PoolAllocator>::reclaim_released_records_up_to"):
    print(f"name=membership fn={f.rsplit('::',1)[1]} in_publish={int(any(x==f for x,_ in publish))} in_rollback={int(any(x==f for x,_ in rollback))} in_judge={int(f in judge_set)}")
out = [(f, p) for f, p in recover if f.startswith("singlefs_core::") and f not in both]
print(f"name=closure-recover-outside-union-lines fns={len(out)} fn_lines={sum(lines_of(p) for _, p in out)} includes_mapping_read={int(any('locations_of_key' in f for f, _ in out))} includes_read_mapped_tree_root={int(any(f.endswith('::read_mapped_tree_root') for f, _ in out))} includes_replay_journal={int(any(f.endswith('::replay_journal') for f, _ in out))}")
print(f"name=closure-sizes publish_fn={len(publish)} rollback_fn={len(rollback)} judge_fn={len(judge)} recover_fn={len(recover)}")
