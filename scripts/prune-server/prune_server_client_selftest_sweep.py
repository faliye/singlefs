"""prune_server_client_selftest.py 的一块：弄坏开关轮与整份自证各格并行的子进程（run_sweep_job、run_break_sweep、judge_cells_in_children、--cells 的解析、格名到函数的表 selftest_cell_functions），以及测它们的三格（sweep-child-runs-only-named-cells、sweep-stops-hung-child、cells-run-in-parallel-children）。入口 prune_server_client_selftest.py 导入并再导出这里的名字。"""
import concurrent.futures
import contextlib
import dataclasses
import os
import secrets
import shutil
import signal
import subprocess
import sys
import threading
import time
from prune_server_client import break_is
from prune_server_client_selftest_service import (
    CHILD_TIMEOUT_SECONDS, ENTRY_SCRIPT_PATH, SERVER_READY_SECONDS, cell_binary_not_built_without_target_directory, selftest_root
)
from prune_server_client_selftest_cells import (
    STORAGE_FAILURE_LINE_SECONDS, cell_batched_read_answer_over_frame_limit_split, cell_busy_retried_after_delay, cell_command_line_read,
    cell_connect_bounded_by_reconnect_total, cell_connection_variables_refused_outside_temporary_service,
    cell_conclusion_kinds_match_the_verdict_store, cell_delete_by_item_name_picks_by_value, cell_delete_conclusions_by_kind,
    cell_delete_conclusions_refuses_a_source_that_differs, cell_delete_conclusions_refuses_while_claims_are_live,
    cell_delete_prefix_by_namespace, cell_families_over_service,
    cell_gpu_claim_listing_lists_live_claims, cell_gpu_conclusion_write_guarded, cell_gpu_pending_rows_written_only_if_absent,
    cell_large_value_round_trip, cell_namespace_only_under_temporary_service, cell_oversize_frame_reported_at_once, cell_prefix_scan,
    cell_prefix_scan_continues_after_the_last_key, cell_round_trip_unix, cell_scan_row_over_frame_refused,
    cell_segment_count_over_index_digits_refused, cell_segment_key_shapes_only_reserved, cell_segmented_read_incomplete_refused,
    cell_segmented_read_sees_a_rewrite, cell_segmented_rewrite_survives_writer_death, cell_service_problem_judges_given_environment,
    cell_stopped_exits_75, cell_tcp_round_trip, cell_temporary_service_binds_the_address_of_this_host,
    cell_temporary_service_exits_with_parent, cell_temporary_service_refuses_without_readable_hosts, cell_unreachable_exits_75,
    cell_unreachable_family_exits_75, cell_write_storage_failure_replayed, process_state_and_start
)


CELLS = ("round-trip-unix", "prefix-scan", "tcp-round-trip", "busy-retried-after-delay",
         "stopped-exits-75", "unreachable-exits-75", "families-over-service", "unreachable-family-exits-75",
         "connect-bounded-by-reconnect-total", "large-value-round-trip", "segmented-read-incomplete-refused",
         "segmented-read-sees-a-rewrite", "segmented-rewrite-survives-writer-death", "segment-count-over-index-digits-refused",
         "scan-row-over-frame-refused", "prefix-scan-continues-after-the-last-key", "segment-key-shapes-only-reserved", "oversize-frame-reported-at-once", "temporary-service-exits-with-parent", "service-problem-judges-given-environment",
         "connection-variables-refused-outside-temporary-service", "command-line-read", "namespace-only-under-temporary-service",
         "batched-read-answer-over-frame-limit-split", "binary-not-built-without-target-directory", "delete-prefix-by-namespace",
         "delete-by-item-name-picks-by-value", "delete-conclusions-by-kind", "delete-conclusions-refuses-while-claims-are-live",
         "delete-conclusions-refuses-a-source-that-differs", "conclusion-kinds-match-the-verdict-store", "write-storage-failure-replayed",
         "gpu-pending-rows-written-only-if-absent", "gpu-conclusion-write-guarded", "gpu-claim-listing-lists-live-claims",
         "temporary-service-binds-the-address-of-this-host", "temporary-service-refuses-without-readable-hosts",
         "sweep-child-runs-only-named-cells", "sweep-stops-hung-child", "cells-run-in-parallel-children")

