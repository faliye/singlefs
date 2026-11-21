#!/usr/bin/env python3
"""读发布路径探针的产物（装置 construction-timing 打的 name=publish_change_probe_row / _phase 与 name=construction_phase 行），
另算一遍每段每张表的分布，与装置自己打的分布行逐格比，再照跑前写死的翻面读数逐条判。

用法：summarize_publish_change_probe.py <产物文件>
退出码：0 两条路算出的分布逐格对上、完整性闸过；1 有一格对不上或完整性闸不过（这一趟作废）；2 用法错或产物里没有探针行。
翻面读数的判定只打行（name=flip_reading … flipped=yes|no），不进退出码。
"""
import sys

# 跑前写死的翻面读数（报告「d 的探针」一节同文）：判在交错写那一段（interleaved_half），另两段只报。
JUDGED_PHASE = "interleaved_half"
REPORTED_PHASES = ("sequential_half", "interleaved_half", "unlinking_the_spacer_files")
COPY_RATIO_FLIP = 0.25          # T1、T4：按 64 条一块要拷的条数占 N 的比，中位不小于它就翻
LEFTOVER_CLONE_FLIP = 0.05      # T5：记录表与空闲图（每盘 DeviceFreeMap 整个）之外剩下的集合（聚簇段集、已回收集、被抛弃根的落点）项数之和，占分配器记录条数的比，最大不小于它就翻
TOLERANCE = 1e-6


def fields_of(line):
    """一行里 key=value 的词；没有等号的词（行首前缀）丢掉。"""
    fields = {}
    for word in line.split():
        if "=" in word:
            key, value = word.split("=", 1)
            fields[key] = value
    return fields


