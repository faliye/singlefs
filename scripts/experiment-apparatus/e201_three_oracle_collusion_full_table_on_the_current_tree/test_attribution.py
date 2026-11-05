"""attribution.py 的单测：跑前登记第五节 5.2 第二张表「Q1·有格变且归得到」「有的归不到」「只在一边」的归因那一半、F6，与第九节 V6–V9 各自会红的那一格。
参照树、今天的树、装置与研究一侧都是每条用例自己在临时目录里造的小文件（单元照真的 lib_function_units 切），不读真的仓。
跑法：在本目录 `python3 -m unittest test_attribution`。"""

from __future__ import annotations

import os
import tempfile
import unittest
from unittest import mock

import attribution
import e171_research_side

CORE_FILE = "crates/singlefs-core/src/allocator_probe.rs"
REFERENCE_FILESYSTEM_FILE = "crates/singlefs-reference-filesystem/src/lib.rs"
DEVICE_FILE = "crates/singlefs-verification/src/bin/device_probe.rs"
CORE_TODAY = "fn alpha() {\n    let anchor_here = 1;\n}\n\nfn beta() {\n    let other = 2;\n}\n"
REFERENCE_FILESYSTEM_TODAY = "fn submit() {\n    let a = 1;\n}\nfn allowed_set_for_a_crash_in() {\n    let b = 2;\n}\nfn crash_and_recover() {}\n"
DEVICE_TEXT = "fn judge_state() {\n    let verdict = 1;\n}\n"
RESEARCH_TEXT = "def state_is_red(state):\n    return state == 'red'\n"
DEVICE_MAP = "group\tjudging\tjudge_state\ncell\tP1\trow\t*\tjudging\ncell\tP3\tq1_row\t*\tjudging\n"
RESEARCH_MAP = "cell\tP1\trow\t*\tcollusion_matrix.py::state_is_red\ncell\tP3\tq1_row\t*\tcollusion_matrix.py::state_is_red\n"


class Mutation:
    def __init__(self, name: str, file: str, original: str):
        self.name, self.file, self.original = name, file, original

    def original_text(self) -> str:
        return self.original


def write(root: str, relative: str, text: str) -> None:
    os.makedirs(os.path.dirname(os.path.join(root, relative)), exist_ok=True)
    with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
        handle.write(text)


def make_inputs(directory: str, core_reference: str = CORE_TODAY, reference_filesystem_reference: str = REFERENCE_FILESYSTEM_TODAY,
                p2_sha: str = "a" * 64, research_reference: str | None = RESEARCH_TEXT, p0_paths=frozenset()) -> attribution.AttributionInputs:
    reference, today, research = (os.path.join(directory, name) for name in ("reference", "today", "research"))
    write(reference, CORE_FILE, core_reference)
    write(today, CORE_FILE, CORE_TODAY)
    write(reference, REFERENCE_FILESYSTEM_FILE, reference_filesystem_reference)
    write(today, REFERENCE_FILESYSTEM_FILE, REFERENCE_FILESYSTEM_TODAY)
    write(today, DEVICE_FILE, DEVICE_TEXT)
    write(research, os.path.join(e171_research_side.E171_RELATIVE, "collusion_matrix.py"), RESEARCH_TEXT)
    row = {"mutated_row_name": "m15", "substituted_from": "-", "status": "ok"}
    return attribution.AttributionInputs(
        reference_root=reference, today_root=today, today_research_root=research, r1_device=DEVICE_TEXT, device_marker="exact",
        today_device_path=DEVICE_FILE, p0_paths=frozenset(p0_paths), uncommitted_paths=frozenset(),
        p2_rows={15: dict(row, mutation_line_sha256=p2_sha)}, t2_rows={15: dict(row, mutation_line_sha256="a" * 64)},
        today_mutations={"m15": Mutation("m15", CORE_FILE, "let anchor_here = 1;")}, call_labels={},
        device_map=attribution.read_function_map(DEVICE_MAP), research_map=attribution.read_function_map(RESEARCH_MAP),
        research_reference={"collusion_matrix.py": research_reference}, scratch_root=os.path.join(directory, "scratch"))


