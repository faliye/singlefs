#!/usr/bin/env python3
"""e203-b8-criterion-r2 攻方腿的读数模型：只读 research/results/ 下的留存产物，不起项目代码、不编译。

用法：python3 -I b8_r2_readings.py --repo <仓根>
每一节打 name=... 读数行与 name=check 行（ok=true / ok=false）。
退出码：0 全部 check 过；1 有 check 没过；2 用法错或产物读不到。
"""
import hashlib
import math
import os
import sys

FAILED = []
TRAINING_POINT = "files_100_units_100"
TRAINING_SCRIPTS = ("walk_forward", "walk_backward", "first_repeated", "create_then_first")
BEFORE_ADMISSION_SEGMENTS = ("caller", "before_landing", "between")


def say(line):
    print(line, flush=True)


def check(label, ok, detail):
    say(f"name=check label={label} ok={'true' if ok else 'false'} {detail}")
    if not ok:
        FAILED.append(label)


def fields(line):
    out = {}
    for word in line.split():
        if "=" in word:
            key, value = word.split("=", 1)
            out[key] = value
    return out


def strip_prefixes(raw):
    words = raw.rstrip("\n").split(" ")
    while words and words[0].endswith("RESULT"):
        words = words[1:]
    return " ".join(words)


def lines_of(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            yield strip_prefixes(raw)


def sha16(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()[:16]


def growth(y_low, y_high, x_low, x_high):
    return math.log(y_high / y_low) / math.log(x_high / x_low)


def ends_of(text):
    points = []
    for part in text.split(","):
        steps, units, x = part.split("/")
        points.append((int(steps), int(units), int(x)))
    return points


def first_line(path, prefix):
    for line in lines_of(path):
        if line.startswith(prefix):
            return line
    raise LookupError(f"{path} 里没有以 {prefix!r} 起头的行")


# ---- 第一节：K100 同一次挂载 later_peak 那一格两端量窗不对称 ----
def window_asymmetry(results):
    first = os.path.join(results, "e207-strict-scan-steps-report-first-2026-11-13-r3.out")
    repeat = os.path.join(results, "e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out")
    cell = first_line(first, "name=cell segment=new set=K100 state=same_mount tier=one_user_unit "
                             "pick=later_peak filter=steady arm=tested ")
    low, high = ends_of(fields(cell)["ends"])
    recorded = growth(low[0], high[0], low[2], high[2])
    series = first_line(repeat, "name=series_cell state=same_mount tier=one_user_unit pick=later_peak "
                                "filter=steady window=common_window arm=tested primary=yes ")
    points = ends_of(fields(series)["ends"])
    low_in_series = [point for point in points if point[2] == low[2]]
    common_high = points[-1]
    common = growth(low_in_series[0][0], common_high[0], low_in_series[0][2], common_high[2])
    say(f"name=window_asymmetry recorded_low={low[0]}/{low[2]} recorded_high={high[0]}/{high[2]} "
        f"alpha_recorded={recorded:.3f} common_high={common_high[0]}/{common_high[2]} "
        f"alpha_common_two_points={common:.3f}")
    check("recorded_cell_alpha", round(recorded, 3) == 0.502, f"alpha={recorded:.4f}")
    check("common_window_alpha_between", 0.25 < common < 0.5, f"alpha={common:.4f}")
    say(f"name=same_point_peak_rise steps={common_high[0]}->{high[0]} records={common_high[2]}->{high[2]} "
        f"steps_ratio={high[0] / common_high[0]:.3f} records_ratio={high[2] / common_high[2]:.3f}")
    for point, expected in (("files_100_units_100", 80), ("files_1000_units_100", 401)):
        path = os.path.join(results, f"e207-strict-scan-steps-{point}-head-2026-11-13.out")
        prefix = f"name=declared item={point}__head phase=window state=same_mount "
        count = sum(1 for line in lines_of(path) if line.startswith(prefix))
        say(f"name=window_operations point={point} state=same_mount operations={count}")
        check(f"window_operations_{point}", count == expected, f"operations={count} expected={expected}")


# ---- 第二节：K1000L 的 uK 档上 C1 红不红（推的：被测臂各格的 α 照第一段 K1000 外推到一百倍跨度）----
def control_extrapolation(results):
    first = os.path.join(results, "e207-strict-scan-steps-report-first-2026-11-13-r3.out")
    middle = os.path.join(results, "e207-strict-scan-steps-files_100_units_1000_unit_area_2_pow_21-head-2026-11-13.out")
    state = first_line(middle, "name=state item=files_100_units_1000_unit_area_2_pow_21__head phase=window "
                               "state=same_mount operation=W1 operation_index=1 ")
    records_middle = int(fields(state)["allocation_records"])
    say(f"name=k1000l_middle_records allocation_records={records_middle} source=measured_middle_point")
    cells = {}
    for line in lines_of(first):
        if not line.startswith("name=cell segment=new set=K1000 "):
            continue
        cell = fields(line)
        if cell.get("tier") != "user_units_equal_declared" or cell.get("filter") != "all":
            continue
        cells.setdefault((cell["state"], cell["pick"]), {})[cell["arm"]] = cell
    check("k1000_user_units_cells", len(cells) == 6, f"cells={len(cells)}")
    tested_first = ends_of(cells[("same_mount", "first")]["tested"]["ends"])
    control_first = ends_of(cells[("same_mount", "first")]["c1"]["ends"])
    check("c1_is_tested_plus_records",
          all(control_first[i][0] == tested_first[i][0] + tested_first[i][2] for i in (0, 1)),
          f"tested={tested_first} c1={control_first}")
    for high_records in (2_000_000, 2_500_000, 10 * records_middle):
        not_red = []
        for (state_name, pick), arms in sorted(cells.items()):
            (low_steps, units, low_records), (high_steps, _, high_records_k1000) = ends_of(arms["tested"]["ends"])
            tested_alpha = growth(low_steps, high_steps, low_records, high_records_k1000)
            per_unit_low = low_steps / units
            per_unit_high = per_unit_low * (high_records / low_records) ** tested_alpha
            control_alpha = growth(per_unit_low + low_records / units, per_unit_high + high_records / units,
                                   low_records, high_records)
            say(f"name=c1_extrapolated high_records={high_records} state={state_name} pick={pick} "
                f"per_unit_low={per_unit_low:.1f} tested_alpha_k1000={tested_alpha:.3f} "
                f"per_unit_high_extrapolated={per_unit_high:.1f} c1_alpha={control_alpha:.3f} "
                f"red={'yes' if control_alpha >= 0.5 else 'no'} source=extrapolated")
            if control_alpha < 0.5:
                not_red.append(f"{state_name}/{pick}")
        say(f"name=c1_extrapolated_summary high_records={high_records} cells=6 not_red={len(not_red)} "
            f"which={','.join(not_red) or '-'} source=extrapolated")


# ---- 第三节：E209 行一窄读法的两处不符是不是旗线在一两个操作的窗里退化造出来的 ----
def lower_median(values):
    ordered = sorted(values)
    return ordered[(len(ordered) - 1) // 2]


def row_kind(row):
    point, script = row["item"].split("__head__", 1)
    if point == TRAINING_POINT and script in TRAINING_SCRIPTS and row["mount_epoch"] == "0":
        return "training"
    return "script" if point == TRAINING_POINT else "geometry"


def narrow_rule(results):
    path = os.path.join(results, "e209-spike-cause-report-third-2026-11-13.out")
    rows = [fields(line) for line in lines_of(path) if line.startswith("name=operation_row ")]
    windows = {}
    for row in rows:
        windows.setdefault((row["item"], row["mount_epoch"]), []).append(int(row["final_nodes"]))
    differing = 0
    for row in rows:
        window = windows[(row["item"], row["mount_epoch"])]
        registered_line = max(28, 4 * lower_median(window))
        nodes = int(row["final_nodes"])
        row["flag_registered"] = nodes >= registered_line
        row["flag_floor_in_small_windows"] = nodes >= (28 if len(window) < 3 else registered_line)
        differing += row["flag_registered"] != (row["flagged"] == "true")
    check("flag_line_reproduced", differing == 0, f"rows={len(rows)} rows_whose_flag_differs={differing}")
    for kind in ("training", "script", "geometry"):
        members = [row for row in rows if row_kind(row) == kind]
        flagged = sum(row["flag_registered"] for row in members)
        say(f"name=class_size kind={kind} rows={len(members)} flagged={flagged} unflagged={len(members) - flagged}")
    for row in rows:
        if row["flag_registered"] != row["flag_floor_in_small_windows"]:
            say(f"name=flag_changes_with_the_floor item={row['item']} mount_epoch={row['mount_epoch']} "
                f"step_index={row['step_index']} final_nodes={row['final_nodes']} "
                f"window_operations={len(windows[(row['item'], row['mount_epoch'])])} "
                f"fresh_mount_narrow={row['fresh_mount_narrow']}")
    expected = {("fresh_mount_narrow", "flag_registered"): (0, 1, 1),
                ("fresh_ever_narrow", "flag_registered"): (0, 1, 1),
                ("fresh_mount_narrow", "flag_floor_in_small_windows"): (0, 0, 0),
                ("fresh_ever_narrow", "flag_floor_in_small_windows"): (0, 0, 0)}
    for variant in ("fresh_mount_narrow", "fresh_ever_narrow", "fresh_mount_wide", "fresh_ever_wide"):
        for flag in ("flag_registered", "flag_floor_in_small_windows"):
            training = [row for row in rows if row_kind(row) == "training"]
            threshold = min(int(row[variant]) for row in training if row[flag])
            counts = []
            for kind in ("training", "script", "geometry"):
                members = [row for row in rows if row_kind(row) == kind]
                if kind == "training":
                    counts.append(sum(1 for row in members if int(row[variant]) >= threshold and not row[flag]))
                else:
                    counts.append(sum(1 for row in members if (int(row[variant]) >= threshold) != row[flag]))
            say(f"name=freshness_recheck variant={variant} flags={flag} threshold={threshold} "
                f"training_failures={counts[0]} script_mismatches={counts[1]} geometry_mismatches={counts[2]}")
            if (variant, flag) in expected:
                check(f"{variant}_{flag}", tuple(counts) == expected[(variant, flag)],
                      f"counts={tuple(counts)} expected={expected[(variant, flag)]}")


# ---- 第四节：E209 行五判「分散」的旗上操作，可比操作与它是不是同一次挂载 ----
def comparables(results):
    path = os.path.join(results, "e209-spike-cause-report-third-2026-11-13.out")
    epoch_of = {}
    extra_lines = []
    for line in lines_of(path):
        if line.startswith("name=operation_row "):
            row = fields(line)
            epoch_of[f"{row['item']}#{row['step_index']}"] = row["mount_epoch"]
        elif line.startswith("name=extra_steps "):
            extra_lines.append(fields(line))
    spread = 0
    same_mount = 0
    for extra in extra_lines:
        if extra.get("verdict") != "spread_over_many_rows":
            continue
        spread += 1
        epochs = [epoch_of.get(extra[key], "?") for key in ("operation", "comparable_first", "comparable_second")]
        same = epochs[0] != "?" and epochs[0] == epochs[1] == epochs[2]
        same_mount += same
        say(f"name=spread_operation operation={extra['operation']} epoch={epochs[0]} "
            f"comparable_first={extra['comparable_first']} epoch_first={epochs[1]} "
            f"comparable_second={extra['comparable_second']} epoch_second={epochs[2]} "
            f"noise={extra.get('noise', '-')} top_share={extra.get('top_share', '-')} same_mount={'yes' if same else 'no'}")
    check("spread_operations", spread == 8, f"spread={spread}")
    check("spread_with_same_mount_comparables", same_mount >= 1, f"same_mount={same_mount}")


# ---- 第五节：今天的实现上，一次覆盖写准入之前的 B1 各行随记录数涨多少（产物里的 name=row 行） ----
def before_admission_rows(results):
    # K1000 两点的 W2 第 3 次改动大小不同（1 对 1000 个单元），只取 W1 第 3 次
    for low_point, high_point, operations in (("files_100_units_100", "files_1000_units_100", (("W1", "3"), ("W2", "3"))),
                                              ("files_10_units_1000", "files_100_units_1000", (("W1", "3"),))):
        for operation, index in operations:
            readings = []
            for point in (low_point, high_point):
                path = os.path.join(results, f"e207-strict-scan-steps-{point}-head-2026-11-13.out")
                head = f"item={point}__head phase=window state=same_mount operation={operation} operation_index={index} "
                declared = fields(first_line(path, "name=declared " + head))
                state = fields(first_line(path, "name=state " + head))
                publish = f"publish=same_mount/{operation}/{index}/0 "
                sums = {"records_of_each_leaf": 0, "whole_allocator_copy_in_a_settling_round": 0,
                        "whole_allocator_copy_before_the_judgement": 0, "all": 0}
                for line in lines_of(path):
                    if not line.startswith("name=row ") or publish not in line:
                        continue
                    row = fields(line)
                    if row.get("segment") not in BEFORE_ADMISSION_SEGMENTS:
                        continue
                    steps = int(row["steps"])
                    sums["all"] += steps
                    if row["row"] in sums:
                        sums[row["row"]] += steps
                readings.append((int(state["allocation_records"]), declared["user_units"], sums))
            (low_records, low_units, low_sums), (high_records, high_units, high_sums) = readings
            check(f"same_change_{low_point}_{operation}_{index}", low_units == high_units,
                  f"user_units={low_units}/{high_units}")
            parts = " ".join(f"{name}={low_sums[name]}->{high_sums[name]}:alpha={growth(low_sums[name], high_sums[name], low_records, high_records):.3f}"
                             for name in sums)
            say(f"name=before_admission_b1 set={low_point}->{high_point} operation={operation}#{index} "
                f"user_units={low_units} allocation_records={low_records}->{high_records} {parts} "
                f"records_of_each_leaf_per_record={low_sums['records_of_each_leaf'] / low_records:.2f},"
                f"{high_sums['records_of_each_leaf'] / high_records:.2f}")


# ---- 第六节：重挂载的写行那一次发布带出的节点数随文件数涨多少（E209 行四的读数） ----
def remount_carried(results):
    path = os.path.join(results, "e209-spike-cause-report-third-2026-11-13.out")
    carried = {}
    for line in lines_of(path):
        if line.startswith("name=mount_carries "):
            row = fields(line)
            carried[row["point"]] = (int(row["nodes_of_the_spike"]), int(row["carried_by_the_mount"]))
    by_files = [(75, "files_75_units_100"), (100, "files_100_units_100"), (200, "files_200_units_100")]
    for files, point in by_files:
        say(f"name=remount_carried point={point} files={files} nodes_of_the_spike={carried[point][0]} "
            f"carried_by_the_mount={carried[point][1]}")
    low, high = carried[by_files[0][1]][1], carried[by_files[-1][1]][1]
    say(f"name=remount_carried_growth files=75->200 carried={low}->{high} "
        f"alpha_against_files={growth(low, high, 75, 200):.3f} source=measured_nodes_not_steps")


def main(arguments):
    if len(arguments) != 2 or arguments[0] != "--repo":
        print("用法：python3 -I b8_r2_readings.py --repo <仓根>", file=sys.stderr)
        return 2
    results = os.path.join(arguments[1], "research", "results")
    names = ["e207-strict-scan-steps-report-first-2026-11-13-r3.out",
             "e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out",
             "e207-strict-scan-steps-files_100_units_100-head-2026-11-13.out",
             "e207-strict-scan-steps-files_1000_units_100-head-2026-11-13.out",
             "e207-strict-scan-steps-files_10_units_1000-head-2026-11-13.out",
             "e207-strict-scan-steps-files_100_units_1000-head-2026-11-13.out",
             "e207-strict-scan-steps-files_100_units_1000_unit_area_2_pow_21-head-2026-11-13.out",
             "e209-spike-cause-report-third-2026-11-13.out"]
    for name in names:
        path = os.path.join(results, name)
        if not os.path.isfile(path):
            print(f"读不到产物 {path}", file=sys.stderr)
            return 2
        say(f"name=input file=research/results/{name} sha256_16={sha16(path)}")
    for section in (window_asymmetry, control_extrapolation, narrow_rule, comparables,
                    before_admission_rows, remount_carried):
        section(results)
    say(f"name=done checks_failed={len(FAILED)} failed={','.join(FAILED) or '-'}")
    return 1 if FAILED else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
