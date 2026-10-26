#!/usr/bin/env python3
# admission: always 每一次调都比此刻点名的产物（前、后各两份），上一次的结论不替这一次作保
# run-condition: none 只读点名的产物文本，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置
"""E185 第 1 次重跑（research/prompts/e185-r1-prereg.md）第六节 C 组的比法：七·6 合入前后一批 CPU 随池大小的增长。

只读产物文本的算术，独立手写，不 import crates/ 任何东西。门槛跑前写死：指数 alpha <= 1.25 判「两点之间接近线性」，
alpha >= 1.75 判「两点之间近平方」，中间判「介于两者」，每轮判定不全相同判「不稳定」。

用法：
  e185_r1_compare.py --before-wall-clock <前的第一段产物> --before-stage-split <前的第二段产物>
                     [--after-wall-clock <后的第一段产物>] [--after-stage-split <后的第二段产物>]
                     [--inputs-list <research/results/e185-r1-inputs-日期.txt>]
      开算前先核「前」两份的 sha256（登记第五节那两个值），不同就停、退 3，一步不算。
      缺的一侧（后的产物没给）对应的格报「本次未跑」，不当 0。每行 E7RESULT name=<名> 键=值，末行 name=done。
      退出码：0 算完；2 用法或产物读不了；3 「前」的产物 sha256 不对；4 比法锚点不符或输入清单不合（这一份输出作废）。
单测在同目录的 e185_r1_compare_test.py（合成产物上的自证，每一项标签就是登记第九节变异表 e185_r1_compare.mutations.tsv 点名要红的那一格）。
"""
import hashlib
import math
import re
import statistics
import sys

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

TOKEN = re.compile(r'(\w+)=("[^"]*"|\S+)')

# 登记第五节：上一趟（2026-10-20）两份产物的 sha256
BASELINE_WALL_CLOCK_SHA256 = "a6afc7bf2f380d86f3b51a070c91465c3206a2476c216af6ab59fbe700a7d87b"
BASELINE_STAGE_SPLIT_SHA256 = "e087a1b6834a381322c87ebd0a53a242b66ce035812fab39d7628b79e3a9e125"
# 登记第十一节「装置变了」：装置 sha256；输入清单里要有它、不许出现计数特性
DEVICE_SHA256 = "b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470"
COUNTER_FEATURE = "lookup-work-counters"

POOL_SIZE_SMALL = "1024"
POOL_SIZE_LARGE = "8192"
POOL_SIZE_RATIO = 8
# 登记第六节 C1 的门槛，跑前写死
NEAR_LINEAR_LIMIT = 1.25
NEAR_SQUARE_LIMIT = 1.75
BATCH_KINDS = ("batch", "final_batch")
CALL_FIELDS_COMPARED = (
    "kind",
    "scan_read_bytes_in_the_region",
    "release_check_read_bytes",
    "relocation_copy_bytes",
)
CALL_FIELDS_REPORTED = ("write_bytes", "metadata_write_bytes")
# 登记第七节乙类：数据单元 32768 字节；R 里可搬的单元数 m = 32 f；签发那一次扫 R 读到的字节
UNIT_BYTES = 32768
MOVABLE_UNITS = {"0.25": 8, "0.75": 24}
ISSUE_SCAN_BYTES = {"0.25": 262144, "0.75": 786432}


class Unreadable(Exception):
    pass


def parse_rows(text):
    """每个 E7RESULT 行一个字典；同名键取第一次出现的（call 行里 write_bytes 出现两次，第一个是整次调用的）。"""
    rows = []
    for line_number, line in enumerate(text.splitlines(), 1):
        if not line.startswith("E7RESULT "):
            continue
        row = {"_line": line_number}
        for key, value in TOKEN.findall(line):
            row.setdefault(key, value.strip('"'))
        rows.append(row)
    return rows


def rows_named(rows, name):
    return [row for row in rows if row.get("name") == name]