def cell(name: str = "category_registration_batch_1", key: str = "row:15", kind: str = "row", product: str = "P1", cell_class: str = "decision",
         r1: str = "counted", today: str = "untouched") -> dict[str, str]:
    return {"product": product, "kind": kind, "key": key, "field": name, "class": cell_class, "r1": r1, "today": today, "batch": "1"}


class SevenKinds(unittest.TestCase):
    def test_nothing_changed_anywhere_is_unattributable(self):
        """5.2「Q1·有格变有的归不到」：七种都不中，记归不到；Q1 判「有的格归不到」。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory)).attribute(cell())
        self.assertEqual(result.status, attribution.STATUS_UNATTRIBUTABLE)
        self.assertEqual(result.outcomes, {"g1": "miss", "g2": "miss", "g3": "miss", "g4": "miss", "g5": "miss", "g6": "na", "g7": "miss"})
        self.assertEqual(attribution.q1_verdict(1, 1, 0), "some_cells_unattributable")

    def test_the_landing_function_changed_is_g4(self):
        """5.2「Q1·有格变且归得到」：参照树里落点所在的函数 alpha 不同，G4 中。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory, core_reference=CORE_TODAY.replace("= 1;", "= 7;"))).attribute(cell())
        self.assertEqual((result.status, result.outcomes["g4"]), (attribution.STATUS_ATTRIBUTED, "hit"))

    def test_another_function_of_the_same_file_changed_is_not_g4(self):
        """V7：同文件另一个函数 beta 变了、落点函数 alpha 没变，G4 不中（按单元指纹比，不比整份文件）。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory, core_reference=CORE_TODAY.replace("= 2;", "= 9;"))).attribute(cell())
        self.assertEqual(result.outcomes["g4"], "miss")

    def test_a_landing_file_in_p0_is_marked_reference_approximate(self):
        """V8：落点文件在 P0 那几份路径里，那一条带 reference_approximate。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory, p0_paths={CORE_FILE})).attribute(cell())
        self.assertIn(f"reference_approximate:{CORE_FILE}", result.notes)

    def test_a_changed_mutation_line_with_the_same_name_is_g2(self):
        """V6：名字相同、变异行 sha256 不同，G2 中。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory, p2_sha="b" * 64)).attribute(cell())
        self.assertEqual((result.status, result.outcomes["g2"]), (attribution.STATUS_ATTRIBUTED, "hit"))

    def test_a_withdrawn_row_present_only_in_r1_is_g1(self):
        """5.2「Q1·只在一边」：r1 有第 236 行、今天没有，那一格 G1 中。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory)).attribute(cell(name="<present>", key="row:236", r1="present", today="<absent>"))
        self.assertEqual((result.status, result.outcomes["g1"]), (attribution.STATUS_ATTRIBUTED, "hit"))

    def test_a_changed_reference_filesystem_entry_is_g6_for_the_reference_filesystem_column(self):
        """V9：只有 allowed_set_for_a_crash_in 的指纹变了、RefFS 那一列的格变了，G6 中。"""
        with tempfile.TemporaryDirectory() as directory:
            inputs = make_inputs(directory, reference_filesystem_reference=REFERENCE_FILESYSTEM_TODAY.replace("= 2;", "= 5;"))
            result = attribution.Attributor(inputs).attribute(
                cell(name="O1-RefFS_red", key="row:15,stream:new_pool_file_creation,batch:1", kind="q1_row", product="P3", cell_class="decision_positive",
                     r1="0", today="2"))
        self.assertEqual((result.status, result.outcomes["g6"]), (attribution.STATUS_ATTRIBUTED, "hit"))

    def test_a_missing_research_reference_makes_the_cell_uncheckable(self):
        """F6：研究一侧 r1 那一版找不到，G7 查不了；别的都不中时那一格记查不了，Q1 判不够判。"""
        with tempfile.TemporaryDirectory() as directory:
            result = attribution.Attributor(make_inputs(directory, research_reference=None)).attribute(cell())
        self.assertEqual((result.status, result.outcomes["g7"]), (attribution.STATUS_UNCHECKABLE, "uncheckable"))
        self.assertIn("research_reference_missing:collusion_matrix.py", result.notes)
        self.assertEqual(attribution.q1_verdict(1, 0, 1), "not_enough_f6")


