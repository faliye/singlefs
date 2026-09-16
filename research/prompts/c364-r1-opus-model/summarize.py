#!/usr/bin/env python3
"""把 results-SELF-FUT-G64.txt 的 C364SCAN 行按 (R 外空洞, R 内索引节点, R 内用户单元) 汇总。复跑：python3 summarize.py > scan-summary.txt"""
import collections

rows = [dict(kv.split("=", 1) for kv in line.split()[1:])
        for line in open("results-SELF-FUT-G64.txt") if line.startswith("C364SCAN")]
groups = collections.defaultdict(list)
for row in rows:
    groups[(int(row["holes"]), int(row["index_nodes_in_R"]), int(row["user_units_in_R"]))].append(row)


def count(items, key, value="True"):
    return sum(1 for item in items if item[key] == value)


print(f"C364SCANSUM rows={len(rows)} enospc={count(rows, 'enospc')}")
print("| R 外空洞对数 | R 内索引节点 | R 内用户单元 | 格数 | 完成那一刻 occ 差 ≥ 1（S甲 在 AC+occ 下判有产出） | 完成那一刻 rec 差 ≥ 1 | "
      "S丙 在 occ 下 (a) | S丙 在 rec 下 (a) | 之后 horizon 里 occ 差曾 ≥ 1 | 对照（不整理）occ 差区间 |")
print("|---|---|---|---|---|---|---|---|---|---|")
for key in sorted(groups):
    items = groups[key]
    control = sorted({(row["control_no_compaction_min"], row["control_no_compaction_max"]) for row in items})
    print(f"| {key[0]} | {key[1]} | {key[2]} | {len(items)} | {count(items, 'Sjia_fires_at_done_occ')} | "
          f"{count(items, 'Sjia_fires_at_done_rec')} | {count(items, 'Sbing_a_at_done_occ')} | "
          f"{count(items, 'Sbing_a_at_done_rec')} | {count(items, 'Sjia_fires_in_quiesce_occ')} | {control} |")
total = len(rows)
print(f"| 合计 | | | {total} | {count(rows, 'Sjia_fires_at_done_occ')} | {count(rows, 'Sjia_fires_at_done_rec')} | "
      f"{count(rows, 'Sbing_a_at_done_occ')} | {count(rows, 'Sbing_a_at_done_rec')} | "
      f"{count(rows, 'Sjia_fires_in_quiesce_occ')} | |")
