"""E171 第二段 O3 一侧的单测：冻结版本核对、执行器产物的读回、坏镜像的判法（拷来的那一半）、驱动的 O3 一步与线道队列。手造数据，不起执行器、不起装置。

跑法（在本目录）：`python3 -m unittest test_o3_columns`（与 `test_collusion_matrix`、`test_collusion_pipeline` 一起由变异驱动跑）。
每个比较旁边配钉绝对值的断言：冻结的种子写死成登记的 `85441ae491419411`，实际目录与冻结行逐个比。
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import queue
import sys
import tempfile
import threading
import time
import types
import unittest
import unittest.mock

import frozen_o3
import o3_bad_image_judge as judge
import row_driver
from bad_implementation_table import STATUS_LOCATED, MutationRow

HERE = os.path.dirname(os.path.abspath(__file__))
REPOSITORY_ROOT = os.environ.get("E171_REPOSITORY_ROOT") or os.path.abspath(os.path.join(HERE, "../../.."))  # 变异驱动在副本里跑单测时给真的仓根
REGISTERED_SEED = "85441ae491419411"
FROZEN_FILE_COUNT = 15

EXECUTOR_OUTPUT = """name=config dumps=d streams=a batches=1,2 workers=1
name=base_recovery stream=fixed_script_through_e_then_unmount recovered=false detail=x
name=load stream=new_pool_file_creation load_ns=1
name=state stream=new_pool_file_creation state=7 layer0=7 segment=0 batch=1 digest=match strict=in provisional=in time_ns=100 worker=11
name=state stream=new_pool_file_creation state=3 layer0=3 segment=0 batch=1 digest=match strict=spec_gap provisional=not_in time_ns=50 worker=12
name=state stream=new_pool_file_creation state=5 layer0=5 segment=0 batch=2 digest=mismatch strict=error provisional=error error=boom time_ns=0 worker=13
name=dump_problem stream=new_pool_file_creation detail=y
name=freeze file=a.py sha256=aa
name=freeze_seed sha256_prefix16=0123456789abcdef
name=complete
"""


def write(path: str, text: str | bytes) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(text, bytes) else "w", **({} if isinstance(text, bytes) else {"encoding": "utf-8"})) as handle:
        handle.write(text)


def read_tsv(path: str) -> list[list[str]]:
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").split("\t") for line in handle]


def make_frozen_repository(root: str, files: dict[str, bytes]) -> list[str]:
    """临时仓里造一个冻结目录与对应的冻结行；交回冻结行（文件行与种子行）。"""
    directory = os.path.join(root, frozen_o3.FROZEN_DIRECTORY_RELATIVE)
    lines = []
    hasher = hashlib.sha256()
    for name in sorted(files):
        write(os.path.join(directory, name), files[name])
        hasher.update(files[name])
        lines.append(f"name=freeze file={name} sha256={hashlib.sha256(files[name]).hexdigest()}")
    lines.append(f"name=freeze_seed sha256_prefix16={hasher.hexdigest()[:16]}")
    return lines


class ExecutorOutputTests(unittest.TestCase):
    def test_states_unrecoverable_streams_errors_digest_mismatches_and_the_freeze_lines_are_read(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "o3.out")
            write(path, EXECUTOR_OUTPUT)
            reading = frozen_o3.read_executor_output(path)
        self.assertEqual(reading.verdicts, {("new_pool_file_creation", 7): ("in", "in"), ("new_pool_file_creation", 3): ("spec_gap", "not_in"),
                                            ("new_pool_file_creation", 5): ("error", "error")})
        self.assertEqual(reading.base_unrecoverable_streams, ["fixed_script_through_e_then_unmount"])
        self.assertEqual((reading.error_states, reading.digest_mismatch_states, reading.dump_problems, reading.state_lines), (1, 1, 1, 3))
        self.assertEqual(reading.in_process_state_ns, 150)
        self.assertEqual(reading.freeze_lines, ["name=freeze file=a.py sha256=aa", "name=freeze_seed sha256_prefix16=0123456789abcdef"])
        self.assertTrue(reading.complete)

    def test_an_output_without_the_complete_marker_is_not_complete(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "o3.out")
            write(path, EXECUTOR_OUTPUT.replace("name=complete\n", ""))
            self.assertFalse(frozen_o3.read_executor_output(path).complete)

    def test_the_command_goes_through_the_wrapper_lists_only_the_given_streams_and_asks_for_batches_one_and_two(self) -> None:
        command = frozen_o3.executor_command("/repo", "/d", ["s1"], "1,2", 3, "/o", "4G")
        self.assertEqual(command[:4], ["bash", "/repo/scripts/hosts/run-with-memory-cap.sh", "4G", "bash"])
        self.assertEqual(command[4:7], ["/repo/scripts/hosts/capped.sh", "3", "nice"])
        self.assertEqual(command[-10:], ["--dumps", "/d", "--streams", "s1", "--batches", "1,2", "--workers", "3", "--output", "/o"])


class FreezeTests(unittest.TestCase):
    def test_the_seed_is_the_prefix_of_the_sha256_of_the_contents_in_name_order(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "b.py"), b"y")
            write(os.path.join(root, "a.py"), b"x")
            write(os.path.join(root, "comparator_x.py"), b"zzz")
            write(os.path.join(root, "mutation_driver.py"), b"zzz")
            write(os.path.join(root, "c.pyc"), b"zzz")
            os.makedirs(os.path.join(root, "__pycache__"))
            self.assertEqual(frozen_o3.frozen_files_in(root), ["a.py", "b.py"])
            self.assertEqual(frozen_o3.seed_from_directory(root), hashlib.sha256(b"xy").hexdigest()[:16])

    def test_a_matching_directory_has_no_difference_and_each_kind_of_change_is_named(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            lines = make_frozen_repository(root, {"a.py": b"x", "b.py": b"y"})
            self.assertEqual(frozen_o3.freeze_differences(root, lines), [])
            directory = os.path.join(root, frozen_o3.FROZEN_DIRECTORY_RELATIVE)
            write(os.path.join(directory, "a.py"), b"changed")
            self.assertEqual(frozen_o3.freeze_differences(root, lines), ["a.py:sha256_differs", "freeze_seed:differs"])
            write(os.path.join(directory, "a.py"), b"x")
            write(os.path.join(directory, "extra.py"), b"new")
            self.assertEqual(frozen_o3.freeze_differences(root, lines), ["extra.py:not_in_the_freeze", "freeze_seed:differs"])
            os.remove(os.path.join(directory, "extra.py"))
            os.remove(os.path.join(directory, "b.py"))
            self.assertEqual(frozen_o3.freeze_differences(root, lines), ["b.py:missing", "freeze_seed:differs"])

    def test_a_change_registered_as_path_only_is_not_a_difference_and_a_later_change_still_is(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            lines = make_frozen_repository(root, {"a.py": b"x", "b.py": b"y"})
            directory = os.path.join(root, frozen_o3.FROZEN_DIRECTORY_RELATIVE)
            frozen_digest, frozen_seed = frozen_o3.frozen_hashes_of(lines)["a.py"], frozen_o3.seed_of(lines)
            write(os.path.join(directory, "a.py"), b"x with a moved path")
            registered = {"a.py": (frozen_digest, frozen_o3.sha256_of_file(os.path.join(directory, "a.py"))),
                          frozen_o3.SEED_NAME: (frozen_seed, frozen_o3.seed_from_directory(directory))}
            self.assertEqual(frozen_o3.freeze_differences(root, lines, path_only_changes={}), ["a.py:sha256_differs", "freeze_seed:differs"],
                             "没登记时只差路径的改动照判不同")
            self.assertEqual(frozen_o3.freeze_differences(root, lines, path_only_changes=registered), [],
                             "登记了、冻结时与现在的两个值都对上的不算不同")
            write(os.path.join(directory, "a.py"), b"x changed again after the registration")
            self.assertEqual(frozen_o3.freeze_differences(root, lines, path_only_changes=registered), ["a.py:sha256_differs", "freeze_seed:differs"],
                             "登记之后又改了的照判不同")

    def test_a_path_only_change_row_with_the_wrong_number_of_columns_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            table = os.path.join(root, "freeze-path-only-changes.tsv")
            write(table, b"# header\nexecutor_main.py\tfrozen\tnow\tcommit\n")
            with self.assertRaises(ValueError):
                frozen_o3.path_only_changes_of(table)

    def test_a_missing_freeze_reference_is_reported_not_run_and_never_read_as_an_empty_freeze(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            make_frozen_repository(root, {"a.py": b"x"})
            with self.assertRaises(frozen_o3.FreezeReferenceMissing) as raised:
                frozen_o3.freeze_differences(root)
            line = raised.exception.not_run_line("test_o3_columns")
            self.assertTrue(line.startswith(f"{frozen_o3.NOT_RUN_MARKER} test=test_o3_columns reason={frozen_o3.FREEZE_REFERENCE_RELATIVE} 不在"), line)
            with unittest.mock.patch("sys.stderr", new_callable=io.StringIO) as stderr:
                self.assertTrue(frozen_o3.report_missing_freeze_reference(root, "comparator.py"))
            self.assertIn(f"{frozen_o3.NOT_RUN_MARKER} test=comparator.py reason=", stderr.getvalue())
            self.assertEqual(frozen_o3.NOT_RUN_EXIT_CODE, 77)

    def test_the_real_frozen_directory_matches_the_first_rerun_freeze_lines_and_the_registered_seed(self) -> None:
        try:
            reference = frozen_o3.freeze_reference_lines(REPOSITORY_ROOT)
        except frozen_o3.FreezeReferenceMissing as missing:   # 参照产物已归档：报本次未跑（直写标准输出，unittest 不截）后跳过
            sys.__stdout__.write(missing.not_run_line("test_o3_columns." + self.id().rsplit(".", 1)[-1]) + "\n")
            self.skipTest(missing.not_run_line(self.id()))
        self.assertEqual(frozen_o3.seed_of(reference), REGISTERED_SEED)
        self.assertEqual(len(frozen_o3.frozen_hashes_of(reference)), FROZEN_FILE_COUNT)
        self.assertEqual(frozen_o3.freeze_differences(REPOSITORY_ROOT, reference), [])
        # 冻结之后只改了路径的那几处登记在 freeze-path-only-changes.tsv：目录现算的种子是登记的「现在」那一个，登记的「冻结时」那一个就是登记的种子
        frozen_seed_registered, seed_now_registered = frozen_o3.path_only_changes_of().get(frozen_o3.SEED_NAME, (REGISTERED_SEED, REGISTERED_SEED))
        self.assertEqual(frozen_seed_registered, REGISTERED_SEED)
        self.assertEqual(frozen_o3.seed_from_directory(os.path.join(REPOSITORY_ROOT, frozen_o3.FROZEN_DIRECTORY_RELATIVE)), seed_now_registered)


class ImageJudgeTests(unittest.TestCase):
    def test_items_carry_the_row_the_landing_vector_both_digests_and_the_patches_cut_from_the_blob(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "bad-patches.bin"), b"AAAABBBBBB")
            write(os.path.join(root, "bad-states.tsv"), "3\t222\tBASE\tPATCHED\t0:512:0:4,1:512:4:6\n")
            items = judge.read_image_items(root, "bad-states.tsv", "bad-patches.bin",
                                           read_tsv)
        self.assertEqual(items, [judge.ImageItem(3, "222", "base", "patched", ((0, 512, b"AAAA"), (1, 512, b"BBBBBB")))])

    def test_a_patch_that_runs_past_the_end_of_the_blob_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "p.bin"), b"AA")
            write(os.path.join(root, "s.tsv"), "0\t2\tb\tp\t0:0:0:3\n")
            with self.assertRaises(AssertionError):
                judge.read_image_items(root, "s.tsv", "p.bin", read_tsv)

    def test_the_base_state_of_a_label_is_taken_from_the_row_with_the_smallest_number(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "bad-states.tsv"), "0\t222\tb0\tp0\t0:0:0:1\n6\t200\tb6\tp6\t0:0:0:1\n7\t201\tb7\tp7\t0:0:0:1\n")
            write(os.path.join(root, "bad-expected.tsv"), "0\tP1-F\tP\tF\t-\tnot_in\tnone\tx\n6\tP1-Mk\tP\tMk\t-\tnot_in\tnone\tx\n7\tP2-Mk\tP\tMk\t-\tnot_in\tnone\tx\n")
            labels = judge.base_labels(root, read_tsv)
        self.assertEqual(labels, {"F": (0, "222"), "Mk": (6, "200")})

    def test_the_patch_is_laid_over_the_image_the_executor_builds_and_the_names_are_put_back_afterwards(self) -> None:
        class Image:
            def __init__(self) -> None:
                self.bytes = {}

            def digest(self) -> str:
                return "base"

            def write_bytes(self, device: int, offset: int, payload: bytes) -> None:
                self.bytes[(device, offset)] = payload

        built: list[Image] = []

        def builder(dump, prepared_writes, landing_vector):
            image = Image()
            built.append(image)
            return image

        def recover(*arguments):
            return types.SimpleNamespace(landed="landed")

        def judge_state(context, state, cache):
            image = executor_core.build_crash_image(context.dump, [], state.landing_vector)
            executor_core.recover()
            return types.SimpleNamespace(digest_matches=True, strict_verdict="not_in", provisional_verdict="not_in", gaps_hit=[])

        executor_core = types.SimpleNamespace(build_crash_image=builder, recover=recover, judge_state=judge_state)
        crash_dump = types.SimpleNamespace(SampledState=lambda *arguments: types.SimpleNamespace(landing_vector=arguments[4]))
        replay_prefix = types.SimpleNamespace(UnitChecksumCache=lambda: None)
        context = types.SimpleNamespace(dump=types.SimpleNamespace(stream_name="s"))
        items = [judge.ImageItem(0, "22", "base", "patched", ((0, 512, b"X"),)), judge.ImageItem(1, "22", "other", "patched", ())]
        results = judge.judge_images((executor_core, crash_dump, replay_prefix), context, items)
        self.assertEqual(built[0].bytes, {(0, 512): b"X"})
        self.assertEqual(built[1].bytes, {})
        self.assertEqual([(result.row_id, result.base_digest_matches, result.landed_projection) for result in results],
                         [(0, True, "landed"), (1, False, "landed")])
        self.assertIs(executor_core.build_crash_image, builder)
        self.assertIs(executor_core.recover, recover)

    def test_a_judgement_that_raises_is_recorded_as_a_failure_and_the_names_are_still_put_back(self) -> None:
        def builder(dump, prepared_writes, landing_vector):
            raise RuntimeError("boom")

        def judge_state(context, state, cache):
            executor_core.build_crash_image(None, [], "2")

        executor_core = types.SimpleNamespace(build_crash_image=builder, recover=lambda *arguments: None, judge_state=judge_state)
        crash_dump = types.SimpleNamespace(SampledState=lambda *arguments: None)
        replay_prefix = types.SimpleNamespace(UnitChecksumCache=lambda: None)
        context = types.SimpleNamespace(dump=types.SimpleNamespace(stream_name="s"))
        results = judge.judge_images((executor_core, crash_dump, replay_prefix), context, [judge.ImageItem(0, "2", "b", "p", ())])
        self.assertIsNone(results[0].judgement)
        self.assertEqual(results[0].failure, "RuntimeError:boom")
        self.assertIs(executor_core.build_crash_image, builder)


class DriverO3StepTests(unittest.TestCase):
    def arguments(self, **overrides) -> argparse.Namespace:
        values = {"no_executor": False, "positive_control": None, "repo_root": "/repo", "threads": 2, "memory_cap": "1G"}
        values.update(overrides)
        return argparse.Namespace(**values)

    def test_only_streams_with_an_oracle_verdict_table_count_as_exported(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, row_driver.STREAM_NAMES[1], "oracle-verdicts.tsv"), "")
            write(os.path.join(root, row_driver.STREAM_NAMES[0], "states.tsv"), "")
            self.assertEqual(row_driver.exported_streams(root), [row_driver.STREAM_NAMES[1]])

    def test_the_step_is_skipped_on_request_and_has_nothing_to_judge_when_no_stream_was_exported(self) -> None:
        self.assertEqual(row_driver.run_o3_step(self.arguments(no_executor=True), "/nowhere")["o3_status"], row_driver.O3_STATUS_SKIPPED_BY_REQUEST)
        with tempfile.TemporaryDirectory() as root:
            fields = row_driver.run_o3_step(self.arguments(), root)
        self.assertEqual((fields["o3_status"], fields["o3_ns"]), (row_driver.O3_STATUS_NO_EXPORTED_STREAM, "-"))

    def test_a_finished_row_directory_is_read_back_and_an_interrupted_one_is_cleared_when_skipping_finished_rows(self) -> None:
        class Stop(Exception):
            pass

        def stop_after_clearing(source_root, destination_root):
            raise Stop

        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "row-3", "row-result.tsv"), "status\tok\ndevice_exit\t0\n")
            write(os.path.join(root, "row-4", "partial.txt"), "left over from an interrupted run")
            arguments = argparse.Namespace(skip_finished=True, output_root=root, work_root=os.path.join(root, "work"))
            self.assertEqual(row_driver.run_row_on_the_lane(3, None, None, [], set(), set(), arguments, 0), {"status": "ok", "device_exit": "0"})
            original = row_driver.sync_tree
            row_driver.sync_tree = stop_after_clearing
            try:
                with self.assertRaises(Stop):
                    row_driver.run_row_on_the_lane(4, None, None, [], set(), set(), arguments, 0)
            finally:
                row_driver.sync_tree = original
            self.assertEqual(os.listdir(os.path.join(root, "row-4")), [], "中途停下的行目录要先清掉")
            with self.assertRaises(FileExistsError):
                row_driver.run_row_on_the_lane(4, None, None, [], set(), set(), argparse.Namespace(skip_finished=False, output_root=root, work_root=root), 0)

    def test_every_job_is_taken_by_exactly_one_lane_and_a_row_that_raises_is_recorded_without_stopping_the_others(self) -> None:
        taken: list[tuple[int, int]] = []
        guard = threading.Lock()

        def fake_run(number, selected, empty_of, table, rows_in_the_table, used_substitutes, arguments, lane):
            with guard:
                taken.append((number, lane))
            if number == 3:
                raise RuntimeError("row three broke")
            return {"row_number": number, "status": "ok"}

        jobs: "queue.Queue" = queue.Queue()
        for position, number in enumerate(range(1, 8)):
            jobs.put((position, number, None, None))
        results: dict[int, dict[str, object]] = {}
        original = row_driver.run_row_on_the_lane
        row_driver.run_row_on_the_lane = fake_run
        try:
            workers = [threading.Thread(target=row_driver.lane_worker, args=(lane, jobs, results, [], set(), set(), None)) for lane in range(3)]
            for worker in workers:
                worker.start()
            for worker in workers:
                worker.join()
        finally:
            row_driver.run_row_on_the_lane = original
        self.assertEqual(sorted(number for number, _ in taken), [1, 2, 3, 4, 5, 6, 7])
        self.assertEqual(sorted(results), [0, 1, 2, 3, 4, 5, 6])
        self.assertEqual(results[2]["status"], "driver_error")
        self.assertEqual(results[2]["error"], "RuntimeError:row three broke")
        self.assertEqual([results[position]["status"] for position in (0, 1, 3)], ["ok", "ok", "ok"])


PREFIX_FILE = "crates/pkg-prefix/src/lib.rs"
CORE_FILE = "crates/pkg-core/src/lib.rs"
CLEAN_SOURCES = {PREFIX_FILE: "pub fn prefix() -> u32 { 1 }\n", CORE_FILE: "pub fn core() -> u32 { 10 }\n"}
PACKAGE_DEPENDENCIES = (("crates/pkg-prefix", ()), ("crates/pkg-core", ("crates/pkg-prefix",)))  # 依赖在前；core 依赖 prefix，同 singlefs-core 依赖 singlefs-replay-prefix


def mutation_of(file: str, original: str, replacement: str) -> MutationRow:
    return MutationRow(f"{file} {original}->{replacement}", file, original, replacement, "-", "-", f"{file}\t{original}\t{replacement}")


class DriverRebuildTests(unittest.TestCase):
    """登记修订 5：上一行改过、这一行还原的文件，这一行一定重编。用真的 `sync_tree`（rsync -a）与真的还原（copy2），换掉 cargo、装置与挑行：
    假 cargo 照 cargo 的判法（草稿探针在 cargo 1.98 上实测）——包里任一源文件的修改时间晚于上一次编这个包的起点、或它依赖的包这次重编了，才重编，
    重编时把源文件内容记成「编进去的」；JSON 消息照 `--message-format=json` 的 `compiler-artifact` 写 `fresh`。
    第一行改 prefix、第二行改 core：干净的驱动在第二行把 prefix 重编回快照的内容；不置时间，第二行编进去的 prefix 还是第一行的变异。"""

    def setUp(self) -> None:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = holder.name
        self.work_root = os.path.join(self.root, "work")
        snapshot = os.path.join(self.work_root, "snapshot")
        an_hour_ago_ns = time.time_ns() - 3600 * 10**9  # 快照里的源文件是早先改的：修改时间早于这条线道上任何一次编译
        for path, text in CLEAN_SOURCES.items():
            write(os.path.join(snapshot, path), text)
            os.utime(os.path.join(snapshot, path), ns=(an_hour_ago_ns, an_hour_ago_ns))
        self.built_contents: dict[str, str] = {}
        self.build_started_ns: dict[str, int] = {}
        self.compile_exit = 0
        self.compile_fails_on: str | None = None  # 编进去的源文件里有这一串时假 cargo 退 101（那个包这次不算编成）
        self.next_mutation: MutationRow | None = None
        replaced = {"run_compile": self.fake_compile, "run_device": lambda *unused: (0, 0),
                    "prepare_row": lambda *unused: row_driver.RowChoice(self.next_mutation, STATUS_LOCATED, "-", True, 0, ())}
        originals = {"run_compile": row_driver.run_compile, "run_device": row_driver.run_device, "prepare_row": row_driver.prepare_row}  # 换掉之前逐个点名留下原件
        for name, replacement in replaced.items():
            setattr(row_driver, name, replacement)
            self.addCleanup(setattr, row_driver, name, originals[name])

    def fake_compile(self, lane_tree, lane_target, repo_root, threads, log_path, artifacts_path):
        started_ns = time.time_ns()
        os.makedirs(lane_target, exist_ok=True)
        rebuilt: set[str] = set()
        messages = []
        for package, dependencies in PACKAGE_DEPENDENCIES:
            source = os.path.join(lane_tree, package, "src", "lib.rs")
            newer = package not in self.build_started_ns or os.stat(source).st_mtime_ns > self.build_started_ns[package]
            if newer or any(dependency in rebuilt for dependency in dependencies):
                with open(source, encoding="utf-8") as handle:
                    text = handle.read()
                if self.compile_fails_on is not None and self.compile_fails_on in text:
                    write(artifacts_path, "\n".join(messages) + "\n")
                    write(log_path, "error[E0425]: cannot find value\n")
                    time.sleep(0.05)
                    return 101, 1
                rebuilt.add(package)
                self.build_started_ns[package] = started_ns
                self.built_contents[package] = text
            messages.append(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(lane_tree, package, "Cargo.toml"),
                                        "target": {"kind": ["lib"], "name": package.split("/")[1]}, "fresh": package not in rebuilt}))
        write(artifacts_path, "\n".join(messages) + "\n")
        write(log_path, "")
        time.sleep(0.05)  # 编译要花时间：文件系统的修改时间是粗粒度的钟，之后写的文件要落在这次起点之后
        return self.compile_exit, 1

    def run_row(self, number: int, mutation: MutationRow) -> dict[str, object]:
        self.next_mutation = mutation
        arguments = argparse.Namespace(skip_finished=False, output_root=os.path.join(self.root, "rows"), work_root=self.work_root, repo_root="/repo",
                                       threads=1, memory_cap="1G", positive_control=None, disable_oracle=None, no_executor=True)
        return row_driver.run_row_on_the_lane(number, None, None, [], set(), set(), arguments, 0)

    def record(self) -> row_driver.LaneBuildRecord | None:
        return row_driver.read_lane_build_record(os.path.join(self.work_root, "lane0"))

    def test_a_file_the_previous_row_changed_is_rebuilt_from_the_snapshot_in_this_row(self) -> None:
        first = self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        self.assertEqual(self.built_contents["crates/pkg-prefix"], "pub fn prefix() -> u32 { 2 }\n")
        self.assertEqual(first[row_driver.REBUILD_CHECK_FIELD], "crates/pkg-prefix")
        second = self.run_row(2, mutation_of(CORE_FILE, "{ 10 }", "{ 20 }"))
        self.assertEqual(self.built_contents["crates/pkg-prefix"], CLEAN_SOURCES[PREFIX_FILE], "第二行编进去的 prefix 要是快照的内容，不是第一行的变异")
        self.assertEqual(self.built_contents["crates/pkg-core"], "pub fn core() -> u32 { 20 }\n")
        self.assertEqual((second["status"], second["device_exit"], second[row_driver.REBUILD_CHECK_FIELD]),
                         ("ok", 0, "crates/pkg-core,crates/pkg-prefix"))
        self.assertEqual(self.record().files_to_rebuild, (CORE_FILE,))

    def test_without_marking_the_restored_file_the_check_before_the_compile_stops_the_row(self) -> None:
        self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        with unittest.mock.patch.object(row_driver, "mark_files_newer_than_every_build", lambda *unused: None):
            with self.assertRaises(row_driver.MeasurementPipeDefect) as caught:
                self.run_row(2, mutation_of(CORE_FILE, "{ 10 }", "{ 20 }"))
        self.assertEqual((caught.exception.stage, caught.exception.details), ("before_the_compile", [f"{PREFIX_FILE}:not_newer_than_the_last_compile"]))

    def test_without_marking_and_the_check_before_the_compile_the_check_after_it_names_the_package_built_with_the_previous_mutation(self) -> None:
        self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        with unittest.mock.patch.object(row_driver, "mark_files_newer_than_every_build", lambda *unused: None), \
                unittest.mock.patch.object(row_driver, "files_that_would_not_be_rebuilt", lambda *unused: []):
            with self.assertRaises(row_driver.MeasurementPipeDefect) as caught:
                self.run_row(2, mutation_of(CORE_FILE, "{ 10 }", "{ 20 }"))
        self.assertEqual((caught.exception.stage, caught.exception.details), ("after_the_compile", ["crates/pkg-prefix:fresh_not_rebuilt"]))
        self.assertEqual(self.built_contents["crates/pkg-prefix"], "pub fn prefix() -> u32 { 2 }\n", "假 cargo 真的复现了带着跑")
        self.assertEqual(self.record().files_to_rebuild, (CORE_FILE, PREFIX_FILE))

    def test_a_failed_compile_keeps_every_file_in_the_record_for_the_next_row(self) -> None:
        self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        self.compile_exit = 101
        failed = self.run_row(2, mutation_of(CORE_FILE, "{ 10 }", "{ 20 }"))
        self.assertEqual((failed["status"], self.record().files_to_rebuild), ("compile_failed", (CORE_FILE, PREFIX_FILE)))

    def test_a_row_that_does_not_compile_is_built_again_with_its_substitute_in_the_same_row_directory(self) -> None:
        self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        broken = mutation_of(CORE_FILE, "{ 10 }", "{ BROKEN }")
        substitute = mutation_of(CORE_FILE, "{ 10 }", "{ 30 }")
        self.compile_fails_on = "BROKEN"
        asked: list[str] = []

        def next_row(selected, table, rows_in_the_table, used_substitutes, snapshot_root, failed):
            asked.append(failed.mutation.name)
            return row_driver.RowChoice(substitute, STATUS_LOCATED, "表里的原名", False, 1, ((failed.mutation.name, "compile_failed"),))
        with unittest.mock.patch.object(row_driver, "substitute_after_a_compile_failure", next_row):
            fields = self.run_row(2, broken)
        self.assertEqual(asked, [broken.name])
        self.assertEqual((fields["status"], fields["substituted_from"], fields["mutated_row_name"], fields["failed_attempts"],
                          fields["failed_attempt_1"], fields[row_driver.REBUILD_CHECK_FIELD]),
                         ("ok", "表里的原名", substitute.name, 1, f"compile_failed:{broken.name}", "crates/pkg-core,crates/pkg-prefix"))
        self.assertEqual(self.built_contents["crates/pkg-core"], "pub fn core() -> u32 { 30 }\n")
        self.assertEqual(self.built_contents["crates/pkg-prefix"], CLEAN_SOURCES[PREFIX_FILE], "补位那一次编进去的 prefix 是快照的，不是第一行的变异")
        row_directory = os.path.join(self.root, "rows", "row-2")
        self.assertEqual(sorted(name for name in os.listdir(row_directory) if name.startswith("compile")), ["compile-failed-1.log", "compile.log"])
        self.assertEqual(self.record().files_to_rebuild, (CORE_FILE,))
        written = dict(read_tsv(os.path.join(row_directory, "row-result.tsv")))
        self.assertEqual((written["substituted_from"], written["failed_attempt_1"]), ("表里的原名", f"compile_failed:{broken.name}"))

    def test_the_replay_switch_keeps_a_compile_failure_without_asking_for_a_substitute(self) -> None:
        self.compile_fails_on = "BROKEN"
        broken = mutation_of(CORE_FILE, "{ 10 }", "{ BROKEN }")
        asked: list[str] = []
        self.next_mutation = broken
        arguments = argparse.Namespace(skip_finished=False, output_root=os.path.join(self.root, "rows"), work_root=self.work_root, repo_root="/repo",
                                       threads=1, memory_cap="1G", positive_control=None, disable_oracle=None, no_executor=True,
                                       keep_compile_failures_as_they_are=True)
        with unittest.mock.patch.object(row_driver, "substitute_after_a_compile_failure", lambda *unused: asked.append("asked")):
            fields = row_driver.run_row_on_the_lane(1, None, None, [], set(), set(), arguments, 0)
        self.assertEqual((asked, fields["status"], fields["substituted_from"], fields["failed_attempt_1"]),
                         ([], "compile_failed", "-", f"compile_failed:{broken.name}"))

    def test_a_row_whose_substitutes_all_fail_to_compile_is_a_compile_failure_with_every_attempt_listed(self) -> None:
        self.compile_fails_on = "BROKEN"
        broken = mutation_of(CORE_FILE, "{ 10 }", "{ BROKEN }")
        fields = self.run_row(1, broken)  # 空表：组里没有可换的行，substitute_after_a_compile_failure 交回补不上
        self.assertEqual((fields["status"], fields["mutated_row_name"], fields["failed_attempts"], fields["failed_attempt_1"], fields["device_exit"]),
                         ("compile_failed", "-", 1, f"compile_failed:{broken.name}", "-"))
        self.assertTrue(os.path.exists(os.path.join(self.root, "rows", "row-1", "compile-failed-1.log")))
        self.assertEqual(self.record().files_to_rebuild, (CORE_FILE,))

    def test_a_lane_built_without_a_record_or_on_another_snapshot_is_refused(self) -> None:
        os.makedirs(os.path.join(self.work_root, "lane0", "target"))
        self.assertEqual(row_driver.work_root_refusals(self.work_root, 2, "d1"), ["lane0:built_without_a_build_record", "snapshot_digest_missing"])
        with self.assertRaises(row_driver.MeasurementPipeDefect) as caught:
            self.run_row(1, mutation_of(PREFIX_FILE, "{ 1 }", "{ 2 }"))
        self.assertEqual(caught.exception.stage, "lane_without_a_build_record")
        row_driver.write_lane_build_record(os.path.join(self.work_root, "lane0"), row_driver.LaneBuildRecord(5, (CORE_FILE,)))
        write(os.path.join(self.work_root, row_driver.SNAPSHOT_DIGEST_FILE), "d1\n")
        self.assertEqual(row_driver.work_root_refusals(self.work_root, 2, "d1"), [])
        self.assertEqual(row_driver.work_root_refusals(self.work_root, 2, "d2"), ["snapshot_changed:d1->d2"])
        self.assertEqual(row_driver.read_lane_build_record(os.path.join(self.work_root, "lane0")), row_driver.LaneBuildRecord(5, (CORE_FILE,)))

    def test_cargo_messages_are_read_per_library_and_a_package_without_a_library_artifact_is_named(self) -> None:
        artifacts = os.path.join(self.root, "artifacts.jsonl")
        write(artifacts, "\n".join([
            json.dumps({"reason": "compiler-artifact", "manifest_path": "/t/crates/a/Cargo.toml", "target": {"kind": ["lib"]}, "fresh": True}),
            json.dumps({"reason": "compiler-artifact", "manifest_path": "/t/crates/b/Cargo.toml", "target": {"kind": ["bin"]}, "fresh": False}),
            json.dumps({"reason": "build-finished", "success": True}), "not json from a wrapper"]) + "\n")
        freshness = row_driver.library_freshness_by_package(artifacts, "/t")
        self.assertEqual(freshness, {"crates/a": True})
        self.assertEqual(row_driver.packages_left_stale(freshness, ("crates/a/src/x.rs", "crates/b/src/y.rs")),
                         ["crates/a:fresh_not_rebuilt", "crates/b:no_library_artifact"])
