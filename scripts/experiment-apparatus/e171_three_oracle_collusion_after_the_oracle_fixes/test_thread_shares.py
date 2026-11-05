"""线程额度只在一条任务的进程里分（E171（三oracle合谋测试） 第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 7）的单测：
第一段单元编排（segment_one_units_revision_8.py）给读数单元、小单元与逐行编排各分多少、合起来不超过额度、没设 SINGLEFS_THREAD_CAP 时照登记值、
余数怎么给、分到的数设进子进程环境里的线程数变量（直接起，任务里面不套线程包装，登记修订 8）；第 7 步单元 10 线程与逐行驱动每道 5 线程的比例（逐行限时 max(1800 秒, 3T) 依据的那个比例）；
第二段逐行编排（rows_orchestrator_segment_two.py）本机与转交到别的台的驱动各给一条线道、它那一槽的线程；
逐行驱动的编译、装置、O3 阳性对照，冻结执行器那一步与 RP2 那一趟装置各把线道（这一趟）的线程设进子进程的环境（修订 8）。
研究一侧变异表 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv
经 mutation_driver.py --unit-test-module test_thread_shares --unit-test-module test_revision_8 跑。
每条用例一个场景、自己建自己删临时目录；起子进程的那几处换成只记命令的假件，一个进程都不起。"""
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import types
import unittest
from unittest import mock

import frozen_o3
import reuse_library
import row_driver
import rows_orchestrator
import rows_orchestrator_segment_two as segment_two
import segment_one_units_revision_8 as units

REGISTRY = os.path.join(units.HERE, "..", "..", "..", ".claude", "heavy-thread-default")
READING_UNITS = sorted(name for name, unit in units.unit_table().items() if name not in ("step-9-z1", "step-10-z7"))


def registered_threads_in_the_file() -> int:
    """登记文件第一条非注释行的数（不经被测的取法读）。"""
    with open(REGISTRY, encoding="utf-8") as handle:
        for line in handle:
            if line.strip() and not line.lstrip().startswith("#"):
                return int(line.split()[0])
    raise AssertionError("登记文件里没有数")


def environment_without_the_cap() -> dict:
    environment = dict(os.environ)
    environment.pop("SINGLEFS_THREAD_CAP", None)
    return environment


def peak_of_placed(placed: list[tuple[str, float, float]], table: dict) -> int:
    """simulate 排出的 [(单元, 起, 止)] 里同一时刻在跑的单元分到的线程之和的峰值（同一时刻先收后起）。"""
    changes = sorted([(start, 1, table[name].threads) for name, start, _ in placed] + [(end, 0, -table[name].threads) for name, _, end in placed])
    total, peak = 0, 0
    for _, _, change in changes:
        total += change
        peak = max(peak, total)
    return peak


class LaunchRecorder:
    """subprocess.Popen 的假件：只记命令，不起进程。"""

    def __init__(self) -> None:
        self.commands: list[list[str]] = []
        self.environments: list[dict | None] = []

    def __call__(self, command, **options):
        self.commands.append(list(command))
        self.environments.append(options.get("env"))
        return types.SimpleNamespace(pid=0, poll=lambda: 0, returncode=0)


def thread_variables_in(environment: dict | None) -> dict[str, str | None]:
    """环境里线程数变量那一组各是什么（没设记 None）。"""
    return {name: (environment or {}).get(name) for name in frozen_o3.CHILD_THREAD_VARIABLES}


def every_thread_variable_set_to(threads: int) -> dict[str, str]:
    return {name: str(threads) for name in frozen_o3.CHILD_THREAD_VARIABLES}


