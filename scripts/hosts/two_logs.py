#!/usr/bin/env python3
# admission: always 直接执行只包一条命令或现筛一份已有的日志，判的是这一次的命令与这一刻的文件，上一次的结论不替这一次作保
# run-condition: none 只用标准库，除了 python3 与被包的命令之外没有环境要求；入口是同目录的 two-logs.sh，它先过一遍准入再 exec 这一份
"""日志出两份：一份全量，一份只收要看的行（错误与警告）。two-logs.sh 的实现，筛选写成纯函数，relay 一处。

    python3 scripts/hosts/two_logs.py [--quiet] <名> -- <命令…>     包住命令：标准输出与标准错误各自照原样转打，同时写 <名>.log（全量）
                                                                   与 <名>.problems.log（只收要看的行，每行前面带它在全量日志里的行号）
    python3 scripts/hosts/two_logs.py --from-log <名>               不跑命令：从已有的 <名>.log 现筛出 <名>.problems.log
    python3 scripts/hosts/two_logs.py --selftest                    自证

filter_problems(lines, breaks=frozenset()) -> list[str]：把一份全量日志的行（不带换行）筛成 problems 日志的行（「行号:原行」，不含收尾行）。
ProblemFilter 是同一个判法的逐行版（relay 边收边筛用它），filter_problems 只是把整份喂给它。收哪些行：
  门禁符号：带 ✗ 的行；✗ 与 ⚠ 之后跟着的 → 出路行，以及 → 之前的明细行、→ 之后缩进的续行（缩进 4 格以上，一块至多 DETAIL_CAP 行，超出的不收、
            在块里写一行说明；遇到不是 → 也不是缩进行的那一行，这一块结束）；带 ⚠ 的行；带「本次未跑」的行；
            逐格汇总：「── 汇总」起头的行与整行是「<格名>：绿 / 红 / 红（退出 N） / 本次未跑」的行。
  工具报错：cargo / rustc 的 error、error[E…]、warning: 行，连同紧跟其后、指到文件行号的 --> 那一行；libtest 的 FAILED 与 panicked；
            Python 的 Traceback 起的那一段（缩进的栈帧行，到收尾的异常行为止；「During handling…」「The above exception…」接着收下一段）；
            报非 0 退出码的行（「退出码 N」「退出 N」「退 N」、exit status / exit code / exited with N，N 是 1 起的整数）。
别的行一概不收（✓ 行、Compiling、test … ok）。

run_wrapped 的合同：
  标准输出与标准错误各开一根管道，relay 按到达次序逐块读；每一块先照原样转打到本进程的标准输出或标准错误（--quiet 不转），再按换行切行写进全量日志、
  逐行喂给筛选；最后一行没有换行的，在两份日志里补一个换行收下。被包命令退出码原样交回（被信号杀的按 shell 的写法 128+信号号）。
  本进程收到 SIGTERM、SIGHUP 原样转给被包的命令；SIGINT 本进程什么都不做、也不转（终端上的 Ctrl-C 本来就发给整个前台进程组，被包的命令自己收得到；不用 SIG_IGN，免得命令继承「不理 Ctrl-C」）。
  被包命令退出之后，管道要是被它放到后台的孙进程占着一直不关，再等 DRAIN_SECONDS（环境变量 TWO_LOGS_DRAIN_SECONDS 可改）就收工，不挂住。
  开跑前把两份日志截成空的（旧的不残留）；收尾往 problems 日志写一行「收了几行、全量几行、退出码」，再往标准错误打一行两份日志的路径（--quiet 不打）。
退出码：被包命令的；125 用法不对或这一份自己出了问题（同 env 的约定）；126 命令起不来（没有执行权限）；127 找不到命令。

弄坏开关 TWO_LOGS_BREAK=<项>（逗号分隔，只给自证证红；认不出的项直接拒，退 125）：逐个写在 two-logs.sh 文件头，连同打开后红的格；
自证不设开关时末了跑开关轮，逐个打开、核红的格与 BREAK_RED_CELLS 逐字相等，开关名都要写在 two-logs.sh 文件头里。
"""
import concurrent.futures
import math
import os
import re
import selectors
import signal
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True

DETAIL_CAP = 40                 # 一块 ✗ / ⚠ 之后收的明细与续行至多这么多行，多的只写一行说明
MAXIMUM_PENDING_BYTES = 1 << 20  # 一行没有换行就攒到这么多字节，当一行收下（进度条一类只用回车刷新的输出）
DEFAULT_DRAIN_SECONDS = 2.0
READ_BYTES = 1 << 16
EXIT_USAGE = 125
EXIT_CANNOT_EXECUTE = 126
EXIT_NOT_FOUND = 127

FAIL_MARK = "✗"
WARN_MARK = "⚠"
NOT_RUN_WORDS = "本次未跑"
SUMMARY_HEADER = "── 汇总"
SUMMARY_CELL_LINE = re.compile(r"^\s*[^\s：]+：(?:绿|红(?:（退出 \d+）)?|本次未跑)\s*$")
CARGO_ERROR = re.compile(r"^\s*error(?:\[[A-Za-z0-9]+\])?:")
CARGO_WARNING = re.compile(r"^\s*warning(?:\[[A-Za-z0-9_-]+\])?:")
CARGO_LOCATION = re.compile(r"^\s*-->\s+\S.*:\d+")
LIBTEST_FAILED = re.compile(r"\bFAILED\b")
PANICKED = re.compile(r"\bpanicked\b")
TRACEBACK_HEADER = re.compile(r"^\s*Traceback \(most recent call last\):")
TRACEBACK_CHAIN = re.compile(r"^(?:During handling of the above exception|The above exception was the direct cause)")
EXIT_CODE_LINES = (
    re.compile(r"(?:退出码|退出|退)\s*[:：=]?\s*[1-9][0-9]*(?![0-9])"),
    re.compile(r"\bexit(?:ed)?(?:\s+(?:code|status|with(?:\s+(?:code|status))?))?\s*[:=]?\s*[1-9][0-9]*\b", re.IGNORECASE),
)

