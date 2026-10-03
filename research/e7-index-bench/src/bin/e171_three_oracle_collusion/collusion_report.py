"""E171 比对器的行文：坏实现表每一行的分类、矩阵、触达、轨迹，以及 Q1 / Q3 / Q4 的汇总（登记第六节）。纯函数出结果行，`comparator.py` 负责读盘与落盘。

分类（登记第一节读法 5）：计入分母的行叫 `counted`；`untouched`（未触达）与 `only_in_the_ignored_pass` 不进分母但各自报；编不过、锚点不唯一、找不到、
实现自崩、超时合计与所跑行数比（F6）。O3 是冻结的执行器（登记第一节读法 8）判的：严格跑法判「规约欠一句」、执行器出错、底镜像恢复不出都算弃权、不红（读法 7）；
O3 没跑的行（`o3` 是 None，例如 Q3-核「驱动不跑执行器」那一次）O3 那几列写 `not_run`。
"""

from __future__ import annotations

from dataclasses import dataclass

from collusion_matrix import (CATEGORIES_THAT_COUNT_AGAINST_THE_TABLE, CATEGORY_ANCHOR_NOT_UNIQUE, CATEGORY_COMPILE_FAILED,
                              CATEGORY_COUNTED, CATEGORY_NOT_FOUND, CATEGORY_ONLY_IN_THE_IGNORED_PASS, CATEGORY_SELF_CRASH,
                              CATEGORY_TIMEOUT, CATEGORY_UNTOUCHED, FIRST_BATCH, ORACLE_COMPARISON, ORACLE_CRASH_JUDGE,
                              ORACLE_ONE, ORACLE_THREE_PROVISIONAL, ORACLE_THREE_STRICT, ORACLE_TWO, SECOND_BATCH, O3Verdicts,
                              TouchReport, all_quiet, column_is_red, comparison_trajectory, coverage_enough_to_judge,
                              crash_judge_not_applicable_counts, errored_calls_left_unjudged, layer0_oracle_red_in_batch,
                              not_applicable_states_with_another_source, o3_judged_state_count,
                              oracles_that_alone_hold_the_row, red_calls, red_columns, red_state_counts,
                              strict_reading_is_unreachable, touch_report, trajectory)
from result_lines import ResultLines, field
from run_directory import STREAM_NAMES, RunDirectory

CATEGORY_DEVICE_STOP = "device_stop"
CATEGORY_WRAPPER_ENDED = "wrapper_ended"
DRIVER_STATUS_OK = "ok"
TIMEOUT_EXIT_CODE = 253
WRAPPER_EXIT_CODES = range(250, 255)
DEVICE_STOP_EXIT_CODE = 3
O3_STATUS_RAN = "ran"
O3_STATUS_EXECUTOR_FAILED = "executor_failed"

# Q1 的「全放行」谓词看哪几条 oracle（登记第六节 Q1-甲、Q1-乙、Q1-丙）：O1（对拍或崩溃判官任一红）、O2、O3。
Q1_ORACLES_STRICT = (ORACLE_ONE, ORACLE_TWO, ORACLE_THREE_STRICT)
Q1_ORACLES_PROVISIONAL = (ORACLE_ONE, ORACLE_TWO, ORACLE_THREE_PROVISIONAL)
Q1_ORACLES_COMPARISON_ONLY = (ORACLE_COMPARISON, ORACLE_TWO, ORACLE_THREE_PROVISIONAL)
Q1_ORACLES_WITHOUT_O3 = (ORACLE_ONE, ORACLE_TWO)
Q1_READINGS = (("q1_alpha", Q1_ORACLES_STRICT), ("q1_beta", Q1_ORACLES_PROVISIONAL), ("q1_gamma", Q1_ORACLES_COMPARISON_ONLY))


@dataclass
class RowAnalysis:
    """坏实现表里的一行跑完之后比对器手里的东西。`run` 是 None 表示这一行没有装置输出（编不过、锚点有问题、找不到、超时、包装自己的结局）。
    `o3` 是这一行 O3 的判定（{(流名, 状态号): (严格, 暂代)}），None 表示 O3 没跑；`o3_status` 是驱动记的 O3 一步的结局。"""

    number: int
    name: str
    driver_fields: dict[str, str]
    run: RunDirectory | None
    category: str
    touch: TouchReport | None
    o3: O3Verdicts | None
    o3_status: str = "-"
    o3_base_unrecoverable_streams: tuple[str, ...] = ()
    o3_error_states: int = 0
    o3_digest_mismatch_states: int = 0


