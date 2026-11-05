"""q2_misses.py 的单测：跑前登记第五节 5.2 第二张表「Q2」那几格（手造 D 上的两边读数）、第八节判别力自证（「多了」按集合，挪成按个数要翻）、
第九节 V10–V12 各自会红的那一格、F4。跑法：在本目录 `python3 -m unittest test_q2_misses`。"""

from __future__ import annotations

import unittest

import q2_misses
from q2_misses import STREAM_ONE, STREAM_THREE, STREAM_TWO

COLUMN_FIELDS = ("o1_comparison_red_calls", "O1-模型崩溃判官_red", "O1-RefFS_red", "O2_red", "O3甲_red", "O3乙_red")


def q1_row(row: int, stream: str, batch: int = 1, **reds: int) -> str:
    """一条 q1_row 行；reds 的键是 q1_row 里的列字段名（带连字符的用 ** 字典交），没给的都是 0。"""
    values = {name: 0 for name in COLUMN_FIELDS}
    values.update(reds)
    return f"E7RESULT name=q1_row row={row} stream={stream} batch={batch} " + " ".join(f"{name}={value}" for name, value in values.items())


def categories(row: int, category: str = "counted") -> str:
    return (f"E7RESULT name=row row={row} status=ok category_registration_batch_1={category} category_registration_batch_2={category} "
            f"category_original_reading_5_batch_1={category} category_original_reading_5_batch_2={category}")


def all_red(row: int, streams=(STREAM_ONE, STREAM_TWO), batches=(1, 2), **overrides: int) -> list[str]:
    """一行在每条流每一批上每一列都红一处（不漏），overrides 改某几列。"""
    reds = {name: 1 for name in COLUMN_FIELDS}
    reds.update(overrides)
    return [q1_row(row, stream, batch, **reds) for stream in streams for batch in batches]


class Recorder:
    """ResultLines 的替身：只收行。"""

    def __init__(self):
        self.lines: list[str] = []

    def result(self, body: str) -> None:
        self.lines.append(body)


def judge(p1, p3, t1, t3, segment="streams_1_2"):
    recorder = Recorder()
    verdict, agrees = q2_misses.q2_lines(recorder, "\n".join(p1), "\n".join(p3), "\n".join(t1), "\n".join(t3), segment)
    return verdict, agrees, recorder.lines


def column_line(lines: list[str], column: str, reading="registration", batch="1") -> dict[str, str]:
    for line in lines:
        if line.startswith(f"name=q2_column ") and f" reading={reading} batch={batch} column={column} " in line + " ":
            return dict(token.split("=", 1) for token in line.split(" ")[1:])
    raise AssertionError(f"没有 {column} 那一行")


def domain_line(lines: list[str], reading="registration", batch="1") -> dict[str, str]:
    for line in lines:
        if line.startswith(f"name=q2_domain ") and f" reading={reading} batch={batch} " in line:
            return dict(token.split("=", 1) for token in line.split(" ")[1:])
    raise AssertionError("没有 D 那一行")


