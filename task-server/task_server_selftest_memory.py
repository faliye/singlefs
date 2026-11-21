"""task_server_selftest.py 拆出的一份：内存上限、重试放大、资源记录、钉值的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import threading
import time
import types
import unittest.mock
import task_server_lib as lib
from task_server_selftest_fakes import (
    AT_CAP_PEAK_BYTES, AT_CAP_PINNED_TEXT, AT_CAP_SEED_PEAK_BYTES, BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, MEBIBYTE, PEAKS,
    PLACEMENT_WAIT_SECONDS, SEEDED_ENDED_AT, SELFTEST_LOCAL_ADDRESS, SELFTEST_PEER_ADDRESS, SERVER, UNRECORDED_DEFAULT_PINNED_TEXT,
)
from task_server_selftest_world import (
    CPU_GATE_PROBE_KEY, RUNNING_BOOKING_LABEL, SelftestFailure, attempt_has_run_for, attention_lines, by_host_text, cargo_targets,
    cpu_gate_scheduler, describe, fake_cargo_extra, first_attempt, loaded_host_dispatch, long_run_split_scheduler, peer_log_scheduler,
    process_memory_stat, running_booking_scheduler, seed_ended_attempt, server_modules, shown_local_pin, start_burning_scope, write_executable,
)


RECORD_WINDOW_OVERRUN = 6   # 比 task_server.py 的 RECORD_WINDOW（近几次的名额，5）多一次：量不到的那几次要是占了名额，导进来的那一次就被挤出去

AT_CAP_GROWN_PINNED_TEXT = "2304M"

# pinned-cap-by-host 钉住的两台钉值：SLICE_MEMORY_CAP_BY_HOST 那一项减那台的假余量 2G（SELFTEST_MARGIN_GIBIBYTES；task_server.py 的 pin_value 与 pinned_on_host）；本机写 20G、第二台写 30G。
# 钉值的口径只由这一格钉数，别的钉值格照 control pin-cap 报的此刻钉值核（shown_local_pin）
PINNED_LOCAL_TEXT = "18432M"
PINNED_PEER_TEXT = "28672M"

UNRECORDED_GROWTH_PINNED_TEXTS = ("4096M", "6144M", "9216M")
UNRECORDED_BURST_COUNT = 5      # unrecorded-burst-balanced 连交几条没记录的命令（各记 4096M）：两台都放得下时按剩余占比分到两台
# 没跑完的尝试量到的峰值：现查的那一次提前退 1 的命令就是这么多（records/2026-10-30-编译绕过内存总上限.md「服务修复两件合起来验」#8），乘 1.5 是 63M
UNFINISHED_PEAK_BYTES = 42 * MEBIBYTE
# 库里记一次跑完的峰值 2048 MiB：定的上限照它 2048M（近五次最大，不乘 1.5）；撞了它乘 1.5 是 3072M（钉的是验收原话里的两个数）
CAP_HIT_SEED_PEAK_BYTES = 2048 * MEBIBYTE
CAP_HIT_PINNED_TEXT = "2048M"
CAP_HIT_GROWN_PINNED_TEXT = "3072M"


@contextlib.contextmanager
def patched_server_name(name, replacement):
    """服务拆成几份之后，一个名字在每份模块里各有一个绑定（定义它的那一份、import 它的几份与入口）：把每一份里有这个名字的绑定都换成 replacement，出来时还原；调它的方法落在哪一份都看到换过的。"""
    with contextlib.ExitStack() as stack:
        for module in server_modules():
            if hasattr(module, name):
                stack.enter_context(unittest.mock.patch.object(module, name, replacement))
        yield


def task_command_ok(summary, outcome, attempts=None):
    task = summary["task"]
    return task.get("outcome") == outcome and (attempts is None or len(summary["attempts"]) == attempts)


def cell_memory_cap_growth(world):
    world.seed_peak("selftest-growth", 1536 * MEBIBYTE)   # 上限照近五次最大 1536M 起，不乘 1.5；撞了才乘
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-growth"})
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if caps != ["1536M", "2304M", "3456M"] or outcomes != ["memory-cap-hit", "memory-cap-hit", "succeeded"] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"连撞两次上限：上限应当 1536M、2304M（1.5 倍）、3456M，三次尝试都在记录里、第三次 succeeded，实际上限 {caps}、类型 {outcomes}、{describe(summary)}")
    return "上限 1536M → 2304M → 3456M，三次尝试都留着，第三次跑完"


def cell_unrecorded_default_small(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    summary = world.finished(task_id)
    attempt = summary["attempts"][0] if summary["attempts"] else {}
    calls = world.calls_of(task_id)
    pinned_bytes = int(UNRECORDED_DEFAULT_PINNED_TEXT[:-1]) * MEBIBYTE
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT or attempt.get("memory_cap") != pinned_bytes
            or attempt.get("booked") != pinned_bytes or attempt.get("prediction") != lib.UNRECORDED_HISTORY
            or not calls or f" cap={UNRECORDED_DEFAULT_PINNED_TEXT} " not in calls[0]):
        raise SelftestFailure(f"服务的记录里没有的命令：第一次尝试的上限与记账都应当是 {UNRECORDED_DEFAULT_PINNED_TEXT}（{pinned_bytes} 字节）、"
                              f"prediction 记 {lib.UNRECORDED_HISTORY}、内存包装收到 cap={UNRECORDED_DEFAULT_PINNED_TEXT}，实际上限 {attempt.get('memory_cap_text')}"
                              f"（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')} 字节、prediction {attempt.get('prediction')}、{describe(summary)}、"
                              f"包装调用 {calls}", world.server_log_lines())
    return f"没记录的命令第一次上限与记账都是 {UNRECORDED_DEFAULT_PINNED_TEXT}，内存包装收到的也是它"


def cell_unrecorded_cap_growth(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"])
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    numbers = [attempt.get("number") for attempt in summary["attempts"]]
    wrapper_caps = [line.split(" cap=", 1)[1].split(" ", 1)[0] for line in world.calls_of(task_id) if " cap=" in line]
    expected_caps = list(UNRECORDED_GROWTH_PINNED_TEXTS)
    if (caps != expected_caps or wrapper_caps != expected_caps or outcomes != ["memory-cap-hit", "memory-cap-hit", "succeeded"] or numbers != [1, 2, 3]
            or summary["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"没记录的命令假包装连退两次 250：上限应当 {' → '.join(expected_caps)}（小缺省起，每次乘 1.5）、三次尝试都在记录里（第 1、2、3 次）、第三次 succeeded，"
                              f"实际上限 {caps}、包装收到 {wrapper_caps}、类型 {outcomes}、次序 {numbers}、{describe(summary)}", world.server_log_lines())
    return f"没记录的命令撞了再放大：上限 {' → '.join(expected_caps)}，三次尝试都留着，第三次跑完"


def cell_unrecorded_burst_balanced(world):
    world.resources(local=(40, 40), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    # 放行文件写成相对假仓根的名字：本机那几条在 world.repo 里等它，假第二台照 fake-peer-wait 在假仓根里等同一个名字（两边认的是同一份放行文件）。
    # 派到第二台的那几条也要一直占着到放行（fake-peer-wait）：假第二台不起命令、当场退 0 的话，交得慢时每一条起之前前一条已在第二台收尾、第二台总显得空，五条全去第二台
    releases = [f"release-burst-{number}" for number in range(1, UNRECORDED_BURST_COUNT + 1)]
    task_ids = [world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release, f"fake-peer-wait={release}"], cwd=world.repo) for release in releases]
    placed = {}
    try:
        for task_id in task_ids:
            summary = world.wait_for(task_id, lambda summary: bool(summary["attempts"]), "起第一次尝试")
            placed[task_id] = summary["attempts"][0].get("host")
    finally:
        for release in releases:
            open(os.path.join(world.repo, release), "w").close()
    for task_id in task_ids:
        world.finished(task_id)
    counts = {host: list(placed.values()).count(host) for host in ("local", "peer")}
    if counts["local"] < 1 or counts["peer"] < 1 or sum(counts.values()) != UNRECORDED_BURST_COUNT:
        raise SelftestFailure(f"本机总上限 40G、第二台 64G，连交 {UNRECORDED_BURST_COUNT} 条没记录的命令（各记 {UNRECORDED_DEFAULT_PINNED_TEXT}、都派得到第二台）："
                              f"按剩余占比均衡，两台都应当分到，实际各条第一次尝试的台 {placed}", world.server_log_lines())
    return f"连交 {UNRECORDED_BURST_COUNT} 条没记录的命令各记 {UNRECORDED_DEFAULT_PINNED_TEXT}，按剩余占比分到两台：本机 {counts['local']}、第二台 {counts['peer']}"


def cell_memory_cap_at_maximum(world):
    world.resources(local=(55, 50))   # 总上限 50G（真机上这个数取多机配置 SLICE_MEMORY_CAP_BY_HOST 里本机那一项），没有第二台
    world.seed_peak("selftest-at-maximum", 30 * GIBIBYTE)
    world.seed_peak("selftest-at-maximum-switch", 30 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-at-maximum"})
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if caps != ["30720M", "46080M"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        raise SelftestFailure(f"总上限 50G、没有第二台：记录峰值 30G 起 30720M（不乘 1.5），撞了乘 1.5 到 46080M，再撞（再乘 1.5 就过总上限）记 memory-cap-at-maximum 打回、退 236，"
                              f"实际上限 {caps}、{describe(summary)}")
    completed = world.client(["wait", str(task_id)])
    last = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    expected = (f"{lib.RESULT_MARK} task={task_id} type=memory-cap-at-maximum exit=236 ", " host=本机 reached=46080M next=撞到动态上限、打回：本机已给到 46080M（峰值 ")
    if (completed.returncode != 236 or not last.startswith(expected[0]) or expected[1] not in last
            or "核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交" not in last):
        raise SelftestFailure(f"两台都到顶时 wait 应当退 236、末行写撞了哪台（本机）、到了多少（46080M）、峰值，修改意见写核实范围、分片与同时起的个数，"
                              f"实际退 {completed.returncode}、末行「{last}」", [completed.stderr])
    world.resources(local=(55, 50), peer=(3, 80), peer_cap_gibibytes=80)   # 第二台总上限 80G、此刻只空 3G：头两次只能在本机
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-at-maximum-switch"}, cwd=world.repo)
    world.wait_for(task_id, lambda summary: len(summary["attempts"]) >= 2 and summary["attempts"][1].get("state") == "ended", "本机那两次收尾")
    world.resources(local=(55, 50), peer=(85, 80), peer_cap_gibibytes=80)
    summary = world.finished(task_id)
    placed = [(attempt.get("host"), attempt.get("memory_cap_text")) for attempt in summary["attempts"]]
    if placed != [("local", "30720M"), ("local", "46080M"), ("peer", "69120M")] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"本机 46080M 还撞、再乘 1.5（69120M）就过本机总上限 50G、第二台总上限 80G：应当换到第二台、上限 69120M、跑完，实际 {placed}、{describe(summary)}")
    return "撞上限乘 1.5 重排；再乘就到总上限的记 memory-cap-at-maximum 打回；有总上限更大的另一台就换过去"


def cell_retry_kinds(world):
    world.start_server()
    plain = ["bash", "-c", "exit 0"]
    squeezed = world.submit(plain + ["fake-wrapper-exits=254,run"])
    unlisted = world.submit(plain + ["fake-wrapper-exits=245,0"])
    wrong = []
    for task_id, attempts, outcome in ((squeezed, 2, "succeeded"), (unlisted, 1, "command-failed")):
        summary = world.finished(task_id)
        if not task_command_ok(summary, outcome, attempts):
            wrong.append(f"任务 {task_id} 应当尝试 {attempts} 次、结局 {outcome}，实际 {describe(summary)}")
    if wrong:
        raise SelftestFailure("254 同一上限重排、表外的 245 不重排：" + "；".join(wrong))
    return "254 重排一次跑完、表外的 245 记 command-failed 不重排"


def cell_resource_record(world):
    scope = os.path.join(world.root, "fake-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(300 * MEBIBYTE))
    with open(os.path.join(scope, "memory.peak"), "w", encoding="utf-8") as handle:
        handle.write(f"{300 * MEBIBYTE}\n")
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 2000000\nuser_usec 1500000\nsystem_usec 500000\n")
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    command = ["bash", "-c", "sleep 1.2"]
    first = world.finished(world.submit(command))["attempts"][0]
    if (first.get("memory_peak") != 300 * MEBIBYTE or first.get("cpu_usec") != 2000000 or not isinstance(first.get("seconds"), (int, float))
            or first["seconds"] < 1.0 or not first.get("cores") or abs(first["cores"] - 2.0 / first["seconds"]) > 0.05):
        raise SelftestFailure(f"跑完应当记下用时（≥ 1 秒）、内存峰值 {300 * MEBIBYTE}、usage_usec 2000000 与平均在用核数（2 秒 CPU ÷ 用时），实际 "
                              f"用时 {first.get('seconds')}、峰值 {first.get('memory_peak')}、usage {first.get('cpu_usec')}、核数 {first.get('cores')}")
    peak_segment = first.get(SERVER.PEAK_SEGMENT_CORES_COLUMN)   # 跑不满一段：整次就是一段，收尾那一刻的累计 2 秒 CPU ÷ 服务量的已跑秒数（比命令自己的用时略长）
    if not isinstance(peak_segment, (int, float)) or isinstance(peak_segment, bool) or not 0 < peak_segment <= 2.0 / first["seconds"] + 0.05:
        raise SelftestFailure(f"收尾应当记峰值段核数（{SERVER.PEAK_SEGMENT_CORES_COLUMN}，跑不满一段的照整次：大于 0、不超过 2 秒 CPU ÷ 用时），实际 {peak_segment}、用时 {first.get('seconds')}")
    second_id = world.submit(command)
    second = world.finished(second_id)["attempts"][0]
    if second.get("memory_cap_text") != "300M" or not any("cap=300M" in line for line in world.calls_of(second_id)):
        raise SelftestFailure(f"同一命令下一次应当按记录的峰值 300M 定上限 300M（近五次最大，不乘 1.5），实际 {second.get('memory_cap_text')}、包装调用 {world.calls_of(second_id)}")
    return f"记下用时 {first['seconds']} 秒、峰值 300M、平均 {first['cores']} 核；下一次上限 300M"


def cell_retry_keeps_earlier_attempts(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "echo attempt-$TASK_SERVER_ATTEMPT; exit 0", "fake-wrapper-exits=254,run"])
    summary = world.finished(task_id)
    numbers = [attempt.get("number") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    logs = [attempt.get("log") for attempt in summary["attempts"]]
    if numbers != [1, 2] or outcomes != ["squeezed-out", "succeeded"] or len(set(logs)) != 2 or not all(log and os.path.isfile(log) for log in logs):
        raise SelftestFailure(f"重排一次之后两次尝试的记录都要在（第 1 次 squeezed-out、第 2 次 succeeded，各有各的日志），实际次序 {numbers}、类型 {outcomes}、日志 {logs}")
    return "重排之后前一次的记录照留：两次尝试、两份日志"


def cell_stalled_at_cap_retried(world):
    """判卡住那一刻 scope 的内存峰值已到上限（不低于 STALL_AT_CAP_FRACTION）：不记 stalled，记 memory-cap-hit、乘 1.5 重排，结局说明写「顶在上限上卡住」；
    峰值远低于上限的照旧记 stalled。假 scope 的进程内存（memory.stat 的 anon）与 memory.peak 钉在第一条的上限上，CPU 一直不涨。"""
    scope = os.path.join(world.root, "at-cap-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.stat", process_memory_stat(AT_CAP_PEAK_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-at-cap", AT_CAP_SEED_PEAK_BYTES)
    world.seed_peak("selftest-far-below-cap", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    # 第 1 次不出声地睡（顶在上限上卡住）；重排之后那一次每半秒打一行、跑完
    script = 'if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; sleep 30; fi; for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done'
    at_cap = world.finished(world.submit(["bash", "-c", script, "at-cap"], extra={lib.KEY_VARIABLE: "selftest-at-cap"}), timeout=60)
    far = world.finished(world.submit(["bash", "-c", "echo started; sleep 30", "far-below-cap"], extra={lib.KEY_VARIABLE: "selftest-far-below-cap"}), timeout=60)
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in at_cap["attempts"]]
    note = (at_cap["attempts"] or [{}])[0].get("stall_note") or ""
    problems = []
    if seen != [(AT_CAP_PINNED_TEXT, "memory-cap-hit"), (AT_CAP_GROWN_PINNED_TEXT, "succeeded")] or at_cap["task"].get("outcome") != "succeeded":
        problems.append(f"峰值顶在上限 {AT_CAP_PINNED_TEXT} 上卡住的应当记 memory-cap-hit、乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排、跑完，实际各次 {seen}、{describe(at_cap)}")
    if "顶在上限上卡住" not in note:
        problems.append(f"那一次的说明应当写「顶在上限上卡住」，实际「{note}」")
    if far["task"].get("outcome") != "stalled":
        problems.append(f"峰值远低于上限的应当照旧记 stalled，实际 {describe(far)}")
    if problems:
        raise SelftestFailure("顶在上限上卡住：" + "；".join(problems), world.server_log_lines())
    return f"顶在上限上卡住的记 memory-cap-hit、{AT_CAP_PINNED_TEXT} 乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排跑完（「{note[:60]}」）；峰值远低于上限的照旧记 stalled"


def cell_busy_cpu_pinned_at_cap_retried(world):
    """日志满卡住阈值没涨、此刻内存贴着上限、CPU 一直在涨（任务 7723：在回收里打转、跑了 37775 秒没人停；task_server.py 的 judge_stall_stop）：CPU 在涨不算在算，
    停掉、记 memory-cap-hit、乘 1.5 重排、跑完，说明写「顶在上限上卡住」与此刻内存。库里记一次跑完的峰值 1536 MiB（上限照它 1536M），
    假 scope 的进程内存（memory.stat 的 anon）、memory.current 与 memory.peak 钉在 1536 MiB，CPU 每 0.2 秒涨 0.2 秒。
    弄坏开关 at-cap-busy-cpu-kept-running 下照改前当在算、第一次睡满 30 秒退 0，这一格红。"""
    scope, stop = start_burning_scope(world, "pinned-busy-scope", {"memory.current": AT_CAP_PEAK_BYTES, "memory.peak": AT_CAP_PEAK_BYTES,
                                                                   "memory.stat": process_memory_stat(AT_CAP_PEAK_BYTES).rstrip("\n")})
    world.seed_peak("selftest-pinned-busy", AT_CAP_SEED_PEAK_BYTES)
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
        # 第 1 次不出声地睡（贴着上限、CPU 在涨）；重排之后那一次每半秒打一行、跑完（上限 2304M，此刻内存只到它的三分之二）
        script = 'if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; sleep 30; fi; for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done'
        summary = world.finished(world.submit(["bash", "-c", script, "pinned-busy"], extra={lib.KEY_VARIABLE: "selftest-pinned-busy"}), timeout=60)
    finally:
        stop.set()
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    note = first_attempt(summary).get("stall_note") or ""
    problems = []
    if seen != [(AT_CAP_PINNED_TEXT, "memory-cap-hit"), (AT_CAP_GROWN_PINNED_TEXT, "succeeded")] or summary["task"].get("outcome") != "succeeded":
        problems.append(f"此刻内存贴着上限 {AT_CAP_PINNED_TEXT}、CPU 在涨的应当记 memory-cap-hit、乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排、跑完，实际各次 {seen}、{describe(summary)}")
    if "顶在上限上卡住" not in note or "此刻内存" not in note:   # CPU 那半句不判：窗口开头那一刻 cpu_usec 可能还没采到，读不到时照停、不写核数
        problems.append(f"那一次的说明应当写「顶在上限上卡住」与「此刻内存」，实际「{note}」")
    if problems:
        raise SelftestFailure("贴着上限而 CPU 在涨：" + "；".join(problems), world.server_log_lines())
    return f"贴着上限而 CPU 在涨的记 memory-cap-hit、{AT_CAP_PINNED_TEXT} 乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排跑完（「{note[:80]}」）"


def cell_busy_cpu_below_cap_keeps_running(world):
    """对照：日志满卡住阈值没涨、CPU 一直在涨、此刻内存离上限远（假 scope 的进程内存与 memory.current 是上限 1536M 的八分之一）：在算，照跑完，只一次尝试。
    弄坏开关 at-cap-fraction-ignored 下读得到此刻内存就当贴着上限、停掉重排，cpu-ignored-for-stall 下不看 CPU 照停，这一格红。"""
    scope, stop = start_burning_scope(world, "below-cap-busy-scope", {"memory.current": AT_CAP_PEAK_BYTES // 8, "memory.peak": AT_CAP_PEAK_BYTES // 8,
                                                                      "memory.stat": process_memory_stat(AT_CAP_PEAK_BYTES // 8).rstrip("\n")})
    world.seed_peak("selftest-below-cap-busy", AT_CAP_SEED_PEAK_BYTES)
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
        summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 7", "below-cap-busy"], extra={lib.KEY_VARIABLE: "selftest-below-cap-busy"}), timeout=60)
    finally:
        stop.set()
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if seen != [(AT_CAP_PINNED_TEXT, "succeeded")] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 7 秒不涨（阈值 2 秒）、CPU 在涨、此刻内存只到上限的八分之一：在算，应当照跑完、只一次尝试，实际各次 {seen}、{describe(summary)}",
                              world.server_log_lines())
    return f"此刻内存离上限远、CPU 在涨的照跑完（各次 {seen}）"


PEER_PINNED_SEED_PEAK_BYTES = 10 * GIBIBYTE   # peer-pinned-at-cap-busy-cpu-retried：上限 15G，本机只空 8G、派到第二台
PEER_PINNED_CURRENT_BYTES = 15 * GIBIBYTE     # 那一格第一次尝试每轮报的此刻内存：钉在上限上


def cell_peer_pinned_at_cap_busy_cpu_retried(world):
    """第二台同样判贴着上限（task_server.py 的 PeerMachine.memory_now 读 MULTI_HOST_RUN_PEER_MEMORY；用户：「所有任务必须不受限制地多机跑」）：
    派到第二台的那一条第一次只打 multi-host-run.sh 那一层每轮的两行——MULTI_HOST_RUN_PEER_CPU（在涨，一个核）与 MULTI_HOST_RUN_PEER_MEMORY（此刻内存钉在上限 15G 上）——
    这两行都不算日志在涨；满卡住阈值之后停掉、记 memory-cap-hit、乘 1.5 重排、跑完。弄坏开关 at-cap-busy-cpu-kept-running、peer-memory-line-unread 下第一次照跑满、退 0，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-pinned", PEER_PINNED_SEED_PEAK_BYTES)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    script = ('if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then for i in $(seq 30); do echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=$((i*500000)) read=yes"; '
              'echo "MULTI_HOST_RUN_PEER_MEMORY poll=$i current_bytes=$1 peak_bytes=$1 read=yes"; sleep 0.5; done; exit 0; fi; '
              'for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done')
    summary = world.finished(world.submit(["bash", "-c", script, "peer-pinned", str(PEER_PINNED_CURRENT_BYTES), "fake-peer-run"],
                                          extra={lib.KEY_VARIABLE: "selftest-peer-pinned"}, cwd=world.repo), timeout=60)
    attempts = summary["attempts"]
    seen = [(attempt.get("host"), attempt.get("outcome")) for attempt in attempts]
    caps = [int(attempt.get("memory_cap") or 0) for attempt in attempts]
    note = first_attempt(summary).get("stall_note") or ""
    problems = []
    if [outcome for _, outcome in seen] != ["memory-cap-hit", "succeeded"] or seen[0][0] != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"第二台上此刻内存贴着上限、CPU 在涨的应当记 memory-cap-hit、重排跑完，实际各次（台、结局）{seen}、{describe(summary)}")
    elif caps[1] <= caps[0]:
        problems.append(f"重排那一次的上限应当比第一次大（乘 1.5），实际 {caps}")
    if "顶在上限上卡住" not in note or "此刻内存" not in note:
        problems.append(f"第一次的说明应当写「顶在上限上卡住」与第二台报的「此刻内存」，实际「{note}」")
    if problems:
        raise SelftestFailure("第二台贴着上限而 CPU 在涨：" + "；".join(problems), world.server_log_lines())
    return f"第二台报的此刻内存贴着上限、CPU 在涨：记 memory-cap-hit、上限 {caps} 重排跑完（「{note[:80]}」）"