def load_product(path, expected_segment):
    """读一份产物：segment 对得上、末行 name=done 且 emitted 等于行数，否则 Unreadable。返回 (sha256, 行)。"""
    try:
        with open(path, "rb") as handle:
            data = handle.read()
    except OSError as error:
        raise Unreadable(f"{path} 读不了：{error}")
    rows = parse_rows(data.decode("utf-8", errors="replace"))
    configuration = rows_named(rows, "configuration")
    if len(configuration) != 1 or configuration[0].get("segment") != expected_segment:
        raise Unreadable(f"{path} 不是第 {expected_segment} 段的产物（name=configuration 的 segment 对不上）")
    if not rows or rows[-1].get("name") != "done" or rows[-1].get("emitted") != str(len(rows)):
        raise Unreadable(f"{path} 没写完：末行不是 name=done，或 emitted 不等于 E7RESULT 行数 {len(rows)}")
    return hashlib.sha256(data).hexdigest(), rows


def emit(lines, name, **fields):
    lines.append("E7RESULT name=" + name + "".join(f" {key}={value}" for key, value in fields.items()))


def fmt(number, digits=6):
    return "none" if number is None else f"{number:.{digits}f}"


# ── 选行：内存臂、无注入、有效轮 ────────────────────────────────────────────

def is_memory_without_injection(row):
    return row.get("arm") == "memory" and row.get("injection") == "none"


def round_key(row):
    return (row["fraction"], row["pool_data_units"], row["batch_units"], int(row["round"]))


def valid_attempt_by_round(rows):
    """{(fraction, pool_data_units, batch_units, round): 有效那一次的 attempt}，来自 name=intent 的 valid=true。"""
    attempts = {}
    for row in rows_named(rows, "intent"):
        if is_memory_without_injection(row) and row.get("valid") == "true":
            attempts[round_key(row)] = row["attempt"]
    return attempts


def batch_row_with_least_call_index(call_rows):
    """call_index 最小的那一行批（签发那一次不算）。"""
    batches = [row for row in call_rows if row["kind"] in BATCH_KINDS]
    return min(batches, key=lambda row: int(row["call_index"])) if batches else None


def valid_call_rows_by_round(rows):
    """{(fraction, pool_data_units, batch_units, round): [有效轮的 call 行]}"""
    attempts = valid_attempt_by_round(rows)
    grouped = {}
    for row in rows_named(rows, "call"):
        key = round_key(row)
        if is_memory_without_injection(row) and key in attempts and row["attempt"] == attempts[key]:
            grouped.setdefault(key, []).append(row)
    return grouped


# ── 一批 CPU 的序列（登记第六节 C1、C1b） ────────────────────────────────────

def wall_clock_series(rows):
    """来源 W：第一段 name=call 内存臂无注入、有效轮、call_index 最小的批那一行的 cpu_nanoseconds。
    返回 (values, unreadable)：values[(fraction, batch_units)][pool_data_units][round]，unreadable 数 cpu_unreadable=true 的轮。"""
    values, unreadable = {}, {}
    for key, call_rows in valid_call_rows_by_round(rows).items():
        fraction, pool, batch, round_number = key
        leading_batch = batch_row_with_least_call_index(call_rows)
        cell = values.setdefault((fraction, batch), {}).setdefault(pool, {})
        if leading_batch is None or leading_batch.get("cpu_unreadable") == "true":
            unreadable[(fraction, batch, pool)] = unreadable.get((fraction, batch, pool), 0) + 1
            continue
        cell[round_number] = float(leading_batch["cpu_nanoseconds"])
    return values, unreadable


def stage_series(rows, remainder=False):
    """来源 S：第二段 name=stage，call_index 最小的批那一行的 call_cpu_nanoseconds；remainder 时减 scan_cpu_nanoseconds（C1b）。
    作废轮第二段不打 name=stage，所以这里不看 name=intent。"""
    grouped = {}
    for row in rows_named(rows, "stage"):
        key = (row["fraction"], row["pool_data_units"], row["batch_units"], int(row["round"]))
        grouped.setdefault(key, []).append(row)
    values = {}
    for (fraction, pool, batch, round_number), stage_rows in grouped.items():
        leading_batch = batch_row_with_least_call_index(stage_rows)
        cell = values.setdefault((fraction, batch), {}).setdefault(pool, {})
        if leading_batch is None:
            continue
        value = float(leading_batch["call_cpu_nanoseconds"])
        if remainder:
            value -= float(leading_batch["scan_cpu_nanoseconds"])
        cell[round_number] = value
    return values


# ── 指数与判定 ────────────────────────────────────────────────────────────

def median_of(values):
    return statistics.median(values)