FILTER_BREAKS = (
    "fail-mark-ignored", "fail-arrow-dropped", "fail-details-dropped", "warn-mark-ignored", "not-run-ignored", "summary-ignored",
    "cargo-error-ignored", "cargo-warning-ignored", "cargo-location-dropped", "libtest-failed-ignored", "panicked-ignored",
    "traceback-body-dropped", "exit-code-ignored", "collect-everything",
)
RELAY_BREAKS = (
    "line-number-missing", "footer-missing", "exit-code-lost", "stdout-not-forwarded", "stderr-not-forwarded", "stderr-merged-into-stdout",
    "full-log-skips-stderr", "partial-line-dropped", "stale-logs-kept", "signal-not-forwarded", "drain-unbounded", "quiet-forwards", "strict-decoding",
    "sigint-ignored-inherited",
)
ALL_BREAKS = FILTER_BREAKS + RELAY_BREAKS


def parse_breaks(text):
    """逗号分隔的弄坏开关 → frozenset；认不出的项抛 ValueError（拼错的开关不许被当成没开）。"""
    names = frozenset(part.strip() for part in (text or "").split(",") if part.strip())
    unknown = sorted(names - set(ALL_BREAKS))
    if unknown:
        raise ValueError("认不出的弄坏开关 " + "、".join(unknown) + "；认得的在 two_logs.py 文件头")
    return names


def indentation(text):
    return len(text) - len(text.lstrip(" "))


class ProblemFilter:
    """逐行筛。feed(行号, 行) 交回这一行引出的 problems 日志的行（零到两行：本行，加上明细超限时的一行说明）。

    状态只有四样：✗ / ⚠ 块（后面跟着的出路与明细）、cargo 诊断后等 --> 行、Traceback 段。"""

    def __init__(self, breaks=frozenset()):
        self.breaks = breaks
        self.block_open = False        # 在 ✗ / ⚠ 块里
        self.block_detail_lines = 0
        self.block_overflow_noted = False
        self.expect_location = False
        self.in_traceback = False
        self.collected = 0

    def _take(self, number, text, out):
        out.append(f"{number}:{text}")
        self.collected += 1

    def feed(self, number, text):
        out = []
        stripped = text.strip()
        breaks = self.breaks
        if "collect-everything" in breaks:
            self._take(number, text, out)
            return out
        # Traceback 段里：缩进的栈帧行收，收尾的异常行收了就结束（链式的两段接着收），空行结束
        if self.in_traceback:
            if not stripped:
                self.in_traceback = False
            elif text[:1] in (" ", "\t"):
                if "traceback-body-dropped" not in breaks:
                    self._take(number, text, out)
                return out
            else:
                if "traceback-body-dropped" not in breaks:
                    self._take(number, text, out)
                if not TRACEBACK_CHAIN.match(text):
                    self.in_traceback = False
                return out
        # cargo / rustc 诊断后紧跟的位置行
        if self.expect_location:
            self.expect_location = False
            if CARGO_LOCATION.match(text):
                if "cargo-location-dropped" not in breaks:
                    self._take(number, text, out)
                return out
        has_fail = FAIL_MARK in text and "fail-mark-ignored" not in breaks
        has_warn = WARN_MARK in text and "warn-mark-ignored" not in breaks
        if has_fail or has_warn:
            self.block_open = "fail-arrow-dropped" not in breaks or "fail-details-dropped" not in breaks
            self.block_detail_lines = 0
            self.block_overflow_noted = False
            self._take(number, text, out)
            return out
        # ✗ / ⚠ 块里的后续行
        if self.block_open:
            if stripped.startswith("→"):
                if "fail-arrow-dropped" not in breaks:
                    self._take(number, text, out)
                    return out
                return out
            if indentation(text) >= 4 and stripped:
                if "fail-details-dropped" not in breaks:
                    if self.block_detail_lines < DETAIL_CAP:
                        self.block_detail_lines += 1
                        self._take(number, text, out)
                    elif not self.block_overflow_noted:
                        self.block_overflow_noted = True
                        out.append(f"{number}:（这一块的明细与续行超过 {DETAIL_CAP} 行，后面的不收，见全量日志从这一行起）")
                return out
            self.block_open = False
        if (stripped.startswith(SUMMARY_HEADER) or SUMMARY_CELL_LINE.match(text)) and "summary-ignored" not in breaks:
            self._take(number, text, out)
            return out
        if NOT_RUN_WORDS in text and "not-run-ignored" not in breaks:
            self._take(number, text, out)
            return out
        if CARGO_ERROR.match(text) and "cargo-error-ignored" not in breaks:
            self.expect_location = True
            self._take(number, text, out)
            return out
        if CARGO_WARNING.match(text) and "cargo-warning-ignored" not in breaks:
            self.expect_location = True
            self._take(number, text, out)
            return out
        if (LIBTEST_FAILED.search(text) and "libtest-failed-ignored" not in breaks) or (PANICKED.search(text) and "panicked-ignored" not in breaks):
            self._take(number, text, out)
            return out
        if TRACEBACK_HEADER.match(text):
            self.in_traceback = True
            self._take(number, text, out)
            return out
        if "exit-code-ignored" not in breaks and any(pattern.search(text) for pattern in EXIT_CODE_LINES):
            self._take(number, text, out)
            return out
        return out


def filter_problems(lines, breaks=frozenset()):
    """整份全量日志的行 → problems 日志的行（「行号:原行」，行号从 1 起，不含收尾行）。"""
    problem_filter = ProblemFilter(breaks)
    out = []
    for number, text in enumerate(lines, 1):
        out += problem_filter.feed(number, text)
    return out


