"""row_driver.py 的分片、合并、只跑空变异，与装置环境（CARGO）、编装置带的特性：不起 cargo、不起装置，编译与装置两步换成桩，
快照同步用真的 rsync（临时目录里造一棵小仓）。long_run_item_verdicts 的算键换成请求原文的 sha256、判定库换成内存里的假库（fake_item_verdicts）；
真算键、连临时 prune-server 的复用三问与哨兵在 row_driver.py 自己的自证里。

跑法：在这个目录里 `python3 -B -m unittest test_row_shards`。
弄坏开关（row_driver.py 文件头逐项写了打开后红的用例）：
  E171_ROW_DRIVER_BREAK=device-without-cargo-variable、compile-without-gpu-feature、row-range-ignored、empty-mutation-only-runs-the-table、
  device-output-defect-ignored、engine-start-refused-ignored、configuration-check-not-synced、progress-not-reported、reused-row-not-unpacked、
  unstorable-outcome-recorded-green、refused-row-requested、take-only-judges-missing-rows。
只取那一趟（shared_steps.py take-rows）经 Run.take 在进程里调 shared_steps.main，桩与假库同 Run.main。
"""
import argparse
import contextlib
import hashlib
import io
import os
import shutil
import stat
import tempfile
import time
import unittest
from types import SimpleNamespace
from unittest import mock

import bad_implementation_table as table
import row_driver
import shared_steps

FILE_A = "crates/singlefs-core/src/a.rs"
FILE_B = "crates/singlefs-core/src/b.rs"
FILE_C = "crates/singlefs-core/src/c.rs"
ROWS = (  # (序号, 文件, 变异名, 锚点)
    (1, FILE_A, "a1", "anchor_a1"), (2, FILE_A, "a2", "anchor_a2"), (3, FILE_A, "a3", "anchor_a3"),
    (4, FILE_B, "b1", "anchor_b1"), (5, FILE_B, "b2", "anchor_b2"),
    (7, FILE_C, "c1", "anchor_c1"), (8, FILE_C, "c2", "anchor_c2"), (9, FILE_C, "c3", "anchor_c3"), (10, FILE_C, "c4", "anchor_c4"),
)


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def selected(number: int, file: str, name: str) -> table.SelectedRow:
    return table.SelectedRow(number, 2, file, 0, name, "0" * 12)


def selected_rows() -> list[table.SelectedRow]:
    return [selected(number, file, name) for number, file, name, _ in ROWS]


def tiny_repository(root: str, source_suffix: str = "") -> str:
    """三个源文件、一张变异表、一张选行表的小仓；交回选行表的路径。"""
    write(os.path.join(root, "Cargo.toml"), "[workspace]\n")
    write(os.path.join(root, "Cargo.lock"), "# lock\n")
    for entry in row_driver.CONFIGURATION_CHECK_ENTRIES:  # main 在快照里缺多机配置判法时拒跑；小仓带上样本
        write(os.path.join(root, entry), "# sample\n")
    for file in (FILE_A, FILE_B, FILE_C):
        letter = os.path.basename(file)[0]
        write(os.path.join(root, file), "".join(f"anchor_{letter}{index}\n" for index in range(1, 5)) + source_suffix)
    write(os.path.join(root, "crates/mutations.tsv"),
          "".join(f"{name}\t{file}\t{anchor}\treplaced\t-p core\ttest_{name}\n" for _, file, name, anchor in ROWS))
    selection = os.path.join(root, "selected.tsv")
    write(selection, "".join(f"{number}\t2\t{file}\t0\t{name}\t{'0' * 12}\n" for number, file, name, _ in ROWS))
    return selection


class FakeLibrary:
    """内存里的假判定库（只给单测：long_run_item_verdicts 的 library_of 换成它），与 prune_server_client 的库 read_many / write 同形。"""

    def __init__(self) -> None:
        self.entries: dict[str, bytes] = {}

    def read_many(self, relatives: list[str]) -> list[bytes | None]:
        return [self.entries.get(relative) for relative in relatives]

    def write(self, relative: str, data: bytes | str) -> None:
        self.entries[relative] = data.encode("utf-8") if isinstance(data, str) else data


