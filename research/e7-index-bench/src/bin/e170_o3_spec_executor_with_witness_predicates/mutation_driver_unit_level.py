"""E170 第 2 次重跑第一段甲的单测层变异驱动（不进冻结：登记 5.6 的 mutation_driver* 前缀）。

逐条：先核每条变异的原文在执行器目录（*.py 与 *.tsv，本驱动除外）里合起来恰好命中一次（不是 1 次的整张表不跑，退 3）；
把执行器目录拷进草稿目录，在副本里替换（表里的 \\n 还原成换行），先 py_compile 改过的那一份（编不过记无效），
再在副本里跑 python3 -m unittest test_executor_on_synthetic_pool，结局照 .claude/rules/mutation-sampling.md 记：
✅ 抓到（单测有红：失败、出错，含载入时断言红）；⏭ 无效（替换之后编不过）；❌ 没红；⏱ 超时。原目录一个字节不动，
收尾核原目录逐文件 sha256 与开跑前相同、再在新副本上跑一遍基线，都过才打「已还原，基线仍全绿」。

用法：python3 mutation_driver_unit_level.py --executor-directory <目录> --table <变异表> --scratch <草稿目录> --output <新文件>
"""

from __future__ import annotations

import argparse
import hashlib
import os
import py_compile
import re
import shutil
import subprocess
import sys
import time

TEST_MODULE = "test_executor_on_synthetic_pool"
PER_RUN_TIMEOUT_SECONDS = 180


def source_files(directory: str) -> list[str]:
    return sorted(name for name in os.listdir(directory)
                  if os.path.isfile(os.path.join(directory, name)) and name.endswith((".py", ".tsv"))
                  and not name.startswith("mutation_driver"))


def digests_of(directory: str) -> dict[str, str]:
    result = {}
    for name in source_files(directory):
        with open(os.path.join(directory, name), "rb") as handle:
            result[name] = hashlib.sha256(handle.read()).hexdigest()
    return result


def read_table(path: str) -> list[tuple[str, str, str]]:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle.read().split("\n"):
            if not line.strip() or line.startswith("#"):
                continue
            name, anchor, replacement = line.split("\t")
            rows.append((name, anchor.replace("\\n", "\n"), replacement.replace("\\n", "\n")))
    return rows