def footer_text(collected, total, exit_code):
    code = "未知" if exit_code is None else str(exit_code)
    return f"· 错误警告日志到此为止：收了 {collected} 行，全量日志共 {total} 行，被包的命令退出码 {code}"


def log_paths(name):
    return name + ".log", name + ".problems.log"


def write_report_line(full_path, problems_path, total, collected, exit_code=None):
    note = "没有错误警告" if collected == 0 else f"收了 {collected} 行"
    code = f"；被包命令退出码 {exit_code}" if exit_code else ""
    sys.stderr.write(f"  · 日志两份：全量 {full_path}（{total} 行）；错误警告 {problems_path}（{note}）{code}\n")
    sys.stderr.flush()


def write_all(fd, data):
    """整块写到文件描述符；写不下去（管道另一头关了、描述符没开）返回 False，调用方不再往这一路转打，日志照写。"""
    view = memoryview(data)
    while view:
        try:
            written = os.write(fd, view)
        except OSError:
            return False
        view = view[written:]
    return True


def exit_code_of(returncode):
    return 128 - returncode if returncode < 0 else returncode


def run_wrapped(name, command, quiet=False, breaks=frozenset(), drain_seconds=None):
    """包住 command（列表）：转打、写两份日志，交回被包命令的退出码（文件头「run_wrapped 的合同」）。"""
    full_path, problems_path = log_paths(name)
    try:
        drain_seconds = float(os.environ.get("TWO_LOGS_DRAIN_SECONDS", DEFAULT_DRAIN_SECONDS) if drain_seconds is None else drain_seconds)
        if not math.isfinite(drain_seconds) or drain_seconds < 0:
            raise ValueError("not finite or negative")
    except (TypeError, ValueError):
        return usage_error("TWO_LOGS_DRAIN_SECONDS 要是不小于 0 的有限秒数")
    directory = os.path.dirname(os.path.abspath(name))
    try:
        os.makedirs(directory, exist_ok=True)
        mode = "ab" if "stale-logs-kept" in breaks else "wb"
        full = open(full_path, mode)
        problems = open(problems_path, mode)
    except OSError as error:
        sys.stderr.write(f"  ✗ two-logs：建不了日志文件 {full_path} / {problems_path}：{error}\n")
        sys.stderr.write("     → 怎么办：名写成一个可写目录下的前缀（不带 .log 后缀）；盘满了先清出空间。\n")
        return EXIT_USAGE
    problem_filter = ProblemFilter(breaks)
    state = {"lines": 0, "process": None, "early_signals": []}

    def forward_signal(signum, frame):
        process = state["process"]
        if process is None:
            state["early_signals"].append(signum)
            return
        try:
            process.send_signal(signum)
        except ProcessLookupError:
            pass

    previous_handlers = {}
    if "signal-not-forwarded" not in breaks:
        for signum in (signal.SIGTERM, signal.SIGHUP):
            previous_handlers[signum] = signal.signal(signum, forward_signal)
    # SIGINT：本进程什么都不做（终端上的 Ctrl-C 发给整个前台进程组，被包的命令自己收得到）。不能用 SIG_IGN：被忽略的信号 exec 之后仍然被忽略，命令会继承「不理 Ctrl-C」；
    #   Python 的处理函数在 exec 之后恢复默认。弄坏开关 sigint-ignored-inherited：用 SIG_IGN
    previous_handlers[signal.SIGINT] = signal.signal(signal.SIGINT, signal.SIG_IGN if "sigint-ignored-inherited" in breaks else (lambda signum, frame: None))
    exit_code = None
    try:
        try:
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
        except FileNotFoundError:
            sys.stderr.write(f"  ✗ two-logs：找不到命令 {command[0]}\n     → 怎么办：-- 后面写能执行的命令（在 PATH 上，或写路径）。\n")
            exit_code = EXIT_NOT_FOUND
        except PermissionError:
            sys.stderr.write(f"  ✗ two-logs：命令 {command[0]} 没有执行权限\n     → 怎么办：给它执行权限，或前面加 bash / python3。\n")
            exit_code = EXIT_CANNOT_EXECUTE
        else:
            state["process"] = process
            for signum in state["early_signals"]:
                process.send_signal(signum)

            def emit_line(raw, stream):
                if stream == "err" and "full-log-skips-stderr" in breaks:
                    return
                state["lines"] += 1
                full.write(raw + b"\n")
                decoded = raw.decode("utf-8", errors="strict" if "strict-decoding" in breaks else "replace").rstrip("\r")
                for line in problem_filter.feed(state["lines"], decoded):
                    problems.write((line.split(":", 1)[1] if "line-number-missing" in breaks else line).encode("utf-8") + b"\n")

            forward_fds = {"out": 1, "err": 2}
            if "stderr-merged-into-stdout" in breaks:
                forward_fds["err"] = 1
            forwarding = {"out": not quiet or "quiet-forwards" in breaks, "err": not quiet or "quiet-forwards" in breaks}
            if "stdout-not-forwarded" in breaks:
                forwarding["out"] = False
            if "stderr-not-forwarded" in breaks:
                forwarding["err"] = False
            pending = {"out": bytearray(), "err": bytearray()}
            selector = selectors.DefaultSelector()
            selector.register(process.stdout, selectors.EVENT_READ, "out")
            selector.register(process.stderr, selectors.EVENT_READ, "err")
            exited_at = None
            while selector.get_map():
                events = selector.select(timeout=0.2)
                for key, _ in events:
                    stream = key.data
                    chunk = os.read(key.fileobj.fileno(), READ_BYTES)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    if forwarding[stream] and not write_all(forward_fds[stream], chunk):
                        forwarding[stream] = False
                    pending[stream] += chunk
                    while True:
                        newline = pending[stream].find(b"\n")
                        if newline < 0:
                            break
                        raw = bytes(pending[stream][:newline])
                        del pending[stream][:newline + 1]
                        emit_line(raw, stream)
                    if len(pending[stream]) > MAXIMUM_PENDING_BYTES:
                        emit_line(bytes(pending[stream]), stream)
                        pending[stream].clear()
                full.flush()
                problems.flush()
                if process.poll() is not None:
                    if exited_at is None:
                        exited_at = time.monotonic()
                    elif time.monotonic() - exited_at >= drain_seconds and "drain-unbounded" not in breaks:
                        break
            for stream in ("out", "err"):
                if pending[stream] and "partial-line-dropped" not in breaks:
                    emit_line(bytes(pending[stream]), stream)
            process.wait()
            exit_code = exit_code_of(process.returncode)
            for pipe in (process.stdout, process.stderr):
                pipe.close()
    finally:
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)
        if "footer-missing" not in breaks:
            problems.write((footer_text(problem_filter.collected, state["lines"], exit_code) + "\n").encode("utf-8"))
        full.close()
        problems.close()
    if not quiet:
        write_report_line(full_path, problems_path, state["lines"], problem_filter.collected, exit_code)
    if "exit-code-lost" in breaks and exit_code not in (EXIT_NOT_FOUND, EXIT_CANNOT_EXECUTE):
        return 0
    return exit_code


