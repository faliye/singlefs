"""task_server_selftest.py 拆出的一份：跑满上限、长任务提醒、撞上限的提醒、循环泄漏、外部杀的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import re
import shlex
import time
import types
import task_server_lib as lib
from task_server_selftest_fakes import BLOCK_SCRIPT, GIBIBYTE, MEBIBYTE, POLL_SECONDS, SERVER, SESSION_A, SESSION_B, UNRECORDED_DEFAULT_PINNED_TEXT
from task_server_selftest_world import (
    ATTENTION_WAIT_SECONDS, FAKE_COMMIT_GATE, FAKE_MERGE_ENTRY, FILLS_CAP_SEEDED_CAP_BYTES, FILLS_CAP_SEEDED_PEAK_BYTES, FakeInbox,
    INBOX_QUIET_SECONDS, INBOX_WAIT_SECONDS, LONG_RUN_KIND, LONG_RUN_WHY_SAMPLE, LONG_TICKS_SCRIPT, REPUSH_SAMPLE_SECONDS, RUN_LIMIT_SAMPLE_SECONDS,
    SCOPE_COMMIT, SCOPE_DAILY, SelftestFailure, attention_lines, attention_problems, blocking_command, child_roles, describe, first_attempt,
    inbox_problems, long_run_split_scheduler, outcome_head, process_memory_stat, raise_cell_problems, read_text, reminder_head, shard_children,
    shard_outcomes, start_inbox_server, start_three_shards, table_advice, wait_last_line, write_executable, write_fake_entry,
)


LONG_RUN_SELFTEST_SECONDS = 20   # long-run-failure-noted：自证里「跑得久」的界（TASK_SERVER_SELFTEST_LONG_RUN_SECONDS）；立刻退的那一条在机器忙时起跑、收尾也远不到它
LONG_RUN_SELFTEST_SLEEP_SECONDS = LONG_RUN_SELFTEST_SECONDS + 1   # 那一格「跑得久」的几条睡这么久：只会比它更久，一定过界


def cell_long_run_failure_noted(world):
    """跑超过时限而失败、又没登记续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」（task_server.py 的 finished_preface；时限在自证里缩成 20 秒）：
    登记了续跑的（文件头 # task-server-resumable:）、没到时限的（立刻退 3）、跑成了的都不写。跑得久的几条睡 21 秒（睡只会更久，一定过界）；四条先都交上再逐条等收尾。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_SECONDS": str(LONG_RUN_SELFTEST_SECONDS)})
    with open(os.path.join(world.work, "resumable.sh"), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} segment\nsleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}\nexit 3\n")
    cases = ((f"跑了 {LONG_RUN_SELFTEST_SLEEP_SECONDS} 秒、退 3、没登记续跑", ["bash", "-c", f"sleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}; exit 3"], True),
             ("登记了续跑", ["bash", "resumable.sh", "segment"], False),
             ("没到时限", ["bash", "-c", "exit 3"], False),
             ("跑成了", ["bash", "-c", f"sleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}; exit 0"], False))
    problems = []
    submitted = [(name, world.submit(command, cwd=world.work), expected) for name, command, expected in cases]
    for name, task_id, expected in submitted:
        summary = world.finished(task_id)
        note = summary["task"].get("note") or ""
        if ("没有续跑：重交会从头跑" in note) != expected:
            problems.append(f"{name}：结局说明{'应当' if expected else '不该'}写「没有续跑：重交会从头跑」，实际「{note}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("跑得久而失败的说明：" + "；".join(problems), world.server_log_lines())
    return "跑过时限、失败、没登记续跑的结局说明写重交会从头跑；登记了续跑、没到时限、跑成了的不写"


LOOP_LEAK_BOUNCE_KEY = "selftest-loop-leak-bounce"
LOOP_LEAK_BOUNCE_CAPS = ("30720M", "46080M", "69120M")   # 记录峰值 30G 起 30720M（不乘 1.5）、撞了乘 1.5；69120M 再乘 1.5 就过本机可用上限（总上限 80G 减余量 2G）
LOOP_LEAK_BOUNCE_PEAKS = (30200 * MEBIBYTE, 45500 * MEBIBYTE, 68500 * MEBIBYTE)   # 那三次的峰值：一次比一次大、都不低于那一次上限的 98%（STALL_AT_CAP_FRACTION）
LOOP_LEAK_SCOPE_FILES = {"memory.current": 10628 * MEBIBYTE, "memory.stat": process_memory_stat(900 * MEBIBYTE, 9728 * MEBIBYTE).rstrip("\n")}   # 那三次内存最高那一刻的明细：大头是页缓存


def cell_loop_leak_trend_sentences(world):
    """连着撞上限那一段的走势句（task_server_lib.py 文件头「打回的理由」；任务 10412 四次都记 18 MiB、写了一次比一次大）：严格变大写「峰值一次比一次大」，全部相等写「峰值没变」，
    每一次都没顶到上限的另写 LOOP_LEAK_READING_MISMATCH，顶到了的不写。静态喂 lib.loop_leak_check_text，不起服务。
    弄坏开关 loop-leak-equal-peaks-growing 下相等也写一次比一次大，loop-leak-reading-mismatch-silent 下不写那一句，这一格红。"""
    def attempts(*peaks, cap=64 * MEBIBYTE):
        return [{"number": number, "memory_cap": cap, "memory_cap_text": "64M", "memory_peak": peak} for number, peak in enumerate(peaks, 1)]
    low, mid, high = 18 * MEBIBYTE, 40 * MEBIBYTE, 64 * MEBIBYTE
    problems = []
    for what, cap_hits, wanted, unwanted in (
            ("严格变大", attempts(low, mid, high), ["；峰值一次比一次大；"], ["峰值没变"]),
            ("全部相等", attempts(low, low, low), ["；峰值没变；", lib.LOOP_LEAK_READING_MISMATCH], ["；峰值一次比一次大；"]),
            ("都顶到上限", attempts(high, high), ["；峰值没变；", "每次都顶到上限"], [lib.LOOP_LEAK_READING_MISMATCH]),
            ("先大后小", attempts(mid, low), ["；峰值不是一次比一次大；"], ["峰值没变"])):
        text = lib.loop_leak_check_text(cap_hits)
        missing = [piece for piece in wanted if piece not in text]
        extra = [piece for piece in unwanted if piece in text]
        if missing or extra:
            problems.append(f"{what}的那一段缺 {missing}、不该有 {extra}，实际「{text}」")
    if problems:
        raise SelftestFailure("连着撞上限的走势句：" + "；".join(problems))
    return "严格变大写一次比一次大，全部相等写峰值没变，每一次都没顶到上限的另写读数不准那一句"


def cell_loop_leak_check_on_bounce(world):
    """连着撞上限之后打回（task_server_lib.py 文件头「打回的理由」的 loop_leak=）：总上限 80G、记录峰值 30G，假包装三次都退 250、假 scope 给内存最高那一刻的明细，
    峰值 30200M、45500M、68500M（一次比一次大、都顶满）；第三次再乘 1.5 就到顶，记 memory-cap-at-maximum 打回。wait 的理由行带 loop_leak=，写要查的两类、
    逐次的上限与峰值、「峰值一次比一次大」「每次都顶到上限」与最后一次的「大头是页缓存」；推给会话的结局正文另起一行写同一句。
    另静态喂 reason_fields：排队那一刻就打回（一次都没起）的、只撞过一次的不带 loop_leak=。弄坏开关 loop-leak-check-omitted 下不带，这一格红。"""
    problems = []
    single = {"number": 1, "state": "ended", "outcome": "memory-cap-hit", "memory_cap": 30720 * MEBIBYTE, "memory_cap_text": LOOP_LEAK_BOUNCE_CAPS[0],
              "memory_peak": LOOP_LEAK_BOUNCE_PEAKS[0]}
    for what, attempts in (("排队那一刻就打回、一次都没起的", []), ("只撞过一次就打回的", [single])):
        fields = dict(lib.reason_fields({"id": 1, "outcome": "memory-cap-at-maximum", "exit_code": 236}, attempts, ""))
        if "loop_leak" in fields:
            problems.append(f"{what}不该带 loop_leak=，实际「{fields['loop_leak'][:200]}」")
    world.resources(local=(85, 80))
    world.seed_peak(LOOP_LEAK_BOUNCE_KEY, 30 * GIBIBYTE)
    inbox = FakeInbox(world, "a")
    scope = os.path.join(world.root, "loop-leak-scope")
    os.makedirs(scope)
    for name, value in LOOP_LEAK_SCOPE_FILES.items():
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(f"{value}\n")
    start_inbox_server(world, {"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    peaks = ",".join(str(peak) for peak in LOOP_LEAK_BOUNCE_PEAKS)
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250", f"fake-wrapper-exit-peaks={peaks}"],
                           extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: LOOP_LEAK_BOUNCE_KEY}))
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if summary["task"].get("outcome") != "memory-cap-at-maximum" or caps != list(LOOP_LEAK_BOUNCE_CAPS):
        problems.append(f"上限应当 {' → '.join(LOOP_LEAK_BOUNCE_CAPS)}、第三次到顶记 memory-cap-at-maximum，实际上限 {caps}、{describe(summary)}")
    completed = world.client(["wait", str(task_id)])
    reason = next((line for line in completed.stdout.splitlines() if line.startswith(lib.REASON_MARK + " ")), "")
    try:
        paragraph = dict(word.split("=", 1) for word in shlex.split(reason)[1:] if "=" in word).get("loop_leak", "")
    except ValueError as error:
        paragraph = ""
        problems.append(f"wait 的理由行 shlex 拆不开（{error}）：「{reason[:200]}」")
    wanted = [lib.LOOP_LEAK_CHECK_HEAD, "连着撞了 3 次内存上限", lib.LOOP_LEAK_KINDS, "峰值一次比一次大", "每次都顶到上限", "大头是页缓存"]
    wanted += [f"第 {number} 次上限 {cap} 峰值 {lib.size_text(peak)}" for number, (cap, peak) in enumerate(zip(LOOP_LEAK_BOUNCE_CAPS, LOOP_LEAK_BOUNCE_PEAKS), 1)]
    missing = [text for text in wanted if text not in paragraph]
    if missing:
        problems.append(f"wait 的理由行应当带 loop_leak=「{lib.LOOP_LEAK_CHECK_HEAD}…」、写全两类与逐次的依据（缺 {missing}），实际理由行「{reason[:600]}」")
    head = outcome_head(task_id, "memory-cap-at-maximum")
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "连着撞上限打回的结局")
    lines = messages[0]["text"].split("\n") if messages else []
    if paragraph and paragraph not in lines:
        problems.append(f"推给会话的结局正文应当另起一行写 loop_leak= 那一段，实际「{chr(10).join(lines)[:600]}」")
    if problems:
        raise SelftestFailure("连着撞上限打回带检查逻辑循环与内存泄漏的那一段：" + "；".join(problems), world.server_log_lines())
    return "三次撞上限、峰值一次比一次大且都顶满：wait 理由行带 loop_leak=、结局正文另起一行写同一段；一次都没起、只撞过一次的不带"


