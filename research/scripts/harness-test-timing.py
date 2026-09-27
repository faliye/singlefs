#!/usr/bin/env python3
# admission: always 量的是此刻工作区里的 harness 档用例、判的是此刻的标记与表，上一次的结论不替这一次作保
# run-condition: command cargo git
"""harness 档用例的单条耗时：量、按门槛标重档、核标记与表对得上。重档由量出来的数定，不由整份全量日志里 libtest 的
「has been running for over 60 seconds」定——那句话在十几个测试目标同时抢核时报的是排队，不是用例本身慢。

用法：
  harness-test-timing.py measure [--jobs N] [--memory 8G] [--targets 目标,…]   # 量，写 crates/singlefs-harness/test-timing.tsv
  harness-test-timing.py apply                                               # 按表给过门槛的标重档、没过的摘掉
  harness-test-timing.py check [项目根]                                       # 标记与表对不对得上（门禁用），退 0 / 1
  harness-test-timing.py --selftest                                           # HARNESS_TEST_TIMING_BREAK=<项> 时必须判红
量法：每个测试目标（tests/<名>.rs 与库）单线程跑一遍（--include-ignored --test-threads=1，libtest 的 --report-time 报单条耗时，
要 RUSTC_BOOTSTRAP=1），目标之间并行、默认 nproc ÷ 4 个，编译目录默认 target/harness-timing（不占日常编译目录的锁），各经 research/scripts/run-with-memory-cap.sh；只量 singlefs-harness，
checker 档的用例归提交时的 54 号，这里不起。每个目标报的计时条数要等于它 test result 行的 passed + failed，对不上整次作废、不写表。
门槛：单线程 debug 下 HEAVY_THRESHOLD_SECONDS 秒及以上是重档，标 #[ignore = "harness 重档：…"]；以下的不标。
核法：带「harness 重档」标记的用例，表里要有它、耗时过门槛；表里过门槛的用例要带标记。表里没有的用例报「没量过」、不判红（新写的用例量过再说）。
"""
import concurrent.futures
import glob
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

HEAVY_THRESHOLD_SECONDS = 60.0
PACKAGE = "singlefs-harness"
PACKAGE_DIRECTORY = os.path.join("crates", PACKAGE)
TABLE = os.path.join(PACKAGE_DIRECTORY, "test-timing.tsv")
HEAVY_MARK_PREFIX = '#[ignore = "harness 重档：'
TIMED_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED) <([0-9.]+)s>")
RESULT_LINE = re.compile(r"^test result: \w+\. (\d+) passed; (\d+) failed;")
FUNCTION = re.compile(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)([ \t]*)(?:pub\s+)?fn\s+(\w+)\s*\(")
BREAK = os.environ.get("HARNESS_TEST_TIMING_BREAK", "")
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))


def heavy_attribute(seconds):
    return (f'{HEAVY_MARK_PREFIX}单线程 debug 下 {seconds:.0f} 秒（{TABLE}）；'
            f'随时跑：cargo test -p {PACKAGE} -- --ignored，经内存包装"]')


def targets_of(root):
    """[(目标名, cargo test 挑目标的参数, 源文件或目录)]：tests/ 下每个 .rs 一个，库一个。"""
    tests = [(os.path.basename(path)[:-3], ["--test", os.path.basename(path)[:-3]], path)
             for path in sorted(glob.glob(os.path.join(root, PACKAGE_DIRECTORY, "tests", "*.rs")))]
    return tests + [("lib", ["--lib"], os.path.join(root, PACKAGE_DIRECTORY, "src"))]


def parse_timings(log_text):
    """(按单条的 [(名, 结局, 秒)], 问题)：计时条数要等于 test result 行的 passed + failed。"""
    timed = [(match.group(1), match.group(2), float(match.group(3)))
             for match in map(TIMED_LINE.match, log_text.split("\n")) if match]
    reported = sum(int(match.group(1)) + int(match.group(2))
                   for match in map(RESULT_LINE.match, log_text.split("\n")) if match)
    if not any(RESULT_LINE.match(line) for line in log_text.split("\n")):
        return timed, "没有 test result 行（没编过、没跑完，或者日志被截断）"
    if len(timed) != reported and BREAK != "count-unchecked":
        return timed, f"收到 {len(timed)} 条计时，test result 报 {reported} 条"
    return timed, None