def filter_existing_log(name, breaks=frozenset()):
    """--from-log：读 <名>.log 现筛出 <名>.problems.log（覆盖旧的），打一行路径。"""
    full_path, problems_path = log_paths(name)
    try:
        with open(full_path, "rb") as handle:
            raw_lines = handle.read().split(b"\n")
    except OSError as error:
        sys.stderr.write(f"  ✗ two-logs：读不了全量日志 {full_path}：{error}\n     → 怎么办：名写成 <名>.log 去掉 .log 的前缀，并确认那份日志在。\n")
        return EXIT_USAGE
    if raw_lines and raw_lines[-1] == b"":
        raw_lines.pop()
    lines = [raw.decode("utf-8", errors="replace").rstrip("\r") for raw in raw_lines]
    selected = filter_problems(lines, breaks)
    collected = len(selected) - sum(1 for line in selected if "这一块的明细与续行超过" in line)
    with open(problems_path, "w", encoding="utf-8") as handle:
        handle.write("".join(line + "\n" for line in selected))
        handle.write(footer_text(collected, len(lines), None) + "\n")
    write_report_line(full_path, problems_path, len(lines), collected)
    return 0


USAGE = """用法：
  two-logs.sh [--quiet] <名> -- <命令…>   包住命令：转打，同时写 <名>.log（全量）与 <名>.problems.log（错误警告，行前带全量日志里的行号）
  two-logs.sh --from-log <名>             不跑命令：从已有的 <名>.log 现筛出 <名>.problems.log
  two-logs.sh --selftest                  自证
<名> 是日志路径去掉 .log 的前缀；退出码是被包命令的（125 用法不对，126 起不来，127 找不到）。"""


def usage_error(message):
    sys.stderr.write(f"  ✗ two-logs：{message}\n     → 怎么办：照下面的用法写。\n{USAGE}\n")
    return EXIT_USAGE


def main(argv):
    try:
        breaks = parse_breaks(os.environ.get("TWO_LOGS_BREAK", ""))
    except ValueError as error:
        sys.stderr.write(f"  ✗ two-logs：{error}\n     → 怎么办：TWO_LOGS_BREAK 只给自证证红用，不设就不会拒。\n")
        return EXIT_USAGE
    if argv[:1] in (["-h"], ["--help"]):
        sys.stdout.write(USAGE + "\n")
        return 0
    if argv[:1] == ["--selftest"]:
        return selftest(breaks)
    if argv[:1] == ["--from-log"]:
        if len(argv) != 2 or argv[1].endswith(".log"):
            return usage_error("--from-log 后面只写一个名（日志路径去掉 .log）")
        return filter_existing_log(argv[1], breaks)
    quiet = False
    rest = list(argv)
    if rest[:1] == ["--quiet"]:
        quiet = True
        rest = rest[1:]
    if "--" not in rest:
        return usage_error("缺 --：<名> 与命令之间要隔一个 --")
    separator = rest.index("--")
    names, command = rest[:separator], rest[separator + 1:]
    if len(names) != 1 or not names[0]:
        return usage_error("-- 前面要正好一个名")
    if names[0].endswith(".log"):
        return usage_error(f"名「{names[0]}」不要带 .log 后缀（会写成 {names[0]}.log.log）；去掉后缀再写")
    if not command:
        return usage_error("-- 后面没有命令")
    return run_wrapped(names[0], command, quiet, breaks)


# ── 自证 ──────────────────────────────────────────────────────────────────────

def numbered(lines, picked):
    """picked：1 起的行号列表 → 期望的 problems 行。"""
    return [f"{number}:{lines[number - 1]}" for number in picked]