def exponent(smaller, larger):
    """alpha = ln(c(8192) ÷ c(1024)) ÷ ln 8；读不到或不为正的当读不到，不当 0。"""
    if smaller <= 0 or larger <= 0:
        return None
    return math.log(larger / smaller) / math.log(POOL_SIZE_RATIO)


def judge_exponent(alpha, near_linear_limit=NEAR_LINEAR_LIMIT, near_square_limit=NEAR_SQUARE_LIMIT):
    if alpha is None:
        return "not_readable"
    if alpha <= near_linear_limit:
        return "near_linear_between_the_two_points"
    if alpha >= near_square_limit:
        return "near_square_between_the_two_points"
    return "between_the_two"


def exponent_cell(small_by_round, large_by_round):
    """一个（fraction, batch_units）格：c(P) 取有效轮的中位，alpha 用两个中位；另按轮号配对报每轮 alpha。"""
    if not small_by_round or not large_by_round:
        return {"alpha": None, "judgement": "not_readable", "small_median": None, "large_median": None,
                "per_round": {}, "per_round_judgements": [], "stable": False,
                "rounds_small": len(small_by_round), "rounds_large": len(large_by_round)}
    small_median = median_of(list(small_by_round.values()))
    large_median = median_of(list(large_by_round.values()))
    alpha = exponent(small_median, large_median)
    common = sorted(set(small_by_round) & set(large_by_round))
    small_values = [small_by_round[round_number] for round_number in common]
    large_values = [large_by_round[round_number] for round_number in common]
    per_round = [exponent(small, large) for small, large in zip(small_values, large_values)]
    judgements = [judge_exponent(each) for each in per_round]
    stable = len(set(judgements)) == 1
    return {"alpha": alpha, "judgement": judge_exponent(alpha) if stable else "unstable",
            "median_judgement": judge_exponent(alpha), "small_median": small_median, "large_median": large_median,
            "per_round": dict(zip(common, per_round)), "per_round_judgements": judgements, "stable": stable,
            "rounds_small": len(small_by_round), "rounds_large": len(large_by_round)}


def exponent_cells(values):
    """values[(fraction, batch_units)][pool][round] → {(fraction, batch_units): exponent_cell}，两个 P 点都读。"""
    return {key: exponent_cell(by_pool.get(POOL_SIZE_SMALL, {}), by_pool.get(POOL_SIZE_LARGE, {}))
            for key, by_pool in values.items()}


# ── 前后是不是同一份活（登记第六节 C0） ──────────────────────────────────────

def work_description(rows):
    """一份产物里「同一份活」要比的几样：几何的 choice 与 formed、造池后的映射条目数、一条意图的批数、第 1 轮每个调用的读写字节。"""
    geometry = {(row["fraction"], row["pool_data_units"]): {"choice": row["choice"], "formed": row["formed"]}
                for row in rows_named(rows, "geometry")}
    mapped_units = {}
    for row in rows_named(rows, "geometry_attempt"):
        mapped_units[(row["fraction"], row["pool_data_units"])] = row["mapped_units"]  # 最后一次
    attempts = valid_attempt_by_round(rows)
    batches = {}
    for row in rows_named(rows, "intent"):
        key = round_key(row)
        if is_memory_without_injection(row) and key in attempts and attempts[key] == row["attempt"] and key[3] == 1:
            batches[key[:3]] = row["batches"]
    calls = {}
    for key, call_rows in valid_call_rows_by_round(rows).items():
        if key[3] == 1:
            calls[key[:3]] = {int(row["call_index"]): row for row in call_rows}
    return {"geometry": geometry, "mapped_units": mapped_units, "batches": batches, "calls": calls}


