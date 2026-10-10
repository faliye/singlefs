#!/usr/bin/env python3
"""把 k1-random-b*.txt 的 OPUS_K1R 行汇成一张表。用法：summarize_random.py <放 k1-random-b*.txt 的目录>。"""
import glob
import re
import sys

rows = []
for path in sorted(glob.glob(f"{sys.argv[1]}/k1-random-b*.txt")):
    for line in open(path, encoding="utf-8"):
        if not line.startswith("OPUS_K1R "):
            continue
        number = lambda name: int(re.search(name + r"=(\d+)", line).group(1))
        knobs = re.search(r"knobs=Knobs \{ tombstone: (\d+), lineage: (\d+), retirement: (\d+), node_entries: (\d+) \}", line).groups()
        ops = dict((name, int(count)) for name, count in re.findall(r'"([a-z-]+)": (\d+)', line.split("ops=")[1].split("}")[0]))
        final = line.split("final=")[1]

        def containers(tree):
            match = re.search(tree + r"=\[(.*?)\]", final)
            return len(re.findall(r"\(", match.group(1))) if match else 0

        rows.append((number("seed"), re.search(r"mode=(\w+)", line).group(1), knobs, ops.get("remount", 0), ops.get("rollback-ok", 0),
                     number("checks"), number("violating"), number("non_rightmost_judged"),
                     containers("tombstone"), containers("lineage"), containers("retirement")))
print(f"合计：{len(rows)} 个种子，判 {sum(r[5] for r in rows)} 次，有违例的 {sum(r[6] for r in rows)} 次，"
      f"新判判到的不是最右的容器（按步累计）{sum(r[7] for r in rows)} 片，重挂 {sum(r[3] for r in rows)} 次，回退 {sum(r[4] for r in rows)} 次")
print("| 种子 | 取法 | 旋钮 墓碑/谱系/退役/节点 | 重挂 | 回退 | 判次 | 有违例 | 新判判到的片（累计） | 末版片数 墓碑/谱系/退役 |")
print("|---|---|---|---|---|---|---|---|---|")
for r in sorted(rows):
    print(f"| {r[0]} | {r[1]} | {'/'.join(r[2])} | {r[3]} | {r[4]} | {r[5]} | {r[6]} | {r[7]} | {r[8]}/{r[9]}/{r[10]} |")