def cell_squeezed_at_total_is_cap_hit(world):
    world.resources(local=(55, 50))   # 总上限 50G（真机上取多机配置 SLICE_MEMORY_CAP_BY_HOST 里本机那一项）
    world.seed_peak("selftest-squeeze-at-total", 48 * GIBIBYTE)   # 峰值正好是总上限减这台的假余量 2G：上限就是 48G
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=254,254,254,254"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-squeeze-at-total"})
    summary = world.finished(task_id)
    caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if [outcome for _, outcome in caps] != ["memory-cap-hit"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        raise SelftestFailure(f"上限已到收的那个数（总上限 50G 减这台的假余量 2G）的那一条被 slice 那一层杀（254）：就是它自己撞了上限，应当记 memory-cap-hit、再乘 1.5 就过总上限、打回 memory-cap-at-maximum（退 236），"
                              f"实际各次 {caps}、{describe(summary)}", world.server_log_lines())
    return "上限已到总上限的那一条被 slice 那一层杀，记撞上限、打回 memory-cap-at-maximum，不原样重排"


def cell_unmeasured_not_zero(world):
    key = "selftest-unmeasured"
    world.seed_peak(key, 3 * GIBIBYTE)   # 上限照它 3072M（近五次最大，不乘 1.5）
    world.start_server()
    caps, peaks = [], []
    for _ in range(RECORD_WINDOW_OVERRUN):
        summary = world.finished(world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key}))
        attempt = summary["attempts"][0] if summary["attempts"] else {}
        caps.append(attempt.get("memory_cap_text"))
        peaks.append(attempt.get("memory_peak", "缺这一栏"))
    if caps != ["3072M"] * RECORD_WINDOW_OVERRUN or peaks != [None] * RECORD_WINDOW_OVERRUN:
        raise SelftestFailure(f"短到量不到峰值的命令跑 {RECORD_WINDOW_OVERRUN} 次（比近几次的名额多）：每次都应当记「没量到」（null，不是 0），上限一直照记下的那一次 3072M，"
                              f"实际上限 {caps}、记的峰值 {peaks}")
    return f"量不到的 {RECORD_WINDOW_OVERRUN} 次都记 null、不占名额，上限一直 3072M"


def wrapper_cap_texts(world, task_id):
    """假内存包装收到的各次上限（调用记录里这一件的 wrapper 行的 cap=）。"""
    return [line.split(" cap=", 1)[1].split(" ", 1)[0] for line in world.calls_of(task_id) if " cap=" in line]


def cell_unfinished_peak_ignored(world):
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(UNFINISHED_PEAK_BYTES)})
    pinned_bytes = int(UNRECORDED_DEFAULT_PINNED_TEXT[:-1]) * MEBIBYTE
    problems = []
    for key, first_command, first_outcome in (("selftest-unfinished-command-failed", ["bash", "-c", "exit 1"], "command-failed"),
                                              ("selftest-unfinished-squeezed", ["bash", "-c", "exit 0", "fake-wrapper-exits=254"], "squeezed-out")):
        first = world.finished(world.submit(first_command, extra={lib.KEY_VARIABLE: key}))
        peaks = [attempt.get("memory_peak") for attempt in first["attempts"]]
        if first["task"].get("outcome") != first_outcome or not peaks or any(peak != UNFINISHED_PEAK_BYTES for peak in peaks):
            problems.append(f"{key}：头一件应当记 {first_outcome}、每次尝试都量到峰值 {UNFINISHED_PEAK_BYTES}，实际 {describe(first)}、峰值 {peaks}")
            continue
        second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
        second = world.finished(second_id)
        attempt = second["attempts"][0] if second["attempts"] else {}
        wrapper_caps = wrapper_cap_texts(world, second_id)
        if (second["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT
                or attempt.get("memory_cap") != pinned_bytes or attempt.get("prediction") != lib.UNRECORDED_HISTORY
                or wrapper_caps[:1] != [UNRECORDED_DEFAULT_PINNED_TEXT]):
            problems.append(f"{key}：只有一件 {first_outcome}（峰值 {UNFINISHED_PEAK_BYTES}）之后再交同一条，上限应当照没记录取 "
                            f"{UNRECORDED_DEFAULT_PINNED_TEXT}、prediction 记 {lib.UNRECORDED_HISTORY}，不取那次峰值 × 1.5，"
                            f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、包装收到 {wrapper_caps}、{describe(second)}")
    if problems:
        raise SelftestFailure("没跑完的尝试的峰值进了定额度：" + "；".join(problems), world.server_log_lines())
    return f"提前退 1 与被总上限挤掉的尝试峰值都不进定额度：再交同一条，上限照没记录取 {UNRECORDED_DEFAULT_PINNED_TEXT}"


def cell_completed_after_cap_hit(world):
    # 跑完那次的峰值大于撞过的上限的一例（取它本身），与小于撞过的上限的一例（取撞过的上限）：再交照近五次最大，不乘 1.5；
    # 假世界两次共用一个峰值，要到上限的一半才算真撞（cap_hit_measurement）
    cases = (("selftest-completed-above-cap-hit", 2500 * MEBIBYTE, "2500M"), ("selftest-completed-below-cap-hit", 1100 * MEBIBYTE, CAP_HIT_PINNED_TEXT))
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(cases[0][1])})
    problems = []
    for key, completed_peak, expected in cases:
        world.write_fake_scope(completed_peak)
        world.seed_peak(key, CAP_HIT_SEED_PEAK_BYTES)
        first = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key}))
        seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in first["attempts"]]
        if seen != [(CAP_HIT_PINNED_TEXT, "memory-cap-hit"), (CAP_HIT_GROWN_PINNED_TEXT, "succeeded")]:
            problems.append(f"{key}：头一件应当 {CAP_HIT_PINNED_TEXT} 撞上限、{CAP_HIT_GROWN_PINNED_TEXT} 跑完，两次尝试都在记录里，实际 {seen}")
            continue
        second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
        second = world.finished(second_id)
        cap = second["attempts"][0].get("memory_cap_text") if second["attempts"] else None
        if cap != expected or wrapper_cap_texts(world, second_id)[:1] != [expected]:
            problems.append(f"{key}：撞过 {CAP_HIT_PINNED_TEXT}、在 {CAP_HIT_GROWN_PINNED_TEXT} 跑完、跑完那次峰值 {completed_peak}，再交同一条上限应当是 {expected}"
                            f"（近五次最大：跑完那次的峰值与撞过的上限取大，不乘 1.5），实际 {cap}、包装收到 {wrapper_cap_texts(world, second_id)}")
    if problems:
        raise SelftestFailure("撞过上限后跑完的命令：" + "；".join(problems), world.server_log_lines())
    return f"撞过 {CAP_HIT_PINNED_TEXT} 后跑完：再交同一条，峰值 2500M 的取 2500M，峰值 1100M 的取撞过的 {CAP_HIT_PINNED_TEXT}"


def cell_cap_hit_starts_above(world):
    key = "selftest-cap-hit-only"
    # 本机 slice 总上限 4G、余量 0：2048M 撞了乘 1.5 到 3072M 再撞，再乘就过 4G、到顶收尾；这条命令只留两次撞上限的尝试（与库里记的那一次跑完的 2048M）
    world.resources(local=(30, 4), local_margin=0)
    world.seed_peak(key, CAP_HIT_SEED_PEAK_BYTES)
    world.start_server()
    first = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250"], extra={lib.KEY_VARIABLE: key}))
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in first["attempts"]]
    if seen != [(CAP_HIT_PINNED_TEXT, "memory-cap-hit"), (CAP_HIT_GROWN_PINNED_TEXT, "memory-cap-hit")] or first["task"].get("outcome") != "memory-cap-at-maximum":
        raise SelftestFailure(f"本机总上限 4G：头一件应当 {CAP_HIT_PINNED_TEXT}、{CAP_HIT_GROWN_PINNED_TEXT} 各撞一次上限、到顶记 memory-cap-at-maximum，实际 {seen}、{describe(first)}")
    world.resources(local=(30, 30))
    second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
    second = world.finished(second_id)
    attempt = second["attempts"][0] if second["attempts"] else {}
    wrapper_caps = wrapper_cap_texts(world, second_id)
    above = f"{int(int(CAP_HIT_GROWN_PINNED_TEXT[:-1]) * lib.CAP_GROWTH_FACTOR)}M"   # 撞过、之后这把键没跑完过：照撞过的最大上限再乘 1.5（cap_hit_measurement 的 unresolved）
    if second["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != above or wrapper_caps[:1] != [above]:
        raise SelftestFailure(f"撞过 {CAP_HIT_PINNED_TEXT} 与 {CAP_HIT_GROWN_PINNED_TEXT}、之后没跑完过的命令再交：第一次就应当从 {above} 起（撞过的最大上限乘 1.5），"
                              f"实际上限 {attempt.get('memory_cap_text')}、包装收到 {wrapper_caps}、{describe(second)}", world.server_log_lines())
    return f"撞过 {CAP_HIT_PINNED_TEXT} 与 {CAP_HIT_GROWN_PINNED_TEXT}、之后没跑完过的命令再交，第一次就从撞过的最大上限乘 1.5 的 {above} 起"


def cell_memory_cap_passed_to_command(world):
    key = "selftest-cap-to-command"
    world.seed_peak(key, GIBIBYTE)
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    seen = os.path.join(world.root, "seen-cap")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.MEMORY_CAP_VARIABLE}:-unset}}" > "$1"', "show-cap", seen], extra={lib.KEY_VARIABLE: key}))
    local_attempt = (local_summary["attempts"] or [{}])[0]
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-cap"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_attempt = (peer_summary["attempts"] or [{}])[0]
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if local_attempt.get("host") != "local" or local_seen != str(local_attempt.get("memory_cap")):
        problems.append(f"本机那一条：命令环境里的 {lib.MEMORY_CAP_VARIABLE} 应当等于这一次的上限 {local_attempt.get('memory_cap')}（字节），实际读到「{local_seen}」、{describe(local_summary)}")
    if peer_attempt.get("host") != "peer" or f"{lib.MEMORY_CAP_VARIABLE}={peer_attempt.get('memory_cap')}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.MEMORY_CAP_VARIABLE}={peer_attempt.get('memory_cap')}，"
                        f"实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机那一条命令读到 {lib.MEMORY_CAP_VARIABLE}={local_seen}（这一次的上限），第二台那一条经 env 那一段带过去 {peer_attempt.get('memory_cap')}"