def fake_unit_key(_context, lines: list[str]) -> SimpleNamespace:
    """算键换成请求行原文的 sha256（单测不收闭包）。"""
    return SimpleNamespace(key=hashlib.sha256("\n".join(lines).encode("utf-8")).hexdigest(), refusals=[])


@contextlib.contextmanager
def fake_item_verdicts(library: FakeLibrary):
    """long_run_item_verdicts 的库与算键换成假的；键相同的全当复用哨兵、不抽强制现判。"""
    verdicts = row_driver.item_verdicts_module()
    with mock.patch.object(verdicts, "library_of", return_value=library), mock.patch.object(verdicts, "keys_of_lines", side_effect=fake_unit_key), \
            mock.patch.dict(os.environ, {verdicts.lib_reuse_checks.SENTINEL_COUNT_VARIABLE: verdicts.ALL_SENTINELS}):
        yield verdicts


class Run:
    """一次 `row_driver.main`：编译与装置两步换成桩（编译记下变异名，装置只建输出文件），判定库是 `library`（不给是一份新的空假库），
    交回 (退出码, 标准输出, 标准错误, 编译过的变异名)。"""

    def __init__(self, failing_names: tuple[str, ...] = (), library: FakeLibrary | None = None) -> None:
        self.compiled: list[str] = []
        self.failing_names = failing_names
        self.library = library if library is not None else FakeLibrary()
        self.devices: list[str] = []

    def compile_stub(self, mutation, lane_tree, lane_target, snapshot_root, lane_directory, row_directory, arguments):
        self.compiled.append(mutation.name)
        if mutation.name in self.failing_names:
            write(os.path.join(row_directory, "compile.log"), "error\n")
            return 101, 3, "-"
        return 0, 3, "crates/singlefs-core"

    @staticmethod
    def device_stub(repo_root, binary, row_directory, threads, extra):
        for name in ("device.out", "device.err"):
            write(os.path.join(row_directory, name), "")
        write(os.path.join(row_directory, "dump", "stream", "oracle-verdicts.tsv"), "state\tverdict\n")
        return 0, 7

    def main(self, arguments: list[str]) -> tuple[int, str, str, list[str]]:
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(row_driver, "BREAK_ACCEPTED_IN_PROCESS", True), \
                mock.patch.object(row_driver, "compile_one_mutation", self.compile_stub), \
                mock.patch.object(row_driver, "run_device", self.device_stub), fake_item_verdicts(self.library), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = row_driver.main(arguments)
        return code, out.getvalue(), err.getvalue(), list(self.compiled)

    def recording_device(self, repo_root, binary, row_directory, threads, extra):
        self.devices.append(os.path.basename(row_directory))
        return self.device_stub(repo_root, binary, row_directory, threads, extra)

    def take(self, arguments: list[str]) -> tuple[int, str, str, list[str]]:
        """只取（`shared_steps.py take-rows`，row_driver.py 文件头「只取」）：编译与装置照样换成桩（装置另记下起过哪一行，self.devices），判定库是 `library`；
        交回 (退出码, 标准输出, 标准错误, 编译过的变异名)。"""
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(row_driver, "BREAK_ACCEPTED_IN_PROCESS", True), \
                mock.patch.object(row_driver, "compile_one_mutation", self.compile_stub), \
                mock.patch.object(row_driver, "run_device", self.recording_device), fake_item_verdicts(self.library), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = shared_steps.main(["take-rows", *arguments])
        return code, out.getvalue(), err.getvalue(), list(self.compiled)


def driver_arguments(root: str, selection: str, work: str, output: str, *more: str) -> list[str]:
    return ["--repo-root", root, "--work-root", work, "--output-root", output, "--selected-rows", selection, "--no-executor",
            "--lanes", "2", "--threads", "1", *more]


def result_fields(row_directory: str) -> dict[str, object]:
    """行目录的 row-result.tsv，去掉每趟都变的：线道和各段耗时。"""
    fields = row_driver.read_result(row_directory)
    return {key: value for key, value in fields.items() if key != "lane" and not key.endswith("_ns")}


