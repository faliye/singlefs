"""E201（三oracle合谋今树整表） 整表矩阵（跑前登记 research/prompts/e201-preregistration.md 第五节 5.5、第八节轨迹、第六节 Q1-今天独有与 Q2-整表）：
整表每行、每条流、每批、每列（O1 三个子列、O2、O3 严格与暂代）的红数（O1-对拍按调用，其余按态）、可判数（O1-对拍是 O1-对拍判过的调用数，其余是这一批的态数）、
峰值、为正段数、期末值、单点数（collusion_matrix.trajectory），与每一判定列的整表漏判集（读法 9 第一条的定义：登记读法批 1 计入分母、这一列在流一流二批 1 上一处都没红）。
描述量，不进 Q1、Q2 的判定；r1 一侧写「git 里没有逐列读数」。

只读行输出根（`row-<序号>/dump/` 与 `o3.out`）与今天的 T1；合并那一步由 full_table_run.py 在进程里调 matrix_lines。单测见 test_whole_table_matrix.py。
"""

from __future__ import annotations

import os

import e171_research_side  # noqa: F401  （把 E171 研究一侧放进 import 路径）
import collusion_matrix as matrix
from q2_misses import CATEGORY_COUNTED, row_categories
from result_lines import ResultLines, field
from run_directory import load_run_directory
from segment_one_comparator import read_o3

STATE_COLUMNS = (matrix.SUBCOLUMN_CRASH_JUDGE, matrix.SUBCOLUMN_REFERENCE_FILESYSTEM, matrix.ORACLE_TWO,
                 matrix.ORACLE_THREE_STRICT, matrix.ORACLE_THREE_PROVISIONAL)
DECISION_COLUMNS = (matrix.ORACLE_ONE, matrix.ORACLE_TWO, matrix.ORACLE_THREE_STRICT, matrix.ORACLE_THREE_PROVISIONAL)
BATCHES = (matrix.FIRST_BATCH, matrix.SECOND_BATCH)


def row_numbers_in(rows_root: str) -> list[int]:
    return sorted(int(name.removeprefix("row-")) for name in os.listdir(rows_root)
                  if name.startswith("row-") and name.removeprefix("row-").isdigit() and int(name.removeprefix("row-")) > 0)


def stream_cells(run, stream: str, o3) -> list[str]:
    """一行一条流两批的格（每格一段 `<列>_<量>=<值>` 的文字，次序确定）。"""
    cells = []
    calls = run.streams[stream].calls
    for batch in BATCHES:
        parts = [f"batch={batch}", f"O1-对拍_red={sum(call.is_a_model_disagreement() for call in calls)}",
                 f"O1-对拍_judged={sum(call.was_judged_by_the_comparison() for call in calls)}"]
        states = len(matrix.batch_states(run.streams[stream], batch))
        for column in STATE_COLUMNS:
            path = matrix.trajectory(run.streams[stream], stream, column, batch, o3)
            parts.append(f"{field(column)}_red={sum(count for _, count in path.red_by_segment)} {field(column)}_judged={states if o3 is not None or column not in (matrix.ORACLE_THREE_STRICT, matrix.ORACLE_THREE_PROVISIONAL) else 0} "
                         f"{field(column)}_peak={path.peak} {field(column)}_positive_segments={path.positive_segments} "
                         f"{field(column)}_last={path.last_segment_value} {field(column)}_single_points={len(path.single_point_segments)}")
        cells.append(" ".join(parts))
    return cells


def matrix_lines(lines: ResultLines, rows_root: str, t1_text: str, streams: tuple[str, ...]) -> dict[str, list[int]]:
    """逐行逐流两批的格、每一判定列的整表漏判集。交回 {列: 漏判的行}。"""
    categories = row_categories(t1_text)
    missed: dict[str, list[int]] = {column: [] for column in DECISION_COLUMNS}
    counted_rows = 0
    for number in row_numbers_in(rows_root):
        dump = os.path.join(rows_root, f"row-{number}", "dump")
        if not os.path.isdir(dump):
            lines.result(f"name=matrix_row row={number} ran=false")
            continue
        run = load_run_directory(dump)
        o3 = read_o3(os.path.join(rows_root, f"row-{number}", "o3.out"))
        for stream in matrix.streams_present(run, streams):
            for text in stream_cells(run, stream, o3):
                lines.result(f"name=matrix_row row={number} stream={stream} o3_ran={str(o3 is not None).lower()} {text}")
        if categories.get(number, {}).get("category_registration_batch_1") != CATEGORY_COUNTED:
            continue
        counted_rows += 1
        for column in DECISION_COLUMNS:
            if matrix.column_is_red(run, column, matrix.FIRST_BATCH, o3, streams) is False:
                missed[column].append(number)
    for column in DECISION_COLUMNS:
        lines.result(f"name=matrix_missed column={field(column)} counted_rows={counted_rows} missed_rows={len(missed[column])} "
                     f"rows={field(','.join(map(str, missed[column])) or '-')} r1=no_per_column_reading_in_git")
    return missed
