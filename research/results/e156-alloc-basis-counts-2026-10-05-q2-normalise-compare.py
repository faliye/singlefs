#!/usr/bin/env python3
# E156 第 5 次重跑登记第六节 Q2a、Q2b：规范化两份产物、逐行比，不同的行再逐字段列。
# 用法：q2.py <第 4 次产物> <新产物>
# 「第一项」按行首 `E7RESULT ` 标签之后的第一个空白分隔项算。
import sys

VERDICT_NAMES = {
    "verdict", "failure_clauses", "geometry_sensitivity", "q1c", "q1d", "q1c_front_threshold_moved",
    "q1_cell_checks", "pc_judge", "pc_c2", "pc_over_withheld", "pc_hole_counters_zeroed", "pq1", "pq2",
    "stage_two_not_run", "q1_cell_not_run", "q1_cell_truncated", "u11_forward_rollback",
    "u13a_rollback_to_oldest_candidate",
}
TAG = "E7RESULT "


def items(line):
    assert line.startswith(TAG), line[:80]
    return line[len(TAG):].split(" ")


def normalise(path, drop_rows, drop_item_prefix):
    kept = []
    dropped_rows = {}
    dropped_items = 0
    for line in open(path, encoding="utf-8").read().split("\n"):
        if line == "":
            continue
        parts = items(line)
        first = parts[0]
        why = drop_rows(parts)
        if why:
            dropped_rows[why] = dropped_rows.get(why, 0) + 1
            continue
        if first == "name=verdict" and "stage=second" in parts[1:2]:
            filtered = [part for part in parts if not part.startswith(drop_item_prefix)]
            dropped_items += len(parts) - len(filtered)
            parts = filtered
        kept.append(parts)
    return kept, dropped_rows, dropped_items


def old_rows(parts):
    if "family=PC-allocator" in parts:
        return "family=PC-allocator"
    if parts[0] == "name=pc_allocator":
        return "name=pc_allocator"
    return None


def new_rows(parts):
    if parts[0] in ("name=pc_pinned_floor", "name=pc_pinned_floor_summary"):
        return parts[0]
    return None


old_path, new_path = sys.argv[1], sys.argv[2]
old, old_dropped, old_items = normalise(old_path, old_rows, "v2_pc_allocator_seen=")
new, new_dropped, new_items = normalise(new_path, new_rows, "v2_pc_pinned_floor_seen=")
print(f"old_dropped_rows={old_dropped} old_dropped_items={old_items}")
print(f"new_dropped_rows={new_dropped} new_dropped_items={new_items}")
equal = sum(1 for a, b in zip(old, new) if a == b)
unequal = []
if len(old) == len(new):
    unequal = [(index, a, b) for index, (a, b) in enumerate(zip(old, new)) if a != b]
print(f"lines_old={len(old)} lines_new={len(new)} equal={equal} unequal={len(unequal)}")
for index, a, b in unequal:
    name = a[0][len("name="):] if a[0].startswith("name=") else a[0]
    kind = "判决行" if name in VERDICT_NAMES else "计数行"
    print(f"--- 第 {index + 1} 行（规范化后行号）{kind} name={name}")
    old_map = {part.split("=", 1)[0]: part for part in a if "=" in part}
    new_map = {part.split("=", 1)[0]: part for part in b if "=" in part}
    for key in list(old_map) + [key for key in new_map if key not in old_map]:
        if old_map.get(key) != new_map.get(key):
            print(f"    键 {key}: 旧 {old_map.get(key)!r} 新 {new_map.get(key)!r}")
