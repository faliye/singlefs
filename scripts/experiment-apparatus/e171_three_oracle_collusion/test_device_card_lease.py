"""row_driver.py 的按需领卡（文件头「按需领卡」）：不起 cargo、不起装置，编译与装置两步换成桩，领卡客户端是小仓里一份假的 task-server/task
（领一回发一张新卡 card-<序号>，往事件记录里记「take <卡>」「return <卡>」；FAKE_CARD_LEASE=refused 时领卡退 3，return-unanswered 时还卡退 1）。
编译桩记「compile <变异名>」，装置桩记「device <行目录名> stage=<record|judge|-> cards=<交给装置的卡>」（不分段起的写 -），几样记进同一份事件记录，按先后核；
装置桩照 failing_stage 演哪一段退 3。判定库是每一趟一份新的内存假库（test_row_shards.py 的 fake_item_verdicts）。

跑法：在这个目录里 `python3 -B -m unittest test_device_card_lease`。
弄坏开关（row_driver.py 文件头「按需领卡的几处」逐项写了打开后红的用例）：
  E171_ROW_DRIVER_BREAK=card-held-through-the-compile、card-kept-after-the-device、lanes-share-one-card、card-lease-refusal-runs-the-device、
  card-lease-channel-unchecked、card-return-failure-unreported、card-held-through-the-record-stage、cards-left-in-the-record-environment、
  card-returned-before-the-judge-stage、judge-stage-after-a-failed-record、card-kept-after-a-failed-judge。
"""
import contextlib
import io
import os
import stat
import tempfile
import threading
import unittest
from unittest import mock

import device_card_lease
import row_driver
from test_row_shards import FakeLibrary, fake_item_verdicts, tiny_repository, write

FAKE_CLIENT = """#!/usr/bin/env bash
log="${FAKE_CARD_LEASE_LOG:?}"
case "${1:-} ${2:-}" in
  "lease take")
    if [[ "${FAKE_CARD_LEASE:-}" == refused ]]; then echo "  ✗ 服务没给这一回领卡：自证演的拒" >&2; exit 3; fi
    exec 9>>"$log.lock"
    flock 9
    number=$(( $(cat "$log.count" 2>/dev/null || echo 0) + 1 ))
    echo "$number" >"$log.count"
    echo "take card-$number" >>"$log"
    echo "card-$number" ;;
  "lease return")
    echo "return ${3:-all}" >>"$log"
    if [[ "${FAKE_CARD_LEASE:-}" == return-unanswered ]]; then echo "  ✗ 还卡的请求 120 秒没有回执（自证演的）" >&2; exit 1; fi ;;
  *) echo "假领卡客户端只认 lease take、lease return，收到：$*" >&2; exit 2 ;;
esac
"""
FAILED_STAGE_EXIT = 3


