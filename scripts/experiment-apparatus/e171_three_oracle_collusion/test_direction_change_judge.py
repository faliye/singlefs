"""direction_change_judge.py 的单测：五类各给一对已知答案的合成产物（research/prompts/e171-r3-prereg.md 第六节 V6、第九节 M6 至 M8）。"""

import collections
import os
import tempfile
import unittest

import direction_change_judge as judge

DEFINED = frozenset({"I-3.1", "I-3.11", "I-3.15", "I-4.2", "I-8.14"})
SOURCE_LINES = ['    "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",']
REFUSED = 'failed:InvariantViolated_{_invariant:_"E142_走读同款",_detail:_"记账条目数（inode_号水位的台阶行不算）不是_3_+_6_×_盘数"_}'
UNKNOWN_REFUSED = 'failed:InvariantViolated_{_detail:_"产品里没有这一句"_}'


def rows_of(*lines: str) -> collections.OrderedDict:
    rows = collections.OrderedDict()
    seen: collections.Counter = collections.Counter()
    for line in lines:
        row = judge.parse_line("E7RESULT " + line)
        key = judge.key_of(row)
        seen[key] += 1
        rows[key + (seen[key],)] = row
    return rows


def context(labels=None, entries=None) -> judge.Context:
    return judge.Context(labels or {}, DEFINED, SOURCE_LINES, entries or {})


def bad_row(row: int, expected: str, red: str, outcome: str = "file_read") -> str:
    return (f"name=bad_row stream=s row={row} name=X family=P base=F patches=1 expected={expected} expected_landing=none "
            f"impl_consulted_outcome={outcome} impl_pool_checker_red={red} impl_both_pass={'true' if red == '-' else 'false'}")


def entry(target: str, red: str, target_is_red: str = "true") -> str:
    return f"name=known_bad_entry index=0 target={target} o2=red patched_red={red} target_is_red={target_is_red} patches=2"


def findings_of(old_lines: list[str], new_lines: list[str], ctx: judge.Context) -> list[judge.Finding]:
    pairs = judge.classify(rows_of(*old_lines), rows_of(*new_lines), ctx)
    return [finding for pair in pairs for finding in pair.findings]


class PassToRedAndStricter(unittest.TestCase):
    def test_a_pass_that_turns_red_on_a_not_in_row_with_a_defined_invariant_is_normal(self) -> None:
        [finding] = findings_of([bad_row(1, "not_in", "-")], [bad_row(1, "not_in", "I-8.14")], context())
        self.assertEqual((finding.cls, finding.result), (judge.PASS_TO_RED, judge.NORMAL))

    def test_a_new_red_on_an_expected_in_row_is_a_problem(self) -> None:
        [finding] = findings_of([bad_row(1, "in", "I-4.2")], [bad_row(1, "in", "I-4.2,I-8.14")], context())
        self.assertEqual((finding.cls, finding.result), (judge.STRICTER, judge.PROBLEM))

    def test_a_new_red_without_a_definition_line_is_unplaced(self) -> None:
        [finding] = findings_of([bad_row(1, "not_in", "-")], [bad_row(1, "not_in", "I-9.99")], context())
        self.assertEqual((finding.cls, finding.result), (judge.PASS_TO_RED, judge.UNPLACED))

    def test_a_row_without_expected_is_unplaced_unless_labels_give_it(self) -> None:
        old = "name=e170_bad_row stream=s row=3 o2=green o2_red=-"
        new = "name=e170_bad_row stream=s row=3 o2=red o2_red=I-8.14"
        [unlabelled] = findings_of([old], [new], context())
        self.assertEqual(unlabelled.result, judge.UNPLACED)
        [labelled] = findings_of([old], [new], context(labels={("s", "3"): "not_in"}))
        self.assertEqual(labelled.result, judge.NORMAL)
        [contradicted] = findings_of([old], [new], context(labels={("s", "3"): "in"}))
        self.assertEqual(contradicted.result, judge.PROBLEM)

    def test_a_known_bad_entry_gaining_a_defined_invariant_is_normal(self) -> None:
        [finding] = findings_of([entry("I-1.1", "I-1.1")], [entry("I-1.1", "I-1.1,I-4.2")], context())
        self.assertEqual((finding.cls, finding.result), (judge.STRICTER, judge.NORMAL))