def cell_pinned_cap_from_configuration(world):
    """钉值取多机配置（task_server.py 的 pinned_prediction 与 pin_value）：钉了之后起的那一次的上限就是 control pin-cap 报的此刻钉值；配置从 20G 改成 30G，
    pin-cap 报的钉值跟着多 10240M、起的那一次照它；撞了上限不放大、记 memory-cap-at-maximum；unpin-cap 之后照学到的记录（1536M）。"""
    world.set_slice_cap("20G")
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", "pinned-cap-probe"]
    hit = ["bash", "-c", "exit 0", "pinned-cap-hit", "fake-wrapper-exits=250,run"]
    world.seed_peak(PEAKS.command_key(probe, None), 1536 * MEBIBYTE)   # 学到的峰值 1536M：没钉时上限照它 1536M，钉了不顶掉钉值
    world.start_server()
    problems = []
    for command in (hit, probe):
        pinned = world.control(["pin-cap", "--", *command])
        if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
            problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    before = shown_local_pin(pinned)   # 配置写 20G 时 pin-cap 报的本机钉值

    def first_attempt(command):
        summary = world.finished(world.submit(command))
        return summary, (summary["attempts"] or [{}])[0]

    summary, attempt = first_attempt(probe)
    shown = world.client(["status", str(summary["task"]["id"])])
    if before is None or attempt.get("memory_cap_text") != f"{before}M" or attempt.get("prediction") != lib.PINNED_HISTORY or "钉值" not in shown.stdout:
        problems.append(f"钉了的命令第一次尝试的上限应当是 pin-cap 报的此刻钉值（配置两台都写 20G 时报 {before}M、prediction {lib.PINNED_HISTORY}）、status 写钉值，"
                        f"实际 {describe(summary)}、prediction {attempt.get('prediction')}、status「{shown.stdout.strip()[:300]}」")
    world.set_slice_cap("30G")
    again = world.control(["pin-cap", "--", *probe])   # 已钉着：照退 0、报配置改了之后此刻的钉值
    after = shown_local_pin(again)
    if before is None or after is None or after - before != 10 * 1024:
        problems.append(f"配置从 20G 改成 30G，pin-cap 报的本机钉值应当跟着多 10240M，实际改前 {before}M、改后 {after}M（「{(again.stdout + again.stderr).strip()[-300:]}」）")
    summary, attempt = first_attempt(probe)
    if after is None or attempt.get("memory_cap_text") != f"{after}M":
        problems.append(f"配置改成 30G 之后钉了的命令应当照此刻的钉值起（{after}M），实际 {describe(summary)}")
    summary = world.finished(world.submit(hit))
    caps = [item.get("memory_cap_text") for item in summary["attempts"]]
    if after is None or caps != [f"{after}M"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        problems.append(f"钉了的命令撞了上限不放大：只有一次 {after}M、结局 memory-cap-at-maximum（退 236），实际上限 {caps}、{describe(summary)}")
    unpinned = world.control(["unpin-cap", "--", *probe])
    summary, attempt = first_attempt(probe)
    if unpinned.returncode != 0 or attempt.get("memory_cap_text") != "1536M" or attempt.get("prediction") == lib.PINNED_HISTORY:
        problems.append(f"unpin-cap 之后照学到的峰值定（1536M），实际 unpin-cap 退 {unpinned.returncode}、{describe(summary)}、prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("钉值：" + "；".join(problems), world.server_log_lines())
    return (f"pin-cap 之后第一次上限就是 pin-cap 报的此刻钉值（{before}M）、改配置跟着变（{after}M）、撞上限不放大记 memory-cap-at-maximum；"
            "unpin-cap 之后照记录（1536M）")


def cell_pinned_cap_over_limit_reported(world):
    """钉值不超过 slice 总上限减余量（task_server.py 的 pin_value、pin_over_limit_lines；任务 10289 钉到 51200M 等于 slice 总上限）：配置写 60G、那一台总上限 50G、假余量 2G：
    control pin-cap 退 0、报本机 48G（49152M）、打一行「!」写超过与当多少用；起的那一次上限就是它。弄坏开关 pinned-over-limit-unreported 下报 58G、不打那一行，这一格红。"""
    world.set_slice_cap("60G")
    world.resources(local=(55, 50))
    probe = ["bash", "-c", "exit 0", "pinned-over-limit"]
    world.start_server()
    pinned = world.control(["pin-cap", "--", *probe])
    problems = []
    if pinned.returncode != 0 or shown_local_pin(pinned) != 49152 or "超过这台 slice 总上限减余量" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、报本机 49152M、打一行超过那一句，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-500:]}」")
    summary = world.finished(world.submit(probe))
    if first_attempt(summary).get("memory_cap_text") != "49152M":
        problems.append(f"起的那一次上限应当是 49152M，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("钉值超过 slice 总上限减余量：" + "；".join(problems), world.server_log_lines())
    return "配置写 60G、总上限 50G、余量 2G：pin-cap 回读报 49152M 与超过那一句，起的那一次照它"


def cell_pinned_squeezed_requeued(world):
    """钉了的命令被 slice 挤掉（task_server.py 文件头 ②「内存」的钉值；任务 10289 内存包装退 254、写「这一条的结果不算数，重跑」，服务却打回 memory-cap-at-maximum）：
    钉值等于这台单条能给的最大、第一次退 254：记 squeezed-out、同一上限重排、第二次跑完 succeeded。弄坏开关 pinned-squeeze-as-cap-hit 下记撞上限、打回，这一格红。"""
    world.set_slice_cap("50G")
    world.resources(local=(55, 50))
    command = ["bash", "-c", "exit 0", "pinned-squeezed", "fake-wrapper-exits=254,run"]
    world.start_server()
    pinned = world.control(["pin-cap", "--", *command])
    summary = world.finished(world.submit(command))
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if pinned.returncode != 0 or [outcome for _, outcome in seen] != ["squeezed-out", "succeeded"] or seen[0][0] != seen[1][0] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"钉了的命令退 254 应当记 squeezed-out、同一上限重排、跑完，实际 pin-cap 退 {pinned.returncode}、各次 {seen}、{describe(summary)}", world.server_log_lines())
    return "钉了的命令退 254 记 squeezed-out、同一上限重排、跑完，不打回"


def cell_pin_cap_help_not_pinned(world):
    """control pin-cap --help、-h 打用法、不钉（操作记录有一条把 --help 钉成命令的）。弄坏开关 pin-cap-help-pinned 下钉上了，这一格红。"""
    world.start_server()
    problems = []
    for word in ("--help", "-h"):
        shown = world.control(["pin-cap", word])
        if shown.returncode != 0 or "写成 python3 task-server/control pin-cap" not in shown.stdout or "钉上了" in shown.stdout:
            problems.append(f"pin-cap {word} 应当退 0、打用法、不钉，实际退 {shown.returncode}、「{(shown.stdout + shown.stderr).strip()[-300:]}」")
        left = world.control(["unpin-cap", "--", word])
        if left.returncode != 0 or "原来就没钉" not in left.stdout:
            problems.append(f"pin-cap {word} 之后命令 {word} 应当没钉，实际 unpin-cap 退 {left.returncode}、「{(left.stdout + left.stderr).strip()[-300:]}」")
    if problems:
        raise SelftestFailure("pin-cap 的用法：" + "；".join(problems), world.server_log_lines())
    return "pin-cap --help、-h 打用法退 0，没钉"


def cell_pinned_cap_unreadable_refused(world):
    """钉了上限而多机配置的 SLICE_MEMORY_CAP_BY_HOST 此刻读不到（两台那一项都写法不对）：这条不起、结局 run-condition-unmet（退 78），说明与 wait 的 TASK_SERVER_REASON 点名读不到的原因与出路；
    学到的峰值不顶上来（task_server.py 的 pinned_prediction）。弄坏开关 pin-unreadable-predicted 下照改前照学到的记录起、退 0。"""
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", "pinned-cap-unreadable-probe"]
    world.seed_peak(PEAKS.command_key(probe, None), GIBIBYTE)   # 学到的峰值 1G：照改前退回记录会起成 1536M
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_cap("lots")   # 写法不对：服务现读读不出数
    task_id = world.submit(probe)
    summary = world.finished(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or lib.PIN_SOURCE not in note or "写法不对" not in note or "unpin-cap" not in note):
        problems.append(f"钉了而 {lib.PIN_SOURCE} 写法不对：应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明点名 {lib.PIN_SOURCE} 写法不对与 unpin-cap 的出路，"
                        f"实际 {describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:300]}」")
    completed = world.client(["wait", str(task_id)])
    lines = completed.stdout.strip().splitlines()
    reason = next((line for line in lines if line.startswith(lib.REASON_MARK + " ")), "")
    if (completed.returncode != lib.RUN_CONDITION_EXIT or not lines or not lines[-1].startswith(f"{lib.RESULT_MARK} task={task_id} type=run-condition-unmet ")
            or "type=run-condition-unmet" not in reason or lib.PIN_SOURCE not in reason or "unpin-cap" not in reason):
        problems.append(f"wait 应当退 {lib.RUN_CONDITION_EXIT}、末行 type=run-condition-unmet、它前面的 {lib.REASON_MARK} 写读不到的原因与出路，"
                        f"实际退 {completed.returncode}、理由行「{reason[:300]}」、末行「{lines[-1] if lines else ''}」")
    if problems:
        raise SelftestFailure("钉值读不到：" + "；".join(problems), world.server_log_lines())
    return f"钉了而 {lib.PIN_SOURCE} 写法不对：一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、wait 的理由行点名读不到与 unpin-cap 的出路"