def build_filter_cells():
    """[(格名, 输入行, 期望的 problems 行 或 (期望, 必须不在里面的行号))]；每格一份答案已知的合成输入。"""
    cells = []

    def cell(label, lines, picked):
        cells.append((label, lines, numbered(lines, picked)))

    gate_block = ["  ✓ 前面一格过了", "  ✗ 红了一处 foo", "      src/lib.rs:3 明细一", "      src/lib.rs:9 明细二",
                  "     → 怎么办：改 foo", "       续行说明", "  ✓ 后面一格过了"]
    cell("✗ 行连同明细与出路", gate_block, [2, 3, 4, 5, 6])
    arrows = ["  ✗ 摘要", "     → 怎么办：先这样", "     → 再这样", "  普通输出", "     → 孤零零的出路"]
    cell("✗ 之后的 → 出路行到下一处非 → 续行为止", arrows, [1, 2, 3])
    cell("⚠ 行连同出路", ["  ✓ 一", "  ⚠ 慢了", "     → 怎么办：加线程", "  ✓ 二"], [2, 3])
    cell("本次未跑的行", ["  ✓ 一", "  ! 阶段 X：这一道本次未跑（不记通过）", "  ✓ 二"], [2])
    cell("逐格汇总段", ["── 格甲：标题", "  过程输出", "── 汇总：demo.sh 跑了 3 格（全部）", "  cell-a：绿", "  cell-b：红（退出 1）", "  cell-c：本次未跑", "  ✓ 总结"],
         [3, 4, 5, 6])
    cell("cargo error 连同 --> 行", ["   Compiling demo v0.1.0", "error[E0425]: cannot find value `x` in this scope", " --> src/lib.rs:3:5", "  |", "3 |     x", "  |     ^ not found"],
         [2, 3])
    cell("cargo warning 连同 --> 行", ["warning: unused variable: `y`", " --> src/main.rs:2:9", "  |", "warning: `demo` (bin) generated 1 warning", "    Finished dev"], [1, 2, 4])
    cell("libtest FAILED 与 panicked", ["test alpha ... ok", "test beta ... FAILED", "thread 'beta' panicked at src/x.rs:1:1:", "test result: FAILED. 1 passed; 1 failed"], [2, 3, 4])
    cell("Python Traceback 整段", ["正常一行", "Traceback (most recent call last):", '  File "x.py", line 1, in <module>', "    foo()", "ValueError: bad", "后面的正常行"], [2, 3, 4, 5])
    cell("报非 0 退出码的行", ["内存包装退 252", "process didn't exit successfully (exit status: 101)", "退出码 0", "exit=0", "（退出 1）", "exit code: 2"], [1, 2, 5, 6])
    cell("绿行与普通输出一行不收", ["   Compiling a v0.1.0", "test x ... ok", "  ✓ 全部通过", "REPLAY_TOTAL dispatched=3 collected=3", "──  格乙：标题"], [])
    overflow = ["  ✗ 很多明细"] + [f"      明细 {index}" for index in range(1, DETAIL_CAP + 11)] + ["     → 怎么办：看全量日志"]
    expected_overflow = [f"1:{overflow[0]}"] + [f"{index + 1}:{overflow[index]}" for index in range(1, DETAIL_CAP + 1)]
    expected_overflow.append(f"{DETAIL_CAP + 2}:（这一块的明细与续行超过 {DETAIL_CAP} 行，后面的不收，见全量日志从这一行起）")
    expected_overflow.append(f"{len(overflow)}:{overflow[-1]}")
    cells.append(("明细超过上限只收到上限并说明", overflow, expected_overflow))
    return cells


def run_filter_cells(breaks):
    """交回红了的格名（有序列表）与每格的明细。"""
    red, details = [], {}
    for label, lines, expected in build_filter_cells():
        actual = filter_problems(lines, breaks)
        if actual != expected:
            red.append(label)
            details[label] = f"期望 {expected!r}，实际 {actual!r}"
    return red, details


def run_command_cell(arguments, environment_overrides=None, timeout=20):
    environment = dict(os.environ)
    environment.pop("TWO_LOGS_BREAK", None)
    environment.update(environment_overrides or {})
    return subprocess.run([sys.executable, "-B", os.path.abspath(__file__)] + arguments, capture_output=True, env=environment, timeout=timeout)


SLEEP_SIXTY = b"sleep\x0060\x00"     # 自证里 exec 出来的那条 sleep 60 的 /proc/<进程号>/cmdline
SLEEP_FOUR = b"sleep\x004\x00"


