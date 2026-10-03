#!/usr/bin/env python3
"""把 B18TRACE 行按（第一次崩溃的段号、挂载写表的切段）分组，看同一组里写行那一段的二次崩溃是不是同一个。"""
import re, sys, collections
pattern = re.compile(r"first_seg=(\d+) first_bits=([01]*) first_version=(.*?) mount_writes=(\d+) mount_segments=([\d+]*) row_publish_writes=(\d+) second_seg=(\d+) second_kinds=(\[.*?\]) second_bits=([01]*) phase=(\w+) second_version=(.*?) remount_refused=(\d+) c670_judged=(\d+) c670_illegal=(\d+)")
phase_wanted = sys.argv[2] if len(sys.argv) > 2 else "row_publish"
groups = collections.defaultdict(list)
total = 0
for line in open(sys.argv[1]):
    m = pattern.search(line)
    if not m or m.group(10) != phase_wanted:
        continue
    total += 1
    fs, fb, fv, mw, segs, n, s2, kinds, sb, ph, sv, ref, judged, ill = m.groups()
    short = {"unit_write": "U", "journal_record": "J", "root_record_fua": "R", "system_configuration_slot": "S", "zero_fill": "Z"}
    k = "".join(short.get(x.strip().strip('"'), "?") for x in kinds.strip("[]").split(",") if x.strip())
    groups[(int(fs), segs, n)].append((fb, int(s2), k, sb, int(ref), int(judged), int(ill)))
print(f"phase={phase_wanted} second_crashes={total} groups={len(groups)}")
for (fs, segs, n), items in sorted(groups.items()):
    distinct = sorted({(s2, k, sb) for _, s2, k, sb, *_ in items})
    ill = sum(i[-1] for i in items)
    ref = sum(i[4] for i in items)
    print(f"first_seg={fs} mount_segments={segs} n={n} first_states={len(items)} distinct_second_points={len(distinct)} {distinct} refused={ref} c670_illegal={ill}")