def cell_pinned_cap_by_host(world):
    """钉值按台（task_server.py 文件头 ②「内存」的钉值）：SLICE_MEMORY_CAP_BY_HOST 两台写不同的数（本机 20G、第二台 30G），钉值是那一项减那台的假余量 2G（pin_value、pinned_on_host）。
    只有本机时钉了的命令在本机起、上限 18432M；本机放不下、第二台放得下时在第二台起、上限 28672M（取第二台那一项）；control pin-cap 报两台各自的钉值。
    弄坏开关 pinned-cap-this-host-only 下两台都取本机那一项：第二台那一次也是 18432M；pinned-cap-at-slice-total 下照那一项记满：20480M、30720M。"""
    world.set_slice_caps(by_host_text("20G", "30G"))
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)   # pin-cap 报两台各自的钉值：两台的余量都要量得到
    probe = ["bash", "-c", "exit 0", "pinned-cap-by-host-probe"]
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    world.resources(local=(100, 100))   # 只有本机：钉了的那一条在本机起
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout or f"本机 {PINNED_LOCAL_TEXT}、第二台 {PINNED_PEER_TEXT}" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了、报两台各自的钉值（本机 {PINNED_LOCAL_TEXT}、第二台 {PINNED_PEER_TEXT}），"
                        f"实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    summary = world.finished(world.submit(probe))
    attempt = (summary["attempts"] or [{}])[0]
    if attempt.get("host") != "local" or attempt.get("memory_cap_text") != PINNED_LOCAL_TEXT or attempt.get("prediction") != lib.PINNED_HISTORY:
        problems.append(f"只有本机时钉了的命令应当在本机起、上限取本机那一项减余量 {PINNED_LOCAL_TEXT}（prediction {lib.PINNED_HISTORY}），实际 {describe(summary)}、"
                        f"台 {attempt.get('host')}、prediction {attempt.get('prediction')}")
    world.resources(local=(1, 100), peer=(100, 100), peer_cap_gibibytes=64)   # 本机只空 1G：钉了的那一条只放得进第二台
    summary = world.finished(world.submit(probe), timeout=PLACEMENT_WAIT_SECONDS)
    attempt = (summary["attempts"] or [{}])[0]
    if attempt.get("host") != "peer" or attempt.get("memory_cap_text") != PINNED_PEER_TEXT or attempt.get("prediction") != lib.PINNED_HISTORY:
        problems.append(f"本机放不下时钉了的命令应当在第二台起、上限取第二台那一项减余量 {PINNED_PEER_TEXT}（prediction {lib.PINNED_HISTORY}），实际 {describe(summary)}、"
                        f"台 {attempt.get('host')}、prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("钉值按台：" + "；".join(problems), world.server_log_lines())
    return f"钉值按台：在本机起取本机那一项减余量（{PINNED_LOCAL_TEXT}）、在第二台起取第二台那一项减余量（{PINNED_PEER_TEXT}），control pin-cap 报两台各自的钉值"


def by_host_written_wrong(world, value, wanted, probe_name):
    """按台的键写坏的那两格共用：钉了的命令起之前把 SLICE_MEMORY_CAP_BY_HOST 换成 value（服务已经起着），这一条应当一次都不起、结局 run-condition-unmet，
    说明里有 wanted（共用取法 multi_host_addresses.py 的 by_host_values 判的那一句）与 unpin-cap 的出路。→ 问题列表。"""
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", probe_name]
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_caps(value)
    task_id = world.submit(probe)
    summary = world.finished(task_id)
    note = summary["task"].get("note") or ""
    if (summary["task"].get("outcome") != "run-condition-unmet" or summary["task"].get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"]
            or world.calls_of(task_id) or wanted not in note or "unpin-cap" not in note):
        problems.append(f"{lib.PIN_SOURCE}={value}：钉了的命令应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明里有「{wanted}」与 unpin-cap 的出路，"
                        f"实际 {describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:400]}」")
    return problems


def cell_by_host_missing_host_refused(world):
    """按台的键少一台（task_server_lib.py 文件头「路径」按台的三个键）：SLICE_MEMORY_CAP_BY_HOST 只写了本机那一项 ⇒ 判写坏、点名缺的第二台，钉了的命令不起。
    弄坏开关 by-host-missing-accepted 下少了也照收（说明里没有「少了 <第二台> 那一台」）。"""
    problems = by_host_written_wrong(world, f"{SELFTEST_LOCAL_ADDRESS}=20G", f"少了 {SELFTEST_PEER_ADDRESS} 那一台", "by-host-missing-host-probe")
    if problems:
        raise SelftestFailure("按台的键少一台：" + "；".join(problems), world.server_log_lines())
    return f"{lib.PIN_SOURCE} 少了第二台那一项：判写坏、点名缺的那一台，钉了的命令一次都不起（run-condition-unmet）"


def cell_by_host_outside_address_refused(world):
    """按台的键多一项不在 HOSTS 里的地址（task_server_lib.py 文件头「路径」按台的三个键）：SLICE_MEMORY_CAP_BY_HOST 多写一项 192.0.2.99 ⇒ 判写坏、点名那一项，
    钉了的命令不起。弄坏开关 by-host-outside-accepted 下多了也照收（说明里没有「192.0.2.99 不在 HOSTS 里」）。"""
    outside = "192.0.2.99"
    problems = by_host_written_wrong(world, f"{by_host_text('20G', '20G')} {outside}=20G", f"{outside} 不在 {lib.HOSTS_KEY} 里", "by-host-outside-address-probe")
    if problems:
        raise SelftestFailure("按台的键多一项不在 HOSTS 里的地址：" + "；".join(problems), world.server_log_lines())
    return f"{lib.PIN_SOURCE} 多一项不在 {lib.HOSTS_KEY} 里的地址：判写坏、点名那一项，钉了的命令一次都不起（run-condition-unmet）"


MEMORY_FLOOR_RECORDED_SEED_BYTES = 1371 * MEBIBYTE   # retry-cap-not-floored：记下的那一次峰值，比下限 1024M 大、比 4G 小
MEMORY_FLOOR_RECORDED_TEXT = "1371M"       # 那一格头一次的上限：记下的峰值本身（近五次最大，不乘 1.5），有记录的不兜
MEMORY_FLOOR_RETRY_TEXT = "2057M"          # 那一格撞了重排那一次的上限：1371M × 1.5 按 MiB 向上取整，不兜到 4G
RECORDED_SMALL_PEAK_BYTES = 1300 * MEBIBYTE  # recorded-cap-not-floored：记下的那一次峰值（有记录、比下限 1024M 大）
RECORDED_SMALL_CAP_TEXT = "1300M"            # 记下的峰值本身（近五次最大，不乘 1.5）
# 内存上限的下限（lib.MEMORY_FLOOR_BYTES，1024M）：recorded-cap-not-floored 与 retry-cap-not-floored 两格照真的下限（小世界写 0 的换成空串）
FLOOR_TEXT = "1024M"
FLOOR_TINY_PEAK_BYTES = 5 * MEBIBYTE        # 记录的峰值很小（任务 11127 那一种）：照它只有 5M，装不下读文件带进来的页缓存，上限取下限
FLOOR_NEAR_PEAK_BYTES = 200 * MEBIBYTE      # 照它 200M，低于下限，上限仍取下限 1024M
FLOOR_RETRY_TEXT = "1536M"                  # 撞了重排那一次：1024M × 1.5
PINNED_BELOW_FLOOR_SLICE_TEXT = "2176M"     # 多机配置里两台那一项：减假余量 2G 是钉值 128M，低于下限
PINNED_BELOW_FLOOR_BYTES = 128 * MEBIBYTE


def cell_workspace_writer_not_retried(world):
    """改主工作区的命令（打补丁的 apply-writer-patch.py）撞了上限记 memory-cap-hit、被总上限挤掉记 squeezed-out，都只有一次尝试、打回，不原样重排
    （WORKSPACE_WRITER_NO_RETRY_OUTCOMES）；合入后验证 merge-and-verify.sh（只在快照上打与验，不算改主工作区）与 hook-swap --feed-only 的照乘 1.5 重排跑完
    （task_server.py 的 modifies_workspace、finish_attempt）。
    弄坏开关 workspace-writers-retried 下打补丁的那两条照重排，feed-only-counted-as-writer 下只喂的那一条、merge-and-verify-counted-as-writer 下合入后验证那一条只跑一次、打回，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    writer = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "fake-wrapper-exits=250,run"], cwd=world.repo))
    squeezed = world.finished(world.submit(["bash", "-c", "exit 0", "y", "scripts/gate/apply-writer-patch.py", "fake-wrapper-exits=254,run"], cwd=world.repo))
    skipped = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/merge-and-verify.sh", "fake-wrapper-exits=250,run"], cwd=world.repo))
    feed = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "--feed-only", "fake-wrapper-exits=250,run"], cwd=world.repo))
    if (writer["task"].get("outcome") != "memory-cap-hit" or len(writer["attempts"]) != 1
            or squeezed["task"].get("outcome") != "squeezed-out" or len(squeezed["attempts"]) != 1
            or skipped["task"].get("outcome") != "succeeded" or len(skipped["attempts"]) != 2
            or feed["task"].get("outcome") != "succeeded" or len(feed["attempts"]) != 2):
        raise SelftestFailure(f"打补丁的工具（apply-writer-patch.py）撞上限应当记 memory-cap-hit、被挤掉应当记 squeezed-out，都只有一次尝试；合入后验证（只在快照上打与验）与 hook-swap --feed-only 的应当乘 1.5 重排跑完；"
                              f"实际撞上限的 {describe(writer)}、被挤掉的 {describe(squeezed)}、合入后验证的 {describe(skipped)}、只喂的 {describe(feed)}", world.server_log_lines())
    return "改主工作区的打补丁工具撞上限与被挤掉都只有一次、照那一种结局打回；合入后验证与 hook-swap --feed-only 的乘 1.5 重排跑完"


def cell_retry_cap_not_floored(world):
    """有记录的照记录、不设 4G 底，只设 1024M 的下限（task_server.py 文件头 ②「内存」；这一格照真的下限，不用各格小世界的 0，弄坏开关 recorded-cap-floored 下兜的就是它）：
    同一命令键记下的峰值 1371M，头一次照记录取 1371M（近五次最大，不乘 1.5）、假包装退 250 撞上限，重排那一次照 1371M × 1.5 = 2057M（不兜到 4096M）、记账照上限、跑完；
    记下的峰值只有 5M 的命令头一次上限与记账都取下限 1024M（5M 低于下限）、撞上限重排那一次 1536M（1024M × 1.5）；
    钉了上限的命令（多机配置 SLICE_MEMORY_CAP_BY_HOST 两台都写 2176M）照 pin-cap 报的钉值 128M：低于下限也照钉值，不抬到 1024M。
    弄坏开关 recorded-cap-floored 下前两条头一次都是 4096M，cap-floor-skipped 下峰值 5M 的那条是 5M、8M，这一格红。"""
    key = "selftest-memory-floor"
    tiny_key = "selftest-memory-floor-tiny"
    world.seed_peak(key, MEMORY_FLOOR_RECORDED_SEED_BYTES)
    world.seed_peak(tiny_key, FLOOR_TINY_PEAK_BYTES)
    world.set_slice_cap(PINNED_BELOW_FLOOR_SLICE_TEXT)   # 钉值 = 那一项减这台的假余量 2G（pin_value）：128M，低于下限
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    retried = world.finished(world.submit(["bash", "-c", "exit 0", "memory-floor-retried", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key}))
    floored = world.finished(world.submit(["bash", "-c", "exit 0", "memory-floor-tiny-retried", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: tiny_key}))
    probe = ["bash", "-c", "exit 0", "memory-floor-pinned"]
    pinned_reply = world.control(["pin-cap", "--", *probe])
    shown = shown_local_pin(pinned_reply)
    pinned = world.finished(world.submit(probe))
    problems = []
    caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in retried["attempts"]]
    booked = [int(attempt.get("booked") or 0) for attempt in retried["attempts"]]
    retry_bytes = int(MEMORY_FLOOR_RETRY_TEXT[:-1]) * MEBIBYTE
    if (caps != [(MEMORY_FLOOR_RECORDED_TEXT, "memory-cap-hit"), (MEMORY_FLOOR_RETRY_TEXT, "succeeded")] or booked[1:] != [retry_bytes]
            or retried["task"].get("outcome") != "succeeded"):
        problems.append(f"记下 1371M 的命令头一次应当照记录 {MEMORY_FLOOR_RECORDED_TEXT} 撞上限，重排那一次上限 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限、跑完，"
                        f"实际各次 {caps}、记账 {booked}、{describe(retried)}")
    floored_caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in floored["attempts"]]
    floored_booked = [int(attempt.get("booked") or 0) for attempt in floored["attempts"]]
    floor_bytes = int(FLOOR_TEXT[:-1]) * MEBIBYTE
    floor_retry_bytes = int(FLOOR_RETRY_TEXT[:-1]) * MEBIBYTE
    if (floored_caps != [(FLOOR_TEXT, "memory-cap-hit"), (FLOOR_RETRY_TEXT, "succeeded")] or floored_booked != [floor_bytes, floor_retry_bytes]
            or floored["task"].get("outcome") != "succeeded"):
        problems.append(f"记下 5M 的命令头一次应当取下限 {FLOOR_TEXT} 撞上限，重排那一次上限 {FLOOR_RETRY_TEXT}（{FLOOR_TEXT} × 1.5）、记账各次照上限、跑完，"
                        f"实际各次 {floored_caps}、记账 {floored_booked}、{describe(floored)}")
    attempt = first_attempt(pinned)
    if (pinned_reply.returncode != 0 or shown != PINNED_BELOW_FLOOR_BYTES // MEBIBYTE or attempt.get("memory_cap_text") != f"{shown}M"
            or attempt.get("memory_cap") != PINNED_BELOW_FLOOR_BYTES or attempt.get("prediction") != lib.PINNED_HISTORY):
        problems.append(f"钉了上限的命令（SLICE_MEMORY_CAP_BY_HOST 两台都写 {PINNED_BELOW_FLOOR_SLICE_TEXT}）应当照 pin-cap 报的钉值（{PINNED_BELOW_FLOOR_BYTES // MEBIBYTE}M，低于下限 {FLOOR_TEXT}）、"
                        f"不抬到下限，实际 pin-cap 退 {pinned_reply.returncode}、报的钉值 {shown}M、上限 {attempt.get('memory_cap_text')}、"
                        f"prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("重排不设 4G 底、设 1024M 的下限：" + "；".join(problems), world.server_log_lines())
    return (f"记下 1371M 的命令键：头一次照记录 {MEMORY_FLOOR_RECORDED_TEXT}、撞了重排那一次 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限；"
            f"记下 5M 的头一次 {FLOOR_TEXT}、重排 {FLOOR_RETRY_TEXT}；配置写 {PINNED_BELOW_FLOOR_SLICE_TEXT} 的钉值照 {shown}M（低于下限也照钉值）")


def cell_recorded_cap_not_floored(world):
    """有真实峰值记录的上限照峰值定（task_server.py 文件头 ②「内存」的有记录的照记录；这一格照真的下限 1024M，不用各格小世界的 0）：同一命令键记下的峰值 1300M，
    上限与记账都是 1300M（近五次最大，不乘 1.5）、prediction 记 RECORD_HISTORY、内存包装收到 cap=1300M，不兜到 4096M；没记录的照旧 4096M；
    记下的峰值很小的（5M 与 200M，都不到下限）上限与记账都取下限 1024M，prediction 仍记 RECORD_HISTORY。
    弄坏开关 recorded-cap-floored 下记录定的也兜到 4096M，cap-floor-skipped 下峰值很小的两条是 5M 与 200M，这一格红。"""
    key = "selftest-recorded-small"
    tiny_key = "selftest-recorded-tiny"
    near_key = "selftest-recorded-near-floor"
    world.seed_peak(key, RECORDED_SMALL_PEAK_BYTES)
    world.seed_peak(tiny_key, FLOOR_TINY_PEAK_BYTES)
    world.seed_peak(near_key, FLOOR_NEAR_PEAK_BYTES)
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    recorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small"], extra={lib.KEY_VARIABLE: key})
    unrecorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small-unrecorded"])
    tiny_id = world.submit(["bash", "-c", "exit 0", "recorded-tiny"], extra={lib.KEY_VARIABLE: tiny_key})
    near_id = world.submit(["bash", "-c", "exit 0", "recorded-near-floor"], extra={lib.KEY_VARIABLE: near_key})
    expected = {"有记录的": (recorded_id, RECORDED_SMALL_CAP_TEXT, lib.RECORD_HISTORY),
                "没记录的": (unrecorded_id, UNRECORDED_DEFAULT_PINNED_TEXT, lib.UNRECORDED_HISTORY),
                "记下 5M 的": (tiny_id, FLOOR_TEXT, lib.RECORD_HISTORY),
                "记下 200M 的": (near_id, FLOOR_TEXT, lib.RECORD_HISTORY)}
    problems = []
    for name, (task_id, cap_text, history) in expected.items():
        summary = world.finished(task_id)
        attempt = first_attempt(summary)
        cap_bytes = int(cap_text[:-1]) * MEBIBYTE
        wrapper = wrapper_cap_texts(world, task_id)
        if (attempt.get("memory_cap_text") != cap_text or attempt.get("memory_cap") != cap_bytes or attempt.get("booked") != cap_bytes
                or attempt.get("prediction") != history or wrapper[:1] != [cap_text] or summary["task"].get("outcome") != "succeeded"):
            problems.append(f"{name}上限与记账都应当是 {cap_text}（{cap_bytes} 字节）、prediction 记 {history}、内存包装收到 cap={cap_text}、跑完，"
                            f"实际上限 {attempt.get('memory_cap_text')}（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')}、"
                            f"prediction {attempt.get('prediction')}、包装收到 {wrapper}、{describe(summary)}")
    if problems:
        raise SelftestFailure("有记录的照峰值定上限：" + "；".join(problems), world.server_log_lines())
    return (f"记下 1300M 的命令上限与记账都是 {RECORDED_SMALL_CAP_TEXT}（不兜到 4G），没记录的照旧 {UNRECORDED_DEFAULT_PINNED_TEXT}，"
            f"记下 5M 与 200M 的取下限 {FLOOR_TEXT}")


def cell_peer_memory_peak_recorded(world):
    """问题 86：第二台那条命令结束时 multi-host-run.sh 打 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<字节>：那次尝试记下这个峰值，同一命令下一次照它定上限
    （近五次最大 3G，3072M，不乘 1.5）；不打这一行的照旧记「没量到」（null）。"""
    world.resources(local=(2, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G，只有第二台放得下
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-peer-memory"}
    measured = world.finished(world.submit(["bash", "-c", "exit 0", f"fake-peer-memory-peak={3 * GIBIBYTE}"], extra=key, cwd=world.repo))
    silent = world.finished(world.submit(["bash", "-c", "exit 0", "selftest-peer-silent"], extra={lib.KEY_VARIABLE: "selftest-peer-silent"}, cwd=world.repo))
    again = world.finished(world.submit(["bash", "-c", "exit 0"], extra=key, cwd=world.repo))
    first, quiet, next_one = (summary["attempts"][0] if summary["attempts"] else {} for summary in (measured, silent, again))
    problems = []
    if first.get("host") != "peer" or first.get("memory_peak") != 3 * GIBIBYTE:
        problems.append(f"打了那一行的尝试应当在第二台、记峰值 {3 * GIBIBYTE}，实际在 {first.get('host')}、记 {first.get('memory_peak')}")
    if quiet.get("host") != "peer" or quiet.get("memory_peak", "缺这一栏") is not None:
        problems.append(f"没打那一行的应当记「没量到」（null），实际在 {quiet.get('host')}、记 {quiet.get('memory_peak', '缺这一栏')}")
    if next_one.get("memory_cap_text") != "3072M":
        problems.append(f"同一命令下一次应当照第二台量到的 3G 定上限 3072M，实际 {next_one.get('memory_cap_text')}")
    if problems:
        raise SelftestFailure("第二台的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "第二台打了峰值那一行的记下 3G、下一次上限 3072M；不打的记 null"


SHORT_EXIT_PEAK_BYTES = 123 * MEBIBYTE   # short-command-peak-recorded：假包装演的退出时峰值
SHORT_SAMPLED_PEAK_BYTES = 200 * MEBIBYTE   # 同一格后半段：按轮采到的假 scope 峰值


def cell_short_command_peak_recorded(world):
    """跑 0.1 秒就退的命令：按轮一次都采不到（假 scope 是空目录），结局的 memory_peak 照内存包装退出时交的那一个记（attempt_runner.py 文件头 ⑥，
    假包装照 fake-wrapper-exit-peak= 往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写），不是 null；两样都没有的照旧 null。
    后半段换上进程内存 200M 的假 scope 重起服务：按轮采到了的与退出时交的（扣了页缓存）取大的（attempt_runner.py 文件头 ⑥；一轮之间冒出来的峰值在退出时交的那一个里）。
    弄坏开关 exit-peak-only-when-unsampled 下按轮采到了就不看它，「400M 比 200M 大」那一例记 200M，这一格红。"""
    world.start_server()
    short = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={SHORT_EXIT_PEAK_BYTES}"], extra={lib.KEY_VARIABLE: "selftest-short-peak"}))
    silent = world.finished(world.submit(["bash", "-c", "sleep 0.1", "short-silent"], extra={lib.KEY_VARIABLE: "selftest-short-silent"}))
    world.stop_server()
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(SHORT_SAMPLED_PEAK_BYTES)})
    larger = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={2 * SHORT_SAMPLED_PEAK_BYTES}"],
                                         extra={lib.KEY_VARIABLE: "selftest-short-larger"}))
    smaller = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={SHORT_SAMPLED_PEAK_BYTES // 2}"],
                                          extra={lib.KEY_VARIABLE: "selftest-short-smaller"}))
    problems = []
    for name, summary, wanted in (("按轮采不到、退出时读到 123M 的", short, SHORT_EXIT_PEAK_BYTES), ("两样都没有的", silent, None),
                                  ("退出时交的 400M 比按轮的 200M 大的（照大的）", larger, 2 * SHORT_SAMPLED_PEAK_BYTES),
                                  ("退出时交的 100M 比按轮的 200M 小的（照大的）", smaller, SHORT_SAMPLED_PEAK_BYTES)):
        attempt = first_attempt(summary)
        if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_peak", "缺这一栏") != wanted:
            problems.append(f"{name}应当跑完、记 memory_peak {wanted}，实际 {describe(summary)}、记 {attempt.get('memory_peak', '缺这一栏')}")
    if problems:
        raise SelftestFailure("跑得短的命令的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "跑 0.1 秒就退、按轮采不到的记了退出时交的 123M；两样都没有的记 null；按轮采到了的与退出时交的取大的（400M、200M）"


PEAK_STAT_KEY = "selftest-peak-memory-stat"
PEAK_STAT_ANON_BYTES = 9216 * MEBIBYTE   # peak-memory-stat-in-note：峰值时的堆
PEAK_STAT_FILE_BYTES = 1024 * MEBIBYTE   # 同一刻的页缓存
PEAK_STAT_TEXTS = ("内存最高那一刻", "anon 9216M", "file 1024M", "大头是进程自己的内存")
# 命令跑到一半把假 scope 换成被杀之后的样子（堆已释放，任务 10279：峰值 45.46 GiB、退出时读到 anon 311296），再退 250
PEAK_STAT_KILLED_SCRIPT = ('printf "%s\\n" "$2" > "$1/memory.current"; printf "anon %s\\nfile %s\\nshmem 0\\n" "$3" "$4" > "$1/memory.stat"; sleep 0.6; '
                           'printf "4096\\n" > "$1/memory.current"; printf "anon 0\\nfile 0\\nshmem 0\\n" > "$1/memory.stat"; sleep 0.5; exit 250')


def cell_peak_memory_stat_in_note(world):
    """内存最高那一刻的明细（task_server.py 文件头 ①「内存最高那一刻」）：假 scope 峰值时 anon 9216M、file 1024M，命令退 250 之前换成 anon 0（被杀之后读的样子）：
    总上限 50G、记录峰值 20G，两次撞上限到顶记 memory-cap-at-maximum，结局说明与 wait 那一行理由都写峰值那一刻的读数与「大头是进程自己的内存」。
    弄坏开关 peak-memory-stat-read-at-exit 下留最后读到的 anon 0，cap-hit-stat-unreported 下不抄，这一格红。"""
    world.resources(local=(55, 50))
    world.seed_peak(PEAK_STAT_KEY, 20 * GIBIBYTE)
    scope = os.path.join(world.root, "peak-stat-scope")
    os.makedirs(scope)
    current = PEAK_STAT_ANON_BYTES + PEAK_STAT_FILE_BYTES
    with open(os.path.join(scope, "memory.current"), "w", encoding="utf-8") as handle:
        handle.write(f"{current}\n")
    with open(os.path.join(scope, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(PEAK_STAT_ANON_BYTES, PEAK_STAT_FILE_BYTES))
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    task_id = world.submit(["bash", "-c", PEAK_STAT_KILLED_SCRIPT, "peak-stat", scope, str(current), str(PEAK_STAT_ANON_BYTES), str(PEAK_STAT_FILE_BYTES)],
                           extra={"RUN_WITH_MEMORY_CAP_KEY": PEAK_STAT_KEY})
    summary = world.finished(task_id)
    note = summary["task"].get("note") or ""
    completed = world.client(["wait", str(task_id)])
    reason = next((line for line in completed.stdout.splitlines() if line.startswith(lib.REASON_MARK)), "")
    problems = []
    if summary["task"].get("outcome") != "memory-cap-at-maximum":
        problems.append(f"两次撞上限应当到顶记 memory-cap-at-maximum，实际 {describe(summary)}")
    for where, text in (("结局说明", note), ("wait 的理由行", reason)):
        missing = [wanted for wanted in PEAK_STAT_TEXTS if wanted not in text]
        if missing:
            problems.append(f"{where}应当写内存最高那一刻的明细（缺 {missing}），实际「{text[:500]}」")
    if problems:
        raise SelftestFailure("内存最高那一刻的明细：" + "；".join(problems), world.server_log_lines())
    return "假 scope 峰值时堆大、被杀后读 0：结局说明与 wait 的理由行写峰值那一刻的 anon 9216M、file 1024M，大头是进程自己的内存"


def peer_world(world, key):
    """派到第二台那几格的世界：本机只空 8G、第二台 100G，这把命令键记 10G（上限 15G）。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak(key, PEER_PINNED_SEED_PEAK_BYTES)


PEER_STAT_KEY = "selftest-peer-peak-stat"
PEER_STAT_SCRIPT = ('echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=1 current_bytes=$(( $1 + $2 )) anon=$1 file=$2 file_dirty=0 file_writeback=0 shmem=0 read=yes"; '
                    'echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=2 current_bytes=4096 anon=0 file=0 file_dirty=0 file_writeback=0 shmem=0 read=yes"; '
                    'echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=3 current_bytes=0 anon=0 file=0 file_dirty=0 file_writeback=0 shmem=0 read=no"; exit 250')


def cell_peer_peak_memory_stat_in_note(world):
    """第二台内存最高那一刻的明细（task_server.py 文件头 ①「内存最高那一刻」；multi-host-run.sh 每轮一行 MULTI_HOST_RUN_PEER_MEMORY_STAT）：头一行堆大、后两行被杀之后读 0、read=no：
    第 1 次尝试记的 peak_memory_note 写头一行的读数。弄坏开关 peak-memory-stat-read-at-exit 下留最后一行 read=yes 的 anon 0，cap-hit-stat-unreported 下不记，这一格红。"""
    peer_world(world, PEER_STAT_KEY)
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", PEER_STAT_SCRIPT, "peer-stat", str(PEAK_STAT_ANON_BYTES), str(PEAK_STAT_FILE_BYTES), "fake-peer-run"],
                                          extra={lib.KEY_VARIABLE: PEER_STAT_KEY}, cwd=world.repo), timeout=60)
    attempt = first_attempt(summary)
    note = attempt.get("peak_memory_note") or ""
    missing = [wanted for wanted in PEAK_STAT_TEXTS if wanted not in note]
    if attempt.get("host") != "peer" or attempt.get("outcome") != "memory-cap-hit" or missing:
        raise SelftestFailure(f"第二台撞上限的那一次应当记内存最高那一刻的明细（缺 {missing}），实际第 1 次在 {attempt.get('host')}、{attempt.get('outcome')}、记「{note[:300]}」", world.server_log_lines())
    return "第二台每轮报的明细留 current_bytes 最大的那一行：头一行堆大、后面读 0，记的是头一行的读数"