SWEEP_CHILD_SECONDS = 2 * (SERVER_READY_SECONDS + STORAGE_FAILURE_LINE_SECONDS + CHILD_TIMEOUT_SECONDS)   # 弄坏开关轮里一个开关的子进程、整份趟里一格的子进程的时限：各格里内部时限合计最长的一格（write-storage-failure-replayed：起服务、等故障日志、收写线程）的两倍
SWEEP_STOP_GRACE_SECONDS = 5           # 同一处：对整组发 TERM 之后等多久再发 KILL，也是发 KILL 之后核进程都没了、收尾输出各等多久
SWEEP_TOKEN_VARIABLE = "PRUNE_CLIENT_SELFTEST_SWEEP_JOB"   # 开关子进程环境里的标记表（逗号隔开，后代继承）：收尾按它找出换了进程组、换了会话、被收养的后代
SWEEP_CELL_LIMIT_SECONDS = 3           # sweep-stops-hung-child：假卡死的子进程的时限
SWEEP_CELL_GRACE_SECONDS = 2           # 同一格：宽限
SWEEP_CELL_EXIT_SLACK_SECONDS = 5      # 同一格：卡死的那一个从起到记完要在时限加两倍宽限再加这么多秒里（起 python 子进程、核 /proc）
SWEEP_CELL_RETURN_SLACK_SECONDS = 15   # 同一格：整轮要在时限加两倍宽限再加这么多秒里回来，没回来就判红，留下的假子进程点名列出、等它们自己退
SWEEP_FAKE_BACKSTOP_SECONDS = 15       # 同一格：假进程的自退时限（比正常的一轮长、比整轮回来的时限短）
# sweep-stops-hung-child 的假进程，命令行参数交代模式与自退时限：忽略 TERM；开关子进程的临时目录（TMPDIR）没了、或到自退时限就退；
# 模式 parent 另外盯着起它的进程，那个进程没了就退。
SWEEP_HOLDER = """import os, signal, sys, time
signal.signal(signal.SIGTERM, signal.SIG_IGN)
mode, backstop, parent, directory = sys.argv[1], float(sys.argv[2]), os.getppid(), os.environ["TMPDIR"]
deadline = time.monotonic() + backstop
while time.monotonic() < deadline and os.path.isdir(directory) and (mode != "parent" or os.getppid() == parent):
    time.sleep(0.05)
"""
# 假卡死子进程：先起两个孙进程（一个同组、模式 directory；一个自己一个会话、模式照命令行），再打出临时目录，自己也按 SWEEP_HOLDER 的退法等着（到点没被停才会等满）
SWEEP_HUNG_CHILD = """import os, subprocess, sys
backstop, mode, holder = sys.argv[1:4]
subprocess.Popen([sys.executable, "-c", holder, "directory", backstop])
subprocess.Popen([sys.executable, "-c", holder, mode, backstop], start_new_session=True)
print("tmpdir=" + os.environ["TMPDIR"], flush=True)
sys.argv = ["-c", "directory", backstop]
exec(holder)
"""


PARALLEL_CELL_LIMIT_SECONDS = 30       # cells-run-in-parallel-children：假的格子进程的时限
FAKE_CELL_RENDEZVOUS_SECONDS = 8       # 同一格：假的格子进程等齐其余几个的上限（一路一路跑时头一个等不到，到点记没等齐）
FAKE_CELL_RELEASE_GAP_SECONDS = 1.0    # 同一格：等齐之后各假子进程按派出的次序倒着、隔这么久一个地退出（跑完的次序与派出的次序相反）
# cells-run-in-parallel-children 的假格子进程，命令行参数：格名、等齐用的目录（空串不等）、要等齐的个数、等齐上限秒数、等齐之后的停留秒数、
# 模式（green 打 ✓ 行与一格全绿的末行；red 打 ✗ 行与一格红的末行；silent 什么都不打）、退出码。
FAKE_CELL_CHILD = """import os, sys, time
name, directory, count, wait, delay, mode, code = sys.argv[1:8]
met = 1
if directory:
    open(os.path.join(directory, name), "w").close()
    deadline = time.monotonic() + float(wait)
    while len(os.listdir(directory)) < int(count) and time.monotonic() < deadline:
        time.sleep(0.02)
    met = int(len(os.listdir(directory)) >= int(count))
time.sleep(float(delay))
if mode == "green":
    print(f"  ✓ 自证 {name}：met={met} pid={os.getpid()} group={os.getpgrp()} tmpdir={os.environ['TMPDIR']}")
    print("PRUNE_SERVER_CLIENT_SELFTEST cells=1 passed=1 failed=0 failed_cells=-")
elif mode == "red":
    print(f"  ✗ 自证 {name}：红了")
    print(f"PRUNE_SERVER_CLIENT_SELFTEST cells=1 passed=0 failed=1 failed_cells={name}")
sys.exit(int(code))
"""


