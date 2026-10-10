#!/usr/bin/env python3
"""把 k2-random-b*.txt 的 OPUS_K2R 行汇成一张表。用法：summarize_random.py <放 k2-random-b*.txt 的目录>。"""
import glob
import re
import sys

rows = []
for path in sorted(glob.glob(f"{sys.argv[1]}/k2-random-b*.txt")):
    for line in open(path, encoding="utf-8"):
        if not line.startswith("OPUS_K2R "):
            continue
        seed = int(re.search(r"seed=(\d+)", line).group(1))
        knobs = re.search(r"knobs=Knobs \{ tombstone: (\d+), lineage: (\d+), retirement: (\d+), node_entries: (\d+) \}", line).groups()
        ops = dict((name, int(count)) for name, count in re.findall(r'"([a-z-]+)": (\d+)', line.split("ops=")[1].split("}")[0]))
        checks = int(re.search(r"checks=(\d+)", line).group(1))
        violating = int(re.search(r"violating=(\d+)", line).group(1))
        held_22 = re.search(r"held_I-9.22=(\d+)", line).group(1)
        held_37 = re.search(r"held_I-9.37=(\d+)", line).group(1)
        final = line.split("final=")[1]

        def containers(tree):
            match = re.search(tree + r"=\[(.*?)\]", final)
            return len(re.findall(r"\(", match.group(1))) if match else 0

        rows.append((seed, knobs, ops.get("remount", 0), ops.get("rollback-ok", 0), checks, violating, held_22, held_37,
                     containers("tombstone"), containers("lineage"), containers("retirement")))
print(f"合计：{len(rows)} 个种子，判 {sum(r[4] for r in rows)} 次，有违例的 {sum(r[5] for r in rows)} 次，"
      f"重挂 {sum(r[2] for r in rows)} 次，回退 {sum(r[3] for r in rows)} 次")
print("| 种子 | 旋钮 墓碑/谱系/退役/节点 | 重挂 | 回退 | 判次 | 有违例 | I-9.22 成立 | I-9.37 成立 | 末版片数 墓碑/谱系/退役 |")
print("|---|---|---|---|---|---|---|---|---|")
for r in sorted(rows):
    print(f"| {r[0]} | {'/'.join(r[1])} | {r[2]} | {r[3]} | {r[4]} | {r[5]} | {r[6]} | {r[7]} | {r[8]}/{r[9]}/{r[10]} |")