FAKE_APPLY_WRITER_PATCH = r'''#!/usr/bin/env python3
"""task-server 自证的假 apply-writer-patch.py：只认 --read-outcome <结局文件>：里面有一行以「完成<TAB>」起头的打「打全了」、退 0，
没有的逐行打「没打完」、退 3，读不了退 2（真的那一份的退出码，apply-writer-patch.py 文件头「结局文件」）。别的用法退 0、什么都不写（自证里命令本身由假包装演结局）。"""
import sys

if len(sys.argv) >= 3 and sys.argv[1] == "--read-outcome":
    try:
        with open(sys.argv[2], encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        print(f"结局文件读不了：{error}")
        sys.exit(2)
    if any(line.startswith("完成\t") for line in lines):
        print("打全了")
        sys.exit(0)
    print("没打完：" + "；".join(line.replace("\t", " ") for line in lines if line.startswith("文件\t")))
    sys.exit(3)
sys.exit(0)
'''
FAKE_HOOK_SWAP = r'''#!/usr/bin/env python3
"""task-server 自证的假 hook-swap.py：只认 --read-outcome [--root <仓根>] [--scratch <草稿目录>]：读 <仓根>/fake-hook-swap-outcome.txt（不给 --root 的是这份脚本往上三层的假仓根），
头一行是退出码、其余几行照打；没有这份打「读不了」、退 2（真的那一份没有结局文件时的退出码，hook-swap.py 文件头「结局文件」）。别的用法退 0、什么都不写。"""
import os
import sys

arguments = sys.argv[1:]
if "--read-outcome" in arguments:
    root = (arguments[arguments.index("--root") + 1] if "--root" in arguments[:-1]
            else os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    try:
        with open(os.path.join(root, "fake-hook-swap-outcome.txt"), encoding="utf-8") as handle:
            code, *lines = handle.read().splitlines()
    except OSError as error:
        print(f"✗ 读不了：{error}")
        sys.exit(2)
    print("\n".join(lines))
    sys.exit(int(code))
sys.exit(0)
'''
FAKE_HOOK_SWAP_OUTCOME_NAME = "fake-hook-swap-outcome.txt"   # FAKE_HOOK_SWAP 在假仓根里读的那一份：头一行退出码，其余是 --read-outcome 打的行
HALF_APPLIED_OUTCOME = "开始\t/selftest/patch\t/selftest/repo\t1\t0\n文件\tcrates/a.rs\t改\taaaa\tbbbb\n"
FULLY_APPLIED_OUTCOME = HALF_APPLIED_OUTCOME + "回读\tcrates/a.rs\tbbbb\n完成\t1\t1\n"


# ── 近几次最大、打补丁脚本干跑照常重排、第二台重试接着用编译目录（task_server_lib.py 文件头「资源记录」的定额度、「编译目录」的重试复用）──
HISTORY_CAP_HIT_BYTES = 13824 * MEBIBYTE    # history-peak-first-attempt 与 history-peak-window-raises：同键撞过的上限（任务 7349、7416 撞过的那一档）
HISTORY_CAP_HIT_TEXT = "13824M"             # 照撞过的上限本身，不乘 1.5
HISTORY_SMALL_PEAK_BYTES = GIBIBYTE         # 跑成的小峰值：history-peak-first-attempt 里近 RECORD_WINDOW 次只有它们，起点回落到 1G
HISTORY_SMALL_PEAK_TEXT = "1024M"


def cell_cap_hit_low_peak_not_recorded_at_cap(world):
    """撞上限那一次峰值远低于上限（task_server.py 的 cap_hit_measurement；任务 10731：上限 30.4G、峰值 1.0G）：同键上一次上限 13824M、峰值 1G 记了撞上限，
    新提交的头一次按峰值起在 1G（1024M）、跑完。弄坏开关 cap-hit-low-peak-counted-at-cap 下照上限起在 13824M，这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-cap-hit-low-peak"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES, memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_SMALL_PEAK_TEXT:
        raise SelftestFailure(f"同键上一次上限 13824M、峰值 1G 记了撞上限：新提交的头一次应当起在 {HISTORY_SMALL_PEAK_TEXT}、跑完，实际上限 {attempt.get('memory_cap_text')}、{describe(summary)}",
                              world.server_log_lines())
    return f"撞上限那一次峰值远低于上限：新提交的头一次按峰值起在 {HISTORY_SMALL_PEAK_TEXT}"


def cell_history_peak_first_attempt(world):
    """起点能降（task_server.py 的 recorded_prediction 只取近 RECORD_WINDOW 次）：同键早先撞过 13824M，
    之后 RECORD_WINDOW 次跑成的峰值都是 1G：撞过的那一次出了窗口，新提交的头一次回落到 1G（1024M）、prediction 记 record，跑完。"""
    world.resources(local=(40, 40))
    key = "selftest-history-first"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(5):   # task_server.py 的 RECORD_WINDOW：近几次的名额全是小峰值
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_SMALL_PEAK_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键早先撞过 13824M、之后五次跑成都是 1G：撞过的那一次出了窗口，新提交的头一次应当回落到 {HISTORY_SMALL_PEAK_TEXT}、"
                              f"prediction 记 {lib.RECORD_HISTORY}、跑完，实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}",
                              world.server_log_lines())
    return f"同键早先撞过 13824M、之后五次跑成都是 1G：新提交的头一次回落到 {HISTORY_SMALL_PEAK_TEXT}"


def cell_history_peak_window_raises(world):
    """起点能升（task_server.py 的 recorded_prediction）：同键近 RECORD_WINDOW 次里有一次撞过 13824M（之后又跑成四次 1G，撞过的那一次还在窗口里），
    新提交的头一次上限 13824M、prediction 记 record，跑完。弄坏开关 cap-hit-not-counted 下撞过的那一次不进定额度、起在 1024M，这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-history-window"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(4):   # task_server.py 的 RECORD_WINDOW 是 5：撞过的那一次与四次小峰值都在窗口里
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_CAP_HIT_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键近五次里撞过 13824M、另四次跑成 1G：新提交的头一次应当起在 {HISTORY_CAP_HIT_TEXT}、prediction 记 {lib.RECORD_HISTORY}、跑完，"
                              f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}", world.server_log_lines())
    return f"窗口里刚撞过 13824M：新提交的头一次起在 {HISTORY_CAP_HIT_TEXT}"


def cell_history_peak_unrecorded_default(world):
    """没有任何记录的命令从没记录的 4096M 起（task_server.py 的 unrecorded_prediction）：prediction 记 default。
    弄坏开关 unrecorded-default-grown 下照 4096M × 1.5 起。"""
    world.resources(local=(40, 40))
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: "selftest-history-unrecorded"}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT
            or attempt.get("prediction") != lib.UNRECORDED_HISTORY):
        raise SelftestFailure(f"没有任何记录的命令应当照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起、prediction 记 {lib.UNRECORDED_HISTORY}，"
                              f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}", world.server_log_lines())
    return f"没有任何记录的命令照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起"


def cell_writer_dry_run_retried(world):
    """改工作区的工具带只读参数的不算改主工作区（task_server.py 的 modifies_workspace、WORKSPACE_READ_ONLY_ARGUMENTS）：apply-writer-patch.py --dry-run 只在临时树上核
    （任务 7442、7444）、stage-mine.py --selftest 只跑自证，撞了上限都照常乘 1.5 重排跑完；不带只读参数的真打照旧只有一次尝试、记 memory-cap-hit 打回。
    弄坏开关 read-only-arguments-counted-as-writer 下干跑与自证也只有一次、打回。"""
    world.resources(local=(30, 30))
    world.start_server()
    dry = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-dir", "--dry-run", "fake-wrapper-exits=250,run"], cwd=world.repo))
    selftest = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/stage-mine.py", "--selftest", "fake-wrapper-exits=250,run"], cwd=world.repo))
    real = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-dir", "fake-wrapper-exits=250,run"], cwd=world.repo))
    if (dry["task"].get("outcome") != "succeeded" or len(dry["attempts"]) != 2
            or selftest["task"].get("outcome") != "succeeded" or len(selftest["attempts"]) != 2
            or real["task"].get("outcome") != "memory-cap-hit" or len(real["attempts"]) != 1):
        raise SelftestFailure(f"打补丁脚本干跑、暂存脚本自证撞上限应当乘 1.5 重排跑完（两次尝试），真打撞上限应当只有一次、记 memory-cap-hit；"
                              f"实际干跑 {describe(dry)}、自证 {describe(selftest)}、真打 {describe(real)}", world.server_log_lines())
    return "打补丁脚本干跑、暂存脚本自证撞上限照常重排跑完；真打撞上限只有一次、打回"


# 认包着的写工作区工具那几格（撞上限重排的 wrapped-writer-*-retried 三格与提醒的 wrapped_swap_scheduler 三格）的串行脚本正文：写着的工具前面先 exit 0
# （重排那一次的 run 起它时当场退 0，不起写着的工具），写着的那几行只供服务读正文认
WRAPPED_WRITER_SHELL_BODY = ("#!/usr/bin/env bash\n"
                             "set -e\n"
                             "exit 0\n"
                             "python3 scripts/gate/hook-swap.py --spec \"$1\" --scratch swap-scratch\n")
WRAPPED_WRITER_PYTHON_BODY = ("import subprocess\n"
                              "raise SystemExit(0)\n"
                              "subprocess.run([\"python3\", \"scripts/gate/apply-writer-patch.py\", \"patch-dir\"], check=True)\n")
# 只在注释、引号包着的整句与别的命令的参数里提到工具名，命令位置上没有写工作区工具
WRAPPED_WRITER_MENTION_BODY = ("#!/usr/bin/env bash\n"
                               "# 换上用 python3 scripts/gate/hook-swap.py --spec <规格>\n"
                               "exit 0\n"
                               "echo \"python3 scripts/gate/hook-swap.py --spec spec.json\"\n"
                               "printf '%s\\n' scripts/gate/stage-mine.py\n")
# 命令位置上的写工作区工具都带只读参数或 --feed-only
WRAPPED_WRITER_READ_ONLY_BODY = ("#!/usr/bin/env bash\n"
                                 "exit 0\n"
                                 "python3 scripts/gate/stage-mine.py --selftest\n"
                                 "python3 scripts/gate/hook-swap.py --feed-only --spec spec.json\n")


def wrapped_writer_attempt_problems(world, cases):
    """撞上限重排的 wrapped-writer-*-retried 三格共用：逐条交 (说明, 命令, 算不算改主工作区)，命令都带 fake-wrapper-exits=250,run（头一次撞上限）。算的应当只有一次尝试、记 memory-cap-hit；
    不算的应当乘 1.5 重排、两次尝试、succeeded。→ 对不上的那几句。"""
    problems = []
    for what, command, writer in cases:
        summary = world.finished(world.submit(command, cwd=world.repo))
        wanted = ("memory-cap-hit", 1) if writer else ("succeeded", 2)
        if (summary["task"].get("outcome"), len(summary["attempts"])) != wanted:
            problems.append(f"{what}：应当结局 {wanted[0]}、{wanted[1]} 次尝试，实际 {describe(summary)}")
    return problems


def cell_wrapped_writer_shell_string_not_retried(world):
    """bash -c 的串里写着改主工作区的工具（task_server_scheduler_building.py 的 wrapped_workspace_commands、shell_text_pieces）：set -e 起头、分号隔开的 hook-swap.py，
    sh -ec 换行隔开的 stage-mine.py，同一个串里先一条 hook-swap.py --feed-only、&& 之后一条真换的，撞了上限都只有一次尝试、记 memory-cap-hit 打回
    （串里写着的工具前面先 exit 0，重排那一次不起它们）。弄坏开关 shell-string-writers-unread、workspace-writers-retried 下照乘 1.5 重排跑完，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    plan = "fake-wrapper-exits=250,run"
    problems = wrapped_writer_attempt_problems(world, (
        ("分号隔开的换钩子", ["bash", "-c", "set -e; exit 0; python3 scripts/gate/hook-swap.py --spec spec.json", "x", plan], True),
        ("sh -ec 换行隔开的暂存", ["sh", "-ec", "exit 0\npython3 scripts/gate/stage-mine.py --match selftest", "x", plan], True),
        ("只喂之后真换", ["bash", "-c", "exit 0; python3 scripts/gate/hook-swap.py --feed-only --spec a.json && python3 scripts/gate/hook-swap.py --spec b.json", "x", plan],
         True)))
    if problems:
        raise SelftestFailure("bash -c 串里写着改主工作区的工具撞上限不重排：" + "；".join(problems), world.server_log_lines())
    return "bash -c、sh -ec 串里分号、换行、&& 隔开的换钩子与暂存撞上限都只有一次、打回；只喂那一条不免同一个串里真换的那一条"


def cell_wrapped_writer_script_body_not_retried(world):
    """命令位置上起的串行脚本正文里写着改主工作区的工具（task_server_scheduler_building.py 的 script_body_workspace_commands）：工作根下的 bash 脚本写着 hook-swap.py
    （bash <它>、直接起它两种），仓里的 python 脚本 subprocess 起 apply-writer-patch.py（python3 <它>），撞了上限都只有一次尝试、记 memory-cap-hit 打回；
    正文里只在注释、引号包着的整句与别的命令的参数里提到工具名的，照乘 1.5 重排跑完（写着的工具前面先 exit 0，重排那一次不起它们）。
    弄坏开关 script-body-writers-unread、workspace-writers-retried 下写着工具的那三条也重排跑完，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    chain = os.path.join(world.work_root, "swap-chain.sh")
    mention = os.path.join(world.work_root, "mention-only.sh")
    write_executable(chain, WRAPPED_WRITER_SHELL_BODY)
    write_executable(mention, WRAPPED_WRITER_MENTION_BODY)
    write_executable(os.path.join(world.repo, "patch-chain.py"), WRAPPED_WRITER_PYTHON_BODY)
    plan = "fake-wrapper-exits=250,run"
    problems = wrapped_writer_attempt_problems(world, (
        ("bash 起工作根下的换钩子脚本", ["bash", chain, "spec.json", plan], True),
        ("直接起工作根下的换钩子脚本", [chain, "spec.json", plan], True),
        ("python3 起仓里的打补丁脚本", ["python3", "patch-chain.py", plan], True),
        ("只提到工具名的脚本", ["bash", mention, plan], False)))
    if problems:
        raise SelftestFailure("串行脚本正文里写着改主工作区的工具撞上限不重排：" + "；".join(problems), world.server_log_lines())
    return "bash 起、直接起、python3 起的串行脚本正文里写着换钩子、打补丁的撞上限都只有一次、打回；只提到工具名的照常重排跑完"