def measure(arguments):
    jobs = max(1, (os.cpu_count() or 4) // 4)
    memory = "8G"
    only = None
    position = 0
    while position < len(arguments):
        if arguments[position] == "--jobs":
            jobs = int(arguments[position + 1]); position += 2
        elif arguments[position] == "--memory":
            memory = arguments[position + 1]; position += 2
        elif arguments[position] == "--targets":
            only = set(arguments[position + 1].split(",")); position += 2
        else:
            print(f"  ✗ 认不出的参数 {arguments[position]}", file=sys.stderr)
            print("     → 用法见文件头：measure [--jobs N] [--memory 8G] [--targets 目标,…]", file=sys.stderr)
            return 2
    targets = [target for target in targets_of(".") if only is None or target[0] in only]
    # 自己的编译目录：cargo test 跑测试的整个期间占着编译目录的锁，量一趟要几十分钟，不能让日常的编译排在它后面。
    environment = dict(os.environ, RUSTC_BOOTSTRAP="1", CARGO_TARGET_DIR=os.environ.get("CARGO_TARGET_DIR", os.path.join("target", "harness-timing")))
    with tempfile.TemporaryDirectory(prefix="harness-test-timing-") as directory:
        def run_one(target):
            name, selector, _source = target
            log_path = os.path.join(directory, f"{name}.log")
            command = ["bash", os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh"), memory, "cargo", "test", "--offline", "-p", PACKAGE,
                       *selector, "--", "--include-ignored", "--test-threads=1", "-Zunstable-options", "--report-time"]
            with open(log_path, "w", encoding="utf-8") as handle:
                subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT, env=environment)
            return name, log_path
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            finished = list(pool.map(run_one, targets))
        rows, problems = [], []
        for name, log_path in finished:
            timed, problem = parse_timings(open(log_path, encoding="utf-8", errors="replace").read())
            if problem:
                problems.append(f"{name}：{problem}（日志 {log_path}）")
                continue
            rows += [(name, test, seconds, outcome) for test, outcome, seconds in timed]
        if len(finished) != len(targets):
            problems.append(f"派出 {len(targets)} 个目标，收回 {len(finished)} 个")
        if problems:
            print(f"  ✗ {len(problems)} 个目标的计时取不准，表不写：", file=sys.stderr)  # gate-lint:summary
            for problem in problems:
                print(f"      {problem}", file=sys.stderr)  # gate-lint:detail
            print("     → 怎么办：先让那几个目标编得过、跑得完（用 --targets 单量它们）；日志在临时目录里，这一次退出之后就删了，要留就另存。", file=sys.stderr)
            return 1
    kept = []
    if only is not None and os.path.isfile(TABLE):
        kept = [line for line in open(TABLE, encoding="utf-8").read().split("\n") if line and not line.startswith("#") and line.split("\t")[0] not in only]
    commit = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    header = ["# harness 档用例的单线程 debug 耗时（research/scripts/harness-test-timing.py measure 写，别手改）",
              f"# 量的那次：HEAD {commit}（加上当时工作区的改动），{os.cpu_count()} 核，目标之间并行 {jobs} 个；门槛 {HEAVY_THRESHOLD_SECONDS:.0f} 秒",
              "# 测试目标\t测试名（libtest 完整名）\t秒\t结局"]
    body = sorted(kept + [f"{name}\t{test}\t{seconds:.2f}\t{outcome}" for name, test, seconds, outcome in rows])
    with open(TABLE, "w", encoding="utf-8") as handle:
        handle.write("\n".join(header + body) + "\n")
    heavy = sum(1 for line in body if float(line.split("\t")[2]) >= HEAVY_THRESHOLD_SECONDS)
    print(f"  ✓ 量了 {len(targets)} 个目标、{len(rows)} 条用例，写进 {TABLE}；过门槛 {HEAVY_THRESHOLD_SECONDS:.0f} 秒的 {heavy} 条")
    return 0


