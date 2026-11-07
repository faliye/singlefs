#!/usr/bin/env python3
# admission: always 每一趟照此刻的输入与这份判法现算每一项的键、现查判定库（scripts/experiment-apparatus/long_run_item_verdicts.py）；键相同、库里判绿的复用
# run-condition: command python3
"""E170 第 2 次重跑进复跑（`scripts/experiment-apparatus/replay.sh` 的 E170 几行）用的结果行出口。文件名以 run_ 起头，不进冻结（登记 5.6）。

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
  python3 run_replay_result_lines.py wrap --input <原始输出> --output <新文件> [--input <…> --output <…>]…   几对照给的次序逐对改写
  python3 run_replay_result_lines.py --selftest

按项判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：unit-tests 一趟一条、wrap 每一对一条，经 scripts/experiment-apparatus/long_run_item_verdicts.py
  的 judge_items 存进 prune-server 的 apparatus-items 族、种类 run_replay_result_lines。键 = 判法的单元闭包（unit-tests 是 unit_test_lines，连同 EXECUTOR_DIRECTORIES 里各版执行器的模块；
  wrap 是 wrapped_lines；admission.py 的 compute_unit_key 算）、解释器版本、unit-tests 的臂名与选的执行器（目录名）连同那个目录里被测的每个文件的内容（@files），
  wrap 的原始输出的内容（@input）；路径不进键，产物逐字节可比（compare=exact）。键相同、库里判绿的那一项把库里的结果行写进 --output，不重跑单测、不重改写。
  判定库那几行打到标准错误；成功那句（标准错误末尾的 LONG_RUN_ITEMS 行）报「现判 N、复用 M、拒算 R」。退出码：0 成；1 输入不合（同前）、复用哨兵红或键在跑的中途变了；
  2 用法错、产物已在；75 prune-server 不可用。
自证（--selftest）：经 long_run_item_verdicts.py 的复用格跑那六格（reuse-three-questions、key-independent-of-directory、reuse-sentinel 与哨兵的三种红），
  一趟 wrap 两对，假驱动是 LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE（判法换成假的）；只在临时 prune-server 上跑（客户端 selftest_service_problem 判，不在退 2），起法：
    python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_replay_result_lines.py --selftest
  弄坏开关：复用那一侧的分支都在 long_run_item_verdicts.py 里，开关是它文件头的 LONG_RUN_ITEMS_BREAK（打开后这里的自证红在它点名的同名格）；
  这一份自己判红的分支（收尾行不对、有行不以 name= 起头）由 replay.sh 的 E170 几行复跑证（产物逐字节比）。
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import os
import re
import shutil
import sys
import tempfile
import unittest

TEST_MODULE_NAME = "test_executor_on_synthetic_pool"
FILES_OUTSIDE_THE_TESTED_SET = ("comparator", "mutation_driver", "witness_reference", "run_")  # 同 executor_main.FILES_OUTSIDE_THE_FREEZE
TIMING_FIELD_KEY_SUFFIX = "_ns"
# E170 各版执行器的目录（--executor-directory 只认这几份之一）：显式列出来，函数单元库照这张表把几版的模块都收进闭包
EXPERIMENTS_DIRECTORY = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXECUTOR_DIRECTORIES = (os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates_journal_header_487"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor"),
                        os.path.join(EXPERIMENTS_DIRECTORY, "e170_second_run_o3_spec_executor_journal_header_487"))


def registered_executor_directory(executor_directory: str) -> str:
    """--executor-directory 给的那一份 → EXECUTOR_DIRECTORIES 里同一个目录；不是其中之一就退出、说清给哪几份。"""
    for directory in EXECUTOR_DIRECTORIES:
        if os.path.realpath(directory) == os.path.realpath(executor_directory):
            return directory
    raise SystemExit(f"  ✗ --executor-directory {executor_directory} 不是 E170 登记的执行器目录之一\n"
                     "  → 怎么办：给 EXECUTOR_DIRECTORIES 里的一份（scripts/experiment-apparatus/ 底下那五个 e170_* 执行器目录）")
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
    chosen = registered_executor_directory(executor_directory)
    directory = os.path.abspath(chosen)
    own_directory = os.path.dirname(os.path.abspath(__file__))
    os.makedirs(scratch, exist_ok=True)
    os.environ["E170_TEST_SCRATCH"] = os.path.abspath(scratch)  # 单测模块载入时读它（SCRATCH_ROOT）
    sys.dont_write_bytecode = True
    os.chdir(directory)
    # 本文件所在目录（v2-487）不许排在被测目录前面：判 v1-487 时要载入 v1-487 自己的模块
    for entry in list(sys.path):
        if os.path.abspath(entry or os.curdir) in (own_directory, directory):
            sys.path.remove(entry)
    for candidate in EXECUTOR_DIRECTORIES:  # 插的是表里那一项本身：插进 sys.path 的目录就是表里写着的那几个之一
        if candidate == chosen:
            sys.path.insert(0, candidate)
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


def lines_bytes(lines: list[str]) -> bytes:
    return "".join(line + "\n" for line in lines).encode("utf-8")


# ── 按项判定（文件头「按项判定」）──

SCRIPT_KEY = "scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_replay_result_lines.py"
LIBRARY_KIND = "run_replay_result_lines"


def repository_root() -> str:
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))))


def item_verdicts_module():
    """scripts/experiment-apparatus/long_run_item_verdicts.py；这份脚本不在仓里（上一层没有它）时交 None。"""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), ".."))
    try:
        import long_run_item_verdicts
    except ModuleNotFoundError as error:
        if error.name != "long_run_item_verdicts":
            raise
        return None
    return long_run_item_verdicts


def main(argument_list: list[str]) -> int:
    if argument_list == ["--selftest"]:
        return selftest()
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑进复跑用的结果行出口")
    commands = parser.add_subparsers(dest="command", required=True)
    unit_tests = commands.add_parser("unit-tests")
    unit_tests.add_argument("--executor-directory", required=True)
    unit_tests.add_argument("--arm", required=True)
    unit_tests.add_argument("--scratch", required=True)
    unit_tests.add_argument("--output", required=True)
    wrap = commands.add_parser("wrap")
    wrap.add_argument("--input", required=True, action="append")
    wrap.add_argument("--output", required=True, action="append")
    arguments = parser.parse_args(argument_list)
    if arguments.command == "unit-tests":
        chosen = registered_executor_directory(arguments.executor_directory)
        name = f"unit-tests-{re.sub(r'[^A-Za-z0-9._-]', '-', arguments.arm)}"
        jobs = {name: (os.path.abspath(arguments.output), lambda: unit_test_lines(arguments.executor_directory, arguments.arm, arguments.scratch))}
        request_lines = ["\t".join([name, f"{SCRIPT_KEY}::unit_test_lines", f"line:arm={arguments.arm}", f"line:executor={os.path.basename(chosen)}",
                                    "line:compare=exact"])]
        request_lines += [f"@files\t{name}\t{os.path.join(os.path.abspath(chosen), file_name)}" for file_name in tested_files(os.path.abspath(chosen))]
    else:
        if len(arguments.input) != len(arguments.output):
            print(f"wrap 的 --input 有 {len(arguments.input)} 个、--output 有 {len(arguments.output)} 个，要一对一对给", file=sys.stderr)
            return 2
        jobs, request_lines = {}, []
        for index, (input_path, output_path) in enumerate(zip(arguments.input, arguments.output), start=1):
            name = f"wrap-{index}-{re.sub(r'[^A-Za-z0-9._-]', '-', os.path.basename(input_path))}"
            jobs[name] = (os.path.abspath(output_path), lambda input_path=input_path: wrapped_lines(input_path))
            request_lines += [f"{name}\t{SCRIPT_KEY}::wrapped_lines\tline:compare=exact", f"@input\t{name}\tsource\t{os.path.abspath(input_path)}"]
    for output_path, _compute in jobs.values():
        if os.path.exists(output_path):
            print(f"产物 {output_path} 已存在，不覆盖", file=sys.stderr)
            return 2
    verdicts = item_verdicts_module()
    finish_code = 0
    if verdicts is None:
        print(f"  · 拒算：{os.path.realpath(__file__)} 不在仓里（旁边没有 ../long_run_item_verdicts.py），照判、不复用、不入库", file=sys.stderr)
        results = {name: (0, lines_bytes(compute())) for name, (_output, compute) in jobs.items()}
    else:
        finish_code, results = verdicts.judge_items(SCRIPT_KEY, LIBRARY_KIND, repository_root(), "\n".join(request_lines) + "\n",
                                                    lambda name: (0, lines_bytes(jobs[name][1]()), True),
                                                    printer=lambda text: print(text, file=sys.stderr))
    for name, (output_path, _compute) in jobs.items():
        write_result_lines(output_path, results[name][1].decode("utf-8").split("\n")[:-1])
    return finish_code


def selftest() -> int:
    """文件头「自证」：样板树里是这份脚本的拷贝，long_run_item_verdicts.py 与 .claude 是指回真仓的符号链接；原始输出是两份假文件（判法换成假的，不读它们）。"""
    verdicts = item_verdicts_module()
    if verdicts is None:
        print(f"  ✗ run_replay_result_lines 自证没跑：{os.path.realpath(__file__)} 旁边找不到 ../long_run_item_verdicts.py")
        print("     → 怎么办：在仓里的那一份上起自证（起法见文件头「自证」）")
        return 2
    problem = verdicts.prune_server_client.selftest_service_problem()
    if problem:
        print(f"  ✗ run_replay_result_lines 自证没跑：{problem}")
        print(f"     → 怎么办：{verdicts.prune_server_client.SELFTEST_SERVICE_REMEDY}")
        return 2
    work = tempfile.mkdtemp(prefix="run-replay-result-lines-selftest-")
    try:
        template = os.path.join(work, "template")
        os.makedirs(os.path.join(template, os.path.dirname(SCRIPT_KEY)))
        shutil.copyfile(os.path.realpath(__file__), os.path.join(template, SCRIPT_KEY))
        os.symlink(os.path.join(repository_root(), verdicts.SCRIPT_KEY), os.path.join(template, verdicts.SCRIPT_KEY))
        os.symlink(os.path.join(repository_root(), ".claude"), os.path.join(template, ".claude"))
        for name in ("alpha.out", "beta.out"):
            with open(os.path.join(work, name), "w", encoding="utf-8") as handle:
                handle.write(f"name=selftest source={name}\nname=complete\n")
        # 产物每一趟一个新名字（--output 已在就拒）：经 bash 拿进程号拼
        command = ["bash", "-c", 'exec python3 "$1" wrap --input "$2/alpha.out" --output "$2/alpha-$$.result" --input "$2/beta.out" --output "$2/beta-$$.result"',
                   "run-replay-selftest", "{tree}/" + SCRIPT_KEY, work]
        red = verdicts.reuse_cells(SCRIPT_KEY, template, ["wrap-1-alpha.out", "wrap-2-beta.out"], os.path.join(work, "reuse"), 0, command, fake_judge=True)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    print(f"run_replay_result_lines.py 自证：6 格，红 {red} 格")
    return 1 if red else 0


if __name__ == "__main__":
    PREFLIGHT_DIRECTORY = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", "..", ".claude", "scripts")
    if os.path.isdir(PREFLIGHT_DIRECTORY):   # 拷到仓外的副本旁边没有 .claude：准入判不了，照「拒算」那一路原样跑
        sys.path.insert(0, PREFLIGHT_DIRECTORY)
        from project_preflight import preflight
        preflight(__file__)
    sys.exit(main(sys.argv[1:]))