def read_bytes(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except OSError:
        return None


def own_process_alive(pid, expected_arguments):
    """进程号 pid 还活着、而且命令行正是自证自己起的那一条（进程号被别人复用了不算）。"""
    data = read_bytes(f"/proc/{pid}/cmdline")
    return data is not None and data == expected_arguments


def stop_own_process(pid, expected_arguments):
    """只停命令行逐字等于自证自己起的那一条的进程（按点名的进程号，不按名字找）。"""
    if own_process_alive(pid, expected_arguments):
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def relay_cells(scratch, breaks):
    """[(格名, 函数)]：函数交回空串（过了）或没过的原因。被测的是另起的一个进程，开关经环境变量 TWO_LOGS_BREAK 交给它。"""
    environment = {"TWO_LOGS_BREAK": ",".join(sorted(breaks))} if breaks else {}
    directory = scratch
    shim = os.path.join(os.path.dirname(os.path.abspath(__file__)), "two-logs.sh")

    def wrapped(name, shell_text, quiet=False, extra_environment=None, timeout=20):
        arguments = (["--quiet"] if quiet else []) + [os.path.join(directory, name), "--", "bash", "-c", shell_text]
        combined = dict(environment)
        combined.update(extra_environment or {})
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__)] + arguments, capture_output=True, env={**os.environ, **combined}, timeout=timeout)
        return completed

    def log_lines(name, suffix):
        data = read_bytes(os.path.join(directory, name + suffix))
        return None if data is None else data.decode("utf-8", errors="replace").splitlines()

    cells = []
    red_mark = chr(0x2717)   # 假命令打的红行开头的叉号；拼进下面的 shell 文本，字面量不写进这些检查行里

    def forwarding_and_full_log():
        completed = wrapped("forward", "echo out-one; echo err-one >&2; echo out-two; echo err-two >&2; printf 'out-three\\n'")
        if completed.stdout != b"out-one\nout-two\nout-three\n":
            return f"标准输出应当原样转打，实际 {completed.stdout!r}"
        if not completed.stderr.startswith(b"err-one\nerr-two\n"):
            return f"标准错误应当原样转打（以 err-one、err-two 起头，后面接两份日志的路径行），实际 {completed.stderr!r}"
        full = log_lines("forward", ".log")
        if full is None or sorted(full) != sorted(["out-one", "err-one", "out-two", "err-two", "out-three"]):
            return f"全量日志应当收齐标准输出与标准错误共五行，实际 {full!r}"
        if [line for line in full if line.startswith("out-")] != ["out-one", "out-two", "out-three"]:
            return f"全量日志里标准输出的次序不对，实际 {full!r}"
        return ""

    cells.append(("标准输出与标准错误各自照原样转打、全量日志收全", forwarding_and_full_log))

    def exit_code_passes():
        for code in (0, 7):
            completed = wrapped(f"exit{code}", f"exit {code}")
            if completed.returncode != code:
                return f"被包命令退 {code}，two-logs 退了 {completed.returncode}"
        return ""

    cells.append(("退出码原样交回", exit_code_passes))

    def problems_with_numbers_and_footer():
        wrapped("numbers", f"echo 普通一; sleep 0.2; echo 'error: boom' >&2; sleep 0.2; echo '  {red_mark} 红了'; echo 普通四")
        problems = log_lines("numbers", ".problems.log")
        if problems is None:
            return "没有 problems 日志"
        if problems[:2] != ["2:error: boom", "3:  ✗ 红了"]:
            return f"problems 日志的行应当是「行号:原行」（error 行是全量日志的 2 行，✗ 行是 3 行），实际 {problems!r}"
        if "收了 2 行" not in problems[-1] or "全量日志共 4 行" not in problems[-1] or "退出码 0" not in problems[-1]:
            return f"末行应当写收了 2 行、全量日志共 4 行、退出码 0，实际 {problems[-1:]!r}"
        return ""

    cells.append(("problems 日志的行带行号、末行写收了几行全量几行", problems_with_numbers_and_footer))

    def partial_last_line():
        wrapped("partial", f"printf 'first\\n  {red_mark} 没有换行的最后一行'")
        full = read_bytes(os.path.join(directory, "partial.log"))
        problems = log_lines("partial", ".problems.log")
        if full != "first\n  ✗ 没有换行的最后一行\n".encode("utf-8"):
            return f"全量日志应当把最后一行补上换行收下，实际 {full!r}"
        if not problems or problems[0] != "2:  ✗ 没有换行的最后一行":
            return f"problems 日志应当收到最后一行（行号 2），实际 {problems!r}"
        return ""

    cells.append(("没有换行的最后一行", partial_last_line))

    def stale_logs_removed():
        for suffix in (".log", ".problems.log"):
            with open(os.path.join(directory, "stale" + suffix), "wb") as handle:
                handle.write("旧的残留\n".encode("utf-8"))
        wrapped("stale", "echo 新的")
        for suffix in (".log", ".problems.log"):
            data = read_bytes(os.path.join(directory, "stale" + suffix)) or b""
            if "旧的残留".encode("utf-8") in data:
                return f"开跑前应当把 stale{suffix} 截空，实际还留着旧内容"
        return ""

    cells.append(("旧日志不残留", stale_logs_removed))

    def non_utf8_bytes():
        completed = wrapped("binary", "printf '\\377\\376 bytes\\n  ✗ 后面还有\\n'")
        if completed.stdout != b"\xff\xfe bytes\n  \xe2\x9c\x97 \xe5\x90\x8e\xe9\x9d\xa2\xe8\xbf\x98\xe6\x9c\x89\n":
            return f"非 UTF-8 字节应当原样转打，实际 {completed.stdout!r}（返回码 {completed.returncode}，标准错误 {completed.stderr[-200:]!r}）"
        if read_bytes(os.path.join(directory, "binary.log")) != completed.stdout:
            return "全量日志应当与转打的字节逐字节相同"
        problems = log_lines("binary", ".problems.log") or []
        if not problems or not problems[0].startswith("2:"):
            return f"后面那一行的 ✗ 应当照常收到（行号 2），实际 {problems!r}"
        return ""

    cells.append(("非 UTF-8 字节原样转打", non_utf8_bytes))

    def quiet_writes_only_logs():
        completed = wrapped("quiet", "echo 只进日志; echo 标准错误也只进日志 >&2", quiet=True)
        if completed.stdout or completed.stderr:
            return f"--quiet 不应当转打，实际标准输出 {completed.stdout!r}、标准错误 {completed.stderr!r}"
        full = log_lines("quiet", ".log")
        if full is None or "只进日志" not in full:
            return f"--quiet 也要写全量日志，实际 {full!r}"
        return ""

    cells.append(("--quiet 只写日志不转打", quiet_writes_only_logs))

    def signal_goes_to_command():
        pid_file = os.path.join(directory, "signal-child.pid")
        command = [sys.executable, "-B", os.path.abspath(__file__), os.path.join(directory, "signal"), "--", "bash", "-c", f"echo $$ > '{pid_file}'; exec sleep 60"]
        process = subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env={**os.environ, **environment})
        child_pid = None
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline and child_pid is None:
                pid_text = (read_bytes(pid_file) or b"").decode().strip()
                if pid_text.isdigit() and own_process_alive(int(pid_text), SLEEP_SIXTY):
                    child_pid = int(pid_text)
                else:
                    time.sleep(0.05)
            if child_pid is None:
                return "被包的命令 10 秒内没有起来"
            process.send_signal(signal.SIGTERM)
            try:
                returncode = process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
                return "two-logs 收到 SIGTERM 之后 10 秒内没有退出（SIGTERM 没转给被包的命令）"
            deadline = time.monotonic() + 5
            alive = True
            while time.monotonic() < deadline:
                if not own_process_alive(child_pid, SLEEP_SIXTY):
                    alive = False
                    break
                time.sleep(0.05)
            if alive:
                return f"two-logs 已经退出（退 {returncode}），被包的命令（进程号 {child_pid}）还活着：SIGTERM 没转给它"
            return ""
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            if child_pid is not None:
                stop_own_process(child_pid, SLEEP_SIXTY)

    cells.append(("SIGTERM 转给被包的命令", signal_goes_to_command))

    def sigint_stays_default_in_command():
        baseline = subprocess.run(["grep", "SigIgn", "/proc/self/status"], capture_output=True, timeout=20).stdout.decode().split()
        if baseline and int(baseline[-1], 16) & (1 << (signal.SIGINT - 1)):
            return ""   # 自证自己就是在忽略 SIGINT 的环境里起的（后台作业），分不出继承来的，这一格不判
        wrapped("sigint", "grep SigIgn /proc/self/status")
        logged = (log_lines("sigint", ".log") or [""])[0]   # 读全量日志，不读转打的输出：转打坏了不连累这一格
        words = logged.split()
        if not words or int(words[-1], 16) & (1 << (signal.SIGINT - 1)):
            return f"被包的命令里 SIGINT 被忽略了（SigIgn 一行是 {logged!r}）：two-logs 自己不理 SIGINT 不能让命令继承「不理」"
        return ""

    cells.append(("被包的命令的 SIGINT 仍是默认处置", sigint_stays_default_in_command))

    def background_grandchild_holds_pipe():
        pid_file = os.path.join(directory, "grandchild.pid")
        started = time.monotonic()
        completed = wrapped("grandchild", f"echo 命令退出了; (sleep 4 & echo $! > '{pid_file}')", extra_environment={"TWO_LOGS_DRAIN_SECONDS": "0.5"}, timeout=30)
        elapsed = time.monotonic() - started
        try:
            grandchild = int((read_bytes(pid_file) or b"0").decode().strip() or 0)
        except ValueError:
            grandchild = 0
        if grandchild:
            stop_own_process(grandchild, SLEEP_FOUR)
        if completed.returncode != 0:
            return f"退出码应当是 0，实际 {completed.returncode}"
        if elapsed > 2.5:
            return f"被包命令退出后管道被后台孙进程占着（它再活 4 秒），two-logs 应当等 0.5 秒就收工，实际等了 {elapsed:.1f} 秒"
        return ""

    cells.append(("后台孙进程占着管道不挂住", background_grandchild_holds_pipe))

    def shim_passes_arguments_and_exit_code():
        environment_for_shim = {**os.environ, **environment}
        completed = subprocess.run(["bash", shim, os.path.join(directory, "shim"), "--", "bash", "-c", "echo 经入口脚本; exit 3"], capture_output=True, env=environment_for_shim, timeout=60)
        if completed.returncode != 3 or not completed.stdout.startswith("经入口脚本".encode("utf-8")):
            return f"two-logs.sh 应当转打并交回被包命令的退出码 3，实际退 {completed.returncode}，标准输出 {completed.stdout!r}，标准错误 {completed.stderr[-300:]!r}"
        return ""

    cells.append(("入口脚本 two-logs.sh 转参数、交退出码", shim_passes_arguments_and_exit_code))

    def refusals():
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), os.path.join(directory, "x.log"), "--", "true"],
                                   capture_output=True, env={**os.environ, **environment}, timeout=20)
        if completed.returncode != 125 or "不要带 .log 后缀" not in completed.stderr.decode("utf-8", errors="replace"):
            return f"名带 .log 后缀应当退 125 并写明出路，实际退 {completed.returncode}：{completed.stderr!r}"
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), os.path.join(directory, "nope"), "--", "/nonexistent/command-for-two-logs"],
                                   capture_output=True, env={**os.environ, **environment}, timeout=20)
        if completed.returncode != 127:
            return f"找不到命令应当退 127，实际 {completed.returncode}"
        return ""

    cells.append(("用法不对与找不到命令", refusals))

    def from_log():
        with open(os.path.join(directory, "existing.log"), "wb") as handle:
            handle.write("一\n  ✗ 红\n三\n".encode("utf-8"))
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "--from-log", os.path.join(directory, "existing")], capture_output=True,
                                   env={**os.environ, **environment}, timeout=20)
        problems = log_lines("existing", ".problems.log")
        if completed.returncode != 0 or problems is None or problems[:1] != ["2:  ✗ 红"] or "收了 1 行" not in problems[-1] or "全量日志共 3 行" not in problems[-1]:
            return f"--from-log 应当现筛出 problems 日志（行号 2、收了 1 行、全量 3 行），实际退 {completed.returncode}：{problems!r}"
        return ""

    cells.append(("--from-log 现筛已有日志", from_log))
    return cells