def same_work_cells(before_rows, after_rows):
    """每个（fraction, pool_data_units, batch_units）一格：same_work / different_work / unreadable（某一侧第 1 轮没有有效的数）。"""
    before, after = work_description(before_rows), work_description(after_rows)
    cells = {}
    for triple in sorted(set(before["calls"]) | set(after["calls"])):
        point = triple[:2]
        if triple not in before["calls"] or triple not in after["calls"]:
            cells[triple] = {"judgement": "unreadable", "differences": ["calls_of_the_comparison_round_missing_on_one_side"], "write_differences": []}
            continue
        differences, write_differences = [], []
        for field in ("choice", "formed"):
            seen = (before["geometry"].get(point, {}).get(field), after["geometry"].get(point, {}).get(field))
            if seen[0] != seen[1]:
                differences.append(f"{field}:{seen[0]}->{seen[1]}")
        for label, table in (("mapped_units", "mapped_units"), ("batches", "batches")):
            key = point if table == "mapped_units" else triple
            seen = (before[table].get(key), after[table].get(key))
            if seen[0] != seen[1]:
                differences.append(f"{label}:{seen[0]}->{seen[1]}")
        before_calls, after_calls = before["calls"][triple], after["calls"][triple]
        if set(before_calls) != set(after_calls):
            differences.append(f"call_indexes:{sorted(before_calls)}->{sorted(after_calls)}".replace(" ", ""))
        for index in sorted(set(before_calls) & set(after_calls)):
            for field in CALL_FIELDS_COMPARED:
                if before_calls[index].get(field) != after_calls[index].get(field):
                    differences.append(f"call{index}_{field}:{before_calls[index].get(field)}->{after_calls[index].get(field)}")
            for field in CALL_FIELDS_REPORTED:
                if before_calls[index].get(field) != after_calls[index].get(field):
                    write_differences.append(f"call{index}_{field}:{before_calls[index].get(field)}->{after_calls[index].get(field)}")
        cells[triple] = {"judgement": "different_work" if differences else "same_work",
                         "differences": differences, "write_differences": write_differences}
    return cells


# ── 比法锚点（登记第七节乙类，只在 name=geometry 为 formed=true 的点上判） ────────

def anchor_cells(rows):
    """每个（fraction, pool_data_units, batch_units）一格：第一批的 relocation_copy_bytes 是 2 × min(K, m) × 32768，
    签发那一次的 scan_read_bytes_in_the_region 是 f = 0.25 的 262144、f = 0.75 的 786432。每个有效轮都核。"""
    formed = {(row["fraction"], row["pool_data_units"]) for row in rows_named(rows, "geometry") if row.get("formed") == "true"}
    by_triple = {}
    for key, call_rows in valid_call_rows_by_round(rows).items():
        by_triple.setdefault(key[:3], []).append(call_rows)
    cells = {}
    for triple in sorted(by_triple):
        fraction, pool, batch = triple
        if (fraction, pool) not in formed or fraction not in MOVABLE_UNITS:
            cells[triple] = {"status": "not_checked", "mismatches": 0, "rounds_checked": 0}
            continue
        expected_copy = 2 * min(int(batch), MOVABLE_UNITS[fraction]) * UNIT_BYTES
        expected_scan = ISSUE_SCAN_BYTES[fraction]
        copy_seen, scan_seen, mismatches = set(), set(), 0
        for call_rows in by_triple[triple]:
            leading_batch = batch_row_with_least_call_index(call_rows)
            issues = [row for row in call_rows if row["kind"] == "issue"]
            copy_value = leading_batch.get("relocation_copy_bytes") if leading_batch else None
            scan_value = issues[0].get("scan_read_bytes_in_the_region") if issues else None
            copy_seen.add(copy_value)
            scan_seen.add(scan_value)
            if copy_value != str(expected_copy) or scan_value != str(expected_scan):
                mismatches += 1
        cells[triple] = {"status": "ok" if mismatches == 0 else "mismatch", "mismatches": mismatches,
                         "rounds_checked": len(by_triple[triple]), "expected_copy": expected_copy, "copy_seen": sorted(map(str, copy_seen)),
                         "expected_scan": expected_scan, "scan_seen": sorted(map(str, scan_seen))}
    return cells


# ── 岔路量并列（登记第六节 C3）与盘、输入清单的核对 ──────────────────────────

