"""new_o3_executor_checks.py 的单测：各检查在合成的小输入上分得出红与绿（每条红的用例对应登记第九节的一条变异或第六节的一个门槛）。

跑法（在本目录）：python3 -m unittest -v test_new_o3_executor_checks
真目录相关的用例（种下的故障每一处替换在新目录里恰好命中一次）读 ../e170_second_run_o3_spec_executor_journal_header_496/。
"""

from __future__ import annotations

import contextlib
import io
import os
import tempfile
import unittest

import new_o3_executor_checks as checks

NEW_EXECUTOR_DIRECTORY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "e170_second_run_o3_spec_executor_journal_header_496")
SCRATCH_ROOT = os.environ.get("E171_CHECKS_TEST_SCRATCH")


def quietly(function, *arguments):
    with contextlib.redirect_stdout(io.StringIO()):
        return function(*arguments)


def write_files(directory: str, files: dict[str, str]) -> None:
    os.makedirs(directory, exist_ok=True)
    for name, text in files.items():
        with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
            handle.write(text)


GOOD_OUTPUT = [
    "name=config dumps=d streams=new_pool_file_creation,fixed_script_through_e_then_unmount batches=1,2,3 workers=6",
    "name=load stream=new_pool_file_creation load_ns=1 prepare_ns=2",
    "name=dump_check stream=new_pool_file_creation count_mismatches=0 checksum_mismatches=0 version_table_anomalies=0",
    "name=anchor_assertion id=A6 stream=new_pool_file_creation holds=true versions=4",
    "name=anchor_assertion id=A1 stream=new_pool_file_creation holds=true landed=(1, 3) allowed=[(1, 2), (1, 3)]",
    "name=anchor_assertion id=A2 stream=new_pool_file_creation holds=true roots=[]",
    "name=anchor_assertion id=A3 stream=new_pool_file_creation holds=true ring={}",
    "name=anchor_assertion id=A4 stream=new_pool_file_creation holds=true chosen={}",
    "name=anchor_assertion id=A5 stream=new_pool_file_creation holds=true versions=[]",
    "name=anchor_assertion id=A7 stream=new_pool_file_creation holds=true",
    "name=torn_reading stream=new_pool_file_creation executor_third_state_writes=3 dumped_third_state_writes=3 differing_entries=0",
    "name=load stream=fixed_script_through_e_then_unmount load_ns=1 prepare_ns=2",
    "name=dump_check stream=fixed_script_through_e_then_unmount count_mismatches=0 checksum_mismatches=0 version_table_anomalies=0",
    "name=anchor_assertion id=A6 stream=fixed_script_through_e_then_unmount holds=true versions=9",
    "name=anchor_assertion id=A11 stream=fixed_script_through_e_then_unmount holds=true chosen_root_flags=1 landed=(1, 9)",
    "name=torn_reading stream=fixed_script_through_e_then_unmount executor_third_state_writes=3 dumped_third_state_writes=3 differing_entries=0",
    "name=verdict batch=1 stream=new_pool_file_creation states=4",
    "name=complete",
]