def sweep_hung_child_command():
    """假卡死子进程的命令行。换了会话的孙进程盯着起它的子进程（模式 parent），子进程没了它就退；
    弄坏开关 sweep-descendants-left 下它不盯（模式 directory），子进程没了它还留着，直到临时目录没了。"""
    mode = "directory" if break_is("sweep-descendants-left") else "parent"
    return [sys.executable, "-c", SWEEP_HUNG_CHILD, str(SWEEP_FAKE_BACKSTOP_SECONDS), mode, SWEEP_HOLDER]


@dataclasses.dataclass(frozen=True)
class SweepJob:
    """一个子进程的活：name 是开关名（整份趟里是格名），red_cells 是打开后要红的格，command 与 environment 是它的子进程。
    must_pass 为真时不核红格，核的是子进程退 0（整份趟里一格一个子进程）。"""
    name: str
    red_cells: tuple
    command: list
    environment: dict
    must_pass: bool = False


@dataclasses.dataclass(frozen=True)
class SweepOutcome:
    """一个开关的结果：matched 是红在点名的格且没有留下进程，detail 是没对上的原因（对上的写空串），output 是子进程的标准输出，
    stopped_in_group 是到点按进程组停掉的进程数，outside_seen 是到点时进程组外带标记的进程数，left 是宽限之后还留着的带标记的进程数（点名写在 detail 里，不停），
    seconds 是从起到记完的用时，returncode 是子进程的退出码（起不来为 None），error 是它的标准错误。"""
    name: str
    matched: bool
    detail: str
    output: str
    stopped_in_group: int
    outside_seen: int
    left: int
    seconds: float
    returncode: object = None
    error: str = ""


def wait_until(condition, seconds):
    """每 0.05 秒问一次 condition：成立交 True，过了 seconds 秒还不成立交 False。"""
    deadline = time.monotonic() + seconds
    while True:
        if condition():
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(0.05)


def live_group_members(group):
    """/proc 里进程组号是 group、又不是僵尸的进程号。"""
    members = []
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        if ")" not in text:
            continue   # 读的当口进程退出了，stat 是空的
        fields = text[text.rindex(")") + 2:].split()
        if fields[2] == str(group) and fields[0] not in ("Z", "X"):
            members.append(int(name))
    return members


def job_token(root, index):
    return f"{root}:{index}"


def marked_processes(token):
    """环境里 SWEEP_TOKEN_VARIABLE 的标记表含 token、又不是僵尸的进程：[(进程号, 起始时刻)…]（后代换了进程组、换了会话、被收养都还带着标记）。"""
    prefix = (SWEEP_TOKEN_VARIABLE + "=").encode()
    found = []
    for name in os.listdir("/proc"):
        if not name.isdigit() or int(name) == os.getpid():
            continue
        try:
            with open(f"/proc/{name}/environ", "rb") as handle:
                environment = handle.read()
        except OSError:
            continue
        for entry in environment.split(b"\0"):
            if entry.startswith(prefix) and token in entry[len(prefix):].decode(errors="replace").split(","):
                state = process_state_and_start(int(name))
                if state is not None and state[0] not in ("Z", "X"):
                    found.append((int(name), state[1]))
                break
    return found


def process_description(process_id):
    """进程号加命令行开头，点名留下的进程用；命令行读不到（进程已退）写明。"""
    try:
        with open(f"/proc/{process_id}/cmdline", "rb") as handle:
            words = handle.read().split(b"\0")
    except OSError:
        return f"{process_id}（读不到命令行）"
    text = " ".join(word.decode(errors="replace") for word in words).replace("\n", " ")
    return f"{process_id}（{text[:80]}）"


def text_of(value):
    return value.decode(errors="replace") if isinstance(value, bytes) else (value or "")