def anchor_hits(directory: str, anchor: str) -> list[tuple[str, int]]:
    hits = []
    for name in source_files(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            count = handle.read().count(anchor)
        if count:
            hits.append((name, count))
    return hits


def fresh_copy(source_directory: str, scratch: str, label: str) -> str:
    copy = os.path.join(scratch, label)
    if os.path.exists(copy):
        shutil.rmtree(copy)
    os.makedirs(copy)
    for name in source_files(source_directory):
        shutil.copy2(os.path.join(source_directory, name), os.path.join(copy, name))
    return copy


def run_unit_tests(directory: str, scratch: str) -> tuple[str, int | None, str]:
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", E170_TEST_SCRATCH=scratch)
    try:
        completed = subprocess.run([sys.executable, "-m", "unittest", TEST_MODULE], cwd=directory, env=environment,
                                   capture_output=True, text=True, timeout=PER_RUN_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return "timeout", None, ""
    return "finished", completed.returncode, completed.stderr + completed.stdout


def red_test_names(output: str) -> list[str]:
    return sorted(set(re.findall(r"^(?:FAIL|ERROR): (\S+)", output, flags=re.MULTILINE)))


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executor-directory", required=True)
    parser.add_argument("--table", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    if os.path.exists(arguments.output):
        print(f"产物 {arguments.output} 已存在，不覆盖", file=sys.stderr)
        return 2
    directory = os.path.abspath(arguments.executor_directory)
    rows = read_table(arguments.table)
    os.makedirs(arguments.scratch, exist_ok=True)
    with open(arguments.output, "x", encoding="utf-8", buffering=1) as output:
        def emit(line: str) -> None:
            output.write(line + "\n")
            print(line)

        emit(f"name=mutation_config executor_directory={directory} table={arguments.table} rows={len(rows)} "
             f"python={sys.version.split()[0]} per_run_timeout_seconds={PER_RUN_TIMEOUT_SECONDS}")
        bad_anchors = [(name, anchor_hits(directory, anchor)) for name, anchor, _replacement in rows]
        bad_anchors = [(name, hits) for name, hits in bad_anchors if sum(count for _file, count in hits) != 1]
        for name, hits in bad_anchors:
            emit(f"name=anchor_problem mutation={name} hits={hits}")
        if bad_anchors:
            emit("name=mutation_aborted reason=anchor_not_hit_exactly_once")
            return 3
        before = digests_of(directory)
        baseline_state, baseline_code, baseline_output = run_unit_tests(fresh_copy(directory, arguments.scratch, "baseline"), arguments.scratch)
        baseline_ran = re.findall(r"^Ran (\d+) tests?", baseline_output, flags=re.MULTILINE)
        emit(f"name=baseline state={baseline_state} exit={baseline_code} ran={baseline_ran[-1] if baseline_ran else '-'}")
        if baseline_state != "finished" or baseline_code != 0:
            emit("name=mutation_aborted reason=baseline_is_red")
            return 4
        counts = {"✅": 0, "⏭": 0, "❌": 0, "⏱": 0}
        for name, anchor, replacement in rows:
            started = time.perf_counter_ns()
            copy = fresh_copy(directory, arguments.scratch, "mutated")
            file_name = anchor_hits(copy, anchor)[0][0]
            path = os.path.join(copy, file_name)
            with open(path, encoding="utf-8") as handle:
                text = handle.read()
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text.replace(anchor, replacement, 1))
            if file_name.endswith(".py"):
                try:
                    py_compile.compile(path, cfile=os.path.join(arguments.scratch, "compile-check.pyc"), doraise=True)
                except py_compile.PyCompileError as error:
                    counts["⏭"] += 1
                    emit(f"⏭ [{name}] 无效：{file_name} 替换之后编不过（{str(error).splitlines()[-1][:160]}）")
                    continue
            state, code, test_output = run_unit_tests(copy, arguments.scratch)
            elapsed_seconds = (time.perf_counter_ns() - started) / 1e9
            if state == "timeout":
                counts["⏱"] += 1
                emit(f"⏱ [{name}] 超时（{PER_RUN_TIMEOUT_SECONDS} 秒）：{file_name}")
            elif code == 0:
                counts["❌"] += 1
                ran = re.findall(r"^Ran (\d+) tests?", test_output, flags=re.MULTILINE)
                emit(f"❌ [{name}] 没红：{file_name}，单测 {ran[-1] if ran else '?'} 条全过（{elapsed_seconds:.1f} 秒）")
            else:
                counts["✅"] += 1
                red_names = red_test_names(test_output)
                # 一个单测名都没有：载入时就红了（例：on_disk_layout 的载入断言），记下输出里最后一行带 Error 的，证明不是别的原因退出
                load_time_reason = "" if red_names else "，载入时红：" + next(
                    (line.strip()[:200] for line in reversed(test_output.splitlines()) if "Error" in line), "输出里没有带 Error 的行")
                emit(f"✅ [{name}] 抓到：{file_name}，红的单测 {red_names}{load_time_reason}（{elapsed_seconds:.1f} 秒）")
        after = digests_of(directory)
        final_state, final_code, _final_output = run_unit_tests(fresh_copy(directory, arguments.scratch, "baseline"), arguments.scratch)
        if after == before and final_state == "finished" and final_code == 0:
            emit("已还原，基线仍全绿")
        else:
            emit(f"name=restore_problem source_unchanged={after == before} final_baseline_state={final_state} exit={final_code}")
        emit(f"计数：抓到 {counts['✅']}、无效 {counts['⏭']}、没红 {counts['❌']}、超时 {counts['⏱']}，合计 {sum(counts.values())} / 表 {len(rows)} 条")
        emit("name=complete")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