POINT_KEYS = ("fraction", "pool_data_units", "batch_units")
# (段, 行名, 键字段, 取的字段, 哪些格靠真盘：disk 全靠、by_arm 看键里的 arm、none 不靠, 是否每轮一行要取各轮中位)
PAIRED_SPECS = (
    ("wall_clock", "q1", POINT_KEYS, ("rho_median", "judgement"), "disk", False),
    ("wall_clock", "q2", POINT_KEYS, ("intent_wall_ratio_median", "judgement"), "disk", False),
    ("wall_clock", "q3", POINT_KEYS, ("judgement",), "disk", False),
    ("wall_clock", "q4", ("arm", "fraction", "pool_data_units", "batch_units_pair"), ("median", "judgement"), "by_arm", False),
    ("wall_clock", "q14", POINT_KEYS, ("write_bytes_per_second_median",), "disk", False),
    ("wall_clock", "q15", (), ("direct_with_fdatasync_bytes_per_second_median",), "disk", False),
    ("wall_clock", "q16", (), ("maximum_ratio", "judgement"), "disk", False),
    ("stage_split", "stage_judgement", POINT_KEYS + ("batch_index",),
     ("crc_share_judgement", "scan_increment_share_judgement", "choose_share_judgement"), "none", False),
    ("stage_split", "stage", POINT_KEYS + ("call_index", "kind"),
     ("crc_share", "scan_increment_share", "choose_share_per_batch", "crc_bytes_per_second", "scan_increment_bytes_per_second"), "none", True),
    ("stage_split", "saturation", POINT_KEYS + ("stage", "threads"), ("bytes_per_second",), "none", True),
    ("stage_split", "choose_disk_size", POINT_KEYS, ("cpu_nanoseconds_per_full_segment",), "none", True),
)


