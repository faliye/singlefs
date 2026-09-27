#!/usr/bin/env python3
# admission: always 读的是这一次给的那份日志，上一次的结论不替这一次作保
# run-condition: none 只读一份文本日志、打印命令，不起 cargo，也不碰仓里的文件
"""从一份 cargo test 日志里取出失败的用例，按测试目标各打印一条只跑它们的命令；自己不起 cargo。

用法：
  rerun-failed-tests.py <cargo test 的日志> -p <包名>      # 每个有失败的测试目标一行命令，打到 stdout
  rerun-failed-tests.py --selftest
打印出来的命令交给调用方经 Bash 起（加内存包装、线程上限照派发提示），重型测试闸看得见它；脚本里起就绕过了闸。
认的段头：`Running tests/<目标>.rs (…)` → `--test <目标>`；`Running unittests src/lib.rs (…)` → `--lib`；
`Running unittests src/bin/<名>.rs (…)` → `--bin <名>`；`Doc-tests` 段不认（文档测试按名字重跑不了），有失败就判红要人看。
完整性闸：每一段收到的 `test <名> ... FAILED` 条数要等于那一段 `test result:` 行报的 failed 数，对不上判红、stdout 不出命令。
一个失败都没有：打印「没有失败的用例」到 stderr、退出 0、stdout 为空。
"""
import os
import re
import shlex
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SECTION_HEADER = re.compile(r"^\s*Running (?:unittests )?(\S+?\.rs) \(")
DOCTEST_HEADER = re.compile(r"^\s*Doc-tests (\S+)")
FAILED_LINE = re.compile(r"^test (\S+) \.\.\. FAILED\s*$")
RESULT_LINE = re.compile(r"^test result: \w+\. \d+ passed; (\d+) failed;")
BREAK_VARIABLE = "RERUN_FAILED_TESTS_BREAK"


def target_selector_of(source_path):
    """段头里的源文件 → cargo test 挑目标的参数；认不出交 None。"""
    if re.fullmatch(r"tests/[^/]+\.rs", source_path):
        return ["--test", os.path.basename(source_path)[:-len(".rs")]]
    if source_path == "src/lib.rs":
        return ["--lib"]
    if re.fullmatch(r"src/bin/[^/]+\.rs", source_path):
        return ["--bin", os.path.basename(source_path)[:-len(".rs")]]
    return None


def failed_tests_by_target(log_text):
    """(按段的 [(挑目标的参数, [失败的用例名])], [问题])。"""
    sections, problems = [], []
    current = None
    for line in log_text.split("\n"):
        header = SECTION_HEADER.match(line)
        doctest = DOCTEST_HEADER.match(line)
        if header or doctest:
            if os.environ.get(BREAK_VARIABLE) == "merge-sections" and current is not None:
                continue
            current = {"source": header.group(1) if header else f"doc-tests {doctest.group(1)}",
                       "selector": target_selector_of(header.group(1)) if header else None,
                       "failed": [], "reported": None}
            sections.append(current)
            continue
        if current is None:
            continue
        failed = FAILED_LINE.match(line)
        if failed:
            current["failed"].append(failed.group(1))
            continue
        result = RESULT_LINE.match(line)
        if result:
            current["reported"] = (current["reported"] or 0) + int(result.group(1))
    commands = []
    for section in sections:
        if section["reported"] is None:
            problems.append(f"{section['source']}：没有 test result 行（这一段没跑完，或者日志被截断）")
            continue
        if len(section["failed"]) != section["reported"]:
            problems.append(f"{section['source']}：收到 {len(section['failed'])} 条 FAILED，test result 报 {section['reported']} 条")
            continue
        if not section["failed"]:
            continue
        if section["selector"] is None:
            problems.append(f"{section['source']}：有 {len(section['failed'])} 条失败，这一段按名字重跑不了")
            continue
        commands.append((section["selector"], sorted(set(section["failed"]))))
    return commands, problems


