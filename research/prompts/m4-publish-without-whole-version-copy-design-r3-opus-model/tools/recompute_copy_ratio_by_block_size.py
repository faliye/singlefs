#!/usr/bin/env python3
"""读发布路径探针的产物（name=publish_change_probe_row / _phase），按块长 16、64、256 重算「一次发布按块要拷的条数占全表的比」，
照跑前写死的 T1（分配记录）、T4（中央映射条目）翻面线逐块长判；另报已回收集占分配记录条数之比最大的那一行落在哪一段、
已回收集在相邻两次发布之间缩了几次。

用法：recompute_copy_ratio_by_block_size.py <产物文件>
口径：拷贝比 =（碰到的块数 × 块长 + ⌈N ÷ 块长⌉ 个块指针）÷ N，N 取两版里大的那个（与装置、第二轮汇总脚本同一个式子）。
  块长 64、256 的碰到块数产物逐行有（records_blocks64、records_blocks256 这类）；块长 16 产物里没有，只给界：
  下界取碰到的 16 条块不少于碰到的 64 条块，上界取不多于 4 × 碰到的 64 条块、也不多于改到的条数 d。
  另一种块：分配记录按分配记录树的叶分块（产物行带 records_leaves_touched 那三项时才有，探针副本加的）：
  拷贝比 =（碰到的叶里的记录数 + 有记录的叶数）÷ N，同样判 T1。
退出码：0 块长 64、256 两条路（这里另算的中位与装置打的分布行）逐格对上；1 有一格对不上；2 用法错或产物里没有探针行。
翻面判定只打行（name=block_size_flip … flipped=yes|no|undetermined），不进退出码。
"""
import sys

JUDGED_PHASE = "interleaved_half"
REPORTED_PHASES = ("sequential_half", "interleaved_half", "unlinking_the_spacer_files")
COPY_RATIO_FLIP = 0.25
TOLERANCE = 1e-6
TABLES = ("units", "mapping", "records")
JUDGED_TABLES = (("T1_allocation_records", "records"), ("T4_central_mapping_entries", "mapping"))


def fields_of(line):
    """一行里 key=value 的词。"""
    return dict(word.split("=", 1) for word in line.split() if "=" in word)


