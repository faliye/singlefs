#!/usr/bin/env python3
"""ring_patch_crash_states.py 的按类列表：每个变体的崩溃状态按 (下次挂载择到的根 chosen、施加记录之后的 effective、下次挂载的回收门槛、
候选引用已被盖的单元、环里有内部洞) 分类，并列出每类状态数。落点取法取 low（high 在总表里逐个变体同数）。"""
import sys
from collections import Counter

import ring_patch_crash_states as m

h = 3 * m.S + 5 if len(sys.argv) < 2 or sys.argv[1] == "back" else 5
print(f"S={m.S} h={h} pos(h)={m.pos(h)}  列: (chosen, effective, floor_next, red, interior_hole) -> 状态数")
for variant in ("none", "P1", "P2-late", "P2-eager", "P3", "P1-wipe"):
    disk, release = m.run_history(h, "low")
    segments = m.mount_plan(variant, h, disk, release, "low")
    classes = Counter()
    for image in m.crash_states(disk, segments):
        chosen, effective, lo, red, hole = m.judge(image)
        classes[(chosen, effective, lo, red, hole)] += 1
    print(f"-- {variant}")
    for key, count in sorted(classes.items()):
        print(f"   {key} -> {count}")
