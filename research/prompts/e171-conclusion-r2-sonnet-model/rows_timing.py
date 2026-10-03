#!/usr/bin/env python3
"""只读：从 rows-timing 产物里两行 table_wall_ns 算整表、每行与 39 行、14 行的挂钟。用法：rows_timing.py <rows-timing.out>"""
import re, sys
walls = []
for line in open(sys.argv[1], encoding="utf-8"):
    if line.startswith("E7TIMING name=table source=table-driver.out"):
        rows = int(re.search(r" rows=(\d+)", line).group(1)); wall = int(re.search(r"table_wall_ns=(\d+)", line).group(1)) / 1e9
        walls.append((rows, wall))
total = sum(w for _, w in walls); n = sum(r for r, _ in walls)
for r, w in walls:
    print(f"rows={r} wall={w:.1f}s per_row={w / r:.2f}s")
mean = total / n
print(f"total_rows={n} total_wall={total:.1f}s ({total / 3600:.2f} h) mean_per_row={mean:.2f}s 39_rows={39 * mean:.0f}s ({39 * mean / 60:.1f} min) 14_rows={14 * mean:.0f}s ({14 * mean / 60:.1f} min)")
