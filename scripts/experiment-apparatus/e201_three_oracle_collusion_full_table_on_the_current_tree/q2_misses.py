"""E201（三oracle合谋今树整表） 问题单 Q2：D 与各判定列的漏判集、多了、少了（跑前登记 research/prompts/e201-preregistration.md 第一节读法 9、
修订 1 第 1.1、1.2 条；第六节 Q2-域、Q2-判、Q2-流三；第七节 A6；第十节 F4）。

D = R7 里同时满足四样的行：P3 有它这几条流这一批的 `q1_row` 行；P1 里它这一读法这一批的类是 counted；T1 里同样 counted；T3 有它这几条流这一批的 `q1_row` 行。
一行在 X 列漏判 = 它在 D 里、而 X 在这几条流这一批上一处都没判红（O1 = 三个子列合起来，任一子列红算红；O3 严格只数「不在」，暂代另一列）。
判定列四条：O1、O2、O3 严格（O3甲）、O3 暂代（O3乙）；O1 三个子列另报（描述）。多了 = M_today 里有、M_r1 里没有的行；少了反过来；两样各报，个数之差另报。
判定取登记读法、批 1、流一流二那一份（`--segment streams_1_2`）；批 2 与原登记读法 5 并列报（第八节几何敏感性）；`--segment stream_three` 是修订 1 第 1.2 条的
「Q2-流三」（三条流批 1 任一处红，T1 仍取第一段那一份，各占一行）。

只读、写一份新产物（已存在就拒）。跑法（在本目录，整条命令交 `task-server/task submit --`；合并那一步由 full_table_run.py 在进程里调 q2_lines）：
  python3 q2_misses.py --segment streams_1_2|stream_three --p1 <P1> --p3 <P3> --t1 <T1> --t3 <T3> --output <新文件>
退出码：0 写完；1 A6 两种算法对不上；2 用法错。
"""

from __future__ import annotations

import argparse
import dataclasses
import sys

import e171_research_side  # noqa: F401  （把 E171 研究一侧放进 import 路径）
from contrast_against_r1 import STREAM_ONE, STREAM_THREE, STREAM_TWO, read_product_entries
from result_lines import ResultLines, field

R7 = (15, 23, 42, 242, 269, 270, 271)
SUBCOLUMN_FIELDS = {"O1-对拍": "o1_comparison_red_calls", "O1-模型崩溃判官": "O1-模型崩溃判官_red", "O1-RefFS": "O1-RefFS_red"}
DECISION_COLUMNS = {"O1": tuple(SUBCOLUMN_FIELDS.values()), "O2": ("O2_red",), "O3甲": ("O3甲_red",), "O3乙": ("O3乙_red",)}
STREAMS_OF_THE_SEGMENT = {"streams_1_2": (STREAM_ONE, STREAM_TWO), "stream_three": (STREAM_ONE, STREAM_TWO, STREAM_THREE)}
READINGS = ("registration", "original_reading_5")
BATCHES = ("1", "2")
CATEGORY_COUNTED = "counted"
VERDICT_MORE, VERDICT_NOT_MORE, VERDICT_NOT_ENOUGH = "some_column_more", "no_column_more", "not_enough_f4"


def q1_row_counts(text: str) -> dict[tuple[int, str, str], dict[str, str]]:
    """一份第一段形态产物里的 `q1_row` 行：{(行, 流, 批): 字段}。"""
    return {(int(fields["row"]), fields["stream"], fields["batch"]): fields
            for kind, fields in read_product_entries(text) if kind == "q1_row"}


def row_categories(text: str) -> dict[int, dict[str, str]]:
    """第二段比对器形态产物里的 `row` 行：{行: 字段}。"""
    return {int(fields["row"]): fields for kind, fields in read_product_entries(text) if kind == "row"}


def counted(categories: dict[int, dict[str, str]], row: int, reading: str, batch: str) -> bool:
    return categories.get(row, {}).get(f"category_{reading}_batch_{batch}") == CATEGORY_COUNTED


def has_rows(counts: dict[tuple[int, str, str], dict[str, str]], row: int, streams: tuple[str, ...], batch: str) -> bool:
    return all((row, stream, batch) in counts for stream in streams)


def domain(r1_counts, r1_categories, today_counts, today_categories, streams: tuple[str, ...], reading: str, batch: str
           ) -> tuple[list[int], dict[int, str]]:
    """读法 9 的 D（升序）与 R7 里每一行不进 D 的原因（四样里第一样不满足的那一样）。"""
    members, excluded = [], {}
    for row in R7:
        if not has_rows(r1_counts, row, streams, batch):
            excluded[row] = "r1_has_no_q1_row"
        elif not counted(r1_categories, row, reading, batch):
            excluded[row] = "r1_not_counted"
        elif not counted(today_categories, row, reading, batch):
            excluded[row] = "today_not_counted"
        elif not has_rows(today_counts, row, streams, batch):
            excluded[row] = "today_has_no_q1_row"
        else:
            members.append(row)
    return members, excluded