class WideAttribution(unittest.TestCase):
    """读法 8 末条宽归因（描述）：坏实现所在文件里指纹变了的 fn 单元数，装置 main 与那一列 oracle 入口的闭包里指纹变了或新出的单元数。
    闭包（lib_function_units.rust_closure 要整个工作区）换成假的：参照树与今天的树各给一份 {(文件, 单元): 指纹}。"""

    def test_the_wide_attribution_counts_changed_and_new_units_of_the_file_and_of_each_closure(self):
        def fake_closure(root, entry):
            today = root.endswith("today")
            if entry.endswith("::main"):
                return {("d", "fn main"): "1", ("d", "fn helper"): "3" if today else "2", **({("d", "fn added"): "4"} if today else {})}
            return {("o", f"fn {entry}"): "5", ("o", "fn shared"): "7" if today else "6"}

        with tempfile.TemporaryDirectory() as directory, mock.patch.object(attribution, "closure_members", fake_closure):
            inputs = make_inputs(directory, core_reference=CORE_TODAY.replace("let other = 2;", "let other = 9;"))
            counts = attribution.wide_attribution(attribution.Attributor(inputs), cell(name="O1-RefFS", kind="untouched_row_red", cell_class="decision_count"))
        self.assertEqual(counts["mutated_file_changed_units"], "1", "落点文件里 beta 变了、alpha 没变")
        self.assertEqual(counts["device_main_closure_changed_units"], "2", "helper 指纹变了、added 是新出的")
        self.assertEqual(counts["oracle_closure_changed_units"], "1,1,1", "RefFS 列今天那份文件里三个入口，各自闭包里 shared 变了")

    def test_a_closure_that_cannot_be_computed_is_written_as_a_dash(self):
        with tempfile.TemporaryDirectory() as directory, mock.patch.object(attribution, "closure_members", lambda _root, _entry: None):
            counts = attribution.wide_attribution(attribution.Attributor(make_inputs(directory)), cell())
        self.assertEqual(counts, {"mutated_file_changed_units": "0", "device_main_closure_changed_units": "-", "oracle_closure_changed_units": "-"})


class Pieces(unittest.TestCase):
    def test_a_row_set_cell_attributes_the_symmetric_difference(self):
        self.assertEqual(attribution.rows_of_cell(cell(name="row_numbers", key="reading:registration,batch:1,class:counted", kind="q3_denominator_class",
                                                       cell_class="decision_set", r1="15,23,236", today="15,23,70")), (70, 236))

    def test_columns_of_a_strict_q1_cell_include_the_strict_o3(self):
        self.assertEqual(attribution.columns_of_cell(cell(name="all_quiet_rows", key="reading:strict,streams:two_streams,batch:1", kind="q1",
                                                          product="P3", cell_class="decision_set")), attribution.STRICT_COLUMNS)

    def test_p0_status_lines_give_their_paths(self):
        text = "E7RESULT name=frozen_tree key=git_status line=_M_Cargo.lock\nE7RESULT name=frozen_tree key=git_status line=MM_crates/mutations.tsv\n"
        self.assertEqual(attribution.paths_of_status_lines(text), frozenset({"Cargo.lock", "crates/mutations.tsv"}))

    def test_the_verdict_follows_the_three_counts(self):
        self.assertEqual([attribution.q1_verdict(*counts) for counts in ((0, 0, 0), (2, 0, 0), (2, 1, 1), (2, 0, 1))],
                         ["no_cell_changed", "every_cell_attributed", "some_cells_unattributable", "not_enough_f6"])

    def test_the_two_function_maps_are_the_registered_ones(self):
        for path, digest in ((attribution.DEVICE_MAP_PATH, attribution.DEVICE_MAP_SHA256), (attribution.RESEARCH_MAP_PATH, attribution.RESEARCH_MAP_SHA256)):
            attribution.load_function_map(path, digest)
            with self.assertRaises(ValueError):
                attribution.load_function_map(path, "0" * 64)


if __name__ == "__main__":
    unittest.main()