def command_lines(commands, package):
    return [shlex.join(["cargo", "test", "-p", package, *selector, "--", "--exact", *names]) for selector, names in commands]


def run(log_path, package):
    with open(log_path, encoding="utf-8", errors="replace") as handle:
        commands, problems = failed_tests_by_target(handle.read())
    if problems:
        print(f"  ✗ {log_path} 里有 {len(problems)} 段取不准失败的用例，一条命令都不打：", file=sys.stderr)  # gate-lint:summary
        for problem in problems:
            print(f"      {problem}", file=sys.stderr)  # gate-lint:detail
        print("     → 怎么办：日志不全就整份重跑一次再取；Doc-tests 有失败的照日志里的名字手跑；条数对不上多半是日志里混进了别的输出，换一份干净的日志。", file=sys.stderr)
        return 1
    if not commands:
        print("  没有失败的用例：不用重跑", file=sys.stderr)
        return 0
    for line in command_lines(commands, package):
        print(line)
    print(f"  ✓ {sum(len(names) for _, names in commands)} 条失败的用例，分在 {len(commands)} 个测试目标里", file=sys.stderr)
    return 0


def selftest():
    sample = "\n".join([
        "     Running tests/alpha.rs (target/debug/deps/alpha-0123456789abcdef)",
        "test alpha_one ... ok",
        "test alpha_two ... FAILED",
        "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
        "     Running tests/beta.rs (target/debug/deps/beta-0123456789abcdef)",
        "test beta_one ... ok",
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        "     Running unittests src/lib.rs (target/debug/deps/sample-0123456789abcdef)",
        "test module::inner_case ... FAILED",
        "test module::other_case ... FAILED",
        "test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out",
    ])
    failures = []
    commands, problems = failed_tests_by_target(sample)
    lines = command_lines(commands, "sample")
    expected = ["cargo test -p sample --test alpha -- --exact alpha_two",
                "cargo test -p sample --lib -- --exact module::inner_case module::other_case"]
    if problems or lines != expected:
        failures.append(f"按段取失败用例：期望 {expected}，实际 {lines}，问题 {problems}")
    truncated = sample.rsplit("\n", 1)[0]
    _commands, problems = failed_tests_by_target(truncated)
    if not any("没有 test result" in problem for problem in problems):
        failures.append(f"最后一段缺 test result 行应当判出问题，实际 {problems}")
    miscounted = sample.replace("test alpha_two ... FAILED\n", "")
    _commands, problems = failed_tests_by_target(miscounted)
    if not any("收到 0 条 FAILED" in problem for problem in problems):
        failures.append(f"FAILED 行与 test result 对不上应当判出问题，实际 {problems}")
    with tempfile.TemporaryDirectory() as directory:
        empty_log = os.path.join(directory, "clean.log")
        with open(empty_log, "w", encoding="utf-8") as handle:
            handle.write("     Running tests/beta.rs (x)\ntest beta_one ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n")
        if run(empty_log, "sample") != 0:
            failures.append("没有失败的用例应当退 0")
    if failures:
        for failure in failures:
            print(f"  ✗ rerun-failed-tests 自检：{failure}")  # gate-lint:detail
        print(f"     → 看 failed_tests_by_target() 的分段与条数核对；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print("  ✓ rerun-failed-tests 自检通过（4 格：按段分目标、缺 test result、条数对不上、没有失败）")
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    if len(arguments) != 3 or arguments[1] != "-p":
        print("  ✗ 用法：rerun-failed-tests.py <cargo test 的日志> -p <包名>", file=sys.stderr)
        print("     → 例：rerun-failed-tests.py /tmp/harness.log -p singlefs-harness；自检：rerun-failed-tests.py --selftest", file=sys.stderr)
        sys.exit(2)
    sys.exit(run(arguments[0], arguments[2]))


if __name__ == '__main__':
    preflight(__file__)
    main()