def run_sweep_job(job, token, time_limit, grace):
    """起一个开关的子进程（自己一个进程组、一个临时目录，环境里带 token 标记）并等它：time_limit 秒没退出就按进程组停（先 TERM，过 grace 秒组里还有活的再 KILL，
    再核；进程组号就是子进程的进程号）。超时与正常退出一样，之后等带标记的后代 grace 秒，还留着的逐个点名记进没对上，不停它们。
    临时目录无论如何删掉（弄坏开关 sweep-job-directory-kept 下不删）。弄坏开关 sweep-child-without-time-limit 下不设时限，
    sweep-group-not-signalled 下到点只给子进程自己发信号，sweep-child-term-only 下不发 KILL。"""
    directory = selftest_root()
    environment = dict(job.environment, TMPDIR=directory)
    environment[SWEEP_TOKEN_VARIABLE] = ",".join(filter(None, [job.environment.get(SWEEP_TOKEN_VARIABLE), token]))
    started = time.monotonic()
    try:
        try:
            process = subprocess.Popen(job.command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment, start_new_session=True)
        except OSError as error:
            return SweepOutcome(job.name, False, f"子进程起不来：{type(error).__name__}：{error}", "", 0, 0, 0, time.monotonic() - started)

        def signal_job_group(signal_number):
            with contextlib.suppress(ProcessLookupError):
                if break_is("sweep-group-not-signalled"):
                    os.kill(process.pid, signal_number)
                else:
                    os.killpg(process.pid, signal_number)

        def stop_group():
            before = len(live_group_members(process.pid))
            signal_job_group(signal.SIGTERM)
            if not wait_until(lambda: not live_group_members(process.pid), grace) and not break_is("sweep-child-term-only"):
                signal_job_group(signal.SIGKILL)
                wait_until(lambda: not live_group_members(process.pid), grace)
            return before - len(live_group_members(process.pid))

        timed_out, in_group, outside_seen = False, 0, 0
        try:
            output, error_output = process.communicate(timeout=None if break_is("sweep-child-without-time-limit") else time_limit)
        except subprocess.TimeoutExpired:
            timed_out = True
            members = set(live_group_members(process.pid))
            outside_seen = len([process_id for process_id, _start in marked_processes(token) if process_id not in members])
            in_group = stop_group()
        wait_until(lambda: not marked_processes(token), grace)
        left = [process_description(process_id) for process_id, _start in marked_processes(token)]
        if timed_out:
            try:
                output, error_output = process.communicate(timeout=grace)
            except subprocess.TimeoutExpired as error:
                output, error_output = text_of(error.stdout), text_of(error.stderr)
                process.stdout.close()
                process.stderr.close()
        problems = []
        if timed_out:
            problems.append(f"超时：{time_limit} 秒没退出，按进程组停了 {in_group} 个进程，到点时进程组外带标记的有 {outside_seen} 个")
        if left:
            problems.append(f"{'停完进程组' if timed_out else '子进程退出'}之后 {grace} 秒还留着 {len(left)} 个带标记的进程：{'、'.join(left)}（不由自证去停，要它们在父进程没了之后自己退）")
        if job.must_pass:
            if not timed_out and process.returncode != 0:
                problems.append(f"退 {process.returncode}，要 0")
        elif not timed_out and not (process.returncode != 0 and all(f"✗ 自证 {cell}：" in output for cell in job.red_cells)):
            problems.append(f"要红在 {'、'.join(job.red_cells)}，退 {process.returncode}")
        return SweepOutcome(job.name, not problems, "；".join(problems), output, in_group, outside_seen, len(left), time.monotonic() - started,
                            process.returncode, text_of(error_output))
    finally:
        if not break_is("sweep-job-directory-kept"):
            shutil.rmtree(directory, ignore_errors=True)


def run_break_sweep(jobs, threads, time_limit, grace, token_root):
    """各开关的子进程并行跑（至多 threads 路），逐个收回结果、按派出的次序交；收回的条数不等于派出的条数报错。
    弄坏开关 sweep-outcome-dropped 下收回之后丢掉最后一条再核条数。"""
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(len(jobs), threads))) as executor:
        futures = [executor.submit(run_sweep_job, job, job_token(token_root, index), time_limit, grace) for index, job in enumerate(jobs)]
        outcomes = [future.result() for future in futures]
    if break_is("sweep-outcome-dropped"):
        outcomes = outcomes[:-1]
    if len(outcomes) != len(jobs):
        raise RuntimeError(f"弄坏开关轮派出 {len(jobs)} 条，只收回 {len(outcomes)} 条")
    return outcomes


def sweep_child_command(binary, cells):
    """一个开关的子进程：本文件的 --selftest，只走 cells 里的格。弄坏开关 sweep-child-runs-all-cells 下不带 --cells。"""
    command = [sys.executable, ENTRY_SCRIPT_PATH, "--selftest", "--server-binary", binary]
    if not break_is("sweep-child-runs-all-cells"):
        command += ["--cells", ",".join(cells)]
    return command


def cell_child_command(binary, name):
    """整份趟里一格的子进程：本文件的 --selftest，只走这一格（子进程里走串行路）。写死带 --cells：不带它子进程又会把各格再并行一遍。"""
    return [sys.executable, ENTRY_SCRIPT_PATH, "--selftest", "--server-binary", binary, "--cells", name]


def cell_block(output):
    """一格的子进程标准输出：交 (末行之前的行, 末行)；末行是 PRUNE_SERVER_CLIENT_SELFTEST 起头的那一行，没有交空串。"""
    lines = []
    for line in output.splitlines():
        if line.startswith("PRUNE_SERVER_CLIENT_SELFTEST "):
            return lines, line
        lines.append(line)
    return lines, ""


