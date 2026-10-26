"""E171 比对器与驱动其余部分的单测（登记第九节 X9、X10，第七节 S5 的现算、Q3-核）：临时目录里手造小流，不起装置、不起 cargo。

跑法（在本目录）：`python3 -m unittest test_collusion_pipeline`。
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
import unittest

import baseline_checks as checks
import bad_implementation_table as table
import bad_images_report
import row_driver
from result_lines import ResultLines, field
from run_directory import STREAM_NAMES, load_run_directory, parse_result_line

HERE = os.path.dirname(os.path.abspath(__file__))


def operations(*kinds: tuple[str, str]) -> list[list[str]]:
    """(盘, 种类) 的列表 → ops.tsv 的行（只填比对器读的那几列）。"""
    return [[str(index), device, kind, "0", "512", "0", "-"] for index, (device, kind) in enumerate(kinds)]


class SegmentationTests(unittest.TestCase):
    def test_a_barrier_on_every_device_closes_the_segment_and_a_fua_write_releases_only_its_own_device(self) -> None:
        rows = operations(("0", "write"), ("1", "write"), ("0", "barrier"), ("1", "barrier"),
                          ("0", "write"), ("0", "write_fua"), ("1", "write"), ("1", "barrier"))
        self.assertEqual(checks.segments_of_the_write_stream(rows), [[0, 1], [2, 3], [4]])
        shared = operations(("0", "write"), ("1", "write_fua"), ("0", "write"), ("0", "barrier"), ("1", "barrier"))
        self.assertEqual(checks.segments_of_the_write_stream(shared), [[0, 1, 2]], "FUA 只放行它自己那块盘，盘 0 上的写要等盘 0 的屏障")

    def test_a_trailing_open_segment_is_its_own_segment_and_leading_barriers_do_not_close_anything(self) -> None:
        rows = operations(("0", "barrier"), ("0", "write"), ("0", "write"))
        self.assertEqual(checks.segments_of_the_write_stream(rows), [[0, 1]])

    def test_state_counts_are_the_choice_product_minus_one_per_segment_plus_the_final_state(self) -> None:
        ranges, total = checks.state_ranges_of_the_segments([[0, 1], [2]], {0})
        self.assertEqual([(state_range.start, state_range.stop) for state_range in ranges], [(0, 5), (5, 6)])
        self.assertEqual(total, 7)

    def test_the_landing_vector_decodes_mixed_radix_with_the_first_write_as_the_lowest_digit(self) -> None:
        segments, tearable = [[0, 1], [2]], {0}
        ranges, total = checks.state_ranges_of_the_segments(segments, tearable)
        landings = [checks.landing_vector_of_ordinal(ordinal, segments, ranges, tearable, 3) for ordinal in range(total)]
        self.assertEqual(landings, ["000", "100", "200", "020", "120", "220", "222"])

    def test_the_first_batch_takes_ends_interior_points_the_last_state_and_up_to_four_torn_states(self) -> None:
        segments, tearable = [[0]], {0}
        ranges, total = checks.state_ranges_of_the_segments(segments, tearable)
        chosen = checks.first_batch_ordinals(segments, ranges, total, tearable, 1)
        self.assertEqual(chosen, {0, 1, 2})
        wide = [list(range(0, 10))]
        wide_ranges, wide_total = checks.state_ranges_of_the_segments(wide, set())
        chosen_wide = checks.first_batch_ordinals(wide, wide_ranges, wide_total, set(), 10)
        self.assertEqual(chosen_wide, {0, 146, 292, 438, 584, 730, 876, 1022, 1023})


class WithoutTimingTests(unittest.TestCase):
    def test_only_the_timing_column_and_the_verdicts_digest_line_are_dropped(self) -> None:
        self.assertEqual(checks.contents_without_timing("impl-verdicts.tsv", b"0\ta\t123\n1\tb\t456\n"), b"0\ta\n1\tb\n")
        manifest = b"stream\tx\nsha256:impl-verdicts.tsv\tdead\nsha256:ops.tsv\tbeef\n"
        self.assertEqual(checks.contents_without_timing("manifest.tsv", manifest), b"stream\tx\nsha256:ops.tsv\tbeef\n")
        self.assertEqual(checks.contents_without_timing("ops.tsv", b"a\tb\n"), b"a\tb\n")


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def minimal_run(root: str, o2_word: str = "green", model_word: str = "agrees") -> None:
    for name in STREAM_NAMES:
        base = os.path.join(root, name)
        write(os.path.join(base, "ops.tsv"), "0\t0\twrite\t0\t512\t0\t-\n")
        write(os.path.join(base, "calls.tsv"), "0\tpublish\t0\t1\tok\n")
        write(os.path.join(base, "base-sectors.bin"), "b")
        write(os.path.join(base, "ops-payload.bin"), "p")
        write(os.path.join(base, "states.tsv"), "0\t0\t0\t1\t2\tdigest\n")
        write(os.path.join(base, "oracle-verdicts.tsv"),
              f"0\t1\t0\t0\t0\tgreen\t-\t{o2_word}\t-\ttrue\t1,3\tfile_read\tabc\t1\t1,3\tfile_read\tabc\t1\t-\n")
        write(os.path.join(base, "call-outcomes.tsv"), f"0\tpublish\t0\t1\tok\t{model_word}\t-\t-\t-\n")
        write(os.path.join(base, "stream-status.tsv"), "ending\tcompleted\nmodel_stopped_at_call\tnone\n")


class RunDirectoryTests(unittest.TestCase):
    def test_a_minimal_run_directory_loads_and_a_missing_stream_is_not_exported(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            minimal_run(root)
            run = load_run_directory(root)
            first = run.streams[STREAM_NAMES[0]]
            self.assertTrue(first.exported)
            self.assertEqual((len(first.states), len(first.calls), first.ending()), (1, 1, "completed"))
            self.assertEqual(first.states[0].consulted.records_applied, 1)
            self.assertIsNone(first.model_stopped_at_call())
        with tempfile.TemporaryDirectory() as empty:
            self.assertFalse(load_run_directory(empty).streams[STREAM_NAMES[0]].exported)

    def test_closing_one_oracle_only_changes_its_own_columns(self) -> None:
        with tempfile.TemporaryDirectory() as full, tempfile.TemporaryDirectory() as closed_o2, tempfile.TemporaryDirectory() as closed_o1:
            minimal_run(full)
            minimal_run(closed_o2, o2_word="off")
            minimal_run(closed_o1, model_word="not_modelled")
            self.assertEqual(checks.differences_after_closing_one_oracle(full, closed_o2, "O2"), [])
            self.assertEqual(checks.differences_after_closing_one_oracle(full, closed_o1, "O1"), [])
            self.assertEqual(len(checks.differences_after_closing_one_oracle(full, closed_o1, "O2")), 2)
            self.assertEqual(len(checks.differences_after_closing_one_oracle(full, closed_o2, "O1")), 2)

    def test_closing_o1_drops_only_the_model_stop_line_of_the_stream_status(self) -> None:
        """登记修订 4：模型停在第几次调用（`stream-status.tsv` 的 `model_stopped_at_call`）是 O1 自己的状态，关 O1 时只去掉这一行；
        别的行照比，关 O2 时这一行也照比。第 32 行 Q3-核关 O1 那一趟是 4 对 14。"""
        def status(root: str, stopped: str, calls: str) -> None:
            for name in STREAM_NAMES:
                write(os.path.join(root, name, "stream-status.tsv"), f"ending\tcompleted\nmodel_stopped_at_call\t{stopped}\ncalls\t{calls}\n")
        with tempfile.TemporaryDirectory() as full, tempfile.TemporaryDirectory() as closed_o1, \
                tempfile.TemporaryDirectory() as closed_o1_other_line, tempfile.TemporaryDirectory() as closed_o2:
            minimal_run(full)
            status(full, "4", "15")
            minimal_run(closed_o1, model_word="not_modelled")
            status(closed_o1, "14", "15")
            minimal_run(closed_o1_other_line, model_word="not_modelled")
            status(closed_o1_other_line, "14", "16")
            minimal_run(closed_o2, o2_word="off")
            status(closed_o2, "14", "15")
            self.assertEqual(checks.differences_after_closing_one_oracle(full, closed_o1, "O1"), [])
            expected = [f"{name}/stream-status.tsv（被关的那一行之外）" for name in STREAM_NAMES]
            self.assertEqual(checks.differences_after_closing_one_oracle(full, closed_o1_other_line, "O1"), expected)
            self.assertEqual(checks.differences_after_closing_one_oracle(full, closed_o2, "O2"), expected)

    def test_the_result_line_parser_and_the_field_writer_agree_on_blanks(self) -> None:
        self.assertEqual(parse_result_line("E7RESULT name=a x=1 y=-"), {"name": "a", "x": "1", "y": "-"})
        self.assertIsNone(parse_result_line("E7TIMING name=a"))
        self.assertEqual(field("a b\tc"), "a_b_c")
        self.assertEqual(field(""), "-")

    def test_result_lines_count_the_done_line_and_refuse_an_existing_file(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "out")
            lines = ResultLines(path)
            lines.result("name=a")
            lines.timing("name=t")
            lines.finish()
            with open(path, encoding="utf-8") as handle:
                self.assertEqual(handle.read().splitlines(), ["E7RESULT name=a", "E7TIMING name=t", "E7RESULT name=done emitted=2"])
            with self.assertRaises(FileExistsError):
                ResultLines(path)


def mutation_row(name: str, file: str = "crates/singlefs-core/src/a.rs", original: str = "x", replacement: str = "y") -> table.MutationRow:
    return table.MutationRow(name, file, original, replacement, "-p c", "t", "\t".join([name, file, original, replacement, "-p c", "t"]))


class BadImplementationTableTests(unittest.TestCase):
    def test_the_anchor_must_hit_exactly_once_and_newlines_unescape(self) -> None:
        row = mutation_row("r", original="a\\nb", replacement="c")
        self.assertEqual(table.apply_mutation("xa\nby", row), "xcy")
        with self.assertRaises(ValueError):
            table.apply_mutation("a\nb a\nb", row)
        with self.assertRaises(ValueError):
            table.apply_mutation("none", row)

    def test_a_substitute_is_the_next_row_of_the_group_that_is_not_already_in_the_table(self) -> None:
        group = ["a", "b", "c"]
        self.assertEqual(table.substitute_for(group, "a", {"a", "b"}, set()), "c")
        self.assertEqual(table.substitute_for(group, "c", {"c"}, set()), "a")
        self.assertEqual(table.substitute_for(group, "a", {"a", "b"}, {"c"}), None)
        self.assertEqual(table.substitute_for(group, "zzz", set(), set()), None)

    def test_the_empty_mutation_replaces_the_original_with_itself(self) -> None:
        row = mutation_row("first", original="keep", replacement="break")
        empty = table.empty_mutation(row)
        self.assertEqual(table.apply_mutation("a keep b", empty), "a keep b")

    def test_a_row_is_located_by_name_and_reports_a_changed_line(self) -> None:
        row = mutation_row("name")
        located = table.locate_by_name(table.SelectedRow(1, 1, row.file, 0, "name", "000000000000"), [row])
        self.assertEqual((located.status, located.line_matches_the_registration), ("ok", False))
        located = table.locate_by_name(table.SelectedRow(1, 1, row.file, 0, "name", row.line_digest_prefix()), [row])
        self.assertTrue(located.line_matches_the_registration)
        self.assertEqual(table.locate_by_name(table.SelectedRow(1, 1, row.file, 0, "missing", "0"), [row]).status, "not_found")
        self.assertEqual(table.locate_by_name(table.SelectedRow(1, 1, row.file, 0, "name", "0"), [row, row]).status, "ambiguous_name")

    def test_the_device_own_rows_are_left_out_of_the_selection_and_the_count_and_the_rest_is_kept_in_order(self) -> None:
        text = "# 表头\nrow a\tf\to\tr\ta\tt\nE171 装置 取号恒拒\tf\to\tr\ta\tt\nrow b\tf\to\tr\ta\tt\nE171 装置 另一行\tf\to\tr\ta\tt\n"
        kept, removed = table.without_the_rows_of_the_device_itself(text)
        self.assertEqual(removed, 2)
        self.assertEqual(kept, "# 表头\nrow a\tf\to\tr\ta\tt\nrow b\tf\to\tr\ta\tt\n")
        self.assertEqual(table.without_the_rows_of_the_device_itself("row E171 装置\tf\n"), ("row E171 装置\tf\n", 0))


class SelectionScriptTests(unittest.TestCase):
    def selection(self, rows: int) -> list[str]:
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "mutations.tsv")
            with open(path, "w", encoding="utf-8") as handle:
                for index in range(rows):
                    handle.write(f"row {index}\tcrates/singlefs-core/src/mount.rs\to\tr\t-p c\tt\n")
                handle.write("format row\tcrates/singlefs-format/src/lib.rs\to\tr\t-p c\tt\n")
            output = subprocess.run([sys.executable, os.path.join(HERE, "select_bad_implementations.py"), path, "8"], check=True,
                                    capture_output=True, text=True).stdout
        return output.splitlines()

    def test_a_group_of_n_rows_gives_ceiling_n_over_eight_rows_and_leaves_the_format_crate_out(self) -> None:
        self.assertEqual(len(self.selection(204)), 26)
        self.assertEqual(len(self.selection(16)), 2)
        self.assertEqual(len(self.selection(17)), 3)
        self.assertEqual(len(self.selection(1)), 1)
        self.assertTrue(all("singlefs-format" not in line for line in self.selection(5)))


class RowPreparationTests(unittest.TestCase):
    def test_a_row_whose_anchor_hits_twice_is_replaced_by_the_next_row_of_its_group(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "crates/singlefs-core/src/a.rs"), "x x\nonly_here\n")
            rows = [mutation_row("first", original="x"), mutation_row("second", original="only_here"), mutation_row("third", original="only_here")]
            selected = table.SelectedRow(1, 1, "crates/singlefs-core/src/a.rs", 0, "first", rows[0].line_digest_prefix())
            used: set[str] = set()
            choice = row_driver.prepare_row(selected, rows, {"first"}, used, root, None)
            self.assertEqual((choice.mutation.name if choice.mutation else None, choice.status, choice.substituted_from,
                              choice.line_matches_the_registration, choice.substitutions_used, choice.failed_attempts),
                             ("second", "ok", "first", False, 1, (("first", "anchor_not_unique"),)))
            self.assertEqual(used, {"second"})

    def test_a_row_with_no_usable_anchor_in_its_group_is_reported_with_the_failure(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "crates/singlefs-core/src/a.rs"), "nothing\n")
            rows = [mutation_row("first", original="x")]
            selected = table.SelectedRow(1, 1, "crates/singlefs-core/src/a.rs", 0, "first", rows[0].line_digest_prefix())
            choice = row_driver.prepare_row(selected, rows, {"first"}, set(), root, None)
            self.assertEqual((choice.mutation, choice.status), (None, "anchor_not_unique"))


class CompileFailureSubstitutionTests(unittest.TestCase):
    """登记第一节读法 5「替换之后编不过（第八类，按 5.2 的补位规则换行）」：编不过的行换同一文件组里下标 j + 1、两张表都没有的那一行，换上的行编不过再换，
    编之前的失败与编不过合起来一行至多三次；组里不含 E171 装置自己的变异行。手造组 a.rs：r0 … r5，表里有 r0、r2；装置行插在 r3 后面。"""

    FILE = "crates/singlefs-core/src/a.rs"

    def setUp(self) -> None:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = holder.name
        write(os.path.join(self.root, self.FILE), "".join(f"anchor_{index}\n" for index in range(6)))
        self.rows = [mutation_row(f"r{index}", file=self.FILE, original=f"anchor_{index}") for index in range(6)]
        self.rows.insert(4, mutation_row("E171 装置 取号恒拒", file=self.FILE, original="anchor_3"))
        self.selected = table.SelectedRow(9, 2, self.FILE, 0, "r0", self.rows[0].line_digest_prefix())
        self.used: set[str] = set()

    def fail(self, choice: row_driver.RowChoice) -> row_driver.RowChoice:
        return row_driver.substitute_after_a_compile_failure(self.selected, self.rows, {"r0", "r2"}, self.used, self.root, choice)

    def test_a_row_that_does_not_compile_is_replaced_by_the_next_row_of_its_group_not_in_either_table(self) -> None:
        first = row_driver.prepare_row(self.selected, self.rows, {"r0", "r2"}, self.used, self.root, None)
        self.assertEqual((first.mutation.name, first.substituted_from, first.substitutions_used), ("r0", "-", 0))
        second = self.fail(first)
        self.assertEqual((second.mutation.name, second.status, second.substituted_from, second.line_matches_the_registration,
                          second.substitutions_used, second.failed_attempts),
                         ("r1", "ok", "r0", False, 1, (("r0", "compile_failed"),)))
        third = self.fail(second)
        self.assertEqual((third.mutation.name, third.substituted_from, third.substitutions_used), ("r3", "r0", 2),
                         "r1 编不过再换：跳过表里的 r2，装置行不在组里")
        fourth = self.fail(third)
        self.assertEqual((fourth.mutation.name, fourth.substitutions_used), ("r4", 3))
        self.assertEqual(self.fail(fourth), row_driver.RowChoice(
            None, "compile_failed", "r0", False, 3, (("r0", "compile_failed"), ("r1", "compile_failed"), ("r3", "compile_failed"), ("r4", "compile_failed"))))
        self.assertEqual(self.used, {"r1", "r3", "r4"})

    def test_a_substitute_whose_anchor_fails_counts_against_the_same_three_and_the_row_stays_a_compile_failure(self) -> None:
        write(os.path.join(self.root, self.FILE), "anchor_0\nanchor_1\nanchor_1\nanchor_3\n")  # r1 两处、r4 零处
        second = self.fail(row_driver.prepare_row(self.selected, self.rows, {"r0", "r2"}, self.used, self.root, None))
        self.assertEqual((second.mutation.name, second.substitutions_used, second.failed_attempts),
                         ("r3", 2, (("r0", "compile_failed"), ("r1", "anchor_not_unique"))))
        last = self.fail(second)
        self.assertEqual((last.mutation, last.status, last.substitutions_used, last.failed_attempts),
                         (None, "compile_failed", 3, (("r0", "compile_failed"), ("r1", "anchor_not_unique"), ("r3", "compile_failed"),
                                                      ("r4", "anchor_not_unique"))))

    def test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute(self) -> None:
        """等价留档：编不过之后从编不过的那一行往下找补位，与从表里的原名往下找，在所有输入上同值——两者之间的每个名字不是在表里就是已用过。
        这里在手造组上逐个起点、逐种「表里的名字」核一遍（不是证明，是留档）。"""
        group = [f"r{index}" for index in range(6)]
        for in_the_table in ({"r0"}, {"r0", "r2"}, {"r0", "r3", "r4"}, {"r0", "r5"}):
            used: set[str] = set()
            last_failed = "r0"
            while True:
                from_last = table.substitute_for(group, last_failed, in_the_table, used)
                self.assertEqual(table.substitute_for(group, "r0", in_the_table, used), from_last, (sorted(in_the_table), last_failed))
                if from_last is None:
                    break
                used.add(from_last)
                last_failed = from_last

    def test_the_empty_mutation_is_never_substituted(self) -> None:
        empty = row_driver.prepare_row(None, self.rows, set(), self.used, self.root, self.rows[0])
        self.assertEqual(row_driver.substitute_after_a_compile_failure(None, self.rows, set(), self.used, self.root, empty).mutation, None)
        self.assertEqual(self.used, set())

    def test_the_group_leaves_the_device_own_rows_out(self) -> None:
        self.assertEqual(table.group_names_of_file(self.rows, self.FILE), [f"r{index}" for index in range(6)])

    def test_the_rerun_takes_the_rows_whose_result_is_a_compile_failure_and_refuses_a_row_in_two_roots(self) -> None:
        for relative, status in (("first/row-3", "compile_failed"), ("first/row-4", "ok"), ("second/row-120", "compile_failed"),
                                 ("second/row-121", "self_crash"), ("again/row-4", "ok")):
            write(os.path.join(self.root, relative, "row-result.tsv"), f"row_number\t{relative.split('-')[1]}\nstatus\t{status}\n")
        os.makedirs(os.path.join(self.root, "first", "row-5"))  # 中途停下、没有 row-result.tsv 的行目录不算
        self.assertEqual(row_driver.compile_failed_row_numbers([os.path.join(self.root, "first"), os.path.join(self.root, "second")]), {3, 120})
        with self.assertRaises(ValueError):
            row_driver.compile_failed_row_numbers([os.path.join(self.root, "first"), os.path.join(self.root, "again")])


class BadImagesReportTests(unittest.TestCase):
    def test_negative_bases_come_from_p_and_n_rows_and_the_blind_family_has_its_own_bases(self) -> None:
        rows = [bad_images_report.BadImageRow("P", "s1", 0, "P1-F", "F", "red", "I-7.13", "-"),
                bad_images_report.BadImageRow("P", "s1", 1, "P1-Mk", "Mk", "red", "I-7.13", "-"),
                bad_images_report.BadImageRow("N", "s1", 2, "N5", "Mk-", "green", "-", "I-9.9"),
                bad_images_report.BadImageRow("B", "s1", 3, "B01", "Mk@1", "green", "-", "-")]
        self.assertEqual(bad_images_report.negative_bases(rows), {("s1", "F"): "-", ("s1", "Mk"): "-", ("s1", "Mk-"): "I-9.9"})


class ComparatorSectionTests(unittest.TestCase):
    def test_a_positive_control_counts_red_only_when_the_matrix_carries_the_red(self) -> None:
        import collections

        import comparator
        with tempfile.TemporaryDirectory() as red_run, tempfile.TemporaryDirectory() as quiet_run, tempfile.TemporaryDirectory() as output:
            minimal_run(red_run, model_word="disagrees:RefusedWhenModelRequiresSuccess")
            minimal_run(quiet_run)
            tally = collections.Counter()
            lines = ResultLines(os.path.join(output, "out"))
            comparator.control_lines([f"O1-对拍:baseline:{red_run}", f"O1-对拍:empty_mutation:{quiet_run}"], lines, tally)
            lines.finish()
            self.assertEqual((tally["positive_controls_run"], tally["positive_controls_not_red"]), (2, 1))
            with open(os.path.join(output, "out"), encoding="utf-8") as handle:
                text = handle.read()
            self.assertIn("name=positive_control oracle=O1-对拍 run=baseline red_units=2 red=true", text)
            self.assertIn("name=positive_control oracle=O1-对拍 run=empty_mutation red_units=0 red=false", text)
            with self.assertRaises(ValueError):
                comparator.control_lines(["O9:baseline:x"], lines, tally)

    def test_the_registered_table_section_checks_the_per_file_counts_against_the_registration(self) -> None:
        import comparator
        self.assertEqual(sum(comparator.EXPECTED_ROWS_PER_FILE.values()), 117)
        self.assertEqual(len(comparator.EXPECTED_ROWS_PER_FILE), 24)
        self.assertEqual(comparator.EXPECTED_ROWS_PER_FILE["core/mount.rs"], 26)


class DriverArgumentTests(unittest.TestCase):
    def test_the_device_command_goes_through_the_memory_wrapper_with_the_thread_cap(self) -> None:
        command = row_driver.device_command("/repo", "/bin/dev", "/out/dump", 6, "4G", ["--positive-control", "O2"])
        self.assertEqual(command[:4], ["bash", "/repo/scripts/hosts/run-with-memory-cap.sh", "4G", "bash"])
        self.assertEqual(command[4:7], ["/repo/scripts/hosts/capped.sh", "6", "/bin/dev"])
        self.assertEqual(command[7:], ["row", "--output-directory", "/out/dump", "--positive-control", "O2"])
        self.assertIsInstance(argparse.Namespace(), argparse.Namespace)


if __name__ == "__main__":
    unittest.main()