def median_of(values):
    """与装置同一个取法：排好之后第 ⌊n ÷ 2⌋ 个；空列交 0。"""
    if not values:
        return 0.0
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def ceiling_division(numerator, denominator):
    return -(-numerator // denominator)


def ratios_of_a_row(row, table):
    """一行一张表：d、N 与块长 16 的下界、上界，块长 64、256 的拷贝比。"""
    before, after = int(row[f"{table}_before"]), int(row[f"{table}_after"])
    n = max(before, after, 1)
    d = int(row[f"{table}_added"]) + int(row[f"{table}_removed"]) + int(row[f"{table}_changed"])
    touched64 = int(row[f"{table}_blocks64"])
    touched256 = int(row[f"{table}_blocks256"])
    touched16_low = touched64
    touched16_high = min(4 * touched64, d)
    leaf = None
    if table == "records" and "records_leaves_touched" in row:
        leaf = (int(row["records_in_the_leaves_touched"]) + int(row["records_leaves"])) / n
    return {
        "leaf": leaf,
        "d": d,
        "n": n,
        "copy16_low": (touched16_low * 16 + ceiling_division(n, 16)) / n,
        "copy16_high": (touched16_high * 16 + ceiling_division(n, 16)) / n,
        "copy64": (touched64 * 64 + ceiling_division(n, 64)) / n,
        "copy256": (touched256 * 256 + ceiling_division(n, 256)) / n,
    }


def main(arguments):
    if len(arguments) != 1:
        print("用法：recompute_copy_ratio_by_block_size.py <产物文件>", file=sys.stderr)
        return 2
    rows, phase_lines = [], []
    with open(arguments[0], encoding="utf-8") as product:
        for line in product:
            if "name=publish_change_probe_row " in line:
                rows.append(fields_of(line))
            elif "name=publish_change_probe_phase " in line:
                phase_lines.append(fields_of(line))
    if not rows:
        print("  ✗ 产物里没有 name=publish_change_probe_row 行", file=sys.stderr)
        return 2
    by_phase = {}
    for row in rows:
        by_phase.setdefault((row["live_data_units"], row["phase"]), []).append(row)
    every_p = sorted({row["live_data_units"] for row in rows}, key=int)
    columns = {}
    for p in every_p:
        for phase in REPORTED_PHASES:
            for table in TABLES:
                per_row = [ratios_of_a_row(row, table) for row in by_phase.get((p, phase), [])
                           if row["had_a_previous_version"] == "true"]
                columns[(p, phase, table)] = {key: [ratios[key] for ratios in per_row]
                                              for key in ("d", "n", "copy16_low", "copy16_high", "copy64", "copy256")}
                columns[(p, phase, table)]["leaf"] = [ratios["leaf"] for ratios in per_row if ratios["leaf"] is not None]
    failures = []
    for line in phase_lines:
        key = (line["live_data_units"], line["phase"], line["table"])
        if key not in columns:
            continue
        for name, column in (("copy64_over_n_median", "copy64"), ("copy256_over_n_median", "copy256")):
            reported, recomputed = float(line[name]), median_of(columns[key][column])
            if abs(reported - recomputed) > max(TOLERANCE, TOLERANCE * abs(recomputed)):
                failures.append(f"P={key[0]} {key[1]} {key[2]} {name}：装置打 {reported}、另算 {recomputed}")
    for (p, phase, table), column in columns.items():
        print(f"name=copy_ratio_by_block_size live_data_units={p} phase={phase} table={table} publishes={len(column['d'])} "
              f"d_median={median_of(column['d']):g} n_median={median_of(column['n']):g} "
              f"copy16_low_median={median_of(column['copy16_low']):.6f} copy16_high_median={median_of(column['copy16_high']):.6f} "
              f"copy64_median={median_of(column['copy64']):.6f} copy256_median={median_of(column['copy256']):.6f} "
              f"copy256_max={max(column['copy256'], default=0.0):.6f} "
              f"leaf_rows={len(column['leaf'])} copy_by_leaf_median={median_of(column['leaf']):.6f} "
              f"copy_by_leaf_max={max(column['leaf'], default=0.0):.6f}")
    for p in every_p:
        for reading, table in JUDGED_TABLES:
            column = columns[(p, JUDGED_PHASE, table)]
            low, high = median_of(column["copy16_low"]), median_of(column["copy16_high"])
            verdict16 = "yes" if low >= COPY_RATIO_FLIP else ("no" if high < COPY_RATIO_FLIP else "undetermined")
            print(f"name=block_size_flip live_data_units={p} reading={reading} block=16 copy_over_n_median_low={low:.6f} "
                  f"copy_over_n_median_high={high:.6f} threshold={COPY_RATIO_FLIP} flipped={verdict16}")
            for block in (64, 256):
                value = median_of(column[f"copy{block}"])
                print(f"name=block_size_flip live_data_units={p} reading={reading} block={block} copy_over_n_median={value:.6f} "
                      f"threshold={COPY_RATIO_FLIP} flipped={'yes' if value >= COPY_RATIO_FLIP else 'no'}")
        column = columns[(p, JUDGED_PHASE, "records")]
        if column["leaf"]:
            value = median_of(column["leaf"])
            print(f"name=block_size_flip live_data_units={p} reading=T1_allocation_records block=allocation_record_tree_leaf "
                  f"copy_over_n_median={value:.6f} threshold={COPY_RATIO_FLIP} flipped={'yes' if value >= COPY_RATIO_FLIP else 'no'}")
        else:
            print(f"name=block_size_flip live_data_units={p} reading=T1_allocation_records block=allocation_record_tree_leaf "
                  f"flipped=not_judged reason=the_product_has_no_leaf_counts")
    report_the_reclaimed_set(rows)
    if failures:
        for failure in failures:
            print(f"  ✗ {failure}")
        print("    → 怎么办：先看装置的 probe_lines_of_the_phase 与这里 ratios_of_a_row 的取法是不是同一个式子")
        return 1
    print("  ✓ 块长 64、256 两条路的中位逐格对上")
    return 0


def report_the_reclaimed_set(rows):
    """（聚簇段集 + 已回收集 + 被抛弃根的落点）÷ 分配器记录条数：每段的最大与它在哪一次发布；已回收集在相邻两次发布之间涨、缩、不变各几次。"""
    worst_of_each_phase = {}
    for row in rows:
        if row["phase"].startswith("compaction_"):
            continue
        leftover = (int(row["cluster_segments"]) + int(row["reclaimed_placements"])
                    + int(row["abandoned_root_placements"]))
        ratio = leftover / max(int(row["allocator_records"]), 1)
        phase = row["phase"]
        if phase not in worst_of_each_phase or ratio > worst_of_each_phase[phase][0]:
            worst_of_each_phase[phase] = (ratio, row)
    for phase, (ratio, row) in worst_of_each_phase.items():
        print(f"name=leftover_over_records_of_a_phase phase={phase} max={ratio:.6f} txg={row['txg']} "
              f"reclaimed_placements={row['reclaimed_placements']} cluster_segments={row['cluster_segments']} "
              f"abandoned_root_placements={row['abandoned_root_placements']} allocator_records={row['allocator_records']}")
    ratio, row = max(worst_of_each_phase.values(), key=lambda pair: pair[0])
    print(f"name=leftover_over_records_overall max={ratio:.6f} phase={row['phase']} txg={row['txg']}")
    steps = {}
    previous = None
    for row in rows:
        if row["phase"].startswith("compaction_"):
            continue
        value = int(row["reclaimed_placements"])
        if previous is not None:
            kind = "grew" if value > previous else ("shrank" if value < previous else "unchanged")
            counts = steps.setdefault(row["phase"], {"grew": 0, "shrank": 0, "unchanged": 0})
            counts[kind] += 1
        previous = value
    for phase, counts in steps.items():
        print(f"name=reclaimed_set_steps phase={phase} grew={counts['grew']} shrank={counts['shrank']} unchanged={counts['unchanged']}")
    shrank = sum(counts["shrank"] for counts in steps.values())
    print(f"name=reclaimed_set_steps_overall shrank={shrank} publishes={len(rows)}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