def column_red(counts, row: int, streams: tuple[str, ...], batch: str, field_names: tuple[str, ...]) -> bool:
    """这一行在这几个字段（一列或一个子列）上、这几条流这一批里有没有一处红（计数大于 0）。"""
    return any(int(counts[(row, stream, batch)].get(name, "0")) > 0 for stream in streams for name in field_names)


def missed_rows(counts, members: list[int], streams: tuple[str, ...], batch: str, field_names: tuple[str, ...]) -> frozenset[int]:
    return frozenset(row for row in members if not column_red(counts, row, streams, batch, field_names))


@dataclasses.dataclass(frozen=True)
class ColumnDifference:
    """一列在 D 上两边的漏判集与差；loop_agrees 是 A6 那一份逐行循环算的多了、少了与集合差相同。"""

    missed_r1: frozenset[int]
    missed_today: frozenset[int]
    more: frozenset[int]
    fewer: frozenset[int]
    loop_agrees: bool


def column_difference(missed_r1: frozenset[int], missed_today: frozenset[int], members: list[int]) -> ColumnDifference:
    more = missed_today - missed_r1
    fewer = missed_r1 - missed_today
    loop_more = [row for row in members if row in missed_today and row not in missed_r1]
    loop_fewer = [row for row in members if row in missed_r1 and row not in missed_today]
    agrees = sorted(more) == loop_more and sorted(fewer) == loop_fewer and missed_r1 <= set(members) and missed_today <= set(members)
    return ColumnDifference(missed_r1, missed_today, more, fewer, agrees)


def rows_text(rows) -> str:
    return ",".join(map(str, sorted(rows))) or "-"


def q2_lines(lines: ResultLines, p1_text: str, p3_text: str, t1_text: str, t3_text: str, segment: str) -> tuple[str, bool]:
    """Q2 的全部行；交回 (登记读法、批 1 那一份的判定, A6 两种算法处处相同)。"""
    streams = STREAMS_OF_THE_SEGMENT[segment]
    r1_counts, today_counts = q1_row_counts(p3_text), q1_row_counts(t3_text)
    r1_categories, today_categories = row_categories(p1_text), row_categories(t1_text)
    decisive, all_agree = VERDICT_NOT_ENOUGH, True
    for reading in READINGS:
        for batch in BATCHES:
            members, excluded = domain(r1_counts, r1_categories, today_counts, today_categories, streams, reading, batch)
            lines.result(f"name=q2_domain segment={segment} reading={reading} batch={batch} r7={rows_text(R7)} d={rows_text(members)} "
                         f"d_size={len(members)} excluded={field(','.join(f'{row}:{why}' for row, why in sorted(excluded.items())) or '-')}")
            more_columns = []
            for column, field_names in list(DECISION_COLUMNS.items()) + [(name, (value,)) for name, value in SUBCOLUMN_FIELDS.items()]:
                difference = column_difference(missed_rows(r1_counts, members, streams, batch, field_names),
                                               missed_rows(today_counts, members, streams, batch, field_names), members)
                all_agree = all_agree and difference.loop_agrees
                is_decision_column = column in DECISION_COLUMNS
                if is_decision_column and difference.more:
                    more_columns.append(column)
                lines.result(f"name=q2_column segment={segment} reading={reading} batch={batch} column={column} decision_column={str(is_decision_column).lower()} "
                             f"missed_r1={rows_text(difference.missed_r1)} missed_today={rows_text(difference.missed_today)} more={rows_text(difference.more)} "
                             f"fewer={rows_text(difference.fewer)} size_difference={len(difference.missed_today) - len(difference.missed_r1)} "
                             f"loop_agrees={str(difference.loop_agrees).lower()}")
            verdict = VERDICT_NOT_ENOUGH if not members else (VERDICT_MORE if more_columns else VERDICT_NOT_MORE)
            lines.result(f"name=q2_judgement segment={segment} reading={reading} batch={batch} verdict={verdict} more_columns={field(','.join(more_columns) or '-')}")
            if reading == READINGS[0] and batch == BATCHES[0]:
                decisive = verdict
    return decisive, all_agree


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E201 问题单 Q2 的 D 与漏判")
    parser.add_argument("--segment", required=True, choices=tuple(STREAMS_OF_THE_SEGMENT))
    for name in ("p1", "p3", "t1", "t3"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    texts = {}
    for name in ("p1", "p3", "t1", "t3"):
        with open(getattr(arguments, name), encoding="utf-8") as handle:
            texts[name] = handle.read()
    lines = ResultLines(arguments.output)
    lines.result(f"name=config script=q2_misses segment={arguments.segment}")
    verdict, agrees = q2_lines(lines, texts["p1"], texts["p3"], texts["t1"], texts["t3"], arguments.segment)
    lines.result(f"name=q2_verdict segment={arguments.segment} q2={verdict} a6_two_ways_agree={str(agrees).lower()}")
    lines.finish()
    return 0 if agrees else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