class ReadV2(unittest.TestCase):
    def read(self, lines):
        _text, ok, failing = checks.read_v2_lines(lines)
        return ok, failing

    def test_a_full_good_output_passes_every_cell(self):
        self.assertEqual(self.read(GOOD_OUTPUT), (True, []))

    def test_recovered_false_on_either_stream_is_red(self):
        for stream in checks.STREAM_NAMES:
            lines = [line for line in GOOD_OUTPUT if f"stream={stream}" not in line or line.startswith("name=verdict")]
            lines.insert(1, f"name=base_recovery stream={stream} recovered=false detail=no_choosable_system_configuration_on_any_device")
            ok, failing = self.read(lines)
            self.assertFalse(ok)
            self.assertIn(f"base_recovery:{stream}", failing)

    def test_an_anchor_that_does_not_hold_or_is_not_run_or_is_absent_is_red(self):
        for replacement in ("holds=false", "holds=not_run"):
            lines = [line.replace("id=A3 stream=new_pool_file_creation holds=true", f"id=A3 stream=new_pool_file_creation {replacement}") for line in GOOD_OUTPUT]
            self.assertEqual(self.read(lines), (False, [f"anchor_A3:{checks.STREAM_NAMES[0]}"]))
        lines = [line for line in GOOD_OUTPUT if "id=A3 " not in line]
        self.assertEqual(self.read(lines), (False, [f"anchor_A3:{checks.STREAM_NAMES[0]}"]))

    def test_the_informational_anchor_and_the_verdict_line_are_reported_but_do_not_gate(self):
        text, ok, _failing = checks.read_v2_lines([line.replace("id=A7 stream=new_pool_file_creation holds=true", "id=A7 stream=new_pool_file_creation holds=false")
                                                   for line in GOOD_OUTPUT])
        self.assertTrue(ok)
        self.assertIn("name=v2_info anchor=A7 stream=new_pool_file_creation holds=false", text)
        self.assertTrue(any(line.startswith("name=v2_info verdict_line=") for line in text))

    def test_any_nonzero_dump_check_count_or_torn_difference_is_red(self):
        lines = [line.replace("checksum_mismatches=0", "checksum_mismatches=2") if "fixed_script" not in line else line for line in GOOD_OUTPUT]
        self.assertEqual(self.read(lines), (False, [f"dump_check:{checks.STREAM_NAMES[0]}"]))
        lines = [line.replace("differing_entries=0", "differing_entries=1") if "fixed_script" in line else line for line in GOOD_OUTPUT]
        self.assertEqual(self.read(lines), (False, [f"torn_reading:{checks.STREAM_NAMES[1]}"]))

    def test_a_missing_complete_marker_is_red(self):
        self.assertEqual(self.read(GOOD_OUTPUT[:-1]), (False, ["complete_marker:-"]))

    def test_an_empty_output_is_red_everywhere(self):
        ok, failing = self.read([])
        self.assertFalse(ok)
        self.assertIn(f"base_recovery:{checks.STREAM_NAMES[0]}", failing)
        self.assertIn("complete_marker:-", failing)


class Footprint(unittest.TestCase):
    def directories(self, base_extra=None, new_extra=None):
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        names = set(checks.CHANGED_FILES_ALLOWED) | set(checks.UNCHANGED_FILES)
        base, new = os.path.join(temporary.name, "base"), os.path.join(temporary.name, "new")
        main_name = checks.EXECUTOR_MAIN_FILE
        write_files(base, {name: f"base {name}" for name in names} | {main_name: self.MAIN_BASE} | (base_extra or {}))
        write_files(new, {name: f"base {name}" for name in names} | {name: f"new {name}" for name in checks.CHANGED_FILES_ALLOWED}
                    | {main_name: self.MAIN_WITH_THE_NEW_PIN} | (new_extra or {}))
        return base, new

    MAIN_BASE = "x = 1\nemit(f\"name=anchor_assertion id=A4 holds={all(v == (5, 1, 3, 1) for v in c)}\")\ny = 2\n"
    MAIN_WITH_THE_NEW_PIN = MAIN_BASE.replace("(5, 1, 3, 1)", "(6, 0, 3, 1)")

    def test_the_nine_allowed_files_changed_and_the_six_equal_passes(self):
        base, new = self.directories()
        lines, ok = quietly(checks.footprint_lines, base, new)
        self.assertTrue(ok, lines)
        self.assertIn("name=v1_outside_allowed count=0 files=-", lines)
        self.assertIn("name=v1_changed_files count=9 files=" + ",".join(sorted(checks.CHANGED_FILES_ALLOWED)), lines)

    def test_an_anchor_edit_in_executor_main_other_than_the_a4_pin_is_red_m10(self):
        for edited in (self.MAIN_WITH_THE_NEW_PIN.replace("y = 2", "y = 3"),
                       self.MAIN_BASE.replace("id=A4", "id=A5"),
                       self.MAIN_WITH_THE_NEW_PIN.replace("(6, 0, 3, 1)", "(6, 0, 3, 2)")):
            base, new = self.directories(new_extra={"executor_main.py": edited})
            lines, ok = quietly(checks.footprint_lines, base, new)
            self.assertFalse(ok, edited)
            self.assertIn("name=v1_executor_main_diff only_the_a4_pin=false before=(5, 1, 3, 1) after=(6, 0, 3, 1)", lines)

    def test_executor_main_left_with_the_old_a4_pin_is_red(self):
        base, new = self.directories(new_extra={"executor_main.py": self.MAIN_BASE})
        self.assertFalse(quietly(checks.footprint_lines, base, new)[1])

    def test_a_change_in_one_of_the_six_untouched_files_is_red(self):
        base, new = self.directories(new_extra={"root_ring.py": "changed"})
        lines, ok = quietly(checks.footprint_lines, base, new)
        self.assertFalse(ok)
        self.assertTrue(any("file=root_ring.py" in line and "equal=false" in line for line in lines))

    def test_a_file_outside_the_allowed_set_is_red(self):
        base, new = self.directories(new_extra={"extra_helper.py": "x"})
        lines, ok = quietly(checks.footprint_lines, base, new)
        self.assertFalse(ok)
        self.assertIn("name=v1_outside_allowed count=1 files=extra_helper.py", lines)

    def test_manifest_lines_are_sha256sum_shaped_sorted_by_path_and_skip_pycache(self):
        base, _new = self.directories()
        os.makedirs(os.path.join(base, "__pycache__"))
        write_files(os.path.join(base, "__pycache__"), {"x.cpython.pyc": "bytecode"})
        manifest = checks.manifest_lines([base])
        self.assertEqual([line.split("  ", 1)[1] for line in manifest], sorted(line.split("  ", 1)[1] for line in manifest))
        self.assertTrue(all(len(line.split("  ", 1)[0]) == 64 for line in manifest))
        self.assertFalse(any("__pycache__" in line for line in manifest))
        self.assertEqual(len(manifest), 15)