class OutcomeChanged(unittest.TestCase):
    def test_a_read_turned_into_a_refusal_on_a_not_in_row_with_a_known_sentence_is_normal(self) -> None:
        [finding] = findings_of([bad_row(1, "not_in", "I-4.2")], [bad_row(1, "not_in", "I-4.2", REFUSED)], context())
        self.assertEqual((finding.cls, finding.result), (judge.OUTCOME_CHANGED, judge.NORMAL))
        self.assertIn("第 1 行", finding.reason)

    def test_a_refusal_with_a_sentence_missing_from_the_source_is_unplaced(self) -> None:
        [finding] = findings_of([bad_row(1, "not_in", "I-4.2")], [bad_row(1, "not_in", "I-4.2", UNKNOWN_REFUSED)], context())
        self.assertEqual(finding.result, judge.UNPLACED)

    def test_a_refusal_on_an_expected_in_row_is_a_problem(self) -> None:
        [finding] = findings_of([bad_row(1, "in", "-")], [bad_row(1, "in", "-", REFUSED)], context())
        self.assertEqual(finding.result, judge.PROBLEM)

    def test_a_refusal_turned_into_a_read_on_a_not_in_row_is_a_problem(self) -> None:
        [finding] = findings_of([bad_row(1, "not_in", "I-4.2", REFUSED)], [bad_row(1, "not_in", "I-4.2")], context())
        self.assertEqual((finding.cls, finding.result), (judge.OUTCOME_CHANGED, judge.PROBLEM))


class RedToPass(unittest.TestCase):
    OLD = entry("I-5.1", "I-3.1,I-5.1")

    def test_a_target_that_stops_being_red_is_a_problem(self) -> None:
        findings = findings_of([self.OLD], [entry("I-5.1", "I-3.15", target_is_red="false")], context(entries={"I-3.1": "true"}))
        self.assertEqual([finding.result for finding in findings if finding.cls == judge.RED_TO_PASS], [judge.PROBLEM])

    def test_a_dropped_invariant_whose_own_entry_is_red_is_left_to_the_main_agent(self) -> None:
        findings = findings_of([self.OLD], [entry("I-5.1", "I-5.1")], context(entries={"I-3.1": "true"}))
        removal = [finding for finding in findings if finding.cls == judge.RED_TO_PASS]
        self.assertEqual([finding.result for finding in removal], [judge.UNPLACED])
        self.assertNotIn(judge.NORMAL, [finding.result for finding in removal])

    def test_a_dropped_invariant_whose_own_entry_is_not_red_is_a_problem(self) -> None:
        findings = findings_of([self.OLD], [entry("I-5.1", "I-5.1")], context(entries={"I-3.1": "false"}))
        self.assertEqual([finding.result for finding in findings if finding.cls == judge.RED_TO_PASS], [judge.PROBLEM])

    def test_a_dropped_invariant_without_an_entry_of_its_own_is_unplaced(self) -> None:
        findings = findings_of([self.OLD], [entry("I-5.1", "I-5.1")], context())
        self.assertEqual([finding.result for finding in findings if finding.cls == judge.RED_TO_PASS], [judge.UNPLACED])