def cell_wrapped_writer_read_only_retried(world):
    """包在里面的写工作区工具带只读参数、只喂的，照命令那一层同一种例外不算改主工作区（task_server_scheduler_building.py 的 writer_command_verdict）：
    bash -c 串里的 apply-writer-patch.py --dry-run 与 hook-swap.py --feed-only，工作根下的串行脚本正文里的 stage-mine.py --selftest 与 hook-swap.py --feed-only，
    撞了上限都照乘 1.5 重排跑完。弄坏开关 read-only-arguments-counted-as-writer 下干跑那一条与那份脚本、feed-only-counted-as-writer 下只喂那一条与那份脚本只跑一次、打回，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    reader = os.path.join(world.work_root, "read-only-chain.sh")
    write_executable(reader, WRAPPED_WRITER_READ_ONLY_BODY)
    plan = "fake-wrapper-exits=250,run"
    problems = wrapped_writer_attempt_problems(world, (
        ("串里干跑的打补丁", ["bash", "-c", "exit 0; python3 scripts/gate/apply-writer-patch.py patch-dir --dry-run", "x", plan], False),
        ("串里只喂的换钩子", ["bash", "-c", "exit 0; python3 scripts/gate/hook-swap.py --feed-only --spec spec.json", "x", plan], False),
        ("正文里自证与只喂的脚本", ["bash", reader, plan], False)))
    if problems:
        raise SelftestFailure("包在里面的只读、只喂的写工作区工具撞上限应当照常重排：" + "；".join(problems), world.server_log_lines())
    return "bash -c 串里干跑与只喂的、串行脚本正文里自证与只喂的撞上限照常重排跑完"


def write_patch_writer_world(world):
    """这一格世界的假仓根里放假 apply-writer-patch.py 与两份补丁目录（write_patch_writer_tree）。"""
    write_patch_writer_tree(world.repo)


def write_patch_writer_tree(repo):
    """给的仓根里放假 apply-writer-patch.py 与两份补丁目录：patch-half 的结局文件没有完成（打了一半），patch-full 的有完成（打全了）。"""
    gate = os.path.join(repo, "scripts", "gate")
    os.makedirs(gate, exist_ok=True)
    write_executable(os.path.join(gate, "apply-writer-patch.py"), FAKE_APPLY_WRITER_PATCH)
    for name, text in (("patch-half", HALF_APPLIED_OUTCOME), ("patch-full", FULLY_APPLIED_OUTCOME)):
        os.makedirs(os.path.join(repo, name), exist_ok=True)
        with open(os.path.join(repo, name, SERVER.PATCH_WRITER_OUTCOME_NAME), "w", encoding="utf-8") as handle:
            handle.write(text)


def cell_workspace_write_interrupted_reported(world):
    """改主工作区的命令没正常结束、可能写了一半，wait 报一行 workspace-write-interrupted（task_server.py 的 workspace_write_item）：apply-writer-patch.py 撞了上限
    而补丁目录的结局文件没有完成（假 --read-outcome 退 3），wait 在理由行之前打 TASK_SERVER_ATTENTION、点名补丁目录、退码仍是任务的（250）；
    stage-mine.py 被信号杀（命令自己 kill -KILL 自己，退 137）同样报。弄坏开关 workspace-write-interrupted-unreported 下不报。"""
    write_patch_writer_world(world)
    world.start_server()
    problems = []
    for name, command, code in (("打了一半的 apply-writer-patch.py", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "fake-wrapper-exits=250"], 250),
                                ("被信号杀的 stage-mine.py", ["bash", "-c", "kill -KILL $$", "x", "scripts/gate/stage-mine.py", "fake-wrapper-exits=run"], 137)):
        task_id = world.submit(command, cwd=world.repo)
        world.finished(task_id)
        completed = world.client(["wait", str(task_id)])
        lines = attention_lines(completed)
        result = [line for line in completed.stdout.splitlines() if line.startswith(f"{lib.RESULT_MARK} ")]
        if (completed.returncode != code or len(lines) != 1 or "kind=workspace-write-interrupted" not in lines[0]
                or ("patch-half" not in lines[0] and "apply-writer-patch" in name) or not result):
            problems.append(f"{name}：wait 应当退 {code}、在末行之前打一行 kind=workspace-write-interrupted 的提醒（点名补丁目录），"
                            f"实际退 {completed.returncode}、提醒 {lines}、末行 {result}")
    if problems:
        raise SelftestFailure("改主工作区的命令没正常结束：" + "；".join(problems), world.server_log_lines())
    return "打了一半的 apply-writer-patch.py 撞上限、被信号杀的 stage-mine.py：wait 各打一行 workspace-write-interrupted，退码仍是任务的"


def cell_workspace_write_complete_not_reported(world):
    """打全了不报（task_server.py 的 workspace_write_item）：apply-writer-patch.py 撞了上限而补丁目录的结局文件有完成（假 --read-outcome 退 0）、带 --dry-run 的、
    正常跑完的，wait 都不打 workspace-write-interrupted。弄坏开关 apply-outcome-ignored 下不读结局文件，打全了的也报。"""
    write_patch_writer_world(world)
    world.start_server()
    problems = []
    for name, command in (("打全了的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-full", "fake-wrapper-exits=250"]),
                          ("--dry-run 的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "--dry-run", "fake-wrapper-exits=250"]),
                          ("正常跑完的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "fake-wrapper-exits=0"])):
        task_id = world.submit(command, cwd=world.repo)
        world.finished(task_id)
        lines = attention_lines(world.client(["wait", str(task_id)]))
        if lines:
            problems.append(f"{name} apply-writer-patch.py 不该报 workspace-write-interrupted，实际 {lines}")
    if problems:
        raise SelftestFailure("改主工作区的命令打全了不报：" + "；".join(problems), world.server_log_lines())
    return "结局文件写着完成的、--dry-run 的、正常跑完的 apply-writer-patch.py 都不报"


def cell_workspace_write_blocked_reminded(world):
    """换钩子被上一趟没写完成的挡住，提醒发给被挡的这个会话（task_server.py 的 attention_of_record、workspace_write_blocked_items；scripts/gate/hook-swap.py 退 7）：
    出了结局的任务命令是 scripts/gate/hook-swap.py（不带 --feed-only）、结局 command-failed 退 7：出一项 workspace-write-blocked，依据点名上一趟疑似是库里更早的哪条被杀的换钩子任务；
    带 --feed-only 的、退 0 的、命令是别的脚本的不出。
    弄坏开关 workspace-write-blocked-unreported 下一条都不出，这一格红。"""
    fake, root = long_run_split_scheduler(world)
    earlier = {"id": 905, "state": "finished", "outcome": "command-failed", "exit_code": 137, "command": ["python3", "scripts/gate/hook-swap.py", "--spec", "old.json"], "cwd": root}
    swap = ["python3", "scripts/gate/hook-swap.py", "--spec", "new.json"]
    problems = []

    def blocked(record):
        found = dict(record, state="finished", cwd=root)
        fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [], select=lambda table, where="1", parameters=(), order="id": [(earlier, True)])
        items = [item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time()) if item["kind"] == "workspace-write-blocked"]
        return items[0] if items else None

    item = blocked({"id": 910, "outcome": "command-failed", "exit_code": 7, "command": swap})
    if item is None or item["threshold"] != 1 or "任务 905" not in item["basis"] or "退 7" not in item["basis"]:
        problems.append(f"换钩子退 7：应当出一项 workspace-write-blocked（依据点名上一趟疑似是任务 905），实际 {item}")
    for what, record in (("带 --feed-only", {"id": 911, "outcome": "command-failed", "exit_code": 7, "command": swap + ["--feed-only"]}),
                         ("退 0", {"id": 912, "outcome": "succeeded", "exit_code": 0, "command": swap}),
                         ("命令是别的脚本", {"id": 913, "outcome": "command-failed", "exit_code": 7, "command": ["python3", "scripts/gate/other.py"]})):
        if blocked(record) is not None:
            problems.append(f"{what}：不该出")
    if problems:
        raise SelftestFailure("换钩子被没写完成的挡住：" + "；".join(problems))
    return "换钩子退 7 出一项 workspace-write-blocked、点名上一趟的任务；--feed-only、退 0、别的脚本不出"


def wrapped_swap_scheduler(world):
    """wrapped-hook-swap-reminded、wrapped-previous-swap-named、wrapped-writer-interrupted-judged 共用的假调度器（long_run_split_scheduler）：仓根里放两份串行脚本，
    swap-chain.sh 的正文在命令位置上写着换钩子（WRAPPED_WRITER_SHELL_BODY），feed-chain.sh 的只写着自证与只喂的（WRAPPED_WRITER_READ_ONLY_BODY），
    再放假 apply-writer-patch.py 与两份补丁目录（write_patch_writer_tree）、假 hook-swap.py（FAKE_HOOK_SWAP；不放它读的那一份，--read-outcome 退 2、换钩子被杀照报）。
    这几格只读正文、不起脚本（服务判被杀的写工具时起的是这两份假的 --read-outcome）。→ (假调度器, 仓根)。"""
    fake, root = long_run_split_scheduler(world)
    write_executable(os.path.join(root, "swap-chain.sh"), WRAPPED_WRITER_SHELL_BODY)
    write_executable(os.path.join(root, "feed-chain.sh"), WRAPPED_WRITER_READ_ONLY_BODY)
    write_patch_writer_tree(root)
    write_executable(os.path.join(root, "scripts", "gate", "hook-swap.py"), FAKE_HOOK_SWAP)
    return fake, root


def finished_attention_kinds(fake, root, record, earlier=()):
    """假调度器上这条出了结局的任务（state finished、工作目录是仓根）此刻要处置的提醒 {种类: 项}：库里更早的任务只有 earlier 那几条，尝试一次都没有。"""
    found = dict(record, state="finished", cwd=root)
    fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [],
                                       select=lambda table, where="1", parameters=(), order="id": [(other, True) for other in earlier])
    return {item["kind"]: item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time())}


def cell_wrapped_hook_swap_reminded(world):
    """包在里面的换钩子退 7、9、10 照命令那一层同样报（task_server_scheduler_lifecycle.py 的 hook_swap_command、wrapped_hook_swap；
    task_server_scheduler_building.py 的 wrapped_workspace_commands）：bash -c 串里与 bash 起的脚本正文里写着 hook-swap.py 的，结局 command-failed 退 7 出一项 workspace-write-blocked、
    退 9 与退 10 各出一项 previous-swap-running（依据写那个退出码）；包着的那一条带 --feed-only 的退 7、9、10 两种都不出。
    弄坏开关 wrapped-hook-swap-unread、shell-string-writers-unread、script-body-writers-unread、workspace-write-blocked-unreported、previous-swap-running-unreported、
    swap-scratch-busy-unreported 下包着的换钩子有该出而不出的，feed-only-counted-as-writer 下只喂的也出，这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    problems = []
    swaps = (("bash -c 串里的换钩子", ["bash", "-c", "set -e; python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch"]),
             ("bash 起的脚本正文里的换钩子", ["bash", "swap-chain.sh", "new.json"]))
    feeds = (("bash -c 串里只喂的换钩子", ["bash", "-c", "python3 scripts/gate/hook-swap.py --feed-only --spec new.json"]),
             ("bash 起的脚本正文里只喂的换钩子", ["bash", "feed-chain.sh", "new.json"]))
    task_id = 940
    for code, kind in ((7, "workspace-write-blocked"), (9, "previous-swap-running"), (10, "previous-swap-running")):
        for what, command in swaps:
            task_id += 1
            item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": code, "command": command}).get(kind)
            if item is None or f"退 {code}" not in item["basis"]:
                problems.append(f"{what}退 {code}：应当出一项 {kind}（依据写退 {code}），实际 {item}")
        for what, command in feeds:
            task_id += 1
            kinds = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": code, "command": command})
            if "workspace-write-blocked" in kinds or "previous-swap-running" in kinds:
                problems.append(f"{what}退 {code}：只喂不换，不该出换钩子的提醒，实际 {sorted(kinds)}")
    if problems:
        raise SelftestFailure("包在里面的换钩子被挡：" + "；".join(problems))
    return "bash -c 串里与脚本正文里的换钩子退 7 出 workspace-write-blocked、退 9 与退 10 出 previous-swap-running；包着的那一条只喂的都不出"


def cell_wrapped_previous_swap_named(world):
    """换钩子退 7 找上一趟疑似是哪条任务，包在里面起的被杀的换钩子照样点名（task_server_scheduler_lifecycle.py 的 workspace_write_blocked_items、writer_is_hook_swap）：
    这一条是命令那一层的 hook-swap.py 退 7；库里更早的只有一条被杀（退 137）的 bash -c 串里的换钩子时依据点名它，只有一条被杀的 bash 起的脚本正文里的换钩子时点名它，
    只有一条被杀的、bash -c 串里那一条带 --feed-only 的不点名它（照样出这一项）。
    弄坏开关 wrapped-previous-swap-unnamed、shell-string-writers-unread、script-body-writers-unread 下包着的换钩子点不出，feed-only-counted-as-writer 下只喂的也点名，
    workspace-write-blocked-unreported 下这一项不出，这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    swap = {"id": 960, "outcome": "command-failed", "exit_code": 7, "command": ["python3", "scripts/gate/hook-swap.py", "--spec", "new.json"]}
    problems = []
    for what, earlier_id, command, named in (
            ("bash -c 串里被杀的换钩子", 956, ["bash", "-c", "python3 scripts/gate/hook-swap.py --spec old.json"], True),
            ("bash 起的脚本正文里被杀的换钩子", 957, ["bash", "swap-chain.sh", "old.json"], True),
            ("bash -c 串里被杀的只喂的换钩子", 958, ["bash", "-c", "python3 scripts/gate/hook-swap.py --feed-only --spec old.json"], False)):
        earlier = {"id": earlier_id, "state": "finished", "outcome": "command-failed", "exit_code": 137, "command": command, "cwd": root}
        item = finished_attention_kinds(fake, root, swap, [earlier]).get("workspace-write-blocked")
        if item is None or (f"任务 {earlier_id}" in item["basis"]) != named:
            problems.append(f"上一趟是{what}：应当出一项 workspace-write-blocked、依据{'点名' if named else '不点名'}任务 {earlier_id}，实际 {item}")
    if problems:
        raise SelftestFailure("换钩子退 7 找上一趟：" + "；".join(problems))
    return "上一趟是 bash -c 串里、脚本正文里被杀的换钩子的都点名；只喂的不点名"


def cell_wrapped_writer_interrupted_judged(world):
    """包在里面的写工作区工具被信号杀（退 137），workspace-write-interrupted 的写工具与要核的路径取包着的那一条的词（task_server_scheduler_lifecycle.py 的 workspace_write_judged）：
    bash -c 串里打了一半的 apply-writer-patch.py 照结局文件判（依据写判没打完、出路点名 --read-outcome 与补丁目录 patch-half），打全了的不报；
    bash 起的脚本正文里的 hook-swap.py 点名正文里那一条的路径（$1、swap-scratch），bash -c 串里的 stage-mine.py 点名 selftest-case；写工具写包着的那一份，不写 bash。
    弄坏开关 wrapped-writer-words-unused、shell-string-writers-unread、script-body-writers-unread、workspace-write-interrupted-unreported 下写工具、路径不对或不报，
    apply-outcome-ignored 下打全了的也报，这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    problems = []
    for what, task_id, command, writer, wanted in (
            ("bash -c 串里打了一半的打补丁", 971, ["bash", "-c", "python3 scripts/gate/apply-writer-patch.py patch-half"],
             "scripts/gate/apply-writer-patch.py", ("--read-outcome", "补丁目录 patch-half")),
            ("bash 起的脚本正文里的换钩子", 972, ["bash", "swap-chain.sh", "new.json"], "scripts/gate/hook-swap.py", ("这几处：$1、swap-scratch；",)),
            ("bash -c 串里的暂存", 973, ["bash", "-c", "python3 scripts/gate/stage-mine.py --match selftest-case"], "scripts/gate/stage-mine.py",
             ("这几处：selftest-case；",))):
        item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": 137, "command": command}).get("workspace-write-interrupted")
        if (item is None or f"（{writer}）" not in item["why"] or any(text not in item["next"] for text in wanted)
                or (writer == SERVER.WRITER_PATCH_ENTRY and "判没打完" not in item["basis"])):
            problems.append(f"{what}被杀：应当出一项 workspace-write-interrupted、写工具写 {writer}、出路点名 {'、'.join(wanted)}，实际 {item}")
    full = finished_attention_kinds(fake, root, {"id": 974, "outcome": "command-failed", "exit_code": 137,
                                                 "command": ["bash", "-c", "python3 scripts/gate/apply-writer-patch.py patch-full"]})
    if "workspace-write-interrupted" in full:
        problems.append(f"bash -c 串里打全了的打补丁被杀：结局文件写着完成，不该报，实际 {full['workspace-write-interrupted']}")
    if problems:
        raise SelftestFailure("包在里面的写工作区工具被杀：" + "；".join(problems))
    return "包着的打补丁照结局文件判、换钩子与暂存点名包着的那一条的路径，写工具写包着的那一份；打全了的不报"


# wrapped-writers-each-judged 的串：先打补丁（patch-full 的结局文件写着完成）、再换钩子
PATCH_THEN_SWAP_CHAIN = ("python3 scripts/gate/apply-writer-patch.py patch-full && "
                         "python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch")


def cell_wrapped_writers_each_judged(world):
    """一个 bash -c 串里有几个写工作区工具，三处提醒逐条判（task_server_scheduler_building.py 的 workspace_writer_commands、wrapped_workspace_commands；
    task_server_scheduler_lifecycle.py 的 wrapped_hook_swap、writer_is_hook_swap、workspace_write_judged）：先打补丁（打全了）后换钩子的串，
    退 7 出 workspace-write-blocked、退 9 与退 10 出 previous-swap-running，被杀（退 137）出 workspace-write-interrupted、写工具只写 hook-swap.py（打全了的那一条不报）；
    命令那一层的换钩子退 7 时，库里更早被杀的这样一条串点名成上一趟；串里那一条打补丁带 --dry-run 的只免它自己，被杀照报换钩子那一条、不点名它的补丁目录。
    弄坏开关 wrapped-writers-first-only、shell-string-writers-unread、wrapped-hook-swap-unread、wrapped-previous-swap-unnamed、workspace-write-blocked-unreported、
    previous-swap-running-unreported、swap-scratch-busy-unreported、wrapped-writer-words-unused、workspace-write-interrupted-unreported、apply-outcome-ignored 下这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    chain = ["bash", "-c", PATCH_THEN_SWAP_CHAIN]
    problems = []
    for code, kind in ((7, "workspace-write-blocked"), (9, "previous-swap-running"), (10, "previous-swap-running")):
        item = finished_attention_kinds(fake, root, {"id": 1100 + code, "outcome": "command-failed", "exit_code": code, "command": chain}).get(kind)
        if item is None or f"退 {code}" not in item["basis"]:
            problems.append(f"先打补丁后换钩子的串退 {code}：应当出一项 {kind}（依据写退 {code}），实际 {item}")
    for what, task_id, command, absent in (
            ("先打补丁后换钩子的串", 1121, chain, "apply-writer-patch"),
            ("干跑的打补丁之后换钩子的串", 1122,
             ["bash", "-c", "python3 scripts/gate/apply-writer-patch.py patch-half --dry-run && python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch"],
             "patch-half")):
        item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": 137, "command": command}).get("workspace-write-interrupted")
        if (item is None or "（scripts/gate/hook-swap.py）" not in item["why"] or "这几处：new.json、swap-scratch" not in item["next"]
                or absent in item["why"] + item["next"]):
            problems.append(f"{what}被杀：应当出一项 workspace-write-interrupted、写工具只写 scripts/gate/hook-swap.py、点名 new.json 与 swap-scratch、不出现 {absent}，实际 {item}")
    earlier = {"id": 1130, "state": "finished", "outcome": "command-failed", "exit_code": 137, "command": chain, "cwd": root}
    swap = {"id": 1131, "outcome": "command-failed", "exit_code": 7, "command": ["python3", "scripts/gate/hook-swap.py", "--spec", "new.json"]}
    item = finished_attention_kinds(fake, root, swap, [earlier]).get("workspace-write-blocked")
    if item is None or "任务 1130" not in item["basis"]:
        problems.append(f"换钩子退 7、库里更早被杀的是先打补丁后换钩子的串：依据应当点名任务 1130，实际 {item}")
    if problems:
        raise SelftestFailure("一个串里几个写工作区工具：" + "；".join(problems))
    return "先打补丁后换钩子的串退 7、9、10 照报、被杀照报换钩子那一条、当上一趟照点名；干跑的打补丁只免它自己"


# wrapped-patch-directory-resolved 的串行脚本正文：补丁目录是位置参数（写着的工具前面先 exit 0，这一格只读正文、不起脚本）
PATCH_FROM_ARGUMENT_BODY = ("#!/usr/bin/env bash\n"
                            "set -e\n"
                            "exit 0\n"
                            "python3 scripts/gate/apply-writer-patch.py \"$1\"\n")


def cell_wrapped_patch_directory_resolved(world):
    """包着的打补丁被杀（退 137），补丁目录照这一条命令里认得出的 cd 与赋值算，算不出的照可能写了一半报（task_server_scheduler_building.py 的 shell_text_runs；
    task_server_scheduler_lifecycle.py 的 writer_interruption）：for 循环变量（$p）、脚本正文里的位置参数（$1）、cd 到位置参数（cd "$1" 之后的相对路径）的照报、依据写算不出
    （补丁目录照原样拼出来会落到打全了的 patch-full 或不在的路径上）；前面平铺着赋过值的 $p 代入之后照结局文件判（patch-half 报判没打完、patch-full 不报）；
    cd nested 之后的相对补丁目录照 nested 下那一份判（报判没打完、点名 nested 下的结局文件）。
    弄坏开关 wrapped-writer-paths-literal、shell-string-writers-unread、script-body-writers-unread、wrapped-writer-words-unused、workspace-write-interrupted-unreported、
    apply-outcome-ignored 下这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    write_executable(os.path.join(root, "apply-chain.sh"), PATCH_FROM_ARGUMENT_BODY)
    nested = os.path.join(root, "nested", "patch-nested-half")
    os.makedirs(nested)
    with open(os.path.join(nested, SERVER.PATCH_WRITER_OUTCOME_NAME), "w", encoding="utf-8") as handle:
        handle.write(HALF_APPLIED_OUTCOME)
    patch = "python3 scripts/gate/apply-writer-patch.py"
    problems = []
    for what, task_id, command, wanted in (
            ("for 循环变量当补丁目录", 1201, ["bash", "-c", f"for p in patch-full; do {patch} \"$p\"; done"], ("算不出",)),
            ("脚本正文里位置参数当补丁目录", 1202, ["bash", "apply-chain.sh", "patch-full"], ("算不出",)),
            ("cd 到位置参数之后的相对补丁目录", 1203, ["bash", "-c", f"cd \"$1\" && {patch} patch-full", "x", "nested"], ("算不出",)),
            ("赋过值的变量当补丁目录（没打完）", 1204, ["bash", "-c", f"p=patch-half; {patch} \"$p\""], ("判没打完",)),
            ("cd nested 之后的相对补丁目录（没打完）", 1205, ["bash", "-c", f"cd nested && {patch} patch-nested-half"],
             ("判没打完", os.path.join("nested", "patch-nested-half")))):
        item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": 137, "command": command}).get("workspace-write-interrupted")
        if item is None or any(text not in item["basis"] for text in wanted):
            problems.append(f"{what}被杀：应当出一项 workspace-write-interrupted、依据写 {'、'.join(wanted)}，实际 {item}")
    full = finished_attention_kinds(fake, root, {"id": 1206, "outcome": "command-failed", "exit_code": 137, "command": ["bash", "-c", f"p=patch-full; {patch} \"$p\""]})
    if "workspace-write-interrupted" in full:
        problems.append(f"赋过值的变量当补丁目录（打全了）被杀：结局文件写着完成，不该报，实际 {full['workspace-write-interrupted']}")
    if problems:
        raise SelftestFailure("包着的打补丁的补丁目录：" + "；".join(problems))
    return "循环变量、位置参数、cd 到算不出的目录照报算不出；赋过值的变量与 cd 之后的相对路径照算出的结局文件判"