REPEATED_CAP_HITS_KEY = "selftest-repeated-cap-hits"
REPEATED_CAP_HITS_RELEASE = "release-repeated-cap-hits"   # 第五次尝试等这个放行文件（相对假仓根：本机与假第二台认同一个名字）


def cell_repeated_cap_hits_reminded_once(world):
    """连着撞上限还在重排（task_server.py 文件头 ①「要处置的提醒」的 repeated-cap-hits）：没记录的命令假包装连退四次 250、第五次起命令等放行；
    第五次跑着的时候收件 socket 恰好收到一条「task-server：提醒（…，不是结局）任务 <号> 连着撞上限（repeated-cap-hits；第 1 回」，出路以异常表那一行的修改意见起头，另起一行写
    「检查代码有没有逻辑循环或内存泄漏：」那一段（两类、第 1 次到第 3 次的上限与峰值），末尾的 TASK_SERVER_ATTENTION 带 loop_leak=；第四次再撞不再推；
    放行之后照跑完（四次 memory-cap-hit、第五次 succeeded），结局正文不带那一段。弄坏开关 repeated-cap-hits-silent 下一条都不推，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(100, 100))
    start_inbox_server(world)
    command = ["bash", "-c", BLOCK_SCRIPT, "block", REPEATED_CAP_HITS_RELEASE, f"fake-peer-wait={REPEATED_CAP_HITS_RELEASE}", "fake-wrapper-exits=250,250,250,250,run"]
    task_id = world.submit(command, extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: REPEATED_CAP_HITS_KEY}), cwd=world.repo)
    head = (reminder_head(task_id, "repeated-cap-hits", state="任务还在排"), reminder_head(task_id, "repeated-cap-hits"))   # 撞上限重排的当口是在排还是在跑，推的那一刻都可能
    problems = []
    try:
        world.wait_for(task_id, lambda summary: any(int(attempt.get("number") or 0) >= 5 and attempt.get("state") == "running" for attempt in summary["attempts"]),
                       "第五次尝试跑起来")
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)   # 只推一回：再看几轮不该多一条
        reminders = inbox.headed(head)
        problems += inbox_problems(reminders, head, "连着撞上限还在重排")
        text = reminders[0]["text"] if reminders else ""
        lines = text.split("\n")
        wanted = (f"出路：{table_advice('repeated-cap-hits', task_id)}",
                  f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=repeated-cap-hits basis=", "loop_leak=")
        missing = [piece for piece in wanted if piece not in text]
        paragraph = next((line for line in lines if line.startswith(lib.LOOP_LEAK_CHECK_HEAD)), "")
        missing += [piece for piece in ("第 1 次上限", "第 3 次上限", lib.LOOP_LEAK_KINDS) if piece not in paragraph]
        if reminders and missing:
            problems.append(f"那一条的正文应当带 {missing}（「{lib.LOOP_LEAK_CHECK_HEAD}…」另起一行），实际「{text[:700]}」")
    finally:
        open(os.path.join(world.repo, REPEATED_CAP_HITS_RELEASE), "w").close()
    done = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in sorted(done["attempts"], key=lambda attempt: int(attempt.get("number") or 0))]
    if outcomes != ["memory-cap-hit"] * 4 + ["succeeded"] or done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：应当四次 memory-cap-hit 之后第五次跑完，实际各次 {outcomes}、{describe(done)}")
    finished = inbox.wait_for(world, outcome_head(task_id, "succeeded"))
    time.sleep(INBOX_QUIET_SECONDS)
    if len(inbox.headed(head)) != 1:
        problems.append(f"同一条任务只推一回：跑完之后 repeated-cap-hits 那一条应当仍只有一条，实际 {len(inbox.headed(head))} 条")
    if finished and lib.LOOP_LEAK_CHECK_HEAD in finished[0]["text"]:
        problems.append(f"跑成了的结局正文不该带「{lib.LOOP_LEAK_CHECK_HEAD}」那一段，实际「{finished[0]['text'][:400]}」")
    if problems:
        raise SelftestFailure("连着撞上限还在重排的提醒：" + "；".join(problems), world.server_log_lines())
    return "四次撞上限、照乘 1.5 重排：第五次跑着时收到恰好一条 repeated-cap-hits（另起一行写检查逻辑循环与内存泄漏），第四次撞不再推，照跑完"


RUN_LIMIT_ACROSS_SECONDS = "6"   # long-run-reminder-across-cap-hits：前四次各跑 1.5 秒撞上限、第五次跑约 1 秒，界 6 秒（哪一次都不到界、各次加起来过界；判据读服务记的各次用时，单次就过界的那一轮作废）
LONG_RUN_ACROSS_ROUNDS = 4       # long-run-reminder-across-cap-hits：作废的一轮（机器忙，有一次单独就跑过了界）重来，至多几轮
RUN_LIMIT_SHORT_SECONDS = "1"    # long-run-confirm-silences、long-run-sessionless-noted：界压到 1 秒，命令等放行才退

ACROSS_CAP_HITS_SCRIPT = ('if [[ "$TASK_SERVER_ATTEMPT" -lt 5 ]]; then echo "attempt $TASK_SERVER_ATTEMPT"; sleep 1.5; exit 250; fi; '
                          'for round in $(seq 1 5); do echo "tick $round"; sleep 0.2; done')   # 前四次各睡 1.5 秒撞上限，第五次约 1 秒跑完

CLIENT_NO_SUCH_TASK_EXIT = 4   # 客户端「没有这个任务」的退出码（task-server/task 的 NO_SUCH_TASK_EXIT）


def long_run_head(task_id):
    """推给会话的跑满两小时提醒的第一行到「（long-run-unconfirmed」为止的一段（头一回与再推都以它起头）。"""
    return reminder_head(task_id, LONG_RUN_KIND)


def reminded_problems(world, task_id, what):
    """没带全量标识的那几条（wait 那一路）：满界 wait 打一行 kind=long-run-unconfirmed、退 ATTENTION_EXIT，任务照跑完 succeeded。→ 问题表。"""
    completed = world.client(["wait", str(task_id)])
    problems, _ = attention_problems(completed, task_id, LONG_RUN_KIND, lib.ATTENTION_EXIT)
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"满界服务不停它，应当跑完 succeeded，实际 {describe(summary)}")
    return [f"{what}{problem}" for problem in problems]


def running_attempt_log(world, task_id):
    """等这条任务有一次在跑的尝试：→ 那一次的日志路径。"""
    summary = world.wait_for(task_id, lambda seen: bool(seen["attempts"]) and seen["attempts"][-1].get("state") in SERVER.ACTIVE_ATTEMPT_STATES, "有一次在跑的尝试")
    return summary["attempts"][-1].get("log") or ""


def cell_long_run_reminded_not_stopped(world):
    """跑满两小时提醒（task_server_lib.py 文件头「异常表」long-run-unconfirmed；界压到 3 秒）：日常开发的一条跑 6 秒，满 3 秒服务不停它、跑完 succeeded、退 0；
    推给会话恰好一条（再推要等 30 分钟，这一格到不了），写命令、范围声明、累计与各次用时、日志末几行，出路写 task cancel 与 task confirm <号> --why 整行。
    弄坏开关 long-run-unconfirmed-silent 下不推，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    found = inbox.wait_for(world, long_run_head(task_id))
    text = found[0]["text"] if found else ""
    summary = world.finished(task_id)
    problems = []
    if summary["task"].get("outcome") != "succeeded" or summary["task"].get("exit_code") != 0:
        problems.append(f"满界服务不停它，应当跑完 succeeded、退 0，实际 {describe(summary)}")
    wanted = ("累计跑了", "各次 第 1 次", "命令：", "范围声明：", "自证：日常开发跑一阵子", "日志末几行：", "tick", f"task-server/task cancel {task_id}", f"task-server/task confirm {task_id} --why")
    missing = [piece for piece in wanted if piece not in text]
    if not found or missing:
        problems.append(f"推给会话的那一条应当写累计与各次用时、命令、范围声明、日志末几行与两条出路（缺 {missing}），实际「{text[:600]}」")
    pushed = inbox.headed(long_run_head(task_id))
    if len(pushed) != 1:
        problems.append(f"满界应当恰好推一回，实际 {len(pushed)} 回")
    if problems:
        raise SelftestFailure("跑满两小时提醒：" + "；".join(problems), world.server_log_lines())
    return f"日常开发的一条累计满 {RUN_LIMIT_SAMPLE_SECONDS} 秒推一回、写全了依据与两条出路，服务不停它、跑完 succeeded"


def cell_long_run_confirm_silences(world):
    """没确认的再推、确认了不再推（task_server.py 文件头 ⑦「跑满两小时」；界压到 1 秒、再推的门槛 2 秒）：等放行的一条满界推一回，没处置再推「第 1 回再推」；
    照整行 task confirm <号> --why 确认，退 0，之后不再推，task status、日志与结局那一条都有「长跑已确认：<why>」；放行之后跑完 succeeded。
    弄坏开关 anomaly-not-repushed 下不再推，long-run-confirmed-still-reminded 下确认了还推，long-run-confirmed-flag-hidden 下没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-long-run-confirm")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SHORT_SECONDS, "TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    problems = []
    flag = f"长跑已确认：{LONG_RUN_WHY_SAMPLE}"
    try:
        log = running_attempt_log(world, task_id)
        first_head, again_head = f"{long_run_head(task_id)}；", f"{long_run_head(task_id)}，没处置、第 1 回再推；第 2 回"
        problems += inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
        problems += inbox_problems(inbox.wait_for(world, again_head), again_head, "再推")
        confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
        if confirmed.returncode != 0 or LONG_RUN_WHY_SAMPLE not in confirmed.stdout:
            problems.append(f"task confirm <号> --why 应当退 0、打那一句，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」")
        settled = len(inbox.headed(long_run_head(task_id)))
        time.sleep(3 * float(REPUSH_SAMPLE_SECONDS))
        if len(inbox.headed(long_run_head(task_id))) != settled:
            problems.append(f"确认了之后不该再推，实际确认时 {settled} 条、等了之后 {len(inbox.headed(long_run_head(task_id)))} 条")
        shown = world.client(["status", str(task_id)])
        if flag not in shown.stdout + shown.stderr:
            problems.append(f"task status 应当有一行「{flag}」，实际「{(shown.stdout + shown.stderr).strip()[:400]}」")
        if LONG_RUN_WHY_SAMPLE not in read_text(log):
            problems.append(f"这一次尝试的日志应当接一行确认的 --why，实际日志「{read_text(log)[-300:]}」")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完 succeeded，实际 {describe(summary)}")
    found = inbox.wait_for(world, outcome_head(task_id, "succeeded"))
    if not found or flag not in found[0]["text"]:
        problems.append(f"推给会话的结局那一条应当有「{flag}」，实际「{(found[0]['text'] if found else '没收到')[:400]}」")
    if problems:
        raise SelftestFailure("跑满两小时确认：" + "；".join(problems), world.server_log_lines())
    return "满界推一回、没处置再推，确认之后不再推，status、日志与结局都有长跑已确认那一行，跑完 succeeded"


def cell_long_run_confirmed_covers_shards(world):
    """分片的子任务照父任务确认过没有算确认过（task_server.py 的 long_run_confirmed_of、run_limit_exempt；文件头 ①「跑满两小时提醒」）：父任务确认过的，它的子任务满界不再提醒；
    子任务自己确认过的照算；父任务没确认的、没有父任务又没确认的不算。套在假调度器上，不起服务。
    弄坏开关 long-run-why-children-reminded 下子任务不看父任务，父任务确认过的那一条也算没确认，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    parents = {1: {"id": 1, "long_run_confirmed": True}, 2: {"id": 2}}
    fake.store = types.SimpleNamespace(task=lambda task_id: (parents.get(task_id), task_id in parents))
    cases = {"父任务确认过的子任务": ({"id": 3, "parent_id": 1}, True), "父任务没确认的子任务": ({"id": 4, "parent_id": 2}, False),
             "自己确认过、父任务没确认的子任务": ({"id": 5, "parent_id": 2, "long_run_confirmed": True}, True), "没有父任务也没确认的": ({"id": 6}, False)}
    found = {name: SERVER.Scheduler.long_run_confirmed_of(fake, task) for name, (task, _) in cases.items()}
    wrong = {name: found[name] for name, (_, wanted) in cases.items() if found[name] != wanted}
    if wrong:
        raise SelftestFailure(f"确认过没有的判法判错了的（名: 实际结果）{wrong}；各条应当是 {[(name, wanted) for name, (_, wanted) in cases.items()]}")
    return "父任务确认过的子任务算确认过，自己确认过的照算，父任务没确认的与没有父任务又没确认的不算"


