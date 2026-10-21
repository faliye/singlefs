#!/usr/bin/env python3
"""按步数汇总 out/load_profile.out：写请求数、最长段、全量两态闭式 log2、甲二下界 log2 的最小 / 中位 / 最大。
用法：python3 -I summarize.py <load_profile.out>"""
import statistics
import sys

rows = []
for line in open(sys.argv[1], encoding="utf-8"):
    if line.startswith("name=load_profile ") and "outcome=panicked" not in line:
        rows.append(dict(field.split("=", 1) for field in line.split() if "=" in field))
by_operations = {}
for row in rows:
    by_operations.setdefault(int(row["operations"]), []).append(row)
for operations in sorted(by_operations):
    group = by_operations[operations]
    def spread(name, cast=float, digits=2):
        values = [cast(row[name]) for row in group]
        middle = statistics.median(values)
        return f"{min(values):.{digits}f}/{middle:.{digits}f}/{max(values):.{digits}f}"
    long_segments = sum(int(row["segments_by_length"].split(",")[4].split(":")[1]) for row in group)
    outcomes = sorted({row["outcome"] for row in group})
    print(f"operations={operations} histories={len(group)} writes_after_mkfs={spread('writes_after_mkfs', int, 0)} "
          f"longest_segment={spread('longest_segment', int, 0)} log2_full_two_state={spread('log2_full_two_state')} "
          f"log2_quick_tier_lower_bound={spread('log2_quick_tier_lower_bound')} segments_ge64={long_segments} outcomes={','.join(outcomes)}")
