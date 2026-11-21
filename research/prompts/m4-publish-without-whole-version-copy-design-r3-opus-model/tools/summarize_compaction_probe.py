#!/usr/bin/env python3
"""读探针副本上 construction-timing 跑出的产物里整理那一段（phase=compaction_cpu_arm_user_changes_<次数>，副本里
probe/compaction-probe-on-the-cpu-arm.diff 加的：造完起步镜像之后在 CPU 臂上不计时地交替跑），把带搬迁的发布（relocated_units > 0）
与同一段里别的发布分开，按 P 报三张表的 d、N、拷贝比，照跑前写死的 C1–C4 判。

用法：summarize_compaction_probe.py <产物文件>
退出码：0 完整性闸过、至少一个 P 量到了带搬迁的发布；1 完整性闸不过或一次带搬迁的发布都没量到（这一趟作废）；2 用法错或产物里没有整理那一段的行。
翻面判定只打行（name=compaction_flip … flipped=yes|no|not_judged），不进退出码。
"""
import sys

PHASE_PREFIX = "compaction_cpu_arm_user_changes_"
TABLES = ("units", "mapping", "records")
COPY_RATIO_FLIP = 0.25        # C1–C3：带搬迁的发布，按 64 条一块要拷的比中位不小于它就翻
GROWTH_SHARE_FLIP = 0.5       # C4：最大 P 对最小 P，d 中位的倍数不小于 N 中位倍数的一半就翻（d 随全版涨）


def fields_of(line):
    return dict(word.split("=", 1) for word in line.split() if "=" in word)


