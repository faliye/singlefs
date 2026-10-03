"""E170 第 1 次重跑第二段：比对器——冻结的第二版执行器在重导的批 1、批 2 上的判定（执行器产物里的 name=state 行）对今天实现的判定
（重导的 impl-verdicts.tsv），逐态比（问题单 Q2；登记 `research/prompts/e170-r1-prereg.md` 第六节 Q2-a、Q2-b、Q2-c、Q2-d、Q2-e、Q2-f、Q2-g、
Q2-pc，第八节 G1、G2，第十节 F6、F7、F11）。从第一次的比对器（`e170_o3_spec_executor/comparator_q2_implementation_verdicts.py`）拷来改：
Q2 的判定取严格跑法（登记读法 5），暂代跑法只当附带（Q2-g）；批 3 不判（登记第十一节「不适用」）；Q2-pc 取批 1 里严格跑法给出判定的第一态。

不改执行器、不重判：执行器那一侧只读执行器产物里的 name=state 行与 name=path_counts 行；实现那一侧只读 impl-verdicts.tsv；
状态的批号与段号读 states.tsv。「实现都放行」= 池级 checker 一条 Violated 都没有（pool_checker_red 列为 -），且记录核对器两项都为假
（原登记第一节读法 6，重跑登记读法 7 照原样）。两份 oracle 的结论不进 Q2，这里不读它们。

跑法（在执行器目录里）：
  python3 comparator_q2_implementation_verdicts.py --dumps <重导目录> --frozen-verdicts <执行器产物> --output <新文件>
      [--freeze-reference <第一段产物>] [--primary-field strict]
文件名以 comparator 起头，不进冻结的 sha256（executor_main.FILES_OUTSIDE_THE_FREEZE）。--primary-field 只给单测式自查用（拿第一次的产物试跑
比对器的逻辑），正式跑取默认值 strict。
"""

from __future__ import annotations

import argparse
import os
import re
import sys

import executor_main
from comparator_result_lines import ResultLines
from crash_dump import read_tab_separated_rows

IMPLEMENTATION_VERDICTS_FILE = "impl-verdicts.tsv"
STATES_FILE = "states.tsv"
VERDICTS_GIVEN = ("in", "not_in")
LANDED_PAIR = re.compile(r"^\((\d+),(\d+),")
HISTOGRAM_FIELDS = ("path", "stop", "refusal")  # Q2-e：执行器 name=state 行里的轨迹字段，按值数一遍


def implementation_passes_both(columns: list[str]) -> bool:
    """impl-verdicts.tsv 一行：状态号、两种政策的根与 outcome、red_items、池级 checker 判红的不变量、记录核对器两项、耗时。"""
    return columns[6] == "-" and columns[7] == "false" and columns[8] == "false"