def judge_cell_outcome(name, outcome):
    """一格的子进程结果判成 (是否绿, 要打的文字)：绿要同时有它打的「✓ 自证 <格名>：」行、一格全绿的末行、子进程退 0 且没留下进程；
    打的是它自己的块（多行的说明连着）。红时有它打的「✗ 自证 <格名>：」块就原样打，另附子进程的问题；没有就补一行说明子进程交了什么。
    弄坏开关 cell-child-exit-code-ignored 下不看退出码、时限与留下的进程，只看它打没打 ✓ 行与末行。"""
    block, summary = cell_block(outcome.output)
    printed_green = any(line.startswith(f"  ✓ 自证 {name}：") for line in block)
    printed_red = any(line.startswith(f"  ✗ 自证 {name}：") for line in block)
    summary_green = summary == "PRUNE_SERVER_CLIENT_SELFTEST cells=1 passed=1 failed=0 failed_cells=-"
    exited_clean = outcome.matched or break_is("cell-child-exit-code-ignored")
    if printed_green and summary_green and exited_clean:
        return True, "\n".join(block)
    if printed_red:
        text = "\n".join(block)
        return False, text + (f"\n     → 这一格的子进程另有：{outcome.detail}" if outcome.detail else "")  # gate-lint:detail
    return False, (f"  ✗ 自证 {name}：这一格的子进程没交出绿的结果（{outcome.detail or '退 ' + str(outcome.returncode)}）；"  # gate-lint:detail
                   f"标准输出末尾「{' '.join(block)[-200:]}」；标准错误末尾「{outcome.error.strip()[-300:]}」")


def judge_cells_in_children(selected, command_of, threads, time_limit, grace, token_root):
    """selected 里的格各起一个子进程并行跑（至多 threads 路；run_sweep_job：各自一个进程组、一个临时目录、带时限和标记，退出之后核留下的进程），
    按 selected 的次序逐格判，判完一格交一个 (格名, 是否绿, 要打的文字)；后面的格先跑完也等前面的。交完核收回的条数，不等于派出的条数抛 RuntimeError。
    弄坏开关 cells-run-serially 下一路一路跑，cells-merged-by-completion 下按跑完的次序交，cell-result-dropped 下收回之后丢掉最后一条再核条数。"""
    jobs = [SweepJob(name, (name,), command_of(name), dict(os.environ), must_pass=True) for name in selected]
    paths = 1 if break_is("cells-run-serially") else max(1, min(len(jobs), threads))
    with concurrent.futures.ThreadPoolExecutor(max_workers=paths) as executor:
        futures = [executor.submit(run_sweep_job, job, job_token(token_root, index), time_limit, grace) for index, job in enumerate(jobs)]
        collected = list(concurrent.futures.as_completed(futures)) if break_is("cells-merged-by-completion") else futures
        if break_is("cell-result-dropped"):
            collected = collected[:-1]
        returned = 0
        for future in collected:
            outcome = future.result()
            returned += 1
            passed, text = judge_cell_outcome(outcome.name, outcome)
            yield outcome.name, passed, text
    if returned != len(jobs):
        raise RuntimeError(f"各格的子进程派出 {len(jobs)} 条，只收回 {returned} 条")


def parse_selftest_options(arguments):
    """--selftest 后面的 --cells <格名>[,<格名>…]：交 (要走的格, 拒绝的原因)。没给 --cells 走全部 CELLS；给了只走点名的，次序照 CELLS、重复的走一次；
    缺格名、有空项、认不出的格名交 (None, 原因)，原因里列出可用的格名。
    弄坏开关 cells-option-ignored 下给了也走全部，unknown-cell-name-accepted 下认不出的格名悄悄丢掉。"""
    if "--cells" not in arguments:
        return CELLS, ""
    index = arguments.index("--cells")
    value = arguments[index + 1] if index + 1 < len(arguments) else ""
    names = value.split(",")
    available = f"可用的格名：{'、'.join(CELLS)}"
    if not value or "" in names:
        return None, f"--cells 后面要写 <格名>[,<格名>…]（给的是「{value}」）；{available}"
    unknown = [name for name in names if name not in selftest_cell_functions()]
    if unknown and not break_is("unknown-cell-name-accepted"):
        return None, f"--cells 认不出 {'、'.join(unknown)}；{available}"
    if break_is("cells-option-ignored"):
        return CELLS, ""
    return tuple(name for name in CELLS if name in names), ""


