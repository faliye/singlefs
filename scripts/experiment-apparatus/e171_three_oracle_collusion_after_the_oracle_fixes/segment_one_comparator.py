"""E171 第 1 次重跑第一段的比对器（登记第六节 Q1–Q5 的结果行、第七节 A1、A8、A11、第八节轨迹与几何、作废条款 Z1）。
第六节 Q4-（ii）逐态清单（`name=q4_counter_branch_state`）与第八节 Q4、Q5 轨迹（`name=q4_trajectory`、`name=q4_trajectory_by_inode_creation`、
`name=q5_trajectory_cell`、`name=q5_trajectory`）接在 Q5 判定之后、判决行之前；判决行与它之前各行的写法不因它们而变。

只读：两次基线的导出目录、逐行驱动的输出根（`row-<序号>/` 里的 `row-result.tsv`、`dump/`、`o3.out`）、基线的执行器产物、Q5 两个取样点的产物、
阳性对照各趟的产物；写一份新产物（已存在就拒绝）。不读 s4 的任何产物（登记读法 11）。

跑法（在本目录）：
  python3 segment_one_comparator.py --baseline <导出目录> [--second-baseline <导出目录>] [--baseline-o3 <执行器产物>]
      [--rows-root <输出根>] [--q5-main <产物>] [--q5-reverse <产物>] [--control <对照名>=<产物> …] --output <新文件>
      --dumps-root <这一趟的导出根> --product-prefix <这一趟产物名的前缀> --product-suffix=<这一趟产物名的后缀（以 - 起头，要写成 = 连着）>
导出目录不在导出根之下、产物名不带这一趟的前缀与后缀的，退 2、不写产物（登记修订 8 第 8.5 节第 1 条：两趟各读各的）。
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
from collections.abc import Callable
from dataclasses import dataclass

import collusion_matrix as matrix
import frozen_o3
from result_lines import ResultLines, field
from run_directory import (STREAM_NAMES, TOUCH_STREAM_NAMES, RunDirectory, file_sha256, load_run_directory, read_rows,
                           result_lines_named)

# 问题单 Q1 的七行、Q2 点名的七行、Q3 的那一行、Q4 的两行（整表编号，登记第一节）
Q1_ROWS = (15, 23, 42, 242, 269, 270, 271)
Q2_NAMED_ROWS = (49, 110, 155, 161, 162, 174, 252)
Q3_ROW = 125
Q4_ROWS = (242, 269)
Q1_ORACLES_STRICT = (matrix.ORACLE_ONE, matrix.ORACLE_TWO, matrix.ORACLE_THREE_STRICT)
Q1_ORACLES_PROVISIONAL = (matrix.ORACLE_ONE, matrix.ORACLE_TWO, matrix.ORACLE_THREE_PROVISIONAL)
# Z1：基线两次逐字节相同的文件（`impl-verdicts.tsv` 去掉耗时那一列）
Z1_FILES = ("base-sectors.bin", "ops.tsv", "ops-payload.bin", "calls.tsv", "torn.tsv", "torn-payload.bin", "states.tsv", "quick-tier.tsv",
            "oracle-verdicts.tsv", "call-outcomes.tsv", "writable-mount-probe.tsv")
# 第七节 A8：每样阳性对照在基线那一趟的精确数（结果行里取哪个数、要等于几）
A8_EXPECTED = {
    "O1-对拍": ("o1_comparison_red_calls", 1),
    "O1-对拍-卸载": ("o1_comparison_red_calls", 1),
    "O1-崩溃判官": ("o1_crash_judge_red_states", 3),
    "RefFS": ("o1_reference_filesystem_red_states", 3),
    "O2": ("o2_red_states", 1),
    "见证": ("counter_branch_states", 1),
}


def read_o3(path: str | None) -> matrix.O3Verdicts | None:
    if path is None or not os.path.exists(path):
        return None
    return frozen_o3.read_executor_output(path).verdicts


def lines_without_the_last_column(path: str) -> bytes:
    with open(path, encoding="utf-8") as handle:
        return "".join(line.rstrip("\n").rsplit("\t", 1)[0] + "\n" for line in handle).encode("utf-8")


def z1_lines(lines: ResultLines, first: str, second: str) -> int:
    """Z1：两次基线逐文件比（`call-return-values.tsv` 拿掉两次就不稳的内存对象之后比，读法 4）。交回不同的文件数。"""
    differing = 0
    first_run, second_run = load_run_directory(first), load_run_directory(second)
    unstable = matrix.unstable_memory_objects(first_run, second_run)
    lines.result(f"name=z1_unstable_memory_objects names={field(','.join(sorted(unstable)) or '-')}")
    for stream in STREAM_NAMES:
        for name in Z1_FILES + ("impl-verdicts.tsv",):
            left, right = os.path.join(first, stream, name), os.path.join(second, stream, name)
            if not os.path.exists(left) and not os.path.exists(right):
                continue
            if not (os.path.exists(left) and os.path.exists(right)):
                same = False
            elif name == "impl-verdicts.tsv":
                same = lines_without_the_last_column(left) == lines_without_the_last_column(right)
            else:
                same = file_sha256(left) == file_sha256(right)
            differing += not same
            lines.result(f"name=z1 stream={stream} file={name} identical={str(same).lower()}")
        stable_differences = matrix.return_value_differences(first_run, second_run, unstable, (stream,))
        differing += bool(stable_differences)
        lines.result(f"name=z1 stream={stream} file=call-return-values.tsv_without_unstable_objects identical={str(not stable_differences).lower()}")
    return differing


def a1_lines(lines: ResultLines, baseline: RunDirectory, o3: matrix.O3Verdicts | None, line_name: str = "a1") -> int:
    """A1：基线（或空变异那一行，`line_name` 换成 `a1_empty_mutation`）三条流批 1 批 2 上 O1 三个子列、O2、O3 甲乙判红都是 0 处。交回判红的处数。"""
    reds = 0
    for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
        comparison = sum(matrix.red_calls(baseline).values()) if batch == matrix.FIRST_BATCH else 0
        counts = {name: sum(matrix.red_state_counts(baseline, name, batch, o3).values())
                  for name in (matrix.SUBCOLUMN_CRASH_JUDGE, matrix.SUBCOLUMN_REFERENCE_FILESYSTEM, matrix.ORACLE_TWO,
                               matrix.ORACLE_THREE_STRICT, matrix.ORACLE_THREE_PROVISIONAL)}
        reds += comparison + sum(counts.values())
        lines.result(f"name={line_name} batch={batch} o1_comparison_red_calls={comparison} " + " ".join(f"{field(key)}={value}" for key, value in counts.items())
                     + f" o3_ran={str(o3 is not None).lower()}")
    return reds


EMPTY_MUTATION_ROW = 0  # 逐行驱动 `--empty-mutation` 那一行的目录是 row-0


def empty_mutation_a1(lines: ResultLines, baseline: RunDirectory, run: RunDirectory, o3: matrix.O3Verdicts | None) -> tuple[int, bool]:
    """第七节 A1 的空变异那一半：判红的处数（同基线的数法）与触达否（读法 4，空变异必须未触达）。"""
    reds = a1_lines(lines, run, o3, "a1_empty_mutation")
    touched = matrix.touch_report(baseline, run).touched()
    lines.result(f"name=a1_empty_mutation_touch row={EMPTY_MUTATION_ROW} streams_present={len(matrix.streams_present(run))} touched={str(touched).lower()}")
    return reds, touched


def a8_lines(lines: ResultLines, controls: dict[str, list[str]]) -> int:
    """A8：每样阳性对照（几份产物的计数相加，比如流一流二那一份加流三那一份）与登记的精确数比。交回不等的对照数。"""
    failing = 0
    for control, paths in sorted(controls.items()):
        if control not in A8_EXPECTED:
            lines.result(f"name=a8 control={field(control)} products={len(paths)} checked=false why=judged_from_the_call_lines_by_hand")
            continue
        key, expected = A8_EXPECTED[control]
        observed = sum(int(verdict.get(key, "0")) for path in paths for verdict in result_lines_named(path, "verdict"))
        holds = observed == expected
        failing += not holds
        lines.result(f"name=a8 control={field(control)} products={len(paths)} counted={key} expected={expected} observed={observed} holds={str(holds).lower()}")
    return failing


def write_stream_indexes(stream_directory: str) -> list[int]:
    """`ops.tsv` 里每次写（屏障除外）在录制流里的下标，按写序。"""
    return [int(columns[0]) for columns in read_rows(os.path.join(stream_directory, "ops.tsv")) if columns[2] != "barrier"]


def segment_closing_indexes(stream_directory: str) -> list[int | None]:
    """照 `segments::SegmentClosingRule` 在 `ops.tsv` 上重切一遍（比对器自己的一份）：每一段由录制流第几步关上，流尾没关上的那一段是 None。"""
    closing: list[int | None] = []
    writes_in_the_segment = 0
    unreleased: set[str] = set()
    for columns in read_rows(os.path.join(stream_directory, "ops.tsv")):
        index, device, kind = int(columns[0]), columns[1], columns[2]
        if kind in ("write", "write_zeroes"):
            writes_in_the_segment += 1
            unreleased.add(device)
            continue
        if kind == "write_fua":
            writes_in_the_segment += 1
        unreleased.discard(device)
        if writes_in_the_segment > 0 and not unreleased:
            closing.append(index)
            writes_in_the_segment = 0
    if writes_in_the_segment > 0:
        closing.append(None)
    return closing


def calls_reaching_each_write(first_operations: list[int], write_indexes: list[int]) -> list[int]:
    """每个写（按写序）落在哪次调用里：第一条 op 不晚于这个写的调用里序号最大的那一个，没有就是 -1。
    写的下标严格递增（`ops.tsv` 按录制次序），所以这个序号随写序不减；不递增就抛错，不静默算错。"""
    if any(later <= earlier for earlier, later in zip(write_indexes, write_indexes[1:])):
        raise ValueError("ops.tsv 里写的下标不是严格递增")
    calls_by_first_operation = sorted(enumerate(first_operations), key=lambda pair: pair[1])
    reaching, highest_call, next_call = [], -1, 0
    for write_index in write_indexes:
        while next_call < len(calls_by_first_operation) and calls_by_first_operation[next_call][1] <= write_index:
            highest_call = max(highest_call, calls_by_first_operation[next_call][0])
            next_call += 1
        reaching.append(highest_call)
    return reaching


def last_touched_call_of_a_landing(landing: str, call_of_write: list[int]) -> int | None:
    """一态落了的写（落地串里不是 `0` 的位）所在调用的最大序号；一个都没落是 None。`call_of_write` 随写序不减，
    最大值就在最后一个落了的写上（与逐个写取最大值同值，`test_segment_one.py` 的 A11 用例拿逐个取的写法对拍）。"""
    last_landed = len(landing.rstrip("0")) - 1
    return None if last_landed < 0 else call_of_write[last_landed]


def recomputed_applicable_and_voting(run: RunDirectory, stream: str) -> tuple[int, int, int]:
    """A11：比对器从 `calls.tsv`、`states.tsv`、`ops.tsv` 现算的 (取样态数, O1-模型崩溃判官适用的态数, RefFS 投了票的态数)。
    适用：这一态落了的写都属于模型停下的那次调用之前的调用（读法 6）；投票：崩溃时刻（关上这一段的那一步，段没关上算在全部步之后）
    在 RefFS 翻译停下的那一步之前（unlink 写了盘、回退没返回 Ok 那一步停）。"""
    stream_run = run.streams[stream]
    directory = os.path.join(run.path, stream)
    calls = read_rows(os.path.join(directory, "calls.tsv"))
    first_operations = [int(columns[2]) for columns in calls]
    return_points = [int(columns[3]) for columns in calls]
    stopped = stream_run.model_stopped_at_call()
    write_indexes = write_stream_indexes(directory)
    closing = segment_closing_indexes(directory)
    translation_stop = next((index for index, columns in enumerate(calls)
                             if columns[1] in ("unlink_intent", "unlink_step") or (columns[1] == "roll_back" and columns[4] != "ok")), None)
    call_of_write = calls_reaching_each_write(first_operations, write_indexes)
    applicable = voting = 0
    for state in stream_run.states:
        touched = last_touched_call_of_a_landing(state.landing, call_of_write)
        applicable += stopped is None or touched is None or touched < stopped
        closing_index = closing[state.segment] if state.segment < len(closing) else None
        if translation_stop is None:
            voting += 1
            continue
        if closing_index is None:
            continue
        step = next((index for index, return_point in enumerate(return_points) if return_point > closing_index), None)
        voting += step is not None and step < translation_stop
    return len(stream_run.states), applicable, voting


def a11_lines(lines: ResultLines, baseline: RunDirectory) -> int:
    """A11：装置写下的（`oracle-verdicts.tsv` 里不是 `na` 的、不是弃权的）与比对器现算的相等。交回不等的流数。"""
    failing = 0
    for stream in matrix.streams_present(baseline):
        states, applicable, voting = recomputed_applicable_and_voting(baseline, stream)
        device_applicable = sum(state.crash_judge not in ("na",) for state in baseline.streams[stream].states)
        device_voting = sum(state.reference_filesystem in ("green", "red") for state in baseline.streams[stream].states)
        holds = (applicable, voting) == (device_applicable, device_voting)
        failing += not holds
        lines.result(f"name=a11 stream={stream} states={states} crash_judge_applicable_device={device_applicable} crash_judge_applicable_recomputed={applicable} "
                     f"reference_filesystem_voting_device={device_voting} reference_filesystem_voting_recomputed={voting} holds={str(holds).lower()}")
    return failing


def read_row_result(row_directory: str) -> dict[str, str]:
    path = os.path.join(row_directory, "row-result.tsv")
    if not os.path.exists(path):
        return {}
    return {columns[0]: columns[1] for columns in read_rows(path) if len(columns) > 1}


def q1_lines(lines: ResultLines, baseline: RunDirectory, rows_root: str, rows: tuple[int, ...] = Q1_ROWS) -> dict[str, str]:
    """Q1（第六节 Q1-有效、Q1-甲、Q1-乙、Q1-两流、Q1-落点）：七行逐行三列、O1 三个子列各自、三条流各自，批 1 定判、批 2 另报（第八节）。
    交回 {读法: 判定}。"""
    verdicts = {}
    for reading, oracles in (("strict", Q1_ORACLES_STRICT), ("provisional", Q1_ORACLES_PROVISIONAL)):
        for streams_label, streams in (("three_streams", STREAM_NAMES), ("two_streams", TOUCH_STREAM_NAMES)):
            for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
                quiet_rows, invalid_rows = [], []
                for number in rows:
                    row_directory = os.path.join(rows_root, f"row-{number}")
                    result = read_row_result(row_directory)
                    if result.get("status") != "ok" or result.get("device_exit") not in ("0", "3"):
                        invalid_rows.append(number)
                        continue
                    run = load_run_directory(os.path.join(row_directory, "dump"))
                    complete = all(run.streams[name].present() for name in streams)
                    touched = matrix.touch_report(baseline, run).touched()
                    if not (complete and touched):
                        invalid_rows.append(number)
                        continue
                    o3 = read_o3(os.path.join(row_directory, "o3.out"))
                    if matrix.all_quiet(run, oracles, batch, o3, streams):
                        quiet_rows.append(number)
                    if reading == "strict" and streams_label == "three_streams":
                        row_column_lines(lines, number, run, o3, batch, streams)
                # F3：没触达、没跑完的行记「不够判」、不当被抓也不当全放行；其余各行照判，总判定写「k 行有效」。
                valid_rows = len(rows) - len(invalid_rows)
                verdict = "yi" if quiet_rows else ("jia" if not invalid_rows else f"jia_over_{valid_rows}_valid_rows")
                if reading == "strict" and streams_label == "three_streams" and batch == matrix.FIRST_BATCH:
                    verdicts["q1"] = verdict
                lines.result(f"name=q1 reading={reading} streams={streams_label} batch={batch} rows={len(rows)} invalid_rows={field(','.join(map(str, invalid_rows)) or '-')} "
                             f"all_quiet_rows={field(','.join(map(str, quiet_rows)) or '-')} verdict={verdict}")
    return verdicts


def row_column_lines(lines: ResultLines, number: int, run: RunDirectory, o3: matrix.O3Verdicts | None, batch: int,
                     streams: tuple[str, ...]) -> None:
    """一行的落点（Q1-落点）：O1 三个子列、O2、O3 两读在每条流这一批里红的数（O1-对拍按调用），与各列的轨迹（峰值、为正段数、期末值、单点红）。"""
    for stream in matrix.streams_present(run, streams):
        comparison = sum(call.is_a_model_disagreement() for call in run.streams[stream].calls)
        parts = [f"o1_comparison_red_calls={comparison}"]
        for column in (matrix.SUBCOLUMN_CRASH_JUDGE, matrix.SUBCOLUMN_REFERENCE_FILESYSTEM, matrix.ORACLE_TWO,
                       matrix.ORACLE_THREE_STRICT, matrix.ORACLE_THREE_PROVISIONAL):
            path = matrix.trajectory(run.streams[stream], stream, column, batch, o3)
            parts.append(f"{field(column)}_red={sum(count for _, count in path.red_by_segment)}"
                         f" {field(column)}_peak={path.peak} {field(column)}_positive_segments={path.positive_segments}"
                         f" {field(column)}_last={path.last_segment_value} {field(column)}_single_points={len(path.single_point_segments)}")
        lines.result(f"name=q1_row row={number} stream={stream} batch={batch} " + " ".join(parts))


def q2_named_row_lines(lines: ResultLines, baseline: RunDirectory, rows_root: str, unstable: frozenset[str],
                       rows: tuple[int, ...] = Q2_NAMED_ROWS) -> None:
    """Q2-七行（第六节，第一段先报）：C672 点名的七行，流一流二（Q2 的口径）与流三（第八节，另报）各自：触达否、交回值哪几个顶层字段不同、
    哪个内存对象不同、胶水交给模型的观测同不同、O1-对拍红没红。"""
    for number in rows:
        row_directory = os.path.join(rows_root, f"row-{number}")
        result = read_row_result(row_directory)
        if result.get("status") != "ok" or not os.path.isdir(os.path.join(row_directory, "dump")):
            lines.result(f"name=q2_named_row row={number} ran=false status={field(result.get('status', 'missing'))}")
            continue
        run = load_run_directory(os.path.join(row_directory, "dump"))
        touched = matrix.touch_report(baseline, run).touched()
        for streams_label, streams in (("streams_1_2", TOUCH_STREAM_NAMES), ("stream_3", STREAM_NAMES[2:])):
            differences = matrix.return_value_differences(baseline, run, unstable, streams)
            comparison_red = sum(matrix.red_calls(run, streams).values())
            observed_differs = sum(difference.observation_differs for difference in differences)
            lines.result(f"name=q2_named_row row={number} streams={streams_label} touched={str(touched).lower()} calls_with_a_difference={len(differences)} "
                         f"calls_whose_observation_differs={observed_differs} calls_whose_observation_is_the_same={len(differences) - observed_differs} "
                         f"o1_comparison_red_calls={comparison_red}")
            for difference in differences:
                lines.result(f"name=q2_named_row_call row={number} stream={difference.stream} call={difference.call_index} "
                             f"fields={field(','.join(difference.differing_fields) or '-')} objects={field(','.join(difference.differing_objects) or '-')} "
                             f"observation_differs={str(difference.observation_differs).lower()}")


def q3_lines(lines: ResultLines, baseline: RunDirectory, rows_root: str, number: int = Q3_ROW) -> str:
    """Q3（第六节 Q3-有效、Q3-判）：那一行流一、流二上第一处调用 panic 之前 O1-对拍判过的调用数 c、截断之后判了的态数、触达否。交回判定。"""
    row_directory = os.path.join(rows_root, f"row-{number}")
    result = read_row_result(row_directory)
    if result.get("status") != "ok" or not os.path.isdir(os.path.join(row_directory, "dump")):
        lines.result(f"name=q3 row={number} ran=false status={field(result.get('status', 'missing'))} verdict=not_enough")
        return "not_enough"
    run = load_run_directory(os.path.join(row_directory, "dump"))
    judged = matrix.judged_calls_before_the_first_panic(run)
    recovery_panics = sum(state.recovery_panic != "-" for name in TOUCH_STREAM_NAMES for state in run.streams[name].states)
    touched = matrix.touch_report(baseline, run).touched()
    states_after_the_truncation = sum(len(run.streams[name].states) for name in TOUCH_STREAM_NAMES)
    if judged is None:
        verdict = "not_enough_f6"
    elif judged >= matrix.MINIMUM_JUDGED_CALLS_BEFORE_THE_FIRST_PANIC:
        verdict = "jia" if touched else "judgeable_but_untouched"
    else:
        verdict = "yi"
    lines.result(f"name=q3 row={number} call_panic={str(judged is not None).lower()} judged_calls_before_the_first_panic={judged if judged is not None else '-'} "
                 f"states_judged={states_after_the_truncation} recovery_panic_states={recovery_panics} touched={str(touched).lower()} verdict={verdict}")
    return verdict


def q4_lines(lines: ResultLines, trips: dict[str, RunDirectory]) -> dict[str, str]:
    """Q4（第六节 Q4-有效、Q4-（ii）、Q4-（i）、Q4-对账）：每一趟、每条流、每批的探针计数；判定按趟分开写（基线、第 242、269 行）。"""
    verdicts = {}
    for trip, run in trips.items():
        counter_total = legal_total = 0
        cells_without_a_probe = 0
        for stream in matrix.streams_present(run):
            for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
                counts = matrix.probe_counts(run, stream, batch)
                counter_total += counts.counter_branch
                legal_total += counts.counter_branch_judged_legal
                cells_without_a_probe += counts.probed == 0
                lines.result(f"name=q4 trip={field(trip)} stream={stream} batch={batch} probed={counts.probed} enough={str(counts.probed > 0).lower()} "
                             f"counter_branch={counts.counter_branch} counter_branch_judged_legal={counts.counter_branch_judged_legal} "
                             f"counter_branch_judged_illegal={counts.counter_branch_judged_illegal} undecidable_branch={counts.undecidable_branch} "
                             f"witness_refusals_without_either_branch={counts.witness_refusals_without_either_branch} counter_branch_mounted={counts.counter_branch_mounted}")
        # Q4-有效：每格（流 × 批）至少探过 1 态那一格才判；有一格 0 态，这一趟的「没有这种态」不判（甲不判，乙丙照数照判）。
        verdict = matrix.q4_verdict(counter_total, legal_total)
        if verdict == "jia" and (cells_without_a_probe or len(matrix.streams_present(run)) < len(STREAM_NAMES)):
            verdict = "not_enough"
        verdicts[trip] = verdict
        lines.result(f"name=q4_trip trip={field(trip)} counter_branch={counter_total} counter_branch_judged_legal={legal_total} "
                     f"cells_without_a_probe={cells_without_a_probe} verdict={verdict}")
    return verdicts


def q5_lines(lines: ResultLines, product: str | None, sampling_point: str) -> str:
    """Q5（第六节 Q5-有效、Q5-判、Q5-阴性）：从装置那一趟的 `name=q5` 行读 L 与四个数，按登记判；那一趟没跑（`not_run`）记不够判。"""
    if product is None or not os.path.exists(product):
        lines.result(f"name=q5_judged sampling_point={sampling_point} product=missing verdict=not_enough")
        return "not_enough"
    found = result_lines_named(product, "q5")
    if not found or "not_run" in found[0]:
        why = found[0].get("not_run", "missing") if found else "missing"
        lines.result(f"name=q5_judged sampling_point={sampling_point} not_run={field(why)} verdict=not_enough")
        return "not_enough"
    q5 = found[0]
    lost = int(q5["lost_the_confirmed_version"])
    verdict = matrix.q5_verdict(lost, int(q5["reference_filesystem_red"]), int(q5["model_crash_judge_red"]), int(q5["both_green"]),
                                int(q5["reference_filesystem_abstained"]))
    negative = int(q5["not_lost_o1_red"])
    lines.result(f"name=q5_judged sampling_point={sampling_point} lost={lost} reference_filesystem_red={q5['reference_filesystem_red']} "
                 f"model_crash_judge_red={q5['model_crash_judge_red']} both_green={q5['both_green']} reference_filesystem_abstained={q5['reference_filesystem_abstained']} "
                 f"q5_negative_o1_red={negative} a10_states_equal_the_product={q5['a10_states_equal_the_product']} "
                 f"reference_filesystem_judge_panic={field(q5.get('reference_filesystem_judge_panic', '-'))} "
                 f"model_crash_judge_judge_panic={field(q5.get('model_crash_judge_judge_panic', '-'))} verdict={verdict}")
    return verdict


CREATE_INODE_CALL_LABEL = "create_inode"  # 流三逐次建 inode 那一种调用（`calls.tsv` 第 2 列）


@dataclass(frozen=True)
class SegmentCalls:
    """一条流每一段的写由哪次调用发出（段号即下标），与建 inode 的调用是第几次（从 1 数，只收建 inode 的调用）。"""

    call_of_segment: tuple[int, ...]
    creation_ordinal_of_call: dict[int, int]
    create_inode_calls: int


def segment_calls_from_the_export(run: RunDirectory, stream: str) -> SegmentCalls:
    """从导出的 `ops.tsv`、`calls.tsv` 现算：段照比对器自己重切的那一份（`segment_closing_indexes`，与装置的段数由 A9 互核），
    一段的调用取这一段最后一个写所在的调用（`calls_reaching_each_write`）。"""
    directory = os.path.join(run.path, stream)
    calls = read_rows(os.path.join(directory, "calls.tsv"))
    write_indexes = write_stream_indexes(directory)
    call_of_write = calls_reaching_each_write([int(columns[2]) for columns in calls], write_indexes)
    call_of_segment = []
    write_position = 0
    for closing_index in segment_closing_indexes(directory):
        writes_of_segment = []
        while write_position < len(write_indexes) and (closing_index is None or write_indexes[write_position] <= closing_index):
            writes_of_segment.append(write_position)
            write_position += 1
        if not writes_of_segment:
            raise ValueError(f"{directory} 有一段没有写：段按「有写才关」切，不该出现")
        call_of_segment.append(call_of_write[writes_of_segment[-1]])
    creation_ordinal_of_call: dict[int, int] = {}
    for index, columns in enumerate(calls):
        if columns[1] == CREATE_INODE_CALL_LABEL:
            creation_ordinal_of_call[index] = len(creation_ordinal_of_call) + 1
    return SegmentCalls(tuple(call_of_segment), creation_ordinal_of_call, len(creation_ordinal_of_call))


def call_of_a_segment(calls: SegmentCalls, segment: int) -> int | None:
    """这一段的写由哪次调用发出。段号等于段数的是全持久态（装置把它记在最后一段之后），交回 None（崩在全部调用都返回之后）；更大的抛错。"""
    if segment == len(calls.call_of_segment):
        return None
    if segment > len(calls.call_of_segment):
        raise ValueError(f"段号 {segment} 超出比对器重切出的 {len(calls.call_of_segment)} 段")
    return calls.call_of_segment[segment]


AFTER_EVERY_CALL = "after_every_call"


SegmentCallsOf = Callable[[RunDirectory, str], SegmentCalls]


def q4_counter_branch_state_lines(lines: ResultLines, trips: dict[str, RunDirectory],
                                  segment_calls_of: SegmentCallsOf = segment_calls_from_the_export) -> int:
    """第六节 Q4-（ii）逐态清单：（ii）为真的每一态一行——趟、流、批、序号、落地向量摘要（落地串的 sha256）、c_见证、所选版本、挂载结局、拒因、
    第二截判定与没满足的条件；另带状态号、段号、这一段由哪次调用发出（流三另带第几次建 inode）。交回列出的态数。"""
    listed = 0
    for trip, run in trips.items():
        for stream in matrix.streams_present(run):
            for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
                found = matrix.counter_branch_states(run.streams[stream], batch)
                if not found:
                    continue
                calls = segment_calls_of(run, stream)
                for item in found:
                    state, probe = item.state, item.probe
                    call = call_of_a_segment(calls, state.segment)
                    verdict, unmet = matrix.second_crop_verdict_and_unmet_condition(probe.second_crop)
                    landing_sha256 = hashlib.sha256(state.landing.encode("utf-8")).hexdigest()
                    lines.result(f"name=q4_counter_branch_state trip={field(trip)} stream={stream} batch={batch} ordinal={state.ordinal} "
                                 f"state={state.state_number} segment={state.segment} call_of_the_segment={AFTER_EVERY_CALL if call is None else call} "
                                 f"inode_creation={'-' if call is None else calls.creation_ordinal_of_call.get(call, '-')} landing_sha256={landing_sha256} "
                                 f"image_digest={field(state.digest)} counter_sentence={field(probe.counter_branch)} witnessed_counter={probe.witnessed_counter} "
                                 f"selected_version={field(probe.selected_version)} mount={field(probe.mount)} refusal_member={field(probe.refusal_member)} "
                                 f"refusal_reason_on_the_d23_list={field(probe.refusal_on_the_d23_list)} undecidable_branch={str(probe.undecidable_branch).lower()} "
                                 f"second_crop={field(verdict)} unmet_condition={field(unmet)}")
                    listed += 1
    return listed


def series_fields(prefix: str, cells: tuple[tuple[int, int], ...]) -> str:
    """一串计数在结果行里的写法：为正的格、峰值与它第一次出现的格、为正的格数、末格的值。"""
    series = matrix.count_series(cells)
    return (f"{prefix}_by_cell={matrix.positive_cells_text(cells)} {prefix}_peak={series.peak} {prefix}_peak_at={series.peak_at if series.peak_at is not None else '-'} "
            f"{prefix}_positive_cells={series.positive_cells} {prefix}_last={series.last_cell_value}")


def q4_trajectory_lines(lines: ResultLines, trips: dict[str, RunDirectory],
                        segment_calls_of: SegmentCallsOf = segment_calls_from_the_export) -> None:
    """第八节 Q4：三趟每条流每批，按段号排开的（ii）态数与其中判合法的态数（格 = 这一批探过的态所在的段）；流三另按「第几次建 inode 发布」排开
    （格 = 第 1 至最后一次建 inode，没有态的格记 0；段不在建 inode 调用里的（ii）态单列，全持久态那一段也单列）。"""
    for trip, run in trips.items():
        for stream in matrix.streams_present(run):
            for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
                counter, legal = matrix.counter_branch_by_segment(run.streams[stream], batch)
                segments = [segment for segment, _ in counter]
                lines.result(f"name=q4_trajectory trip={field(trip)} stream={stream} batch={batch} cells=segments segments={len(segments)} "
                             f"first_segment={segments[0] if segments else '-'} last_segment={segments[-1] if segments else '-'} "
                             + series_fields("counter_branch", counter) + " " + series_fields("judged_legal", legal))
        third = STREAM_NAMES[2]
        if not run.streams[third].present():
            continue
        calls = segment_calls_of(run, third)
        for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
            by_creation = {ordinal: [0, 0] for ordinal in range(1, calls.create_inode_calls + 1)}
            outside, after_every_call = [0, 0], [0, 0]
            counter, legal = matrix.counter_branch_by_segment(run.streams[third], batch)
            for (segment, counter_states), (_, legal_states) in zip(counter, legal):
                call = call_of_a_segment(calls, segment)
                cell = after_every_call if call is None else by_creation.get(calls.creation_ordinal_of_call.get(call, 0), outside)
                cell[0] += counter_states
                cell[1] += legal_states
            lines.result(f"name=q4_trajectory_by_inode_creation trip={field(trip)} stream={third} batch={batch} create_inode_calls={calls.create_inode_calls} "
                         f"counter_branch_outside_create_inode_calls={outside[0]} judged_legal_outside_create_inode_calls={outside[1]} "
                         f"counter_branch_after_every_call={after_every_call[0]} judged_legal_after_every_call={after_every_call[1]} "
                         + series_fields("counter_branch", tuple((ordinal, cell[0]) for ordinal, cell in by_creation.items())) + " "
                         + series_fields("judged_legal", tuple((ordinal, cell[1]) for ordinal, cell in by_creation.items())))


def q5_trajectory_lines(lines: ResultLines, product: str | None, sampling_point: str) -> None:
    """第八节 Q5：L 个态按「末段里落了几个写」排开的 RefFS 红态数与模型崩溃判官红态数（装置 `name=q5_trajectory` 行，只出有 L 态的格；
    这里补成第 0 至末段写数格，没有的记 0），报峰值、为正的格数、末格（末段全落）的值；各格之和与 `name=q5` 那一行的 L 与两个红数核一遍。"""
    found = result_lines_named(product, "q5") if product is not None and os.path.exists(product) else []
    q5 = found[0] if found else None
    if q5 is None or "not_run" in q5:
        lines.result(f"name=q5_trajectory sampling_point={sampling_point} ran=false")
        return
    last_segment_writes = int(q5["last_segment_writes"])
    cells: dict[int, tuple[int, int, int]] = {}
    for cell in result_lines_named(product, "q5_trajectory"):
        if cell["sampling_point"] != q5["sampling_point"]:
            continue
        landed = int(cell["writes_landed_in_the_last_segment"])
        if landed in cells or not 0 <= landed <= last_segment_writes:
            raise ValueError(f"{product} 的 q5_trajectory 格 {landed} 重复或不在 0 至 {last_segment_writes} 里")
        cells[landed] = (int(cell["lost_states"]), int(cell["reference_filesystem_red"]), int(cell["model_crash_judge_red"]))
    filled = tuple((landed, cells.get(landed, (0, 0, 0))) for landed in range(last_segment_writes + 1))
    for landed, (lost, reference_red, model_red) in filled:
        lines.result(f"name=q5_trajectory_cell sampling_point={sampling_point} writes_landed_in_the_last_segment={landed} lost_states={lost} "
                     f"reference_filesystem_red={reference_red} model_crash_judge_red={model_red}")
    sums = tuple(sum(counts[which] for _, counts in filled) for which in range(3))
    adds_up = sums == (int(q5["lost_the_confirmed_version"]), int(q5["reference_filesystem_red"]), int(q5["model_crash_judge_red"]))
    lines.result(f"name=q5_trajectory sampling_point={sampling_point} cells={len(filled)} last_segment_writes={last_segment_writes} "
                 f"cells_add_up_to_the_q5_line={str(adds_up).lower()} "
                 + series_fields("reference_filesystem_red", tuple((landed, counts[1]) for landed, counts in filled)) + " "
                 + series_fields("model_crash_judge_red", tuple((landed, counts[2]) for landed, counts in filled)))


def o3_third_stream_coverage(lines: ResultLines, baseline: RunDirectory, o3: matrix.O3Verdicts | None) -> str:
    """第六节「流三覆盖（O3）」与 F7：基线流三批 1、批 2 的取样态里，O3 严格给出「在 / 不在」的态数。0 走 F7；O3 没跑记 not_run。"""
    stream = STREAM_NAMES[2]
    sampled = [state for state in baseline.streams[stream].states if state.batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH)]
    given = sum(matrix.o3_verdict_of(o3, stream, state, 0) in (matrix.O3_VERDICT_IN, matrix.O3_VERDICT_NOT_IN) for state in sampled)
    f7 = "not_run" if o3 is None else str(given == 0).lower()
    lines.result(f"name=o3_third_stream_coverage stream={stream} sampled_states={len(sampled)} strict_in_or_not_in_states={given} f7={f7}")
    return f7


def inputs_outside_this_run(dumps_root: str, product_prefix: str, product_suffix: str, directories: list[str | None],
                            products: list[str | None]) -> list[str]:
    """登记修订 8 第 8.5 节第 1 条：比对器只读自己那一趟的导出根与产物前缀。导出目录不在 `dumps_root` 之下、产物文件名不以
    `product_prefix` 起头或不以 `product_suffix` 收尾的，逐个交回；空表示都在这一趟里。"""
    root = os.path.realpath(dumps_root)
    outside = [path for path in directories
               if path is not None and os.path.commonpath([root, os.path.realpath(path)]) != root]
    outside += [path for path in products
                if path is not None and not (os.path.basename(path).startswith(product_prefix) and os.path.basename(path).endswith(product_suffix))]
    return outside


# 本次未跑（用户 2026-10-18 定：拿留存产物与那一趟的导出比的，缺了一律报本次未跑，不静默跳过、不当绿、不判红）：
# 给的输入有不在的，每一样往 stderr 打一行 NOT_RUN_ARCHIVED_PRODUCT test=<比对器> reason=<哪一样不在>，退 77，一行产物都不写。
# 写法照 scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER；segment_two_comparator.py 用同一个。
NOT_RUN_MARKER = "NOT_RUN_ARCHIVED_PRODUCT"
NOT_RUN_EXIT_CODE = 77


def report_missing_inputs(test_name: str, paths: list[str | None]) -> bool:
    """给了而不在的输入（导出目录、行输出根、产物）：每一样往 stderr 打一行本次未跑、交 True；都在（或没给）交 False。"""
    missing = [path for path in paths if path is not None and not os.path.exists(path)]
    for path in missing:
        print(f"{NOT_RUN_MARKER} test={test_name} reason={path} 不在，这一项本次未跑；留存产物从 git 历史取回，那一趟的导出照登记"
              " research/prompts/e171-r1-prereg.md 重出到草稿目录、经驱动的环境变量指过去", file=sys.stderr)
    return bool(missing)


def parse_controls(items: list[str]) -> dict[str, list[str]]:
    """`--control 名=产物` 几次 → {名: [产物…]}（同一样对照分几份产物跑的，几份的计数相加）。"""
    controls: dict[str, list[str]] = {}
    for item in items:
        name, separator, path = item.partition("=")
        if not separator:
            raise ValueError(f"--control 要写成 名=产物：{item}")
        controls.setdefault(name, []).append(path)
    return controls


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 第 1 次重跑第一段的比对器")
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--second-baseline")
    parser.add_argument("--baseline-o3")
    parser.add_argument("--rows-root")
    parser.add_argument("--q5-main")
    parser.add_argument("--q5-reverse")
    parser.add_argument("--control", action="append", default=[])
    parser.add_argument("--output", required=True)
    parser.add_argument("--dumps-root", required=True)
    parser.add_argument("--product-prefix", required=True)
    parser.add_argument("--product-suffix", required=True)
    arguments = parser.parse_args(argument_list)
    controls = parse_controls(arguments.control)
    if report_missing_inputs("segment_one_comparator.py", [arguments.dumps_root, arguments.baseline, arguments.second_baseline, arguments.rows_root,
                                                          arguments.baseline_o3, arguments.q5_main, arguments.q5_reverse]
                             + [path for paths in controls.values() for path in paths]):
        return NOT_RUN_EXIT_CODE
    outside = inputs_outside_this_run(arguments.dumps_root, arguments.product_prefix, arguments.product_suffix,
                                      [arguments.baseline, arguments.second_baseline, arguments.rows_root],
                                      [arguments.baseline_o3, arguments.q5_main, arguments.q5_reverse]
                                      + [path for paths in controls.values() for path in paths])
    if outside:
        print("比对器只读这一趟的导出根与产物前缀（登记修订 8 第 8.5 节第 1 条），不在这一趟里的输入：" + "，".join(outside), file=sys.stderr)
        return 2
    lines = ResultLines(arguments.output)
    lines.result(f"name=config segment=s1 comparator=segment_one baseline={field(arguments.baseline)} second_baseline={field(arguments.second_baseline or '-')} "
                 f"rows_root={field(arguments.rows_root or '-')} dumps_root={field(arguments.dumps_root)} product_prefix={field(arguments.product_prefix)} "
                 f"product_suffix={field(arguments.product_suffix)}")
    baseline = load_run_directory(arguments.baseline)
    baseline_o3 = read_o3(arguments.baseline_o3)
    baseline_reds = a1_lines(lines, baseline, baseline_o3)
    f7 = o3_third_stream_coverage(lines, baseline, baseline_o3)
    a11_failing = a11_lines(lines, baseline)
    z1_differing = z1_lines(lines, arguments.baseline, arguments.second_baseline) if arguments.second_baseline else None
    unstable = (matrix.unstable_memory_objects(baseline, load_run_directory(arguments.second_baseline))
                if arguments.second_baseline else frozenset())
    a8_failing = a8_lines(lines, controls)
    verdicts: dict[str, str] = {}
    empty_reds, empty_touched = "not_run", "not_run"
    if arguments.rows_root:
        empty_directory = os.path.join(arguments.rows_root, f"row-{EMPTY_MUTATION_ROW}")
        empty_result = read_row_result(empty_directory)
        if empty_result.get("status") == "ok" and os.path.isdir(os.path.join(empty_directory, "dump")):
            reds, touched = empty_mutation_a1(lines, baseline, load_run_directory(os.path.join(empty_directory, "dump")),
                                              read_o3(os.path.join(empty_directory, "o3.out")))
            empty_reds, empty_touched = str(reds), str(touched).lower()
        else:
            lines.result(f"name=a1_empty_mutation_touch row={EMPTY_MUTATION_ROW} ran=false status={field(empty_result.get('status', 'missing'))}")
        verdicts.update(q1_lines(lines, baseline, arguments.rows_root))
        q2_named_row_lines(lines, baseline, arguments.rows_root, unstable)
        verdicts["q3"] = q3_lines(lines, baseline, arguments.rows_root)
        trips = {"baseline": baseline}
        for number in Q4_ROWS:
            dump = os.path.join(arguments.rows_root, f"row-{number}", "dump")
            if os.path.isdir(dump):
                trips[f"row_{number}"] = load_run_directory(dump)
        verdicts.update({f"q4_{trip}": verdict for trip, verdict in q4_lines(lines, trips).items()})
    else:
        trips = {"baseline": baseline}
        verdicts.update({f"q4_{trip}": verdict for trip, verdict in q4_lines(lines, trips).items()})
    verdicts["q5_main"] = q5_lines(lines, arguments.q5_main, "main")
    verdicts["q5_reverse"] = q5_lines(lines, arguments.q5_reverse, "reverse")
    # 第六节 Q4-（ii）逐态清单与第八节 Q4、Q5 轨迹（第一段比对器交回之后补的量，只多出这几种新行，接在 Q5 判定之后、判决行之前）
    q4_counter_branch_state_lines(lines, trips)
    q4_trajectory_lines(lines, trips)
    q5_trajectory_lines(lines, arguments.q5_main, "main")
    q5_trajectory_lines(lines, arguments.q5_reverse, "reverse")
    q5_stable = verdicts["q5_main"] == verdicts["q5_reverse"]
    lines.result("name=verdict " + " ".join(f"{key}={value}" for key, value in sorted(verdicts.items()))
                 + f" q5_same_at_both_sampling_points={str(q5_stable).lower()} a1_baseline_red={baseline_reds} a8_failing_controls={a8_failing}"
                 + f" a11_failing_streams={a11_failing} z1_differing_files={'not_run' if z1_differing is None else z1_differing} f7={f7}"
                 + f" a1_empty_mutation_red={empty_reds} empty_mutation_touched={empty_touched}")
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
