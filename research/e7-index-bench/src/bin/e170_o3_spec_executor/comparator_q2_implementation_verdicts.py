"""E170（O3规约执行器第一版） 第三段：比对器——冻结那一份执行器判定（第二段产物，暂代跑法）对实现判定（第一段 impl-verdicts.tsv），逐态比
（问题单 Q2；登记第六节 Q2-a、Q2-b、Q2-c、Q2-d、Q2-e、Q2-f、Q2-pc，第八节 G1、G2，第十节 F6、F7）。

不改执行器、不重判：执行器那一侧只读第二段产物里的 name=state 行与 name=path_counts 行；实现那一侧只读 impl-verdicts.tsv；
状态的批号与段号读 states.tsv。「实现都放行」= 池级 checker 一条 Violated 都没有（pool_checker_red 列为 -），且记录核对器两项都为假
（登记第一节读法 6）。两份 oracle 的结论不进 Q2（读法 6 末句），这里不读它们。

跑法（在本目录）：
  python3 comparator_q2_implementation_verdicts.py --dumps <导出目录> --frozen-verdicts <第二段产物> --output <新文件>
文件名以 comparator 起头，不进冻结的 sha256（executor_main.FILES_OUTSIDE_THE_FREEZE）。
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


def implementation_passes_both(columns: list[str]) -> bool:
    """impl-verdicts.tsv 一行：状态号、两种政策的根与 outcome、red_items、池级 checker 判红的不变量、记录核对器两项、耗时。"""
    return columns[6] == "-" and columns[7] == "false" and columns[8] == "false"


def read_frozen_lines(frozen_verdicts: str, line_name: str) -> list[dict[str, str]]:
    rows = []
    with open(frozen_verdicts, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith(f"name={line_name} "):
                rows.append(dict(token.split("=", 1) for token in line.split()))
    return rows


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
    verdict_field 取 provisional（暂代跑法，Q2-a、Q2-d）或 strict（严格跑法，Q2-c）。"""
    different: list[tuple[str, int, int]] = []
    compared = 0
    not_given = 0
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
                continue
            compared += 1
            if (verdict == "in") != implementation_passes_both(implementation_row):
                different.append((stream_name, state_number, segment))
                by_segment[(stream_name, segment)] += 1
    return {"different": different, "compared": compared, "not_given": not_given,
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


def positive_control_differences(frozen, states, implementation) -> tuple[int, str]:
    """比对器阳性对照：翻掉批 1 流一第一态的放行再比，两次的不同之处取对称差，必须恰 1 处（登记第六节 Q2-pc）。"""
    first_stream = executor_main.STREAM_NAMES[0]
    first_state = min(int(columns[0]) for columns in states[first_stream] if int(columns[3]) == 1)
    original = set(disagreements(frozen, states, implementation, 1, "provisional")["different"])
    flipped = set(disagreements(frozen, states, flipped_copy(implementation, first_stream, first_state), 1, "provisional")["different"])
    return len(original ^ flipped), f"{first_stream}:{first_state}"


def trajectory_text(by_segment: dict[tuple[str, int], int], stream_name: str) -> str:
    """登记第八节「轨迹」：沿段序的不同态数——峰值、为正的段数、最后一段的值。"""
    counts = [count for (name, _segment), count in sorted(by_segment.items()) if name == stream_name]
    return (f"segments={len(counts)} peak={max(counts, default=0)} positive_segments={sum(count > 0 for count in counts)} "
            f"last_segment_value={counts[-1] if counts else 0}")


def cross_tabulation(frozen, states, implementation, batch: int, stream_name: str) -> str:
    cells: dict[str, int] = {}
    for columns in states[stream_name]:
        if int(columns[3]) != batch:
            continue
        state_number = int(columns[0])
        executor = frozen[(stream_name, state_number)]["provisional"]
        passes = "impl_pass" if implementation_passes_both(implementation[stream_name][state_number]) else "impl_red"
        key = f"exec_{executor}_{passes}"
        cells[key] = cells.get(key, 0) + 1
    return " ".join(f"{key}={value}" for key, value in sorted(cells.items()))


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第三段：冻结的执行器判定对实现判定逐态比（Q2）")
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--frozen-verdicts", required=True, help="第二段产物（6 进程那一份，name=state 行）")
    parser.add_argument("--output", required=True, help="产物路径；已存在就拒绝")
    arguments = parser.parse_args(argument_list)
    lines = ResultLines(arguments.output)
    frozen_rows = read_frozen_lines(arguments.frozen_verdicts, "state")
    frozen = {(row["stream"], int(row["state"])): row for row in frozen_rows}
    states, implementation = read_stream_tables(arguments.dumps)
    lines.result(f"name=config segment=s3 mode=q2_comparator streams={','.join(executor_main.STREAM_NAMES)} "
                 f"frozen_state_lines={len(frozen_rows)} verdict_field_provisional=provisional verdict_field_strict=strict")
    for stream_name in executor_main.STREAM_NAMES:
        state_numbers = {int(columns[0]) for columns in states[stream_name]}
        frozen_numbers = {number for (name, number) in frozen if name == stream_name}
        lines.result(f"name=coverage stream={stream_name} states={len(state_numbers)} implementation_rows={len(implementation[stream_name])} "
                     f"frozen_rows={len(frozen_numbers)} states_without_frozen_row={len(state_numbers - frozen_numbers)} "
                     f"states_without_implementation_row={len(state_numbers - set(implementation[stream_name]))}")
    for batch in (1, 2, 3):
        for verdict_field in ("provisional", "strict"):
            result = disagreements(frozen, states, implementation, batch, verdict_field)
            label = {(1, "provisional"): "q2a", (1, "strict"): "q2c", (2, "provisional"): "q2d",
                     (2, "strict"): "q2d_strict", (3, "provisional"): "batch3_incidental",
                     (3, "strict"): "batch3_strict_incidental"}[(batch, verdict_field)]
            lines.result(f"name=q2 measure={label} batch={batch} verdict_field={verdict_field} compared={result['compared']} "
                         f"not_given={result['not_given']} different={len(result['different'])} "
                         f"q2f_landed_pairs_differ={result['landed_pairs_differ']}")
            for stream_name, state_number, segment in result["different"]:
                executor = frozen[(stream_name, state_number)]
                lines.result(f"name=q2b_state measure={label} stream={stream_name} state={state_number} segment={segment} "
                             f"executor_{verdict_field}={executor[verdict_field]} landed={executor['landed']} "
                             f"chosen_root={executor['chosen_root']} last_counter={executor['last_counter']} stop={executor['stop']} "
                             f"gaps={executor['gaps']} implementation={'|'.join(implementation[stream_name][state_number][1:9])}")
            if verdict_field == "provisional":
                for stream_name in executor_main.STREAM_NAMES:
                    lines.result(f"name=q2_trajectory measure={label} stream={stream_name} {trajectory_text(result['by_segment'], stream_name)}")
                    lines.result(f"name=q2_cross measure={label} stream={stream_name} "
                                 f"{cross_tabulation(frozen, states, implementation, batch, stream_name)}")
    for row in read_frozen_lines(arguments.frozen_verdicts, "path_counts"):
        lines.result("name=q2e_from_segment_two " + " ".join(f"{key}={value}" for key, value in row.items() if key != "name"))
    differences, flipped_state = positive_control_differences(frozen, states, implementation)
    lines.result(f"name=q2_positive_control flipped_state={flipped_state} differences={differences} "
                 f"exactly_one={str(differences == 1).lower()}")
    q2a = disagreements(frozen, states, implementation, 1, "provisional")
    q2d = disagreements(frozen, states, implementation, 2, "provisional")
    lines.result(f"name=verdict q2a_different={len(q2a['different'])} q2a_compared={q2a['compared']} "
                 f"q2d_different={len(q2d['different'])} q2d_compared={q2d['compared']} q2_pc_differences={differences} "
                 f"q2_pc_exactly_one={str(differences == 1).lower()}")
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
