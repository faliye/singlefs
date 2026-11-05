"""current_tree_row_driver.py 的单测：编装置的命令带 gpu 特性、前面不套 nice 与任何包装（假 cargo、假 nice 记下真起的进程）；装置命令不带内存包装；
超过限时杀整组、退出码记 253。弄坏开关 E201_ROW_DRIVER_BREAK 见 current_tree_row_driver.py 文件头。
跑法：在本目录 `python3 -B -m unittest test_current_tree_row_driver`。"""

from __future__ import annotations

import argparse
import json
import os
import stat
import subprocess
import tempfile
import time
import unittest
from unittest import mock

import current_tree_row_driver as driver


def executable(path: str, text: str) -> None:
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


class SyncTreeTests(unittest.TestCase):
    """登记修订 5：线道树从快照同步时带上 prune-server（crates 经 path 依赖到它），不带它的 target。"""

    def test_the_lane_tree_has_the_prune_server_manifest_and_not_its_target(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            snapshot, lane = os.path.join(root, "snapshot"), os.path.join(root, "lane0", "tree")
            for relative in ("Cargo.toml", "crates/a/src/lib.rs", "prune-server/Cargo.toml", "prune-server/src/lib.rs", "prune-server/target/release/stale.rs"):
                os.makedirs(os.path.dirname(os.path.join(snapshot, relative)), exist_ok=True)
                with open(os.path.join(snapshot, relative), "w", encoding="utf-8") as handle:
                    handle.write("x\n")
            driver.sync_tree(snapshot, lane)
            copied = sorted(os.path.relpath(os.path.join(base, name), lane) for base, _, names in os.walk(lane) for name in names)
        self.assertIn("prune-server/Cargo.toml", copied)
        self.assertIn("prune-server/src/lib.rs", copied)
        self.assertNotIn("prune-server/target/release/stale.rs", copied)

    def test_the_synced_entries_name_the_prune_server_directory(self) -> None:
        self.assertIn("prune-server", driver.SYNCED_ENTRIES)

    def test_the_driver_hands_the_main_configuration_to_what_it_starts(self) -> None:
        """线道树不是 git 工作树：逐行驱动一开头（参数解析之前）就把主工作区那一份多机配置的绝对路径交给 SINGLEFS_MULTI_HOST_CONFIG，之后起的装置与判法都继承。"""
        environment = {name: value for name, value in os.environ.items() if name not in ("SINGLEFS_MULTI_HOST_CONFIG", driver.BREAK_VARIABLE)}
        with mock.patch.dict(os.environ, environment, clear=True), mock.patch("sys.stderr"):
            with self.assertRaises(SystemExit):
                driver.main([])
            handed = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")
        self.assertEqual(handed, driver.lib_repository_copy.configuration_path(driver.e171_research_side.REPOSITORY_ROOT, {}))

    def test_the_lane_tree_has_the_repository_root_files_under_their_directories_and_nothing_beside_them(self) -> None:
        """登记修订 6：单个文件的条目同步进它在线道树里的上级目录（rsync 不建缺的上级目录）；同目录里没点名的文件不同步。"""
        with tempfile.TemporaryDirectory() as root:
            snapshot, lane = os.path.join(root, "snapshot"), os.path.join(root, "lane0", "tree")
            for relative in ("Cargo.toml", ".claude/heavy-thread-default", "scripts/hosts/peer-host-lib.sh", ".claude/scripts/preflight.sh",
                             ".claude/scripts/other.sh"):
                os.makedirs(os.path.dirname(os.path.join(snapshot, relative)), exist_ok=True)
                with open(os.path.join(snapshot, relative), "w", encoding="utf-8") as handle:
                    handle.write("x\n")
            driver.sync_tree(snapshot, lane)
            copied = sorted(os.path.relpath(os.path.join(base, name), lane) for base, _, names in os.walk(lane) for name in names)
        self.assertEqual(copied, [".claude/heavy-thread-default", ".claude/scripts/preflight.sh", "Cargo.toml", "scripts/hosts/peer-host-lib.sh"])


class CompileCommandTests(unittest.TestCase):
    def test_the_device_is_compiled_with_the_gpu_feature_and_without_a_wrapper(self) -> None:
        """登记 5.5：编装置带 `--features gpu`、不套 nice（假 cargo 与假 nice 在 PATH 最前，各记一行真起的进程）。"""
        with tempfile.TemporaryDirectory() as root:
            fakes, record = os.path.join(root, "fakes"), os.path.join(root, "record")
            os.makedirs(fakes)
            executable(os.path.join(fakes, "cargo"), '#!/bin/sh\necho "cargo $*" >> "$E201_FAKE_RECORD"\n')
            executable(os.path.join(fakes, "nice"), '#!/bin/sh\necho "nice $*" >> "$E201_FAKE_RECORD"\nshift 2\nexec "$@"\n')
            with mock.patch.dict(os.environ, {"PATH": fakes + os.pathsep + os.environ["PATH"], "E201_FAKE_RECORD": record}):
                exit_code, _ = driver.run_compile(root, os.path.join(root, "target"), root, 2, os.path.join(root, "compile.log"),
                                                  os.path.join(root, "artifacts.jsonl"))
            with open(record, encoding="utf-8") as handle:
                started = handle.read().splitlines()
        self.assertEqual(exit_code, 0)
        self.assertEqual(len(started), 1, started)
        words = started[0].split(" ")
        self.assertEqual(words[0], "cargo", started)
        self.assertIn("--features gpu", started[0])
        self.assertEqual(words[words.index("--bin") + 1], driver.DEVICE_BINARY_NAME)


class DeviceCommandTests(unittest.TestCase):
    def test_the_device_command_starts_the_timing_wrapper_itself_with_no_wrapper_in_front(self) -> None:
        command = driver.device_command("/bin/device", "/out/dump", ["--streams", "1,2"])
        self.assertEqual(command[1:5], ["-B", driver.DEVICE_TIMING_SCRIPT, "run", "--"])
        self.assertEqual(command[5:], ["/bin/device", "row", "--output-directory", "/out/dump", "--streams", "1,2"])
        self.assertFalse(any(word.endswith(".sh") or "memory-cap" in word or word == "nice" for word in command), command)

    def test_the_nine_positive_controls_are_accepted(self) -> None:
        """登记 5.2 第一张表九样：装置八样加 O3；可写挂载探针那一样不收。"""
        self.assertEqual(sorted(driver.DEVICE_POSITIVE_CONTROLS) + [driver.O3_CONTROL],
                         sorted(["O1-对拍", "O1-对拍-卸载", "O1-崩溃判官", "RefFS", "O2", "交回值", "内存对象", "panic"]) + ["O3"])


CORE_SOURCE = "crates/singlefs-core/src/allocator.rs"


def write_text(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


class RebuildCheckTests(unittest.TestCase):
    """作废 Z4（登记修订 5 的编前编后核对，compile_one_mutation）：cargo 换成假的 run_compile，按用例写编译单元消息（fresh 真假）。"""

    def compile_with_a_fake_cargo(self, fresh: bool, lane_text: str = "fn alpha() { 1 }\n", record_files: tuple[str, ...] = ()):
        from bad_implementation_table import MutationRow
        seen: list[str] = []

        def fake_compile(lane_tree, _lane_target, _repo_root, _threads, log_path, artifacts_path):
            with open(os.path.join(lane_tree, CORE_SOURCE), encoding="utf-8") as handle:
                seen.append(handle.read())
            write_text(log_path, "Compiling singlefs-core\n")
            message = {"reason": "compiler-artifact", "target": {"kind": ["lib"]}, "fresh": fresh,
                       "manifest_path": os.path.join(lane_tree, "crates", "singlefs-core", "Cargo.toml")}
            write_text(artifacts_path, json.dumps(message) + "\n")
            return 0, 5

        with tempfile.TemporaryDirectory() as root:
            snapshot, lane_tree, lane_directory, row_directory = (os.path.join(root, name) for name in ("snapshot", "lane-tree", "lane", "row"))
            write_text(os.path.join(snapshot, CORE_SOURCE), "fn alpha() { 1 }\n")
            write_text(os.path.join(lane_tree, CORE_SOURCE), lane_text)
            os.makedirs(row_directory)
            if record_files:
                driver.write_lane_build_record(lane_directory, driver.LaneBuildRecord(0, record_files))
            mutation = MutationRow("m1", CORE_SOURCE, "{ 1 }", "{ 2 }", "-", "-", "m1\tline")
            arguments = argparse.Namespace(repo_root=root, threads=2)
            with mock.patch.object(driver, "run_compile", side_effect=fake_compile) as compiled:
                try:
                    outcome = driver.compile_one_mutation(mutation, lane_tree, os.path.join(lane_directory, "target"), snapshot, lane_directory,
                                                          row_directory, arguments)
                except driver.MeasurementPipeDefect as defect:
                    outcome = defect
                with open(os.path.join(lane_tree, CORE_SOURCE), encoding="utf-8") as handle:
                    restored = handle.read()
                record = driver.read_lane_build_record(lane_directory)
            return outcome, seen, restored, record, compiled

    def test_a_rebuilt_package_is_reported_and_the_file_is_restored(self):
        outcome, seen, restored, record, _ = self.compile_with_a_fake_cargo(fresh=False)
        self.assertEqual(outcome, (0, 5, "crates/singlefs-core"))
        self.assertEqual(seen, ["fn alpha() { 2 }\n"], "编的那一刻是改坏的那一份")
        self.assertEqual(restored, "fn alpha() { 1 }\n")
        self.assertEqual(record.files_to_rebuild, (CORE_SOURCE,))

    def test_a_package_left_fresh_after_the_compile_is_a_measurement_pipe_defect(self):
        outcome, _, restored, _, _ = self.compile_with_a_fake_cargo(fresh=True)
        self.assertIsInstance(outcome, driver.MeasurementPipeDefect)
        self.assertEqual((outcome.stage, outcome.details), ("after_the_compile", ["crates/singlefs-core:fresh_not_rebuilt"]))
        self.assertEqual(restored, "fn alpha() { 1 }\n")

    def test_a_file_left_unlike_the_snapshot_before_the_compile_is_a_measurement_pipe_defect(self):
        outcome, _, _, _, compiled = self.compile_with_a_fake_cargo(fresh=False, lane_text="fn alpha() { 3 }\n", record_files=(CORE_SOURCE,))
        self.assertIsInstance(outcome, driver.MeasurementPipeDefect)
        self.assertEqual(outcome.stage, "before_the_compile")
        self.assertEqual(outcome.details, [f"{CORE_SOURCE}:content_differs_from_the_snapshot"])
        compiled.assert_not_called()


class RowTimeLimitTests(unittest.TestCase):
    def test_a_row_over_the_time_limit_is_killed_as_a_group_and_exits_253(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            marker = os.path.join(root, "grandchild-alive")
            # 子进程再起一个睡着的孙进程，超时要把整组杀掉：孙进程不留下标记文件
            command = ["bash", "-c", f"(sleep 3; : > {marker}) & sleep 30"]
            started = time.monotonic()
            exit_code = driver.run_with_time_limit(command, 1, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.assertEqual(exit_code, driver.ROW_TIMEOUT_EXIT_CODE)
            self.assertLess(time.monotonic() - started, 10)
            time.sleep(3.5)
            self.assertFalse(os.path.exists(marker), "孙进程没被一起杀掉")

    def test_a_row_inside_the_time_limit_returns_its_own_exit_code(self) -> None:
        self.assertEqual(driver.run_with_time_limit(["bash", "-c", "exit 7"], 30, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL), 7)


if __name__ == "__main__":
    unittest.main()