def read_table(root):
    path = os.path.join(root, TABLE)
    if not os.path.isfile(path):
        return None
    table = {}
    for line in open(path, encoding="utf-8").read().split("\n"):
        if not line or line.startswith("#"):
            continue
        target, test, seconds, _outcome = line.split("\t")
        table[(target, test.split("::")[-1])] = float(seconds)
    return table


def marked_functions(root):
    """[(目标名, 函数名, 文件, 带不带重档标记)]：tests/<目标>.rs 顶层与库里的每个 #[test] 函数。"""
    found = []
    for target, _selector, source in targets_of(root):
        paths = [source] if source.endswith(".rs") else sorted(glob.glob(os.path.join(source, "**", "*.rs"), recursive=True))
        for path in paths:
            text = open(path, encoding="utf-8").read()
            for match in FUNCTION.finditer(text):
                if "#[test]" not in match.group(1):
                    continue
                found.append((target, match.group(3), path, HEAVY_MARK_PREFIX in match.group(1)))
    return found


def check(root):
    table = read_table(root)
    if table is None:
        print(f"  ✗ 没有 {TABLE}：重档标记没有量出来的数撑着")
        print("     → 怎么办：跑 python3 research/scripts/harness-test-timing.py measure 再 apply，表与标记一起提交。")
        return 1
    marked_without_measure, marked_below, unmarked_above, unmeasured = [], [], [], []
    for target, function, path, is_marked in marked_functions(root):
        seconds = table.get((target, function))
        if seconds is None:
            (marked_without_measure if is_marked else unmeasured).append(f"{os.path.relpath(path, root)} {function}")
        elif is_marked and seconds < HEAVY_THRESHOLD_SECONDS and BREAK != "below-threshold-unseen":
            marked_below.append(f"{os.path.relpath(path, root)} {function}：{seconds:.1f} 秒")
        elif not is_marked and seconds >= HEAVY_THRESHOLD_SECONDS:
            unmarked_above.append(f"{os.path.relpath(path, root)} {function}：{seconds:.1f} 秒")
    failed = False
    for title, entries in (("标了 harness 重档、表里却没量过它", marked_without_measure),
                           (f"标了 harness 重档、量出来不到 {HEAVY_THRESHOLD_SECONDS:.0f} 秒", marked_below),
                           (f"量出来过了 {HEAVY_THRESHOLD_SECONDS:.0f} 秒、却没标 harness 重档", unmarked_above)):
        if entries:
            failed = True
            print(f"  ✗ {len(entries)} 条用例{title}：")  # gate-lint:summary
            for entry in entries:
                print(f"      {entry}")  # gate-lint:detail
    if failed:
        print("     → 怎么办：跑 python3 research/scripts/harness-test-timing.py measure（只量变了的目标用 --targets）再 apply，表与标记一起提交；别手标手摘。")
        return 1
    total = len(marked_functions(root))
    heavy = sum(1 for seconds in table.values() if seconds >= HEAVY_THRESHOLD_SECONDS)
    print(f"  ✓ harness 档 {total} 条用例：标重档的 {heavy} 条都量过、都过 {HEAVY_THRESHOLD_SECONDS:.0f} 秒；"
          f"没量过的 {len(unmeasured)} 条（新写的用例量过再判）")
    return 0