def classify_row(driver_fields: dict[str, str], run: RunDirectory | None, touch: TouchReport | None,
                 device_streams: dict[str, dict[str, str]]) -> str:
    """一行的分类：驱动没造出可跑的装置（编不过、锚点、找不到）→ 超时与包装自己的结局 → 装置里实现 panic → 装置自己停机（S3、S4 对不上）→ 触达与否。"""
    status = driver_fields.get("status", "")
    if status in (CATEGORY_COMPILE_FAILED, CATEGORY_ANCHOR_NOT_UNIQUE, CATEGORY_NOT_FOUND):
        return status
    exit_code = int(driver_fields.get("device_exit", "0") or 0)
    if exit_code == TIMEOUT_EXIT_CODE:
        return CATEGORY_TIMEOUT
    if exit_code in WRAPPER_EXIT_CODES:
        return CATEGORY_WRAPPER_ENDED
    for summary in device_streams.values():
        why = summary.get("not_exported_why", "-")
        if why.startswith(("implementation_crashed", "pipeline_panicked")):
            return CATEGORY_SELF_CRASH
    if exit_code == DEVICE_STOP_EXIT_CODE:
        return CATEGORY_DEVICE_STOP
    if run is None or touch is None:
        return CATEGORY_NOT_FOUND
    if touch.touched():
        return CATEGORY_COUNTED
    return CATEGORY_ONLY_IN_THE_IGNORED_PASS if touch.only_in_the_ignored_pass() else CATEGORY_UNTOUCHED