def median_of(values):
    """与装置同一个取法：排好之后第 ⌊n ÷ 2⌋ 个；空列交 0。"""
    if not values:
        return 0.0
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def table_columns(rows, table):
    """一张表每次发布（有上一版的）的 d、N、d ÷ N、按 64 条与 256 条一块的拷贝比。"""
    columns = {"d": [], "n": [], "ratio": [], "copy64": [], "copy256": []}
    for row in rows:
        if row["had_a_previous_version"] != "true":
            continue
        before, after = int(row[f"{table}_before"]), int(row[f"{table}_after"])
        d = int(row[f"{table}_added"]) + int(row[f"{table}_removed"]) + int(row[f"{table}_changed"])
        n = float(max(before, after, 1))
        columns["d"].append(float(d))
        columns["n"].append(n)
        columns["ratio"].append(d / n)
        for block, name in ((64, "copy64"), (256, "copy256")):
            touched = int(row[f"{table}_blocks{block}"])
            columns[name].append((touched * block + -(-n // block)) / n)
    return columns


def main(arguments):
    if len(arguments) != 1:
        print("用法：summarize_publish_change_probe.py <产物文件>", file=sys.stderr)
        return 2
    rows, phase_lines, construction = [], [], {}
    with open(arguments[0], encoding="utf-8") as product:
        for line in product:
            if "name=publish_change_probe_row " in line:
                rows.append(fields_of(line))
            elif "name=publish_change_probe_phase " in line:
                phase_lines.append(fields_of(line))
            elif "name=construction_phase " in line:
                fields = fields_of(line)
                construction[fields["phase"]] = fields
    if not rows:
        print("  ✗ 产物里没有 name=publish_change_probe_row 行：探针没接上或装置没跑到造池", file=sys.stderr)
        return 2
    failures = []
    by_phase = {}
    for row in rows:
        by_phase.setdefault(row["phase"], []).append(row)
    print(f"name=probe_rows rows={len(rows)} phases={','.join(sorted(by_phase))}")
    # 完整性闸：探针数到的、有上一版的发布数，不少于装置那一段自己报的发布数（少了就是有一条发布路径没接上探针）。
    for phase in REPORTED_PHASES:
        counted = sum(1 for row in by_phase.get(phase, []) if row["had_a_previous_version"] == "true")
        reported = int(construction.get(phase, {}).get("publishes", "-1"))
        holds = reported >= 0 and counted >= reported
        print(f"name=integrity phase={phase} counted_with_a_previous_version={counted} apparatus_publishes={reported} holds={'yes' if holds else 'no'}")
        if not holds:
            failures.append(f"{phase} 段探针数到 {counted} 次、装置报 {reported} 次：有发布路径没接上探针，这一趟作废")
    # 两条路逐格比：装置打的分布行与这里另算的。
    for line in phase_lines:
        phase, table = line["phase"], line["table"]
        columns = table_columns(by_phase.get(phase, []), table)
        pairs = (("d_median", median_of(columns["d"])), ("copy64_over_n_median", median_of(columns["copy64"])),
                 ("d_over_n_max", max(columns["ratio"], default=0.0)))
        for key, recomputed in pairs:
            reported = float(line[key])
            if abs(reported - recomputed) > max(TOLERANCE, TOLERANCE * abs(recomputed)):
                failures.append(f"{phase} {table} {key}：装置打 {reported}、另算 {recomputed}")
    return report_and_judge(by_phase, failures)


def spread(values):
    """最小、中位、最大。"""
    return (min(values, default=0.0), median_of(values), max(values, default=0.0))


def report_and_judge(by_phase, failures):
    """每段每张表的分布行、角色与分配器那几样的行，再照翻面读数逐条判。"""
    for phase in REPORTED_PHASES:
        rows = by_phase.get(phase, [])
        for table in ("units", "mapping", "records"):
            columns = table_columns(rows, table)
            d, n, ratio = spread(columns["d"]), spread(columns["n"]), spread(columns["ratio"])
            copy64 = columns["copy64"]
            at_or_above = sum(1 for value in copy64 if value >= COPY_RATIO_FLIP)
            print(f"name=distribution phase={phase} table={table} publishes={len(columns['d'])} "
                  f"d_min={d[0]:g} d_median={d[1]:g} d_max={d[2]:g} n_min={n[0]:g} n_median={n[1]:g} n_max={n[2]:g} "
                  f"d_over_n_min={ratio[0]:.6f} d_over_n_median={ratio[1]:.6f} d_over_n_max={ratio[2]:.6f} "
                  f"copy64_over_n_median={median_of(copy64):.6f} copy64_over_n_max={max(copy64, default=0.0):.6f} "
                  f"publishes_with_copy64_at_or_above_{COPY_RATIO_FLIP}={at_or_above}")
        integer = lambda key: [int(row[key]) for row in rows if row["had_a_previous_version"] == "true"]
        print(f"name=roles phase={phase} rewritten_roles_median={median_of(integer('rewritten_roles')):g} "
              f"carried_under_a_new_role_median={median_of(integer('units_carried_under_a_new_role')):g} "
              f"carried_under_a_new_role_max={max(integer('units_carried_under_a_new_role'), default=0)} "
              f"repeated_roles_max={max(integer('repeated_roles'), default=0)} "
              f"descents_within_a_role_kind_max={max(integer('descents_within_a_role_kind'), default=0)} "
              f"order_flips_among_common_roles_max={max(integer('order_flips_among_common_roles'), default=0)} "
              f"relocated_units_max={max(integer('relocated_units'), default=0)}")
        leftover = [int(row["cluster_segments"]) + int(row["reclaimed_placements"])
                    + int(row["abandoned_root_placements"]) for row in rows]
        records = [int(row["allocator_records"]) for row in rows]
        print(f"name=allocator phase={phase} free_map_slots={rows[-1]['free_map_slots'] if rows else 0} "
              f"free_map_slots_changed_median={median_of(integer('free_map_slots_changed')):g} "
              f"free_map_slots_changed_max={max(integer('free_map_slots_changed'), default=0)} "
              f"free_map_blocks4096_touched_max={max(integer('free_map_blocks4096_touched'), default=0)} "
              f"free_map_segments_changed_max={max(integer('free_map_segments_changed'), default=0)} "
              f"cluster_segments_max={max(integer('cluster_segments'), default=0)} "
              f"reclaimed_placements_max={max(integer('reclaimed_placements'), default=0)} "
              f"segments_with_an_unblocked_slot_pair_max={max(integer('segments_with_an_unblocked_slot_pair'), default=0)} "
              f"leftover_collections_max={max(leftover, default=0)} allocator_records_max={max(records, default=0)}")
    judged = by_phase.get(JUDGED_PHASE, [])
    judge(judged, by_phase)
    if failures:
        for failure in failures:
            print(f"  ✗ {failure}")
        print("    → 怎么办：分布对不上先看装置的 probe_lines_of_the_phase 与这里的 table_columns 取法；完整性闸不过先查哪条带树的发布路径没走 "
              "publish_version_of_trees_relocating_units_after_a_check")
        return 1
    print("  ✓ 两条路的分布逐格对上、完整性闸过")
    return 0


def judge(judged, by_phase):
    """跑前写死的五条翻面读数（T1–T5），各打一行。"""
    def copy64_median(table):
        return median_of(table_columns(judged, table)["copy64"])
    rows_with_previous = [row for row in judged if row["had_a_previous_version"] == "true"]
    t1 = copy64_median("records")
    print(f"name=flip_reading reading=T1_allocation_records copy64_over_n_median={t1:.6f} threshold={COPY_RATIO_FLIP} "
          f"flipped={'yes' if t1 >= COPY_RATIO_FLIP else 'no'}")
    carried = median_of([int(row["units_carried_under_a_new_role"]) for row in rows_with_previous])
    rewritten = median_of([int(row["rewritten_roles"]) for row in rows_with_previous])
    print(f"name=flip_reading reading=T2_units_keyed_by_role carried_under_a_new_role_median={carried:g} "
          f"rewritten_roles_median={rewritten:g} flipped={'yes' if carried > rewritten else 'no'}")
    every_row = [row for rows in by_phase.values() for row in rows]
    repeated = max((int(row["repeated_roles"]) for row in every_row), default=0)
    flips = max((int(row["order_flips_among_common_roles"]) for row in every_row), default=0)
    descents = max((int(row["descents_within_a_role_kind"]) for row in every_row), default=0)
    print(f"name=flip_reading reading=T3_static_order_of_the_roles repeated_roles_max={repeated} "
          f"order_flips_among_common_roles_max={flips} flipped={'yes' if repeated > 0 or flips > 0 else 'no'} "
          f"descents_within_a_role_kind_max={descents} derived_order_within_a_kind_flipped={'yes' if descents > 0 else 'no'}")
    t4 = copy64_median("mapping")
    print(f"name=flip_reading reading=T4_central_mapping_entries copy64_over_n_median={t4:.6f} threshold={COPY_RATIO_FLIP} "
          f"flipped={'yes' if t4 >= COPY_RATIO_FLIP else 'no'}")
    worst = max(((int(row["cluster_segments"]) + int(row["reclaimed_placements"])
                  + int(row["abandoned_root_placements"])) / max(int(row["allocator_records"]), 1)
                 for row in every_row), default=0.0)
    print(f"name=flip_reading reading=T5_allocator_clone_after_records_and_free_map leftover_over_records_max={worst:.6f} "
          f"threshold={LEFTOVER_CLONE_FLIP} flipped={'yes' if worst >= LEFTOVER_CLONE_FLIP else 'no'}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
