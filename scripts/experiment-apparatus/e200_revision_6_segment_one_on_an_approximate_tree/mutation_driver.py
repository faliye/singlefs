"""E200（修订6近似树重跑） 研究一侧的变异驱动（登记第九节；表 scripts/mutation/tables/e200_revision_6_segment_one_on_an_approximate_tree.tsv）。

出处：照抄 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/mutation_driver.py（提交 59ce5a865 那一版，sha256
3399e5ddaeed24b24f96fb43535f08cece70dba3683e2e6fea2364af2f2a22f1），改了五处：这一段文件头、TEST_MODULES、单测环境不再带 E171 的仓根变量、argparse 的说明、
源文件连同本目录的 `.sh`（编译包装 compile_in_the_approximate_tree.sh 的拷贝判法也要能改坏，单测 CompileWrapper 整条起它）。
scripts/mutation/mutate.sh 只跑 cargo，起不了 Python 的单测（登记第五节 5.4「经 E171 研究一侧的 mutation_driver.py 那种做法跑」）。
派活之前逐条核锚点——原文（`\\n` 还原成换行）在本目录的源文件（`.py` 与 `.sh`，本文件与 `test_*.py` 除外）里合起来恰好命中一次，不是就退 3、一条不跑；
每条在草稿目录的一份副本上替换、先查语法（`.py` 用 `py_compile`、`.sh` 用 `bash -n`，过不了记无效）、再跑本目录的单测，单测红记抓到、全绿记没红；每条在新副本上做；
收尾在一份新副本上重跑基线，核「已还原，基线仍全绿」，并核本目录在跑前跑后逐字节相同。结局符号照 .claude/rules/mutation-sampling.md：
✅ 抓到、⏭ 无效、❌ 没红、⏱ 超时。结果行不带耗时，两次跑逐字节相同。

跑法（在本目录，整条命令交 `task-server/task submit --`）：
  python3 mutation_driver.py --table <变异表> --scratch <草稿目录> --output <新文件>
"""

from __future__ import annotations

import argparse
import hashlib
import os
import py_compile
import shutil
import subprocess
import sys

from result_lines import ResultLines, field

DIRECTORY = os.path.dirname(os.path.abspath(__file__))
THIS_FILE = os.path.basename(__file__)
# 第一段（造树与 Q1 的判法，登记第九节 M40–M47）的单测
TEST_MODULES = ("test_build_approximate_tree",)
TIMEOUT_SECONDS = 300
CAUGHT, INVALID, SURVIVED, TIMED_OUT = "✅", "⏭", "❌", "⏱"


def read_table(path: str) -> list[tuple[str, str, str]]:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            stripped = line.rstrip("\n")
            if not stripped or stripped.startswith("#"):
                continue
            name, original, replacement = stripped.split("\t")
            rows.append((name, original.replace("\\n", "\n"), replacement.replace("\\n", "\n")))
    return rows


def source_files(directory: str) -> list[str]:
    return sorted(name for name in os.listdir(directory)
                  if name.endswith((".py", ".sh")) and name != THIS_FILE and not name.startswith("test_"))


def syntax_holds(path: str, directory: str) -> bool:
    """改坏之后的那一份还过得了语法：.py 用 py_compile，.sh 用 bash -n。"""
    if path.endswith(".sh"):
        return subprocess.run(["bash", "-n", path], capture_output=True).returncode == 0
    try:
        py_compile.compile(path, doraise=True, cfile=os.path.join(directory, "compiled-check.pyc"))
    except py_compile.PyCompileError:
        return False
    return True


def directory_digest(directory: str) -> str:
    hasher = hashlib.sha256()
    for name in sorted(os.listdir(directory)):
        path = os.path.join(directory, name)
        if os.path.isfile(path):
            with open(path, "rb") as handle:
                hasher.update(name.encode("utf-8") + handle.read())
    return hasher.hexdigest()