class TemporaryDirectoryCase(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.mkdtemp(prefix="e171-thread-shares-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.scratch, ignore_errors=True)


class SegmentOneSharesTests(TemporaryDirectoryCase):
    def test_step_7_gets_ten_threads_and_a_row_lane_five_so_the_ratio_behind_the_row_time_limit_holds(self) -> None:
        # 逐行限时 max(1800 秒, 3T)：T 是第 7 步单元的进程挂钟，r1 修订 6 那一趟第 7 步 10 线程、逐行驱动每道 5 线程
        table = units.unit_table()
        self.assertEqual(table["step-7"].threads, 10)
        self.assertEqual(units.ROW_LANE_THREADS, 5)
        self.assertEqual(units.ROW_LANE_THREADS, rows_orchestrator.LANE_THREADS_MINIMUM)
        self.assertEqual(table["step-7"].threads, 2 * units.ROW_LANE_THREADS)
        lanes = rows_orchestrator.allocate(rows_orchestrator.two_hosts()[:1], units.rows_threads(31))
        self.assertEqual([(lane.slots, lane.threads_per_slot) for lane in lanes], [(6, 5)])

    def test_every_reading_unit_gets_ten_threads_and_the_two_small_units_one(self) -> None:
        table = units.unit_table()
        self.assertEqual({table[name].threads for name in READING_UNITS}, {10})
        self.assertEqual(len(READING_UNITS), 32)
        self.assertEqual((table["step-9-z1"].threads, table["step-10-z7"].threads), (1, 1))

    def test_the_rows_share_drops_the_remainder_so_every_lane_gets_exactly_five_threads(self) -> None:
        self.assertEqual(units.rows_threads(31), 30)
        self.assertEqual(units.rows_threads(30), 30)
        self.assertEqual(units.rows_threads(14), 10)
        one_host = rows_orchestrator.two_hosts()[:1]
        self.assertEqual(rows_orchestrator.allocate(one_host, units.rows_threads(14))[0].threads_per_slot, 5)
        self.assertEqual(rows_orchestrator.allocate(one_host, 14)[0].threads_per_slot, 7)   # 不去余数时那一道是 7 线程：比例就破了

    def test_the_quota_is_the_cap_the_service_set_and_the_registered_value_when_it_is_unset(self) -> None:
        settings = types.SimpleNamespace(repo=self.scratch)
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="12"), clear=True):
            self.assertEqual(units.local_budget(settings).threads, 12)
        with mock.patch.dict(os.environ, environment_without_the_cap(), clear=True):
            self.assertEqual(units.local_budget(settings).threads, registered_threads_in_the_file())

    def test_units_running_together_never_add_up_past_the_quota(self) -> None:
        table = units.unit_table()
        names = [name for name in table if name != "research-mutations"]   # 第 0 步报研究一侧没有不同时不排它（它与第 1 步同时跑，会占住链尾那一刻的一份）
        placed = units.simulate(table, names, units.Budget(31))
        self.assertEqual(len(placed), len(names))
        self.assertLessEqual(peak_of_placed(placed, table), 31)
        chain_end = max(end for name, _, end in placed if name == "step-2b")
        self.assertEqual(sorted(name for name, start, _ in placed if start == chain_end),
                         ["step-3-o1-crash-judge", "step-7", "step-9"])

    def test_the_remainder_left_by_three_reading_units_goes_only_to_a_small_unit(self) -> None:
        table = units.unit_table()
        three = [table["step-7"], table["step-9"], table["step-3-o1-crash-judge"]]
        budget = units.Budget(31)
        self.assertTrue(units.fits(table["step-10-z7"], three, budget))
        self.assertFalse(units.fits(table["step-3-reference-filesystem"], three, budget))
        self.assertFalse(units.fits(table["step-9-z1"], three + [table["step-10-z7"]], budget))

    def test_the_rows_start_only_after_every_required_unit_is_green_and_none_is_running(self) -> None:
        required = {"step-0", "step-1", "step-2", "step-2b", "step-7", "step-8"}
        gate_green = {"step-0", "step-1", "step-2", "step-2b", "step-7"}
        self.assertFalse(units.rows_may_start(required, gate_green, 1))
        self.assertFalse(units.rows_may_start(required, required, 1))
        self.assertTrue(units.rows_may_start(required, required, 0))

    def world_settings(self):
        world = units.fake_world(self.scratch)
        return units.settings_from_config(world.repo, world.config)

    def test_a_unit_is_started_itself_with_its_share_in_every_thread_variable(self) -> None:
        settings = self.world_settings()
        recorder = LaunchRecorder()
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True), \
                mock.patch.object(subprocess, "Popen", recorder):
            units.launch(settings, units.unit_table()["step-7"], 1, {})
            units.launch(settings, units.unit_table()["step-9-z1"], 1, {})
        self.assertEqual(recorder.commands, [["bash", units.UNIT_SCRIPT, "step-7"], ["bash", units.UNIT_SCRIPT, "step-9-z1"]])
        self.assertEqual([thread_variables_in(environment) for environment in recorder.environments],
                         [every_thread_variable_set_to(10), every_thread_variable_set_to(1)])

    def test_the_rows_orchestrator_is_started_itself_with_the_rows_share_in_every_thread_variable(self) -> None:
        command = units.rows_command(self.world_settings(), {}, "run", units.Budget(31))
        self.assertEqual(command[:3], ["nice", "-n", "19"])
        self.assertFalse(any(word.endswith(".sh") for word in command[:5]), command)
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True):
            environment = units.rows_environment(units.Budget(31))
        self.assertEqual(thread_variables_in(environment), every_thread_variable_set_to(30))
        self.assertEqual(environment.get("PYTHONDONTWRITEBYTECODE"), "1")

    def test_the_thread_variables_are_the_twenty_the_service_sets(self) -> None:
        self.assertEqual(len(frozen_o3.CHILD_THREAD_VARIABLES), 20)
        self.assertEqual(len(set(frozen_o3.CHILD_THREAD_VARIABLES)), 20)
        for name in ("CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "SINGLEFS_THREAD_CAP", "SINGLEFS_CRASH_JUDGE_CPU_THREADS"):
            self.assertIn(name, frozen_o3.CHILD_THREAD_VARIABLES)
        base = {"KEPT": "x", "SINGLEFS_THREAD_CAP": "31"}
        self.assertEqual(frozen_o3.thread_environment(7, base), dict(every_thread_variable_set_to(7), KEPT="x"))
        self.assertEqual(base, {"KEPT": "x", "SINGLEFS_THREAD_CAP": "31"})   # 交回的是拷贝，不改调用方那一份


class RowDriverThreadsTests(TemporaryDirectoryCase):
    """逐行驱动给编译、装置、O3 阳性对照，冻结执行器那一步，与 RP2 那一趟装置：直接起，环境里线程数变量都设成这一条线道（这一趟）的线程。
    这一条任务的额度设成 31，子进程拿到的若是额度而不是线道的数，断言红。"""

    def recorded_runs(self):
        seen: list[tuple[list[str], dict]] = []

        def run(command, **options):
            seen.append((list(command), options.get("env")))
            return types.SimpleNamespace(returncode=0, stdout="")
        return seen, run

    def test_the_compile_the_executor_and_the_image_control_get_the_threads_of_the_lane(self) -> None:
        seen, run = self.recorded_runs()
        arguments = types.SimpleNamespace(repo_root="/repo", threads=5, o3_control_bad_states="/e170")
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True), \
                mock.patch.object(subprocess, "run", run):
            row_driver.run_compile(self.scratch, os.path.join(self.scratch, "target"), "/repo", 5, os.path.join(self.scratch, "compile.log"),
                                   os.path.join(self.scratch, "artifacts"))
            frozen_o3.run_executor("/repo", self.scratch, ["s1"], 5, os.path.join(self.scratch, "o3.out"), os.path.join(self.scratch, "o3.log"))
            row_driver.run_image_control(arguments, self.scratch)
        self.assertEqual([thread_variables_in(environment) for _, environment in seen], [every_thread_variable_set_to(5)] * 3)
        self.assertEqual([command[:4] for command, _ in seen], [["nice", "-n", "19", "cargo"], ["nice", "-n", "19", "python3"],
                                                                ["nice", "-n", "19", row_driver.sys.executable]])
        self.assertEqual(seen[0][1].get("CARGO_TARGET_DIR"), os.path.join(self.scratch, "target"))

    def test_the_device_reads_the_threads_of_its_lane(self) -> None:
        device = os.path.join(self.scratch, "device.sh")
        with open(device, "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\nenv\n")
        os.chmod(device, 0o755)
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True):
            exit_code, _ = row_driver.run_device(self.scratch, device, self.scratch, 5, [], 60)
        self.assertEqual(exit_code, 0)
        with open(os.path.join(self.scratch, "device.out"), encoding="utf-8") as handle:
            seen = dict(line.split("=", 1) for line in handle.read().splitlines() if "=" in line and not line.startswith("E7TIMING"))
        self.assertEqual(thread_variables_in(seen), every_thread_variable_set_to(5))

    def test_the_reuse_selfproof_run_gets_the_threads_it_was_given(self) -> None:
        seen, run = self.recorded_runs()
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True), \
                mock.patch.object(subprocess, "run", run):
            reuse_library.selfproof_run("/repo", "/bin/device", "10", os.path.join(self.scratch, "run-1"))
        self.assertEqual([command for command, _ in seen], [["nice", "-n", "19", "/bin/device", "stream-constants", "--streams", "1,2,3"]])
        self.assertEqual(thread_variables_in(seen[0][1]), every_thread_variable_set_to(10))