def row_lines(stdout: str) -> list[dict[str, str]]:
    """驱动打的 `row …` 行，按序号排，去掉线道和耗时。"""
    lines = []
    for line in stdout.splitlines():
        if line.startswith("row "):
            fields = dict(token.split("=", 1) for token in line.removeprefix("row ").split(" ") if "=" in token)
            lines.append({key: value for key, value in fields.items() if key != "lane" and not key.endswith("_ns")})
    return sorted(lines, key=lambda fields: int(fields["row_number"]))


class ShardRangeTests(unittest.TestCase):
    def test_the_ranges_close_between_file_groups_and_cover_every_row_once(self) -> None:
        rows = selected_rows()
        self.assertEqual(row_driver.shard_ranges(rows, 1), [(1, 10)])
        self.assertEqual(row_driver.shard_ranges(rows, 2), [(1, 5), (7, 10)])
        self.assertEqual(row_driver.shard_ranges(rows, 3), [(1, 3), (4, 5), (7, 10)])
        self.assertEqual(row_driver.shard_ranges(rows, 9), [(1, 3), (4, 5), (7, 10)])
        for count in range(1, 6):
            ranges = row_driver.shard_ranges(rows, count)
            covered = [row.number for low, high in ranges for row in rows if low <= row.number <= high]
            self.assertEqual(covered, [row.number for row in rows])
            self.assertLessEqual(len(ranges), count)
            for low, high in ranges:
                self.assertEqual(row_driver.files_cut_by_the_range(rows, low, high), [])

    def test_the_even_split_ignores_file_groups_and_differs_by_at_most_one_row(self) -> None:
        rows = selected_rows()
        self.assertEqual(row_driver.shard_ranges(rows, 4, may_cut_groups=True), [(1, 2), (3, 4), (5, 7), (8, 10)])
        self.assertEqual(row_driver.shard_ranges(rows, 1, may_cut_groups=True), [(1, 10)])
        self.assertEqual(len(row_driver.shard_ranges(rows, 20, may_cut_groups=True)), 9)
        for count in range(1, 12):
            sizes = [sum(low <= row.number <= high for row in rows) for low, high in row_driver.shard_ranges(rows, count, may_cut_groups=True)]
            self.assertEqual(sum(sizes), 9)
            self.assertLessEqual(max(sizes) - min(sizes), 1)

    def test_a_range_that_cuts_a_file_group_names_the_file(self) -> None:
        rows = selected_rows()
        self.assertEqual(row_driver.files_cut_by_the_range(rows, 1, 2), [FILE_A])
        self.assertEqual(row_driver.files_cut_by_the_range(rows, 3, 7), [FILE_A, FILE_C])
        self.assertEqual(row_driver.files_cut_by_the_range(rows, 4, 5), [])

    def test_a_table_with_a_scattered_file_or_numbers_not_increasing_is_refused(self) -> None:
        scattered = [selected(1, FILE_A, "a1"), selected(2, FILE_B, "b1"), selected(3, FILE_A, "a2")]
        with self.assertRaises(ValueError):
            row_driver.shard_ranges(scattered, 2)
        backwards = [selected(2, FILE_A, "a1"), selected(1, FILE_B, "b1")]
        with self.assertRaises(ValueError):
            row_driver.shard_ranges(backwards, 2)

    def test_a_row_range_takes_only_the_rows_inside_it(self) -> None:
        rows = selected_rows()
        arguments = argparse.Namespace(empty_mutation_only=False, first_slice=False, rows=None, row_range="4-7")
        self.assertEqual([row.number for row in row_driver.choose_rows(arguments, rows)], [4, 5, 7])
        self.assertEqual(row_driver.parse_row_range("3-9"), (3, 9))
        for bad in ("9-3", "3", "a-b", "3-"):
            with self.assertRaises(ValueError):
                row_driver.parse_row_range(bad)