def cell_wrapped_patch_directory_unresolvable(world):
    """包着的打补丁被杀（退 137），bash 真跑时的当前目录不是照平铺的 cd 推得出的三形，补丁目录照算不出报（task_server_scheduler_building.py 的 shell_text_pieces、shell_text_runs；
    task_server_scheduler_lifecycle.py 的 writer_interruption）：花括号组接在管道上、组里 cd nested（整组跑在子 shell 里，cd 留不到组外），heredoc 正文里一行 cd nested
    （那一行是喂给 cat 的文字），串里那一条自己带 env -C nested（只换它起的那个进程的目录）。三形都应当出一项 workspace-write-interrupted、依据写算不出，
    不照推错的目录读结局文件（推错的目录下没有那份结局文件，读了就当还没开始换、不报）。
    弄坏开关 brace-group-not-layered、heredoc-body-as-commands、string-env-chdir-unread 各让一形照推错的目录读，wrapped-writer-paths-literal、shell-string-writers-unread、
    wrapped-writer-words-unused、workspace-write-interrupted-unreported 下这一格同样红。"""
    fake, root = wrapped_swap_scheduler(world)
    nested = os.path.join(root, "nested", "patch-nested-half")
    os.makedirs(nested)
    with open(os.path.join(nested, SERVER.PATCH_WRITER_OUTCOME_NAME), "w", encoding="utf-8") as handle:
        handle.write(HALF_APPLIED_OUTCOME)
    patch = "python3 scripts/gate/apply-writer-patch.py"
    problems = []
    for what, task_id, command in (
            ("花括号组接在管道上、组里 cd nested 之后的相对补丁目录", 1401, ["bash", "-c", f"{{ cd nested; true; }} | cat\n{patch} patch-half"]),
            ("heredoc 正文里一行 cd nested 之后的相对补丁目录", 1402, ["bash", "-c", f"cat > notes.txt <<'EOF'\ncd nested\nEOF\n{patch} patch-half"]),
            ("串里 env -C nested 起的打补丁", 1403, ["bash", "-c", f"env -C nested {patch} patch-nested-half"])):
        item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": 137, "command": command}).get("workspace-write-interrupted")
        if item is None or "算不出" not in item["basis"]:
            problems.append(f"{what}被杀：应当出一项 workspace-write-interrupted、依据写补丁目录算不出，实际 {item}")
    if problems:
        raise SelftestFailure("包着的打补丁的当前目录推不出：" + "；".join(problems))
    return "花括号组接管道里的 cd、heredoc 正文里的 cd、串里的 env -C 都照补丁目录算不出报"


# hook-swap-outcome-judged 的串：两条换钩子先后跑，各给一个草稿目录
TWO_SWAPS_CHAIN = ("python3 scripts/gate/hook-swap.py --spec a.json --scratch s1 && "
                   "python3 scripts/gate/hook-swap.py --spec b.json --scratch s2")