def lane_arguments(command: list[str]) -> list[str]:
    """驱动参数里 --lanes 与 --threads 那四个词（各恰一次才交回，不是就交回整条好让断言打出来）。"""
    if command.count("--lanes") != 1 or command.count("--threads") != 1:
        return command
    lanes, threads = command.index("--lanes"), command.index("--threads")
    return command[lanes:lanes + 2] + command[threads:threads + 2]


class SegmentTwoLaneThreadsTests(TemporaryDirectoryCase):
    """第二段每个驱动只给一条线道、它那一槽的线程（驱动照它设编译、装置、执行器环境里的线程数变量）；这一条任务的额度设成 31，
    驱动拿到的若是额度而不是那一槽的 5，断言红。"""

    def context(self):
        return types.SimpleNamespace(attempt_dir=os.path.join(self.scratch, "attempt-1"), shards=os.path.join(self.scratch, "shards"),
                                     driver_path="row_driver.py", repo=self.scratch, tree="tree", baseline_timing="timing.out",
                                     step_root=lambda step: os.path.join(self.scratch, "rows"), own_host=1,
                                     relay_targets={2: 1}, relay_plan=os.path.join(self.scratch, "plan"), poll_seconds=0.05)

    def test_a_local_row_driver_gets_one_lane_and_the_threads_of_its_slot(self) -> None:
        recorder = LaunchRecorder()
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True), \
                mock.patch.object(subprocess, "Popen", recorder), mock.patch.object(segment_two.one, "driver_hashes", lambda context: {}):
            segment_two.launch_local_two(self.context(), {}, "rows-row-15-h1", segment_two.ROWS_STEP, (15,), 5, self.scratch, "local", set())
        self.assertEqual(lane_arguments(recorder.commands[0]), ["--lanes", "1", "--threads", "5"])

    def test_a_row_driver_handed_to_the_other_host_gets_one_lane_and_the_threads_of_its_slot(self) -> None:
        recorder = LaunchRecorder()
        with mock.patch.dict(os.environ, dict(environment_without_the_cap(), SINGLEFS_THREAD_CAP="31"), clear=True), \
                mock.patch.object(subprocess, "Popen", recorder):
            segment_two.launch_peer_two(self.context(), "kit", "rows-row-15-h2", segment_two.ROWS_STEP, 15, 5, 2, set())
        command = recorder.commands[0]
        driver = command[len(command) - command[::-1].index("--"):]   # 最后一个 -- 之后是交给驱动的参数
        self.assertEqual(lane_arguments(driver), ["--lanes", "1", "--threads", "5"])

    def test_the_slots_on_a_host_add_up_to_no_more_than_its_threads(self) -> None:
        one_host = rows_orchestrator.two_hosts()[:1]
        over = [(threads, lane.slots, lane.threads_per_slot) for threads in range(5, 65) for lane in rows_orchestrator.allocate(one_host, threads)
                if lane.slots * lane.threads_per_slot > threads]
        self.assertEqual(over, [])
        self.assertEqual([(lane.slots, lane.threads_per_slot) for lane in rows_orchestrator.allocate(one_host, 31)], [(6, 5)])


if __name__ == "__main__":
    unittest.main()