class StaticIndependence(unittest.TestCase):
    def directory(self, files: dict[str, str]) -> str:
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        write_files(temporary.name, files)
        return temporary.name

    def test_import_of_a_third_party_module_is_flagged_and_standard_sibling_and_optional_ones_are_not(self):
        directory = self.directory({"a.py": "import os\nimport b\nfrom b import thing\nimport google_crc32c\n", "b.py": "thing = 1\n"})
        self.assertEqual(checks.import_violations(directory), [])
        directory = self.directory({"a.py": "import os\nimport requests\n"})
        self.assertEqual(checks.import_violations(directory), ["a.py:2: import requests"])

    def test_open_with_a_command_line_path_or_a_fixed_dump_file_name_passes_and_any_other_path_is_flagged(self):
        allowed = "import os\ndef f(path, directory):\n    open(path)\n    open(os.path.join(directory, 'manifest.tsv'), 'rb')\n"
        self.assertEqual(checks.open_violations(self.directory({"a.py": allowed})), [])
        for forbidden in ("open('crates/x.rs')", "open(os.path.join(directory, 'secret.txt'))", "open('/etc/passwd')"):
            violations = checks.open_violations(self.directory({"a.py": f"import os\ndef f(directory):\n    {forbidden}\n"}))
            self.assertTrue(violations, forbidden)

    def test_a_repository_path_shaped_string_outside_documentation_is_flagged(self):
        violations = checks.open_violations(self.directory({"a.py": '"""文档串里写 crates/ 不算"""\nX = "scripts/hosts/run.sh"\n'}))
        self.assertEqual(len(violations), 1)
        self.assertIn("a.py:2", violations[0])

    def test_i1_flags_a_crates_hit_that_the_base_directory_does_not_have_and_passes_identical_hits_m9(self):
        base = self.directory({"a.py": "x = 1  # 与 crates/ 无关\n"})
        same = self.directory({"a.py": "x = 1  # 与 crates/ 无关\n"})
        self.assertEqual(checks.static_independence_findings(same, base)["I1"], [])
        different = self.directory({"a.py": "x = 1  # 与 crates/ 无关\nopen('crates/y')\n"})
        self.assertTrue(checks.static_independence_findings(different, base)["I1"])

    def test_i5_the_seeded_open_makes_both_i1_and_i2_red_while_the_clean_copy_is_green(self):
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        source = os.path.join(temporary.name, "source")
        write_files(source, {"system_configuration.py": "import os\n", "other.py": "import system_configuration\n"})
        clean = checks.static_independence_findings(source, source)
        self.assertEqual((clean["I1"], clean["I2"]), ([], []))
        seeded = os.path.join(temporary.name, "seeded")
        checks.seeded_copy_with_crates_open(source, seeded)
        findings = checks.static_independence_findings(seeded, source)
        self.assertTrue(findings["I1"] and findings["I2"])

    def test_i4_a_change_with_a_kb_source_nearby_passes_and_one_without_is_listed(self):
        base = "a = 1\nb = 2\n"
        cited = "a = 1\n# .claude/kb/layout/01-first-txn.md@六、journal\nb = 3\n"
        uncited = "a = 1\nb = 3  # D23 已定项 4\n"
        self.assertEqual(checks.uncited_changes(base, cited), [])
        self.assertEqual(checks.uncited_changes(base, uncited), [(2, "b = 3  # D23 已定项 4")])
        self.assertEqual(checks.uncited_changes("a = 1\n", "a = 1\n"), [])
        self.assertEqual(checks.uncited_changes(base, "a = 1\nb = 3\n"), [], "只改代码、不带注释的区间不查")
        self.assertEqual(checks.uncited_changes(base, "a = 1\nb = 3  # .claude/kb/x.md 没有小节标题\n"),
                         [(2, "b = 3  # .claude/kb/x.md 没有小节标题")], "出处要写成 文件@小节标题")

    def test_i4_a_cited_kb_file_that_is_not_in_the_repository_is_red(self):
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        root, base, new = (os.path.join(temporary.name, name) for name in ("root", "base", "new"))
        write_files(base, {"a.py": "x = 1\n"})
        write_files(new, {"a.py": "x = 1\n# .claude/kb/layout/missing.md@标题\ny = 2\n"})
        write_files(os.path.join(root, ".claude", "kb", "layout"), {})
        self.assertFalse(quietly(checks.citation_lines, [], base, new, root))
        write_files(os.path.join(root, ".claude", "kb", "layout"), {"missing.md": "# 标题\n"})
        self.assertTrue(quietly(checks.citation_lines, [], base, new, root))

    def test_i4_a_crates_path_in_a_changed_line_is_listed_but_not_in_an_unchanged_one(self):
        base = "# 与 crates/ 无关\nx = 1\n"
        self.assertEqual(checks.crates_in_changed_lines(base, base), [])
        self.assertEqual(checks.crates_in_changed_lines(base, base + "# 抄自 crates/a.rs\n"), [(3, "# 抄自 crates/a.rs")])

    def test_normalised_output_drops_timing_lines_time_words_worker_numbers_and_the_dump_path_and_sorts(self):
        lines = ["name=timing batch=1 total_ns=5", "name=state_time batch=1 median_ns=3",
                 "name=state stream=s state=2 time_ns=9 worker=77", "name=config dumps=/x/y workers=6", "name=state stream=s state=1 time_ns=1 worker=78"]
        self.assertEqual(checks.normalised_output(lines),
                         ["name=config workers=6", "name=state stream=s state=1", "name=state stream=s state=2"])