# 开关 → 打开后必须红的格（逐字等于上面 relay_cells / FILTER_CELLS 的格名；文件头逐个写了）
BREAK_RED_CELLS = {
    "fail-mark-ignored": ["✗ 行连同明细与出路", "✗ 之后的 → 出路行到下一处非 → 续行为止", "明细超过上限只收到上限并说明"],
    "fail-arrow-dropped": ["✗ 行连同明细与出路", "✗ 之后的 → 出路行到下一处非 → 续行为止", "⚠ 行连同出路", "明细超过上限只收到上限并说明"],
    "fail-details-dropped": ["✗ 行连同明细与出路", "明细超过上限只收到上限并说明"],
    "warn-mark-ignored": ["⚠ 行连同出路"],
    "not-run-ignored": ["本次未跑的行"],
    "summary-ignored": ["逐格汇总段"],
    "cargo-error-ignored": ["cargo error 连同 --> 行"],
    "cargo-warning-ignored": ["cargo warning 连同 --> 行"],
    "cargo-location-dropped": ["cargo error 连同 --> 行", "cargo warning 连同 --> 行"],
    "libtest-failed-ignored": ["libtest FAILED 与 panicked"],
    "panicked-ignored": ["libtest FAILED 与 panicked"],
    "traceback-body-dropped": ["Python Traceback 整段"],
    "exit-code-ignored": ["报非 0 退出码的行"],
    "collect-everything": None,   # 在 expected_red_cells 里现算：每一格都红
    "line-number-missing": ["problems 日志的行带行号、末行写收了几行全量几行", "没有换行的最后一行", "非 UTF-8 字节原样转打"],
    "footer-missing": ["problems 日志的行带行号、末行写收了几行全量几行"],
    "exit-code-lost": ["退出码原样交回", "入口脚本 two-logs.sh 转参数、交退出码"],
    "stdout-not-forwarded": ["标准输出与标准错误各自照原样转打、全量日志收全", "非 UTF-8 字节原样转打", "入口脚本 two-logs.sh 转参数、交退出码"],
    "stderr-not-forwarded": ["标准输出与标准错误各自照原样转打、全量日志收全"],
    "stderr-merged-into-stdout": ["标准输出与标准错误各自照原样转打、全量日志收全"],
    "full-log-skips-stderr": ["标准输出与标准错误各自照原样转打、全量日志收全", "problems 日志的行带行号、末行写收了几行全量几行"],
    "partial-line-dropped": ["没有换行的最后一行"],
    "stale-logs-kept": ["旧日志不残留"],
    "signal-not-forwarded": ["SIGTERM 转给被包的命令"],
    "drain-unbounded": ["后台孙进程占着管道不挂住"],
    "quiet-forwards": ["--quiet 只写日志不转打"],
    "strict-decoding": ["非 UTF-8 字节原样转打"],
    "sigint-ignored-inherited": ["被包的命令的 SIGINT 仍是默认处置"],
}