def cell_sweep_child_runs_only_named_cells(_binary, _work):
    """开关子进程只走点名的格：sweep_child_command 拼出的命令里 --selftest 后面的部分交给 parse_selftest_options，走的恰是点名的格；
    --cells 缺格名、认不出的格名拒（并列出可用的格名）；不给 --cells 走全部格。"""
    named = ("round-trip-unix", "prefix-scan")
    command = sweep_child_command("/no/such/prune-server", named)
    options = command[command.index("--selftest") + 1:]
    selected, problem = parse_selftest_options(options)
    unknown_selected, unknown_problem = parse_selftest_options(["--cells", "round-trip-unix,no-such-cell"])
    missing_selected, missing_problem = parse_selftest_options(["--cells"])
    empty_selected, empty_problem = parse_selftest_options(["--cells", "round-trip-unix,"])
    whole_selected, whole_problem = parse_selftest_options(["--server-binary", "/no/such/prune-server"])
    named_only = selected == named and not problem
    unknown_refused = unknown_selected is None and "no-such-cell" in unknown_problem and all(name in unknown_problem for name in CELLS)
    missing_refused = missing_selected is None and bool(missing_problem) and empty_selected is None and bool(empty_problem)
    whole_run = whole_selected == CELLS and not whole_problem
    passed = named_only and unknown_refused and missing_refused and whole_run
    return passed, (f"命令 {options} 走的格 {selected}（要 {named}）；认不出的格名拒 {unknown_refused}、缺格名与空项拒 {missing_refused}；不给 --cells 走全部 {len(CELLS)} 格 {whole_run}")


def cell_sweep_stops_hung_child(_binary, _work):
    """弄坏开关轮对卡死的子进程到点就停（run_break_sweep）：三个假子进程并行，卡死的（sweep_hung_child_command）要在时限加两倍宽限再加余量之内被停、记「超时」、
    进程组里停 2 个、到点时组外带标记的 1 个、停完组之后宽限之内这个组外的自己退掉（盯着父进程），没有点名留下的进程、临时目录删了；
    红在点名格的记对上；退 0 没红的记没对上；收回 3 条。整轮没在时限里回来就判红，留下的进程点名列出，不由这一格去停（假子进程与孙进程有自退时限）。
    弄坏开关 sweep-descendants-left 下组外的孙进程不盯父进程，停完整组之后它还留着，被点名列出、这一格判红，临时目录一删它自己退掉。"""
    root = secrets.token_hex(8)
    jobs = [SweepJob("fake-hang", ("fake-cell",), sweep_hung_child_command(), dict(os.environ)),
            SweepJob("fake-red", ("fake-cell",), [sys.executable, "-c", "print('✗ 自证 fake-cell：红了'); print('     → 怎么办：假子进程，红是故意的'); raise SystemExit(1)"], dict(os.environ)),
            SweepJob("fake-not-red", ("fake-cell",), [sys.executable, "-c", "print('✓ 自证 fake-cell：绿了')"], dict(os.environ))]
    result = {}

    def sweep():
        try:
            result["outcomes"] = run_break_sweep(jobs, len(jobs), SWEEP_CELL_LIMIT_SECONDS, SWEEP_CELL_GRACE_SECONDS, root)
        except RuntimeError as error:
            result["error"] = str(error)

    runner = threading.Thread(target=sweep, daemon=True)
    started = time.monotonic()
    runner.start()
    return_within = SWEEP_CELL_LIMIT_SECONDS + 2 * SWEEP_CELL_GRACE_SECONDS + SWEEP_CELL_RETURN_SLACK_SECONDS
    runner.join(timeout=return_within)
    returned = not runner.is_alive()
    elapsed = time.monotonic() - started
    left = [process_description(process_id) for index in range(len(jobs)) for process_id, _start in marked_processes(job_token(root, index))]
    outcomes = result.get("outcomes") or []
    by_name = {outcome.name: outcome for outcome in outcomes}
    hang, red, not_red = by_name.get("fake-hang"), by_name.get("fake-red"), by_name.get("fake-not-red")
    directory = next((line[len("tmpdir="):] for line in (hang.output.splitlines() if hang else []) if line.startswith("tmpdir=")), "")
    directory_left = bool(directory) and os.path.exists(directory)
    if directory_left:
        shutil.rmtree(directory, ignore_errors=True)
    stopped = (hang is not None and not hang.matched and "超时" in hang.detail and hang.stopped_in_group == 2 and hang.outside_seen == 1 and hang.left == 0
               and hang.seconds < SWEEP_CELL_LIMIT_SECONDS + 2 * SWEEP_CELL_GRACE_SECONDS + SWEEP_CELL_EXIT_SLACK_SECONDS)
    judged = red is not None and red.matched and not_red is not None and not not_red.matched and "退 0" in not_red.detail
    passed = returned and "error" not in result and len(outcomes) == len(jobs) and stopped and judged and not left and bool(directory) and not directory_left
    hang_text = (f"{hang.seconds:.1f} 秒记「{hang.detail}」" if hang is not None else "没有结果")
    return passed, (f"整轮{'回来了' if returned else '没回来'}（{elapsed:.1f} 秒，要 {return_within} 秒内）{result.get('error', '')}；收回 {len(outcomes)} 条（要 {len(jobs)} 条）；"
                    f"卡死的：{hang_text}（要超时、组里停 2 个、到点时组外带标记的 1 个、停完组之后一个不留、{SWEEP_CELL_LIMIT_SECONDS + 2 * SWEEP_CELL_GRACE_SECONDS + SWEEP_CELL_EXIT_SLACK_SECONDS} 秒内）；"
                    f"红在点名格的记对上并且退 0 没红的记没对上 {judged}；整轮回来时留下带标记的进程 {len(left)} 个（要 0）{'：' + '、'.join(left) if left else ''}；"
                    f"卡死的子进程的临时目录 {directory or '（没读到）'} {'还在' if directory_left else '删了'}")