class SeededEditsLandOnTheRealDirectory(unittest.TestCase):
    """种下的故障与第九节单测变异的每一处替换，在新目录里恰好命中一次（命中 0 次或 2 次时 apply_edits 抛 ValueError）。"""

    def apply_in_a_copy(self, edits):
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        work = os.path.join(temporary.name, "copy")
        checks.copy_executor_directory(NEW_EXECUTOR_DIRECTORY, work)
        checks.apply_edits(work, tuple(edits))
        return work

    def test_every_seeded_fault_applies_exactly_once_and_changes_a_file(self):
        for fault in checks.SEEDED_FAULTS:
            work = self.apply_in_a_copy(fault.edits)
            changed = [edit.file for edit in fault.edits
                       if checks.read_directory_files(work)[edit.file] != checks.read_directory_files(NEW_EXECUTOR_DIRECTORY)[edit.file]]
            self.assertEqual(sorted(set(changed)), sorted({edit.file for edit in fault.edits}), fault.name)

    def test_every_unit_mutation_applies_exactly_once(self):
        for name, edit, _fragment in checks.unit_mutation_table():
            self.apply_in_a_copy([edit])

    def test_an_edit_that_hits_zero_times_or_twice_is_refused(self):
        temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        self.addCleanup(temporary.cleanup)
        write_files(temporary.name, {"f.py": "a\na\n"})
        for old in ("absent", "a"):
            with self.assertRaises(ValueError):
                checks.apply_edits(temporary.name, (checks.Edit("f.py", old, "b"),))

    def test_the_registered_faults_name_the_cells_the_registration_names(self):
        registered = {fault.name: fault for fault in checks.SEEDED_FAULTS if fault.expected_cells}
        self.assertEqual(sorted(registered), ["P-a", "P-b", "P-c", "P-c-misplaced-rows"])
        self.assertEqual(registered["P-c"].expected_cells, (f"anchor_A2:{checks.STREAM_NAMES[0]}",))
        self.assertTrue(registered["P-c"].crash_counts)
        self.assertFalse(registered["P-c-misplaced-rows"].crash_counts)
        self.assertEqual(registered["P-c-misplaced-rows"].expected_cells,
                         (f"anchor_A2:{checks.STREAM_NAMES[0]}", f"base_recovery:{checks.STREAM_NAMES[0]}"))
        self.assertEqual(registered["P-b"].expectation, "any")