def run_relay_cells(breaks):
    red, details = [], {}
    with tempfile.TemporaryDirectory(prefix="two-logs-selftest-") as scratch:
        for label, function in relay_cells(scratch, breaks):
            try:
                reason = function()
            except subprocess.TimeoutExpired as error:
                reason = f"挂住了（{error.timeout} 秒没结束）"
            if reason:
                red.append(label)
                details[label] = reason
    return red, details


def header_text():
    sibling = os.path.join(os.path.dirname(os.path.abspath(__file__)), "two-logs.sh")
    return (read_bytes(sibling) or b"").decode("utf-8", errors="replace")


def expected_red_cells(break_name):
    if break_name == "collect-everything":
        return sorted(label for label, _lines, _expected in build_filter_cells())
    return sorted(BREAK_RED_CELLS[break_name])


def selftest(breaks):
    failures = []

    def fail(label, reason):
        failures.append(label)
        sys.stdout.write(f"  ✗ two_logs 自证：{label}：{reason}\n")  # gate-lint:detail

    filter_labels = [label for label, _lines, _expected in build_filter_cells()]
    relay_labels = []
    with tempfile.TemporaryDirectory(prefix="two-logs-labels-") as scratch:
        relay_labels = [label for label, _function in relay_cells(scratch, frozenset())]
    known_labels = set(filter_labels) | set(relay_labels)
    unknown = [(name, label) for name in ALL_BREAKS for label in expected_red_cells(name) if label not in known_labels]
    if unknown:
        fail("开关表里的格名都存在", f"BREAK_RED_CELLS 点名了不存在的格：{unknown!r}")
    text = header_text()
    for name in ALL_BREAKS:
        if f"  {name} " not in text and f"  {name}\n" not in text:
            fail("每个弄坏开关都写在 two-logs.sh 文件头", f"开关 {name} 没有写在 two-logs.sh 的文件头")
    for name in ALL_BREAKS:
        found = re.search(r"^#\s+" + re.escape(name) + r"\s.*\n#\s+红：(.*)$", text, re.M)
        if found is None:
            fail("每个弄坏开关都写在 two-logs.sh 文件头", f"开关 {name} 的下一行没有「红：…」")
        elif name != "collect-everything" and sorted(re.findall(r"「([^」]*)」", found.group(1))) != expected_red_cells(name):
            fail("文件头写的红格与 BREAK_RED_CELLS 一致", f"开关 {name}：文件头写 {sorted(re.findall(r'「([^」]*)」', found.group(1)))!r}，表里是 {expected_red_cells(name)!r}")
    for name in ALL_BREAKS:
        if name not in BREAK_RED_CELLS:
            fail("每个弄坏开关都有点名的红格", f"开关 {name} 没有登记在 BREAK_RED_CELLS")
    # 正常一轮：每一格都过
    filter_red, filter_details = run_filter_cells(breaks)
    relay_red, relay_details = run_relay_cells(breaks)
    for label in filter_red:
        fail(label, filter_details[label])
    for label in relay_red:
        fail(label, relay_details[label])
    cells_checked = len(filter_labels) + len(relay_labels)
    if breaks:
        sys.stdout.write(f"  · 设了弄坏开关 {','.join(sorted(breaks))}：红的格 {sorted(set(filter_red + relay_red))!r}（开关轮不跑）\n")
        return 1 if failures else 0
    # 开关轮：逐个打开，红的格与 BREAK_RED_CELLS 逐字相等
    round_checked = 0

    def run_one(name):
        opened = frozenset([name])
        if name in FILTER_BREAKS:
            return name, run_filter_cells(opened)[0]
        return name, run_relay_cells(opened)[0]

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
        for name, red in executor.map(run_one, ALL_BREAKS):
            round_checked += 1
            if sorted(set(red)) != expected_red_cells(name):
                fail(f"弄坏开关 {name}", f"打开后红的格应当是 {expected_red_cells(name)!r}，实际 {sorted(set(red))!r}")
    if failures:
        sys.stdout.write(f"  ✗ two_logs 自证没过：{len(failures)} 处（上面逐条列出）\n")  # gate-lint:summary
        sys.stdout.write("  → 怎么办：正常一轮红的格，修筛选或 relay 让它过；开关轮对不上的，是开关没弄坏它点名的那一格（改开关）或弄坏了别的格（改 BREAK_RED_CELLS 与文件头）。\n")
        return 1
    sys.stdout.write(f"  ✓ two_logs 自证通过（查了 {cells_checked} 格：筛选 {len(filter_labels)} 格、relay {len(relay_labels)} 格；开关轮 {round_checked} 个开关逐个打开，红的格都与点名的相等）\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