def as_number(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def paired_table(rows, spec):
    """{(键串, 字段): 值}；每轮一行的行按键串聚成各轮的中位（取不成数的写 mixed 或原值）。"""
    _, name, keys, fields, _, per_round = spec
    grouped = {}
    for row in rows_named(rows, name):
        grouped.setdefault(";".join(f"{key}={row.get(key, 'none')}" for key in keys), []).append(row)
    table = {}
    for key, group in grouped.items():
        for field in fields:
            texts = [row.get(field) for row in group]
            numbers = [as_number(text) for text in texts]
            if per_round and None not in numbers:
                table[(key, field)] = median_of(numbers)
            elif per_round and len(set(texts)) > 1:
                table[(key, field)] = "mixed"
            elif field.endswith("judgement") or numbers[0] is None:
                table[(key, field)] = texts[0]
            else:
                table[(key, field)] = numbers[0]
    return table


def disk_identity(before_rows, after_rows):
    """两趟第一段 name=configuration 的 image_directory_filesystem 与 image_device 是不是同一个（登记第十一节「真盘不是同一块」）。"""
    before, after = rows_named(before_rows, "configuration")[0], rows_named(after_rows, "configuration")[0]
    fields = {key: (before.get(key), after.get(key)) for key in ("image_directory_filesystem", "image_device")}
    return all(before_value == after_value for before_value, after_value in fields.values()), fields


def inputs_list_problems(text):
    """输入清单（登记第五节）里装置 sha256 要在、cargo tree 的输出里不许有计数特性（登记第十一节两条）。"""
    problems = []
    if DEVICE_SHA256 not in text:
        problems.append("device_sha256_not_in_the_list")
    if COUNTER_FEATURE in text:
        problems.append("counter_feature_in_the_cargo_tree")
    return problems


# ── 一次比较 ──────────────────────────────────────────────────────────────

SOURCES = (
    ("wall_clock_call_cpu", "wall_clock"),
    ("stage_split_call_cpu", "stage_split"),
    ("stage_split_remainder", "stage_split"),
)


def source_values(source, rows):
    if source == "wall_clock_call_cpu":
        return wall_clock_series(rows)[0]
    return stage_series(rows, remainder=(source == "stage_split_remainder"))


def unreadable_rounds(rows):
    return sum(wall_clock_series(rows)[1].values())


def geometry_formed_text(rows, fraction):
    formed = {row["pool_data_units"]: row.get("formed") for row in rows_named(rows, "geometry") if row["fraction"] == fraction}
    return "true" if formed.get(POOL_SIZE_SMALL) == "true" and formed.get(POOL_SIZE_LARGE) == "true" else "false"


def file_sha256(path):
    try:
        with open(path, "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()
    except OSError as error:
        raise Unreadable(f"{path} 读不了：{error}")


def emit_exponent_lines(lines, side, rows_by_segment, counts):
    cells_by_source = {}
    for source, segment in SOURCES:
        rows = rows_by_segment.get(segment)
        if rows is None:
            emit(lines, "cpu_exponent", cell="C1", source=source, side=side, status="not_run")
            continue
        cells = exponent_cells(source_values(source, rows))
        cells_by_source[source] = cells
        for (fraction, batch), cell in sorted(cells.items()):
            if cell["judgement"] == "not_readable":
                counts["unreadable_cells"] += 1
            rounds = sorted(cell["per_round"])
            emit(lines, "cpu_exponent", cell="C1b" if source.endswith("remainder") else "C1", source=source, side=side, fraction=fraction,
                 batch_units=batch, geometry_formed=geometry_formed_text(rows, fraction), cpu_1024=fmt(cell["small_median"], 1),
                 cpu_8192=fmt(cell["large_median"], 1), rounds_1024=cell["rounds_small"], rounds_8192=cell["rounds_large"],
                 alpha=fmt(cell["alpha"]), judgement=cell["judgement"], median_judgement=cell.get("median_judgement", cell["judgement"]),
                 rounds_paired=",".join(map(str, rounds)) or "none", per_round_alpha=",".join(fmt(cell["per_round"][each]) for each in rounds) or "none",
                 per_round_judgements=",".join(cell["per_round_judgements"]) or "none",
                 unreadable_rounds_in_the_segment=unreadable_rounds(rows) if source == "wall_clock_call_cpu" else 0)
    return cells_by_source


def emit_clauses_before(lines, cells_by_source, counts):
    """G2：前一趟这一格不是近平方；G3：同一趟同一格 W 与 S 的判定不同（前一趟）。"""
    for source, cells in cells_by_source.items():
        for (fraction, batch), cell in sorted(cells.items()):
            triggered = cell["alpha"] is not None and cell["alpha"] < NEAR_SQUARE_LIMIT
            counts["g2"] += triggered
            emit(lines, "clause", clause="G2_before_not_near_square", source=source, fraction=fraction, batch_units=batch,
                 alpha=fmt(cell["alpha"]), triggered=str(triggered).lower())
    emit_source_disagreement(lines, "before", cells_by_source, counts)


def emit_source_disagreement(lines, side, cells_by_source, counts):
    wall, stage = cells_by_source.get("wall_clock_call_cpu"), cells_by_source.get("stage_split_call_cpu")
    if wall is None or stage is None:
        emit(lines, "clause", clause="G3_two_sources_disagree", side=side, status="not_run")
        return
    for key in sorted(set(wall) & set(stage)):
        triggered = wall[key]["judgement"] != stage[key]["judgement"]
        counts["g3_" + side] += triggered
        emit(lines, "clause", clause="G3_two_sources_disagree", side=side, fraction=key[0], batch_units=key[1],
             wall_clock_judgement=wall[key]["judgement"], stage_split_judgement=stage[key]["judgement"], triggered=str(triggered).lower())


def emit_after_comparisons(lines, segment, before_rows, after_rows, after_cells_by_source, counts):
    """第一段与第二段各自：同一份活、前后比、G1、G4。"""
    same = same_work_cells(before_rows, after_rows)
    for triple, cell in sorted(same.items()):
        counts["g4"] += cell["judgement"] == "different_work"
        counts["unreadable_cells"] += cell["judgement"] == "unreadable"
        emit(lines, "same_work", cell="C0", segment=segment, fraction=triple[0], pool_data_units=triple[1], batch_units=triple[2],
             judgement=cell["judgement"], differences=",".join(cell["differences"]) or "none",
             write_differences=",".join(cell["write_differences"]) or "none")
    for source, segment_of_source in SOURCES:
        if segment_of_source != segment:
            continue
        before_values = source_values(source, before_rows)
        after_values = source_values(source, after_rows)
        for (fraction, batch) in sorted(set(before_values) | set(after_values)):
            for pool in (POOL_SIZE_SMALL, POOL_SIZE_LARGE):
                before = before_values.get((fraction, batch), {}).get(pool)
                after = after_values.get((fraction, batch), {}).get(pool)
                work = same.get((fraction, pool, batch), {"judgement": "unreadable"})["judgement"]
                ratio = None
                if work == "same_work" and before and after:
                    ratio = median_of(list(after.values())) / median_of(list(before.values()))
                emit(lines, "cpu_ratio", cell="C2", source=source, fraction=fraction, pool_data_units=pool, batch_units=batch,
                     same_work=work, after_over_before=fmt(ratio), status="compared" if ratio is not None else "not_compared")
        for key, cell in sorted(after_cells_by_source.get(source, {}).items()):
            both_same = all(same.get((key[0], pool, key[1]), {}).get("judgement") == "same_work" for pool in (POOL_SIZE_SMALL, POOL_SIZE_LARGE))
            if not both_same:
                emit(lines, "clause", clause="G1_after_not_near_linear", source=source, fraction=key[0], batch_units=key[1], status="same_work_not_judged")
                continue
            triggered = cell["judgement"] != "near_linear_between_the_two_points"
            counts["g1"] += triggered
            emit(lines, "clause", clause="G1_after_not_near_linear", source=source, fraction=key[0], batch_units=key[1],
                 alpha=fmt(cell["alpha"]), judgement=cell["judgement"], triggered=str(triggered).lower())


def format_paired(value):
    return f"{value:.6f}" if isinstance(value, float) else str(value)


def emit_paired_lines(lines, segment, before_rows, after_rows, disk_same, counts):
    """C3：同一格前后并列；判定字段前后不同记 flipped=true（前后翻面，两棵树之间）。缺的一侧每个 spec 一行「本次未跑」。"""
    for spec in (spec for spec in PAIRED_SPECS if spec[0] == segment):
        before = paired_table(before_rows, spec)
        if after_rows is None:
            emit(lines, "paired_value", cell="C3", segment=segment, row=spec[1], before_values=len(before), after="not_run")
            continue
        after = paired_table(after_rows, spec)
        for key, field in sorted(set(before) | set(after)):
            before_value, after_value = before.get((key, field)), after.get((key, field))
            depends_on_disk = spec[4] == "disk" or (spec[4] == "by_arm" and "arm=real_disk" in key)
            if before_value is None or after_value is None:
                status = "only_before" if after_value is None else "only_after"
            elif depends_on_disk and not disk_same:
                status = "different_disk_not_compared"
            else:
                status = "compared"
            flipped = field.endswith("judgement") and status == "compared" and before_value != after_value
            counts["g5"] += flipped
            ratio = None
            if status == "compared" and isinstance(before_value, float) and isinstance(after_value, float) and before_value != 0:
                ratio = after_value / before_value
            emit(lines, "paired_value", cell="C3", segment=segment, row=spec[1], key=key or "none", field=field,
                 before=format_paired(before_value), after=format_paired(after_value), after_over_before=fmt(ratio),
                 flipped=str(flipped).lower(), status=status)


def run_comparison(options, lines, expected_wall_clock=BASELINE_WALL_CLOCK_SHA256, expected_stage_split=BASELINE_STAGE_SPLIT_SHA256):
    """返回退出码；输出行追加进 lines。0 算完，2 产物读不了，3 前的产物 sha256 不对，4 比法锚点不符或输入清单不合。"""
    counts = {"unreadable_cells": 0, "g1": 0, "g2": 0, "g3_before": 0, "g3_after": 0, "g4": 0, "g5": 0, "anchor_mismatches": 0, "inputs_problems": 0}
    after_given = {"wall_clock": options.get("after_wall_clock"), "stage_split": options.get("after_stage_split")}
    emit(lines, "configuration", segment="e185_r1_compare", near_linear_limit=NEAR_LINEAR_LIMIT, near_square_limit=NEAR_SQUARE_LIMIT,
         pool_sizes=f"{POOL_SIZE_SMALL},{POOL_SIZE_LARGE}", after_wall_clock="given" if after_given["wall_clock"] else "not_run",
         after_stage_split="given" if after_given["stage_split"] else "not_run")
    mismatches = 0
    for label, path, expected in (("wall_clock", options["before_wall_clock"], expected_wall_clock),
                                  ("stage_split", options["before_stage_split"], expected_stage_split)):
        seen = file_sha256(path)
        mismatches += seen != expected
        emit(lines, "baseline_check", segment=label, expected_sha256=expected, seen_sha256=seen, ok=str(seen == expected).lower())
    if mismatches:
        emit(lines, "stopped", reason="the_previous_run_products_changed", baseline_sha_mismatches=mismatches)
        emit(lines, "done", emitted=len(lines) + 1)
        return 3
    before = {label: load_product(options[f"before_{label}"], label)[1] for label in ("wall_clock", "stage_split")}
    after = {}
    for label, path in after_given.items():
        if path:
            sha, after[label] = load_product(path, label)
            if sha in (expected_wall_clock, expected_stage_split):
                raise Unreadable(f"{path} 就是上一趟的产物，不能当「后」的")
    if options.get("inputs_list"):
        try:
            with open(options["inputs_list"], encoding="utf-8") as handle:
                problems = inputs_list_problems(handle.read())
        except OSError as error:
            raise Unreadable(f"{options['inputs_list']} 读不了：{error}")
        counts["inputs_problems"] = len(problems)
        emit(lines, "inputs_list_check", problems=",".join(problems) or "none")
    else:
        emit(lines, "inputs_list_check", problems="not_run")
    for side, rows_by_segment in (("before", before), ("after", after)):
        for segment, rows in sorted(rows_by_segment.items()):
            for triple, cell in anchor_cells(rows).items():
                counts["anchor_mismatches"] += cell["mismatches"]
                emit(lines, "anchor", cell="anchor", side=side, segment=segment, fraction=triple[0], pool_data_units=triple[1], batch_units=triple[2],
                     status=cell["status"], rounds_checked=cell["rounds_checked"], mismatches=cell["mismatches"],
                     expected_copy_bytes=cell.get("expected_copy", "none"), copy_bytes_seen=",".join(cell.get("copy_seen", [])) or "none",
                     expected_issue_scan_bytes=cell.get("expected_scan", "none"), issue_scan_bytes_seen=",".join(cell.get("scan_seen", [])) or "none")
    before_cells = emit_exponent_lines(lines, "before", before, counts)
    emit_clauses_before(lines, before_cells, counts)
    after_cells = emit_exponent_lines(lines, "after", after, counts) if after else {}
    if after:
        emit_source_disagreement(lines, "after", after_cells, counts)
    disk_same = None
    if "wall_clock" in after:
        disk_same, fields = disk_identity(before["wall_clock"], after["wall_clock"])
        emit(lines, "disk_identity", same_disk=str(disk_same).lower(),
             **{f"{key}_before_after": f"{pair[0]}->{pair[1]}" for key, pair in fields.items()})
    for segment in ("wall_clock", "stage_split"):
        if segment in after:
            emit_after_comparisons(lines, segment, before[segment], after[segment], after_cells, counts)
        else:
            emit(lines, "same_work", cell="C0", segment=segment, judgement="not_run", reason="the_after_product_was_not_given")
            emit(lines, "cpu_ratio", cell="C2", segment=segment, status="not_run", reason="the_after_product_was_not_given")
        emit_paired_lines(lines, segment, before[segment], after.get(segment), disk_same, counts)
    void = counts["anchor_mismatches"] > 0 or counts["inputs_problems"] > 0
    not_run = "not_run"
    emit(lines, "verdict", baseline_sha_mismatches=0, anchor_mismatches=counts["anchor_mismatches"], inputs_list_problems=counts["inputs_problems"],
         unreadable_cells=counts["unreadable_cells"], g1_after_not_near_linear=counts["g1"] if after else not_run, g2_before_not_near_square=counts["g2"],
         g3_two_sources_disagree_before=counts["g3_before"], g3_two_sources_disagree_after=counts["g3_after"] if after else not_run,
         g4_different_work=counts["g4"] if after else not_run,
         g5_flipped=counts["g5"] if after else not_run, output_void=str(void).lower())
    emit(lines, "done", emitted=len(lines) + 1)
    return 4 if void else 0


def main(argv):
    options, names = {}, {"--before-wall-clock": "before_wall_clock", "--before-stage-split": "before_stage_split",
                          "--after-wall-clock": "after_wall_clock", "--after-stage-split": "after_stage_split", "--inputs-list": "inputs_list"}
    position = 0
    while position < len(argv):
        if argv[position] not in names or position + 1 >= len(argv):
            print(__doc__)
            return 2
        options[names[argv[position]]] = argv[position + 1]
        position += 2
    if "before_wall_clock" not in options or "before_stage_split" not in options:
        print(__doc__)
        return 2
    lines = []
    try:
        code = run_comparison(options, lines)
    except Unreadable as unreadable:
        print(f"  ✗ {unreadable}", file=sys.stderr)
        print("     → 怎么办：核命令行给的产物路径在、读得了，再同一条命令重跑", file=sys.stderr)
        return 2
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
