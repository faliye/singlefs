"""E170 第 2 次重跑进复跑（`scripts/replay.sh` 的 E170 几行）用的结果行出口。文件名以 run_ 起头，不进冻结（登记 5.6）。

产物都走 replay.sh 的 E7RESULT 协议：每行 `E7RESULT name=…`，末行 `E7RESULT name=done emitted=N`，
N 连 done 行自己一起数（replay.sh 文件头闸 2 的口径）。两个子命令：

- unit-tests：在给的执行器目录里跑 test_executor_on_synthetic_pool，逐条记结局。不记耗时，也不记单测自己往
  stdout / stderr 打的话（那里有临时目录名）：结局按单测名记，打的话照 unittest 的 buffer 收走、不进产物。
- wrap：把执行器（executor_main.py）、坏镜像判法（comparator_bad_image_judge.py）、单测层变异驱动
  （mutation_driver_unit_level.py）的输出改写成结果行。删掉键以 `_ns` 结尾的字段（墙钟）与 `worker=`（进程号），
  值是绝对路径的换成末一段名字；别的字段一个不动、行序不动。输入必须以它自己的收尾行结束（`name=complete`，
  或 `name=done emitted=K` 且 K 等于它的行数），每一行都以 `name=` 起头，不然退 1、不出产物。

用法：
  python3 run_replay_result_lines.py unit-tests --executor-directory <目录> --arm <臂名> --scratch <单测临时目录的父目录> --output <新文件>
  python3 run_replay_result_lines.py wrap --input <原始输出> --output <新文件>
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import os
import re
import sys
import unittest

TEST_MODULE_NAME = "test_executor_on_synthetic_pool"
FILES_OUTSIDE_THE_TESTED_SET = ("comparator", "mutation_driver", "witness_reference", "run_")  # 同 executor_main.FILES_OUTSIDE_THE_FREEZE
TIMING_FIELD_KEY_SUFFIX = "_ns"
PROCESS_IDENTIFIER_FIELD_KEY = "worker"
DONE_LINE_PATTERN = re.compile(r"^name=done emitted=(\d+)$")


class OutcomeRecordingResult(unittest.TestResult):
    """按 unittest 报结局的次序记 (单测名, 结局, 异常类名)；buffer 打开，单测打的话不进产物。"""

    def __init__(self) -> None:
        super().__init__()
        self.buffer = True
        self.outcomes: list[tuple[str, str, str]] = []

    def addSuccess(self, test) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addSuccess(test)
        self.outcomes.append((test.id(), "ok", "-"))

    def addFailure(self, test, error_information) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addFailure(test, error_information)
        self.outcomes.append((test.id(), "fail", error_information[0].__name__))

    def addError(self, test, error_information) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addError(test, error_information)
        self.outcomes.append((test.id(), "error", error_information[0].__name__))

    def addSkip(self, test, reason) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addSkip(test, reason)
        self.outcomes.append((test.id(), "skipped", "-"))

    def addExpectedFailure(self, test, error_information) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addExpectedFailure(test, error_information)
        self.outcomes.append((test.id(), "expected_failure", error_information[0].__name__))

    def addUnexpectedSuccess(self, test) -> None:  # naming-lint:external unittest.TestResult 定的方法名
        super().addUnexpectedSuccess(test)
        self.outcomes.append((test.id(), "unexpected_success", "-"))


def tested_files(directory: str) -> list[str]:
    return sorted(name for name in os.listdir(directory)
                  if os.path.isfile(os.path.join(directory, name)) and name.endswith((".py", ".tsv"))
                  and not name.startswith(FILES_OUTSIDE_THE_TESTED_SET))


def unit_test_lines(executor_directory: str, arm: str, scratch: str) -> list[str]:
    directory = os.path.abspath(executor_directory)
    own_directory = os.path.dirname(os.path.abspath(__file__))
    os.makedirs(scratch, exist_ok=True)
    os.environ["E170_TEST_SCRATCH"] = os.path.abspath(scratch)  # 单测模块载入时读它（SCRATCH_ROOT）
    sys.dont_write_bytecode = True
    os.chdir(directory)
    # 本文件所在目录（v2-487）不许排在被测目录前面：判 v1-487 时要载入 v1-487 自己的模块
    sys.path[:] = [directory] + [entry for entry in sys.path if os.path.abspath(entry or os.curdir) not in (own_directory, directory)]
    test_module = importlib.import_module(TEST_MODULE_NAME)
    loaded_from = os.path.dirname(os.path.abspath(test_module.__file__))
    if loaded_from != directory:
        raise SystemExit(f"单测模块载入自 {loaded_from}，不是要测的 {directory}")
    for module_name in ("on_disk_layout", "executor_core", "journal_ring", "replay_prefix"):
        module = sys.modules.get(module_name)
        if module is not None and os.path.dirname(os.path.abspath(module.__file__)) != directory:
            raise SystemExit(f"{module_name} 载入自 {module.__file__}，不是要测的 {directory}")
    result = OutcomeRecordingResult()
    unittest.defaultTestLoader.loadTestsFromModule(test_module).run(result)
    lines = [f"name=unit_tests_config arm={arm} executor={os.path.basename(directory)} module={TEST_MODULE_NAME}"]
    for test_identifier, outcome, exception_name in result.outcomes:
        case_name, _, test_name = test_identifier.removeprefix(TEST_MODULE_NAME + ".").rpartition(".")
        lines.append(f"name=unit_test case={case_name or '-'} test={test_name} result={outcome} exception={exception_name}")
    counted = {outcome: sum(1 for _identifier, recorded, _name in result.outcomes if recorded == outcome)
               for outcome in ("ok", "fail", "error", "skipped", "expected_failure", "unexpected_success")}
    lines.append(f"name=unit_tests_summary arm={arm} ran={result.testsRun} recorded={len(result.outcomes)} "
                 + " ".join(f"{outcome}={count}" for outcome, count in counted.items())
                 + f" successful={str(result.wasSuccessful()).lower()}")
    for file_name in tested_files(directory):
        with open(os.path.join(directory, file_name), "rb") as handle:
            lines.append(f"name=tested_file file={file_name} sha256={hashlib.sha256(handle.read()).hexdigest()}")
    return lines


def rewritten_line(line: str) -> str:
    kept = []
    for token in line.split(" "):
        key, separator, value = token.partition("=")
        if separator and (key.endswith(TIMING_FIELD_KEY_SUFFIX) or key == PROCESS_IDENTIFIER_FIELD_KEY):
            continue
        if separator and value.startswith("/"):
            token = f"{key}={os.path.basename(value.rstrip('/'))}"
        kept.append(token)
    return " ".join(kept)


def wrapped_lines(input_path: str) -> list[str]:
    with open(input_path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    if not lines:
        raise SystemExit(f"{input_path} 是空的")
    marker = DONE_LINE_PATTERN.match(lines[-1])
    if not (lines[-1] == "name=complete" or (marker and int(marker.group(1)) == len(lines))):
        raise SystemExit(f"{input_path} 末行不是它自己的收尾行（name=complete，或 name=done emitted=<行数>）：{lines[-1][:200]}")
    body = lines[:-1]
    not_result_lines = [number for number, line in enumerate(body, start=1) if not line.startswith("name=")]
    if not_result_lines:
        raise SystemExit(f"{input_path} 有 {len(not_result_lines)} 行不以 name= 起头，第一处在第 {not_result_lines[0]} 行")
    return [rewritten_line(line) for line in body]


def write_result_lines(output_path: str, lines: list[str]) -> None:
    with open(output_path, "x", encoding="utf-8") as output:
        for line in lines:
            output.write(f"E7RESULT {line}\n")
        output.write(f"E7RESULT name=done emitted={len(lines) + 1}\n")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑进复跑用的结果行出口")
    commands = parser.add_subparsers(dest="command", required=True)
    unit_tests = commands.add_parser("unit-tests")
    unit_tests.add_argument("--executor-directory", required=True)
    unit_tests.add_argument("--arm", required=True)
    unit_tests.add_argument("--scratch", required=True)
    unit_tests.add_argument("--output", required=True)
    wrap = commands.add_parser("wrap")
    wrap.add_argument("--input", required=True)
    wrap.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    output_path = os.path.abspath(arguments.output)
    if os.path.exists(output_path):
        print(f"产物 {output_path} 已存在，不覆盖", file=sys.stderr)
        return 2
    if arguments.command == "unit-tests":
        lines = unit_test_lines(arguments.executor_directory, arguments.arm, arguments.scratch)
    else:
        lines = wrapped_lines(arguments.input)
    write_result_lines(output_path, lines)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
