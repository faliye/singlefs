#!/usr/bin/env python3
# admission: always 读的是这一次给的那份 cargo test 输出，上一次的结论不替这一次作保
# run-condition: none 只读一份文本、打印取出的用例名，不起子进程，也不碰仓里的文件
"""libtest（cargo test 起的测试二进制与 rustdoc）打的结果行与汇总行，全仓只在这里解析一份。

调它的：scripts/mutation/crates-mutation-rows.py（门禁 checker-tier-crates-mutation-replay 的判法 results_by_test）、
scripts/mutation/prove-red.sh（经 crates-mutation-rows.py 的 verdict_of_run）、scripts/gate/rerun-failed-tests.py、
scripts/mutation/mutate.sh（经下面的命令行 failed-names）、scripts/mutation/broad-mutation.py（取第一条红的测试名与进度行里的条数，libtest_outcome / libtest_failed_name）。各自按文件路径用 importlib 导入（调用方的 sys.path 不一定有本目录）。

认的行（行尾多出的空白不算）：
  结果行  `test <全名> ... <结局>`。全名是带模块路径的用例名；rustdoc 的是 `<源文件> - <条目> (line <N>)`，中间有空格，照样整段算全名。
          #[should_panic] 的用例在全名与 ` ... ` 之间多一段 ` - should panic`：不进全名，记进 should_panic。
          结局只认 ok、FAILED、ignored；`ignored, <理由>` 记 ignored；`--report-time` 在结局后面补的 ` <0.012s>` 不算。
          结局是别的词（--nocapture 下测试自己打的字接在 ` ... ` 后面）的那一行不算结果行。
  汇总行  `test result: <ok|FAILED>. <N> passed; <N> failed; <N> ignored; …`：ignored 及之后的几段可以缺。
入口：
  result_line(line) -> ResultLine | None          ResultLine = (full_name, outcome, should_panic)
  result_lines(text) -> [ResultLine]              按出现次序
  summary_line(line) -> SummaryLine | None        SummaryLine = (outcome, passed, failed, ignored)；ignored 缺了是 None
  summary_lines(text) -> [SummaryLine]
  results_of_named_test(expected, text) -> {全名: [结局…]}   点名可以省掉模块路径：全名等于它、或以「::<它>」收尾
  failed_test_names(text) -> [全名]                 结局是 FAILED 的，按出现次序，不去重
  summaries_report_failure(text) -> bool            有一行汇总行的结局是 FAILED 或 failed 数不是 0
命令行：
  lib_libtest_result_lines.py failed-names <日志|->   一行一个 FAILED 的全名（不带 should panic 那一段），- 读标准输入；
      退 0；汇总行报了失败而一条 FAILED 结果行都没取到退 3（名字取不到：日志不全、或解析又有盲区）；读不了或用法错退 2
  lib_libtest_result_lines.py --selftest
弄坏开关 LIBTEST_RESULT_LINES_BREAK（只给证红用，逗号分隔可给几个），下一行起写打开后自证红在哪几格：
  should-panic-in-name   ` - should panic` 当成全名的一部分（mutate.sh 改前的取法）
    本库「should panic 的失败行取得出全名」「rustdoc 的结果行整段算全名、should panic 照样摘掉」「点名省掉模块路径、should panic 的用例对得上」
    「取 FAILED 的全名按出现次序」「failed-names 从标准输入取名字、退 0」；连带 crates-mutation-rows.py「should panic 的用例红了判抓到、点名省掉模块路径只对上那一个」、
    prove-red.sh「#[should_panic] 的用例红了……应当判抓到」、rerun-failed-tests.py「should panic 的失败用例按全名重跑」、
    mutate.sh 的 should-panic 与 baseline-red-should-panic 两格
  ignored-reason-kept    `ignored, <理由>` 的结局照原样交「ignored,」
    本库「带理由的 ignored 记 ignored」
  summary-failed-unread  汇总行的 failed 数不读、一律当 0，结局也不看
    本库「汇总行报了失败：failed-names 名字取不到退 3」「汇总行取出 passed、failed、ignored」
"""
import collections
import os
import re
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "LIBTEST_RESULT_LINES_BREAK"
RESULT_OUTCOMES = ("ok", "FAILED", "ignored")
SHOULD_PANIC_SUFFIX = " - should panic"
EXIT_NAMES_MISSING = 3
EXIT_USAGE = 2

ResultLine = collections.namedtuple("ResultLine", "full_name outcome should_panic")
SummaryLine = collections.namedtuple("SummaryLine", "outcome passed failed ignored")