class OtherDrift(unittest.TestCase):
    def test_a_digest_only_change_is_normal_drift(self) -> None:
        old = bad_row(1, "not_in", "-") + " base_digest=aa"
        new = bad_row(1, "not_in", "-") + " base_digest=bb"
        [finding] = findings_of([old], [new], context())
        self.assertEqual((finding.cls, finding.result, finding.reason), (judge.OTHER_DRIFT, judge.NORMAL, "只差摘要"))

    def test_a_changed_judgement_word_is_not_drift(self) -> None:
        [finding] = findings_of(["name=s1 stream=a check=x holds=true"], ["name=s1 stream=a check=x holds=false"], context())
        self.assertEqual(finding.result, judge.UNPLACED)

    def test_a_count_named_after_a_green_column_is_drift_not_a_judgement(self) -> None:
        old = "name=batch stream=a batch=1 states=53 o1_crash_judge_green=53"
        new = "name=batch stream=a batch=1 states=61 o1_crash_judge_green=61"
        [finding] = findings_of([old], [new], context())
        self.assertEqual(finding.result, judge.NORMAL)

    def test_a_row_only_in_one_product_is_unplaced_unless_it_is_a_known_kind(self) -> None:
        [finding] = findings_of(["name=call stream=a call=1 x=1"], [], context())
        self.assertEqual(finding.result, judge.UNPLACED)
        [added] = findings_of([], [entry("I-1.5", "I-1.5")], context())
        self.assertEqual(added.result, judge.NORMAL)
        [added_not_red] = findings_of([], [entry("I-1.5", "-", target_is_red="false")], context())
        self.assertEqual(added_not_red.result, judge.PROBLEM)

    def test_identical_products_have_no_findings_and_a_clean_verdict(self) -> None:
        lines = [bad_row(1, "not_in", "I-4.2")]
        self.assertEqual(findings_of(lines, lines, context()), [])


class Accounting(unittest.TestCase):
    def test_the_class_lines_add_up_to_the_diff_lines_and_a_clean_run_is_ok(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            old, new, source, invariants = (os.path.join(directory, name) for name in ("old.out", "new.out", "recovery.rs", "invariants"))
            os.mkdir(invariants)
            with open(os.path.join(invariants, "03.md"), "w", encoding="utf-8") as handle:
                handle.write("| I-8.14 | x | y |\n| I-4.2 | x | y |\n")
            with open(source, "w", encoding="utf-8") as handle:
                handle.write("\n".join(SOURCE_LINES) + "\n")
            for path, rows in ((old, [bad_row(1, "not_in", "-"), bad_row(2, "not_in", "I-4.2")]), (new, [bad_row(1, "not_in", "I-8.14"), bad_row(2, "not_in", "I-4.2")])):
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write("".join(f"E7RESULT {row}\n" for row in rows))
            lines: list[str] = []
            ok = judge.judge_files(old, new, None, invariants, source, None, emit=lines.append)
            self.assertTrue(ok, lines)
            self.assertIn("name=v6_accounting diff_lines=2 classified_lines=2 equal=true", lines)

    def test_a_pair_counts_two_lines_and_a_one_sided_row_counts_one(self) -> None:
        pairs = judge.classify(rows_of(bad_row(1, "not_in", "-"), "name=call stream=a call=1 x=1"), rows_of(bad_row(1, "not_in", "I-8.14")), context())
        self.assertEqual(sorted(pair.lines for pair in pairs), [1, 2])


PINNED_TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "pinned-allowances.tsv")


def named_bad_row(row: int, name: str, expected: str, red: str, stream: str = "s") -> str:
    return (f"name=bad_row stream={stream} row={row} name={name} family=N base=F patches=1 expected={expected} expected_landing=none "
            f"impl_consulted_outcome=file_read impl_pool_checker_red={red} impl_both_pass=false")


def allowance(row: str = "13", name: str = "N1-F", invariant: str = "I-3.21", experiment_row: str = "E171.3", stream: str = "s") -> judge.Allowance:
    return judge.Allowance(experiment_row, stream, row, name, invariant, "理由", "依据")


DEFINED_WITH_I321 = DEFINED | {"I-3.21"}


def allowance_context(*allowances: judge.Allowance, names=None) -> judge.Context:
    return judge.Context({}, DEFINED_WITH_I321, SOURCE_LINES, {}, list(allowances), names or {})