def category_counts(rows: list[RowAnalysis]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        counts[row.category] = counts.get(row.category, 0) + 1
    return counts


def counted_rows(rows: list[RowAnalysis]) -> list[RowAnalysis]:
    return [row for row in rows if row.category == CATEGORY_COUNTED]


def untouched_but_red_rows(rows: list[RowAnalysis]) -> list[RowAnalysis]:
    """没触达（逐字节同行为）而某条 oracle 判了红：与登记第一节读法 4「未触达的行哪条 oracle 都不该红」对不上，列出来交主 agent。O3 也算（跑了的话）。"""
    flagged = []
    for row in rows:
        if row.category not in (CATEGORY_UNTOUCHED, CATEGORY_ONLY_IN_THE_IGNORED_PASS) or row.run is None:
            continue
        if any(column_is_red(row.run, oracle, FIRST_BATCH, row.o3)
               for oracle in (ORACLE_ONE, ORACLE_TWO, ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL)):
            flagged.append(row)
    return flagged


def o3_state_counts(row: RowAnalysis) -> dict[str, int]:
    """这一行 O3 判定的分布（两批合计）：严格的在 / 不在 / 欠一句 / 出错，暂代的在 / 不在 / 出错。"""
    verdicts = list((row.o3 or {}).values())
    return {"strict_in": sum(pair[0] == "in" for pair in verdicts), "strict_not_in": sum(pair[0] == "not_in" for pair in verdicts),
            "strict_spec_gap": sum(pair[0] == "spec_gap" for pair in verdicts), "strict_error": sum(pair[0] == "error" for pair in verdicts),
            "provisional_in": sum(pair[1] == "in" for pair in verdicts), "provisional_not_in": sum(pair[1] == "not_in" for pair in verdicts),
            "provisional_error": sum(pair[1] == "error" for pair in verdicts)}


def first_differing_operation(baseline: RunDirectory, run: RunDirectory) -> str:
    """Q1 逐行列出的「写流从第几条 op 起与基线不同」：两条流各取 `ops.tsv` 第一处不同的行号（0 起），写流全同写 `same`、缺文件写 `unavailable`。"""
    from run_directory import read_rows
    import os
    parts = []
    for name in STREAM_NAMES:
        base_path, row_path = os.path.join(baseline.path, name, "ops.tsv"), os.path.join(run.path, name, "ops.tsv")
        if not (os.path.exists(base_path) and os.path.exists(row_path)):
            parts.append("unavailable")
            continue
        first, second = read_rows(base_path), read_rows(row_path)
        index = next((position for position, (left, right) in enumerate(zip(first, second)) if left != right), None)
        if index is None and len(first) != len(second):
            index = min(len(first), len(second))
        parts.append("same" if index is None else str(index))
    return "/".join(parts)


def row_lines(row: RowAnalysis, lines: ResultLines) -> None:
    """一行的分类、触达三样、矩阵（批 1、批 2）、O3 分布、轨迹。O3 没跑时 O3 那几列写 `not_run`。"""
    lines.result(f"name=row row={row.number} row_name={field(row.name)} category={row.category} "
                 f"substituted_from={field(row.driver_fields.get('substituted_from', '-'))} "
                 f"line_matches_the_registration={row.driver_fields.get('line_matches_the_registration', '-')}")
    if row.run is None or row.touch is None:
        return
    touch = row.touch
    lines.result(f"name=row_touch row={row.number} write_stream_differs={str(touch.write_stream_differs).lower()} "
                 f"call_outcomes_differ={str(touch.call_outcomes_differ).lower()} "
                 f"recovery_report_differs={str(touch.recovery_report_differs).lower()} "
                 f"ignored_pass_differs={str(touch.ignored_pass_differs).lower()} touched={str(touch.touched()).lower()}")
    unjudged, other_source = errored_calls_left_unjudged(row.run), not_applicable_states_with_another_source(row.run)
    lines.result(f"name=row_s5 row={row.number} errored_calls_left_unjudged={'/'.join(str(unjudged[name]) for name in STREAM_NAMES)} "
                 f"o1_not_applicable_states_with_another_source={'/'.join(str(other_source[name]) for name in STREAM_NAMES)}")
    counts = o3_state_counts(row)
    lines.result(f"name=row_o3 row={row.number} status={row.o3_status} base_unrecoverable_streams={','.join(row.o3_base_unrecoverable_streams) or 'none'} "
                 f"executor_error_states={row.o3_error_states} digest_mismatch_states={row.o3_digest_mismatch_states} "
                 f"dump_problem_lines={row.driver_fields.get('o3_dump_problem_lines', '-')} "
                 + " ".join(f"{key}={value}" for key, value in counts.items()) if row.o3 is not None else
                 f"name=row_o3 row={row.number} status={row.o3_status} o3=not_run")
    for batch in (FIRST_BATCH, SECOND_BATCH):
        calls_red = red_calls(row.run)
        crash_red = red_state_counts(row.run, ORACLE_CRASH_JUDGE, batch, row.o3)
        pool_red = red_state_counts(row.run, ORACLE_TWO, batch, row.o3)
        strict_red = red_state_counts(row.run, ORACLE_THREE_STRICT, batch, row.o3) if row.o3 is not None else None
        provisional_red = red_state_counts(row.run, ORACLE_THREE_PROVISIONAL, batch, row.o3) if row.o3 is not None else None
        not_applicable = crash_judge_not_applicable_counts(row.run, batch)
        columns = {oracle: column_is_red(row.run, oracle, batch, row.o3)
                   for oracle in (ORACLE_ONE, ORACLE_COMPARISON, ORACLE_CRASH_JUDGE, ORACLE_TWO,
                                  ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL)}
        lines.result(
            f"name=row_matrix row={row.number} batch={batch} "
            + " ".join(f"{oracle}={'not_run' if value is None else ('red' if value else 'quiet')}" for oracle, value in columns.items())
            + f" o1_comparison_red_calls={'/'.join(str(calls_red[name]) for name in STREAM_NAMES)}"
            + f" o1_crash_judge_red_states={'/'.join(str(crash_red[name]) for name in STREAM_NAMES)}"
            + f" o1_crash_judge_not_applicable_states={'/'.join(str(not_applicable[name]) for name in STREAM_NAMES)}"
            + f" o2_red_states={'/'.join(str(pool_red[name]) for name in STREAM_NAMES)}"
            + " o3_strict_red_states=" + ("not_run" if strict_red is None else "/".join(str(strict_red[name]) for name in STREAM_NAMES))
            + " o3_provisional_red_states=" + ("not_run" if provisional_red is None else "/".join(str(provisional_red[name]) for name in STREAM_NAMES))
            + f" layer0_or_record_checker_red={str(layer0_oracle_red_in_batch(row.run, batch)).lower()}")
    oracles_with_a_state_trajectory = [ORACLE_CRASH_JUDGE, ORACLE_TWO] + ([ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL] if row.o3 is not None else [])
    for name in STREAM_NAMES:
        stream = row.run.streams[name]
        compared_red, first_red_call = comparison_trajectory(stream)
        lines.result(f"name=trajectory row={row.number} stream={name} oracle={ORACLE_COMPARISON} red_calls={compared_red} "
                     f"first_red_call={first_red_call if first_red_call is not None else 'none'}")
        for oracle in oracles_with_a_state_trajectory:
            path = trajectory(stream, name, oracle, FIRST_BATCH, row.o3)
            segments_with_states = len(path.red_by_segment)
            lines.result(
                f"name=trajectory row={row.number} stream={name} oracle={oracle} total_red_states={path.total()} peak={path.peak} "
                f"positive_segments={path.positive_segments} of_segments={segments_with_states} last_segment_value={path.last_segment_value} "
                f"first_red_segment={path.first_red_segment if path.first_red_segment is not None else 'none'} "
                f"first_red_after_call={path.first_red_after_call if path.first_red_after_call is not None else 'none'} "
                f"single_point_segments={','.join(map(str, path.single_point_segments)) or 'none'} "
                f"red_by_segment={','.join(f'{segment}:{count}' for segment, count in path.red_by_segment if count) or 'none'}")


def quiet_rows_in_batch(counted: list[RowAnalysis], oracles: tuple[str, ...], batch: int) -> list[RowAnalysis]:
    return [row for row in counted if row.run is not None and all_quiet(row.run, oracles, batch, row.o3)]


def reading_verdict(quiet: list[RowAnalysis], counted: list[RowAnalysis], strict_reading_unreachable: bool, is_strict_reading: bool) -> str:
    """Q1 一种读法的结论字：有一行全放行 → 有；没有而 D < 30 → 不够判；没有、D ≥ 30，严格读法在 F7 触发时不报「没有」。"""
    if quiet:
        return "yes_a_row_is_quiet_on_every_oracle"
    if not coverage_enough_to_judge(len(counted)):
        return "not_enough_rows"
    if is_strict_reading and strict_reading_unreachable:
        return "not_reported_f7_strict_o3_reaches_almost_no_state"
    return "no_such_row"


def q1_lines(rows: list[RowAnalysis], lines: ResultLines, o3_available: bool, baseline: RunDirectory | None = None) -> None:
    """Q1：分母 D 与各类；O1 与 O2 都不红的行（O3 没跑时它是全放行行数的上界）；O3 跑了时甲、乙、丙、丁与覆盖。"""
    counts = category_counts(rows)
    counted = counted_rows(rows)
    # 包装自己的结局两次都撞（Z3）记进「超时」一类并单列：`wrapper_ended` 单列，F6 里与超时一起算
    table_failures = sum(counts.get(category, 0) for category in CATEGORIES_THAT_COUNT_AGAINST_THE_TABLE) + counts.get(CATEGORY_WRAPPER_ENDED, 0)
    lines.result(
        f"name=q1_denominator rows_run={len(rows)} counted_D={len(counted)} "
        + " ".join(f"{category}={counts.get(category, 0)}" for category in (
            CATEGORY_UNTOUCHED, CATEGORY_ONLY_IN_THE_IGNORED_PASS, CATEGORY_COMPILE_FAILED, CATEGORY_ANCHOR_NOT_UNIQUE,
            CATEGORY_NOT_FOUND, CATEGORY_SELF_CRASH, CATEGORY_TIMEOUT, CATEGORY_DEVICE_STOP, CATEGORY_WRAPPER_ENDED))
        + f" enough_to_judge_none_side={str(coverage_enough_to_judge(len(counted))).lower()}")
    lines.result(f"name=f6_table_failures rows_not_counted_for_the_table={table_failures} rows_run={len(rows)} "
                 f"more_than_half={str(len(rows) > 0 and table_failures * 2 > len(rows)).lower()}")
    silent = quiet_rows_in_batch(counted, Q1_ORACLES_WITHOUT_O3, FIRST_BATCH)
    lines.result(f"name=q1_o1_and_o2_both_quiet counted_rows={len(counted)} rows={len(silent)} "
                 f"row_numbers={','.join(str(row.number) for row in silent) or 'none'} o3={'run' if o3_available else 'not_run'}")
    silent_second = quiet_rows_in_batch(counted, Q1_ORACLES_WITHOUT_O3, SECOND_BATCH)
    unstable = sorted({row.number for row in silent} ^ {row.number for row in silent_second})
    lines.result(f"name=q1_o1_and_o2_both_quiet_batch_2 rows={len(silent_second)} rows_that_differ_from_batch_1={','.join(map(str, unstable)) or 'none'}")
    columns = (ORACLE_ONE, ORACLE_COMPARISON, ORACLE_CRASH_JUDGE, ORACLE_TWO) + ((ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL) if o3_available else ())
    lines.result(f"name=q1_column_red_rows counted_rows={len(counted)} "
                 + " ".join(f"{oracle}={sum(bool(column_is_red(row.run, oracle, FIRST_BATCH, row.o3)) for row in counted if row.run)}" for oracle in columns))
    for name in STREAM_NAMES:
        lines.result(f"name=q1_o1_not_applicable stream={name} counted_rows={len(counted)} batch_1_states="
                     f"{sum(crash_judge_not_applicable_counts(row.run, FIRST_BATCH)[name] for row in counted if row.run)}")
    flagged = untouched_but_red_rows(rows)
    lines.result(f"name=untouched_but_red rows={len(flagged)} row_numbers={','.join(str(row.number) for row in flagged) or 'none'}")
    if not o3_available:
        for label in ("q1_alpha", "q1_beta", "q1_gamma", "q1_o3_coverage", "q3_alpha", "q3_beta"):
            lines.result(f"name={label} status=not_run why=o3_not_run")
        return
    zero_coverage = [row for row in counted if row.run is not None and o3_judged_state_count(row.run, row.o3, 0) == 0]
    unreachable = strict_reading_is_unreachable(len(zero_coverage), len(counted))
    for label, oracles in Q1_READINGS:
        quiet = quiet_rows_in_batch(counted, oracles, FIRST_BATCH)
        second = quiet_rows_in_batch(counted, oracles, SECOND_BATCH)
        differ = sorted({row.number for row in quiet} ^ {row.number for row in second})
        lines.result(f"name={label} counted_rows={len(counted)} all_quiet_rows={len(quiet)} row_numbers={','.join(str(row.number) for row in quiet) or 'none'} "
                     f"batch_2_all_quiet_rows={len(second)} rows_that_differ_from_batch_1={','.join(map(str, differ)) or 'none'} "
                     f"verdict={reading_verdict(quiet, counted, unreachable, label == 'q1_alpha')}")
        for row in quiet:
            red_counts = {oracle: sum(red_state_counts(row.run, oracle, FIRST_BATCH, row.o3).values()) for oracle in (ORACLE_CRASH_JUDGE, ORACLE_TWO)}
            lines.result(f"name=q1_quiet_row reading={label} row={row.number} row_name={field(row.name)} "
                         f"o1_comparison_red_calls={sum(red_calls(row.run).values())} o1_crash_judge_red_states={red_counts[ORACLE_CRASH_JUDGE]} "
                         f"o2_red_states={red_counts[ORACLE_TWO]} o3_judged_states_strict={o3_judged_state_count(row.run, row.o3, 0)} "
                         f"o3_judged_states_provisional={o3_judged_state_count(row.run, row.o3, 1)} "
                         f"first_operation_that_differs_from_the_baseline={first_differing_operation(baseline, row.run) if baseline else 'no_baseline'} "
                         f"layer0_or_record_checker_red={str(layer0_oracle_red_in_batch(row.run, FIRST_BATCH)).lower()}")
    lines.result(f"name=q1_o3_coverage counted_rows={len(counted)} rows_with_no_strict_judgement={len(zero_coverage)} "
                 f"strict_reading_unreachable={str(unreachable).lower()}")
    layer0_on_quiet = sorted({row.number for _, oracles in Q1_READINGS[:2] for row in quiet_rows_in_batch(counted, oracles, FIRST_BATCH)
                              if layer0_oracle_red_in_batch(row.run, FIRST_BATCH)})
    lines.result(f"name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle={','.join(map(str, layer0_on_quiet)) or 'none'}")
    for label, strict in (("q3_alpha", True), ("q3_beta", False)):
        lines.result(q3_line(label, counted, strict))
        for row in counted:
            if row.run is None:
                continue
            first = red_columns(row.run, FIRST_BATCH, row.o3, strict)
            if first:
                lines.result(f"name=q3_row reading={label} row={row.number} red_columns={','.join(sorted(first))} "
                             f"alone_hold={','.join(oracles_that_alone_hold_the_row(first)) or 'none'} "
                             f"batch_2_red_columns={','.join(sorted(red_columns(row.run, SECOND_BATCH, row.o3, strict))) or 'none'}")


def q3_line(label: str, counted: list[RowAnalysis], strict_o3: bool) -> str:
    """Q3：被抓的行（至少一条红）逐行数红的列；只有一条红的行按那一条分组列出；批 2 上红的列与批 1 不同的行另列。"""
    only: dict[str, list[int]] = {}
    differing: list[int] = []
    for row in counted:
        if row.run is None:
            continue
        first = red_columns(row.run, FIRST_BATCH, row.o3, strict_o3)
        if first != red_columns(row.run, SECOND_BATCH, row.o3, strict_o3):
            differing.append(row.number)
        for holder in oracles_that_alone_hold_the_row(first):
            only.setdefault(holder, []).append(row.number)
    holders = [f"only_{holder}={','.join(map(str, numbers))}" for holder, numbers in sorted(only.items())] or ["none_held_by_a_single_oracle"]
    return (f"name={label} caught_rows={sum(bool(red_columns(row.run, FIRST_BATCH, row.o3, strict_o3)) for row in counted if row.run)} "
            + " ".join(holders) + f" rows_whose_red_columns_differ_between_batch_1_and_2={','.join(map(str, differing)) or 'none'}")


def timing_lines(rows: list[RowAnalysis], lines: ResultLines) -> None:
    """Q4-单行：每行拷仓与改坏、编、装置两条流、O3 执行器、比对的耗时；最小、中位、最大。耗时行不进入库产物。"""
    series: dict[str, list[int]] = {}
    for row in rows:
        for key in ("copy_ns", "compile_ns", "device_ns", "o3_ns"):
            value = row.driver_fields.get(key)
            if value not in (None, "", "-"):
                series.setdefault(key, []).append(int(value))
        lines.timing(f"name=row_time row={row.number} " + " ".join(
            f"{key}={row.driver_fields.get(key, '-')}" for key in ("copy_ns", "compile_ns", "compile_kind", "device_ns", "o3_ns")))
    for key, values in sorted(series.items()):
        ordered = sorted(values)
        lines.timing(f"name=row_time_summary column={key} rows={len(ordered)} min_ns={ordered[0]} "
                     f"median_ns={ordered[(len(ordered) - 1) // 2]} max_ns={ordered[-1]}")


def analyse_row(number: int, name: str, driver_fields: dict[str, str], baseline: RunDirectory, run: RunDirectory | None,
                device_streams: dict[str, dict[str, str]], o3: O3Verdicts | None, o3_status: str = "-",
                o3_base_unrecoverable_streams: tuple[str, ...] = (), o3_error_states: int = 0, o3_digest_mismatch_states: int = 0) -> RowAnalysis:
    touch = None if run is None else touch_report(baseline, run)
    return RowAnalysis(number, name, driver_fields, run, classify_row(driver_fields, run, touch, device_streams), touch, o3, o3_status,
                       o3_base_unrecoverable_streams, o3_error_states, o3_digest_mismatch_states)