class Q2Controls(unittest.TestCase):
    def test_identical_misses_are_not_more(self):
        """5.2「Q2·不多」：D 上两边漏判集逐行相同，每列多 0 行。"""
        p3 = all_red(15, O2_red=0) + all_red(23)
        verdict, agrees, lines = judge([categories(15), categories(23)], p3, [categories(15), categories(23)], p3)
        self.assertEqual((verdict, agrees), (q2_misses.VERDICT_NOT_MORE, True))
        self.assertEqual(column_line(lines, "O2")["missed_today"], "15")
        self.assertEqual(column_line(lines, "O2")["more"], "-")

    def test_one_more_missed_row_in_o2_is_more(self):
        """5.2「Q2·多了」：D 上今天 O2 多漏一行。"""
        p1 = [categories(15), categories(23)]
        verdict, _, lines = judge(p1, all_red(15) + all_red(23), p1, all_red(15, O2_red=0) + all_red(23))
        self.assertEqual(verdict, q2_misses.VERDICT_MORE)
        self.assertEqual(column_line(lines, "O2")["more"], "15")

    def test_one_out_one_in_is_more_although_the_sizes_agree(self):
        """5.2「Q2·一出一进」与 V11：O1 一行出、另一行进（个数相同），多了 1 行、少了 1 行、个数之差 0。"""
        p1 = [categories(15), categories(23)]
        r1 = all_red(15, **{"o1_comparison_red_calls": 0, "O1-模型崩溃判官_red": 0, "O1-RefFS_red": 0}) + all_red(23)
        today = all_red(15) + all_red(23, **{"o1_comparison_red_calls": 0, "O1-模型崩溃判官_red": 0, "O1-RefFS_red": 0})
        verdict, _, lines = judge(p1, r1, p1, today)
        line = column_line(lines, "O1")
        self.assertEqual((line["more"], line["fewer"], line["size_difference"]), ("23", "15", "0"))
        self.assertEqual(verdict, q2_misses.VERDICT_MORE)

    def test_a_row_untouched_today_is_left_out_of_d(self):
        """5.2「Q2·缺一边」：R7 里一行 r1 有 q1_row、今天未触达，不进 D，单列原因。"""
        verdict, _, lines = judge([categories(15), categories(42)], all_red(15) + all_red(42),
                                  [categories(15), categories(42, "untouched")], all_red(15))
        self.assertEqual(domain_line(lines)["d"], "15")
        self.assertIn("42:today_not_counted", domain_line(lines)["excluded"])
        self.assertEqual(verdict, q2_misses.VERDICT_NOT_MORE)

    def test_a_row_untouched_in_r1_is_left_out_of_d(self):
        """V12：R7 里一行 r1 是 untouched，不进 D（不当 r1 一侧全漏）。"""
        verdict, _, lines = judge([categories(15, "untouched")], all_red(15, O2_red=0), [categories(15)], all_red(15))
        self.assertEqual(domain_line(lines)["d"], "-")
        self.assertIn("15:r1_not_counted", domain_line(lines)["excluded"])
        self.assertEqual(verdict, q2_misses.VERDICT_NOT_ENOUGH)

    def test_the_judgement_reads_batch_one(self):
        """V10：今天一行批 1 不红、批 2 红——判定那一份（批 1）里它漏判，批 2 那一份里不漏。"""
        p1 = [categories(15)]
        today = [q1_row(15, stream, 1) for stream in (STREAM_ONE, STREAM_TWO)] + [q1_row(15, stream, 2, O2_red=3) for stream in (STREAM_ONE, STREAM_TWO)]
        verdict, _, lines = judge(p1, all_red(15), p1, today)
        self.assertEqual(column_line(lines, "O2", batch="1")["missed_today"], "15")
        self.assertEqual(column_line(lines, "O2", batch="2")["missed_today"], "-")
        self.assertEqual(verdict, q2_misses.VERDICT_MORE)

    def test_the_stream_three_judgement_needs_the_third_stream(self):
        """修订 1 第 1.2 条的 Q2-流三：三条流里有一条红就不漏；r1 没有流三的 q1_row 的行不进 D。"""
        p1 = [categories(15), categories(23)]
        streams = (STREAM_ONE, STREAM_TWO, STREAM_THREE)
        r1 = all_red(15, streams=streams) + all_red(23)
        today = all_red(15, streams=streams) + all_red(23, streams=streams)
        verdict, _, lines = judge(p1, r1, p1, today, segment="stream_three")
        self.assertEqual(domain_line(lines)["d"], "15")
        self.assertIn("23:r1_has_no_q1_row", domain_line(lines)["excluded"])
        self.assertEqual(verdict, q2_misses.VERDICT_NOT_MORE)


class TwoWays(unittest.TestCase):
    def test_the_set_and_the_loop_agree_and_stay_inside_d(self):
        """A6：集合差与逐行循环两种写法相同；漏判集都 ⊆ D。"""
        difference = q2_misses.column_difference(frozenset({15}), frozenset({23}), [15, 23])
        self.assertTrue(difference.loop_agrees)
        outside = q2_misses.column_difference(frozenset({99}), frozenset(), [15])
        self.assertFalse(outside.loop_agrees)


if __name__ == "__main__":
    unittest.main()