class DriverRunTests(unittest.TestCase):
    def setUp(self) -> None:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.base = holder.name
        self.repository = os.path.join(self.base, "repository")
        self.selection = tiny_repository(self.repository)

    def path(self, name: str) -> str:
        return os.path.join(self.base, name)

    def run_driver(self, run: Run, work: str, output: str, *more: str, selection: str | None = None, repository: str | None = None):
        return run.main(driver_arguments(repository or self.repository, selection or self.selection, self.path(work), self.path(output), *more))

    def test_the_shard_ranges_are_printed_without_running_anything(self) -> None:
        run = Run()
        code, stdout, _, compiled = run.main(["--repo-root", self.repository, "--work-root", self.path("unused-w"), "--output-root", self.path("unused-o"),
                                               "--selected-rows", self.selection, "--print-shard-ranges", "2"])
        self.assertEqual((code, compiled), (0, []))
        self.assertFalse(os.path.exists(self.path("unused-w")))
        self.assertEqual(stdout.splitlines(), ["shard_range index=1 low=1 high=5 rows=5", "shard_range index=2 low=7 high=10 rows=4"])

    def test_the_empty_mutation_only_runs_row_zero(self) -> None:
        run = Run()
        code, _, stderr, compiled = self.run_driver(run, "w", "o", "--empty-mutation-only")
        self.assertEqual(code, 0)
        self.assertEqual(sorted(os.listdir(self.path("o"))), ["row-0"])
        self.assertEqual(len(compiled), 1)
        self.assertTrue(compiled[0].startswith("空变异"))
        self.assertIn("scope=narrowed rows=0 of=9", stderr)

    def test_a_shard_that_cuts_a_file_group_is_refused_unless_nothing_is_substituted(self) -> None:
        run = Run()
        code, _, stderr, compiled = self.run_driver(run, "w", "o", "--row-range", "1-2")
        self.assertEqual((code, compiled), (2, []))
        self.assertIn(FILE_A, stderr)
        code, _, stderr, compiled = self.run_driver(run, "w2", "o2", "--row-range", "1-2", "--keep-compile-failures-as-they-are")
        self.assertEqual((code, sorted(compiled)), (0, ["a1", "a2"]))
        self.assertIn("scope=shard rows=2 of=9 range=1-2", stderr)

    def test_sharded_runs_merge_into_the_same_rows_and_lines_as_the_unsharded_run(self) -> None:
        """各片判的行写进同一份库；合并那一趟（整表、另一个输出根）一行都不编，行目录与 `row …` 行与不分片的那一趟相同。"""
        whole = Run()
        code, whole_out, _, whole_compiled = self.run_driver(whole, "w-whole", "o-whole")
        self.assertEqual((code, sorted(whole_compiled)), (0, sorted(name for _, _, name, _ in ROWS)))
        library = FakeLibrary()
        shard_compiled = []
        for index, (low, high) in enumerate(row_driver.shard_ranges(selected_rows(), 3)):
            shard = Run(library=library)
            code, _, _, compiled = self.run_driver(shard, f"w-shard-{index}", "o-shards", "--row-range", f"{low}-{high}")
            self.assertEqual(code, 0)
            shard_compiled += compiled
        self.assertEqual(sorted(shard_compiled), sorted(whole_compiled))
        merge = Run(library=library)
        code, merge_out, _, merge_compiled = self.run_driver(merge, "w-merge", "o-merged")
        self.assertEqual((code, merge_compiled), (0, []))
        self.assertEqual(sorted(os.listdir(self.path("o-merged"))), sorted(os.listdir(self.path("o-whole"))))
        for row_name in os.listdir(self.path("o-whole")):
            self.assertEqual(result_fields(self.path(f"o-merged/{row_name}")), result_fields(self.path(f"o-whole/{row_name}")), row_name)
            self.assertTrue(os.path.isfile(self.path(f"o-merged/{row_name}/dump/stream/oracle-verdicts.tsv")))
        self.assertEqual(row_lines(merge_out), row_lines(whole_out))

    def test_take_only_names_a_row_the_library_lacks_and_runs_nothing(self) -> None:
        """只取（row_driver.py 文件头「只取」）：判过 1、2、4 而第 4 行（b1）编不过、照同一把键记红；只取 1、2、4、5 退 4，逐行点名 row-4（库里记着红）与 row-5
        （库里没有这把键），row-1、row-2 解进输出根、row-4、row-5 不建、标准输出一个字都不打，一行都不编、装置一次都不起；点 1、2 退 0，`row …` 行与判它们的那一趟
        相同、另有 `table lanes=0 …`；只查（--check-only）照样点名、输出根不建。"""
        library = FakeLibrary()
        code, judged_out, stderr, _ = self.run_driver(Run(failing_names=("b1",), library=library), "w-judge", "o-judge", "--rows", "1,2,4",
                                                      "--keep-compile-failures-as-they-are")
        self.assertEqual(code, 0, stderr)
        take = Run(library=library)
        common = ["--repo-root", self.repository, "--selected-rows", self.selection, "--no-executor", "--work-root", self.path("w-take")]
        code, stdout, stderr, compiled = take.take([*common, "--output-root", self.path("o-take"), "--rows", "1,2,4,5"])
        self.assertEqual((code, compiled, take.devices, stdout), (shared_steps.EXIT_MISSING, [], [], ""), stderr)
        self.assertIn("E171_ROW_NOT_IN_THE_LIBRARY row=4 reason=库里这把键记着红", stderr)
        self.assertIn("E171_ROW_NOT_IN_THE_LIBRARY row=5 reason=库里没有这把键", stderr)
        self.assertIn("E171_ROWS_TAKEN taken=2 missing=2 of=4 check_only=no", stderr)
        self.assertEqual(sorted(os.listdir(self.path("o-take"))), ["row-1", "row-2"])
        code, stdout, stderr, compiled = take.take([*common, "--output-root", self.path("o-all"), "--rows", "1,2"])
        self.assertEqual((code, compiled, take.devices), (0, [], []), stderr)
        self.assertEqual(row_lines(stdout), [fields for fields in row_lines(judged_out) if fields["row_number"] in ("1", "2")])
        self.assertTrue(any(line.startswith("table lanes=0 ") for line in stdout.splitlines()), stdout)
        code, stdout, stderr, compiled = take.take([*common, "--output-root", self.path("o-checked"), "--rows", "1,2,4,5", "--check-only"])
        self.assertEqual((code, compiled, take.devices, stdout), (shared_steps.EXIT_MISSING, [], [], ""), stderr)
        self.assertIn("E171_ROW_NOT_IN_THE_LIBRARY row=5 ", stderr)
        self.assertIn("E171_ROWS_TAKEN taken=2 missing=2 of=4 check_only=yes", stderr)
        self.assertFalse(os.path.exists(self.path("o-checked")))

    def test_a_snapshot_without_the_configuration_check_is_refused_before_any_row_runs(self) -> None:
        """第 3 次重跑探路片：线道的树里没有多机配置判法，装置起不了引擎、每行都是废读数；缺了就在开跑前拒（退 2、点名缺的文件），一行都不编。"""
        for entry in row_driver.CONFIGURATION_CHECK_ENTRIES:
            os.remove(os.path.join(self.repository, entry))
        run = Run()
        code, stdout, _, compiled = self.run_driver(run, "w-missing", "o-missing")
        self.assertEqual((code, compiled), (row_driver.EXIT_WORK_ROOT_REFUSED, []))
        self.assertIn("configuration_check_missing entries=" + ",".join(row_driver.CONFIGURATION_CHECK_ENTRIES), stdout)
        self.assertFalse(os.path.exists(self.path("o-missing/row-1")))

    def test_a_row_that_did_not_compile_is_judged_again_on_the_next_run(self) -> None:
        """编不过的行照同一把键记红：同一份库上再起一趟（同一个输出根），只有它重编，别的行从库里取回。"""
        library = FakeLibrary()
        shard = Run(failing_names=("b1",), library=library)
        code, _, _, _ = self.run_driver(shard, "w-shards", "o-shards", "--keep-compile-failures-as-they-are")
        self.assertEqual(code, 0)
        self.assertEqual(row_driver.read_result(self.path("o-shards/row-4"))["status"], "compile_failed")
        again = Run(library=library)
        code, _, _, compiled = self.run_driver(again, "w-again", "o-shards", "--keep-compile-failures-as-they-are")
        self.assertEqual((code, compiled), (0, ["b1"]))
        self.assertEqual(row_driver.read_result(self.path("o-shards/row-4"))["status"], "ok")

    def test_a_row_that_needs_a_substitute_is_kept_out_of_the_library(self) -> None:
        """开跑前锚点不唯一（要补位）的行不进请求、照跑：两趟都打 E171_ROW_ITEM 那一行，long_run_item_verdicts 的计划里没有它；别的行第二趟复用。"""
        with open(os.path.join(self.repository, FILE_B), "a", encoding="utf-8") as handle:
            handle.write("anchor_b1\n")
        library = FakeLibrary()
        for name in ("earlier", "later"):
            code, _, stderr, compiled = self.run_driver(Run(library=library), f"w-{name}", f"o-{name}", "--rows", "4,7")
            self.assertEqual(code, 0, stderr)
            self.assertIn("E171_ROW_ITEM row=4 refused=needs_substitution", stderr)
            self.assertNotIn("LONG_RUN_ITEM\trow-4\t", stderr)
            self.assertTrue(os.path.isfile(self.path(f"o-{name}/row-4/row-result.tsv")), name)
        self.assertIn("LONG_RUN_ITEM\trow-7\treuse\t", stderr)
        self.assertEqual(compiled, [])

    def test_a_row_longer_than_the_progress_interval_reports_progress_on_stderr_only(self) -> None:
        """一行跑得比进度间隔久：标准错误里有 row_progress（做完几行、在做哪一行），标准输出的 `row …` 行与不打进度的那一趟相同、没有 row_progress。"""
        class SlowRun(Run):
            def compile_stub(self, *arguments):
                time.sleep(0.4)
                return super().compile_stub(*arguments)

        _, quick_out, quick_err, _ = self.run_driver(Run(), "w-quick", "o-quick", "--rows", "1")
        with mock.patch.object(row_driver, "PROGRESS_INTERVAL_SECONDS", 0.05):
            code, slow_out, slow_err, _ = self.run_driver(SlowRun(), "w-slow", "o-slow", "--rows", "1")
        self.assertEqual(code, 0)
        progress = [line for line in slow_err.splitlines() if line.startswith("row_progress ")]
        self.assertTrue(progress, slow_err)
        self.assertTrue(any("done=0 of=1 running=row-1@lane" in line for line in progress), progress)
        self.assertNotIn("row_progress", slow_out)
        self.assertNotIn("row_progress", quick_err)
        self.assertEqual(row_lines(slow_out), row_lines(quick_out))