RESULT_LINE_FORM = re.compile(r"^test (?P<name>.+?)(?P<should_panic> - should panic)? \.\.\. (?P<outcome>ok|FAILED|ignored)(?P<rest>, .*| <[^>]*>)?\s*$")
RESULT_LINE_FORM_WITH_SUFFIX_IN_NAME = re.compile(r"^test (?P<name>.+?) \.\.\. (?P<outcome>ok|FAILED|ignored)(?P<rest>, .*| <[^>]*>)?\s*$")
SUMMARY_LINE_FORM = re.compile(r"^test result: (?P<outcome>\w+)\. (?P<passed>\d+) passed; (?P<failed>\d+) failed(?:; (?P<ignored>\d+) ignored)?")


def break_is(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def result_line(line):
    """一行是结果行就交 ResultLine，不是交 None。"""
    line = line.rstrip("\r\n")
    if break_is("should-panic-in-name"):
        found = RESULT_LINE_FORM_WITH_SUFFIX_IN_NAME.match(line)
        should_panic = False
    else:
        found = RESULT_LINE_FORM.match(line)
        should_panic = bool(found and found.group("should_panic"))
    if found is None:
        return None
    outcome = found.group("outcome")
    if outcome == "ignored" and (found.group("rest") or "").startswith(",") and break_is("ignored-reason-kept"):
        outcome = "ignored,"
    return ResultLine(found.group("name"), outcome, should_panic)


def result_lines(text):
    return [parsed for parsed in map(result_line, text.split("\n")) if parsed is not None]


def summary_line(line):
    found = SUMMARY_LINE_FORM.match(line.rstrip("\r\n"))
    if found is None:
        return None
    if break_is("summary-failed-unread"):
        return SummaryLine("ok", int(found.group("passed")), 0, None)
    ignored = found.group("ignored")
    return SummaryLine(found.group("outcome"), int(found.group("passed")), int(found.group("failed")), int(ignored) if ignored is not None else None)


def summary_lines(text):
    return [parsed for parsed in map(summary_line, text.split("\n")) if parsed is not None]


def results_of_named_test(expected, text):
    """{全名: [结局…]}：全名等于点名、或以「::<点名>」收尾的那几条结果行。"""
    results = {}
    for parsed in result_lines(text):
        if parsed.full_name == expected or parsed.full_name.endswith("::" + expected):
            results.setdefault(parsed.full_name, []).append(parsed.outcome)
    return results


def failed_test_names(text):
    return [parsed.full_name for parsed in result_lines(text) if parsed.outcome == "FAILED"]


def summaries_report_failure(text):
    return any(summary.outcome == "FAILED" or summary.failed > 0 for summary in summary_lines(text))


def command_failed_names(arguments, standard_input=None, standard_output=None, standard_error=None):
    standard_input = standard_input or sys.stdin
    standard_output = standard_output or sys.stdout
    standard_error = standard_error or sys.stderr
    if len(arguments) != 1:
        print("  ✗ 用法：lib_libtest_result_lines.py failed-names <日志|->", file=standard_error)
        print("     → 怎么办：给一份 cargo test 的输出文件，或写 - 从标准输入读；自证：lib_libtest_result_lines.py --selftest", file=standard_error)
        return EXIT_USAGE
    try:
        if arguments[0] == "-":
            text = standard_input.read()
        else:
            with open(arguments[0], encoding="utf-8", errors="replace") as handle:
                text = handle.read()
    except OSError as error:
        print(f"  ✗ 读不了 {arguments[0]}：{error}", file=standard_error)
        print("     → 怎么办：看路径对不对、文件在不在；什么都没取", file=standard_error)
        return EXIT_USAGE
    names = failed_test_names(text)
    for name in names:
        print(name, file=standard_output)
    if not names and summaries_report_failure(text):
        return EXIT_NAMES_MISSING
    return 0


def selftest():
    import io
    failures, cells = [], 0

    def check(label, condition, detail):
        nonlocal cells
        cells += 1
        if not condition:
            failures.append(f"{label}：{detail}")

    plain = result_line("test tests::adds ... ok")
    check("ok 的结果行取出全名与结局", plain == ResultLine("tests::adds", "ok", False), repr(plain))
    failed = result_line("test integration::walks_to_ten ... FAILED  ")
    check("FAILED 的结果行（行尾带空白）取出全名与结局", failed == ResultLine("integration::walks_to_ten", "FAILED", False), repr(failed))
    panics = result_line("test tests::answer_panics - should panic ... FAILED")
    check("should panic 的失败行取得出全名", panics == ResultLine("tests::answer_panics", "FAILED", True), repr(panics))
    with_reason = result_line("test tests::slow_case ... ignored, 慢档用例")
    check("带理由的 ignored 记 ignored", with_reason == ResultLine("tests::slow_case", "ignored", False), repr(with_reason))
    timed = result_line("test tests::timed ... ok <0.012s>")
    check("--report-time 补的用时不算进结局", timed == ResultLine("tests::timed", "ok", False), repr(timed))
    doctest = result_line("test crates/demo/src/lib.rs - answer (line 12) - should panic ... ok")
    check("rustdoc 的结果行整段算全名、should panic 照样摘掉", doctest == ResultLine("crates/demo/src/lib.rs - answer (line 12)", "ok", True), repr(doctest))
    not_results = [result_line(line) for line in ("running 2 tests", "test result: FAILED. 1 passed; 1 failed; 0 ignored",
                                                  "test tests::noisy ... 测试自己打的字", "---- tests::adds stdout ----")]
    check("不是结果行的四种（running、汇总行、--nocapture 下接着打的字、失败段头）都不认", not_results == [None] * 4, repr(not_results))
    summary = summary_line("test result: FAILED. 1 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.01s")
    short = summary_line("test result: ok. 2 passed; 0 failed")
    check("汇总行取出 passed、failed、ignored（ignored 缺了是 None）",
          summary == SummaryLine("FAILED", 1, 2, 3) and short == SummaryLine("ok", 2, 0, None), f"{summary!r} {short!r}")
    sample = "\n".join(["running 4 tests", "test tests::adds ... ok", "test tests::answer_panics - should panic ... FAILED",
                        "test other::answer_panics ... ok", "test tests::subtracts ... FAILED", "",
                        "test result: FAILED. 2 passed; 2 failed; 0 ignored"])
    named = results_of_named_test("answer_panics", sample)
    check("点名省掉模块路径、should panic 的用例对得上（两个模块各一个，各记各的）",
          named == {"tests::answer_panics": ["FAILED"], "other::answer_panics": ["ok"]}, repr(named))
    exact = results_of_named_test("tests::adds", sample)
    check("点名写全模块路径只对上那一个", exact == {"tests::adds": ["ok"]}, repr(exact))
    check("取 FAILED 的全名按出现次序", failed_test_names(sample) == ["tests::answer_panics", "tests::subtracts"], repr(failed_test_names(sample)))
    output, errors = io.StringIO(), io.StringIO()
    code = command_failed_names(["-"], standard_input=io.StringIO(sample), standard_output=output, standard_error=errors)
    check("failed-names 从标准输入取名字、退 0", code == 0 and output.getvalue() == "tests::answer_panics\ntests::subtracts\n",
          f"退 {code}，输出 {output.getvalue()!r}")
    blind = "running 1 test\nerror: test failed\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n"
    output = io.StringIO()
    code = command_failed_names(["-"], standard_input=io.StringIO(blind), standard_output=output, standard_error=errors)
    check("汇总行报了失败：failed-names 名字取不到退 3", code == EXIT_NAMES_MISSING and output.getvalue() == "", f"退 {code}，输出 {output.getvalue()!r}")
    with tempfile.TemporaryDirectory() as directory:
        clean = os.path.join(directory, "clean.log")
        with open(clean, "w", encoding="utf-8") as handle:
            handle.write("test tests::adds ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored\n")
        output = io.StringIO()
        code = command_failed_names([clean], standard_output=output, standard_error=errors)
        check("failed-names 读文件、一个失败都没有退 0 且不打", code == 0 and output.getvalue() == "", f"退 {code}，输出 {output.getvalue()!r}")
        code = command_failed_names([os.path.join(directory, "missing.log")], standard_output=io.StringIO(), standard_error=io.StringIO())
        check("failed-names 读不了退 2", code == EXIT_USAGE, f"退 {code}")
    if failures:
        for failure in failures:
            print(f"  ✗ lib_libtest_result_lines 自检：{failure}")  # gate-lint:detail
        print(f"     → 看 RESULT_LINE_FORM、SUMMARY_LINE_FORM 与 result_line / summary_line / command_failed_names；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_libtest_result_lines 自检通过（查了 {cells} 格：ok、FAILED、should panic、带理由的 ignored、用时后缀、rustdoc 全名、"
          "不是结果行的四种、汇总行、点名省掉模块路径、FAILED 全名按次序、failed-names 的标准输入、名字取不到退 3、读文件、读不了退 2）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["failed-names"]:
        return command_failed_names(arguments[1:])
    print("  ✗ 用法：lib_libtest_result_lines.py failed-names <日志|-> 或 --selftest", file=sys.stderr)
    print("     → 怎么办：入口与认的行写在文件头；别的脚本按文件路径 importlib 导入它，调 result_lines / results_of_named_test / failed_test_names", file=sys.stderr)
    return EXIT_USAGE


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
