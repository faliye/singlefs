"""shared_steps.py（E171 整表那几行共用的前置）：不起 cargo、不起装置。小仓里放假 cargo（编只记一笔；跑装置与 E170 导出装置照 --output-directory 建目录、
往标准输出打一行）、假领卡客户端（test_device_card_lease.py 那一份）、假 O3 判官、假执行器、假逐行驱动；几样往同一份事件记录里记，按先后核。
replay.sh 那一格读本仓真的 scripts/experiment-apparatus/replay.sh。long_run_item_verdicts 的算键换成请求原文的 sha256、判定库换成内存里的假库
（test_row_shards.py 的 fake_item_verdicts，一条用例一份）；真算键、连临时 prune-server 的复用三问在 shared_steps.py 自己的自证里。

跑法：在这个目录里 `python3 -B -m unittest test_shared_steps`。
弄坏开关（shared_steps.py 文件头逐项写了打开后红的用例）：
  E171_SHARED_STEPS_BREAK=reused-item-remade、prerequisites-unchecked、card-held-through-the-build、take-ignores-the-library、
  e170-dump-not-renamed、step-drifts-from-replay、request-without-features、failed-item-recorded-green、card-held-through-the-record-stage、
  cards-left-in-the-record-environment、card-returned-before-the-judge-stage、judge-stage-after-a-failed-record、card-kept-after-a-failed-judge。
假 cargo 跑装置时记「run <二进制> <模式> stage=<--stage 的值，没给写 -> cards=<GPU_CARDS_BY_PRIORITY，没设写 none>」；环境里 FAKE_FAILING_STAGE 点名的那一段装置退 9。
"""
import contextlib
import io
import os
import re
import shutil
import stat
import tempfile
import unittest
from unittest import mock

import device_card_lease
import frozen_o3
import row_driver
import shared_steps
from test_device_card_lease import FAKE_CLIENT
from test_row_shards import FakeLibrary, fake_item_verdicts, tiny_repository, write

HERE = os.path.dirname(os.path.abspath(__file__))
REPLAY = os.path.join(HERE, "..", "replay.sh")

FAKE_CARGO = """#!/usr/bin/env bash
case "${1:-}" in
  --version) echo "cargo 0.0.0 (selftest)"; exit 0 ;;
  build) echo "build" >>"$FAKE_EVENT_LOG"; exit 0 ;;
  run) ;;
  *) echo "假 cargo 只认 --version、build、run，收到：$*" >&2; exit 2 ;;
esac
binary=""; arguments=()
while (($#)); do
  if [[ "$1" == --bin ]]; then binary="$2"; shift 2; continue; fi
  if [[ "$1" == -- ]]; then shift; arguments=("$@"); break; fi
  shift
done
output=""; dump=""; stage="-"
for ((index = 0; index < ${#arguments[@]}; index++)); do
  case "${arguments[index]}" in
    --output-directory) output="${arguments[index + 1]}" ;;
    --dump-directory) dump="${arguments[index + 1]}" ;;
    --stage) stage="${arguments[index + 1]}" ;;
  esac
done
echo "run $binary ${arguments[0]} stage=$stage cards=${GPU_CARDS_BY_PRIORITY:-none}" >>"$FAKE_EVENT_LOG"
if [[ "$binary" == e171_three_oracle_collusion && "$stage" == "${FAKE_FAILING_STAGE:-none}" ]]; then exit 9; fi
if [[ -n "$output" ]]; then mkdir -p "$output/stream"; printf '%s\\n' "${arguments[*]}" >"$output/stream/oracle-verdicts.tsv"; fi
if [[ -n "$dump" && "$binary" == e170_specification_executor_export ]]; then printf 'bad\\n' >"$dump/bad-states.tsv"; fi
echo "E7RESULT name=fake ${arguments[0]}"
"""
FAKE_OUTPUT_WRITER = """import os, sys
arguments = sys.argv[1:]
with open(os.environ["FAKE_EVENT_LOG"], "a", encoding="utf-8") as log:
    log.write(LABEL + " " + arguments[0] + "\\n")
output = arguments[arguments.index("--output") + 1]
with open(output, "w", encoding="utf-8") as handle:
    handle.write(LABEL + " " + " ".join(os.path.basename(word) for word in arguments) + "\\n")
"""
FAKE_ROW_DRIVER = """import os, sys
arguments = sys.argv[1:]
with open(os.environ["FAKE_EVENT_LOG"], "a", encoding="utf-8") as log:
    log.write("row_driver " + " ".join(word for word in arguments if word.startswith("--") and word not in ("--repo-root", "--work-root", "--output-root")) + "\\n")
row = os.path.join(arguments[arguments.index("--output-root") + 1], "row-0")
os.makedirs(os.path.join(row, "dump"))
with open(os.path.join(row, "row-result.tsv"), "w", encoding="utf-8") as handle:
    handle.write("positive_control\\t" + arguments[arguments.index("--positive-control") + 1] + "\\n")
"""
PREREQUISITE_DUMPS = ["baseline-run1", "baseline-run2", "disable-O1", "disable-O2", "e170-bad-states-copy-s2", "e170-pinned-dump", "known-bad-s2"]
CONTROL_DUMPS = [f"control-baseline-{control}" for control in shared_steps.CONTROLS_WITH_A_DEVICE_BASELINE] + \
    [f"rows-controls-{control}" for control in (*shared_steps.CONTROLS_WITH_A_DEVICE_BASELINE, "O3")]