def anchor_hits(directory: str, original: str) -> list[tuple[str, int]]:
    hits = []
    for name in source_files(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            count = handle.read().count(original)
        if count:
            hits.append((name, count))
    return hits


def fresh_copy(scratch: str, label: str) -> str:
    destination = os.path.join(scratch, label)
    shutil.rmtree(destination, ignore_errors=True)
    shutil.copytree(DIRECTORY, destination, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    return destination


def run_unit_tests(directory: str, test_modules: tuple[str, ...] = TEST_MODULES) -> tuple[str, int]:
    """本目录全部单测；交回 (结局 抓到 / 没红 / 超时, 退出码)。"""
    # 单测只用自己建的临时 git 仓，不读真的仓
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    try:
        completed = subprocess.run([sys.executable, "-m", "unittest", *test_modules], cwd=directory, env=environment,
                                   capture_output=True, text=True, timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return TIMED_OUT, -1
    return (SURVIVED if completed.returncode == 0 else CAUGHT), completed.returncode


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E200 研究一侧的变异驱动")
    parser.add_argument("--table", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--unit-test-module", action="append", help="要跑的单测模块（可给几次）；不给跑 TEST_MODULES")
    arguments = parser.parse_args(argument_list)
    test_modules = tuple(arguments.unit_test_module) if arguments.unit_test_module else TEST_MODULES
    rows = read_table(arguments.table)
    digest_before = directory_digest(DIRECTORY)
    lines = ResultLines(arguments.output)
    lines.result(f"name=config table_rows={len(rows)} unit_test_modules={','.join(test_modules)} timeout_seconds={TIMEOUT_SECONDS}")
    anchor_problems = 0
    for name, original, _ in rows:
        hits = anchor_hits(DIRECTORY, original)
        total = sum(count for _, count in hits)
        lines.result(f"name=anchor mutation={field(name)} hits={total} files={','.join(file for file, _ in hits) or 'none'}")
        anchor_problems += total != 1
    if anchor_problems:
        lines.result(f"name=verdict anchors_not_exactly_once={anchor_problems} mutations_run=0")
        lines.finish()
        return 3
    baseline_directory = fresh_copy(arguments.scratch, "baseline-before")
    baseline_outcome, _ = run_unit_tests(baseline_directory, test_modules)
    lines.result(f"name=baseline unit_tests={'green' if baseline_outcome == SURVIVED else 'red'}")
    if baseline_outcome != SURVIVED:
        lines.finish()
        return 2
    tally = {CAUGHT: 0, INVALID: 0, SURVIVED: 0, TIMED_OUT: 0}
    for number, (name, original, replacement) in enumerate(rows, start=1):
        directory = fresh_copy(arguments.scratch, f"mutation-{number}")
        target = os.path.join(directory, anchor_hits(DIRECTORY, original)[0][0])
        with open(target, encoding="utf-8") as handle:
            text = handle.read()
        with open(target, "w", encoding="utf-8") as handle:
            handle.write(text.replace(original, replacement, 1))
        if syntax_holds(target, directory):
            outcome, exit_code = run_unit_tests(directory, test_modules)
        else:
            outcome, exit_code = INVALID, -1
        tally[outcome] += 1
        lines.result(f"name=mutation number={number} outcome={outcome} mutation={field(name)} unit_test_exit={exit_code}")
        shutil.rmtree(directory, ignore_errors=True)
    shutil.rmtree(baseline_directory, ignore_errors=True)
    restored = fresh_copy(arguments.scratch, "baseline-after")
    restored_outcome, _ = run_unit_tests(restored, test_modules)
    shutil.rmtree(restored, ignore_errors=True)
    unchanged = directory_digest(DIRECTORY) == digest_before
    lines.result(f"name=verdict mutations={len(rows)} caught={tally[CAUGHT]} invalid={tally[INVALID]} survived={tally[SURVIVED]} "
                 f"timed_out={tally[TIMED_OUT]} restored_and_baseline_green={str(restored_outcome == SURVIVED and unchanged).lower()}")
    lines.finish()
    return 0 if tally[SURVIVED] == 0 and tally[TIMED_OUT] == 0 and restored_outcome == SURVIVED and unchanged else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