def cell_hook_swap_outcome_judged(world):
    """被杀（退 137）的换钩子照它的 --read-outcome 判（task_server_scheduler_lifecycle.py 的 hook_swap_read_back、workspace_write_judged）：假仓根里的假 hook-swap.py 照
    FAKE_HOOK_SWAP_OUTCOME_NAME 打出结局。两条换钩子的串（TWO_SWAPS_CHAIN）：没换完、反向规格在 s2 下的只点名 b.json 与 s2（a.json 那一条已正常换完），在 s1 下的只点名 a.json 与 s1
    （b.json 那一条还没起），在别的草稿目录下的两条都点名；换完了的一条都不点名、不出这一项；命令那一层的一条换钩子换完了同样不出。
    弄坏开关 swap-outcome-unread、swap-outcome-owner-ignored、wrapped-writers-first-only、shell-string-writers-unread、wrapped-writer-words-unused、
    workspace-write-interrupted-unreported 下这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    chain = ["bash", "-c", TWO_SWAPS_CHAIN]
    unfinished_head = "3\n✗ 没换完：结局文件有开始行，没有完成也没有退回\n→ 怎么办：二选一；反向规格 "
    problems = []

    def judged(task_id, command, outcome_text):
        with open(os.path.join(root, FAKE_HOOK_SWAP_OUTCOME_NAME), "w", encoding="utf-8") as handle:
            handle.write(outcome_text)
        return finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": 137, "command": command}).get("workspace-write-interrupted")

    first, second = "这几处：a.json、s1；", "这几处：b.json、s2；"   # 出路里点名每一条换钩子的那一段（要核的是<这一段>）
    for what, task_id, owner, named, unnamed in (
            ("结局文件是后一条（s2）的", 1411, "s2", (second,), (first,)),
            ("结局文件是前一条（s1）的", 1412, "s1", (first,), (second,)),
            ("结局文件是别的草稿目录的", 1413, "elsewhere", (first, second), ())):
        item = judged(task_id, chain, unfinished_head + os.path.join(os.path.abspath(root), owner, "hook-swap", "reverse-1.json") + "\n")
        checked = item["next"] if item is not None else ""
        if item is None or any(text not in checked for text in named) or any(text in checked for text in unnamed) or "没换完" not in item["basis"]:
            problems.append(f"两条换钩子的串被杀、{what}：应当出一项 workspace-write-interrupted、依据写没换完、出路点名「{'」「'.join(named)}」、"
                            f"不点名「{'」「'.join(unnamed) or '（无）'}」，实际 {item}")
    for what, task_id, command in (("两条换钩子的串", 1414, chain),
                                   ("命令那一层的换钩子", 1415, ["python3", "scripts/gate/hook-swap.py", "--spec", "c.json", "--scratch", "s3"])):
        item = judged(task_id, command, "0\n✓ 换完了\n")
        if item is not None:
            problems.append(f"{what}被杀而结局文件写着换完了：不该出 workspace-write-interrupted，实际 {item}")
    if problems:
        raise SelftestFailure("被杀的换钩子照结局判：" + "；".join(problems))
    return "两条换钩子的串只点名结局文件归它的那一条，归别处的两条都点名；换完了的不报"


# wrapped-read-only-hook-swap-reminded 的串行脚本正文：命令位置上的换钩子带 --dry-run（写着的工具前面先 exit 0，这一格只读正文、不起脚本）
DRY_RUN_SWAP_BODY = ("#!/usr/bin/env bash\n"
                     "exit 0\n"
                     "python3 scripts/gate/hook-swap.py --spec \"$1\" --scratch swap-scratch --dry-run\n")


def cell_wrapped_read_only_hook_swap_reminded(world):
    """包着的换钩子带 --dry-run，退 7、9、10 照命令那一层同样报（task_server_scheduler_lifecycle.py 的 wrapped_hook_swap）：bash -c 串里与 bash 起的脚本正文里的
    hook-swap.py --dry-run 退 7 出 workspace-write-blocked、退 9 与退 10 出 previous-swap-running；它只读不写，被杀（退 137）不出 workspace-write-interrupted。
    弄坏开关 wrapped-read-only-hook-swap-unread、wrapped-hook-swap-unread、shell-string-writers-unread、script-body-writers-unread、workspace-write-blocked-unreported、
    previous-swap-running-unreported、swap-scratch-busy-unreported、read-only-arguments-counted-as-writer 下这一格红。"""
    fake, root = wrapped_swap_scheduler(world)
    write_executable(os.path.join(root, "dry-run-chain.sh"), DRY_RUN_SWAP_BODY)
    problems = []
    task_id = 1300
    for code, kind in ((7, "workspace-write-blocked"), (9, "previous-swap-running"), (10, "previous-swap-running")):
        for what, command in (("bash -c 串里干跑的换钩子", ["bash", "-c", "python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch --dry-run"]),
                              ("bash 起的脚本正文里干跑的换钩子", ["bash", "dry-run-chain.sh", "new.json"])):
            task_id += 1
            item = finished_attention_kinds(fake, root, {"id": task_id, "outcome": "command-failed", "exit_code": code, "command": command}).get(kind)
            if item is None or f"退 {code}" not in item["basis"]:
                problems.append(f"{what}退 {code}：应当出一项 {kind}（依据写退 {code}），实际 {item}")
    killed = finished_attention_kinds(fake, root, {"id": 1320, "outcome": "command-failed", "exit_code": 137,
                                                   "command": ["bash", "-c", "python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch --dry-run"]})
    if "workspace-write-interrupted" in killed:
        problems.append(f"bash -c 串里干跑的换钩子被杀：只读不写，不该报 workspace-write-interrupted，实际 {killed['workspace-write-interrupted']}")
    if problems:
        raise SelftestFailure("包着的干跑换钩子：" + "；".join(problems))
    return "bash -c 串里与脚本正文里干跑的换钩子退 7、9、10 照报；被杀不报"


ESTIMATE_COMMAND_KEY = "selftest-last-run-seconds"   # last-run-seconds-from-records：库里那几次尝试同一把命令键


def cell_last_run_seconds_from_records(world):
    """这把命令键上次真跑完用了多久只照库里的记录取（task_server.py 的 last_run_seconds、completed_runs；只给插队规则用）：库里同键的几次尝试（命令自己跑到退出的，被信号杀的不算）：
    退 0 的取最近一次的用时、不取最长一次；退非 0 的取近 RECORD_WINDOW 次里最长的一次，比退 0 的那个数长才取它（交 (用时, 是取自退非 0 的)）；
    最近一次是比退 0 的短的失败时仍取退 0 的；没有退 0 的记录时退非 0 的跑满 FAILED_RUN_RECORD_MINIMUM_SECONDS 才取，没跑满的交 None；退非 0 的在近 RECORD_WINDOW 次之外不算；
    同一把命令键、命令不逐词相同的也认。假库：select 交预备好的几行（最近的在前不必排，completed_runs 自己按结束时刻排）。
    弄坏开关 estimate-longest-succeeded 下退 0 的取最长一次，estimate-only-succeeded 下退非 0 的都不算，estimate-latest-any-outcome 下不分退码取最近一次，
    estimate-short-failure-counted 下没跑满的失败也取，estimate-failure-outside-window 下退非 0 的不限近几次，estimate-by-exact-command 下只认命令逐词相同的，这一格红。"""
    command = ["bash", "-c", "exit 0", "estimate-sample"]

    def run_row(seconds, ended, clean=True, words=command):
        return ({"command_key": ESTIMATE_COMMAND_KEY, "command": list(words), "outcome": "succeeded" if clean else "command-failed", "exit_code": 0 if clean else 1,
                 "seconds": float(seconds), "ended_at": float(ended)}, True)

    window = SERVER.RECORD_WINDOW
    cases = (("退 0 的取最近一次、不取最长一次", [run_row(100, 2000), run_row(300, 1000)], (100.0, False)),
             ("退非 0 的比退 0 的长就取它", [run_row(100, 2000), run_row(700, 1000, clean=False)], (700.0, True)),
             ("最近一次是比退 0 的短的失败，仍取退 0 的", [run_row(50, 2000, clean=False), run_row(100, 1000)], (100.0, False)),
             ("只有没跑满的失败，交 None", [run_row(30, 1000, clean=False)], (None, False)),
             ("退非 0 的在近几次之外不算", [run_row(100, 3000 - 100 * position) for position in range(window)] + [run_row(900, 1000, clean=False)], (100.0, False)),
             ("同一把命令键、命令不逐词相同的也认", [run_row(100, 1000, words=["bash", "-c", "exit 0", "estimate-other"])], (100.0, False)))
    problems = []
    for what, rows, wanted in cases:
        fake = object.__new__(SERVER.Scheduler)
        fake.environment = dict(os.environ)
        fake.store = types.SimpleNamespace(select=lambda table, where="1", parameters=(), order="id", rows=rows: list(rows))
        found = SERVER.Scheduler.last_run_seconds(fake, {"command_key": ESTIMATE_COMMAND_KEY, "command": command})
        if found != wanted:
            problems.append(f"{what}：应当交 {wanted}，实际 {found}")
    if problems:
        raise SelftestFailure("上次运行时长只照记录取：" + "；".join(problems))
    return "退 0 的取最近一次、退非 0 的比它长才取、没跑满的失败与近几次之外的不算、命令不逐词相同的同键也认"


ACTIVITY_GROWTH_PER_TICK_BYTES = 64 * MEBIBYTE   # memory-growth-not-stalled：假 scope 的进程内存每 ACTIVITY_GROWTH_TICK_SECONDS 秒涨这么多（涨一回就够过 ACTIVITY_MEMORY_BYTES，机器忙时窗口里只写成一两回也算有进展）
ACTIVITY_GROWTH_TICK_SECONDS = 0.5               # memory-growth-not-stalled：涨内存的间隔（一个窗口里十回；总量跟着放行时刻走，不会涨到默认上限 4096M 的 0.9）
ACTIVITY_STALL_SECONDS = 5                       # memory-growth-not-stalled：卡住阈值（TASK_SERVER_SELFTEST_STALL_SECONDS），也是服务判一个窗口的长度：服务一轮排队里读一回进程内存，
                                                 # 窗口里要有两回读数才量得出涨没涨，机器忙时一轮排队长到两三秒也要留得下两回
ACTIVITY_HELD_WINDOWS = 3                        # memory-growth-not-stalled：涨内存的那一条，服务记的起跑时刻起跑满这么多个窗口之后才放行
ACTIVITY_QUIET_COMMAND = "echo started; " + HOLD_UNTIL_RELEASED_SCRIPT   # 那一格两条的正文：只打一行、不出声地等放行文件（$1）；不涨内存的那一条由服务停


def grow_process_memory(stat_path, stop):
    """memory-growth-not-stalled 的假 scope：stop 没置位之前，每 ACTIVITY_GROWTH_TICK_SECONDS 秒把 stat_path（memory.stat）里的进程内存加 ACTIVITY_GROWTH_PER_TICK_BYTES，先写临时文件再改名。"""
    used = 64 * MEBIBYTE
    while not stop.is_set():
        used += ACTIVITY_GROWTH_PER_TICK_BYTES
        with open(stat_path + ".partial", "w", encoding="utf-8") as handle:
            handle.write(process_memory_stat(used))
        os.replace(stat_path + ".partial", stat_path)
        time.sleep(ACTIVITY_GROWTH_TICK_SECONDS)


def cell_memory_growth_not_stalled(world):
    """进程内存在涨算有进展（task_server.py 的 note_activity、activity_text、judge_stall_stop）：
    卡住窗口 ACTIVITY_STALL_SECONDS 秒、宽限 0.01 秒，假 scope 的 CPU 不涨。涨内存的那一条只打一行、不出声地等放行文件，这段时间假 scope 的进程内存
    每 ACTIVITY_GROWTH_TICK_SECONDS 秒涨 64 MiB（涨一回就够过一个窗口的门槛）：日志与 CPU 都不涨，照样不停；按服务记的起跑时刻跑满 ACTIVITY_HELD_WINDOWS 个窗口之后放行、跑完；
    之后进程内存不动，不涨内存的那一条同样不出声地等放行文件：由服务停、记 stalled（不靠命令自己跑完，机器忙时服务判窗口晚了也等得到）。
    另静态核 activity_text：进程内存一个窗口里从 100 MiB 涨到 200 MiB 算有进展，只涨 10 MiB 不算，窗口不是这一个的不算。
    弄坏开关 progress-ignored-for-stall 下涨内存的那一条也被停、记 stalled，这一格红。"""
    key = "selftest-memory-growth"
    scope = os.path.join(world.root, "growing-memory-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    stat = os.path.join(scope, "memory.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(64 * MEBIBYTE))
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": str(ACTIVITY_STALL_SECONDS), "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    stop = threading.Event()
    growing_release = os.path.join(world.root, "release-memory-growing")
    flat_release = os.path.join(world.root, "release-memory-flat")
    grower = threading.Thread(target=grow_process_memory, args=(stat, stop), daemon=True)
    grower.start()
    try:
        growing_id = world.submit(["bash", "-c", ACTIVITY_QUIET_COMMAND, "memory-growing", growing_release], extra={lib.KEY_VARIABLE: key})
        world.wait_for(growing_id, lambda summary: summary["task"].get("state") == "finished" or attempt_has_run_for(summary, ACTIVITY_HELD_WINDOWS * ACTIVITY_STALL_SECONDS),
                       f"跑满 {ACTIVITY_HELD_WINDOWS} 个卡住窗口（或被停）")
        open(growing_release, "w").close()   # 先放行、等它收尾，再停涨：放行到命令退出之间内存还在涨，这一下不被当成窗口里不涨
        growing = world.finished(growing_id)
    finally:
        open(growing_release, "w").close()
        stop.set()
        grower.join()
    try:
        flat = world.finished(world.submit(["bash", "-c", ACTIVITY_QUIET_COMMAND, "memory-flat", flat_release], extra={lib.KEY_VARIABLE: key}), timeout=45)
    finally:
        open(flat_release, "w").close()
    problems = []
    if growing["task"].get("outcome") != "succeeded":
        problems.append(f"进程内存每 {ACTIVITY_GROWTH_TICK_SECONDS} 秒涨 {ACTIVITY_GROWTH_PER_TICK_BYTES // MEBIBYTE} MiB 的那一条（日志与 CPU 都不涨）应当照跑完，实际 {describe(growing)}")
    if flat["task"].get("outcome") != "stalled":
        problems.append(f"进程内存不动的那一条应当记 stalled，实际 {describe(flat)}")
    moving, still = (SERVER.ActivityWindow(1.0, first, None, last, None) for first, last in ((100 * MEBIBYTE, 200 * MEBIBYTE), (100 * MEBIBYTE, 110 * MEBIBYTE)))
    if not SERVER.activity_text(moving, 1.0) or SERVER.activity_text(still, 1.0) or SERVER.activity_text(moving, 2.0):
        problems.append(f"进程内存从 100 MiB 涨到 200 MiB 应当算有进展、只涨 10 MiB 与别的窗口的不算，实际「{SERVER.activity_text(moving, 1.0)}」"
                        f"「{SERVER.activity_text(still, 1.0)}」「{SERVER.activity_text(moving, 2.0)}」")
    if problems:
        raise SelftestFailure("有进展的不判卡住：" + "；".join(problems), world.server_log_lines())
    return "进程内存在涨的那一条照跑完、不动的记 stalled；静态核进程内存涨够了才算有进展"


WINDOW_HEAD_ROUND_SECONDS = 20.0   # memory-window-head-read-at-open：假的一轮排队隔多久（比卡住窗口 WINDOW_HEAD_STALL_SECONDS 长得多：机器忙时一轮排队长过窗口）
WINDOW_HEAD_STALL_SECONDS = 5.0    # memory-window-head-read-at-open：假调度器的卡住窗口
WINDOW_HEAD_BASE_BYTES = 1024 * MEBIBYTE   # memory-window-head-read-at-open：头一轮读到的进程内存
WINDOW_HEAD_GROWTH_BYTES = 200 * MEBIBYTE  # memory-window-head-read-at-open：涨后那一轮读到的比头一轮多这么多（过 ACTIVITY_MEMORY_BYTES）


class WindowReadingMachine:
    """memory-window-head-read-at-open 的假本机：memory_now 交各轮之前设的进程内存（字节）、没有峰值；D 状态的进程数交 0。"""

    def __init__(self):
        self.memory = 0

    def memory_now(self, attempt):
        return self.memory, None

    def io_blocked(self, attempt):
        return 0


def cell_memory_window_head_read_at_open(world):
    """判卡住开新窗口的那一轮就读头一份进程内存（task_server.py 的 note_window_head、judge_stall_stop、activity_text；文件头 ①「卡住」的有进展）：假调度器每一轮隔
    WINDOW_HEAD_ROUND_SECONDS 秒（比卡住窗口 WINDOW_HEAD_STALL_SECONDS 秒长得多），日志大小不变、CPU 不涨，进程内存头一轮 1024 MiB、涨后那一轮 1224 MiB：
    涨后那一轮的窗口里有头尾两份读数、涨了过门槛，有进展，不停（免停缘由 SPARED_ACTIVITY），从那一轮再看一个窗口；再下一轮进程内存不动，这个窗口里涨得不够，停、记 stalled。
    弄坏开关 stall-window-head-unread 下头一份读数在涨后那一轮才记，窗口里只有一份、涨了也当没进展，涨后那一轮就停，这一格红。"""
    machine = WindowReadingMachine()
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.stall_seconds = WINDOW_HEAD_STALL_SECONDS
    fake.log_growth, fake.io_waits, fake.activity_windows = {}, {}, {}
    fake.machines = {SERVER.LOCAL: machine}
    fake.log_changed_at = lambda attempt, now: now
    fake.store = types.SimpleNamespace(update=lambda table, record: None)
    fake.stop_running = lambda *arguments, **keywords: None
    task = {"id": 9}
    attempt = {"id": 91, "number": 1, "host": SERVER.LOCAL, "memory_cap": 8 * GIBIBYTE, "started_at": time.time() - 1000.0}
    cpu, size, start = 1_000_000, 4096, 100.0
    rounds = []
    for position, memory in enumerate((WINDOW_HEAD_BASE_BYTES, WINDOW_HEAD_BASE_BYTES + WINDOW_HEAD_GROWTH_BYTES, WINDOW_HEAD_BASE_BYTES + WINDOW_HEAD_GROWTH_BYTES)):
        machine.memory = memory
        rounds.append(SERVER.Scheduler.judge_stall_stop(fake, task, attempt, size, cpu, start + position * WINDOW_HEAD_ROUND_SECONDS))
    spared = fake.log_growth[attempt["id"]][4] if attempt["id"] in fake.log_growth else None
    problems = []
    if rounds != [False, False, True]:
        problems.append(f"三轮的结果应当是 [不停, 不停（窗口里涨了 {WINDOW_HEAD_GROWTH_BYTES // MEBIBYTE} MiB）, 停（涨完之后不动）]，实际 {rounds}")
    elif not attempt.get("stalled"):
        problems.append(f"第三轮停了应当记 stalled，实际尝试 {attempt}")
    if rounds[:2] == [False, False] and spared != SERVER.SPARED_ACTIVITY:
        problems.append(f"第二轮不停的缘由应当记 SPARED_ACTIVITY（{SERVER.SPARED_ACTIVITY}），实际 {spared}")
    if problems:
        raise SelftestFailure("窗口里的头一份进程内存在开窗那一轮读：" + "；".join(problems))
    return f"每轮隔 {WINDOW_HEAD_ROUND_SECONDS:.0f} 秒、窗口 {WINDOW_HEAD_STALL_SECONDS:.0f} 秒：第二轮进程内存涨 {WINDOW_HEAD_GROWTH_BYTES // MEBIBYTE} MiB 不停，第三轮不动停、记 stalled"


def cell_stopped_peaks_floor_record(world):
    """被停、判红的那一次量到的峰值当下限进定额度（task_server.py 的 stopped_peak_floor、recorded_prediction；admission 审核 F6）：撞过 13824M（峰值 13.48G）、之后被取消的那一次
    量到 15.21G：下一次起步不低于 15.21G（不再从撞过的那一档重撞）；跑成 2G、命令退 1 的那一次 100M：照 2G（小的不拉低）；只有一次取消、量到 6G：照 6G 起（比没记录的 4G 大）；
    只有一次取消、量到 1G：交 None，照没记录的 4G 起。弄坏开关 stopped-peaks-not-counted 下被停的峰值丢掉（头一种起步 13824M、第三种照 4G），这一格红。"""
    dispatch = loaded_host_dispatch()
    default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
    base = dispatch.Prediction("selftest-stopped-peaks", default, default, default, lib.UNRECORDED_HISTORY)

    def row(outcome, peak, ended_at, cap=None):
        return ({"outcome": outcome, "memory_peak": int(peak), "memory_cap": cap, "ended_at": ended_at, "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True)
    problems = []
    for what, rows, wanted in (("撞过 13824M、之后取消时量到 15.21G", [row("cancelled", 15.21 * GIBIBYTE, 5), row("memory-cap-hit", 13.48 * GIBIBYTE, 4, 13824 * MEBIBYTE)],
                                int(13824 * MEBIBYTE * lib.CAP_GROWTH_FACTOR)),   # 撞过、之后没跑完过：照撞过的上限乘 1.5（20736M），比取消时量到的 15.21G 大
                               ("跑成 2G、命令退 1 时 100M", [row("command-failed", 100 * MEBIBYTE, 5), row("succeeded", 2 * GIBIBYTE, 4)], 2 * GIBIBYTE),
                               ("只有一次取消、量到 6G", [row("cancelled", 6 * GIBIBYTE, 5)], 6 * GIBIBYTE),
                               ("只有一次取消、量到 1G", [row("cancelled", GIBIBYTE, 5)], None)):
        fake = object.__new__(SERVER.Scheduler)
        fake.environment = dict(os.environ)
        fake.store = types.SimpleNamespace(select=lambda *arguments, table=rows, **options: list(table))
        found = SERVER.Scheduler.recorded_prediction(fake, {"command_key": "selftest-stopped-peaks"}, base)
        predicted = None if found is None else int(found.memory_recorded_peak)
        if predicted != wanted:
            problems.append(f"{what}：起步应当 {wanted}，实际 {predicted}")
    if problems:
        raise SelftestFailure("被停的峰值当下限：" + "；".join(problems))
    return "取消那一次的 15.21G 抬过撞过的 13824M；小的不拉低；只有被停的、比缺省大的照它起"


STOPPED_UNIT_ATTEMPT = 3   # unit-stopped-booking-released：单元已停、收尾还没做完的那一条在跑尝试的号


def cell_unit_stopped_booking_released(world):
    """单元确认停了就放掉这一次尝试的 CPU 与内存记账（task_server.py 的 release_stopped_booking、unit_confirmed_stopped、booking_attempts；任务 13716 到 13718 取消后单元已 inactive、
    两台都没有进程，结局隔了十几分钟才落库，期间 CPU 闸照记各 31 线程）：本机三条在跑（起了 100 秒、命令键记过 31 核，照记录各记 31 线程，共 93 线程），第三条的单元读回 inactive 而且 cgroup.procs 空：
    放掉它之后 CPU 闸记 62 线程；内存一侧（slice 30G、余量 1G，一条在跑、上限 20G、scope 此刻用满 20G）记账 20G、公式剩 9G、待涨量 0，
    放掉之后公式剩 29G、待涨量 0。读不到状态（unit_states 交 None）、cgroup.procs 读不到、cgroup.procs 里还有进程、单元还在跑的，都不算停：照数。收尾记 ended 之后（finish_attempt 清）这一条不再留。
    弄坏开关 unit-stopped-still-booked 下停了的也照数（93 线程、公式剩 9G），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    attempt = {"id": STOPPED_UNIT_ATTEMPT, "label": RUNNING_BOOKING_LABEL, "unit": "stopped-unit-selftest.service", "host": SERVER.LOCAL}
    problems = []
    readings = (("单元 inactive、cgroup.procs 空", ("loaded", "inactive"), [], True), ("读不到单元状态", None, [], False),
                ("cgroup.procs 读不到", ("loaded", "inactive"), None, False), ("cgroup.procs 里还有进程", ("loaded", "inactive"), [4242], False),
                ("单元还在跑", ("loaded", "active"), [], False))
    for what, states, processes, stopped in readings:
        fake, _ = cpu_gate_scheduler(3, load, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * 3)
        with patched_server_name("unit_states", lambda unit, environment, states=states: states), \
                patched_server_name("unit_processes", lambda unit, environment, processes=processes: processes):
            confirmed = SERVER.Scheduler.release_stopped_booking(fake, attempt)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        wanted_threads = 62 if stopped else 93
        if confirmed != stopped or counted != wanted_threads:
            problems.append(f"{what}：应当{'确认停了、记 62 线程' if stopped else '不算停、照记 93 线程'}，实际确认 {confirmed}、记 {counted} 线程")
        if stopped:
            fake.store.running_attempts = lambda: []   # 收尾记了 ended：不在在跑名单里，布告栏里这一条清掉
            SERVER.Scheduler.booking_attempts(fake)
            if fake.unit_stopped:
                problems.append(f"{what}：尝试不在在跑名单里之后应当从放掉的名单里清掉，实际还留着 {sorted(fake.unit_stopped)}")
    memory_attempt = {"id": 1, "label": RUNNING_BOOKING_LABEL, "unit": "stopped-unit-selftest.service", "host": SERVER.LOCAL}
    for released, wanted_room in ((False, 9 * GIBIBYTE), (True, 29 * GIBIBYTE)):
        fake, snapshot, report = running_booking_scheduler(20 * GIBIBYTE, 20 * GIBIBYTE)
        if released:
            with patched_server_name("unit_states", lambda unit, environment: ("loaded", "inactive")), \
                    patched_server_name("unit_processes", lambda unit, environment: []):
                SERVER.Scheduler.release_stopped_booking(fake, memory_attempt)
        room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
        pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
        if room != wanted_room or pending != 0:
            problems.append(f"内存{'放掉之后' if released else '没放掉时'}：公式应当剩 {wanted_room / GIBIBYTE:g}G、待涨量 0，"
                            f"实际剩 {room / GIBIBYTE:g}G、待涨量 {pending / GIBIBYTE:g}G")
    if problems:
        raise SelftestFailure("单元已停先放记账：" + "；".join(problems))
    return "单元回读停了：CPU 闸 93 线程变 62、内存公式剩 9G 变 29G；读不到的、还有进程的、还在跑的照数"


def cell_stall_reads_log_before_cpu(world):
    """判卡住每一轮开头先把日志读到底再读 CPU（task_server.py 的 fresh_cpu_usec）：第二台的日志里刚到一行 MULTI_HOST_RUN_PEER_CPU usage_usec=5000000、还没被读过：
    读到的 CPU 是 5000000。套在假调度器上，不起服务。弄坏开关 stall-cpu-read-before-log 下不先读日志、交读不到，这一格红。"""
    fake, attempt = peer_log_scheduler(world, "cpu-first", ["MULTI_HOST_RUN_PEER_CPU poll=1 usage_usec=5000000 read=yes"])
    usage = SERVER.Scheduler.fresh_cpu_usec(fake, attempt)
    if usage != 5000000:
        raise SelftestFailure(f"第二台日志里刚到的那一行 CPU 应当读到 5000000，实际 {usage}")
    return "第二台刚到的 CPU 行当轮读到"


def cell_retry_build_directory_reused(world):
    """撞上限重排时编译目录接着用（task_server.py 的 take_build_slot 先拿这条任务上一次在同一台用的那一个）：本机先退 250 再跑的会编命令，
    重试那一次的编译目录是上一次那一个（尝试记 build_directory_reused，假 cargo 拿到的 CARGO_TARGET_DIR 在它下面）；头一次在本机、撞了上限之后本机腾不出地方、
    重排换到第二台的，本机那一个照旧留在组里（换台不删，task_server_lib.py 文件头「编译目录」）。
    弄坏开关 build-slot-per-attempt 下重试那一次新开一个，这一格红。"""
    world.resources(local=(30, 30), peer=(10, 10))   # 两台都放得下 4G 与 6G：照剩余占比都挑本机
    crowded = os.path.join(world.root, "resources-local-crowded.json")
    world.resources(local=(5, 30), peer=(100, 100), path=crowded)   # 本机放不下 6G、第二台放得下
    world.start_server()
    extra = fake_cargo_extra(world)
    problems = []
    same = world.finished(world.submit(["cargo", "build", "fake-wrapper-exits=250,run"], extra=extra))
    attempts = same["attempts"]
    directories = [attempt.get("build_directory") or "" for attempt in attempts]
    targets = cargo_targets(world)
    if (same["task"].get("outcome") != "succeeded" or len(attempts) != 2 or not all(directories) or directories[0] != directories[1]
            or not attempts[1].get("build_directory_reused") or targets != [os.path.join(directories[0], "target")]):
        problems.append(f"本机先退 250 再跑：重试那一次的编译目录应当是上一次那一个、记 build_directory_reused、假 cargo 拿到的 CARGO_TARGET_DIR 是它下面的 target，"
                        f"实际 {describe(same)}、编译目录 {directories}、第 2 次 build_directory_reused={attempts[-1].get('build_directory_reused') if attempts else None}、"
                        f"假 cargo 拿到 {targets}")
    moved = world.finished(world.submit(["cargo", "build", "--release", "fake-wrapper-exits=250,run", f"fake-wrapper-swap-resources={crowded}"], extra=extra),
                           timeout=PLACEMENT_WAIT_SECONDS)
    hosts = [attempt.get("host") for attempt in moved["attempts"]]
    local_directory = (moved["attempts"] or [{}])[0].get("build_directory") or ""
    if moved["task"].get("outcome") != "succeeded" or hosts != ["local", "peer"] or not local_directory or not os.path.isdir(local_directory):
        problems.append(f"头一次在本机撞了上限、重排换到第二台：本机那一个编译目录应当照旧留在组里（换台不删），"
                        f"实际 {describe(moved)}、各次在 {hosts}、本机那一个 {local_directory or '（没记）'}{'不在了' if local_directory and not os.path.isdir(local_directory) else ''}")
    if problems:
        raise SelftestFailure("重试的编译目录：" + "；".join(problems), world.server_log_lines())
    return "本机重排的那一次接着用上一次的编译目录；换到第二台的，本机那一个照旧留在组里"