SCRATCH_FILES = ["baseline-run1.stdout", "images-o3.out", "o3-baseline.out", "o3-control-baseline.out"]


def executable(path: str, text: str) -> str:
    write(path, text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)
    return path


def tree_files(root: str) -> dict[str, bytes]:
    found = {}
    for directory, _, files in os.walk(root):
        for name in files:
            path = os.path.join(directory, name)
            with open(path, "rb") as handle:
                found[os.path.relpath(path, root)] = handle.read()
    return found


def driver_body(text: str, driver: str) -> str:
    """replay.sh 里一个驱动的正文：续行接起来、空白折成一个空格，一行一条逻辑行。"""
    lines, inside, pending = [], False, ""
    for line in text.splitlines():
        if line.startswith(f"{driver}() {{"):
            inside = True
            continue
        if inside and line.startswith("}"):
            break
        if not inside:
            continue
        if line.rstrip().endswith("\\"):
            pending += line.rstrip()[:-1] + " "
            continue
        lines.append(" ".join((pending + line).split()))
        pending = ""
    return "\n".join(lines)


class SharedStepsTests(unittest.TestCase):
    def setUp(self) -> None:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.base = holder.name
        self.repository = os.path.join(self.base, "repository")
        tiny_repository(self.repository)
        write(os.path.join(self.repository, shared_steps.RESEARCH_DIRECTORY_RELATIVE, "research.py"), "# 研究一侧\n")
        write(os.path.join(self.repository, frozen_o3.FROZEN_DIRECTORY_RELATIVE, frozen_o3.EXECUTOR_SCRIPT), "LABEL = 'executor'\n" + FAKE_OUTPUT_WRITER)
        write(os.path.join(self.repository, frozen_o3.FREEZE_REFERENCE_RELATIVE), "冻结行\n")
        executable(device_card_lease.client_of(self.repository), FAKE_CLIENT)
        executable(os.path.join(self.base, "bin", "cargo"), FAKE_CARGO)
        self.judge = os.path.join(self.base, "fakes", "judge.py")
        write(self.judge, "LABEL = 'judge'\n" + FAKE_OUTPUT_WRITER)
        self.row_driver = os.path.join(self.base, "fakes", "row_driver.py")
        write(self.row_driver, FAKE_ROW_DRIVER)
        self.log = os.path.join(self.base, "events.log")
        self.root = os.path.join(self.base, "shared")
        self.channel = os.path.join(self.base, "channel")
        os.makedirs(self.channel)
        self.library = FakeLibrary()

    def main(self, *arguments: str, gpu: bool = False) -> tuple[int, str]:
        """在进程里调 shared_steps.main：弄坏开关照起单测时环境里的 E171_SHARED_STEPS_BREAK；`gpu` 给领卡通道（假客户端）。"""
        environment = {"PATH": os.path.join(self.base, "bin") + os.pathsep + os.environ["PATH"], "FAKE_EVENT_LOG": self.log, "FAKE_CARD_LEASE_LOG": self.log}
        err = io.StringIO()
        with mock.patch.dict(os.environ, environment), mock.patch.object(shared_steps, "BREAK_ACCEPTED_IN_PROCESS", True), \
                mock.patch.object(shared_steps, "ROW_DRIVER_SCRIPT", self.row_driver), mock.patch.object(shared_steps, "IMAGE_JUDGE_SCRIPT", self.judge), \
                fake_item_verdicts(self.library), contextlib.redirect_stderr(err):
            for name in (device_card_lease.CHANNEL_DIRECTORY_VARIABLE, device_card_lease.CHANNEL_TOKEN_VARIABLE):
                os.environ.pop(name, None)
            if gpu:
                os.environ.update({device_card_lease.CHANNEL_DIRECTORY_VARIABLE: self.channel, device_card_lease.CHANNEL_TOKEN_VARIABLE: "selftest-token"})
            code = shared_steps.main(list(arguments))
        return code, err.getvalue()

    def run_item(self, item: str, gpu: bool = False) -> tuple[int, str]:
        return self.main("run", item, "--root", self.root, "--repo-root", self.repository, "--device-features", "gpu" if gpu else "none", gpu=gpu)

    def take(self, name: str) -> tuple[int, str, str, str]:
        dumps, scratch = os.path.join(self.base, name, "dumps"), os.path.join(self.base, name, "scratch")
        os.makedirs(dumps)
        os.makedirs(scratch)
        code, stderr = self.main("take", "--root", self.root, "--repo-root", self.repository, "--device-features", "none", "--dumps", dumps, "--scratch", scratch,
                                 "--e170-dump-name", "e170-pinned-dump")
        return code, stderr, dumps, scratch

    def events(self) -> list[str]:
        if not os.path.exists(self.log):
            return []
        with open(self.log, encoding="utf-8") as handle:
            return handle.read().splitlines()

    def run_every_item(self) -> None:
        for item in shared_steps.SHARED_ITEMS:
            code, stderr = self.run_item(item)
            self.assertEqual(code, 0, stderr)

    def test_a_second_run_takes_the_item_from_the_library_and_starts_nothing(self) -> None:
        code, stderr = self.run_item(shared_steps.PREREQUISITES)
        self.assertEqual(code, 0, stderr)
        self.assertIn("taken=no", stderr)
        self.assertTrue(self.events())
        made = tree_files(os.path.join(self.root, "prerequisites"))
        shutil.rmtree(os.path.join(self.root, "prerequisites"))
        os.remove(self.log)
        code, stderr = self.run_item(shared_steps.PREREQUISITES)
        self.assertEqual(code, 0, stderr)
        self.assertIn(f"E171_SHARED_STEP item={shared_steps.PREREQUISITES} taken=yes", stderr)
        self.assertEqual(self.events(), [])
        self.assertEqual(tree_files(os.path.join(self.root, "prerequisites")), made, "从库里取回的项目录与做出来的那一份逐个文件相同")

    def device_runs(self) -> list[tuple[str, str]]:
        """事件记录里装置（不含 E170 导出装置）每一回的 (段, 卡串)。"""
        runs = []
        for event in self.events():
            if event.startswith(f"run {row_driver.DEVICE_BINARY_NAME} "):
                stage, cards = event.split(" stage=", 1)[1].split(" cards=", 1)
                runs.append((stage, cards))
        return runs

    def test_no_card_is_held_while_building_or_recording_and_each_judge_stage_takes_and_returns_one(self) -> None:
        code, stderr = self.run_item(shared_steps.PREREQUISITES, gpu=True)
        self.assertEqual(code, 0, stderr)
        held, records, judges = None, 0, 0
        for event in self.events():
            if event.startswith("take "):
                self.assertIsNone(held, self.events())
                held = event.split(" ", 1)[1]
            elif event.startswith("return "):
                self.assertEqual(event.split(" ", 1)[1], held, self.events())
                held = None
            elif event == "build":
                self.assertIsNone(held, f"编装置的时候持着卡：{self.events()}")
            elif event.startswith(f"run {row_driver.DEVICE_BINARY_NAME} "):
                stage, cards = event.split(" stage=", 1)[1].split(" cards=", 1)
                if stage == device_card_lease.RECORD_STAGE:
                    self.assertTrue(held is None and cards == "none", f"录入段持着卡：{self.events()}")
                    records += 1
                else:
                    self.assertEqual((stage, cards), (device_card_lease.JUDGE_STAGE, held), self.events())
                    judges += 1
            elif event.startswith(f"run {shared_steps.EXPORTER_BINARY_NAME} "):
                self.assertTrue(event.endswith("cards=none") and held is None, self.events())
        self.assertIsNone(held)
        device_steps = sum(step.kind == shared_steps.DEVICE for step in shared_steps.PREREQUISITE_STEPS)
        self.assertEqual((records, judges), (device_steps, device_steps))

    def test_the_record_stage_runs_without_the_cards_of_the_caller(self) -> None:
        with mock.patch.dict(os.environ, {device_card_lease.CARDS_VARIABLE: "card-from-the-caller"}):
            code, stderr = self.run_item(shared_steps.PREREQUISITES, gpu=True)
        self.assertEqual(code, 0, stderr)
        runs = self.device_runs()
        self.assertTrue(runs)
        self.assertEqual([cards for stage, cards in runs if stage == device_card_lease.RECORD_STAGE], ["none"] * (len(runs) // 2), runs)
        self.assertNotIn("card-from-the-caller", [cards for stage, cards in runs if stage == device_card_lease.JUDGE_STAGE], runs)

    def test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage(self) -> None:
        with mock.patch.dict(os.environ, {"FAKE_FAILING_STAGE": device_card_lease.RECORD_STAGE}):
            code, stderr = self.run_item(shared_steps.PREREQUISITES, gpu=True)
        self.assertEqual(code, shared_steps.EXIT_STEP_FAILED, stderr)
        self.assertEqual(self.device_runs(), [(device_card_lease.RECORD_STAGE, "none")], self.events())
        self.assertEqual([event for event in self.events() if event.startswith(("take ", "return "))], [], "录入段退非 0：不领卡")
        self.assertIn("baseline", stderr)

    def test_a_failed_judge_stage_returns_its_card(self) -> None:
        with mock.patch.dict(os.environ, {"FAKE_FAILING_STAGE": device_card_lease.JUDGE_STAGE}):
            code, stderr = self.run_item(shared_steps.PREREQUISITES, gpu=True)
        self.assertEqual(code, shared_steps.EXIT_STEP_FAILED, stderr)
        self.assertEqual(self.device_runs(), [(device_card_lease.RECORD_STAGE, "none"), (device_card_lease.JUDGE_STAGE, "card-1")], self.events())
        self.assertEqual([event for event in self.events() if event.startswith(("take ", "return "))], ["take card-1", "return card-1"])

    def test_without_a_lease_channel_a_gpu_item_exits_78_before_any_step(self) -> None:
        code, stderr = self.main("run", shared_steps.PREREQUISITES, "--root", self.root, "--repo-root", self.repository, "--device-features", "gpu")
        self.assertEqual(code, device_card_lease.EXIT_NO_CHANNEL, stderr)
        self.assertEqual(self.events(), [])

    def test_the_o3_control_without_prerequisites_names_the_missing_item_and_exits_4(self) -> None:
        code, stderr = self.run_item(shared_steps.O3_CONTROL_ITEM)
        self.assertEqual(code, shared_steps.EXIT_MISSING, stderr)
        self.assertIn(f"要先有 {shared_steps.PREREQUISITES}", stderr)
        self.assertEqual(self.events(), [])
        self.assertEqual(self.run_item(shared_steps.PREREQUISITES)[0], 0)
        code, stderr = self.run_item(shared_steps.O3_CONTROL_ITEM)
        self.assertEqual(code, 0, stderr)
        self.assertIn("row_driver --empty-mutation-only --positive-control --o3-control-bad-states", self.events())

    def test_a_failed_step_names_the_step_and_is_judged_again(self) -> None:
        executable(os.path.join(self.base, "bin", "cargo"), FAKE_CARGO.replace('echo "E7RESULT', '[[ "${arguments[0]}" == known-bad-images ]] && exit 9\necho "E7RESULT'))
        code, stderr = self.run_item(shared_steps.PREREQUISITES)
        self.assertEqual(code, shared_steps.EXIT_STEP_FAILED, stderr)
        self.assertIn("known-bad-images", stderr)
        self.assertFalse(os.path.exists(os.path.join(self.root, "work-prerequisites")))
        executable(os.path.join(self.base, "bin", "cargo"), FAKE_CARGO)
        code, stderr = self.run_item(shared_steps.PREREQUISITES)
        self.assertEqual(code, 0, stderr)
        self.assertIn(f"E171_SHARED_STEP item={shared_steps.PREREQUISITES} taken=no", stderr, "有一步退非 0 的那一次照同一把键记红，下一趟现做")

    def test_take_copies_every_product_under_the_names_the_driver_uses(self) -> None:
        self.run_every_item()
        code, stderr, dumps, scratch = self.take("driver")
        self.assertEqual(code, 0, stderr)
        self.assertEqual(sorted(os.listdir(dumps)), sorted(PREREQUISITE_DUMPS + CONTROL_DUMPS))
        self.assertEqual(sorted(os.listdir(scratch)), SCRATCH_FILES)
        shared = tree_files(os.path.join(self.root, "prerequisites", shared_steps.DUMPS, shared_steps.E170_DUMP_SHARED_NAME))
        self.assertEqual(tree_files(os.path.join(dumps, "e170-pinned-dump")), shared)
        with open(os.path.join(scratch, "baseline-run1.stdout"), encoding="utf-8") as handle:
            self.assertEqual(handle.read(), "E7RESULT name=fake baseline\n")

    def test_take_refuses_items_the_library_does_not_hold(self) -> None:
        code, stderr, dumps, _ = self.take("nothing-done")
        self.assertEqual(code, shared_steps.EXIT_MISSING)
        for item in shared_steps.SHARED_ITEMS:
            self.assertIn(f"✗ E171 共用前置 {item} 库里没有", stderr)
        self.assertEqual(os.listdir(dumps), [])
        self.run_every_item()
        self.library = FakeLibrary()   # 共用输出根里的项目录都在，库里没有（另一份库、或键变了）
        code, stderr, dumps, _ = self.take("library-empty")
        self.assertEqual(code, shared_steps.EXIT_MISSING, stderr)
        self.assertIn("--item prerequisites", stderr)
        self.assertEqual(os.listdir(dumps), [])

    def test_the_request_names_the_item_the_features_and_what_each_item_runs(self) -> None:
        def tokens(item: str, features: str = "none") -> list[list[str]]:
            return [line.split("\t") for line in shared_steps.item_request_lines(item, self.repository, features, self.root)]

        with mock.patch.object(shared_steps, "BREAK_ACCEPTED_IN_PROCESS", True):
            prerequisites, prerequisites_on_gpu = tokens(shared_steps.PREREQUISITES), tokens(shared_steps.PREREQUISITES, "gpu")
            o3, o2 = tokens(shared_steps.O3_CONTROL_ITEM), tokens("positive-control:O2")
        self.assertIn("line:device-features=none", prerequisites[0])
        self.assertIn("line:device-features=gpu", prerequisites_on_gpu[0])
        self.assertIn(f"feature:{row_driver.DEVICE_PACKAGE}/gpu", prerequisites_on_gpu[0])
        self.assertNotIn(f"feature:{row_driver.DEVICE_PACKAGE}/gpu", prerequisites[0])
        self.assertEqual([line[1] for line in prerequisites], [shared_steps.DEVICE_ENTRY, shared_steps.EXPORTER_ENTRY])
        # 第一列是 `@files` 的那一行不是项的请求行，是把文件内容收进键的行（项名在它的第二列，下面单独钉）
        self.assertEqual({line[0] for line in prerequisites + o3 + o2 if line[0] != "@files"}, {"prerequisites", "positive-control.O3", "positive-control.O2"})
        self.assertIn(f"judge:{shared_steps.ROW_JUDGE}", o2[0])
        self.assertIn("line:control=O2", o2[0])
        self.assertNotIn(f"judge:{shared_steps.ROW_JUDGE}", prerequisites[0])
        self.assertIn(f"judge:{shared_steps.IMAGE_JUDGE}", o3[0])
        self.assertEqual(o3[1], ["@files", "positive-control.O3", os.path.join(self.root, "prerequisites", shared_steps.DUMPS, shared_steps.BAD_STATES_COPY)])
        self.assertTrue(all(name.isascii() for name in shared_steps.LIBRARY_ITEM_NAMES.values()))

    def test_the_bad_image_steps_match_the_single_row_drivers_and_feed_every_comparator_input(self) -> None:
        """PREREQUISITE_STEPS 是这几步的唯一定义（replay.sh 的 E171 整表四个驱动只取它们的产物，自己不做）。钉两样：
        坏镜像那几步（基线 run1、拷一份、E170 导出装置 bad-states 与它的种子、已知坏镜像、O3 判官）与 replay.sh 里还在的单列驱动逐词相同；
        四个驱动的比对器读的共用前置的每一样，都是这里某一步（或阳性对照那几步）写出的。四个驱动的正文里不再有装置与导出装置的命令。"""
        with open(REPLAY, encoding="utf-8") as handle:
            replay = handle.read()
        with mock.patch.object(shared_steps, "BREAK_ACCEPTED_IN_PROCESS", True):
            steps = shared_steps.steps_of(shared_steps.PREREQUISITES)
        single_row_bodies = " ".join(" ".join(driver_body(replay, helper).split())
                                     for helper in ("e171_s2_prepare_bad_states", "e171_s2_known_bad", "driver_e171_s2_images_o3"))
        command_words = {shared_steps.DEVICE: "e171_device", shared_steps.EXPORTER: "e171_e170_exporter", shared_steps.COPY: "cp -r",
                         shared_steps.JUDGE: shared_steps.SHELL_WORDS[shared_steps.JUDGE]}
        pinned = [step for position, step in enumerate(steps)
                  if position == 0 or (step.kind in command_words and step.arguments[0] in ("{dumps}/baseline-run1", "bad-states", "known-bad-images", "judge"))]
        self.assertEqual(len(pinned), 5, pinned)
        for step in pinned:
            words = [word.format(dumps="$scratch", scratch="$scratch") for word in step.arguments]
            text = " ".join([command_words[step.kind], *(f'"{word}"' if "$" in word else word for word in words)])
            self.assertIn(text, single_row_bodies)
        controls = shared_steps.CONTROLS_WITH_A_DEVICE_BASELINE
        for driver, e170_dump_name in (("driver_e171_s2_comparator", "e170-dump"), ("driver_e171_s4_comparator", "e170-pinned-dump"),
                                       ("driver_e171_s4_f4_comparator", "e170-pinned-dump"), ("driver_e171_s4_f4_substitutes_comparator", "e170-pinned-dump")):
            written = {f"control-baseline-{control}" for control in controls} | {f"rows-controls-{control}" for control in (*controls, row_driver.O3_CONTROL)}
            for step in steps:
                for word in (*step.arguments, step.stdout):
                    written |= {match.format(e170_dump=e170_dump_name) for match in re.findall(r"\{(?:dumps|scratch)\}/([^/\s\"]+)", word)}
            body = driver_body(replay, driver)
            comparator_lines = [line for line in body.splitlines() if "comparator.py" in line]
            self.assertTrue(comparator_lines, driver)
            read = {name for line in comparator_lines for name in re.findall(r'\$(?:dumps|scratch|s4)/([^/\s":]+)', line)}
            own = {name for name in read if name.startswith(("rows-s2", "rows-s4", "comparator")) or name.endswith("-driver.out")
                   or name in ("driver-tables.log", "frozen-tree", "dumps")}
            self.assertEqual(read - own - written, set(), driver)
            self.assertNotIn("e171_device", body, driver)
            self.assertNotIn("e171_e170_exporter", body, driver)
        self.assertEqual((shared_steps.EXECUTOR_DIRECTORY_RELATIVE, shared_steps.EXECUTOR_SCRIPT_NAME), (frozen_o3.FROZEN_DIRECTORY_RELATIVE, frozen_o3.EXECUTOR_SCRIPT))
        self.assertEqual(shared_steps.IMAGE_JUDGE_SCRIPT, row_driver.IMAGE_JUDGE_SCRIPT)
        device = driver_body(replay, "e171_device")
        self.assertIn(f"-p {row_driver.DEVICE_PACKAGE} ", device)
        self.assertIn(f"--bin {row_driver.DEVICE_BINARY_NAME} ", device)
        self.assertIn(f"--bin {shared_steps.EXPORTER_BINARY_NAME} ", driver_body(replay, "e171_e170_exporter"))

    def test_rows_done_names_the_missing_rows(self) -> None:
        selection = os.path.join(self.repository, "selected.tsv")
        rows_root = os.path.join(self.base, "rows")
        for number in (1, 2, 3, 4, 5, 7, 8, 9):
            write(os.path.join(rows_root, f"row-{number}", "row-result.tsv"), "status\tok\n")
        code, stderr = self.main("rows-done", "--selected-rows", selection, "--rows-root", rows_root)
        self.assertEqual(code, shared_steps.EXIT_MISSING)
        self.assertIn("missing=1 of=9 rows=10", stderr)
        write(os.path.join(rows_root, "row-10", "row-result.tsv"), "status\tcompile_failed\n")
        self.assertEqual(self.main("rows-done", "--selected-rows", selection, "--rows-root", rows_root)[0], 0)

    def test_a_break_switch_on_the_command_line_is_refused(self) -> None:
        err = io.StringIO()
        with mock.patch.dict(os.environ, {shared_steps.BREAK_VARIABLE: "reused-item-remade"}), contextlib.redirect_stderr(err):
            code = shared_steps.main(["fetch", shared_steps.PREREQUISITES, "--root", self.root, "--repo-root", self.repository, "--device-features", "none"])
        self.assertEqual(code, shared_steps.EXIT_USAGE)
        self.assertIn("只在自证里认", err.getvalue())


class BreakSwitchTests(unittest.TestCase):
    """弄坏开关由环境给（E171_SHARED_STEPS_BREAK=<项> python3 -B -m unittest test_shared_steps）：设了它，上面点名的那一格该红。这一格只核开关名都在文件头里。"""

    def test_every_break_switch_is_listed_in_the_header(self) -> None:
        names = ("reused-item-remade", "prerequisites-unchecked", "card-held-through-the-build", "take-ignores-the-library", "e170-dump-not-renamed",
                 "step-drifts-from-replay", "request-without-features", "failed-item-recorded-green", "item-finish-ignored",
                 device_card_lease.CARD_HELD_THROUGH_THE_RECORD_STAGE, device_card_lease.CARDS_LEFT_IN_THE_RECORD_ENVIRONMENT,
                 device_card_lease.CARD_RETURNED_BEFORE_THE_JUDGE_STAGE, device_card_lease.JUDGE_STAGE_AFTER_A_FAILED_RECORD,
                 device_card_lease.CARD_KEPT_AFTER_A_FAILED_JUDGE)
        for name in names:
            self.assertIn(f"  {shared_steps.BREAK_VARIABLE}={name}  ", shared_steps.__doc__)


if __name__ == "__main__":
    unittest.main()
