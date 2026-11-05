"""E171 比对器与驱动那一侧的变异驱动（登记第九节 X1–X3、X7–X12；表 scripts/mutation/tables/e171_three_oracle_collusion.tsv）。

scripts/mutation/mutate.sh 只跑 cargo，这一侧照 E170 的先例自己写（登记第九节写的是「经 mutate.sh 跑」，这里改成本驱动：它起不了 Python 的单测）：
派活之前逐条核锚点——原文（`\\n` 还原成换行）在本目录的源文件（本文件与 `test_*.py` 除外）里合起来恰好命中一次，不是就退 3、一条不跑；
每条在草稿目录的一份副本上替换、先 `py_compile`（编不过记无效）、再跑本目录的全部单测，单测红记抓到、全绿记没红；每条在新副本上做；
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
REPOSITORY_ROOT = os.path.abspath(os.path.join(DIRECTORY, "..", "..", ".."))
PATH_FROM_REPOSITORY_ROOT = tuple(os.path.relpath(DIRECTORY, REPOSITORY_ROOT).split(os.sep))   # scripts、experiment-apparatus、本目录名
# 第 1 次重跑（表 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv）：比对器的单测，与逐行限时、进程挂钟的单测（修订 2 第 2.5、2.6 节）
TEST_MODULES = ("test_segment_one", "test_device_timing")
# 第二段比对器的表（scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv）给 `--unit-test-module test_segment_two`；
# 不给时照旧跑上面两个（第一段那张表的复跑产物逐字节不变）
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
                  if name.endswith(".py") and name != THIS_FILE and not name.startswith("test_"))


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


def copy_root(scratch: str, label: str) -> str:
    """一份副本的根：<草稿目录>/<标签>，删副本删它。"""
    return os.path.join(scratch, label)


def fresh_copy(scratch: str, label: str) -> str:
    """在 <草稿目录>/<标签>/ 下搭一份仓形的副本，交回副本里的研究目录。
    研究目录里的模块按自己所在位置往上三层取仓根（.claude/hooks/lib_work_root.py、.claude/heavy-thread-default、搬迁登记表这类）；
    副本的根照仓根的样子：从仓根到研究目录这一条路径上，每一层除了往下那一项与 .git，别的条目都链回真仓，研究目录本身是拷贝。
    副本里同一条相对路径就落到真仓那一份，调用方不用另搭镜像，--scratch 指哪里都一样（第 2 次重跑登记修订 5 第 5.2 条）。"""
    root = copy_root(scratch, label)
    shutil.rmtree(root, ignore_errors=True)
    level_in_copy, level_in_repository = root, REPOSITORY_ROOT
    for part in PATH_FROM_REPOSITORY_ROOT:
        os.makedirs(level_in_copy)
        for entry in sorted(os.listdir(level_in_repository)):
            if entry not in (part, ".git"):
                os.symlink(os.path.join(level_in_repository, entry), os.path.join(level_in_copy, entry))
        level_in_copy, level_in_repository = os.path.join(level_in_copy, part), os.path.join(level_in_repository, part)
    shutil.copytree(DIRECTORY, level_in_copy, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    return level_in_copy


def run_unit_tests(directory: str, test_modules: tuple[str, ...] = TEST_MODULES) -> tuple[str, int]:
    """本目录全部单测；交回 (结局 抓到 / 没红 / 超时, 退出码)。"""
    # 单测在草稿目录的仓形副本里跑，副本的根不带 .git（git 命令认不出它是仓）；冻结目录那一条单测读真的仓，所以把真的仓根交给它
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", E171_REPOSITORY_ROOT=REPOSITORY_ROOT)
    try:
        completed = subprocess.run([sys.executable, "-m", "unittest", *test_modules], cwd=directory, env=environment,
                                   capture_output=True, text=True, timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return TIMED_OUT, -1
    return (SURVIVED if completed.returncode == 0 else CAUGHT), completed.returncode


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 比对器与驱动那一侧的变异驱动")
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
        try:
            py_compile.compile(target, doraise=True, cfile=os.path.join(directory, "compiled-check.pyc"))
            outcome, exit_code = run_unit_tests(directory, test_modules)
        except py_compile.PyCompileError:
            outcome, exit_code = INVALID, -1
        tally[outcome] += 1
        lines.result(f"name=mutation number={number} outcome={outcome} mutation={field(name)} unit_test_exit={exit_code}")
        shutil.rmtree(copy_root(arguments.scratch, f"mutation-{number}"), ignore_errors=True)
    shutil.rmtree(copy_root(arguments.scratch, "baseline-before"), ignore_errors=True)
    restored = fresh_copy(arguments.scratch, "baseline-after")
    restored_outcome, _ = run_unit_tests(restored, test_modules)
    shutil.rmtree(copy_root(arguments.scratch, "baseline-after"), ignore_errors=True)
    unchanged = directory_digest(DIRECTORY) == digest_before
    lines.result(f"name=verdict mutations={len(rows)} caught={tally[CAUGHT]} invalid={tally[INVALID]} survived={tally[SURVIVED]} "
                 f"timed_out={tally[TIMED_OUT]} restored_and_baseline_green={str(restored_outcome == SURVIVED and unchanged).lower()}")
    lines.finish()
    return 0 if tally[SURVIVED] == 0 and tally[TIMED_OUT] == 0 and restored_outcome == SURVIVED and unchanged else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
