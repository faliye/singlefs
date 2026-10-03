"""E170（O3规约执行器第一版） 第三段：执行器与比对器的变异驱动（登记第九节 X1–X11、XC1；表 research/mutations/e170_o3_spec_executor.tsv）。

research/scripts/mutate.sh 只跑 cargo，这一侧照它的形态写（登记第九节）：派活之前逐条核锚点——原文（\\n 还原成换行）在本目录全部 .py
（本文件除外）里合起来恰好命中一次，不是就退 3、一条不跑；每条在草稿目录的一份副本上替换、编译（compile）、跑单测与检查
（comparator_mutation_checks.py）、删副本；收尾在一份新副本上重跑基线，核「已还原，基线仍全绿」，并核本目录在跑前跑后逐字节相同。
结局照 .claude/rules/mutation-sampling.md 的符号：✅ 抓到（单测红，或基线上为真的检查转假）、⏭ 无效（替换之后编不过）、❌ 没红、
💥 检查进程没跑完（单测没红、检查没打出 __complete__）、⏱ 超时。结果行不带耗时，两次跑逐字节相同。

跑法（经 run-with-memory-cap.sh 与 capped.sh）：
  python3 mutation_driver.py --table <变异表> --dumps <导出目录> --frozen-verdicts <第二段产物> --scratch <草稿目录> --output <新文件>
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys

from comparator_result_lines import ResultLines, field

EXECUTOR_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
THIS_FILE = os.path.basename(__file__)
UNIT_TEST_MODULE = "test_executor_on_synthetic_pool"
CHECK_SCRIPT = "comparator_mutation_checks.py"
TIMEOUT_SECONDS = 600


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
    return sorted(name for name in os.listdir(directory) if name.endswith(".py") and name != THIS_FILE)


def directory_digest(directory: str) -> str:
    hasher = hashlib.sha256()
    for name in sorted(os.listdir(directory)):
        path = os.path.join(directory, name)
        if os.path.isfile(path):
            hasher.update(name.encode() + b"\0")
            with open(path, "rb") as handle:
                hasher.update(handle.read())
    return hasher.hexdigest()


def anchor_hits(directory: str, original: str) -> dict[str, int]:
    hits = {}
    for name in source_files(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            count = handle.read().count(original)
        if count:
            hits[name] = count
    return hits


def fresh_copy(scratch: str, label: str) -> str:
    destination = os.path.join(scratch, label)
    if os.path.exists(destination):
        shutil.rmtree(destination)
    shutil.copytree(EXECUTOR_DIRECTORY, destination, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    return destination


def compiles(directory: str) -> str | None:
    """每个 .py 都编得过就交 None，否则交第一处编不过的文件与原因。"""
    for name in source_files(directory):
        path = os.path.join(directory, name)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        try:
            compile(text, path, "exec")
        except SyntaxError as error:
            return f"{name}:{error.lineno}:{error.msg}"
    return None


def run_in(directory: str, command: list[str]) -> tuple[int | None, str]:
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    try:
        completed = subprocess.run(command, cwd=directory, env=environment, capture_output=True, text=True,
                                   timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return None, ""
    return completed.returncode, completed.stdout + completed.stderr


def unit_test_summary(output: str) -> str:
    ran = [line for line in output.splitlines() if line.startswith("Ran ")]
    outcome = [line for line in output.splitlines() if line.startswith(("OK", "FAILED"))]
    return field((ran[-1].split(" in ")[0] if ran else "Ran_unknown") + ":" + (outcome[-1] if outcome else "no_outcome"))


def run_checks(directory: str, dumps: str, frozen_verdicts: str) -> tuple[int | None, dict[str, bool]]:
    exit_code, output = run_in(directory, [sys.executable, CHECK_SCRIPT, "--dumps", dumps, "--frozen-verdicts", frozen_verdicts])
    checks = {}
    for line in output.splitlines():
        if line.startswith("CHECK "):
            fields = dict(token.split("=", 1) for token in line.split()[1:])
            checks[fields["name"]] = fields["holds"] == "true"
    return exit_code, checks


def evaluate(directory: str, dumps: str, frozen_verdicts: str, baseline_green: set[str]) -> dict:
    compile_error = compiles(directory)
    if compile_error is not None:
        return {"outcome": "⏭", "unit": "not_run", "red": [], "detail": field(compile_error)}
    unit_exit, unit_output = run_in(directory, [sys.executable, "-m", "unittest", "-q", UNIT_TEST_MODULE])
    if unit_exit is None:
        return {"outcome": "⏱", "unit": "timeout", "red": [], "detail": "unit_tests_timeout"}
    check_exit, checks = run_checks(directory, dumps, frozen_verdicts)
    red = sorted(name for name in baseline_green if not checks.get(name, False))
    complete = checks.get("__complete__", False)
    if unit_exit != 0:
        outcome = "✅"
    elif check_exit is None:
        outcome = "⏱"
    elif not complete:
        outcome = "💥"
    elif red:
        outcome = "✅"
    else:
        outcome = "❌"
    return {"outcome": outcome, "unit": unit_test_summary(unit_output), "red": red, "checks": checks,
            "detail": f"unit_exit={unit_exit}:check_exit={check_exit}:checks_complete={str(complete).lower()}"}


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第三段：执行器与比对器的变异驱动")
    parser.add_argument("--table", required=True)
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--frozen-verdicts", required=True)
    parser.add_argument("--scratch", required=True, help="草稿目录（每条变异一份执行器目录的副本，跑完删）")
    parser.add_argument("--output", required=True, help="产物路径；已存在就拒绝")
    arguments = parser.parse_args(argument_list)
    lines = ResultLines(arguments.output)
    before = directory_digest(EXECUTOR_DIRECTORY)
    rows = read_table(arguments.table)
    lines.result(f"name=config segment=s3 mode=mutations rows={len(rows)} unit_test_module={UNIT_TEST_MODULE} "
                 f"check_script={CHECK_SCRIPT} timeout_seconds={TIMEOUT_SECONDS}")
    anchor_failures = 0
    for name, original, _ in rows:
        hits = anchor_hits(EXECUTOR_DIRECTORY, original)
        anchor_failures += int(sum(hits.values()) != 1)
        lines.result(f"name=anchor id={name} hits={sum(hits.values())} files={','.join(sorted(hits)) or '-'}")
    if anchor_failures:
        lines.result(f"name=verdict anchor_failures={anchor_failures} mutations_run=0")
        lines.finish()
        return 3
    baseline_directory = fresh_copy(arguments.scratch, "baseline")
    baseline = evaluate(baseline_directory, arguments.dumps, arguments.frozen_verdicts, set())
    shutil.rmtree(baseline_directory)
    baseline_checks = baseline.get("checks", {})
    baseline_green = {name for name, holds in baseline_checks.items() if holds and name != "__complete__"}
    baseline_false = sorted(name for name, holds in baseline_checks.items() if not holds)
    lines.result(f"name=baseline unit={baseline['unit']} checks={len(baseline_checks)} true={len(baseline_green)} "
                 f"false={len(baseline_false)} false_checks={','.join(baseline_false) or '-'} detail={baseline['detail']}")
    if baseline["outcome"] != "❌":  # 基线上「没红」才是对的：单测全过、检查跑完
        lines.result(f"name=verdict baseline_not_green=true baseline_outcome={baseline['outcome']} mutations_run=0")
        lines.finish()
        return 4
    tallies = {"✅": 0, "⏭": 0, "❌": 0, "💥": 0, "⏱": 0}
    for name, original, replacement in rows:
        directory = fresh_copy(arguments.scratch, "mutation")
        (file_name, _), = anchor_hits(directory, original).items()
        path = os.path.join(directory, file_name)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text.replace(original, replacement, 1))
        result = evaluate(directory, arguments.dumps, arguments.frozen_verdicts, baseline_green)
        shutil.rmtree(directory)
        tallies[result["outcome"]] += 1
        lines.result(f"name=mutation id={name} file={file_name} outcome={result['outcome']} unit={result['unit']} "
                     f"red_checks={len(result['red'])} red={','.join(result['red']) or '-'} detail={result['detail']}")
    restored_directory = fresh_copy(arguments.scratch, "restored")
    restored = evaluate(restored_directory, arguments.dumps, arguments.frozen_verdicts, baseline_green)
    shutil.rmtree(restored_directory)
    restored_green = restored["outcome"] == "❌" and restored.get("checks") == baseline_checks
    after = directory_digest(EXECUTOR_DIRECTORY)
    lines.result(f"name=restored baseline_still_green={str(restored_green).lower()} unit={restored['unit']} "
                 f"executor_directory_unchanged={str(before == after).lower()}")
    lines.result("name=verdict " + " ".join(f"{symbol}={count}" for symbol, count in tallies.items())
                 + f" rows={len(rows)} anchor_failures=0 restored_baseline_still_green={str(restored_green).lower()}")
    lines.finish()
    return 0 if restored_green and before == after else 5


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