def apply():
    table = read_table(".")
    if table is None:
        print(f"  ✗ 没有 {TABLE}", file=sys.stderr)
        print("     → 怎么办：先 measure。", file=sys.stderr)
        return 1
    changed = 0
    by_path = {}
    for target, function, path, is_marked in marked_functions("."):
        seconds = table.get((target, function))
        if seconds is None:
            continue
        by_path.setdefault(path, []).append((function, seconds, is_marked))
    for path, functions in by_path.items():
        text = open(path, encoding="utf-8").read()
        for function, seconds, is_marked in functions:
            match = next(m for m in FUNCTION.finditer(text) if m.group(3) == function and "#[test]" in m.group(1))
            attributes, indent = match.group(1), match.group(2)
            lines = [line for line in attributes.split("\n") if not line.strip().startswith(HEAVY_MARK_PREFIX)]
            if seconds >= HEAVY_THRESHOLD_SECONDS:
                test_index = next(i for i, line in enumerate(lines) if line.strip() == "#[test]")
                lines.insert(test_index + 1, indent + heavy_attribute(seconds))
            new_attributes = "\n".join(lines)
            if new_attributes != attributes:
                text = text[:match.start(1)] + new_attributes + text[match.end(1):]
                changed += 1
        temporary = path + ".timing.new"
        open(temporary, "w", encoding="utf-8").write(text)
        os.chmod(temporary, os.stat(path).st_mode)
        os.replace(temporary, path)
    print(f"  ✓ 按 {TABLE} 改了 {changed} 条用例的重档标记（门槛 {HEAVY_THRESHOLD_SECONDS:.0f} 秒）")
    return 0


def selftest():
    failures = []
    log = "\n".join(["     Running tests/a.rs (x)", "test fast_case ... ok <0.20s>", "test slow_case ... ok <75.00s>",
                     "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"])
    timed, problem = parse_timings(log)
    if problem or [name for name, _o, _s in timed] != ["fast_case", "slow_case"]:
        failures.append(f"计时解析：{timed} {problem}")
    _timed, problem = parse_timings(log.replace("test fast_case ... ok <0.20s>\n", ""))
    if not problem:
        failures.append("少一条计时应当判出问题")
    with tempfile.TemporaryDirectory(prefix="harness-test-timing-selftest-") as root:
        tests_directory = os.path.join(root, PACKAGE_DIRECTORY, "tests")
        os.makedirs(tests_directory)
        os.makedirs(os.path.join(root, PACKAGE_DIRECTORY, "src"))
        open(os.path.join(root, PACKAGE_DIRECTORY, "src", "lib.rs"), "w").write("pub fn nothing() {}\n")
        open(os.path.join(root, TABLE), "w").write("# 样本\na\tfast_case\t0.20\tok\na\tslow_case\t75.00\tok\n")
        test_file = os.path.join(tests_directory, "a.rs")
        open(test_file, "w").write("#[test]\nfn fast_case() {}\n\n#[test]\n" + heavy_attribute(75.0) + "\nfn slow_case() {}\n")
        if check(root) != 0:
            failures.append("标记与表一致时 check 应当退 0")
        open(test_file, "w").write("#[test]\n" + heavy_attribute(3.0) + "\nfn fast_case() {}\n\n#[test]\nfn slow_case() {}\n")
        if check(root) != 1:
            failures.append("快用例标了重档、慢用例没标时 check 应当退 1")
        open(test_file, "w").write("#[test]\n" + heavy_attribute(3.0) + "\nfn fast_case() {}\n\n#[test]\n" + heavy_attribute(75.0) + "\nfn slow_case() {}\n")
        if check(root) != 1:
            failures.append("只有快用例错标重档时 check 也应当退 1")
    if failures:
        for failure in failures:
            print(f"  ✗ harness-test-timing 自检：{failure}")  # gate-lint:detail
        print("     → 看 parse_timings() 与 check() 的判法；HARNESS_TEST_TIMING_BREAK 设着的话这里本来就该红")
        return 1
    print("  ✓ harness-test-timing 自检通过（5 格：解析、少一条计时、一致、错标与漏标、只错标）")
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["measure"]:
        return measure(arguments[1:])
    if arguments == ["apply"]:
        return apply()
    if arguments[:1] == ["check"]:
        return check(arguments[1] if len(arguments) > 1 else ".")
    print("  ✗ 用法：harness-test-timing.py measure|apply|check [项目根]|--selftest", file=sys.stderr)
    print("     → 文件头有每个子命令做什么", file=sys.stderr)
    return 2


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main())