class PinnedAllowances(unittest.TestCase):
    OLD = named_bad_row(13, "N1-F", "in", "I-4.2")

    def test_a_pinned_new_red_on_the_pinned_row_is_normal(self) -> None:
        [finding] = findings_of([self.OLD], [named_bad_row(13, "N1-F", "in", "I-4.2,I-3.21")], allowance_context(allowance()))
        self.assertEqual((finding.cls, finding.result), (judge.STRICTER, judge.NORMAL))

    def test_without_the_table_the_same_row_is_a_problem(self) -> None:
        [finding] = findings_of([self.OLD], [named_bad_row(13, "N1-F", "in", "I-4.2,I-3.21")], allowance_context())
        self.assertEqual(finding.result, judge.PROBLEM)

    def test_another_new_red_on_the_pinned_row_is_still_a_problem(self) -> None:
        findings = findings_of([self.OLD], [named_bad_row(13, "N1-F", "in", "I-4.2,I-3.21,I-8.14")], allowance_context(allowance()))
        self.assertEqual([finding.result for finding in findings], [judge.PROBLEM])
        self.assertIn("I-8.14", findings[0].reason)

    def test_an_allowance_for_the_wrong_row_leaves_the_real_row_a_problem_and_the_entry_a_problem(self) -> None:
        findings = findings_of([self.OLD], [named_bad_row(13, "N1-F", "in", "I-4.2,I-3.21")], allowance_context(allowance(row="14")))
        self.assertEqual(sorted(finding.result for finding in findings), [judge.PROBLEM, judge.PROBLEM])

    def test_an_allowance_naming_a_different_row_than_the_product_has_is_not_applied(self) -> None:
        findings = findings_of([self.OLD], [named_bad_row(13, "N2-F", "in", "I-4.2,I-3.21")], allowance_context(allowance()))
        self.assertEqual([finding.result for finding in findings], [judge.PROBLEM, judge.PROBLEM])

    def test_a_stale_allowance_is_a_problem(self) -> None:
        findings = findings_of([self.OLD], [self.OLD], allowance_context(allowance()))
        self.assertEqual([finding.result for finding in findings], [judge.PROBLEM])
        self.assertIn("防表腐烂", findings[0].reason)

    def test_the_name_of_a_row_without_one_comes_from_the_labels_names(self) -> None:
        old = "name=e170_bad_row stream=s row=13 o2=red o2_red=I-4.2"
        new = "name=e170_bad_row stream=s row=13 o2=red o2_red=I-4.2,I-3.21"
        context = allowance_context(allowance(), names={("s", "13"): "N1-F"})
        context.labels = {("s", "13"): "in"}
        [finding] = findings_of([old], [new], context)
        self.assertEqual(finding.result, judge.NORMAL)
        without_names = allowance_context(allowance())
        without_names.labels = {("s", "13"): "in"}
        self.assertEqual(sorted(f.result for f in findings_of([old], [new], without_names)), [judge.PROBLEM, judge.PROBLEM])

    def test_the_audit_lines_do_not_count_as_product_lines(self) -> None:
        pairs = judge.classify(rows_of(self.OLD), rows_of(self.OLD), allowance_context(allowance()))
        self.assertEqual([pair.lines for pair in pairs], [0])