class DeviceEnvironmentTests(unittest.TestCase):
    def test_the_device_environment_names_the_cargo_that_records_the_toolchain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            cargo = os.path.join(directory, "cargo")
            write(cargo, "#!/bin/sh\n")
            os.chmod(cargo, os.stat(cargo).st_mode | stat.S_IXUSR)
            with mock.patch.dict(os.environ, {"PATH": directory}):
                os.environ.pop("CARGO", None)
                self.assertEqual(row_driver.device_environment(3).get("CARGO"), cargo)
            with mock.patch.dict(os.environ, {"PATH": directory, "CARGO": "/given/cargo"}):
                self.assertEqual(row_driver.device_environment(3).get("CARGO"), "/given/cargo")
                self.assertEqual(row_driver.device_environment(3).get("SINGLEFS_THREAD_CAP"), "3")

    def test_the_device_is_built_with_the_gpu_feature(self) -> None:
        command = row_driver.compile_command()
        position = command.index("--features")
        self.assertEqual(command[position + 1], "gpu")
        self.assertEqual(command[command.index("-p") + 1], "singlefs-verification")
        self.assertEqual(command[command.index("--bin") + 1], "e171_three_oracle_collusion")

    def test_a_device_output_that_says_not_run_or_lacks_the_toolchain_is_a_defect(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            write(os.path.join(directory, "device.out"), "E7RESULT name=done emitted=3\n")
            write(os.path.join(directory, "device.err"), "E171_SAMPLED_STATES stream=a runs=3 threads=2\n")
            self.assertEqual(row_driver.device_output_defects(directory), [])
            write(os.path.join(directory, "device.out"), "NOT_RUN given_states judge_not_run_without_the_gpu_feature\n")
            write(os.path.join(directory, "device.err"), f"panicked: 要经 cargo test 起（环境里要有 CARGO）：{row_driver.TOOLCHAIN_MISSING_SENTENCE}\n")
            self.assertEqual(row_driver.device_output_defects(directory), ["device.out:not_run", "device.err:toolchain_missing"])

    def test_a_device_that_could_not_start_its_engine_is_a_defect_but_a_bad_implementation_that_crashes_is_not(self) -> None:
        """e171_three_oracle_collusion.rs 的 given_states_engine_of_this_experiment 起引擎失败的 panic（第 3 次重跑探路片：判法退 127）是测量管子的缺陷；
        坏实现自己崩（streams_not_exported 不为 0、别的 panic 句）不算缺陷。"""
        with tempfile.TemporaryDirectory() as directory:
            write(os.path.join(directory, "device.out"), "E7RESULT name=verdict streams_not_exported=2\n")
            write(os.path.join(directory, "device.err"),
                  "thread 'e171-judging' panicked at crates/singlefs-verification/src/bin/e171_three_oracle_collusion.rs:1872:35:\n"
                  "起不了给定状态表的引擎：多机配置判法认不出这一台（bash x/crash-point-replay-shard-configuration-check.sh 退 127）\n")
            self.assertEqual(row_driver.device_output_defects(directory), ["device.err:engine_start_refused"])
            write(os.path.join(directory, "device.err"), "thread 'main' panicked at crates/singlefs-core/src/a.rs:1:1:\nindex out of bounds\n")
            self.assertEqual(row_driver.device_output_defects(directory), [])

    def test_an_engine_start_refusal_stops_the_row_as_a_measurement_pipe_defect_and_exits_5(self) -> None:
        """整条路：装置退出后输出里有起引擎失败的句子，这一行记 measurement_pipe_defect、驱动退 5（坏实现自崩的行 status ok 照旧）。"""
        def refusing_device(repo_root, binary, row_directory, threads, extra):
            write(os.path.join(row_directory, "device.out"), "E7RESULT name=verdict streams_not_exported=2\n")
            write(os.path.join(row_directory, "device.err"), "起不了给定状态表的引擎：多机配置判法认不出这一台\n")
            return 101, 7

        def crashing_device(repo_root, binary, row_directory, threads, extra):
            write(os.path.join(row_directory, "device.out"), "E7RESULT name=verdict streams_not_exported=2\n")
            write(os.path.join(row_directory, "device.err"), "thread 'main' panicked: index out of bounds\n")
            return 101, 7

        with tempfile.TemporaryDirectory() as base:
            repository = os.path.join(base, "repository")
            selection = tiny_repository(repository)
            for label, device, wanted_code in (("refusing", refusing_device, row_driver.EXIT_MEASUREMENT_PIPE_DEFECT), ("crashing", crashing_device, 0)):
                run = Run()
                out, err = io.StringIO(), io.StringIO()
                with mock.patch.object(row_driver, "BREAK_ACCEPTED_IN_PROCESS", True), \
                        mock.patch.object(row_driver, "compile_one_mutation", run.compile_stub), \
                        mock.patch.object(row_driver, "run_device", device), \
                        contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                    code = row_driver.main(driver_arguments(repository, selection, os.path.join(base, f"w-{label}"), os.path.join(base, f"o-{label}"),
                                                            "--rows", "1"))
                self.assertEqual(code, wanted_code, label)
                if label == "crashing":
                    self.assertEqual(row_driver.read_result(os.path.join(base, "o-crashing/row-1"))["status"], "ok")


if __name__ == "__main__":
    unittest.main()