def cell_long_run_reminder_across_cap_hits(world):
    """撞了内存上限重来的计时归零（task_server.py 的 run_limit_accumulated）：界 6 秒，前四次各跑 1.5 秒退 250（撞上限、乘 1.5 重排），第五次跑约 1 秒：前四次加起来过了界，
    而每一次撞上限之后都从零算、哪一次都不满 6 秒：一条提醒都没有，跑完 succeeded，五次结局是四次 memory-cap-hit、一次 succeeded。
    判据读服务记的各次用时（尝试的 seconds），不信命令睡了多久：一轮里有一次单独就跑满了界（机器忙，哪怕没有开关，满界推提醒也对），或前四次加起来没过界（开关弄坏了也不推），
    这一轮看不出开关，作废、重交一条，至多 LONG_RUN_ACROSS_ROUNDS 轮；每一次单独不到界、前四次加起来过界的那一轮才判有没有提醒。
    弄坏开关 long-run-cap-hits-counted 下撞上限的那几次也累加，第四次跑到 1.5 秒就满界推一回，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(100, 100))
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_ACROSS_SECONDS})
    limit = float(RUN_LIMIT_ACROSS_SECONDS)
    voided = []
    for _ in range(LONG_RUN_ACROSS_ROUNDS):
        task_id = world.submit(["bash", "-c", ACROSS_CAP_HITS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
        summary = world.finished(task_id)
        outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
        if summary["task"].get("outcome") != "succeeded" or outcomes != ["memory-cap-hit"] * 4 + ["succeeded"]:
            raise SelftestFailure(f"四次撞上限、第五次跑完应当 succeeded（各次四个 memory-cap-hit、一个 succeeded），实际 {describe(summary)}、各次 {outcomes}", world.server_log_lines())
        inbox.wait_for(world, outcome_head(task_id, "succeeded"))   # 结局那一条之前的提醒都发完了
        seconds = [float(attempt.get("seconds") or 0.0) for attempt in summary["attempts"]]
        if max(seconds) >= limit or sum(seconds[:4]) < limit:
            voided.append(seconds)
            continue
        pushed = inbox.headed(long_run_head(task_id))
        if pushed:
            raise SelftestFailure(f"每次撞上限后归零，不该提醒：各次用时 {seconds} 秒、哪一次都不到界 {limit:.0f} 秒，实际推了 {len(pushed)} 回：「{pushed[0]['text'][:300]}」", world.server_log_lines())
        return (f"四次撞上限、第五次跑完，各次用时 {seconds} 秒：哪一次都不到 {RUN_LIMIT_ACROSS_SECONDS} 秒的界、前四次加起来过了界，"
                f"每次撞上限后归零、不提醒、照跑完（作废了 {len(voided)} 轮）")
    raise SelftestFailure(f"{LONG_RUN_ACROSS_ROUNDS} 轮里每一轮都看不出开关：有一次单独就跑满了 {RUN_LIMIT_ACROSS_SECONDS} 秒的界、或前四次加起来没过界，各轮各次用时 {voided} 秒", world.server_log_lines())


def cell_long_run_full_gate_not_reminded(world):
    """带全量门禁标识的不提醒：假仓里的 .claude/gate.d/lkmm.sh 带 --full-gate，跑 6 秒过了 3 秒的界，一条提醒都没有、跑完 succeeded。弄坏开关 full-gate-flag-ignored 下照提醒，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", FAKE_COMMIT_GATE], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_COMMIT), cwd=world.repo)
    summary = world.finished(task_id)
    inbox.wait_for(world, outcome_head(task_id, "succeeded"))   # 结局那一条之前的提醒都发完了
    pushed = inbox.headed(long_run_head(task_id))
    if summary["task"].get("outcome") != "succeeded" or pushed:
        raise SelftestFailure(f"提交前跑的提交时门禁带 --full-gate 应当不提醒、跑完 succeeded，实际 {describe(summary)}、推了 {len(pushed)} 回", world.server_log_lines())
    return "带全量标识的提交时门禁过了界不提醒、照跑完"