class CardLeaseTests(unittest.TestCase):
    def setUp(self) -> None:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.base = holder.name
        self.repository = os.path.join(self.base, "repository")
        self.selection = tiny_repository(self.repository)
        client = device_card_lease.client_of(self.repository)
        write(client, FAKE_CLIENT)
        os.chmod(client, os.stat(client).st_mode | stat.S_IXUSR)
        self.log = os.path.join(self.base, "events.log")
        write(self.log, "")
        channel = os.path.join(self.base, "card-lease-channel")
        os.makedirs(channel)
        environment = mock.patch.dict(os.environ, {device_card_lease.CHANNEL_DIRECTORY_VARIABLE: channel,
                                                   device_card_lease.CHANNEL_TOKEN_VARIABLE: "selftest-token", "FAKE_CARD_LEASE_LOG": self.log})
        environment.start()
        self.addCleanup(environment.stop)
        os.environ.pop("FAKE_CARD_LEASE", None)
        self.barrier: threading.Barrier | None = None
        self.failing_stage: str | None = None

    def note(self, line: str) -> None:
        with open(self.log, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")

    def events(self) -> list[str]:
        with open(self.log, encoding="utf-8") as handle:
            return handle.read().splitlines()

    def compile_stub(self, mutation, lane_tree, lane_target, snapshot_root, lane_directory, row_directory, arguments):
        self.note(f"compile {mutation.name}")
        return 0, 3, "crates/singlefs-core"

    def device_stub(self, repo_root, binary, row_directory, threads, extra, cards=None, stage=None, time_limit_seconds=None):
        self.note(f"device {os.path.basename(row_directory)} stage={stage or '-'} cards={cards}")
        if self.barrier is not None and stage == device_card_lease.JUDGE_STAGE:
            self.barrier.wait(timeout=20)   # 两条线道的判段同时在跑
        if stage is not None and stage == self.failing_stage:
            return FAILED_STAGE_EXIT, 7
        for name in ("device.out", "device.err"):
            write(os.path.join(row_directory, name), "")
        write(os.path.join(row_directory, "dump", "stream", "oracle-verdicts.tsv"), "state\tverdict\n")
        return 0, 7

    def run_driver(self, rows: str, lanes: int = 1, lease: bool = True) -> tuple[int, str, str]:
        arguments = ["--repo-root", self.repository, "--work-root", os.path.join(self.base, "work"), "--output-root", os.path.join(self.base, "output"),
                     "--selected-rows", self.selection, "--no-executor", "--lanes", str(lanes), "--threads", "1", "--rows", rows]
        if lease:
            arguments.append("--lease-card-per-device-run")
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(row_driver, "BREAK_ACCEPTED_IN_PROCESS", True), \
                mock.patch.object(row_driver, "compile_one_mutation", self.compile_stub), \
                mock.patch.object(row_driver, "run_device", self.device_stub), fake_item_verdicts(FakeLibrary()), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = row_driver.main(arguments)
        return code, out.getvalue(), err.getvalue()

    def test_no_card_is_held_while_compiling_or_recording_and_one_is_taken_and_returned_around_each_judge_stage(self) -> None:
        code, _, _ = self.run_driver("1,2")
        self.assertEqual(code, 0)
        self.assertEqual(self.events(), ["compile a1", "device row-1 stage=record cards=None", "take card-1", "device row-1 stage=judge cards=card-1", "return card-1",
                                         "compile a2", "device row-2 stage=record cards=None", "take card-2", "device row-2 stage=judge cards=card-2", "return card-2"])

    def test_the_record_stage_environment_carries_no_cards(self) -> None:
        with mock.patch.dict(os.environ, {device_card_lease.CARDS_VARIABLE: "card-from-the-caller"}):
            record = row_driver.device_environment(1, None, device_card_lease.RECORD_STAGE)
            judge = row_driver.device_environment(1, "card-7", device_card_lease.JUDGE_STAGE)
            whole = row_driver.device_environment(1)
        self.assertNotIn(device_card_lease.CARDS_VARIABLE, record, "录入段的环境里不给卡串")
        self.assertEqual(judge[device_card_lease.CARDS_VARIABLE], "card-7")
        self.assertEqual(whole[device_card_lease.CARDS_VARIABLE], "card-from-the-caller", "不分段起照此刻环境里的卡串")
        self.assertEqual(row_driver.device_command("/bin/dev", "/out/dump", [], device_card_lease.RECORD_STAGE)[-2:], ["--stage", "record"])

    def test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage(self) -> None:
        self.failing_stage = device_card_lease.RECORD_STAGE
        code, _, _ = self.run_driver("1")
        self.assertEqual(code, 0)
        self.assertEqual(self.events(), ["compile a1", "device row-1 stage=record cards=None"], "录入段退非 0：不领卡、判段不起")
        self.assertEqual(row_driver.read_result(os.path.join(self.base, "output", "row-1"))["device_exit"], str(FAILED_STAGE_EXIT),
                         "这一行的装置退出码是录入段的")

    def test_a_failed_judge_stage_returns_its_card(self) -> None:
        self.failing_stage = device_card_lease.JUDGE_STAGE
        code, _, _ = self.run_driver("1")
        self.assertEqual(code, 0)
        self.assertEqual(self.events(), ["compile a1", "device row-1 stage=record cards=None", "take card-1", "device row-1 stage=judge cards=card-1", "return card-1"])
        self.assertEqual(row_driver.read_result(os.path.join(self.base, "output", "row-1"))["device_exit"], str(FAILED_STAGE_EXIT))

    def test_two_lanes_running_their_devices_at_once_hold_different_cards(self) -> None:
        self.barrier = threading.Barrier(2)
        code, stdout, _ = self.run_driver("1,4", lanes=2)
        self.assertEqual(code, 0, stdout)
        events = self.events()
        seen = sorted(line.split("cards=", 1)[1] for line in events if line.startswith("device ") and " stage=judge " in line)
        taken = sorted(line.removeprefix("take ") for line in events if line.startswith("take "))
        returned = sorted(line.removeprefix("return ") for line in events if line.startswith("return "))
        self.assertEqual(len(set(seen)), 2, f"两条线道同时在判段，应当各持一张不同的卡：{events}")
        self.assertEqual(seen, taken)
        self.assertEqual(returned, taken, "每条线道照领到的卡串还自己那一张")

    def test_a_refused_card_lease_is_a_measurement_pipe_defect_and_the_judge_stage_is_not_started(self) -> None:
        os.environ["FAKE_CARD_LEASE"] = "refused"
        code, stdout, stderr = self.run_driver("1")
        self.assertEqual(code, row_driver.EXIT_MEASUREMENT_PIPE_DEFECT)
        self.assertEqual([line for line in self.events() if line.startswith("device ") and " stage=record " not in line], [], "领卡没成不起判段")
        self.assertIn(f"status={row_driver.STATUS_MEASUREMENT_PIPE_DEFECT} stage={row_driver.STAGE_CARD_LEASE} detail=take_exit=3", stdout)
        self.assertFalse(os.path.exists(os.path.join(self.base, "output", "row-1", "row-result.tsv")), "领卡没成的行不当成判过")
        self.assertIn("领卡没成", stderr)

    def test_without_a_lease_channel_the_driver_exits_78_before_any_row(self) -> None:
        os.environ.pop(device_card_lease.CHANNEL_DIRECTORY_VARIABLE)
        code, _, stderr = self.run_driver("1")
        self.assertEqual(code, device_card_lease.EXIT_NO_CHANNEL)
        self.assertEqual(self.events(), [], "开跑前就退，一行都不编、不领卡")
        self.assertIn(device_card_lease.CHANNEL_DIRECTORY_VARIABLE, stderr)

    def test_a_failed_card_return_is_reported_and_the_row_stands(self) -> None:
        os.environ["FAKE_CARD_LEASE"] = "return-unanswered"
        code, _, stderr = self.run_driver("1")
        self.assertEqual(code, 0)
        self.assertEqual(row_driver.read_result(os.path.join(self.base, "output", "row-1"))["status"], row_driver.STATUS_OK)
        self.assertIn("还卡没成", stderr)
        self.assertIn("lease return 退 1", stderr)

    def test_without_the_lease_option_no_card_is_taken(self) -> None:
        code, _, _ = self.run_driver("1", lease=False)
        self.assertEqual(code, 0)
        self.assertEqual(self.events(), ["compile a1", "device row-1 stage=- cards=None"])


if __name__ == "__main__":
    unittest.main()