class PerStateComparison(unittest.TestCase):
    HISTORY = "history\twrites=21\ttorn=0,1\tvectors=3\tskeleton_sha256=abc"

    def compare(self, frozen, new):
        return quietly(checks.compare_state_tables, frozen, new)

    def test_identical_tables_pass(self):
        rows = [self.HISTORY, "222\tin\tin\t(1,3,wm=1,floor=0)\tp1", "200\tin\tin\t(1,2,wm=1,floor=0)\tp2"]
        self.assertTrue(self.compare(rows, list(rows)))

    def test_one_differing_landed_text_or_projection_or_verdict_is_red(self):
        frozen = [self.HISTORY, "222\tin\tin\t(1,3,wm=1,floor=0)\tp1"]
        for changed in ("222\tin\tnot_in\t(1,3,wm=1,floor=0)\tp1", "222\tin\tin\t(1,2,wm=1,floor=0)\tp1", "222\tin\tin\t(1,3,wm=1,floor=0)\tpX"):
            self.assertFalse(self.compare(frozen, [self.HISTORY, changed]), changed)

    def test_a_state_missing_on_one_side_or_a_different_abstract_history_is_red(self):
        frozen = [self.HISTORY, "222\tin\tin\tt\tp"]
        self.assertFalse(self.compare(frozen, [self.HISTORY]))
        self.assertFalse(self.compare(frozen, [self.HISTORY.replace("abc", "abd"), "222\tin\tin\tt\tp"]))
        self.assertFalse(self.compare([], []))

    def test_error_rows_are_red_even_when_both_sides_agree(self):
        rows = [self.HISTORY, "222\terror\terror\tKeyError\t-"]
        self.assertFalse(self.compare(rows, list(rows)))

    def test_sampled_landing_vectors_cover_prefixes_single_misses_and_torn_writes_without_duplicates(self):
        vectors = checks.sampled_landing_vectors(21, [0, 5])
        self.assertEqual(len(vectors), len(set(vectors)))
        self.assertTrue(all(len(vector) == 21 and set(vector) <= set("012") for vector in vectors))
        self.assertIn("2" * 21, vectors)
        self.assertIn("0" * 21, vectors)
        self.assertIn("2" * 10 + "0" + "2" * 10, vectors)
        self.assertIn("2" * 5 + "1" + "0" * 15, vectors)
        self.assertIn("1" + "2" * 20, vectors)


if __name__ == "__main__":
    unittest.main()