def parallel_children_run(names, commands, threads, root):
    """judge_cells_in_children 对假子进程跑一趟：交 (逐格交出的 (格名, 是否绿, 文字), 抛出的 RuntimeError 的说明，没抛是空串)。"""
    results, error = [], ""
    try:
        for result in judge_cells_in_children(names, commands.__getitem__, threads, PARALLEL_CELL_LIMIT_SECONDS, SWEEP_CELL_GRACE_SECONDS, root):
            results.append(result)
    except RuntimeError as raised:
        error = str(raised)
    return results, error


def cell_cells_run_in_parallel_children(_binary, work):
    """整份趟里各格在子进程里并行、次序确定（judge_cells_in_children）：四个假的格子进程互相等齐才退（一路一路跑等不齐，记没等齐），
    退出的次序与派出的次序相反；要全部等齐、各自一个进程号与进程组（组号就是自己的进程号）与临时目录（跑完删了）、都不是本进程的，
    逐格交出的次序是派出的次序、全绿、收回 4 条没抛错。另一趟三个假的格子进程各演一种红（打 ✗ 行退 1、打 ✓ 行与全绿的末行却退 1、什么都不打退 0），都要判红，
    红的说明里点名那一格。
    弄坏开关 cells-run-serially 下一路一路跑（等不齐），cells-merged-by-completion 下按跑完的次序交，cell-result-dropped 下收回少一条抛错，
    cell-child-exit-code-ignored 下打了 ✓ 行却退 1 的判绿。"""
    names = ("fake-slowest", "fake-slower", "fake-faster", "fake-fastest")
    rendezvous = os.path.join(work, "rendezvous")
    os.makedirs(rendezvous)

    def green_command(name, delay, mode="green", code=0, directory=""):
        return [sys.executable, "-c", FAKE_CELL_CHILD, name, directory, str(len(names)), str(FAKE_CELL_RENDEZVOUS_SECONDS), str(delay), mode, str(code)]

    commands = {name: green_command(name, FAKE_CELL_RELEASE_GAP_SECONDS * (len(names) - 1 - position), directory=rendezvous) for position, name in enumerate(names)}
    results, error = parallel_children_run(names, commands, len(names), secrets.token_hex(8))
    fields = [dict(word.split("=", 1) for word in text.split("：", 1)[-1].split() if "=" in word) for _name, _passed, text in results]
    order_kept = [name for name, _passed, _text in results] == list(names)
    all_green = len(results) == len(names) and all(passed for _name, passed, _text in results)
    met_together = len(fields) == len(names) and all(entry.get("met") == "1" for entry in fields)
    own_processes = (len({entry.get("pid") for entry in fields}) == len(names) and str(os.getpid()) not in {entry.get("pid") for entry in fields}
                     and all(entry.get("group") == entry.get("pid") for entry in fields))
    directories = [entry.get("tmpdir", "") for entry in fields]
    own_directories = len(set(directories)) == len(names) and os.environ.get("TMPDIR", "") not in directories and not any(os.path.exists(path) for path in directories)
    failing = ("fake-prints-red", "fake-exits-red-after-green", "fake-prints-nothing")
    failing_commands = {"fake-prints-red": green_command("fake-prints-red", 0, "red", 1), "fake-exits-red-after-green": green_command("fake-exits-red-after-green", 0, "green", 1),
                        "fake-prints-nothing": green_command("fake-prints-nothing", 0, "silent", 0)}
    red_results, red_error = parallel_children_run(failing, failing_commands, len(failing), secrets.token_hex(8))
    red_named = [passed is False and f"✗ 自证 {name}：" in text for name, passed, text in red_results]
    red_judged = [name for name, _passed, _text in red_results] == list(failing) and len(red_named) == len(failing) and all(red_named) and not red_error
    passed = (not error and order_kept and all_green and met_together and own_processes and own_directories and red_judged)
    return passed, (f"绿的一趟：次序{'对' if order_kept else '不对'}（{[name for name, _p, _t in results]}），全绿 {all_green}，四个子进程互相等齐 {met_together}，"
                    f"各自的进程与进程组 {own_processes}，各自的临时目录且已删 {own_directories}，抛错「{error}」（要没有）；"
                    f"红的一趟：收回 {len(red_results)} 条、各自判红并点名 {red_named}，抛错「{red_error}」（要没有）")