class AllowanceTableFile(unittest.TestCase):
    def write(self, directory: str, text: str) -> str:
        path = os.path.join(directory, "table.tsv")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        return path

    ROW = "E171.3\ts\t13\tN1-F\tI-3.21\t理由\t依据"

    def test_a_well_formed_table_loads_and_comments_are_skipped(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            [entry] = judge.load_allowances(self.write(directory, "# 注释\n\n" + self.ROW + "\n"), DEFINED_WITH_I321)
            self.assertEqual((entry.experiment_row, entry.row, entry.name, entry.invariant), ("E171.3", "13", "N1-F", "I-3.21"))

    def test_an_extra_column_or_a_missing_column_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            for text in (self.ROW + "\t多一列\n", self.ROW.rsplit("\t", 1)[0] + "\n"):
                with self.assertRaises(judge.AllowanceTableError):
                    judge.load_allowances(self.write(directory, text), DEFINED_WITH_I321)

    def test_an_invariant_without_a_definition_line_or_a_duplicate_cell_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(judge.AllowanceTableError):
                judge.load_allowances(self.write(directory, self.ROW.replace("I-3.21", "I-9.99") + "\n"), DEFINED_WITH_I321)
            with self.assertRaises(judge.AllowanceTableError):
                judge.load_allowances(self.write(directory, self.ROW + "\n" + self.ROW + "\n"), DEFINED_WITH_I321)

    def test_an_allowance_table_needs_an_experiment_row(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            table = self.write(directory, self.ROW + "\n")
            with self.assertRaises(judge.AllowanceTableError):
                judge.judge_files(table, table, None, directory, table, None, allowances_path=table, emit=lambda line: None)

    def test_the_pinned_table_holds_exactly_the_seven_decided_rows(self) -> None:
        entries = judge.load_allowances(PINNED_TABLE, DEFINED_WITH_I321)
        self.assertEqual([(entry.experiment_row, entry.stream, entry.row, entry.name, entry.invariant) for entry in entries],
                         [(experiment_row, "fixed_script_through_e_then_unmount", "13", "N1-F", "I-3.21")
                          for experiment_row in ("E171.2", "E171.3", "E171.4", "E171.7", "E171.9", "E171.10", "E171.11")])


class LabelsFromTheSeedIndependentPrefix(unittest.TestCase):
    def labelled(self, blind_expected: str = "not_in", prefix_second: int = 2) -> collections.OrderedDict:
        lines = []
        for stream in ("a", "b"):
            kept = prefix_second if stream == "b" else 2
            for row in range(kept):
                lines.append(f"name=bad_row stream={stream} row={row} name={'P' if row == 0 else 'N'}1 family={'P' if row == 0 else 'N'} expected={'not_in' if row == 0 else 'in'}")
            for row in range(kept, 4):
                lines.append(f"name=bad_row stream={stream} row={row} name=B{row} family=B expected={blind_expected}")
        return rows_of(*lines)

    def judged(self) -> collections.OrderedDict:
        return rows_of(*[f"name=e170_bad_row stream={stream} row={row} o2=red o2_red=-" for stream in ("a", "b") for row in range(4)])

    def run_labels(self, labelled: collections.OrderedDict, mode: str):
        with tempfile.TemporaryDirectory() as directory:
            path = os.path.join(directory, "labels.out")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("".join(row.text + "\n" for row in labelled.values()))
            return judge.label_table(path, self.judged(), mode)

    def test_the_prefix_rows_keep_their_labels_and_the_blind_rows_are_not_in(self) -> None:
        labels, names, note = self.run_labels(self.labelled(), "seed-independent-prefix")
        self.assertEqual((labels[("a", "0")], labels[("a", "1")], labels[("a", "2")], labels[("b", "3")]), ("not_in", "in", "not_in", "not_in"))
        self.assertEqual((names.get(("a", "1")), names.get(("a", "2"))), ("N1", None))
        self.assertEqual(note, "ok-prefix-2")

    def test_a_blind_row_that_is_not_not_in_or_prefixes_of_different_length_give_no_labels(self) -> None:
        self.assertEqual(self.run_labels(self.labelled(blind_expected="in"), "seed-independent-prefix")[0], {})
        self.assertEqual(self.run_labels(self.labelled(prefix_second=3), "seed-independent-prefix")[0], {})

    def test_the_same_table_mode_takes_every_row_as_it_is(self) -> None:
        labels, _, _ = self.run_labels(self.labelled(blind_expected="in"), "same-table")
        self.assertEqual(labels[("a", "2")], "in")


class TableMovedShape(unittest.TestCase):
    def test_the_four_removed_checks_and_the_one_count_row_are_normal_drift(self) -> None:
        removed = [f"name=s4 check={check} local=1 crates=1 holds=true" for check in sorted(judge.TABLE_MOVE_REMOVED_CHECKS)]
        pairs = judge.classify(rows_of(*removed), rows_of(entry("I-1.1", "I-1.1"), "name=known_bad_entry_count count=1"), context())
        self.assertEqual({finding.result for pair in pairs for finding in pair.findings if pair.new is None or pair.new.kind == "known_bad_entry_count"}, {judge.NORMAL})

    def test_a_count_that_disagrees_with_the_entry_rows_is_unplaced(self) -> None:
        [finding] = [finding for pair in judge.classify(rows_of(entry("I-1.1", "I-1.1")), rows_of(entry("I-1.1", "I-1.1"), "name=known_bad_entry_count count=2"), context())
                     for finding in pair.findings]
        self.assertEqual(finding.result, judge.UNPLACED)

    def test_another_removed_s4_check_stays_unplaced(self) -> None:
        [finding] = findings_of(["name=s4 check=something_else holds=true"], [], context())
        self.assertEqual(finding.result, judge.UNPLACED)


if __name__ == "__main__":
    unittest.main()