def cell_long_run_check_not_exempt(world):
    """没带全量标识的门禁提醒（任务 9604 那种带 --check 的门禁跑了 6.2 小时）：同一道门禁带 --check（范围声明写了提交前也不算）的满 3 秒记提醒、照跑完。
    不带 --check 又没带全量标识的提交时就被拒（whole_package_refusal），这一格不交它。"""
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    checked = world.submit(["bash", FAKE_COMMIT_GATE, "--check", "lkmm-cell"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    problems = reminded_problems(world, checked, "带 --check 的门禁，")
    if problems:
        raise SelftestFailure("提交时门禁的提醒：" + "；".join(problems), world.server_log_lines())
    return "带 --check 的门禁满界提醒、照跑完"


def cell_long_run_merge_and_verify_not_exempt(world):
    """merge-and-verify 同样提醒（给了补丁目录、「为什么」写了提交前也一样）：假仓里的 scripts/gate/merge-and-verify.sh 跑 6 秒，满 3 秒记提醒、照跑完。"""
    world.resources(local=(100, 100))
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", FAKE_MERGE_ENTRY, "selftest-patch-directory"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    problems = reminded_problems(world, task_id, "merge-and-verify，")
    if problems:
        raise SelftestFailure("merge-and-verify 的提醒：" + "；".join(problems), world.server_log_lines())
    return "merge-and-verify 满界提醒、照跑完"


def cell_long_run_confirm_refused(world):
    """task confirm <号> --why 不合的（task_server.py 文件头 ⑦「跑满两小时」；界压到 3 秒）：没满界的、--why 去掉空白不到 12 个字或全是空白的、别的会话的、有了结局的、带 --full-gate 的都退 3，
    没有这个任务退 4，--why 后面没给值退 2；合的退 0，再确认一回照退 0、不改原来那一句。
    弄坏开关 long-run-short-why-accepted 下太短的也照收，long-run-confirm-unchecked 下没满界、别的会话与有了结局的也照收，这一格红。"""
    release = os.path.join(world.root, "release-long-run-refused")
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    finished_id = world.submit(["bash", "-c", "exit 0", "already-finished"])
    world.finished(finished_id)
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    problems = []

    def confirm_exit(what, words, wanted, session=SESSION_A):
        completed = world.client(["confirm", *words], session=session)
        if completed.returncode != wanted:
            problems.append(f"{what}应当退 {wanted}，实际退 {completed.returncode}：「{(completed.stdout + completed.stderr).strip()[-300:]}」")
        return completed
    try:
        running_attempt_log(world, task_id)
        confirm_exit("没满界的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
        time.sleep(float(RUN_LIMIT_SAMPLE_SECONDS) + 0.5)
        short = confirm_exit("--why 太短的", [str(task_id), "--why", "太短理由"], 3)
        if f"至少 {lib.LONG_RUN_WHY_MINIMUM_CHARACTERS}" not in short.stderr:
            problems.append(f"--why 太短的应当写「至少 {lib.LONG_RUN_WHY_MINIMUM_CHARACTERS}」，实际「{short.stderr.strip()[-300:]}」")
        confirm_exit("--why 全是空白的", [str(task_id), "--why", "   \t  "], 3)
        confirm_exit("别的会话的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 3, session=SESSION_B)
        confirm_exit("有了结局的", [str(finished_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
        confirm_exit("没有这个任务的", ["999999", "--why", LONG_RUN_WHY_SAMPLE], CLIENT_NO_SUCH_TASK_EXIT)
        confirm_exit("--why 后面没给值的", [str(task_id), "--why"], lib.USAGE_EXIT)
        confirm_exit("合的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 0)
        again = confirm_exit("再确认一回的", [str(task_id), "--why", "另一句写够十二个字的理由，不该换掉原来的"], 0)
        if LONG_RUN_WHY_SAMPLE not in again.stdout:
            problems.append(f"再确认一回应当说原来就确认过、留原来那一句，实际「{again.stdout.strip()[-300:]}」")
    finally:
        open(release, "w").close()
    world.finished(task_id)
    gate_id = world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    running_attempt_log(world, gate_id)
    time.sleep(float(RUN_LIMIT_SAMPLE_SECONDS) + 0.5)
    confirm_exit("带 --full-gate 的", [str(gate_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
    world.finished(gate_id)
    if problems:
        raise SelftestFailure("task confirm --why 的拒：" + "；".join(problems), world.server_log_lines())
    return "没满界、--why 太短或全是空白、别的会话、有了结局、带 --full-gate 的退 3，没有这个任务退 4，没给值退 2；合的退 0，再确认照退 0、不改原来那一句"


def cell_long_run_sessionless_noted(world):
    """没有会话的跑满两小时（task_server.py 文件头 ①「要处置的提醒」的 long-run-unconfirmed；界压到 1 秒）：人手 --manual 交的一条不推，这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的待确认、
    恰好一行，status 打「长跑待确认」；同样可以确认（退 0），status 换成「长跑已确认」，放行后跑完。
    弄坏开关 long-run-sessionless-unnoted 下不写日志那一行，long-run-unconfirmed-silent 下都没有，这一格红。"""
    release = os.path.join(world.root, "release-long-run-sessionless")
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SHORT_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=None)
    problems = []
    mark = f"{lib.LONG_RUN_LOG_MARK} 长跑待确认"
    try:
        log = running_attempt_log(world, task_id)
        patience = world.patience(INBOX_WAIT_SECONDS)
        while mark not in read_text(log) and patience.waiting():
            time.sleep(POLL_SECONDS)
        lines = [line for line in read_text(log).splitlines() if line.startswith(mark)]
        if len(lines) != 1 or "累计跑了" not in lines[0] or f"task-server/task confirm {task_id} --why" not in lines[0]:
            problems.append(f"日志里应当恰好一行「{mark}…」写累计与确认的写法，实际 {len(lines)} 行：「{(lines or [''])[0][:300]}」")
        shown = world.client(["status", str(task_id)], session=None)
        if "长跑待确认" not in shown.stdout:
            problems.append(f"task status 应当打「长跑待确认」，实际「{shown.stdout.strip()[:400]}」")
        confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE], session=None)
        again = world.client(["status", str(task_id)], session=None)
        if confirmed.returncode != 0 or f"长跑已确认：{LONG_RUN_WHY_SAMPLE}" not in again.stdout or "长跑待确认" in again.stdout:
            problems.append(f"人手交的同样可以确认，确认之后 status 换成已确认，实际退 {confirmed.returncode}、status「{again.stdout.strip()[:400]}」")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完 succeeded，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("没有会话的跑满两小时：" + "；".join(problems), world.server_log_lines())
    return "人手交的不推，日志接恰好一行待确认、status 打一行，同样可以确认"


SPLIT_EXEMPT_ITEM = "selftest-exempt-item"   # long-run-split-reminded-at-outcome：豁免表里点名的项


def split_items_of(fake, record, attempts):
    """attention_of_record 里 long-run-split 那一项（没有交 None）。"""
    fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [(dict(attempt), True) for attempt in attempts])
    found = [item for item in SERVER.Scheduler.attention_of_record(fake, record, time.time()) if item["kind"] == "long-run-split"]
    return found[0] if found else None


def cell_long_run_split_reminded_at_outcome(world):
    """跑过 7200 秒的项出结局时推一条要拆的提醒（task_server.py 的 attention_of_record、long_run_split_items、settled_run_seconds、long_run_split_exempt；.claude/rules/long-run-split.md「判据」「豁免」）：
    出了结局的任务服务记的用时（各次尝试记下的 seconds，撞了内存上限、照大一档重来的从最后一次撞上限之后那一次起算）超过 LONG_RUN_SPLIT_SECONDS 又不在豁免表里的，交一项 long-run-split，
    依据写这是按片或按整条任务的用时判的，出路指到那份规则。假秒，不真等：整条任务跑了 7300 秒的出一项；7100 秒的不出；命令里点名了豁免表里的项的不出（同一份用时，表里没有就出）；
    撞上限重来的：前两次撞上限各 5000、4000 秒，最后一次跑成 3000 秒的不出（从最后一次撞上限之后那一次起算），最后一次跑成 7300 秒的出；分片的一片跑了 7300 秒的出、依据写第几片、按片判。
    弄坏开关 long-run-split-silent 下一条都不出，long-run-split-cap-hits-counted 下撞上限的那几次也累加（撞上限重来、最后一次 3000 秒的也出），
    long-run-split-exempt-ignored 下豁免表不看（点名了豁免项的也出），这一格红。"""
    def attempt(number, seconds, outcome="succeeded"):
        return {"number": number, "seconds": seconds, "outcome": outcome, "state": "ended"}

    problems = []
    command = ["bash", "scripts/gate/some-entry.sh", "--item", "ordinary-item"]
    exempt_command = ["bash", "scripts/gate/some-entry.sh", "--item", SPLIT_EXEMPT_ITEM]
    record = {"id": 701, "state": "finished", "outcome": "succeeded", "exit_code": 0, "command": command}
    fake, _ = long_run_split_scheduler(world, [f"{SPLIT_EXEMPT_ITEM}  # 用户点名豁免（records/selftest.md:1）"])
    item = split_items_of(fake, record, [attempt(1, 7300.0)])
    if item is None or item["threshold"] != 1 or "整条任务" not in item["basis"] or "按整条任务的用时判" not in item["basis"] or ".claude/rules/long-run-split.md" not in item["next"]:
        problems.append(f"整条任务跑了 7300 秒：应当出一项 long-run-split（提醒门槛 1、依据写按整条任务的用时判、出路指到 long-run-split.md），实际 {item}")
    if split_items_of(fake, record, [attempt(1, 7100.0)]) is not None:
        problems.append("整条任务跑了 7100 秒：不到界，不该出")
    if split_items_of(fake, dict(record, command=exempt_command), [attempt(1, 7300.0)]) is not None:
        problems.append(f"命令点名了豁免表里的项 {SPLIT_EXEMPT_ITEM}：不该出")
    reset_hits = [attempt(1, 5000.0, "memory-cap-hit"), attempt(2, 4000.0, "memory-cap-hit")]
    if split_items_of(fake, record, reset_hits + [attempt(3, 3000.0)]) is not None:
        problems.append("撞上限重来的（5000、4000 秒各撞一次上限，最后一次跑成 3000 秒）：从最后一次撞上限之后那一次起算，3000 秒不到界，不该出")
    if split_items_of(fake, record, [attempt(1, 9000.0, "memory-cap-hit"), attempt(2, 7300.0)]) is None:
        problems.append("撞一次上限（9000 秒）之后最后一次跑成 7300 秒：从最后一次撞上限之后那一次起算 7300 秒，应当出")
    shard = split_items_of(fake, dict(record, id=702, parent_id=700, role="shard", shard_index=3), [attempt(1, 7300.0)])
    if shard is None or "第 3 片" not in shard["basis"] or "按片的用时判" not in shard["basis"]:
        problems.append(f"分片的第 3 片跑了 7300 秒：应当出一项、依据写第 3 片、按片的用时判，实际 {shard}")
    if problems:
        raise SelftestFailure("跑过 7200 秒的项出结局时推要拆的提醒：" + "；".join(problems))
    return "整条任务 7300 秒出、7100 秒不出、豁免表里的项不出；撞上限重来的从最后一次之后起算；分片的按片判"


LONG_RUN_IDLE_ATTENTION_SECONDS = "2"   # long-run-idle-cores 那几格：跑满这么久就看（换掉 LONG_RUN_ATTENTION_SECONDS 的 600 秒）


def cell_long_run_idle_cores_reported(world):
    """跑得久而核没用起来（task_server.py 文件头 ①「要处置的提醒」的 long-run-idle-cores）：门槛是跑满 600 秒、平均在用核数不到线程数的 24/31（31 线程时 23.9 核算、24.1 核不算，判法 cores_short_of_threads）；
    假 scope 的 CPU 钉着不涨（平均约 0 核），跑满 2 秒（换掉 600 秒），wait 打一行 kind=long-run-idle-cores、依据写跑了多久、给几线程与平均核数，原因问资源运用是否合理、脚本能不能并行或切片，退 ATTENTION_EXIT；
    只提醒、照跑，放行之后跑完。弄坏开关 long-run-idle-silent 下不报，wait 等到命令自己退；long-run-idle-quarter-threshold 下 23.9 核不算，这一格红。"""
    problems = []
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    if SERVER.LONG_RUN_ATTENTION_SECONDS != 600:
        problems.append(f"门槛应当是 600 秒，实际 {SERVER.LONG_RUN_ATTENTION_SECONDS}")
    if not SERVER.Scheduler.cores_short_of_threads(fake, 23.9, 31) or SERVER.Scheduler.cores_short_of_threads(fake, 24.1, 31):
        problems.append("31 线程：平均 23.9 核应当算没用满、24.1 核不算")
    scope = os.path.join(world.root, "long-run-idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    release = os.path.join(world.root, "release-long-run-idle")
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS": LONG_RUN_IDLE_ATTENTION_SECONDS, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    found, fields = attention_problems(completed, task_id, "long-run-idle-cores", lib.ATTENTION_EXIT)
    problems += found
    basis = fields.get("basis", "")
    if "平均只用" not in basis or "线程" not in basis or "核" not in basis or "已跑" not in basis:
        problems.append(f"依据应当写已跑多久、给几线程与平均只用几核，实际「{basis}」")
    if "资源运用是否合理" not in fields.get("why", "") or "并行或切片" not in fields.get("why", ""):
        problems.append(f"原因应当问资源运用是否合理、脚本能不能并行或切片，实际「{fields.get('why')}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("跑得久而核没用起来：" + "；".join(problems), world.server_log_lines())
    return f"界是 24/31；跑满 {LONG_RUN_IDLE_ATTENTION_SECONDS} 秒、CPU 不涨：wait 打 kind=long-run-idle-cores（依据「{basis[:80]}」），照跑完"


def cell_fills_any_cap_reported(world):
    """按给的上限开并行（task_server.py 文件头 ①「要处置的提醒」的 fills-any-cap）：同一命令键记下三次跑成的尝试，峰值都是上限的 95%；再交一条，wait 打一行 kind=fills-any-cap、
    依据写三次的上限与峰值、出路写「这条命令按给的上限开并行，核实」、退 ATTENTION_EXIT；只提醒、照跑，放行之后跑完。
    弄坏开关 fills-any-cap-silent 下不报，wait 等到命令自己退，这一格红。"""
    key = "selftest-fills-any-cap"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    release = os.path.join(world.root, "release-fills-any-cap")
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key})
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "fills-any-cap", lib.ATTENTION_EXIT)
    if "这条命令按给的上限开并行，核实" not in fields.get("next", "") or "上限" not in fields.get("basis", ""):
        problems.append(f"依据应当写三次的上限与峰值、出路写「这条命令按给的上限开并行，核实」，实际依据「{fields.get('basis')}」、出路「{fields.get('next')}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("按给的上限开并行：" + "；".join(problems), world.server_log_lines())
    return f"同键三次峰值都是上限的 95%：wait 打 kind=fills-any-cap（「{fields.get('basis', '')[:80]}」），照跑完"


OLD_METRIC_PEAK_BYTES = 20 * GIBIBYTE   # old-metric-peaks-not-counted 那一条旧行记的峰值：照改前读它就起步 20480M


def cell_old_metric_peaks_not_counted(world):
    """旧口径的峰值不进起步（task_server_lib.py 文件头「资源记录」的峰值口径一栏；批 92 之前记的 memory_peak 含页缓存）：同一命令键只有一条没写峰值口径的旧行（峰值 20G），
    新交的照没有记录办：上限与记账 4096M、prediction 记 UNRECORDED_HISTORY；同键另一条写了这一版口径的（峰值 1536M）照它定 1536M（不乘 1.5）。
    弄坏开关 old-metric-peaks-read 下旧行照改前进定额度（20480M），这一格红。"""
    old_key, new_key = "selftest-old-metric", "selftest-new-metric"
    world.seed_attempt({"command_key": old_key, "outcome": "succeeded", "memory_peak": OLD_METRIC_PEAK_BYTES}, metric=None)
    world.seed_peak(new_key, 1536 * MEBIBYTE)
    world.start_server()
    expected = {"只有旧行的": (old_key, UNRECORDED_DEFAULT_PINNED_TEXT, lib.UNRECORDED_HISTORY), "写了这一版口径的": (new_key, "1536M", lib.RECORD_HISTORY)}
    problems = []
    for name, (key, cap_text, history) in expected.items():
        summary = world.finished(world.submit(["bash", "-c", "exit 0", key], extra={lib.KEY_VARIABLE: key}))
        attempt = first_attempt(summary)
        if attempt.get("memory_cap_text") != cap_text or attempt.get("prediction") != history or summary["task"].get("outcome") != "succeeded":
            problems.append(f"{name}上限应当是 {cap_text}、prediction 记 {history}、跑完，实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}")
    if problems:
        raise SelftestFailure("旧口径的峰值不进起步：" + "；".join(problems), world.server_log_lines())
    return f"只有旧口径那一行的命令照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，写了这一版口径的照峰值（1536M）"


FILLS_CAP_CLIMB_CAPS = (6144 * MEBIBYTE, 9216 * MEBIBYTE, 13824 * MEBIBYTE)   # fills-any-cap-cap-hits-not-reported：照上限一档档撞上去的三次（6.0、9.0、13.5 GiB）
FILLS_CAP_CLIMB_PEAK_PERCENT = 99             # 那三次撞上限时的峰值占各自上限的百分数（撞的那一次峰值本来就贴着上限）


def cell_fills_any_cap_cap_hits_not_reported(world):
    """照上限一档档撞上去的不算按上限开并行（task_server.py 文件头 ①「要处置的提醒」的 fills-any-cap；任务 9616 那条静态核 6.0、9.0、13.5 GiB）：
    同一命令键记下三次撞了上限（memory-cap-hit）的尝试，峰值都是各自上限的 99%；再交一条：确认单「交的时候判得出的异常」里没有 fills-any-cap，wait 不打 kind=fills-any-cap，跑完。
    同键三次跑成都贴着上限的照报，由 fills-any-cap-reported 那一格钉。弄坏开关 fills-any-cap-counts-cap-hits 下撞了上限的也算进三次，确认单里列出 fills-any-cap，这一格红。"""
    key = "selftest-fills-any-cap-climb"
    for cap in FILLS_CAP_CLIMB_CAPS:
        world.seed_attempt({"command_key": key, "outcome": "memory-cap-hit", "memory_peak": cap * FILLS_CAP_CLIMB_PEAK_PERCENT // 100, "memory_cap": cap})
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "fills-any-cap-climb"], extra={lib.KEY_VARIABLE: key})
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    if submitted.returncode != 0 or not found or lib.confirm_needed_fields(submitted.stdout) is None:
        raise SelftestFailure(f"交了应当退 0、打任务号与确认单，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr] + world.server_log_lines())
    task_id = int(found.group(1))
    if "fills-any-cap：" in submitted.stdout:
        problems.append(f"三次都是撞了上限的，确认单里不该列 fills-any-cap，实际「{submitted.stdout.strip()[-500:]}」")
    world.confirm_submitted(submitted)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    if "kind=fills-any-cap" in completed.stdout:
        problems.append(f"三次都是撞了上限的，wait 不该打 kind=fills-any-cap，实际「{completed.stdout.strip()[-300:]}」")
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("照上限一档档撞上去的：" + "；".join(problems), world.server_log_lines())
    return "同键三次都是撞了上限、峰值贴着上限：确认单与 wait 都不报 fills-any-cap，照跑完"


FILLS_CAP_RECORD_CAPS = (1400 * MEBIBYTE, 1500 * MEBIBYTE, 1000 * MEBIBYTE)   # fills-any-cap-record-caps-not-reported：照记录定的三次上限（任务 13774 那条命令）
FILLS_CAP_RECORD_PEAKS = (1400 * MEBIBYTE, 1400 * MEBIBYTE, 1000 * MEBIBYTE)   # 那三次的峰值：贴着各自的上限
FILLS_CAP_GROWN_CAPS = (4096 * MEBIBYTE, 6144 * MEBIBYTE, 9216 * MEBIBYTE)     # 没记录时起在 4096M、一档档放大的三次上限


def cell_fills_any_cap_record_caps_not_reported(world):
    """上限照记录定的不算按上限开并行（task_server.py 的 fills_any_cap_items；任务 13774 的依据是「上限 1.4 GiB 峰值 1.4 GiB、上限 1.5 GiB 峰值 1.4 GiB、上限 1.0 GiB 峰值 1.0 GiB」，
    而那条命令不读上限）：同一命令键记下三次跑成的尝试，各自的上限照记下的峰值定（尝试的 prediction 一栏是 record）、峰值贴着上限：确认单里没有 fills-any-cap；
    另一个命令键三次跑成、上限是没记录时从 4096M 起一档档放大的（prediction 是 unrecorded-default、retry）、峰值都跟着涨到贴着上限：确认单里照列 fills-any-cap。
    弄坏开关 fills-any-cap-counts-record-caps 下照记录定的上限也算，头一个命令键也被报，这一格红。"""
    keys = {"record": "selftest-fills-any-cap-record", "grown": "selftest-fills-any-cap-grown"}
    for cap, peak in zip(FILLS_CAP_RECORD_CAPS, FILLS_CAP_RECORD_PEAKS):
        world.seed_attempt({"command_key": keys["record"], "outcome": "succeeded", "memory_peak": peak, "memory_cap": cap, "prediction": lib.RECORD_HISTORY})
    for cap, history in zip(FILLS_CAP_GROWN_CAPS, (lib.UNRECORDED_HISTORY, "retry", "retry")):
        world.seed_attempt({"command_key": keys["grown"], "outcome": "succeeded", "memory_peak": cap * 97 // 100, "memory_cap": cap, "prediction": history})
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    for what, key, reported in (("上限照记录定的", keys["record"], False), ("上限一档档放大的", keys["grown"], True)):
        submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", f"fills-any-cap-{key[-6:]}"], extra={lib.KEY_VARIABLE: key})
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
        if submitted.returncode != 0 or not found or lib.confirm_needed_fields(submitted.stdout) is None:
            raise SelftestFailure(f"交了应当退 0、打任务号与确认单，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr] + world.server_log_lines())
        if ("fills-any-cap：" in submitted.stdout) != reported:
            problems.append(f"{what}三次峰值都贴着上限：确认单里应当{'列' if reported else '不列'} fills-any-cap，实际「{submitted.stdout.strip()[-400:]}」")
        world.confirm_submitted(submitted)
        world.client(["wait", found.group(1)], timeout=ATTENTION_WAIT_SECONDS)
        done = world.finished(int(found.group(1)))
        if done["task"].get("outcome") != "succeeded":
            problems.append(f"{what}：应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("上限照记录定的不报按上限开并行：" + "；".join(problems), world.server_log_lines())
    return "三次上限照记录定、峰值贴着：不报 fills-any-cap；三次上限是没记录时一档档放大的、峰值跟着涨：照报"


RELEASE_HOLD_SCRIPT = 'release="$1"; shift; printf "%s\\n" "$@"; for round in $(seq 1 600); do [[ -e "$release" ]] && exit 0; sleep 0.1; done; exit 9'   # 打几行、等放行才退
RELEASE_WAIT_SECONDS = 90.0            # 等判到哪一支的安静时限：开关下等不到的要等命令自己 60 秒退（RELEASE_HOLD_SCRIPT），比它长
HARD_LIMIT_OUT_OF_REACH_SECONDS = "1000000000"   # 起头那一段不该判到硬上限的格的上限（TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS）：格里跑多久都够不着


def held_command(release, *lines, name="held"):
    return ["bash", "-c", RELEASE_HOLD_SCRIPT, name, release, *lines]


RUN_HARD_LIMIT_EXIT = 229                     # over-four-hours 的退出码（task_server_lib.py 的 OUTCOME_TYPES）
RUN_HARD_LIMIT_NOTE = "硬上限"                 # 结局说明、推送正文与确认单都写这个词
RUN_HARD_LIMIT_ADVICE = ".claude/rules/long-run-split.md"   # 修改意见里的出路：指到拆分规则，不再写缩域、收小范围
RUN_HARD_LIMIT_SHARD_SECONDS = 15             # run-hard-limit-shard-stopped：界要够那一件走到第 0、1 片在跑、第 2 片排着
RUN_HARD_LIMIT_CONFIRM_SECONDS = 10           # run-hard-limit-why-still-stops：界要够跑满两小时那一道（压到 1 秒）之后填 why
RUN_HARD_LIMIT_ACROSS_SECONDS = 6             # run-hard-limit-cap-hit-resets：四次撞上限各跑 1.5 秒、第五次约 1 秒：每一次都不到界的一半多，算上撞上限的那几次第四次就满（机器忙时单次放慢到界的 3 倍多才会误判）


def hard_limit_settings(seconds, **changed):
    """四小时硬上限那几格服务的设置：上限压到 seconds 秒。"""
    return dict({"TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS": str(seconds)}, **changed)


def hard_limit_problems(summary, what, attempts=1):
    """四小时硬上限打回的共同判据：记 over-four-hours、退 229、不重排（attempts 次尝试，最后那一次记同一个结局）、说明写硬上限。→ 问题表。"""
    task, problems = summary["task"], []
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if task.get("state") != "finished" or task.get("outcome") != lib.RUN_HARD_LIMIT_OUTCOME or task.get("exit_code") != RUN_HARD_LIMIT_EXIT:
        problems.append(f"{what}：应当记 {lib.RUN_HARD_LIMIT_OUTCOME}、退 {RUN_HARD_LIMIT_EXIT}，实际状态 {task.get('state')}、{describe(summary)}")
    if len(outcomes) != attempts or (attempts and outcomes[-1] != lib.RUN_HARD_LIMIT_OUTCOME):
        problems.append(f"{what}：应当有 {attempts} 次尝试、最后那一次记 {lib.RUN_HARD_LIMIT_OUTCOME}，实际 {outcomes}")
    if RUN_HARD_LIMIT_NOTE not in str(task.get("note")):
        problems.append(f"{what}：说明应当写{RUN_HARD_LIMIT_NOTE}，实际「{task.get('note')}」")
    return problems


def held_until_finished(world, task_id, release):
    """等任务收尾，不论等到没有都放行它的命令。→ 那时的 summary。"""
    try:
        return world.finished(task_id, timeout=RELEASE_WAIT_SECONDS)
    finally:
        open(release, "w").close()


def cell_run_hard_limit_arithmetic(world):
    """四小时硬上限的算术（task_server_lib.py 的 over_run_hard_limit、covered_seconds）：常量是 14400 秒，恰好等于算到、差半秒不算；并集：同时在跑的不重复算、空档不算、套着的不多算。
    弄坏开关 run-hard-limit-exclusive、run-hard-limit-loose、run-hard-limit-parent-summed 各红。"""
    problems = []
    limit = float(SERVER.RUN_HARD_LIMIT_SECONDS)
    if limit != 14400:
        problems.append(f"上限应当是 14400 秒，实际 {limit}")
    for seconds, over in ((14400.0, True), (14400.5, True), (14399.5, False), (7200.0, False)):
        if lib.over_run_hard_limit(seconds, limit) != over:
            problems.append(f"{seconds} 秒：到上限应当 {over}，实际 {lib.over_run_hard_limit(seconds, limit)}")
    for intervals, seconds in (([], 0.0), ([(0.0, 60.0), (0.0, 60.0)], 60.0), ([(0.0, 10.0), (5.0, 20.0)], 20.0), ([(0.0, 10.0), (100.0, 110.0)], 20.0), ([(0.0, 100.0), (10.0, 20.0), (90.0, 200.0)], 200.0)):
        if abs(lib.covered_seconds(intervals) - seconds) > 0.001:
            problems.append(f"时段 {intervals} 的并集应当 {seconds} 秒，实际 {lib.covered_seconds(intervals)}")
    if problems:
        raise SelftestFailure("四小时硬上限的算术：" + "；".join(problems))
    return "上限 14400 秒，恰好等于算到；挂钟取时段的并集，同时在跑的不重复算、空档不算"


def hard_limit_fake_scheduler(records=()):
    """判四小时硬上限那两格的假调度器：上限 100 秒，尝试记录 1 的进度 1/1000，records 是 store.task 读得到的几条任务。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    table = {record["id"]: record for record in records}
    fake.store = types.SimpleNamespace(task=lambda task_id: (table[task_id], True) if task_id in table else None, attempts_of=lambda task_id: [])
    fake.run_hard_limit_seconds, fake.run_limit_seconds, fake.peer_root = 100.0, 60.0, "/"
    fake.attempt_progress = {1: (1, 1000)}
    fake.attempt_progress_baseline = {1: {"at": time.time() - 5, "done": 0, "offset": 0}}
    return fake


def cell_shard_hard_limit_by_each_piece(world):
    """分片的四小时硬上限按每一片自己的累计（task_server.py 的 parent_run_hard_limit、run_hard_limit_verdict；用户定）：假调度器上限 100 秒（同四小时，缩了比例）。
    三片依次排队、各跑 75 秒（照挂钟从头到尾 225 秒、每片都不到 100）：整件不判（parent_run_hard_limit 交 None），每一片照自己的累计都不停；
    一片单跑满 120 秒：那一片停，跑了 50 秒的兄弟片照跑；最先没成的是满界的那一片（first_failed_shard），整件收齐后照它收尾。
    弄坏开关 shard-whole-by-wall-clock 下整件按挂钟（225 秒到 100）判到，run-hard-limit-skipped、run-hard-limit-children-spared 下满界的那一片不停，这一格红。"""
    now = time.time()
    command = {"command": ["bash", "-c", "exit 0"], "cwd": "/"}
    pieces = [{"id": 10 + index, "parent_id": 9, "role": "shard", "shard_index": index, **command} for index in range(3)]

    def row(number, start, end):
        return ({"number": number, "state": "ended" if end is not None else "running", "started_at": now - start, "ended_at": None if end is None else now - end,
                 "outcome": "succeeded", "seconds": None if end is None else float(start - end)}, True)
    sequential = {10: [row(1, 230, 155)], 11: [row(1, 155, 80)], 12: [row(1, 75, None)]}
    problems = []
    fake = hard_limit_fake_scheduler([{"id": 9, **command}])
    fake.store = types.SimpleNamespace(task=lambda task_id: ({"id": 9, **command}, True) if task_id == 9 else None,
                                       attempts_of=lambda task_id: sequential.get(task_id, []))
    whole = SERVER.Scheduler.parent_run_hard_limit(fake, {"id": 9, **command}, pieces)
    if whole is not None:
        problems.append(f"三片依次排队各跑 75 秒：整件不应当判到上限，实际 {whole}")
    for piece in pieces:
        attempts = sequential[piece["id"]]
        running = next((record for record, _ in attempts if record["state"] == "running"), None)
        if running is not None and SERVER.Scheduler.run_hard_limit_verdict(fake, piece, dict(running, id=99)):
            problems.append(f"第 {piece['shard_index']} 片自己只跑了 75 秒：不应当停")
    fake.store = types.SimpleNamespace(task=lambda task_id: ({"id": 9, **command}, True) if task_id == 9 else None, attempts_of=lambda task_id: [])
    if not SERVER.Scheduler.run_hard_limit_verdict(fake, pieces[0], {"id": 98, "number": 1, "started_at": now - 120}):
        problems.append("一片单跑满 120 秒（上限 100）：那一片应当停")
    if SERVER.Scheduler.run_hard_limit_verdict(fake, pieces[1], {"id": 97, "number": 1, "started_at": now - 50}):
        problems.append("兄弟片跑了 50 秒：应当照跑、不停")
    shards = [dict(pieces[0], state="finished", outcome=lib.RUN_HARD_LIMIT_OUTCOME, finished_at=now), dict(pieces[1], state="running"), dict(pieces[2], state="queued")]
    failed = SERVER.Scheduler.first_failed_shard(fake, shards)
    if failed is None or failed.get("id") != pieces[0]["id"]:
        problems.append(f"最先没成的应当是满界停的第 0 片（整件收齐后照它的 {lib.RUN_HARD_LIMIT_OUTCOME} 收尾），实际 {failed}")
    if problems:
        raise SelftestFailure("分片的四小时硬上限按每一片自己的累计：" + "；".join(problems))
    return "三片依次各跑 75 秒的整件不判；一片单跑满界的停那一片、兄弟片照跑，整件照那一片收尾"


def cell_run_hard_limit_stopped(world):
    """满上限停（task_server.py 文件头 ⑦「四小时硬上限」；上限压到 2 秒）：命令一直等放行：服务停这一次、记 over-four-hours、退 229、不重排，推给提交它的会话，wait 退 229、末行 type=over-four-hours。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，inbox-not-sent、inbox-auth-omitted 下收不到或收得不对，exit-code-dropped 下 wait 退 0，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-hard-limit-stopped")
    start_inbox_server(world, hard_limit_settings(2))
    task_id = world.submit(held_command(release, "tick"), extra=inbox.environment())
    summary = held_until_finished(world, task_id, release)
    problems = hard_limit_problems(summary, "满上限的")
    head = outcome_head(task_id, lib.RUN_HARD_LIMIT_OUTCOME)
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "结局那一条")
    if messages and not all(word in messages[0]["text"] for word in (RUN_HARD_LIMIT_NOTE, RUN_HARD_LIMIT_ADVICE)):
        problems.append(f"推送正文应当写{RUN_HARD_LIMIT_NOTE}与出路「{RUN_HARD_LIMIT_ADVICE}」，实际「{messages[0]['text'][:400]}」")
    code, last = wait_last_line(world, task_id)
    if code != RUN_HARD_LIMIT_EXIT or f"type={lib.RUN_HARD_LIMIT_OUTCOME}" not in last:
        problems.append(f"wait 应当退 {RUN_HARD_LIMIT_EXIT}、末行 type={lib.RUN_HARD_LIMIT_OUTCOME}，实际退 {code}、末行「{last}」")
    if problems:
        raise SelftestFailure("满上限停：" + "；".join(problems), world.server_log_lines())
    return "累计满上限：停这一次、记 over-four-hours、不重排，推给会话，wait 退 229"


def cell_run_hard_limit_why_still_stops(world):
    """带 --why 照停：跑满两小时那一道压到 1 秒、硬上限压到 10 秒：跑过 1 秒填 why 退 0、记长跑已确认，到了硬上限照样停、记 over-four-hours。
    弄坏开关 run-hard-limit-confirmed-spared、run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-why")
    world.start_server(extra=hard_limit_settings(RUN_HARD_LIMIT_CONFIRM_SECONDS, TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS=RUN_LIMIT_SHORT_SECONDS))
    task_id = world.submit(held_command(release, "tick"))
    world.running(task_id)
    time.sleep(float(RUN_LIMIT_SHORT_SECONDS) + 0.5)
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    problems = [] if confirmed.returncode == 0 else [f"跑满界填 why 应当退 0，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」"]
    summary = held_until_finished(world, task_id, release)
    problems += hard_limit_problems(summary, "填过 why 的")
    if not summary["task"].get("long_run_confirmed"):
        problems.append(f"应当记着长跑已确认，实际 {summary['task'].get('long_run_confirmed')}")
    if problems:
        raise SelftestFailure("带 --why 照停：" + "；".join(problems), world.server_log_lines())
    return "填了 why 的到了硬上限照停"


def cell_run_hard_limit_full_gate_not_stopped(world):
    """带 --full-gate 的不判：假仓里的提交时门禁跑 3 秒、上限压到 1 秒：跑成。弄坏开关 run-hard-limit-full-gate-judged 下停了，这一格红。"""
    path = os.path.join(world.repo, FAKE_COMMIT_GATE)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    write_executable(path, "#!/usr/bin/env bash\nsleep 3\nexit 0\n")
    world.resources(local=(30, 30))
    world.start_server(extra=hard_limit_settings(1))
    task_id = world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded" or not summary["task"].get("full_gate") or len(summary["attempts"]) != 1:
        raise SelftestFailure(f"带 --full-gate 的过了上限不判、跑成、只有一次尝试，实际 {describe(summary)}、全量标识 {summary['task'].get('full_gate')}", world.server_log_lines())
    return "带 --full-gate 的跑 3 秒过了 1 秒的上限：不停、跑成"


def cell_run_hard_limit_cap_hit_resets(world):
    """撞上限归零后重新计（task_server.py 的 run_limit_accumulated）：上限 6 秒，四次撞上限各跑 1.5 秒、第五次跑约 1 秒，五次加起来过了上限，每一次都不到上限的一半多：跑成，五次结局是四次 memory-cap-hit、一次 succeeded。
    单次用时只占上限的四分之一，机器忙时慢到三倍多才会误判。弄坏开关 long-run-cap-hits-counted 下第四次就满上限、停了，retry-overwrites-attempts 下尝试的记录没了，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server(extra=hard_limit_settings(RUN_HARD_LIMIT_ACROSS_SECONDS))
    task_id = world.submit(["bash", "-c", ACROSS_CAP_HITS_SCRIPT])
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if summary["task"].get("outcome") != "succeeded" or outcomes != ["memory-cap-hit"] * 4 + ["succeeded"]:
        raise SelftestFailure(f"撞上限之后归零：应当跑成，五次结局四个 memory-cap-hit、一个 succeeded，实际 {describe(summary)}、各次 {outcomes}", world.server_log_lines())
    return f"四次撞上限各跑 1.5 秒、第五次跑约 1 秒：每次撞上限后归零，不满 {RUN_HARD_LIMIT_ACROSS_SECONDS} 秒，跑成"


def cell_run_hard_limit_writer_stopped(world):
    """改主工作区的也停（task_server.py 的 workspace_write_item）：hook-swap.py 跑满 2 秒的上限：停、记 over-four-hours；wait 退 229、在末行之前打一行 kind=workspace-write-interrupted。
    弄坏开关 run-hard-limit-writers-spared、run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，run-hard-limit-writer-unreminded、workspace-write-interrupted-unreported 下不报，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-writer")
    world.start_server(extra=hard_limit_settings(2))
    task_id = world.submit(held_command(release, "tick", "scripts/gate/hook-swap.py"), cwd=world.repo)
    summary = held_until_finished(world, task_id, release)
    problems = hard_limit_problems(summary, "改主工作区的")
    completed = world.client(["wait", str(task_id)])
    lines = attention_lines(completed)
    if completed.returncode != RUN_HARD_LIMIT_EXIT or len(lines) != 1 or "kind=workspace-write-interrupted" not in lines[0]:
        problems.append(f"wait 应当退 {RUN_HARD_LIMIT_EXIT}、打一行 kind=workspace-write-interrupted，实际退 {completed.returncode}、提醒 {lines}")
    if problems:
        raise SelftestFailure("改主工作区的也停：" + "；".join(problems), world.server_log_lines())
    return "改主工作区的满上限照停，wait 照 workspace-write-interrupted 提醒核现场"


def cell_run_hard_limit_shard_stopped(world):
    """分片的（task_server_lib.py 文件头「分片」、task_server.py 文件头 ⑦「四小时硬上限」：按每一片自己的累计算）：上限压到 15 秒，第 0、1 片一直在跑、第 2 片排着
    （只有第二台放得下它、第二台用不了）。第 0、1 片各自累计满界：各记 over-four-hours；排着的第 2 片不撤，两片停了之后本机换成 50G 的余量，它照起、跑完（succeeded）；
    收齐之后整件照没成的那一片收尾：over-four-hours、退 229，不合并，那一片的说明写硬上限。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下各片结局不对，shard-whole-by-wall-clock 下整件按挂钟满界、排着的第 2 片被撤（cancelled），这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-shard")
    task_id, children = start_three_shards(world, "hard-limit", [{"block": release}, {"block": release}, {}], extra=hard_limit_settings(RUN_HARD_LIMIT_SHARD_SECONDS))
    problems = []
    try:
        stopped = world.wait_for(task_id, lambda found: all(shard_outcomes(found).get(index) for index in (0, 1)) or found["task"].get("state") == "finished",
                                 "第 0、1 片各自满界收尾", timeout=RELEASE_WAIT_SECONDS)
        if stopped["task"].get("state") != "finished":
            world.resources(local=(50, 50), peer_down="自证：第二台此刻用不了")   # 第 2 片（记过 30G）在本机放得下了
    finally:
        open(release, "w").close()
    summary = world.finished(task_id, timeout=RELEASE_WAIT_SECONDS)
    outcomes = shard_outcomes(summary)
    failed_note = str(shard_children(summary).get(0, {}).get("task", {}).get("note") or "") + str(shard_children(summary).get(1, {}).get("task", {}).get("note") or "")
    if (summary["task"].get("outcome") != lib.RUN_HARD_LIMIT_OUTCOME or summary["task"].get("exit_code") != RUN_HARD_LIMIT_EXIT
            or lib.RUN_HARD_LIMIT_OUTCOME not in str(summary["task"].get("note")) or RUN_HARD_LIMIT_NOTE not in failed_note):
        problems.append(f"整件应当照没成的那一片记 {lib.RUN_HARD_LIMIT_OUTCOME}、退 {RUN_HARD_LIMIT_EXIT}、说明点名那一片的结局，那一片的说明写{RUN_HARD_LIMIT_NOTE}；"
                        f"实际 {describe(summary)}、说明「{summary['task'].get('note')}」、第 0、1 片的说明「{failed_note[:300]}」")
    if outcomes != {0: lib.RUN_HARD_LIMIT_OUTCOME, 1: lib.RUN_HARD_LIMIT_OUTCOME, 2: "succeeded"}:
        problems.append(f"各片按自己的累计判：在跑满界的两片应当记 {lib.RUN_HARD_LIMIT_OUTCOME}、排着的第 2 片照起、跑完（succeeded），实际 {outcomes}")
    if not world.sharded_calls("shard", index=2) or "merge" in child_roles(summary) or world.sharded_calls("merge"):
        problems.append(f"第 2 片应当起过、不合并，实际第 2 片跑的记录 {world.sharded_calls('shard', index=2)}、子任务 {child_roles(summary)}")
    if problems:
        raise SelftestFailure("分片的到上限：" + "；".join(problems), world.server_log_lines())
    return "分片的满上限按每一片自己的累计：满界的两片停、记 over-four-hours，排着的那一片照起、跑完，整件照没成的那一片收尾、不合并"


def cell_run_hard_limit_applies_after_restart(world):
    """服务重启时已在跑的任务同样判：服务起来时上限够大，交一条一直等放行的；停了服务（尝试在它自己的单元里接着跑），换 1 秒的上限再起：判到、停、记 over-four-hours。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，no-detach 下服务一停命令也没了，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-restart")
    world.start_server(extra=hard_limit_settings(HARD_LIMIT_OUT_OF_REACH_SECONDS))
    task_id = world.submit(blocking_command(release))
    world.running(task_id)
    world.stop_server()
    world.start_server(extra=hard_limit_settings(1))
    problems = hard_limit_problems(held_until_finished(world, task_id, release), "重启时已在跑的")
    if problems:
        raise SelftestFailure("重启时已在跑的同样判：" + "；".join(problems), world.server_log_lines())
    return "服务重启时已在跑、累计过了上限的：照判、照停"


def cell_run_hard_limit_stop_retried(world):
    """判了停而头一回没停成的下一轮再停（task_server.py 的 stop_over_limit_attempt；TASK_SERVER_SELFTEST_STOP_FAILS 演一次）：照样记 over-four-hours，演停失败的文件用掉了。
    弄坏开关 hard-limit-stop-once 下不再停，run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-stop-retried")
    stop_fails = os.path.join(world.root, "stop-fails-hard-limit")
    open(stop_fails, "w").close()
    world.start_server(extra=hard_limit_settings(2, TASK_SERVER_SELFTEST_STOP_FAILS=stop_fails))
    task_id = world.submit(held_command(release, "tick"))
    problems = hard_limit_problems(held_until_finished(world, task_id, release), "头一回没停成的")
    if os.path.exists(stop_fails):
        problems.append("演停失败的文件应当用掉了：服务应当起过一回 stop_running")
    if problems:
        raise SelftestFailure("没停成的下一轮再停：" + "；".join(problems), world.server_log_lines())
    return "头一回没停成，下一轮再停：照样记 over-four-hours"


def signal_note_scheduler(stopped_attempt_ids=()):
    """signal-exit-named 的假调度器：stop_running 对哪几次尝试发过停（service_stopped_attempts）。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.service_stopped_attempts = {attempt_id: time.time() for attempt_id in stopped_attempt_ids}
    return fake


def cell_signal_exit_named(world):
    """命令被信号终止时原因句写出信号名、点名服务发没发过停的信号（task_server.py 的 signal_note、service_stop_step；task_server_lib.py 的 reason_fields；
    任务 13455 第 6 次的 SIGTERM，结局只写「命令自己退 1」）：起服务交一条 bash -c 'kill -TERM $$'：结局 command-failed，那一次尝试的 signal_note 与理由行
    的 why 都写「SIGTERM」与「服务这一次没有发过停的信号」。静态再核两种：尝试单元收到 SIGTERM（stopped_signal 15）、命令退 1，而 stop_running 对它发过停：
    写「SIGTERM」与「停尝试的单元」；判卡住停的写「判卡住」；退 1、没收到信号的交空串。弄坏开关 signal-exit-unnamed 下一句都不写，这一格红。"""
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "kill -TERM $$", "signal-self"]))
    attempt = first_attempt(summary)
    note = attempt.get("signal_note") or ""
    why = dict(lib.reason_fields(summary["task"], summary["attempts"], "")).get("why", "")
    problems = []
    if summary["task"].get("outcome") != "command-failed" or "SIGTERM" not in note or "服务这一次没有发过停的信号" not in note or note not in why:
        problems.append(f"自己收到 SIGTERM 的命令应当记 command-failed、signal_note 写 SIGTERM 与「服务这一次没有发过停的信号」、理由行接上它，"
                        f"实际 {describe(summary)}、signal_note「{note}」、why「{why}」")
    stopped = SERVER.Scheduler.signal_note(signal_note_scheduler([7]), {"id": 1}, {"id": 7}, {"stopped_signal": 15}, 1)
    stalled = SERVER.Scheduler.signal_note(signal_note_scheduler(), {"id": 1}, {"id": 8, "stalled": True}, {"stopped_signal": 15}, 1)
    plain = SERVER.Scheduler.signal_note(signal_note_scheduler(), {"id": 1}, {"id": 9}, {"stopped_signal": 0}, 1)
    if "SIGTERM" not in stopped or "停尝试的单元" not in stopped or "判卡住" not in stalled or plain:
        problems.append(f"单元收到 SIGTERM、stop_running 发过的应当写 SIGTERM 与「停尝试的单元」，判卡住的写「判卡住」，没收到信号的空串；"
                        f"实际「{stopped}」「{stalled}」「{plain}」")
    if problems:
        raise SelftestFailure("被信号终止的写出信号名：" + "；".join(problems), world.server_log_lines())
    return f"自己收到 SIGTERM 的写「{note}」；服务停的点名是哪一步"


EXTERNAL_KILL_COMMAND = 'echo $$ > "$1"; sleep 2; if [[ -e "$2" ]]; then exit 0; fi; : > "$2"; exit 1'   # 头一次把自己的进程号写给假 journalctl 当 earlyoom 杀掉的那个、退 1；第二次退 0


def external_kill_scheduler(work, log_pid):
    """external-kill-recorded-and-retried 的假调度器：本机的尝试进程树里见过 4242、4300；假 journalctl（在 work 下）头一路（-t earlyoom）打一行点名 log_pid 的 earlyoom 日志、-k 不打。"""
    journalctl = os.path.join(work, "fake-journalctl-unit")
    write_executable(journalctl, f'#!/usr/bin/env bash\n[[ " $* " == *" -t earlyoom "* ]] && echo \'sending SIGTERM to process {log_pid} uid 1000 "rustc": badness 1414, VmRSS 9708 MiB\'\nexit 0\n')
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ, TASK_SERVER_JOURNALCTL=journalctl)
    fake.journalctl = journalctl
    fake.service_stopped_attempts = {}
    fake.peer_external_kill, fake.log_content = {}, lambda attempt: 0   # 第二台的尝试问 peer_external_kill_note：这个假世界里第二台的日志里没有那一行
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(process_ids_seen=lambda attempt, result: {4242, 4300}),
                     SERVER.PEER: types.SimpleNamespace(process_ids_seen=lambda attempt, result: None)}
    return fake


def external_kill_unit_problems(world):
    """external-kill-recorded-and-retried 的假调度器那一半：日志点名的进程在这一次尝试的进程树里才记；点名别的进程、服务发过停的信号、结局不是 command-failed、
    单元收到过停的信号、第二台的尝试（进程号读不到）都不记；retry_decision 对 killed-externally 同一上限重排两次、第三次起打回。"""
    problems = []
    now = time.time()
    task, attempt, result = {}, {"id": 1, "host": SERVER.LOCAL, "started_at": now - 10}, {"ended_at": now, "stopped_signal": 0}
    fake = external_kill_scheduler(world.root, 4242)
    note = SERVER.Scheduler.external_kill_note(fake, task, attempt, result, "command-failed")
    if "earlyoom" not in note or "4242" not in note:
        problems.append(f"日志点名进程 4242、它在进程树里：应当记、说明写日志来源与进程号，实际「{note}」")
    others = (("日志点名的进程不在进程树里", external_kill_scheduler(world.root, 5555), task, attempt, result, "command-failed"),
              ("服务发过停的信号", fake, task, dict(attempt, stalled=True), result, "command-failed"),
              ("结局不是 command-failed", fake, task, attempt, result, "memory-cap-hit"),
              ("单元收到过停的信号", fake, task, attempt, dict(result, stopped_signal=15), "command-failed"),
              ("第二台的尝试", fake, task, dict(attempt, host=SERVER.PEER), result, "command-failed"))
    for what, scheduler, found_task, found_attempt, found_result, outcome in others:
        found = SERVER.Scheduler.external_kill_note(scheduler, found_task, found_attempt, found_result, outcome)
        if found:
            problems.append(f"{what}：不该记 killed-externally，实际「{found}」")
    for counts, retried in (({lib.EXTERNAL_KILL_OUTCOME: 1}, True), ({lib.EXTERNAL_KILL_OUTCOME: lib.EXTERNAL_KILL_RETRIES}, True), ({lib.EXTERNAL_KILL_OUTCOME: lib.EXTERNAL_KILL_RETRIES + 1}, False)):
        decision = lib.retry_decision(counts, lib.EXTERNAL_KILL_OUTCOME, 2 * GIBIBYTE, SERVER.LOCAL, "", {})
        if decision.retry != retried or (retried and decision.cap_bytes != 2 * GIBIBYTE):
            problems.append(f"killed-externally 出过 {counts[lib.EXTERNAL_KILL_OUTCOME]} 次：应当{'同一上限重排' if retried else '打回'}，实际 retry={decision.retry}、上限 {decision.cap_bytes}")
    return problems


def cell_external_kill_recorded_and_retried(world):
    """命令被不是服务发的信号杀掉、earlyoom 或内核日志里有它的进程号：记 killed-externally 并重排（task_server.py 的 external_kill_note、kill_log_lines，task_server_lib.py 的 retry_decision；
    attempt_runner.py 每轮采样把进程树里见过的进程号记进 result.json 的 process_ids）。假调度器那一半（external_kill_unit_problems）：判法的各支。
    服务那一半：假 journalctl（只在世界目录的 earlyoom-pid 在时才打一行点名它的 earlyoom 日志）；一条命令把自己的进程号写进 earlyoom-pid、睡 2 秒后头一次退 1、第二次退 0：
    头一次尝试记 killed-externally（尝试记 external_kill_note）、第二次跑成，共两次；再一条睡 2 秒退 1、进程号不在日志里的：记 command-failed、不重排，一次尝试。
    弄坏开关 external-kill-unrecognized 下头一条照退出码记 command-failed、不重排，这一格红。"""
    problems = external_kill_unit_problems(world)
    raise_cell_problems(world, problems, "被外部信号杀掉（假调度器）")
    world.start_server(extra={"TASK_SERVER_JOURNALCTL": os.path.join(world.bin, "fake-journalctl-earlyoom")})
    pid_file = os.path.join(world.world, "earlyoom-pid")
    flag = os.path.join(world.root, "external-kill-flag")
    killed = world.submit(["bash", "-c", EXTERNAL_KILL_COMMAND, "external-kill", pid_file, flag])
    killed_summary = world.finished(killed)
    unlogged = world.submit(["bash", "-c", "sleep 2; exit 1", "external-kill-unlogged"])
    unlogged_summary = world.finished(unlogged)
    outcomes = [attempt.get("outcome") for attempt in killed_summary["attempts"]]
    first_note = (killed_summary["attempts"] or [{}])[0].get("external_kill_note") or ""
    if outcomes != [lib.EXTERNAL_KILL_OUTCOME, "succeeded"] or killed_summary["task"].get("outcome") != "succeeded" or "earlyoom" not in first_note:
        problems.append(f"日志点名了这一次尝试的进程：头一次应当记 {lib.EXTERNAL_KILL_OUTCOME}（尝试记 external_kill_note 写 earlyoom）、重排一次、第二次跑成，"
                        f"实际各次 {outcomes}、说明「{first_note}」、{describe(killed_summary)}")
    unlogged_outcomes = [attempt.get("outcome") for attempt in unlogged_summary["attempts"]]
    if unlogged_outcomes != ["command-failed"] or unlogged_summary["task"].get("exit_code") != 1:
        problems.append(f"日志里没有这一次尝试的进程号：应当仍记 command-failed、退 1、不重排，实际各次 {unlogged_outcomes}、{describe(unlogged_summary)}")
    raise_cell_problems(world, problems, "被外部信号杀掉（服务）")
    return f"日志点名的进程在进程树里：记 {lib.EXTERNAL_KILL_OUTCOME}、重排一次跑成；日志里没有的仍记 command-failed、不重排；假调度器上各支（服务发过停的、第二台的、单元收到停的）都不记，重排至多 {lib.EXTERNAL_KILL_RETRIES} 次"


PEER_KILL_LINE = ('MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=4242 line=sending SIGTERM to process 4242 uid 1000 "rustc": badness 1414, VmRSS 9708 MiB')   # external-kill-peer-recorded：第二台那一行
# 那一格内存组到了它自己上限的那一行（内核「Memory cgroup out of memory」：命令自己造的小内存组让进程撞上限，不是整机内存紧张）与整机 OOM 那一行（本机的认法对照）
PEER_CGROUP_KILL_LINE = ("MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=kernel pid=4243 line=Memory cgroup out of memory: Killed process 4243 (python3) total-vm:812345kB, "
                         "anon-rss:65536kB, file-rss:0kB, shmem-rss:0kB, UID:1000 pgtables:400kB oom_score_adj:500")
MACHINE_OOM_KILL_LINE = "Out of memory: Killed process 4244 (rustc) total-vm:9812345kB, anon-rss:9708000kB, file-rss:0kB, shmem-rss:0kB, UID:1000 pgtables:20000kB oom_score_adj:0"


def peer_external_kill_scheduler(world, name, lines):
    """external-kill-peer-recorded 的假调度器：第二台的一次尝试（id 7），日志是 lines 写成的文件，经真的 log_content 记表（peer_external_kill），再问 external_kill_note。"""
    log = os.path.join(world.root, f"peer-external-kill-{name}.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.service_stopped_attempts = {}
    for field in ("log_offsets", "peer_memory", "peer_memory_stat", "attempt_progress", "attempt_progress_baseline", "peer_io_wait",
                  "peer_external_kill", "peer_io_rounds", "peer_thread_rounds", "gpu_not_run_marks"):
        setattr(fake, field, {})
    fake.machines = {SERVER.PEER: SERVER.PeerMachine(fake)}
    return fake, {"id": 7, "task_id": 77, "number": 1, "host": SERVER.PEER, "label": f"task-selftest-peer-kill-{name}", "log": log, "started_at": time.time() - 10}


def cell_external_kill_peer_line_recorded_and_retried(world):
    """被外部信号杀掉的尝试第二台也认，同本机一套判法（task_server.py 的 log_content、peer_external_kill_note、external_kill_note；scripts/hosts/multi-host-run.sh 在第二台那一侧记下这一趟进程树
    的进程号、命令退非 0 之后读第二台的 earlyoom 与内核日志、点名这些进程号之一被杀时打一行 MULTI_HOST_RUN_PEER_EXTERNAL_KILL）：第二台的尝试日志里有这一行、结局 command-failed、
    服务没发过停的信号：记 killed-externally（说明写第二台、来源、被杀的进程号），retry_decision 对第二台同一上限重排；日志里没有这一行的、结局不是 command-failed 的、
    服务发过停的信号的都不记；那一行是内核「Memory cgroup out of memory」（某个内存组到了它自己的上限）的不记，本机的认法（lib.external_kill_pid）同样不交它的进程号、
    整机 OOM 的「Out of memory: Killed process」照交。假调度器上跑，不起服务。
    弄坏开关 peer-external-kill-line-ignored、external-kill-unrecognized 下有这一行的也不记，memory-cgroup-kill-counted 下内存组那一行也记，这一格红。"""
    problems = []
    task, result = {}, {"stopped_signal": 0}
    fake, attempt = peer_external_kill_scheduler(world, "line", ["第二台那条命令的输出", PEER_KILL_LINE, "第二台那条命令退 143"])
    note = SERVER.Scheduler.external_kill_note(fake, task, attempt, result, "command-failed")
    if "第二台" not in note or "earlyoom" not in note or "4242" not in note:
        problems.append(f"第二台日志里有 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行：应当记 killed-externally、说明写第二台、来源与进程号 4242，实际「{note}」")
    unlogged_fake, unlogged_attempt = peer_external_kill_scheduler(world, "none", ["第二台那条命令的输出"])   # 没有那一行的尝试用它自己的日志文件，不借有那一行的那份
    for what, scheduler, found_attempt, outcome in (("日志里没有那一行", unlogged_fake, unlogged_attempt, "command-failed"),
                                                     ("结局不是 command-failed", fake, attempt, "memory-cap-hit"),
                                                     ("服务发过停的信号", fake, dict(attempt, stalled=True), "command-failed")):
        found = SERVER.Scheduler.external_kill_note(scheduler, task, found_attempt, result, outcome)
        if found:
            problems.append(f"{what}：不该记 killed-externally，实际「{found}」")
    cgroup_fake, cgroup_attempt = peer_external_kill_scheduler(world, "cgroup", ["第二台那条命令的输出", PEER_CGROUP_KILL_LINE, "第二台那条命令退 1"])
    cgroup_note = SERVER.Scheduler.external_kill_note(cgroup_fake, task, cgroup_attempt, result, "command-failed")
    if cgroup_note:
        problems.append(f"那一行是 Memory cgroup out of memory（内存组到了它自己的上限）：不该记 killed-externally，实际「{cgroup_note}」")
    local_pids = (lib.external_kill_pid(PEER_CGROUP_KILL_LINE.split("line=", 1)[1]), lib.external_kill_pid(MACHINE_OOM_KILL_LINE))
    if local_pids != (None, 4244):
        problems.append(f"本机的认法：Memory cgroup out of memory 那一行应当不交进程号、整机 Out of memory 那一行交 4244，实际 {local_pids}")
    decision = lib.retry_decision({lib.EXTERNAL_KILL_OUTCOME: 1}, lib.EXTERNAL_KILL_OUTCOME, 2 * GIBIBYTE, SERVER.PEER, "", {})
    if not decision.retry or decision.cap_bytes != 2 * GIBIBYTE:
        problems.append(f"第二台的 killed-externally 出过 1 次：应当同一上限重排，实际 retry={decision.retry}、上限 {decision.cap_bytes}")
    if problems:
        raise SelftestFailure("第二台被外部信号杀掉：" + "；".join(problems))
    return (f"第二台日志里有 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行：记 {lib.EXTERNAL_KILL_OUTCOME}、同一上限重排；没有那一行、结局不是 command-failed、服务发过停的信号的、"
            "那一行是 Memory cgroup out of memory 的都不记；本机的认法同样不认内存组那一行、认整机 OOM 那一行")