def selftest_cell_functions():
    """自证各格，格名到函数。表写在函数体里：写成模块级赋值，会在载入这个模块时把全部格函数连同它们起的子进程拖进载入一方的判法单元闭包。"""
    return {"round-trip-unix": cell_round_trip_unix, "prefix-scan": cell_prefix_scan,
            "tcp-round-trip": cell_tcp_round_trip,
            "busy-retried-after-delay": cell_busy_retried_after_delay, "stopped-exits-75": cell_stopped_exits_75,
            "unreachable-exits-75": cell_unreachable_exits_75, "families-over-service": cell_families_over_service,
            "unreachable-family-exits-75": cell_unreachable_family_exits_75,
            "connect-bounded-by-reconnect-total": cell_connect_bounded_by_reconnect_total,
            "large-value-round-trip": cell_large_value_round_trip,
            "segmented-read-incomplete-refused": cell_segmented_read_incomplete_refused,
            "segmented-read-sees-a-rewrite": cell_segmented_read_sees_a_rewrite,
            "segmented-rewrite-survives-writer-death": cell_segmented_rewrite_survives_writer_death,
            "segment-count-over-index-digits-refused": cell_segment_count_over_index_digits_refused,
            "scan-row-over-frame-refused": cell_scan_row_over_frame_refused,
            "prefix-scan-continues-after-the-last-key": cell_prefix_scan_continues_after_the_last_key,
            "segment-key-shapes-only-reserved": cell_segment_key_shapes_only_reserved,
            "oversize-frame-reported-at-once": cell_oversize_frame_reported_at_once,
            "temporary-service-exits-with-parent": cell_temporary_service_exits_with_parent,
            "service-problem-judges-given-environment": cell_service_problem_judges_given_environment,
            "connection-variables-refused-outside-temporary-service": cell_connection_variables_refused_outside_temporary_service,
            "command-line-read": cell_command_line_read, "namespace-only-under-temporary-service": cell_namespace_only_under_temporary_service,
            "batched-read-answer-over-frame-limit-split": cell_batched_read_answer_over_frame_limit_split,
            "binary-not-built-without-target-directory": cell_binary_not_built_without_target_directory,
            "delete-prefix-by-namespace": cell_delete_prefix_by_namespace,
            "delete-by-item-name-picks-by-value": cell_delete_by_item_name_picks_by_value,
            "delete-conclusions-by-kind": cell_delete_conclusions_by_kind,
            "delete-conclusions-refuses-while-claims-are-live": cell_delete_conclusions_refuses_while_claims_are_live,
            "delete-conclusions-refuses-a-source-that-differs": cell_delete_conclusions_refuses_a_source_that_differs,
            "conclusion-kinds-match-the-verdict-store": cell_conclusion_kinds_match_the_verdict_store,
            "write-storage-failure-replayed": cell_write_storage_failure_replayed,
            "gpu-pending-rows-written-only-if-absent": cell_gpu_pending_rows_written_only_if_absent,
            "gpu-conclusion-write-guarded": cell_gpu_conclusion_write_guarded,
            "gpu-claim-listing-lists-live-claims": cell_gpu_claim_listing_lists_live_claims,
            "temporary-service-binds-the-address-of-this-host": cell_temporary_service_binds_the_address_of_this_host,
            "temporary-service-refuses-without-readable-hosts": cell_temporary_service_refuses_without_readable_hosts,
            "sweep-child-runs-only-named-cells": cell_sweep_child_runs_only_named_cells,
            "sweep-stops-hung-child": cell_sweep_stops_hung_child,
            "cells-run-in-parallel-children": cell_cells_run_in_parallel_children}
