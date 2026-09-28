#!/usr/bin/env python3
"""unreadable-at-mount-r2 云端攻方腿：Y1 轨迹汇总。读 runs/<副本>-traj-<部分>.log 的 name=r2traj-summary 行，
按起点（label）对齐几份副本：每格写这一起点在 6 种用户动作下出现过的模式串（W 可写 / R 只读 / X 都不成）与各项首次出现的挂载序号。
用法：summarize_trajectories.py <runs 目录> 副本...
"""
import collections, pathlib, re, sys
runs = pathlib.Path(sys.argv[1]); copies = sys.argv[2:]
parts = ["torn", "single_slot", "c393_single", "c393_double"]
table = collections.defaultdict(dict)
lines_per = collections.Counter()
for copy in copies:
    for part in parts:
        path = runs / f"{copy}-traj-{part}.log"
        if not path.exists():
            continue
        for line in path.read_text().splitlines():
            if line.startswith("name=r2traj "):
                lines_per[(copy, part)] += 1
            if not line.startswith("name=r2traj-summary "):
                continue
            body = line[len("name=r2traj-summary "):]
            label, _, rest = body.partition(" policy=")
            policy, _, fields = rest.partition(" ")
            values = dict(item.split("=", 1) for item in fields.split(" "))
            table[(part, label)].setdefault(copy, []).append((policy, values))
for (copy, part), count in sorted(lines_per.items()):
    print(f"mount_lines copy={copy} part={part} lines={count}")
def cell(entries):
    modes = collections.Counter(v["modes"] for _, v in entries)
    unchanged = all(v["refusals_left_the_disk_unchanged"] == "true" for _, v in entries)
    first_w = sorted({v["first_writable"] for _, v in entries})
    sc = sorted({v["every_system_configuration_slot_verifies_from_mount"] for _, v in entries})
    ab = sorted({v["abandoned_out_of_ring_from_mount"] for _, v in entries})
    text = "modes=" + "/".join(f"{m}x{n}" for m, n in sorted(modes.items()))
    text += f" first_writable={','.join(first_w)} refusals_unchanged={unchanged} sc_all_verify_from={','.join(sc)}"
    if ab != ["n/a"]:
        text += f" abandoned_out_of_ring_from={','.join(ab)}"
    return text
for (part, label) in sorted(table):
    print(f"\n[{part}] {label}")
    for copy in copies:
        entries = table[(part, label)].get(copy)
        print(f"  {copy}: " + (cell(entries) + f" policies={len(entries)}" if entries else "MISSING"))