def median_of(values):
    if not values:
        return 0.0
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def ceiling_division(numerator, denominator):
    return -(-numerator // denominator)


def table_of_a_row(row, table):
    before, after = int(row[f"{table}_before"]), int(row[f"{table}_after"])
    n = max(before, after, 1)
    d = int(row[f"{table}_added"]) + int(row[f"{table}_removed"]) + int(row[f"{table}_changed"])
    return {
        "d": d,
        "n": n,
        "copy64": (int(row[f"{table}_blocks64"]) * 64 + ceiling_division(n, 64)) / n,
        "copy256": (int(row[f"{table}_blocks256"]) * 256 + ceiling_division(n, 256)) / n,
    }


def spread_line(rows, table):
    per_row = [table_of_a_row(row, table) for row in rows]
    column = lambda key: [values[key] for values in per_row]
    return {
        "publishes": len(per_row),
        "d_min": min(column("d"), default=0), "d_median": median_of(column("d")), "d_max": max(column("d"), default=0),
        "n_median": median_of(column("n")),
        "d_over_n_median": median_of([values["d"] / values["n"] for values in per_row]),
        "copy64_median": median_of(column("copy64")), "copy64_max": max(column("copy64"), default=0.0),
        "copy256_median": median_of(column("copy256")),
        "copy_by_leaf_median": median_of([(int(row["records_in_the_leaves_touched"]) + int(row["records_leaves"]))
                                          / max(int(row["records_before"]), int(row["records_after"]), 1)
                                          for row in rows if table == "records" and "records_leaves" in row]),
    }


def main(arguments):
    if len(arguments) != 1:
        print("用法：summarize_compaction_probe.py <产物文件>", file=sys.stderr)
        return 2
    rows, runs = [], []
    with open(arguments[0], encoding="utf-8") as product:
        for line in product:
            if "name=publish_change_probe_row " in line and f"phase={PHASE_PREFIX}" in line:
                rows.append(fields_of(line))
            elif "name=compaction_probe " in line:
                runs.append(fields_of(line))
    if not runs:
        print("  ✗ 产物里没有 name=compaction_probe 行：副本没打上 compaction-probe-on-the-cpu-arm.diff，或装置没跑到造池之后", file=sys.stderr)
        return 2
    failures = []
    relocation_by_p = {}
    for run in runs:
        p = int(run["live_data_units"])
        if run.get("ran") != "yes":
            print(f"name=compaction_run live_data_units={p} ran=no geometry_formed={run.get('geometry_formed')}")
            continue
        phase = f"{PHASE_PREFIX}{run['user_changes']}"
        of_the_run = [row for row in rows if row["phase"] == phase and int(row["live_data_units"]) == p]
        counted = int(run["publishes_counted"])
        holds = len(of_the_run) == counted
        relocating = [row for row in of_the_run
                      if row["had_a_previous_version"] == "true" and int(row["relocated_units"]) > 0]
        others = [row for row in of_the_run if int(row["relocated_units"]) == 0]
        print(f"name=compaction_run live_data_units={p} user_changes={run['user_changes']} outcome={run['outcome']} "
              f"batches={run['batches']} units_relocated={run['units_relocated']} rows={len(of_the_run)} "
              f"publishes_counted={counted} integrity={'yes' if holds else 'no'} relocating_publishes={len(relocating)} "
              f"other_publishes={len(others)} relocated_units_median={median_of([int(r['relocated_units']) for r in relocating]):g} "
              f"seconds={run['seconds']}")
        if not holds:
            failures.append(f"P={p} {phase}：产物里 {len(of_the_run)} 行、装置报数到 {counted} 次")
        if run["outcome"] == "ok" and relocating:
            relocation_by_p.setdefault(p, []).extend(relocating)
        for label, chosen in (("relocating", relocating), ("other", others)):
            for table in TABLES:
                spread = spread_line(chosen, table)
                print(f"name=compaction_distribution live_data_units={p} user_changes={run['user_changes']} kind={label} table={table} "
                      + " ".join(f"{key}={value:g}" if isinstance(value, (int, float)) and key in ("publishes", "d_min", "d_max")
                                 else f"{key}={value:.6f}" if isinstance(value, float) else f"{key}={value}"
                                 for key, value in spread.items()))
    if not relocation_by_p:
        failures.append("一个 P 都没量到带搬迁的发布（outcome=ok 且 relocated_units > 0）：整理那一段没走到，这一趟作废")
    judge(relocation_by_p)
    if failures:
        for failure in failures:
            print(f"  ✗ {failure}")
        print("    → 怎么办：完整性闸不过先看装置 probe_the_compaction_publishes_on_the_cpu_arm 取走探针表的时点；没量到整理发布先看 name=compaction_probe 行的 outcome 与 batches")
        return 1
    print("  ✓ 完整性闸过、量到了带搬迁的发布")
    return 0


def judge(relocation_by_p):
    """C1–C3 判在量到的最大 P 上；C4 要至少两个 P。"""
    if not relocation_by_p:
        print("name=compaction_flip reading=all flipped=not_judged reason=no_relocating_publishes")
        return
    largest, smallest = max(relocation_by_p), min(relocation_by_p)
    for reading, table in (("C1_allocation_records", "records"), ("C2_central_mapping_entries", "mapping"), ("C3_units", "units")):
        value = spread_line(relocation_by_p[largest], table)["copy64_median"]
        print(f"name=compaction_flip reading={reading} live_data_units={largest} copy64_over_n_median={value:.6f} "
              f"threshold={COPY_RATIO_FLIP} flipped={'yes' if value >= COPY_RATIO_FLIP else 'no'}")
    value = spread_line(relocation_by_p[largest], "records")["copy_by_leaf_median"]
    print(f"name=compaction_flip reading=C1_allocation_records_by_leaf live_data_units={largest} copy_over_n_median={value:.6f} "
          f"threshold={COPY_RATIO_FLIP} flipped={'yes' if value >= COPY_RATIO_FLIP else 'no'}")
    for table in TABLES:
        if largest == smallest:
            print(f"name=compaction_flip reading=C4_d_grows_with_n table={table} flipped=not_judged reason=one_p_only")
            continue
        small, large = spread_line(relocation_by_p[smallest], table), spread_line(relocation_by_p[largest], table)
        growth_d = large["d_median"] / max(small["d_median"], 1)
        growth_n = large["n_median"] / max(small["n_median"], 1)
        print(f"name=compaction_flip reading=C4_d_grows_with_n table={table} smallest_p={smallest} largest_p={largest} "
              f"d_growth={growth_d:.6f} n_growth={growth_n:.6f} share={GROWTH_SHARE_FLIP} "
              f"flipped={'yes' if growth_d >= GROWTH_SHARE_FLIP * growth_n else 'no'}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