def read_frozen_lines(frozen_verdicts: str, line_name: str) -> list[dict[str, str]]:
    rows = []
    with open(frozen_verdicts, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith(f"name={line_name} "):
                rows.append(dict(token.split("=", 1) for token in line.split() if "=" in token))
    return rows


def read_freeze_lines(path: str) -> list[str]:
    """冻结行：name=freeze file=… sha256=… 与 name=freeze_seed …，原样逐行（去行尾换行；允许前面带 E7RESULT 前缀）。"""
    lines = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            text = line.rstrip("\n").removeprefix("E7RESULT ")
            if text.startswith("name=freeze ") or text.startswith("name=freeze_seed "):
                lines.append(text)
    return lines


def read_stream_tables(dumps: str) -> tuple[dict[str, list[list[str]]], dict[str, dict[int, list[str]]]]:
    """每条流：states.tsv 的行（状态号、层 0 序号、段号、批号、落地向量、摘要）与按状态号排好的 impl-verdicts.tsv 行。"""
    states = {}
    implementation = {}
    for stream_name in executor_main.STREAM_NAMES:
        directory = os.path.join(dumps, stream_name)
        states[stream_name] = read_tab_separated_rows(os.path.join(directory, STATES_FILE))
        implementation[stream_name] = {int(columns[0]): columns
                                       for columns in read_tab_separated_rows(os.path.join(directory, IMPLEMENTATION_VERDICTS_FILE))}
    return states, implementation


def landed_pair_of_executor(landed: str) -> str:
    match = LANDED_PAIR.match(landed)
    return "none" if match is None else f"{match.group(1)},{match.group(2)}"


def disagreements(frozen: dict[tuple[str, int], dict[str, str]], states: dict[str, list[list[str]]],
                  implementation: dict[str, dict[int, list[str]]], batch: int, verdict_field: str) -> dict:
    """一批里逐态比：执行器给出在 / 不在的态，「在」而实现没有两判都放行、或「不在」而实现两判都放行，记一处不同。
    verdict_field 取 strict（严格跑法，Q2-a、Q2-c、Q2-d）或 provisional（暂代跑法，Q2-g）。
    not_given 是这一列没给出在 / 不在（规约欠一句或出错）的态数，这些态不进逐态比（登记读法 6）。"""
    different: list[tuple[str, int, int]] = []
    compared = 0
    not_given = 0
    not_given_states: list[tuple[str, int]] = []
    landed_pairs_differ = 0
    by_segment: dict[tuple[str, int], int] = {}
    for stream_name, rows in states.items():
        for columns in rows:
            state_number, segment, batch_number = int(columns[0]), int(columns[2]), int(columns[3])
            if batch_number != batch:
                continue
            by_segment.setdefault((stream_name, segment), 0)
            executor = frozen[(stream_name, state_number)]
            implementation_row = implementation[stream_name][state_number]
            if landed_pair_of_executor(executor["landed"]) != implementation_row[1]:
                landed_pairs_differ += 1
            verdict = executor[verdict_field]
            if verdict not in VERDICTS_GIVEN:
                not_given += 1
                not_given_states.append((stream_name, state_number))
                continue
            compared += 1
            if (verdict == "in") != implementation_passes_both(implementation_row):
                different.append((stream_name, state_number, segment))
                by_segment[(stream_name, segment)] += 1
    return {"different": different, "compared": compared, "not_given": not_given, "not_given_states": not_given_states,
            "landed_pairs_differ": landed_pairs_differ, "by_segment": by_segment}


def flipped_copy(implementation: dict[str, dict[int, list[str]]], stream_name: str, state_number: int) -> dict[str, dict[int, list[str]]]:
    """Q2-pc：实现判定表复制一份，翻掉一态的放行（放行的改成池级 checker 判红一条，不放行的改成两判都放行）。"""
    copied = {name: {number: list(columns) for number, columns in rows.items()} for name, rows in implementation.items()}
    columns = copied[stream_name][state_number]
    if implementation_passes_both(columns):
        columns[6] = "pc_flipped"
    else:
        columns[6], columns[7], columns[8] = "-", "false", "false"
    return copied


def positive_control_differences(frozen, states, implementation, verdict_field: str) -> tuple[int, str]:
    """比对器阳性对照：翻掉批 1 里这一列给出判定的第一态的放行再比，两次的不同之处取对称差，必须恰 1 处（登记第六节 Q2-pc）。
    批 1 里这一列一态也没给出判定时没有可翻的态：差数记 0、翻的态记 none，Q2-pc 不过（≠ 1，V5）。"""
    first_given = None
    for stream_name in executor_main.STREAM_NAMES:
        for columns in states[stream_name]:
            if int(columns[3]) == 1 and frozen[(stream_name, int(columns[0]))][verdict_field] in VERDICTS_GIVEN:
                if first_given is None or (executor_main.STREAM_NAMES.index(stream_name), int(columns[0])) < \
                        (executor_main.STREAM_NAMES.index(first_given[0]), first_given[1]):
                    first_given = (stream_name, int(columns[0]))
    if first_given is None:
        return 0, "none"
    original = set(disagreements(frozen, states, implementation, 1, verdict_field)["different"])
    flipped = set(disagreements(frozen, states, flipped_copy(implementation, *first_given), 1, verdict_field)["different"])
    return len(original ^ flipped), f"{first_given[0]}:{first_given[1]}"


def trajectory_text(by_segment: dict[tuple[str, int], int], stream_name: str) -> str:
    """登记第八节「轨迹」：沿段序的不同态数——峰值、为正的段数、最后一段的值。"""
    counts = [count for (name, _segment), count in sorted(by_segment.items()) if name == stream_name]
    return (f"segments={len(counts)} peak={max(counts, default=0)} positive_segments={sum(count > 0 for count in counts)} "
            f"last_segment_value={counts[-1] if counts else 0}")


def cross_tabulation(frozen, states, implementation, batch: int, stream_name: str, verdict_field: str) -> str:
    cells: dict[str, int] = {}
    for columns in states[stream_name]:
        if int(columns[3]) != batch:
            continue
        state_number = int(columns[0])
        executor = frozen[(stream_name, state_number)][verdict_field]
        passes = "impl_pass" if implementation_passes_both(implementation[stream_name][state_number]) else "impl_red"
        key = f"exec_{executor}_{passes}"
        cells[key] = cells.get(key, 0) + 1
    return " ".join(f"{key}={value}" for key, value in sorted(cells.items()))


def trace_histogram(frozen, states, batch: int, stream_name: str, verdict_field: str) -> str:
    """Q2-e 的补充：批里这一列给出判定的态，执行器轨迹字段（path、stop、refusal，以及名字里带 slot 的字段）按值各数一遍。"""
    counts: dict[str, int] = {}
    for columns in states[stream_name]:
        if int(columns[3]) != batch:
            continue
        executor = frozen[(stream_name, int(columns[0]))]
        if executor[verdict_field] not in VERDICTS_GIVEN:
            continue
        for key, value in executor.items():
            if key in HISTOGRAM_FIELDS or "slot" in key:
                counts[f"{key}={value}"] = counts.get(f"{key}={value}", 0) + 1
    return " ".join(f"{key}*{value}" for key, value in sorted(counts.items())) or "none"


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 1 次重跑第二段：冻结的执行器判定对实现判定逐态比（Q2）")
    parser.add_argument("--dumps", required=True, help="重导目录（research/target/e170-second-run-dumps）")
    parser.add_argument("--frozen-verdicts", required=True, help="冻结的执行器在重导目录上跑出的产物（name=state 行）")
    parser.add_argument("--output", required=True, help="产物路径；已存在就拒绝")
    parser.add_argument("--freeze-reference", help="第一段产物：它的 name=freeze 行与执行器产物里的逐行相同才算同一份执行器")
    parser.add_argument("--primary-field", default="strict", choices=("strict", "provisional"),
                        help="Q2-a、Q2-d、Q2-pc 读哪一列；正式跑取 strict，provisional 只给拿第一次的产物自查比对器逻辑用")
    arguments = parser.parse_args(argument_list)
    primary, secondary = arguments.primary_field, ("provisional" if arguments.primary_field == "strict" else "strict")
    lines = ResultLines(arguments.output)
    frozen_rows = read_frozen_lines(arguments.frozen_verdicts, "state")
    frozen = {(row["stream"], int(row["state"])): row for row in frozen_rows}
    states, implementation = read_stream_tables(arguments.dumps)
    lines.result(f"name=config segment=r1_s2 mode=q2_comparator streams={','.join(executor_main.STREAM_NAMES)} "
                 f"frozen_state_lines={len(frozen_rows)} primary_field={primary} secondary_field={secondary}")
    if arguments.freeze_reference:
        executor_freeze = read_freeze_lines(arguments.frozen_verdicts)
        reference_freeze = read_freeze_lines(arguments.freeze_reference)
        lines.result(f"name=freeze_check executor_freeze_lines={len(executor_freeze)} reference_freeze_lines={len(reference_freeze)} "
                     f"same={str(executor_freeze == reference_freeze and len(executor_freeze) > 0).lower()}")
    for stream_name in executor_main.STREAM_NAMES:
        state_numbers = {int(columns[0]) for columns in states[stream_name]}
        frozen_numbers = {number for (name, number) in frozen if name == stream_name}
        lines.result(f"name=coverage stream={stream_name} states={len(state_numbers)} implementation_rows={len(implementation[stream_name])} "
                     f"frozen_rows={len(frozen_numbers)} states_without_frozen_row={len(state_numbers - frozen_numbers)} "
                     f"states_without_implementation_row={len(state_numbers - set(implementation[stream_name]))}")
    measured = {}
    for batch in (1, 2):
        for field in (primary, secondary):
            result = disagreements(frozen, states, implementation, batch, field)
            measured[(batch, field)] = result
            label = {(1, primary): "q2a", (2, primary): "q2d", (1, secondary): "q2g", (2, secondary): "q2g_batch2"}[(batch, field)]
            lines.result(f"name=q2 measure={label} batch={batch} verdict_field={field} compared={result['compared']} "
                         f"not_given={result['not_given']} different={len(result['different'])} "
                         f"q2f_landed_pairs_differ={result['landed_pairs_differ']}")
            for stream_name, state_number in (result["not_given_states"] if field == primary else []):
                executor = frozen[(stream_name, state_number)]
                lines.result(f"name=q2c_state measure={label} stream={stream_name} state={state_number} "
                             f"verdict={executor[field]} gaps={executor['gaps']}")
            for stream_name, state_number, segment in result["different"]:
                executor = frozen[(stream_name, state_number)]
                lines.result(f"name=q2b_state measure={label} stream={stream_name} state={state_number} segment={segment} "
                             f"executor_{field}={executor[field]} landed={executor['landed']} "
                             f"chosen_root={executor['chosen_root']} last_counter={executor['last_counter']} stop={executor['stop']} "
                             f"path={executor.get('path', '-')} gaps={executor['gaps']} "
                             f"implementation={'|'.join(implementation[stream_name][state_number][1:9])}")
            for stream_name in executor_main.STREAM_NAMES:
                lines.result(f"name=q2_trajectory measure={label} stream={stream_name} {trajectory_text(result['by_segment'], stream_name)}")
                lines.result(f"name=q2_cross measure={label} stream={stream_name} "
                             f"{cross_tabulation(frozen, states, implementation, batch, stream_name, field)}")
                if field == primary:
                    lines.result(f"name=q2e_trace_histogram measure={label} stream={stream_name} "
                                 f"{trace_histogram(frozen, states, batch, stream_name, field)}")
    for row in read_frozen_lines(arguments.frozen_verdicts, "path_counts"):
        lines.result("name=q2e_from_executor_output " + " ".join(f"{key}={value}" for key, value in row.items() if key != "name"))
    differences, flipped_state = positive_control_differences(frozen, states, implementation, primary)
    lines.result(f"name=q2_positive_control verdict_field={primary} flipped_state={flipped_state} differences={differences} "
                 f"exactly_one={str(differences == 1).lower()}")
    q2a, q2d = measured[(1, primary)], measured[(2, primary)]
    lines.result(f"name=verdict q2a_different={len(q2a['different'])} q2a_compared={q2a['compared']} q2c_not_given={q2a['not_given']} "
                 f"q2d_different={len(q2d['different'])} q2d_compared={q2d['compared']} q2d_not_given={q2d['not_given']} "
                 f"q2g_different={len(measured[(1, secondary)]['different'])} q2_pc_differences={differences} "
                 f"q2_pc_exactly_one={str(differences == 1).lower()}")
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
