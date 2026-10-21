#!/usr/bin/env python3
# admission: always 读的是这一次给的那份 cargo test 输出，上一次的结论不替这一次作保
# run-condition: none 只读一份文本、打印取出的用例名，不起子进程，也不碰仓里的文件
"""libtest（cargo test 起的测试二进制与 rustdoc）打的结果行与汇总行，全仓只在这里解析一份。

调它的：scripts/mutation/crates-mutation-rows.py（门禁 checker-tier-crates-mutation-replay 的判法 results_by_test）、
scripts/mutation/prove-red.sh（经 crates-mutation-rows.py 的 verdict_of_run）、scripts/gate/rerun-failed-tests.py、
scripts/mutation/mutate.sh（经下面的命令行 failed-names）、scripts/mutation/broad-mutation.py（取第一条红的测试名与进度行里的条数，libtest_outcome / libtest_failed_name）、
门禁 checker-tier-crash-point-replay 的快档两格（recordable_results：逐条认判绿的测试函数、打了记号的那一条不记，照键记库；判定请求点名
judge:scripts/gate/lib_libtest_result_lines.py::recordable_results）与 checker-tier-research-build-and-replay 的 research-unit-tests 格（binary_runs；判定请求点名
judge:scripts/gate/lib_libtest_result_lines.py::result_line）：点名的那个函数的闭包改了，按条记的判定一起作废。
各自按文件路径用 importlib 导入（调用方的 sys.path 不一定有本目录）。

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
  binary_runs(text) -> [BinaryRun]                  cargo test 的整段输出按测试二进制切：BinaryRun = (source, results, summary, consistent)。
      source 是 cargo 那一行「Running <源文件> (<二进制>)」里的源文件（unittests 的写 src/lib.rs、src/bin/<名>.rs 这一类，集成测试写 tests/<名>.rs；
      Doc-tests 写 doctest:<包>），results 是那一段里的结果行，summary 是那一段的汇总行（没有是 None）；
      consistent：有汇总行、且结果行条数等于汇总行的 passed + failed + ignored（--nocapture 下测试自己打的字把结果行打散了的，这一段对不上，
      research-unit-tests 格按条入库时整段不记：逐条认出测试名与结局要靠它兜）
  recordable_results(text, serial=False, cpu_judging_recordable=False) -> [BinaryAccount]   按条记库（下面「按条记库」）：BinaryAccount = (source, recorded, withheld, markers, unseen)；
      recorded 是判绿可记库的结果行，withheld 是 [(全名, 不记的原因)]，markers 是 [Marker]，Marker = (source, kind, text, candidates, complete)，
      unseen 是汇总行报了、这一段里连「test <名> ... 」都没认出的条数
按条记库（门禁 checker-tier-crash-point-replay 快档两格）：只认结局是 ok 的那几条，下面几种不记，别的照记：
  打散的结果行：--nocapture 下「test <名> ... 」后面接了测试自己打的字，结局另起一行单独打（`ok`、`FAILED`、`ignored`，可带「, 理由」或用时）的，
      接成一条照认；结局粘在别的字后面、或下一条「test <名> ... 」先来了的，那一条认不出结局、不记。这一段另有对不上号的单独一行结局时，接出来的都不认。
  完整性闸：没有汇总行的一段一条不记；认出的结果行（总数，或 ok、FAILED、ignored 任一种）比汇总行报的多，整段不记；同一段里同名的结果行不止一行，那个名字不记。
      汇总行报的比认出的多（有几条连起头都没认出），认出的照记，认不出的本来就不在名单里。
  记号：一行里的 NOT_RUN_<原因>（后面接空白或行尾；kind 是 not-run，text 从它起到同一行下一处 NOT_RUN_ 之前），与判段判在 CPU 的行
      （CRASH_POINT_REPLAY_JUDGE route=cpu、CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu；kind 是 cpu-judging）。记号认作谁打的，那几条不记；
      cpu_judging_recordable（门禁在多机配置没开 GPU 时给，D13 已定项 17「复用与剪枝不看设备」）下 cpu-judging 照样交出记号、不让谁不记，not-run 照旧不记：
      serial（这一趟带了 --test-threads 1，libtest 在测试开跑时就打「test <名> ... 」）：记号之前最近的那一条「test <名> ... 」；之前一条都没有的认作这一段全部。
      不是 serial（几条同时跑，libtest 在测试跑完才打它的结果行）：这一段里结果行（打散的按结局那一行）在记号之后的全部、连同认不出结局而起头在记号之后的，
      即打这一行时还没跑完的都算。complete 是 candidates 一定罩住了打这一行的那一条（serial 下找到了最近那一条；别的情形要有汇总行、这一段 unseen 是 0）。
命令行：
  lib_libtest_result_lines.py failed-names <日志|->   一行一个 FAILED 的全名（不带 should panic 那一段），- 读标准输入；
      退 0；汇总行报了失败而一条 FAILED 结果行都没取到退 3（名字取不到：日志不全、或解析又有盲区）；读不了或用法错退 2
  lib_libtest_result_lines.py binary-results <日志|->  逐条打「<源文件><制表符><全名><制表符><结局><制表符><这一段对得上：1|0>」，末行 BINARY_RESULTS binaries=<段数> results=<条数>
  lib_libtest_result_lines.py recordable <日志|-> [--serial] [--cpu-judging-recordable]  逐条打「green<TAB><源文件><TAB><全名>」「withheld<TAB><源文件><TAB><全名><TAB><原因>」
      「marker<TAB><源文件><TAB><not-run|cpu-judging><TAB><complete：1|0><TAB><那一截><TAB><认作谁打的，一个一列>」，末行 RECORDABLE binaries=… green=… withheld=… markers=…
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
  per-binary-count-unchecked  binary_runs 不核「结果行条数 = 汇总行三个数之和」，一律当对得上
    本库「逐条认出测试名与结局：--nocapture 打散了结果行的那一段对不上」
  marker-withholds-whole-binary  一段里有记号就整段不记（改前门禁 excluded_binaries_of_log 的样子）
    本库「按条记库：一段三条、一条打了 NOT_RUN_…」「按条记库：带 --test-threads 1 的一段…」；连带门禁 checker-tier-crash-point-replay 的 GATE_CRASH_POINT_REPLAY_BREAK=withhold-whole-binary
  count-mismatch-withholds-whole-binary  没打散的结果行条数与汇总行对不上就整段不记（改前 binary_runs 的 consistent 当门槛的样子）
    本库「按条记库：结果行被打散，接得上结局的照记…」「按条记库：带 --test-threads 1 的一段…」；连带同上那个门禁开关
  cpu-judging-always-withheld  给了 cpu_judging_recordable（--cpu-judging-recordable）也照改前把判段判在 CPU 的那一条不记
    本库「按条记库：没开 GPU 时判段判在 CPU 的照记…」「recordable --cpu-judging-recordable…」
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
BinaryRun = collections.namedtuple("BinaryRun", "source results summary consistent")
RUNNING_LINE_FORM = re.compile(r"^\s*Running (?:unittests )?(?P<source>\S+) \(")
DOCTESTS_LINE_FORM = re.compile(r"^\s*Doc-tests (?P<package>\S+)")
SummaryLine = collections.namedtuple("SummaryLine", "outcome passed failed ignored")

RESULT_LINE_FORM = re.compile(r"^test (?P<name>.+?)(?P<should_panic> - should panic)? \.\.\. (?P<outcome>ok|FAILED|ignored)(?P<rest>, .*| <[^>]*>)?\s*$")
RESULT_LINE_FORM_WITH_SUFFIX_IN_NAME = re.compile(r"^test (?P<name>.+?) \.\.\. (?P<outcome>ok|FAILED|ignored)(?P<rest>, .*| <[^>]*>)?\s*$")
SUMMARY_LINE_FORM = re.compile(r"^test result: (?P<outcome>\w+)\. (?P<passed>\d+) passed; (?P<failed>\d+) failed(?:; (?P<ignored>\d+) ignored)?")
# 按条记库（文件头「按条记库」）：被打散的结果行的起头与单独一行的结局、两种记号
SCATTERED_START_FORM = re.compile(r"^test (?P<name>.+?)(?: - should panic)? \.\.\. (?P<rest>.*)$")
STANDALONE_OUTCOME_FORM = re.compile(r"^(?P<outcome>ok|FAILED|ignored)(?:, .*| <[^>]*>)?\s*$")
NOT_RUN_MARKER_FORM = re.compile(r"NOT_RUN_[A-Z][A-Z0-9_]*(?=\s|$)")
CPU_JUDGING_FORMS = (re.compile(r"CRASH_POINT_REPLAY_JUDGE route=cpu(?=\s|$)"),
                     re.compile(r"CRASH_POINT_REPLAY_PARALLEL_(?:START|FINISHED) (?:.*\s)?judge=cpu(?=\s|$)"))
AccountEntry = collections.namedtuple("AccountEntry", "name outcome start end scattered")
Marker = collections.namedtuple("Marker", "source kind text candidates complete")
BinaryAccount = collections.namedtuple("BinaryAccount", "source recorded withheld markers unseen")


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


def binary_runs(text):
    """cargo test 的整段输出按测试二进制切（文件头「入口」binary_runs）。"""
    runs, source, results, summary = [], None, [], None

    def close():
        if source is None:
            return
        count_ok = summary is not None and len(results) == summary.passed + summary.failed + (summary.ignored or 0)
        runs.append(BinaryRun(source, list(results), summary, count_ok or (summary is not None and break_is("per-binary-count-unchecked"))))

    for line in text.split("\n"):
        running = RUNNING_LINE_FORM.match(line)
        doctests = DOCTESTS_LINE_FORM.match(line)
        if running or doctests:
            close()
            source = running.group("source") if running else f"doctest:{doctests.group('package')}"
            results, summary = [], None
            continue
        if source is None:
            continue
        parsed = result_line(line)
        if parsed is not None:
            results.append(parsed)
            continue
        parsed_summary = summary_line(line)
        if parsed_summary is not None and summary is None:
            summary = parsed_summary
    close()
    return runs


def marker_pieces(line):
    """一行里的记号（文件头「按条记库」）：[(种类, 那一截)]。not-run 一处一截：从 NOT_RUN_ 起到同一行下一处 NOT_RUN_ 之前（去掉行尾空白）；
    cpu-judging 一行至多一截：去掉行首空白的整行。"""
    pieces = []
    starts = [found.start() for found in NOT_RUN_MARKER_FORM.finditer(line)]
    for position, start in enumerate(starts):
        end = starts[position + 1] if position + 1 < len(starts) else len(line)
        pieces.append(("not-run", line[start:end].rstrip()))
    if any(form.search(line) for form in CPU_JUDGING_FORMS):
        pieces.append(("cpu-judging", line.strip()))
    return pieces


def account_binary(source, lines, serial, cpu_judging_recordable=False):
    """一个测试二进制那一段（Running 那一行之后、下一段之前的行）逐条认结局、认记号归谁：交 BinaryAccount（文件头「按条记库」）。
    cpu_judging_recordable：判段判在 CPU 的记号照交、不让谁不记（弄坏开关 cpu-judging-always-withheld 下照改前不记）。"""
    cpu_judging_withholds = not cpu_judging_recordable or break_is("cpu-judging-always-withheld")
    entries, lost, starts, raw_markers = [], [], [], []
    pending, summary, orphans = None, None, 0
    for index, raw in enumerate(lines):
        line = raw.rstrip("\r\n")
        parsed = result_line(line)
        if parsed is not None:
            if pending is not None:
                lost.append(pending)
                pending = None
            entries.append(AccountEntry(parsed.full_name, parsed.outcome, index, index, False))
            starts.append((index, parsed.full_name))
            continue
        if summary is None:
            parsed_summary = summary_line(line)
            if parsed_summary is not None:
                summary = parsed_summary
                continue
        started = SCATTERED_START_FORM.match(line)
        if started is not None and not line.startswith("test result: "):
            if pending is not None:
                lost.append(pending)
            pending = (started.group("name"), index)
            starts.append((index, started.group("name")))
            for kind, piece in marker_pieces(started.group("rest")):
                raw_markers.append((index, kind, piece))
            continue
        outcome = STANDALONE_OUTCOME_FORM.match(line)
        if outcome is not None:
            if pending is not None:
                entries.append(AccountEntry(pending[0], outcome.group("outcome"), pending[1], index, True))
                pending = None
            else:
                orphans += 1
            continue
        for kind, piece in marker_pieces(line):
            raw_markers.append((index, kind, piece))
    if pending is not None:
        lost.append(pending)
    names = list(dict.fromkeys([entry.name for entry in entries] + [name for name, _index in lost]))
    withheld = collections.OrderedDict()

    def withhold(name, reason):
        withheld.setdefault(name, reason)

    total = summary.passed + summary.failed + (summary.ignored or 0) if summary is not None else 0
    unseen = max(total - len(entries) - len(lost), 0)
    markers = []
    for index, kind, piece in raw_markers:
        if serial:
            owners = [name for start, name in starts if start <= index]
            candidates, complete = (owners[-1:], True) if owners else (names, summary is not None and unseen == 0)
        else:
            later = [entry.name for entry in entries if entry.end > index] + [name for name, start in lost if start > index]
            candidates, complete = list(dict.fromkeys(later)), summary is not None and unseen == 0
        if break_is("marker-withholds-whole-binary"):
            candidates = names
        markers.append(Marker(source, kind, piece, tuple(candidates), complete))
        if kind == "cpu-judging" and not cpu_judging_withholds:
            continue
        label = "判段判在 CPU 的一行" if kind == "cpu-judging" else piece[:120]
        for name in candidates:
            withhold(name, f"这一段打了 {label}，认作它打的" + ("" if serial else "（几条测试同时跑，打这一行时还没出结果的都算）"))
    if summary is None:
        for name in names:
            withhold(name, "这一段没有汇总行（测试二进制没跑完）")
        return BinaryAccount(source, [], list(withheld.items()), markers, unseen)
    intact = [entry for entry in entries if not entry.scattered]
    if break_is("count-mismatch-withholds-whole-binary") and len(intact) != total:
        for name in names:
            withhold(name, f"弄坏开关 count-mismatch-withholds-whole-binary：结果行 {len(intact)} 条、汇总行 {total} 条，整段不记")
    recognized = entries
    if orphans:
        for entry in entries:
            if entry.scattered:
                withhold(entry.name, f"结果行被打散，这一段另有 {orphans} 行对不上号的结局，接出来的结局不认")
        recognized = intact
    counts = collections.Counter(entry.outcome for entry in recognized)
    if (len(recognized) > total or counts["ok"] > summary.passed or counts["FAILED"] > summary.failed
            or (summary.ignored is not None and counts["ignored"] > summary.ignored)):
        for name in names:
            withhold(name, f"认出的结果行（{len(recognized)} 条）比汇总行报的（{total} 条）多，这一段整段不记")
    for name, count in collections.Counter(entry.name for entry in recognized).items():
        if count > 1:
            withhold(name, f"同一段里这个名字的结果行有 {count} 行")
    for name, _index in lost:
        withhold(name, "结果行被打散、认不出结局")
    recorded = [ResultLine(entry.name, entry.outcome, False) for entry in recognized if entry.outcome == "ok" and entry.name not in withheld]
    return BinaryAccount(source, recorded, list(withheld.items()), markers, unseen)


def recordable_results(text, serial=False, cpu_judging_recordable=False):
    """cargo test 的整段输出按测试二进制切，逐段交 BinaryAccount（文件头「按条记库」）：判绿可记库的、不记的与为什么、记号归谁。
    serial：这一趟带了 --test-threads 1（libtest 在测试开跑时就打「test <名> ... 」）。
    cpu_judging_recordable：多机配置没开 GPU，判段判在 CPU 的那一条照记（D13 已定项 17：判定不与 CPU 还是 GPU 绑定）。"""
    accounts, source, lines = [], None, []
    for line in text.split("\n"):
        running = RUNNING_LINE_FORM.match(line)
        doctests = DOCTESTS_LINE_FORM.match(line)
        if running or doctests:
            if source is not None:
                accounts.append(account_binary(source, lines, serial, cpu_judging_recordable))
            source = running.group("source") if running else f"doctest:{doctests.group('package')}"
            lines = []
            continue
        if source is not None:
            lines.append(line)
    if source is not None:
        accounts.append(account_binary(source, lines, serial, cpu_judging_recordable))
    return accounts


def command_recordable(arguments, standard_input=None, standard_output=None, standard_error=None):
    standard_input = standard_input or sys.stdin
    standard_output = standard_output or sys.stdout
    standard_error = standard_error or sys.stderr
    serial = "--serial" in arguments
    cpu_judging_recordable = "--cpu-judging-recordable" in arguments
    arguments = [argument for argument in arguments if argument not in ("--serial", "--cpu-judging-recordable")]
    if len(arguments) != 1:
        print("  ✗ 用法：lib_libtest_result_lines.py recordable <日志|-> [--serial] [--cpu-judging-recordable]", file=standard_error)
        print("     → 怎么办：给一份 cargo test 的整段输出文件，或写 - 从标准输入读；这一趟带了 --test-threads 1 的加 --serial；"
              "多机配置没开 GPU、判段判在 CPU 的照记的加 --cpu-judging-recordable", file=standard_error)
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
    accounts = recordable_results(text, serial, cpu_judging_recordable)
    for account in accounts:
        for parsed in account.recorded:
            print(f"green\t{account.source}\t{parsed.full_name}", file=standard_output)
        for name, reason in account.withheld:
            print(f"withheld\t{account.source}\t{name}\t{reason}", file=standard_output)
        for marker in account.markers:
            print("\t".join(["marker", marker.source, marker.kind, "1" if marker.complete else "0", marker.text, *marker.candidates]), file=standard_output)
    print(f"RECORDABLE binaries={len(accounts)} green={sum(len(account.recorded) for account in accounts)} "
          f"withheld={sum(len(account.withheld) for account in accounts)} markers={sum(len(account.markers) for account in accounts)}", file=standard_output)
    return 0


def command_binary_results(arguments, standard_input=None, standard_output=None, standard_error=None):
    standard_input = standard_input or sys.stdin
    standard_output = standard_output or sys.stdout
    standard_error = standard_error or sys.stderr
    if len(arguments) != 1:
        print("  ✗ 用法：lib_libtest_result_lines.py binary-results <日志|->", file=standard_error)
        print("     → 怎么办：给一份 cargo test 的整段输出文件，或写 - 从标准输入读", file=standard_error)
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
    runs = binary_runs(text)
    for run in runs:
        for parsed in run.results:
            print(f"{run.source}\t{parsed.full_name}\t{parsed.outcome}\t{1 if run.consistent else 0}", file=standard_output)
    print(f"BINARY_RESULTS binaries={len(runs)} results={sum(len(run.results) for run in runs)}", file=standard_output)
    return 0


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
    cargo_output = "\n".join([
        "   Compiling demo v0.1.0", "     Running unittests src/lib.rs (target/release/deps/demo-0123456789abcdef)", "", "running 2 tests",
        "test tests::adds ... ok", "test tests::slow ... ignored", "", "test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out",
        "     Running tests/setting.rs (target/release/deps/setting-fedcba9876543210)", "running 3 tests",
        "test test_t ... ok", "test test_u ... 测试自己打的字", "ok", "test test_v ... FAILED", "",
        "test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
        "   Doc-tests demo", "running 0 tests", "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"])
    runs = binary_runs(cargo_output)
    check("逐条认出测试名与结局：按「Running <源文件>」切成三段，每段的结果行、汇总行各归各的",
          [run.source for run in runs] == ["src/lib.rs", "tests/setting.rs", "doctest:demo"]
          and [(parsed.full_name, parsed.outcome) for parsed in runs[0].results] == [("tests::adds", "ok"), ("tests::slow", "ignored")]
          and [(parsed.full_name, parsed.outcome) for parsed in runs[1].results] == [("test_t", "ok"), ("test_v", "FAILED")], repr(runs))
    check("逐条认出测试名与结局：结果行条数与汇总行对得上的段算对得上（lib 那一段、0 条的 doctest 那一段）", runs[0].consistent and runs[2].consistent, repr(runs))
    check("逐条认出测试名与结局：--nocapture 打散了结果行的那一段对不上（3 条只认出 2 条），门禁整段不记", not runs[1].consistent, repr(runs[1]))
    output = io.StringIO()
    code = command_binary_results(["-"], standard_input=io.StringIO(cargo_output), standard_output=output, standard_error=errors)
    check("binary-results 逐条打源文件、全名、结局、这一段对不对得上，末行报段数与条数",
          code == 0 and output.getvalue().split("\n")[0] == "src/lib.rs\ttests::adds\tok\t1" and "BINARY_RESULTS binaries=3 results=4" in output.getvalue(),
          f"退 {code}，输出 {output.getvalue()!r}")
    # 按条记库（文件头「按条记库」）：几条同时跑的一段里一条打了 NOT_RUN_，只有打这一行时还没出结果的那一条不记
    threaded = "\n".join([
        "     Running tests/setting.rs (target/release/deps/setting-0000000000000001)", "running 3 tests",
        "test test_a ... ok", "test test_b ... ok", "NOT_RUN_SAMPLE_REASON test=crash_state_enumeration::sample reason=样本", "test test_c ... ok", "",
        "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        "     Running tests/noisy.rs (target/release/deps/noisy-0000000000000002)", "running 3 tests",
        "test noisy_split ... 测试自己打的字", "ok", "test noisy_glued ... 测试自己打的字ok", "test noisy_plain ... ok", "",
        "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"])
    accounts = recordable_results(threaded)
    recorded = [[parsed.full_name for parsed in account.recorded] for account in accounts]
    withheld = [dict(account.withheld) for account in accounts]
    check("按条记库：一段三条、一条打了 NOT_RUN_，另两条照记、那一条不记（几条同时跑：打这一行时还没出结果的才算它打的）",
          len(accounts) == 2 and recorded[0] == ["test_a", "test_b"] and list(withheld[0]) == ["test_c"]
          and accounts[0].markers == [Marker("tests/setting.rs", "not-run", "NOT_RUN_SAMPLE_REASON test=crash_state_enumeration::sample reason=样本", ("test_c",), True)],
          repr(accounts[:1]))
    check("按条记库：结果行被打散，接得上结局的照记、粘在别的字后面认不出结局的那一条不记",
          len(accounts) == 2 and recorded[1] == ["noisy_split", "noisy_plain"] and list(withheld[1]) == ["noisy_glued"], repr(accounts[1:]))
    serial = "\n".join([
        "     Running tests/gpu_pair.rs (target/release/deps/gpu_pair-0000000000000003)", "running 3 tests",
        "test gpu_first ... CRASH_POINT_REPLAY_JUDGE route=gpu cards_used=1", "ok",
        "test gpu_second ... NOT_RUN_NO_GPU_ADAPTER test=gpu_second reason=样本", "ok",
        "test gpu_third ... CRASH_POINT_REPLAY_JUDGE route=cpu why_not_the_gpu=样本", "ok", "",
        "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"])
    serial_accounts = recordable_results(serial, serial=True)
    check("按条记库：带 --test-threads 1 的一段，记号归打它时正在跑的那一条（NOT_RUN_ 与判段判在 CPU 各一条不记，另一条照记）",
          [parsed.full_name for parsed in serial_accounts[0].recorded] == ["gpu_first"]
          and [name for name, _reason in serial_accounts[0].withheld] == ["gpu_second", "gpu_third"]
          and [(marker.kind, marker.candidates) for marker in serial_accounts[0].markers] == [("not-run", ("gpu_second",)), ("cpu-judging", ("gpu_third",))],
          repr(serial_accounts))
    recordable_accounts = recordable_results(serial, serial=True, cpu_judging_recordable=True)
    check("按条记库：没开 GPU 时判段判在 CPU 的照记（同一段带 --test-threads 1，cpu_judging_recordable：判在 CPU 那一条照记、NOT_RUN_ 那一条照旧不记，两处记号照交）",
          [parsed.full_name for parsed in recordable_accounts[0].recorded] == ["gpu_first", "gpu_third"]
          and [name for name, _reason in recordable_accounts[0].withheld] == ["gpu_second"]
          and [(marker.kind, marker.candidates) for marker in recordable_accounts[0].markers] == [("not-run", ("gpu_second",)), ("cpu-judging", ("gpu_third",))],
          repr(recordable_accounts))
    cpu_judged = "\n".join([
        "     Running tests/plain_stream.rs (target/release/deps/plain_stream-0000000000000007)", "running 2 tests",
        "test plain_stream_full ... ignored", "CRASH_POINT_REPLAY_JUDGE route=cpu why_not_the_gpu=gpu_not_enabled_in_the_configuration threads=4",
        "test plain_stream_quick ... ok", "", "test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out"])
    output = io.StringIO()
    code = command_recordable(["-", "--cpu-judging-recordable"], standard_input=io.StringIO(cpu_judged), standard_output=output, standard_error=errors)
    check("recordable --cpu-judging-recordable：几条同时跑的一段里判段判在 CPU 的一行照打 marker、不让打它时还没出结果的那一条不记",
          code == 0 and "green\ttests/plain_stream.rs\tplain_stream_quick" in output.getvalue() and "\nwithheld\t" not in "\n" + output.getvalue()
          and "marker\ttests/plain_stream.rs\tcpu-judging\t1\t" in output.getvalue() and "RECORDABLE binaries=1 green=1 withheld=0 markers=1" in output.getvalue(),
          f"退 {code}，输出 {output.getvalue()!r}")
    orphaned = "\n".join(["     Running tests/noisy.rs (target/release/deps/noisy-0000000000000004)", "running 2 tests",
                          "test noisy_split ... 测试自己打的字", "ok", "ok", "test noisy_plain ... ok", "",
                          "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"])
    orphaned_account = recordable_results(orphaned)[0]
    check("按条记库：这一段另有对不上号的单独一行结局，接出来的结局不认（只记没打散的那一条）",
          [parsed.full_name for parsed in orphaned_account.recorded] == ["noisy_plain"] and [name for name, _reason in orphaned_account.withheld] == ["noisy_split"],
          repr(orphaned_account))
    overclaimed = "\n".join(["     Running tests/fake.rs (target/release/deps/fake-0000000000000005)", "running 1 test",
                             "test fake_echo ... ok", "test fake_real ... ok", "", "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
                             "     Running tests/crashed.rs (target/release/deps/crashed-0000000000000006)", "running 2 tests",
                             "test crashed_first ... ok", "NOT_RUN_SAMPLE_REASON test=crashed_second reason=样本"])
    overclaimed_accounts = recordable_results(overclaimed)
    check("按条记库：认出的结果行比汇总行报的多，整段不记；没有汇总行的一段一条不记，记号照样交出来",
          [account.recorded for account in overclaimed_accounts] == [[], []]
          and [name for name, _reason in overclaimed_accounts[0].withheld] == ["fake_echo", "fake_real"]
          and len(overclaimed_accounts[1].markers) == 1 and not overclaimed_accounts[1].markers[0].complete, repr(overclaimed_accounts))
    output = io.StringIO()
    code = command_recordable(["-"], standard_input=io.StringIO(threaded), standard_output=output, standard_error=errors)
    check("recordable 逐条打 green、withheld、marker，末行报段数与条数",
          code == 0 and "green\ttests/setting.rs\ttest_a" in output.getvalue() and "withheld\ttests/noisy.rs\tnoisy_glued\t结果行被打散、认不出结局" in output.getvalue()
          and "marker\ttests/setting.rs\tnot-run\t1\tNOT_RUN_SAMPLE_REASON" in output.getvalue()
          and "RECORDABLE binaries=2 green=4 withheld=2 markers=1" in output.getvalue(), f"退 {code}，输出 {output.getvalue()!r}")
    if failures:
        for failure in failures:
            print(f"  ✗ lib_libtest_result_lines 自检：{failure}")  # gate-lint:detail
        print(f"     → 看 RESULT_LINE_FORM、SUMMARY_LINE_FORM 与 result_line / summary_line / command_failed_names；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_libtest_result_lines 自检通过（查了 {cells} 格：ok、FAILED、should panic、带理由的 ignored、用时后缀、rustdoc 全名、"
          "不是结果行的四种、汇总行、点名省掉模块路径、FAILED 全名按次序、failed-names 的标准输入、名字取不到退 3、读文件、读不了退 2、"
          "逐条认出测试名与结局（按测试二进制切段、条数与汇总行对不上的段整段不认、binary-results）、"
          "按条记库（打了记号的那一条不记、打散的结果行接得上结局的照记、带 --test-threads 1 的段记号归正在跑的那一条、"
          "没开 GPU 时判段判在 CPU 的照记、对不上号的单独结局、认多了与没有汇总行、recordable 与 --cpu-judging-recordable））")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["failed-names"]:
        return command_failed_names(arguments[1:])
    if arguments[:1] == ["binary-results"]:
        return command_binary_results(arguments[1:])
    if arguments[:1] == ["recordable"]:
        return command_recordable(arguments[1:])
    print("  ✗ 用法：lib_libtest_result_lines.py failed-names <日志|->、binary-results <日志|->、recordable <日志|-> [--serial] 或 --selftest", file=sys.stderr)
    print("     → 怎么办：入口与认的行写在文件头；别的脚本按文件路径 importlib 导入它，调 result_lines / results_of_named_test / failed_test_names", file=sys.stderr)
    return EXIT_USAGE


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
