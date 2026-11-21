"""task_server_selftest.py 拆出的一份：要处置的提醒项、异常表、提醒的发与不重发、给会话发消息、服务健康、GPU 服务的读数的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import io
import json
import re
import shlex
import subprocess
import threading
import time
import types
import unittest.mock
import task_server_lib as lib
from task_server_selftest_fakes import (
    BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, LOCAL_CARD_ZERO, PLACEMENT_WAIT_SECONDS, POLL_SECONDS, SERVER, SESSION_A, SESSION_B,
    STATE_TIMEOUT_SECONDS,
)
from task_server_selftest_world import (
    ATTENTION_QUIET_SECONDS, ATTENTION_WAIT_SECONDS, FAKE_MERGE_ENTRY, FILLS_CAP_SEEDED_CAP_BYTES, FILLS_CAP_SEEDED_PEAK_BYTES, FakeInbox,
    INBOX_QUIET_SECONDS, INBOX_TOKEN_SAMPLE, INBOX_WAIT_SECONDS, LONG_RUN_WHY_SAMPLE, REPUSH_SAMPLE_SECONDS, STARVED_HEALTH_TEXT, SelftestFailure,
    World, append_lines, attention_problems, describe, health_until, inbox_problems, long_run_split_scheduler, outcome_head, peer_log_scheduler,
    queued_shards, reminder_head, reminders_after, restart_surviving_scheduler, shard_children, start_burning_scope, start_inbox_server,
    systemctl_start_count, table_advice, wait_path, write_fake_entry,
)


def cell_unclaimed_finished(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    world.finished(task_id)
    completed = world.client(["status", "--json"], session=SESSION_A)
    listed = [item for item in json.loads(completed.stdout or "{}").get("tasks", []) if item.get("id") == task_id and item.get("unclaimed")]
    if not listed:
        raise SelftestFailure(f"完成了没人取的任务 {task_id} 应当在这个会话的 status 里列着，实际 {completed.stdout.strip()[:300]}")
    world.client(["wait", str(task_id)], session=SESSION_A)
    completed = world.client(["status", "--json"], session=SESSION_A)
    if any(item.get("id") == task_id for item in json.loads(completed.stdout or "{}").get("tasks", [])):
        raise SelftestFailure(f"wait 取过之后任务 {task_id} 不该再列成没人取，实际 {completed.stdout.strip()[:300]}")
    return "完成了没人取的列在 status 里，wait 取过之后不再列"


HEALTH_SETTLE_SECONDS = 2.0   # 服务健康的毛病撤掉之后等这么久（四轮），让服务看到好了、忘掉发过的那一样
INBOX_COLLECTED_SECONDS = 10.0   # 结局那一条收到之后等服务记成已取的安静时限
UNCLAIMED_WARNING_SAMPLE_SECONDS = 1   # unclaimed-results-warned：结局满这么多秒没人取就警告（TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS）


def server_output_text(world):
    """这一格各次服务的输出整份（server-<n>.log）。"""
    texts = []
    for number in range(1, world.server_logs + 1):
        with contextlib.suppress(OSError):
            with open(os.path.join(world.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
    return "\n".join(texts)


def unclaimed_head(task_id, outcome):
    """结局没人取而再推的那一条首行到「（<种类>」为止的一段：状态里另写这条结局还没人取。"""
    return f"{SERVER.INBOX_HEADLINE}结局（任务已结束，这条结局还没人取）任务 {task_id} {SERVER.kind_label(outcome)}（{outcome}"


def inbox_outcome_case(world, script, outcome, exit_text):
    """一条任务有了结局（task_server.py 文件头 ⑥「结局」）：会话的收件 socket 恰好收到一条，先认证行、再 user 那一行，正文第一行「task-server：结局（任务已结束）任务 <号> <中文名>（<结局>，退 <码>；第 1 回）」，
    后面写原因、出路、日志与 wait 的末行原样；令牌不在 status、日志与服务的输出里。→ (问题表, 任务号, 正文)。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world)
    task_id = world.submit(["bash", "-c", script], extra=inbox.environment())
    world.finished(task_id)
    head = outcome_head(task_id, outcome)
    found = inbox.wait_for(world, head)
    time.sleep(INBOX_QUIET_SECONDS)   # 结局只发一次：再看几轮不该多一条
    found = inbox.headed(head)
    problems = inbox_problems(found, head, f"结局 {outcome}")
    text = found[0]["text"] if found else ""
    for wanted in ("原因：", "出路：", "日志：", f"{lib.RESULT_MARK} task={task_id} type={outcome} {exit_text}"):
        if wanted not in text:
            problems.append(f"结局那一条的正文里应当有「{wanted}」，实际「{text[:400]}」")
    shown = world.client(["status", str(task_id), "--json"]).stdout + world.client(["log", str(task_id)]).stdout + server_output_text(world)
    if inbox.token in shown:
        problems.append("会话令牌不该出现在 status、task log 与服务的输出里，实际出现了")
    return problems, task_id, text


def cell_inbox_outcome_succeeded(world):
    """会成功的一条（task_server.py 文件头 ⑥）：收件 socket 收到一条结局 succeeded（inbox_outcome_case 的判据）；发出去了服务记成已取，status 不再列成没人取。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、pushed-outcome-not-collected 下这一格红。"""
    problems, task_id, _ = inbox_outcome_case(world, "echo inbox-done; exit 0", "succeeded", "exit=0")
    patience = world.patience(INBOX_COLLECTED_SECONDS)
    listed = True
    while patience.waiting() and listed:
        listed = any(item.get("id") == task_id for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", []))
        time.sleep(POLL_SECONDS)
    if listed:
        problems.append(f"结局发出去之后任务 {task_id} 应当记成已取、status 不再列成完成了没人取，实际还列着")
    if problems:
        raise SelftestFailure("结局 succeeded 发给会话：" + "；".join(problems), world.server_log_lines())
    return f"收件 socket 收到一条「{outcome_head(task_id, 'succeeded')}…」（认证行在前、令牌对得上），发出去之后记成已取，令牌不进 status 与日志"


def cell_inbox_outcome_failed(world):
    """会失败的一条（命令退 3）：收件 socket 收到一条结局 command-failed（inbox_outcome_case 的判据），末尾 wait 的末行 exit=3。
    弄坏开关 inbox-not-sent、inbox-auth-omitted 下这一格红。"""
    problems, task_id, _ = inbox_outcome_case(world, "echo inbox-failed; exit 3", "command-failed", "exit=3")
    if problems:
        raise SelftestFailure("结局 command-failed 发给会话：" + "；".join(problems), world.server_log_lines())
    return f"收件 socket 收到一条「{outcome_head(task_id, 'command-failed')}…」（认证行在前、令牌对得上），末尾是 exit=3 的末行"


def cell_inbox_queued_too_long_once(world):
    """排队过久（task_server.py 文件头 ⑥「要处置的提醒」，同 attention-reported-once 的世界：第二台用不了、只有它放得下）：排满 6 秒收件 socket 收到一条
    「task-server：提醒（任务还在排，不是结局）任务 <号> 排队过久（queued-too-long；第 1 回」，正文写依据（已排多久、排满 12 秒再提醒一回）、原因、出路、日志与
    TASK_SERVER_ATTENTION 一行（kind=queued-too-long 之后紧接 reminded_after_seconds=6）；再看几轮不多一条；排满 12 秒再收一条 reminded_after_seconds=12 的。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、inbox-reported-every-round 下这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-inbox-queue", 30 * GIBIBYTE)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(["bash", "-c", "exit 0", "inbox-queue"], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: "selftest-inbox-queue"}), cwd=world.repo)
    head, later = reminder_head(task_id, "queued-too-long", "任务还在排"), 2 * ATTENTION_ONCE_SECONDS
    inbox.wait_for(world, head)
    time.sleep(ATTENTION_ONCE_QUIET_SECONDS)
    first = reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS)
    problems = inbox_problems(first, head, f"排满 {ATTENTION_ONCE_SECONDS} 秒那一回")
    text = first[0]["text"] if first else ""
    for wanted in ("依据：已排", f"排满 {later} 秒再提醒一回", "原因：", "出路：",
                   f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=queued-too-long reminded_after_seconds={ATTENTION_ONCE_SECONDS} basis="):
        if wanted not in text:
            problems.append(f"排队过久那一条的正文里应当有「{wanted}」，实际「{text[:400]}」")
    if "档" in text.split("\n", 1)[0] or any("档" in line for line in text.split("\n") if line.startswith("依据：")):
        problems.append(f"首行与依据只写事实与下一回什么时候提醒，不该有「档」，实际「{text[:400]}」")
    inbox.wait_for(world, head, count=2)
    time.sleep(INBOX_QUIET_SECONDS)
    problems += inbox_problems(reminders_after(inbox.headed(head), later), head, f"排满 {later} 秒那一回")
    if len(reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS)) != 1:
        problems.append(f"排满 {later} 秒之后，{ATTENTION_ONCE_SECONDS} 秒那一条仍该只有一条，实际 {len(reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS))} 条")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("排队过久发给会话：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_ONCE_SECONDS} 秒、{later} 秒各收到恰好一条（认证行在前），同一道门槛不重发"


INBOX_SHARD_KEY = "selftest-inbox-shard-queue"   # inbox-shard-attention-merged：三片分片的命令键，三片各记过 30G（只有第二台放得下，第二台用不了，三片一齐排着）
INBOX_SHARD_ATTENTION_SECONDS = 600              # inbox-shard-attention-merged：排队过久的头一道门槛（TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS）：自然排不满，靠改提交时刻一次跨过
INBOX_SHARD_BACKDATE_MARGIN_SECONDS = 5          # inbox-shard-attention-merged：三片的提交时刻往前拨过门槛再多这么久（重起服务到头一轮排队之间不会掉回门槛之下）


def cell_inbox_shard_attention_merged(world):
    """分片的几片同时排久（task_server.py 文件头 ⑥「要处置的提醒」的合成一条，Scheduler.attention_groups）：三片各记过 30G、本机 slice 20G、第二台用不了，
    三片一齐排着；三片同时跨过排队过久的门槛，收件 socket 只收到一条「task-server：提醒（任务还在排，不是结局）任务 <父任务号> 排队过久（queued-too-long，子任务 <号的区间>，共 3 片；第 1 回）」，
    第一行点到三片的号；再看几轮不多一条。三片各记各的提交时刻，机器忙时库写得慢、几片进队相差好几秒，自然排满门槛的时刻也相差好几秒、各落在不同的一轮里，合不成一条：
    所以三片排上之后停服务，把三片的提交时刻改成同一个、都已排过门槛（INBOX_SHARD_ATTENTION_SECONDS，大得自然排不到下一道），再起服务，三片在同一轮跨过门槛。
    弄坏开关 inbox-attention-per-shard 下逐片各发一条（收到三条），
    inbox-headline-state-omitted 下首行不写「提醒（任务还在排，不是结局）」、照首行认不到那一条（收到 0 条），这一格红。"""
    inbox = FakeInbox(world, "a")
    for index in range(3):
        world.seed_peak(lib.shard_child_key(INBOX_SHARD_KEY, "shard", index, 3), 30 * GIBIBYTE, shard_count=3)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    server_extra = {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(INBOX_SHARD_ATTENTION_SECONDS)}
    start_inbox_server(world, server_extra)
    task_id = world.submit(world.write_spec("spec-inbox-shards.json", {"shards": [{}, {}, {}]}), cwd=world.repo,
                           extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: INBOX_SHARD_KEY}))
    summary = world.wait_for(task_id, lambda summary: queued_shards(summary) == [0, 1, 2], "三片都排着")
    children = sorted(child["task"]["id"] for child in shard_children(summary).values())
    world.stop_server()
    submitted_at = time.time() - INBOX_SHARD_ATTENTION_SECONDS - INBOX_SHARD_BACKDATE_MARGIN_SECONDS
    for child_id in children:
        world.rewrite_signed_task(child_id, lambda record: record.update(submitted_at=submitted_at))
    start_inbox_server(world, server_extra)
    head = reminder_head(task_id, "queued-too-long", "任务还在排")
    inbox.wait_for(world, head)
    time.sleep(INBOX_QUIET_SECONDS)
    messages = reminders_after(inbox.headed(head), INBOX_SHARD_ATTENTION_SECONDS)
    problems = inbox_problems(messages, head, "三片同时排久")
    headline = messages[0]["text"].split("\n", 1)[0] if messages else ""
    children_range = f"子任务 {children[0]}–{children[-1]}" if children == list(range(children[0], children[0] + len(children))) else None
    if messages and (children_range is None or children_range not in headline or "共 3 片" not in headline):
        problems.append(f"合成的那一条第一行应当把三片的号 {children} 写成区间「{children_range}」并带「共 3 片」，实际「{headline}」")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("分片几片同时排久合成一条发给会话：" + "；".join(problems)
                              + f"（收到的第一行：{[message['text'].split(chr(10), 1)[0] for message in inbox.headed(head)]}）", world.server_log_lines())
    return f"三片同时跨过 {INBOX_SHARD_ATTENTION_SECONDS} 秒的门槛，会话只收到一条：「{headline}」"


def cell_inbox_unreachable_task_runs(world):
    """收件 socket 不在（会话已关、会话登记里也没有它）：任务照跑到 succeeded，服务写一条告警 inbox-unreachable、那条任务的日志里有 lib.INBOX_LOG_MARK 起头的一行，令牌不进告警与日志；
    同一个场景的另一头：会话环境里没有那两个变量的提交客户端当场拒（退 3，异常表 inbox-missing：出路写从 Claude Code 会话的 Bash 里交），不交给服务、不进库。
    弄坏开关 inbox-not-sent 下不写告警、inbox-missing-accepted 下没有收件地址的也照交，这一格红。"""
    start_inbox_server(world)
    absent = os.path.join(world.short_root, "inbox-absent.sock")
    task_id = world.submit(["bash", "-c", "echo inbox-unreachable; exit 0"], extra={lib.INBOX_SOCKET_VARIABLE: absent, lib.INBOX_TOKEN_VARIABLE: INBOX_TOKEN_SAMPLE})
    done = world.finished(task_id)
    problems = [] if done["task"].get("outcome") == "succeeded" else [f"收件 socket 不在时任务照跑，应当 succeeded，实际 {describe(done)}"]
    patience = world.patience(INBOX_WAIT_SECONDS)
    alerted = []
    while patience.waiting() and not alerted:
        alerted = [alert for alert in world.alerts() if alert.get("kind") == "inbox-unreachable" and alert.get("task") == task_id]
        time.sleep(POLL_SECONDS)
    if not alerted:
        problems.append(f"发不出去应当写一条 inbox-unreachable 告警、点名任务 {task_id}，实际告警 {world.alerts()}")
    log = world.client(["log", str(task_id), "--all"]).stdout
    if lib.INBOX_LOG_MARK not in log:
        problems.append(f"发不出去应当往任务的日志里接一行 {lib.INBOX_LOG_MARK} 起头的告警，实际日志「{log.strip()[-300:]}」")
    if INBOX_TOKEN_SAMPLE in log or INBOX_TOKEN_SAMPLE in json.dumps(world.alerts(), ensure_ascii=False):
        problems.append("会话令牌不该出现在告警与任务日志里，实际出现了")
    before = len(world.signed_rows("tasks"))
    completed = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "inbox-absent"],
                             extra={lib.INBOX_SOCKET_VARIABLE: "", lib.INBOX_TOKEN_VARIABLE: ""})
    wanted = lib.anomaly_advice("inbox-missing", "")
    if (completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or "读不到会话的收件地址" not in completed.stderr or wanted not in completed.stderr
            or len(world.signed_rows("tasks")) != before):
        problems.append(f"没有收件地址的提交应当当场拒（退 3、不打任务号、不进库）、标准错误打「读不到会话的收件地址」与出路「{wanted}」，"
                        f"实际退 {completed.returncode}、库里任务 {before} → {len(world.signed_rows('tasks'))} 条、「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("收件 socket 不在：" + "；".join(problems), world.server_log_lines())
    return "收件 socket 不在时任务照跑到 succeeded、写了 inbox-unreachable 告警与日志里一行，令牌不进告警与日志；没有收件地址的提交客户端当场拒、不进库"


def health_notified_once(world, label, head, arm, disarm, extra=None):
    """服务健康的一样（task_server.py 文件头 ⑥「服务健康」）：一个假会话名下有一条在跑的任务；arm 造出毛病，收件 socket 恰好收到一条第一行「task-server：服务健康 <名字>」的
    （认证行在前、令牌对得上），再看几轮不多一条；disarm 撤掉、任务放行。→ 那一条的正文；判不过抛 SelftestFailure。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-health")
    start_inbox_server(world, extra)
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "health", release], extra=inbox.environment())
        world.running(task_id)
        arm()
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        found = inbox.headed(head)
    finally:
        disarm()
        open(release, "w").close()
    problems = inbox_problems(found, head, label)
    text = found[0]["text"] if found else ""
    if found and ("问题：" not in text or "出路：" not in text or "control check" not in text):
        problems.append(f"服务健康那一条应当写「问题：」与「出路：」（出路点名 control check），实际「{text[:400]}」")
    if problems:
        raise SelftestFailure(f"服务健康（{label}）：" + "；".join(problems), world.server_log_lines())
    return text


def write_unit_state(world, state):
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write(f"{state}\n")


def cell_health_unit_missing_notified(world):
    """服务自己的单元不在（假 systemctl 的 is-active 打 inactive）：发一条「task-server：服务健康 单元不在」（health_notified_once 的判据）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    text = health_notified_once(world, "单元不在", "task-server：服务健康 单元不在", lambda: write_unit_state(world, "inactive"), lambda: write_unit_state(world, "active"))
    if "is-active" not in text:
        raise SelftestFailure(f"单元不在那一条应当写 is-active 打的状态，实际「{text[:300]}」", world.server_log_lines())
    return "单元不在：收件 socket 收到恰好一条服务健康，写了 is-active 打的状态"


def cell_health_nfs_notified(world):
    """NFS 共享前三项不过（假 ssh 的 mountpoint 退 1，nfs_loop 复核判出「挂载」不过）：发一条「task-server：服务健康 NFS 共享判不过」，点名「挂载」那一项。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round、nfs-unchecked 下这一格红。"""
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")

    def arm():
        with open(mount_exit, "w", encoding="utf-8") as handle:
            handle.write("1\n")
    text = health_notified_once(world, "NFS 共享", "task-server：服务健康 NFS 共享判不过", arm, lambda: os.remove(mount_exit))
    if "「挂载」" not in text:
        raise SelftestFailure(f"NFS 那一条应当点名「挂载」那一项不过，实际「{text[:300]}」", world.server_log_lines())
    return "第二台没挂上共享根：收件 socket 收到恰好一条服务健康，点名「挂载」"


def cell_health_peer_unavailable_notified(world):
    """第二台用不了满时限（假余量的 peer_down；自证里 1 秒）：发一条「task-server：服务健康 第二台用不了」，写着那一句原因。任务放在本机（第二台放不下）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64)
    why = "自证：第二台连不上"
    text = health_notified_once(world, "第二台用不了", "task-server：服务健康 第二台用不了",
                                lambda: world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64, peer_down=why),
                                lambda: world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64))
    if why not in text:
        raise SelftestFailure(f"第二台用不了那一条应当写着原因「{why}」，实际「{text[:300]}」", world.server_log_lines())
    return "第二台用不了满时限：收件 socket 收到恰好一条服务健康，写着原因"


GPU_CARD_FAILED_SELFTEST_LINE = ('GPU_VERIFICATION_SERVER_CARD_FAILED card=0 nvidia_smi_index=0 name="Selftest Card" pci=0000:01:00.0 device_lost=yes '
                                 'reason="wgpu error: device lost"')


def cell_health_gpu_card_failed_notified(world):
    """GPU 服务报了卡坏（假的 control cards-failed：TASK_SERVER_SELFTEST_GPU_CARDS_FAILED 指的脚本，arm 那份文件在时多打一行卡坏，
    另打一行只读状态的不算）：发一条「task-server：服务健康 GPU 服务卡坏」，写着那张卡的 PCI 总线号与重置那张卡的出路；告警记录里恰好一条 gpu-card-failed。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round、nfs-unchecked、gpu-card-failure-unwatched 下这一格红。"""
    arm = os.path.join(world.root, "gpu-card-failed-arm")
    fake = os.path.join(world.root, "fake-gpu-cards-failed")
    failed_line = f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=4242 {GPU_CARD_FAILED_SELFTEST_LINE}"
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\n"
                     f"echo 'GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={lib.GPU_THIS_HOST_LABEL} state=read lines=1'\n"
                     f"[ -e {shlex.quote(arm)} ] && echo {shlex.quote(failed_line)}\n"
                     "exit 0\n")
    os.chmod(fake, 0o755)

    def arm_it():
        open(arm, "w").close()
    text = health_notified_once(world, "GPU 服务卡坏", "task-server：服务健康 GPU 服务卡坏", arm_it, lambda: os.remove(arm),
                                extra={"TASK_SERVER_SELFTEST_GPU_CARDS_FAILED": fake})
    problems = []
    if "pci=0000:01:00.0" not in text or "重置" not in text or "gpu-verification-server/control check" not in text:
        problems.append(f"GPU 服务卡坏那一条应当写着那张卡的 PCI 总线号、重置那张卡与 gpu-verification-server/control check，实际「{text[:400]}」")
    alerted = [alert for alert in world.alerts() if alert.get("kind") == lib.GPU_CARD_FAILED_ALERT_KIND]
    if len(alerted) != 1 or "pci=0000:01:00.0" not in str(alerted[0].get("line")):
        problems.append(f"告警记录里应当恰好一条 {lib.GPU_CARD_FAILED_ALERT_KIND}、写着那张卡，实际 {alerted}")
    if problems:
        raise SelftestFailure("服务健康（GPU 服务卡坏）：" + "；".join(problems), world.server_log_lines())
    return "GPU 服务报了卡坏：收件 socket 收到恰好一条服务健康、写着 PCI 与重置的出路，告警记录恰好一条 gpu-card-failed"


GPU_BLOCK_NOT_JUDGEABLE_SELFTEST_LINE = ('GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE family=image-judgement stream=0a0b0c0d0e0f1011 block=12 card=0 '
                                         'why=judge-panicked claim=released reason="judge assertion"')
GPU_PEER_SELFTEST_HOST = "192.0.2.7"   # 自证里第二台在 cards-failed 里写的台名（文档段地址，哪一台都没有）


def cell_health_gpu_block_not_judgeable_notified(world):
    """GPU 服务报了判不了的块（假的 control cards-failed：arm 那份文件在时多打一行判不了）：发一条「task-server：服务健康 GPU 服务判不了的块」，
    写着那一块的族、流键头、块号与「放给另一台、不要重置卡」的出路；告警记录里恰好一条 gpu-block-not-judgeable。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round、gpu-card-failure-unwatched、gpu-block-not-judgeable-unwatched 下这一格红。"""
    arm = os.path.join(world.root, "gpu-block-not-judgeable-arm")
    fake = os.path.join(world.root, "fake-gpu-cards-failed")
    not_judgeable_line = f"{lib.GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=4242 {GPU_BLOCK_NOT_JUDGEABLE_SELFTEST_LINE}"
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\n"
                     f"echo 'GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={lib.GPU_THIS_HOST_LABEL} state=read lines=1'\n"
                     f"[ -e {shlex.quote(arm)} ] && echo {shlex.quote(not_judgeable_line)}\n"
                     "exit 0\n")
    os.chmod(fake, 0o755)

    def arm_it():
        open(arm, "w").close()
    text = health_notified_once(world, "GPU 服务判不了的块", "task-server：服务健康 GPU 服务判不了的块", arm_it, lambda: os.remove(arm),
                                extra={"TASK_SERVER_SELFTEST_GPU_CARDS_FAILED": fake})
    problems = []
    if "family=image-judgement" not in text or "stream=0a0b0c0d0e0f1011" not in text or "block=12" not in text or "另一台" not in text or "不要重置卡" not in text:
        problems.append(f"判不了那一条应当写着族、流键头、块号与放给另一台、不要重置卡的出路，实际「{text[:400]}」")
    alerted = [alert for alert in world.alerts() if alert.get("kind") == lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND]
    if len(alerted) != 1 or "block=12" not in str(alerted[0].get("line")):
        problems.append(f"告警记录里应当恰好一条 {lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}、写着那一块，实际 {alerted}")
    if problems:
        raise SelftestFailure("服务健康（GPU 服务判不了的块）：" + "；".join(problems), world.server_log_lines())
    return "GPU 服务报了判不了的块：收件 socket 收到恰好一条服务健康、写着那一块与出路，告警记录恰好一条 gpu-block-not-judgeable"


def cell_gpu_service_report_keeps_unread_hosts(world):
    """cards-failed 说第二台没读成（task_server_lib.py 的 gpu_service_report、gpu_service_problems_merged）：上一回第二台与这一台各有一张坏卡，
    这一回这一台读到、没有坏卡，第二台打了没读成那一行：第二台那一张留着（不当成好了），这一台那一张当成好了；第二台读成了、没有坏卡的那一回才去掉。
    纯函数，不起服务。弄坏开关 gpu-cards-unread-taken-as-healed 下第二台那一张也去掉，这一格红。"""
    here_line = f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=4242 {GPU_CARD_FAILED_SELFTEST_LINE}"
    peer_line = f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={GPU_PEER_SELFTEST_HOST} process=5151 {GPU_CARD_FAILED_SELFTEST_LINE}"
    before, _, _ = lib.gpu_service_report(f"{here_line}\n{peer_line}\n")
    unread_report = (f"GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={lib.GPU_THIS_HOST_LABEL} state=read lines=0\n"
                     f"GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={GPU_PEER_SELFTEST_HOST} state=unreachable lines=0\n"
                     f"{lib.GPU_CARDS_FAILED_UNREAD_PREFIX} host={GPU_PEER_SELFTEST_HOST} state=unreachable\n")
    found, unread, unparsed = lib.gpu_service_report(unread_report)
    kept = lib.gpu_service_problems_merged(before, found, unread)
    read_report = (f"GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={lib.GPU_THIS_HOST_LABEL} state=read lines=0\n"
                   f"GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host={GPU_PEER_SELFTEST_HOST} state=read lines=0\n")
    found_after, unread_after, _ = lib.gpu_service_report(read_report)
    healed = lib.gpu_service_problems_merged(kept, found_after, unread_after)
    problems = []
    if [host for _, _, host, _ in before] != sorted([lib.GPU_THIS_HOST_LABEL, GPU_PEER_SELFTEST_HOST]) or unparsed:
        problems.append(f"取样：上一回两台各一张坏卡，实际 {before}、认不出 {unparsed}")
    if [(kind, host) for kind, _, host, _ in kept] != [(f"{lib.GPU_CARD_FAILED_ALERT_KIND}:{GPU_PEER_SELFTEST_HOST}:5151:0", GPU_PEER_SELFTEST_HOST)]:
        problems.append(f"第二台没读成的那一回应当只留第二台那一张（这一台读到没有坏卡、当成好了），实际 {kept}")
    if healed:
        problems.append(f"第二台读成了、没有坏卡的那一回应当一样都不剩，实际 {healed}")
    if problems:
        raise SelftestFailure("没读成的台留着上一回读到的：" + "；".join(problems))
    return "第二台没读成时它那一张坏卡留着、这一台的当成好了；读成了才去掉"


def gpu_service_line_fields_of_the_source(constant):
    """gpu-verification-server/src/serving_from_the_library.rs 里以 {<constant>} 起头的那个格式串（服务打那一行用的）的字段名，按出现的先后；找不到交 None。"""
    with open(lib.GPU_SERVICE_SOURCE, encoding="utf-8") as handle:
        source = handle.read()
    found = re.search(r'"\{' + re.escape(constant) + r'\} ((?:[^"\\]|\\.)*)"', source)
    return None if found is None else re.findall(r"(\w+)=", found.group(1))


def cell_gpu_card_lines_match_the_service_fields(world):
    """服务打的卡坏行与判不了那一行的字段（从 gpu-verification-server/src/serving_from_the_library.rs 那两个格式串读字段名）与任务服务的解析逐字段对上
    （task_server_lib.py 的 GPU_CARD_FIELD、GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS、GPU_BLOCK_CLAIM_FIELD、gpu_service_report）：照源码的字段名造一行，解出的种类带着卡号、族、流键头
    （判不了的按流并、不带块号与进程号）；认领那一栏的几种值（源码里的 CLAIM_ 常量）与任务服务认的放了、没放两组合起来逐个相同；
    卡坏行的卡号那一栏改了名（取不到卡号）的不并成「?」，进认不出的行。纯函数，读源码、不起服务。
    弄坏开关 gpu-card-number-defaulted 下取不到卡号的记成「?」照收、gpu-block-not-judgeable-keyed-per-block 下种类带块号与进程号，这一格红。"""
    problems = []
    card_fields = gpu_service_line_fields_of_the_source("CARD_FAILED_LINE_PREFIX")
    block_fields = gpu_service_line_fields_of_the_source("BLOCK_NOT_JUDGEABLE_LINE_PREFIX")
    if card_fields is None or block_fields is None:
        raise SelftestFailure(f"{lib.GPU_SERVICE_SOURCE} 里找不到卡坏行或判不了那一行的格式串（卡坏 {card_fields}、判不了 {block_fields}）")
    if lib.GPU_CARD_FIELD not in card_fields:
        problems.append(f"服务卡坏行的字段 {card_fields} 里没有任务服务取卡号的 {lib.GPU_CARD_FIELD}")
    missing = [name for name in (*lib.GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS, lib.GPU_BLOCK_CLAIM_FIELD) if name not in block_fields]
    if missing:
        problems.append(f"服务判不了那一行的字段 {block_fields} 里没有任务服务认块与认领的 {missing}")
    with open(lib.GPU_SERVICE_SOURCE, encoding="utf-8") as handle:
        claim_values = sorted(re.findall(r'const CLAIM_\w+: &str = "([^"]*)";', handle.read()))
    if claim_values != sorted(lib.GPU_BLOCK_CLAIMS_LET_GO + lib.GPU_BLOCK_CLAIMS_KEPT):
        problems.append(f"服务判不了那一行认领的几种值（源码 CLAIM_ 常量）{claim_values} 与任务服务认的放了 {lib.GPU_BLOCK_CLAIMS_LET_GO}、"
                        f"没放 {lib.GPU_BLOCK_CLAIMS_KEPT} 合起来不相同")
    sample = {"card": "3", "family": "facts-verifier", "stream": "00112233aabbccdd", "block": "41", "claim": lib.GPU_BLOCK_CLAIMS_LET_GO[0]}
    card_line = "GPU_VERIFICATION_SERVER_CARD_FAILED " + " ".join(f"{name}={sample.get(name, 'x')}" for name in card_fields)
    block_line = "GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE " + " ".join(f"{name}={sample.get(name, 'x')}" for name in block_fields)
    report = (f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=42 {card_line}\n"
              f"{lib.GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=42 {block_line}\n")
    found, _, unparsed = lib.gpu_service_report(report)
    wanted = sorted([f"{lib.GPU_CARD_FAILED_ALERT_KIND}:{lib.GPU_THIS_HOST_LABEL}:42:3",
                     f"{lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:{lib.GPU_THIS_HOST_LABEL}:facts-verifier:00112233aabbccdd"])
    if [kind for kind, _, _, _ in found] != wanted or unparsed:
        problems.append(f"照源码字段造的两行应当解成 {wanted}、没有认不出的，实际 {[kind for kind, _, _, _ in found]}、认不出 {unparsed}")
    renamed = card_line.replace(f" {lib.GPU_CARD_FIELD}=", " card_index=", 1)
    found_renamed, _, unparsed_renamed = lib.gpu_service_report(
        f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={lib.GPU_THIS_HOST_LABEL} process=42 {renamed}\n")
    if found_renamed or len(unparsed_renamed) != 1:
        problems.append(f"卡号那一栏改了名的一行应当进认不出的行、不并成「?」，实际读到 {found_renamed}、认不出 {unparsed_renamed}")
    if problems:
        raise SelftestFailure("服务打的行与任务服务的解析逐字段对：" + "；".join(problems))
    return f"卡坏行字段 {card_fields}、判不了那一行字段 {block_fields} 与任务服务认的栏对上；卡号取不到的进认不出的行"


def cell_gpu_health_survives_restart(world):
    """GPU 服务卡坏发过、告警写过，服务停了再起（同一份库上另建调度器，task_server.py 的 watch_health、RESTART_SURVIVING_STATES 的 gpu_alerted）：
    起来之后头一回读 GPU 服务之前（读到的是空的）不当成好了、不忘掉；读完之后同一张卡不再发、不再写告警。套在假调度器上，不起服务。
    弄坏开关 gpu-health-forgotten-before-the-first-read 下头一回读之前就忘掉、读到之后再发一回；gpu-alerted-memory-only 下再写一回告警。这一格红。"""
    gpu_problem = [(f"{lib.GPU_CARD_FAILED_ALERT_KIND}:{lib.GPU_THIS_HOST_LABEL}:4242:0", "GPU 服务卡坏", "这一台的 gpu-verification-server（进程 4242）报卡坏")]
    before = reminder_restart_scheduler(world, "gpu-health-restart.sqlite3")
    before.health_problems = lambda: list(gpu_problem)
    SERVER.Scheduler.watch_health(before)
    after = reminder_restart_scheduler(world, "gpu-health-restart.sqlite3")
    after.health_problems = lambda: []
    SERVER.Scheduler.watch_health(after)
    after.gpu_cards_failed_known = True
    after.health_problems = lambda: list(gpu_problem)
    SERVER.Scheduler.watch_health(after)
    problems = []
    if len(before.sent) != 1 or len([alert for alert in before.alerts if alert[0] == lib.GPU_CARD_FAILED_ALERT_KIND]) != 1:
        problems.append(f"换服务之前应当发一回、写一条告警，实际发 {len(before.sent)} 回、告警 {before.alerts}")
    if after.sent:
        problems.append(f"换服务之后头一回读之前不忘掉、读到之后同一张卡不再发，实际发了 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    if after.alerts:
        problems.append(f"换服务之后同一张卡不再写告警，实际 {after.alerts}")
    if problems:
        raise SelftestFailure("GPU 服务卡坏换服务后不重发：" + "；".join(problems))
    return "GPU 服务卡坏发过一回、告警写过一条，换服务后头一回读之前不忘掉，读到同一张卡不再发、不再写告警"


def gpu_read_scheduler(world, database_name, report):
    """读 GPU 服务那几格共用的假调度器：reminder_restart_scheduler 再套上 Scheduler.__init__ 里读卡坏那几样的初值；读卡坏的那一条是这一格目录里一份
    只 cat 出 report 那份文件的假 cards-failed（TASK_SERVER_SELFTEST_GPU_CARDS_FAILED），服务健康只查读 GPU 服务得来的那几样（task_server.py 的 gpu_card_failures）。
    gpu_cards_failed_reading 一直记成在读：gpu_card_failures 不另起读的线程，读由 gpu_read_round 照次序同步做。"""
    fake = os.path.join(world.root, "fake-gpu-cards-failed-report")
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write(f"#!/bin/sh\ncat {shlex.quote(report)}\n")
    os.chmod(fake, 0o755)
    scheduler = reminder_restart_scheduler(world, database_name)
    scheduler.environment["TASK_SERVER_SELFTEST_GPU_CARDS_FAILED"] = fake
    scheduler.gpu_cards_failed, scheduler.gpu_lines_unparsed, scheduler.gpu_cards_failed_read_at = [], [], 0.0
    scheduler.gpu_cards_failed_reading = True
    scheduler.health_problems = lambda: SERVER.Scheduler.gpu_card_failures(scheduler)
    return scheduler


def gpu_read_round(scheduler, report, text):
    """一回：假 cards-failed 这一回打 text，同步读一回（task_server.py 的 read_gpu_cards_failed），再照读到的走一轮服务健康（watch_health）。"""
    with open(report, "w", encoding="utf-8") as handle:
        handle.write(text)
    SERVER.Scheduler.read_gpu_cards_failed(scheduler)
    scheduler.gpu_cards_failed_reading = True
    SERVER.Scheduler.watch_health(scheduler)


def gpu_read_line(host, state):
    """cards-failed 每台那一行（lines= 自证里不读，写 0）；没读成的两种另带没读成那一行。"""
    line = f"{lib.GPU_CARDS_FAILED_READ_PREFIX} host={host} state={state} lines=0\n"
    return line + (f"{lib.GPU_CARDS_FAILED_UNREAD_PREFIX} host={host} state={state}\n" if state not in lib.GPU_CARDS_FAILED_READ_STATES else "")


def cell_gpu_health_unread_host_survives_restart(world):
    """服务重起后头一回读 GPU 服务，第二台没读成（task_server.py 的 read_gpu_cards_failed、gpu_health_settled、watch_health）：换服务之前第二台一张坏卡发过、
    告警写过（health_notified 落盘）；换服务（同一份库上另建调度器）之后头一回读，这一台读成、第二台 unreachable，第二台那一张不当成好了、health_notified 里还在；
    下一回第二台读成、坏卡还在，全程不再发、不再写告警。套在假调度器上、照字面走 read_gpu_cards_failed 与 gpu_card_failures，不起服务。
    弄坏开关 gpu-health-unread-host-taken-as-healed、gpu-health-forgotten-before-the-first-read 下第二台那一张在没读成那一回就被忘掉、读成之后再发；
    health-notified-memory-only 下换服务之后再发；gpu-alerted-memory-only 下再写告警；health-alerted-every-round 下再发；health-unwatched、gpu-card-failure-unwatched 下
    换服务之前就一条不发。这一格红。"""
    report = os.path.join(world.root, "gpu-unread-host-report")
    peer_kind = f"{lib.GPU_CARD_FAILED_ALERT_KIND}:{GPU_PEER_SELFTEST_HOST}:5151:0"
    both_read = (gpu_read_line(lib.GPU_THIS_HOST_LABEL, "read") + gpu_read_line(GPU_PEER_SELFTEST_HOST, "read")
                 + f"{lib.GPU_FAILED_CARD_OF_A_HOST_PREFIX} host={GPU_PEER_SELFTEST_HOST} process=5151 {GPU_CARD_FAILED_SELFTEST_LINE}\n")
    peer_unread = gpu_read_line(lib.GPU_THIS_HOST_LABEL, "read") + gpu_read_line(GPU_PEER_SELFTEST_HOST, "unreachable")
    before = gpu_read_scheduler(world, "gpu-unread-host-restart.sqlite3", report)
    gpu_read_round(before, report, both_read)
    after = gpu_read_scheduler(world, "gpu-unread-host-restart.sqlite3", report)
    gpu_read_round(after, report, peer_unread)
    kept_while_unread = peer_kind in after.health_notified
    gpu_read_round(after, report, both_read)
    problems = []
    if len(before.sent) != 1 or [alert[0] for alert in before.alerts] != [lib.GPU_CARD_FAILED_ALERT_KIND]:
        problems.append(f"换服务之前第二台那一张坏卡应当发一回、写一条告警，实际发 {len(before.sent)} 回、告警 {before.alerts}")
    if not kept_while_unread:
        problems.append(f"换服务后头一回读第二台没读成，它那一张坏卡（{peer_kind}）应当还记着发过、不当成好了，实际 health_notified 只有 {sorted(after.health_notified)}")
    if after.sent:
        problems.append(f"第二台读成之后同一张坏卡不再发，实际发了 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    if after.alerts:
        problems.append(f"换服务之后同一张坏卡不再写告警，实际 {after.alerts}")
    if problems:
        raise SelftestFailure("服务重起后头一回读有台没读成：" + "；".join(problems))
    return "第二台坏卡发过一回，换服务后头一回读第二台没读成时它那一样留着，读成之后坏卡还在也不再发、不再写告警"


def gpu_block_line(host, process, stream, block, claim_field):
    """cards-failed 转来的判不了那一行：族 image-judgement、这一条流、这一块，认领那一栏照 claim_field 原样（空串就不带认领那一栏）。"""
    claim = f" {claim_field}" if claim_field else ""
    return (f"{lib.GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX} host={host} process={process} GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE "
            f"family=image-judgement stream={stream} block={block} card=0 why=judge-panicked{claim} reason=\"judge assertion\"\n")


def health_next_step_of(text):
    """发给会话的服务健康那一条里「出路：」那一行（去掉开头）；没有交空串。"""
    return next((line[len("出路："):] for line in text.split("\n") if line.startswith("出路：")), "")


def cell_gpu_block_not_judgeable_outlet_follows_claim(world):
    """判不了的块那一条的出路照各块的 claim= 写（task_server_lib.py 的 gpu_service_report，task_server.py 的 gpu_card_failures、gpu_block_not_judgeable_next_step）：
    一读四条流，claim=released 的、claim=not-held 的写放了（另一台会去试）、不写租期；claim=kept 的写没放（等认领租期过）、不写另一台会去试；
    另有一条流的那一行没有 claim=、一条流的 claim 值认不得，这两条不发判不了、并进一样「GPU 服务的卡坏行认不出」。套在假调度器上，不起服务。
    弄坏开关 gpu-block-not-judgeable-claim-ignored 下没放的也写成放了；gpu-block-not-judgeable-unwatched 下判不了的一条不发；health-unwatched、
    gpu-card-failure-unwatched 下一条不发。这一格红。"""
    report = os.path.join(world.root, "gpu-claim-report")
    streams = {"released": "a1a1a1a1a1a1a1a1", "not-held": "b2b2b2b2b2b2b2b2", "kept": "c3c3c3c3c3c3c3c3"}
    claimless, unknown = "d4d4d4d4d4d4d4d4", "e5e5e5e5e5e5e5e5"
    text = gpu_read_line(lib.GPU_THIS_HOST_LABEL, "read") + "".join(
        gpu_block_line(lib.GPU_THIS_HOST_LABEL, 4242, stream, 20, f"claim={claim}") for claim, stream in streams.items())
    text += gpu_block_line(lib.GPU_THIS_HOST_LABEL, 4242, claimless, 21, "") + gpu_block_line(lib.GPU_THIS_HOST_LABEL, 4242, unknown, 22, "claim=unsure")
    scheduler = gpu_read_scheduler(world, "gpu-claim-outlet.sqlite3", report)
    gpu_read_round(scheduler, report, text)
    sent = {stream: [message for message in scheduler.sent if f"流键头 {stream} " in message] for stream in (*streams.values(), claimless, unknown)}
    problems = []
    for claim, stream in streams.items():
        if len(sent[stream]) != 1:
            problems.append(f"claim={claim} 那一条流应当恰好发一条判不了，实际 {len(sent[stream])} 条")
            continue
        next_step = health_next_step_of(sent[stream][0])
        if claim in lib.GPU_BLOCK_CLAIMS_KEPT and ("租期" not in next_step or "会去试" in next_step):
            problems.append(f"claim={claim} 的出路应当写没放、等认领租期过，不写另一台会去试，实际「{next_step[:300]}」")
        if claim in lib.GPU_BLOCK_CLAIMS_LET_GO and ("另一台" not in next_step or "会去试" not in next_step or "租期" in next_step):
            problems.append(f"claim={claim} 的出路应当写放了、另一台会去试，不写租期，实际「{next_step[:300]}」")
    if sent[claimless] or sent[unknown]:
        problems.append(f"没有 claim= 与 claim 值认不得的两条流不该发判不了，实际 {len(sent[claimless])}、{len(sent[unknown])} 条")
    unparsed = [message for message in scheduler.sent if message.startswith(f"{SERVER.INBOX_HEADLINE}服务健康 GPU 服务的卡坏行认不出")]
    if len(unparsed) != 1 or "2 行认不出" not in unparsed[0]:
        problems.append(f"没有 claim= 与 claim 值认不得的两行应当并进恰好一条「GPU 服务的卡坏行认不出」、写 2 行认不出，实际 {[message[:200] for message in unparsed]}")
    if problems:
        raise SelftestFailure("判不了的块的出路照认领：" + "；".join(problems))
    return "claim=released、not-held 的出路写另一台会去试，claim=kept 的写等认领租期过；没有 claim= 与认不得的两行进认不出的行"


def cell_gpu_block_not_judgeable_one_per_stream(world):
    """同一条流判不了的几块并成一条、不带进程号（task_server_lib.py 的 gpu_service_report，task_server.py 的 gpu_health_settled、watch_health）：
    头一回一条流三块（进程 4242）判不了，发恰好一条、写 3 块与三个块号，告警一条；GPU 服务重起（下一回读到的没有这条流），再下一回进程换成 5353、同一条流三块又判不了：
    全程只发那一条、只写那一条告警。套在假调度器上，不起服务。
    弄坏开关 gpu-block-not-judgeable-keyed-per-block 下每块各发一条、换了进程再发；gpu-block-not-judgeable-forgotten-between-reads、
    gpu-health-forgotten-before-the-first-read 下中间那一回就忘掉、再判不了再发；health-alerted-every-round 下再发；gpu-block-not-judgeable-unwatched、
    health-unwatched、gpu-card-failure-unwatched 下一条不发。这一格红。"""
    report = os.path.join(world.root, "gpu-one-per-stream-report")
    stream = "0f0e0d0c0b0a0908"
    blocks = (12, 13, 14)
    here = gpu_read_line(lib.GPU_THIS_HOST_LABEL, "read")
    scheduler = gpu_read_scheduler(world, "gpu-one-per-stream.sqlite3", report)
    gpu_read_round(scheduler, report, here + "".join(gpu_block_line(lib.GPU_THIS_HOST_LABEL, 4242, stream, block, "claim=released") for block in blocks))
    gpu_read_round(scheduler, report, here)
    gpu_read_round(scheduler, report, here + "".join(gpu_block_line(lib.GPU_THIS_HOST_LABEL, 5353, stream, block, "claim=released") for block in blocks))
    problems = []
    headed = [message for message in scheduler.sent if message.startswith(f"{SERVER.INBOX_HEADLINE}服务健康 GPU 服务判不了的块")]
    if len(headed) != 1:
        problems.append(f"同一条流三块、换了进程再报，应当只发一条判不了，实际 {len(headed)} 条")
    elif "的 3 块" not in headed[0] or "块号 12、13、14" not in headed[0]:
        problems.append(f"那一条应当写这条流判不了 3 块与块号 12、13、14，实际「{headed[0][:400]}」")
    alerted = [alert for alert in scheduler.alerts if alert[0] == lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND]
    if len(alerted) != 1:
        problems.append(f"告警记录里应当恰好一条 {lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}，实际 {len(alerted)} 条")
    if problems:
        raise SelftestFailure("同一条流判不了并成一条：" + "；".join(problems))
    return "同一条流三块判不了只发一条、写 3 块与块号；GPU 服务重起之后换了进程再判不了不重发、不再写告警"


def cell_health_database_unwritable_notified(world):
    """库写不进（TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE 点名的文件在时试写当失败）：两个假会话各有一条在跑的任务，两个收件 socket 各收到恰好一条
    「task-server：服务健康 库写不进」；撤掉、等服务看到好了，再造一次，各再收恰好一条（好了再坏再发）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    arm = os.path.join(world.root, "health-write-fails")
    release = os.path.join(world.root, "release-health-database")
    inboxes = {SESSION_A: FakeInbox(world, "a"), SESSION_B: FakeInbox(world, "b", session=SESSION_B)}
    head = "task-server：服务健康 库写不进"
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE": arm})
    problems = []
    try:
        for session, inbox in inboxes.items():
            world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "health-database", release], session=session, extra=inbox.environment()))
        for episode in (1, 2):
            open(arm, "w").close()
            for inbox in inboxes.values():
                inbox.wait_for(world, head, count=episode)
            time.sleep(INBOX_QUIET_SECONDS)
            for session, inbox in inboxes.items():
                problems += inbox_problems(inbox.headed(head), head, f"第 {episode} 回坏、会话 {session}", count=episode)
            os.remove(arm)
            time.sleep(HEALTH_SETTLE_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(arm)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("服务健康（库写不进）：" + "；".join(problems), world.server_log_lines())
    return "库写不进：两个会话各收到一条，好了再坏各再收一条"


def cell_unclaimed_results_warned(world):
    """交任务时会话名下有结局满时限没人取（task_server.py 文件头 ②「提交时警告」②；自证里时限 1 秒）：两条跑完、没起 wait，再交一条照收照跑，标准错误打
    「✗ 严厉警告：会话 <号> 有 2 条任务跑完了没人取（<号>、<号>）」与「task-server/task wait <号> 取走」；三条都 wait 取走之后再交，不打。
    弄坏开关 unclaimed-unwarned 下这一格红。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS": str(UNCLAIMED_WARNING_SAMPLE_SECONDS)})
    left = [world.submit(["bash", "-c", "exit 0", f"unclaimed-{index}"]) for index in (1, 2)]
    for task_id in left:
        world.finished(task_id)
    time.sleep(UNCLAIMED_WARNING_SAMPLE_SECONDS + 0.5)
    problems = []
    warned = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "unclaimed-third"])
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", warned.stdout)
    head = f"✗ 严厉警告：会话 {SESSION_A} 有 2 条任务跑完了没人取（{left[0]}、{left[1]}）"
    if warned.returncode != 0 or not found or head not in warned.stderr or f"task-server/task wait {left[0]}" not in warned.stderr:
        problems.append(f"有两条没人取时交任务应当照收（退 0）、打「{head}」与「task-server/task wait <号> 取走」，实际退 {warned.returncode}、「{warned.stderr.strip()[-400:]}」")
    if found:   # 照收的那一条确认之后才进队、跑完（task_server.py 文件头 ⑦）
        world.confirm_submitted(warned)
    for task_id in left + ([int(found.group(1))] if found else []):
        world.finished(task_id)
        world.client(["wait", str(task_id)])
    quiet = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "unclaimed-fourth"])
    if quiet.returncode != 0 or "严厉警告" in quiet.stderr:
        problems.append(f"都取走之后再交不该打严厉警告，实际退 {quiet.returncode}、「{quiet.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("会话名下有结局没人取：" + "；".join(problems), world.server_log_lines())
    return f"两条跑完没人取时交任务打了严厉警告、点名 {left[0]}、{left[1]}，取走之后再交不打"


# 开了 GPU 的多机配置里换掉的那一段：判法要 ENABLE_GPU=1 时两个显卡键都写（这里各写本机一张卡，判法不判值）
GPU_ON_CONFIGURATION_KEYS = f"ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY={LOCAL_CARD_ZERO}\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED={LOCAL_CARD_ZERO}\n"
# control 起 gpu-verification-server/control 时给的出路里要有的那一句（control 的 GPU_SERVICE_START_HOWTO 开头）
GPU_SERVICE_START_TEXT = "python3 gpu-verification-server/control start"


def gpu_on_configuration(world):
    """这一格世界的多机配置抄一份、只把 ENABLE_GPU=0 那一行换成 GPU_ON_CONFIGURATION_KEYS，交新那一份的路径：task-server 读的键与世界那一份相同（同一个服务、同一个 socket）。"""
    with open(world.config, encoding="utf-8") as handle:
        text = handle.read()
    if text.count("\nENABLE_GPU=0\n") != 1:
        raise SelftestFailure(f"世界的多机配置 {world.config} 里 ENABLE_GPU=0 那一行应当恰好一行，实际 {text.count('ENABLE_GPU=')} 处写了 ENABLE_GPU")
    path = os.path.join(world.root, "gpu-on-multi-host.env")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace("\nENABLE_GPU=0\n", "\n" + GPU_ON_CONFIGURATION_KEYS))
    return path


def gpu_service_calls(world):
    """假 gpu-verification-server/control 收到过的调用（calls 里「gpu-service-control 」起头的行，按先后）。"""
    return [line for line in world.calls() if line.startswith("gpu-service-control ")]


def write_gpu_service_exit(world, code):
    """假 gpu-verification-server/control 之后退几（世界目录里的 gpu-service-exit）。"""
    with open(os.path.join(world.world, "gpu-service-exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")


def cell_gpu_service_required_when_gpu_on(world):
    """开了 GPU（多机配置 ENABLE_GPU=1）：control check 在任务服务那几样都过之后接着起 gpu-verification-server/control check（假的，照 gpu-service-exit 退），
    它退 78 时 check 退 78、出路写起它的命令；control start 在任务服务起好之后接着起 gpu-verification-server/control start，它退 78 时 start 退 78；
    假的退 0 时 check、start 都退 0、各起它一次。"""
    world.start_server()
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: gpu_on_configuration(world), "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "TASK_SERVER_GPU_SERVICE_CONTROL": os.path.join(world.bin, "fake-gpu-service-control")}
    problems = []
    for service_exit, want in ((78, lib.RUN_CONDITION_EXIT), (0, 0)):
        write_gpu_service_exit(world, service_exit)
        for subcommand in ("check", "start"):
            before = len(gpu_service_calls(world))
            completed = world.control([subcommand], extra=extra)
            called = gpu_service_calls(world)[before:]
            named = service_exit == 0 or GPU_SERVICE_START_TEXT in completed.stderr
            if completed.returncode != want or called != [f"gpu-service-control {subcommand}"] or not named:
                problems.append(f"假 GPU 服务退 {service_exit} 时 control {subcommand} 应当起它一次（「gpu-service-control {subcommand}」）、退 {want}"
                                f"{'、出路写「' + GPU_SERVICE_START_TEXT + '」' if service_exit else ''}，实际退 {completed.returncode}、起了 {called}、"
                                f"stderr「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("开了 GPU：" + "；".join(problems))
    return "开了 GPU：control check、start 都接着起 gpu-verification-server/control；它退 78 时 check、start 退 78、出路写起它的命令，退 0 时都退 0"


def cell_gpu_service_untouched_when_gpu_off(world):
    """没开 GPU（多机配置 ENABLE_GPU=0，各格世界的缺省）：假 gpu-verification-server/control 设着、照 gpu-service-exit 退 78，
    control check、start 都不起它，照任务服务自己的结论退 0。"""
    world.start_server()
    write_gpu_service_exit(world, 78)
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "TASK_SERVER_GPU_SERVICE_CONTROL": os.path.join(world.bin, "fake-gpu-service-control")}
    problems = []
    for subcommand in ("check", "start"):
        completed = world.control([subcommand], extra=extra)
        if completed.returncode != 0 or gpu_service_calls(world):
            problems.append(f"control {subcommand} 应当不起 gpu-verification-server/control、退 0，实际退 {completed.returncode}、起了 {gpu_service_calls(world)}、"
                            f"stderr「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没开 GPU：" + "；".join(problems))
    return "没开 GPU：control check、start 都不起 gpu-verification-server/control，退 0"


def cell_health_answers_while_resources_slow(world):
    """余量每读一次睡 SLOW_RESOURCES_SECONDS 秒（排队线程每一轮读一次），排头放得下却不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED）：/health 不等余量，照样报「判能起却没起成」，放开之后任务跑完。
    判「快」拿同一格里同时量的参照比：读一次余量要多久，取假余量睡的秒数与 /health 回话里排队线程距上一轮的最大秒数（scheduler_seconds_since_tick，那一轮就是卡在读余量上）两者中大的；
    答话的中位数要不到它的一半、最慢那一次要短于它。机器忙时库试写变慢，排队线程那一轮也跟着变长，参照随之变长；/health 要是每次都读一回余量，每次答话都不短于读一次余量。
    弄坏开关 health-judges-head 下 /health 现判排头、每次现读余量，这一格红。"""
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused, "TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS": str(SLOW_RESOURCES_SECONDS)})
    open(paused, "w").close()
    task_id = world.submit(["bash", "-c", "exit 0"])
    socket_path = os.path.join(world.runtime, lib.SOCKET_NAME)
    answers, passes_stuck_seconds, health = [], [], {}
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while patience.waiting():
        started = time.monotonic()
        health = lib.call_server(socket_path, "GET", "/health", timeout=4 * SLOW_RESOURCES_SECONDS)[1]
        answers.append(time.monotonic() - started)
        passes_stuck_seconds.append(float(health.get("scheduler_seconds_since_tick") or 0.0))
        if STARVED_HEALTH_TEXT in (health.get("problem") or "") and len(answers) >= HEALTH_PROBES:
            break
        time.sleep(POLL_SECONDS)
    os.remove(paused)
    summary = world.finished(task_id)
    median = sorted(answers)[len(answers) // 2] if answers else None
    one_read_seconds = max([SLOW_RESOURCES_SECONDS] + passes_stuck_seconds)
    if (median is None or median >= one_read_seconds / 2 or max(answers) >= one_read_seconds or STARVED_HEALTH_TEXT not in (health.get("problem") or "")
            or summary["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"余量每读一次要 {SLOW_RESOURCES_SECONDS} 秒（量到排队线程一轮最长 {max(passes_stuck_seconds or [0.0]):.2f} 秒）、排头判能起却不起：/health 应当不等余量"
                              f"（每次都短于读一次余量的 {one_read_seconds:.2f} 秒、中位数不到一半）、照样报「{STARVED_HEALTH_TEXT}」，接着派之后跑完；"
                              f"实际问了 {len(answers)} 次、中位数 {median}、最慢 {max(answers or [0]):.2f} 秒、最后一次 {health}、{describe(summary)}", world.server_log_lines())
    return (f"余量每读一次 {SLOW_RESOURCES_SECONDS} 秒（参照 {one_read_seconds:.2f} 秒）：/health 问了 {len(answers)} 次，中位数 {median:.2f} 秒、最慢 {max(answers):.2f} 秒，"
            f"排头放得下却不起照样报「{health.get('problem')}」")


SLOW_RESOURCES_SECONDS = 10    # health-answers-while-resources-slow 的假余量每读一次睡这么久：比机器忙时库试写的提交能慢到的程度大得多（答话与它比，不与写死的秒数比）
HEALTH_PROBES = 5              # 那一格至少问这么多次


def cell_nfs_share_required(world):
    """多机配置没写 NFS_SHARED_ROOTS（task_server_lib.py 文件头「NFS 共享」）：服务不起、退 78、点名这个键与 README 那一节；control start 起之前就拒、不起 systemctl。
    两处都要是 NFS 判法拒的（stderr 带 NFS_ROOTS_MISSING_TEXT）：共享数据根与第二台地址的判法也拒这份配置、也点名这个键与 README 那一节，只看这两样认不出 NFS 判法漏没漏。"""
    config = os.path.join(world.root, "no-nfs-multi-host.env")
    World.write_config(config, world.state, world.runtime, world.key, world.builds, "")
    named = (lib.NFS_ROOTS_KEY, "## NFS 共享", lib.NFS_ROOTS_MISSING_TEXT)
    problems = []
    code, output = world.serve_until_exit(config)
    if code != lib.RUN_CONDITION_EXIT or not all(text in output for text in named):
        problems.append(f"服务应当不起、退 78、stderr 点名 {lib.NFS_ROOTS_KEY} 与 README 的「## NFS 共享」、说「{lib.NFS_ROOTS_MISSING_TEXT}」，实际{'起来了' if code is None else f'退 {code}'}、"
                        f"stderr 里缺 {[text for text in named if text not in output]}")
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    started = world.control(["start"], extra=extra)
    if started.returncode != lib.RUN_CONDITION_EXIT or not all(text in started.stderr for text in named) or systemctl_start_count(world):
        problems.append(f"control start 应当起之前就退 78、点名同样三样、不起 systemctl start，实际退 {started.returncode}、systemctl start 收到 {systemctl_start_count(world)} 回、"
                        f"stderr「{started.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没写 NFS 共享：" + "；".join(problems), [output[-1500:]])
    return f"没写 {lib.NFS_ROOTS_KEY}：服务退 78、control start 起之前退 78，都是 NFS 判法拒的（「{lib.NFS_ROOTS_MISSING_TEXT}」），点名这个键与 README「## NFS 共享」"


def cell_nfs_share_verified(world):
    """NFS 共享四项（task_server_lib.py 文件头「NFS 共享」）：假 etab、假 ssh、假探针都过时服务起得来、/health 的 nfs 一栏四项都过、探针文件删了、control check 退 0；
    假 etab 去掉那一行、假 ssh 的 mountpoint 退 1、第二台写的探针本机读不回，服务各自不起、退 78、点名那个根与那一项（起着的服务判出挂载没过发消息给会话由 health-nfs-notified 判）。"""
    problems = []
    world.start_server()
    _, health = world.api("GET", "/health")
    roots = (health.get("nfs") or {}).get("roots") or []
    keys = [key for key, _ in lib.NFS_ITEMS]
    if [entry.get("root") for entry in roots] != [world.shared_root] or any(entry.get(key) != lib.NFS_PASSED for entry in roots for key in keys):
        problems.append(f"/health 的 nfs 一栏应当是 {world.shared_root} 四项都过，实际 {roots}")
    left = [name for name in (lib.NFS_PROBE_NAME, lib.NFS_PEER_PROBE_NAME) if os.path.lexists(os.path.join(world.shared_root, name))]
    if left:
        problems.append(f"两向探针读完应当删掉探针文件，实际还在 {left}")
    git_directory = os.path.join(world.shared_root, ".git")   # 根是 git 工作树：探针写进 .git，不在项目根下留条目
    os.makedirs(git_directory, exist_ok=True)
    try:
        placed = (lib.nfs_probe_directory(world.shared_root), lib.nfs_probe_directory(world.world))
        if placed != (git_directory, world.world):
            problems.append(f"根下有 .git 目录时探针应当写进 {git_directory}、没有时写在根下 {world.world}，实际 {placed}")
    finally:
        os.rmdir(git_directory)
    checked = world.control(["check"])
    if checked.returncode != 0 or "NFS 共享 过" not in checked.stdout:
        problems.append(f"四项都过时 control check 应当退 0、说 NFS 共享过，实际退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")
    world.stop_server()
    lost_write = os.path.join(world.world, "fake-ssh-peer-write-lost")
    cases = (("导出", lambda: world.write_exports([]), lambda: world.write_exports([world.shared_root]), ()),
             ("挂载", lambda: open(mount_exit, "w", encoding="utf-8").write("1\n"), lambda: os.remove(mount_exit), ()),
             ("两向探针", lambda: open(lost_write, "w").close(), lambda: os.remove(lost_write), ("第二台→本机",)))
    for item, arm, disarm, more in cases:
        arm()
        try:
            code, output = world.serve_until_exit(world.config)
        finally:
            disarm()
        named = (world.shared_root, f"「{item}」", "## NFS 共享") + more
        if code != lib.RUN_CONDITION_EXIT or not all(text in output for text in named):
            problems.append(f"「{item}」不过时服务应当不起、退 78、点名那个根、那一项与 README 那一节，实际{'起来了' if code is None else f'退 {code}'}、"
                            f"输出里缺 {[text for text in named if text not in output]}")
    if problems:
        raise SelftestFailure("NFS 共享：" + "；".join(problems), world.server_log_lines())
    return "四项都过时起得来、/health 的 nfs 四项都过、探针删了、check 退 0；导出、挂载、两向探针各不过时服务退 78 点名根与那一项"


def cell_peer_unavailable_alerted(world):
    """第二台用不了要告警（task_server.py 的 note_peer）：那一趟没跑成（peer-run-failed）、连不上（假余量的 peer_down）、挂载复核没过（假 ssh 的 mountpoint 退 1）
    各写一条告警 peer-unavailable、/health 的 peer 一栏写明原因（用不了满时限发消息给会话由 health-peer-unavailable-notified 判）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：派第二台
    world.start_server()
    problems = []
    failed_id = world.submit(["bash", "-c", "exit 0", "fake-peer-exit=255"], cwd=world.repo)
    failed = world.finished(failed_id, timeout=PLACEMENT_WAIT_SECONDS)
    if failed["task"].get("outcome") != "peer-run-failed":
        problems.append(f"假第二台退 255 应当记 peer-run-failed，实际 {describe(failed)}")
    if not any(alert.get("kind") == "peer-unavailable" and alert.get("task") == failed_id and "没跑成" in alert.get("line", "") for alert in world.alerts()):
        problems.append(f"那一趟没跑成应当写一条 peer-unavailable 告警、点名任务 {failed_id}，实际告警 {world.alerts()}")
    _, health = world.api("GET", "/health")
    if ((health.get("peer") or {}).get("last_run_failure") or {}).get("task") != failed_id:
        problems.append(f"/health 的 peer 一栏应当记下最近一次那一趟没跑成（任务 {failed_id}），实际 {health.get('peer')}")
    for what, arm, disarm in (("自证：第二台连不上", lambda: world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_down="自证：第二台连不上"),
                               lambda: world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)),
                              ("没挂上共享根", lambda: open(os.path.join(world.world, "fake-ssh-mount-exit"), "w", encoding="utf-8").write("1\n"),
                               lambda: os.remove(os.path.join(world.world, "fake-ssh-mount-exit")))):
        arm()
        patience = world.patience(PLACEMENT_WAIT_SECONDS)
        alerted = []
        while patience.waiting() and not alerted:
            alerted = [alert for alert in world.alerts() if alert.get("kind") == "peer-unavailable" and what in alert.get("line", "")]
            time.sleep(POLL_SECONDS)
        _, health = world.api("GET", "/health")
        peer = health.get("peer") or {}
        if not alerted or peer.get("available") is not False or what not in str(peer.get("why")):
            problems.append(f"第二台用不了（{what}）应当写一条 peer-unavailable 告警、/health 的 peer 一栏写用不了与原因，实际告警 {alerted}、peer 一栏 {peer}")
        disarm()
        back = health_until(world, lambda health: (health.get("peer") or {}).get("available") is True, STATE_TIMEOUT_SECONDS)
        if (back.get("peer") or {}).get("available") is not True:
            problems.append(f"{what}修好之后第二台应当又用得了（/health 的 peer 一栏），实际 {back.get('peer')}")
            break
    if problems:
        raise SelftestFailure("第二台用不了：" + "；".join(problems), world.server_log_lines())
    return "那一趟没跑成、连不上、挂载复核没过各写 peer-unavailable 告警、/health 写明原因"


REASON_LIVE_CASES = (   # reason-line-per-type 经 task wait 真跑的几种：(类型, 命令, 环境)
    ("command-failed", ["bash", "-c", "echo selftest-reason-last-line; exit 3"], {}),
    ("squeezed-out", ["bash", "-c", "exit 0", "fake-wrapper-exits=254"], {}),
    ("timed-out", ["bash", "-c", "exit 0", "fake-wrapper-exits=253"], {}),
    ("run-condition-unmet", ["bash", "-c", "echo selftest-reason-unmet; exit 78"], {}),
    ("not-run", ["bash", "-c", "exit 77"], {}),
)


def reason_problems(line, outcome):
    """一行 TASK_SERVER_REASON 对不对：type 是这一种、REASON_ELEMENTS 给它的要素与 why、next 都在且不空；→ 不对的那几句。"""
    if not line.startswith(lib.REASON_MARK + " "):
        return [f"{outcome}：没有 {lib.REASON_MARK} 那一行（读到「{line[:200]}」）"]
    try:
        fields = dict(word.split("=", 1) for word in shlex.split(line)[1:] if "=" in word)
    except ValueError as error:
        return [f"{outcome}：那一行 shlex 拆不开（{error}）：「{line[:200]}」"]
    wrong = [] if fields.get("type") == outcome else [f"{outcome}：type 应当是 {outcome}，实际 {fields.get('type')}"]
    missing = [name for name in lib.REASON_ELEMENTS.get(outcome, ()) + ("why", "next") if not fields.get(name)]
    return wrong + ([f"{outcome}：缺要素 {missing}（「{line[:300]}」）"] if missing else [])


def cell_reason_line_per_type(world):
    """打回的理由（task_server_lib.py 文件头「错误类型」表后的「打回的理由」）：错误类型表的每一种都在 REASON_ELEMENTS 里、喂一份造好的结局都打得出这一种的要素；
    几种真跑的经 task wait 在末行之前打出 TASK_SERVER_REASON、type 与末行对得上、要素都在（命令自己退的带日志最后一行）。"""
    problems = []
    extra_types = sorted(set(lib.OUTCOME_TYPES) ^ set(lib.REASON_ELEMENTS))
    if extra_types:
        problems.append(f"错误类型表与 REASON_ELEMENTS 对不上：{extra_types}")
    log_text = "MULTI_HOST_RUN_PEER_CPU poll=1 usage_usec=0 read=no\nselftest-last-line\n"
    for outcome in lib.OUTCOME_TYPES:
        task = {"id": 1, "outcome": outcome, "exit_code": lib.final_exit_code(outcome, 3), "note": f"自证造的说明（{outcome}）", "maximum_host": "local",
                "maximum_cap_text": "46080M", "outcome_counts": {outcome: 2}}
        attempt = {"number": 1, "host": "local", "memory_cap_text": "2048M", "memory_peak": 2 * GIBIBYTE, "unit": "singlefs-task-selftest.service",
                   "squeezed_by": "任务 2（上限 4096M）"}
        problems += reason_problems(lib.reason_line(task, [attempt], log_text), outcome)
    world.resources(local=(100, 100))
    world.start_server()
    submitted = [(outcome, world.submit(command, extra=extra)) for outcome, command, extra in REASON_LIVE_CASES]
    for outcome, task_id in submitted:
        world.finished(task_id)
        completed = world.client(["wait", str(task_id)])
        lines = completed.stdout.strip().splitlines()
        if len(lines) < 2 or not lines[-1].startswith(f"{lib.RESULT_MARK} task={task_id} type={outcome} "):
            problems.append(f"任务 {task_id} 应当收尾成 {outcome}、wait 末行写着它，实际末行「{lines[-1] if lines else ''}」")
            continue
        problems += reason_problems(lines[-2], outcome)
        if outcome == "command-failed" and "selftest-reason-last-line" not in lines[-2]:
            problems.append(f"command-failed 的那一行应当带日志最后一行 selftest-reason-last-line，实际「{lines[-2][:300]}」")
    if problems:
        raise SelftestFailure("打回的理由：" + "；".join(problems), world.server_log_lines())
    return (f"错误类型表 {len(lib.OUTCOME_TYPES)} 种各打得出带要素的 {lib.REASON_MARK} 行；真跑的 {len(REASON_LIVE_CASES)} 种 wait 在末行之前打出、type 对得上、要素都在")


ATTENTION_QUEUE_SECONDS = "2"        # attention-queued-too-long：排队过久的提醒门槛（秒；TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS）
ATTENTION_ONCE_SECONDS = 6           # attention-reported-once：排满 6 秒提醒一回、排满 12 秒再提醒一回（每翻一倍一回）
ATTENTION_ONCE_QUIET_SECONDS = 2.5   # inbox-queued-too-long-once：6 秒那一回收到之后再看这么久，不该再多收一条（离 12 秒那一回还有 6 秒）


def cell_attention_queued_too_long(world):
    """排队过久（task_server.py 文件头 ①「要处置的提醒」）：第二台用不了、只有它放得下的命令一直排着（同 down-host-waits-not-never），排满提醒门槛（自证里 2 秒）
    wait 打一行 TASK_SERVER_ATTENTION kind=queued-too-long、依据写已排多久、原因写排队原因（第二台用不了）、退 ATTENTION_EXIT；任务照旧在排（没起尝试、没结局），
    第二台回来之后照派过去跑完。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-queue", 30 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": ATTENTION_QUEUE_SECONDS})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-queue"], extra={lib.KEY_VARIABLE: "selftest-attention-queue"}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    if "已排" not in fields.get("basis", "") or "第二台" not in fields.get("why", ""):
        problems.append(f"依据应当写「已排 …」、原因应当带排队原因里的「第二台」，实际依据「{fields.get('basis')}」、原因「{fields.get('why')}」")
    if during["task"].get("state") != "queued" or during["task"].get("outcome") or during["attempts"]:
        problems.append(f"报了排队过久之后任务应当照旧在排（没起尝试、没结局），实际 {describe(during)}、状态 {during['task'].get('state')}")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("排队过久：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_QUEUE_SECONDS} 秒：wait 打 {lib.ATTENTION_MARK} kind=queued-too-long（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在排，第二台回来后跑完"


def cell_attention_quiet_busy_log(world):
    """日志长期不涨而服务没停它（task_server.py 的 quiet_log_items；任务 7723 日志 37702 秒没涨，服务一直不停也没人叫）：一条只打一行
    就不出声、跑到放行为止，假 scope 的 CPU 一直在涨（卡住阈值 2 秒：CPU 在涨不停），此刻内存读不到（不判贴着上限）；日志满提醒门槛（自证里 3 秒）没涨，
    wait 打一行 kind=quiet-log-busy、依据写日志多久没涨、退 ATTENTION_EXIT；任务照旧在跑，放行之后跑完。
    弄坏开关 attention-quiet-busy-unreported 下 wait 不叫醒、等到命令自己退，这一格红。"""
    scope, stop = start_burning_scope(world, "quiet-busy-scope", {})
    release = os.path.join(world.root, "release-attention-quiet")
    problems, fields = [], {}
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                                  "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS": ATTENTION_QUIET_SECONDS})
        task_id = world.submit(["bash", "-c", "echo started; " + BLOCK_SCRIPT, "quiet-busy", release])
        completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
        during = world.summary(task_id)
        problems, fields = attention_problems(completed, task_id, "quiet-log-busy", lib.ATTENTION_EXIT)
        if "没涨" not in fields.get("basis", ""):
            problems.append(f"依据应当写日志多久「没涨」，实际「{fields.get('basis')}」")
        if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"报了日志长期不涨之后任务应当照旧在跑（running），实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(release, "w").close()
        done = world.finished(task_id)
    finally:
        open(release, "w").close()
        stop.set()
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("日志长期不涨而服务没停它：" + "；".join(problems), world.server_log_lines())
    return f"日志满 {ATTENTION_QUIET_SECONDS} 秒没涨、CPU 在涨：wait 打 {lib.ATTENTION_MARK} kind=quiet-log-busy（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，放行后跑完"


PEER_HEALTH_HEAD = "task-server：服务健康 NFS 共享判不过"
PEER_SLOW_LOG_TEXT = "第二台慢（超时、复查通了）"


def cell_peer_health_slow_rechecked(world):
    """第二台一次超时先当慢（task_server_lib.py 文件头「NFS 共享」的挂载；三个会话各报过一次 30 秒超时的「用不了」）：ssh 时限压到 1 秒、复查前隔 0.2 秒、复查期 0.5 秒（一回复查就到期）；
    假 ssh 头一回复核挂载那一句睡着（fake-ssh-hang-once），复查照常回 nfs：服务日志记一行「第二台慢（超时、复查通了）」，会话收不到服务健康；
    再让每一回都睡着（fake-ssh-hang）：复查期满都超时，会话收到恰好一条「服务健康 NFS 共享判不过」，正文写「仍不通」。
    弄坏开关 peer-health-single-timeout-reported 下头一回超时就报、服务日志没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-peer-health")
    once, hang = os.path.join(world.world, "fake-ssh-hang-once"), os.path.join(world.world, "fake-ssh-hang")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_NFS_SSH_SECONDS": "1", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS": "0.2", "TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS": "0.5"})
    problems = []
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "peer-health", release], extra=inbox.environment())
        world.running(task_id)
        open(once, "w").close()
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while PEER_SLOW_LOG_TEXT not in server_output_text(world) and patience.waiting():
            time.sleep(POLL_SECONDS)
        time.sleep(INBOX_QUIET_SECONDS)
        if PEER_SLOW_LOG_TEXT not in server_output_text(world) or not os.path.exists(once + ".taken"):
            problems.append(f"一次超时、复查回了的应当在服务日志记一行「{PEER_SLOW_LOG_TEXT}」，实际{'没睡着那一回' if not os.path.exists(once + '.taken') else '没有那一行'}")
        early = inbox.headed(PEER_HEALTH_HEAD)
        if early:
            problems.append(f"一次超时、复查回了的不该推服务健康，实际收到 {len(early)} 条：「{early[0]['text'][:300]}」")
        open(hang, "w").close()
        found = inbox.wait_for(world, PEER_HEALTH_HEAD)
        problems += inbox_problems(found, PEER_HEALTH_HEAD, "两次都不通")
        if found and "仍不通" not in found[0]["text"]:
            problems.append(f"复查期满都不通的那一条应当写「仍不通」（连不上），实际「{found[0]['text'][:300]}」")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("第二台一次超时先当慢：" + "；".join(problems), world.server_log_lines())
    return f"一次超时、复查回了：服务日志记「{PEER_SLOW_LOG_TEXT}」、不推；复查期满都不通：推恰好一条服务健康，写仍不通"


def cell_peer_health_slow_within_recheck_period(world):
    """第二台连着慢几回、复查期里回了的只记慢、不群发（task_server_lib.py 的 peer_shell_rechecked；两台的盘被写满那一阵握手超过 30 秒、几分钟后又回来，服务判成连不上、
    给所有有任务的会话群发服务健康）：ssh 时限压到 1 秒、复查前隔 0.2 秒、复查期 3 秒；假 ssh 头两回复核挂载那一句睡着（fake-ssh-hang-count 写 2）、第三回照常回：
    服务日志记「第二台慢（超时、复查通了）」且写「复查到第 2 回回了」、/health 的 nfs 一栏带 slow_last、会话收不到服务健康；之后每一回都睡着（fake-ssh-hang）：
    复查期满仍不通，会话收到恰好一条「服务健康 NFS 共享判不过」。
    弄坏开关 peer-recheck-once-only 下复查期里只复查一回，头两回超时就报，会话收到服务健康、服务日志没有慢的那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-peer-health-period")
    counted, hang = os.path.join(world.world, "fake-ssh-hang-count"), os.path.join(world.world, "fake-ssh-hang")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_NFS_SSH_SECONDS": "1", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS": "0.2", "TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS": "3"})
    problems = []
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "peer-health-period", release], extra=inbox.environment())
        world.running(task_id)
        with open(counted, "w", encoding="utf-8") as handle:
            handle.write("2\n")
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while "复查到第 2 回回了" not in server_output_text(world) and patience.waiting():
            time.sleep(POLL_SECONDS)
        time.sleep(INBOX_QUIET_SECONDS)
        if PEER_SLOW_LOG_TEXT not in server_output_text(world) or "复查到第 2 回回了" not in server_output_text(world):
            problems.append(f"头两回超时、第三回回了的应当在服务日志记「{PEER_SLOW_LOG_TEXT}」并写「复查到第 2 回回了」，实际 fake-ssh-hang-count 剩 {open(counted).read().strip()}")
        early = inbox.headed(PEER_HEALTH_HEAD)
        if early:
            problems.append(f"头两回超时、第三回回了的不该推服务健康，实际收到 {len(early)} 条：「{early[0]['text'][:300]}」")
        slow_last = health_until(world, lambda health: bool((health.get("nfs") or {}).get("slow_last")), STATE_TIMEOUT_SECONDS).get("nfs", {}).get("slow_last")
        if not slow_last or "复查到第 2 回回了" not in " ".join(slow_last.get("slow") or []):
            problems.append(f"/health 的 nfs 一栏应当带 slow_last 并写「复查到第 2 回回了」，实际 {slow_last}")
        open(hang, "w").close()
        found = inbox.wait_for(world, PEER_HEALTH_HEAD)
        problems += inbox_problems(found, PEER_HEALTH_HEAD, "复查期满都不通")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("第二台连着慢几回的复查期：" + "；".join(problems), world.server_log_lines())
    return "头两回超时、第三回回了：服务日志与 /health 记慢、不推；复查期满都不通：推恰好一条服务健康"


def cell_attention_cleanup_pending(world):
    """清理没收尾：派第二台的一条，第二台那一层写 pending 的清理回执（假 multi-host-run.sh 的 fake-peer-cleanup=pending）却一直不确认，服务保留这一次尝试的占用、任务不记结局；
    wait 打一行 kind=cleanup-pending、依据带回执未确认的原因、退 ATTENTION_EXIT；回执改成 confirmed 之后任务跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "attention-cleanup", "fake-peer-cleanup=pending"], cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "cleanup-pending", lib.ATTENTION_EXIT)
    attempt = (during["attempts"] or [{}])[-1]
    if "清理没收尾" not in fields.get("basis", "") or "未确认" not in fields.get("basis", ""):
        problems.append(f"依据应当写「清理没收尾」与回执未确认的原因，实际「{fields.get('basis')}」")
    if during["task"].get("state") != "running" or attempt.get("state") not in ("starting", "running") or not attempt.get("cleanup_pending"):
        problems.append(f"报了清理没收尾之后任务应当照旧在跑、这一次尝试保持在跑并记着 cleanup_pending，实际 {describe(during)}、状态 {during['task'].get('state')}、尝试 {attempt.get('state')}")
    receipt = attempt.get("peer_cleanup_file") or ""
    if receipt:
        with open(receipt + ".partial", "w", encoding="utf-8") as handle:
            json.dump({"status": "confirmed", "label": "selftest"}, handle)
        os.replace(receipt + ".partial", receipt)
    else:
        problems.append(f"尝试记录里应当有 peer_cleanup_file，实际 {attempt}")
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"回执确认之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("清理没收尾：" + "；".join(problems), world.server_log_lines())
    return f"第二台清理回执一直 pending：wait 打 {lib.ATTENTION_MARK} kind=cleanup-pending（依据「{fields.get('basis', '')[:80]}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，回执确认后跑完"


def cell_attention_shard_advance_failed(world):
    """分片父任务推不动：自证造的错（TASK_SERVER_SELFTEST_ADVANCE_FAILS 点名的文件在时每一轮推父任务都抛）让整件停在原地（片在等放行文件）；
    wait 打一行 kind=shard-advance-failed、依据写那个错、退 ATTENTION_EXIT，父任务照旧是 sharded；撤掉那份文件、放行之后整件跑完。"""
    arm = os.path.join(world.root, "advance-fails")
    release = os.path.join(world.root, "release-attention-shard")
    open(arm, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_ADVANCE_FAILS": arm})
    command = world.write_spec("spec-attention-advance.json", {"shards": [{"block": release}]})
    task_id = world.submit(command, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "shard-advance-failed", lib.ATTENTION_EXIT)
    if "自证造的推不动" not in fields.get("basis", ""):
        problems.append(f"依据应当写出推不动的那个错（「自证造的推不动」），实际「{fields.get('basis')}」")
    if during["task"].get("state") != "sharded":
        problems.append(f"报了推不动之后父任务应当照旧是 sharded，实际状态 {during['task'].get('state')}、{describe(during)}")
    os.remove(arm)
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"撤掉造的错、放行之后整件应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("分片父任务推不动：" + "；".join(problems), world.server_log_lines())
    return f"分片父任务每一轮推都出错：wait 打 {lib.ATTENTION_MARK} kind=shard-advance-failed（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，父任务照旧 sharded，撤掉错后跑完"


def cell_attention_reported_once(world):
    """同一条任务同一种提醒、同一道提醒门槛只叫醒一次：排队过久（排满 6 秒、12 秒各一道门槛，每翻一倍一道）叫醒过一次，主 agent 重起 wait 之后不再叫同一道门槛（再起的 wait 一直等着），
    排满下一道门槛才再叫一次（reminded_after_seconds 翻一倍，依据不同、写着再排满一倍再提醒一回）；任务一直是在排。
    判据看再起的 wait 报的是哪一道门槛，不看它隔多久才退：机器忙时头一回叫得晚、或再起的 wait 起得晚，已经过了下一道门槛的，它报下一道也对（读服务记的门槛号，不读客户端的挂钟）。
    弄坏开关 attention-reported-every-wait 下再起的 wait 照报头一回那一道门槛，这一格红。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-once", 30 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-once"], extra={lib.KEY_VARIABLE: "selftest-attention-once"}, cwd=world.repo)
    first = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, first_fields = attention_problems(first, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    second = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    more, second_fields = attention_problems(second, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    problems += more
    first_threshold = first_fields.get("reminded_after_seconds", "")
    later = 2 * int(first_threshold) if first_threshold.isdigit() else None
    reached = (first_threshold, second_fields.get("reminded_after_seconds"))
    if (not first_threshold.isdigit() or int(first_threshold) < ATTENTION_ONCE_SECONDS or reached[1] != str(later)
            or first_fields.get("basis") == second_fields.get("basis") or f"排满 {2 * (later or 0)} 秒再提醒一回" not in second_fields.get("basis", "")):
        problems.append(f"头一回应当 reminded_after_seconds 是 {ATTENTION_ONCE_SECONDS} 或更大的一道门槛、再起的 wait 报的应当是它的下一道（翻一倍：{later}），不重报头一回那一道、"
                        f"依据不同且写着排满 {2 * (later or 0)} 秒再提醒一回，实际 {reached}、依据「{first_fields.get('basis')}」与「{second_fields.get('basis')}」")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("提醒只叫醒一次：" + "；".join(problems), world.server_log_lines())
    return f"排满 {first_threshold} 秒叫醒一次，重起的 wait 不再叫这一道，排满 {later} 秒才再叫（依据「{second_fields.get('basis')}」）"


ATTENTION_RESTART_SECONDS = 8   # attention-reported-survives-restart：排满 8 秒、16 秒各一道提醒门槛（8 秒那一回报过之后重启服务留得出 8 秒）


def cell_attention_reported_survives_restart(world):
    """报到的提醒门槛记进库（task_server.py 文件头 ①「要处置的提醒」；会话 46 的任务 8819 那 30 片发过一轮，主 agent 重启服务之后同一道门槛又发一轮）：
    排队过久（排满 8 秒、16 秒各一道门槛）报过 8 秒那一回，停掉服务、照同一份库重起，再起的 wait 不再报 8 秒那一回、排满 16 秒才报；任务一直在排。
    弄坏开关 attention-reported-memory-only 下照改前只记在服务内存里：重起之后再起的 wait 当场又报 8 秒那一回，这一格红。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-restart", 30 * GIBIBYTE)
    extra = {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_RESTART_SECONDS)}
    world.start_server(extra=extra)
    task_id = world.submit(["bash", "-c", "exit 0", "attention-restart"], extra={lib.KEY_VARIABLE: "selftest-attention-restart"}, cwd=world.repo)
    first = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, first_fields = attention_problems(first, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    world.stop_server()
    world.start_server(extra=extra)
    second = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    more, second_fields = attention_problems(second, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    problems += more
    reached = (first_fields.get("reminded_after_seconds"), second_fields.get("reminded_after_seconds"))
    if reached != (str(ATTENTION_RESTART_SECONDS), str(2 * ATTENTION_RESTART_SECONDS)):
        problems.append(f"重启之前应当报 reminded_after_seconds={ATTENTION_RESTART_SECONDS}，重启之后再起的 wait 不再报那一回、排满 {2 * ATTENTION_RESTART_SECONDS} 秒才报，"
                        f"实际 {reached}（重启之后那一行的依据「{second_fields.get('basis')}」）")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("报到的提醒门槛跨重启：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_RESTART_SECONDS} 秒那一回报过之后重启服务，再起的 wait 不再报那一回，排满 {2 * ATTENTION_RESTART_SECONDS} 秒才报（依据「{second_fields.get('basis')}」）"


REASON_PENDING_QUIET_SECONDS = 3.0   # attention-queue-waits-for-reason：排满提醒门槛（2 秒）之后再看这么久，排队原因一直判不出来的这段时间 wait 不该叫


def cell_attention_queue_waits_for_reason(world):
    """排队原因还没判出来的不报排队过久（task_server.py 文件头 ①「要处置的提醒」；会话 46 那一回重启之后头一轮的原因写的是「排队原因此刻读不到」）：
    本机放得下、排队线程照判不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED：派活判它放得下就不写排队原因，演服务刚重启、这一轮还没判到它），
    排满提醒门槛（2 秒）再过 3 秒，在等的 wait 一直不叫；换成本机放不下、第二台用不了，排队线程判出原因（第二台用不了）的那一轮 wait 才叫，原因不空、点名第二台。
    弄坏开关 attention-queue-reasonless-sent 下照改前照报：一到提醒门槛 wait 就叫、原因写读不到，这一格红。"""
    world.resources(local=(100, 100))
    world.seed_peak("selftest-attention-reason", 30 * GIBIBYTE)
    paused = os.path.join(world.root, "placement-paused-reason")
    open(paused, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": ATTENTION_QUEUE_SECONDS, "TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-reason"], extra={lib.KEY_VARIABLE: "selftest-attention-reason"}, cwd=world.repo)
    waiting = world.client(["wait", str(task_id)], background=True)
    time.sleep(float(ATTENTION_QUEUE_SECONDS) + REASON_PENDING_QUIET_SECONDS)
    quiet = waiting.poll() is None
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")   # 本机放不下 45G、第二台用不了：排队线程这一轮判出原因
    try:
        output, _ = world.communicate(waiting, ATTENTION_WAIT_SECONDS)
    except subprocess.TimeoutExpired:
        output, _ = world.stop_client(waiting)
    completed = subprocess.CompletedProcess(waiting.args, waiting.returncode, output, "")
    problems, fields = attention_problems(completed, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    if not quiet:
        problems.append(f"排队原因判不出来的那几轮 wait 不该叫，实际它在排满 {ATTENTION_QUEUE_SECONDS} 秒之后 {REASON_PENDING_QUIET_SECONDS} 秒里就退了"
                        f"（退 {waiting.returncode}，那一行的原因「{fields.get('why')}」）")
    elif "第二台" not in fields.get("why", "") or "读不到" in fields.get("why", ""):
        problems.append(f"判出原因那一轮报的原因应当点名「第二台」、不是读不到，实际「{fields.get('why')}」")
    os.remove(paused)
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("排队原因没判出来不报：" + "；".join(problems), world.server_log_lines())
    return f"排队原因判不出来的 {REASON_PENDING_QUIET_SECONDS} 秒里 wait 不叫，判出原因那一轮才报（原因「{fields.get('why')}」）"


def cell_attention_type_in_anomaly_table(world):
    """异常表（lib.ANOMALY_TABLE_TEXT）与代码同步：提醒的类型名、退出码、各种 kind（ATTENTION_KINDS）与那一行的标记都写在表里；退出码不与错误类型表里的码、疑似卡住的码撞。"""
    table = lib.ANOMALY_TABLE_TEXT
    taken = {code for code, _ in lib.OUTCOME_TYPES.values() if code is not None} | set(lib.EXIT_TYPES) | {lib.SUSPECT_EXIT}
    problems = []
    if lib.ATTENTION_EXIT in taken:
        problems.append(f"提醒的退出码 {lib.ATTENTION_EXIT} 不该与错误类型表、疑似卡住的码撞")
    if not re.search(rf"{re.escape(lib.ATTENTION_TYPE)}[^\n]*\b{lib.ATTENTION_EXIT}\b", table):
        problems.append(f"异常表该有一行写 {lib.ATTENTION_TYPE} 与退出码 {lib.ATTENTION_EXIT}")
    missing = [name for name in (lib.ATTENTION_MARK, *lib.ATTENTION_KINDS) if name not in table]
    if missing:
        problems.append(f"异常表该写 {missing}")
    if problems:
        raise SelftestFailure("；".join(problems))
    return f"提醒的类型 {lib.ATTENTION_TYPE}、退出码 {lib.ATTENTION_EXIT}、{len(lib.ATTENTION_KINDS)} 种 kind 与标记都在异常表里，码没和错误类型表撞"


class FakeInboxScheduler:
    """inbox-outcomes-resent-after-error 用的假调度器：只备 Scheduler.push_outcomes 读写的几样（待发的号、库里的任务、发消息、记已取）；
    库里的任务记着会话令牌（push_outcomes 只发记着令牌的，收件 socket 由 send_to_session 照会话号现找，这里的 send_to_session 是假的、不找）；
    send_to_session 对 fail_once 点名的那一条抛一次错（演发结局那一轮中途出错），别的记进 sent、交发出去了。"""

    def __init__(self, task_ids, fail_once):
        self.inbox_lock = threading.Lock()
        self.inbox_pending = set(task_ids)
        self.environment = dict(os.environ)
        self.records = {task_id: {"id": task_id, "state": "finished", "inbox": {"token": INBOX_TOKEN_SAMPLE}} for task_id in task_ids}
        self.fail_once = {fail_once}
        self.sent, self.collected = [], []
        self.store = self

    def task(self, task_id):
        return (self.records[task_id], True) if task_id in self.records else None

    def push_attention(self, task, final=False):
        return None

    def summary(self, task_id):
        return {"task": self.records[task_id]}

    def outcome_text(self, summary):
        return f"task-server：任务 {summary['task']['id']} 自证"

    def send_to_session(self, task, text, event):
        if task["id"] in self.fail_once:
            self.fail_once.discard(task["id"])
            raise RuntimeError(f"自证造的发结局出错（任务 {task['id']}）")
        self.sent.append(task["id"])
        return True

    def mark_collected(self, task_id):
        self.collected.append(task_id)
        self.records[task_id]["collected"] = True


INBOX_RESEND_TASKS = (11, 12, 13)   # inbox-outcomes-resent-after-error：同一轮待发的三条结局，发 12 时抛一次错


def cell_inbox_outcomes_resent_after_error(world):
    """发结局的那一轮中途抛错，还没发的下一轮照发（task_server.py 的 push_outcomes；只用它这一个方法与假调度器 FakeInboxScheduler，不起服务）：
    同一轮待发 11、12、13 三条，发 12 时抛一次错：这一轮把错抛给调用方（inbox_loop 打出来、下一轮再判），11 发出去记了已取、不放回，12、13 放回待发；
    下一轮 12、13 照发、记已取，11 不重发。弄坏开关 inbox-pending-dropped-on-error 下抛错时不放回，下一轮一条都不发，这一格红。"""
    scheduler = FakeInboxScheduler(INBOX_RESEND_TASKS, INBOX_RESEND_TASKS[1])
    raised = None
    try:
        SERVER.Scheduler.push_outcomes(scheduler)
    except RuntimeError as error:
        raised = error
    first_sent, first_pending = list(scheduler.sent), sorted(scheduler.inbox_pending)
    SERVER.Scheduler.push_outcomes(scheduler)
    problems = []
    if raised is None or first_sent != [INBOX_RESEND_TASKS[0]] or first_pending != list(INBOX_RESEND_TASKS[1:]):
        problems.append(f"抛错那一轮应当把错抛出来、只发出 {INBOX_RESEND_TASKS[0]}、把 {list(INBOX_RESEND_TASKS[1:])} 放回待发，"
                        f"实际{'抛了' if raised else '没抛'}、发出 {first_sent}、待发 {first_pending}")
    if scheduler.sent != list(INBOX_RESEND_TASKS) or scheduler.collected != list(INBOX_RESEND_TASKS) or scheduler.inbox_pending:
        problems.append(f"下一轮应当接着发 {list(INBOX_RESEND_TASKS[1:])}、{INBOX_RESEND_TASKS[0]} 不重发，三条各记一次已取、待发空了，"
                        f"实际先后发出 {scheduler.sent}、记已取 {scheduler.collected}、待发 {sorted(scheduler.inbox_pending)}")
    if problems:
        raise SelftestFailure("发结局中途抛错：" + "；".join(problems))
    return (f"同一轮 {list(INBOX_RESEND_TASKS)} 发 {INBOX_RESEND_TASKS[1]} 时抛错：{INBOX_RESEND_TASKS[0]} 发了不放回，"
            f"{list(INBOX_RESEND_TASKS[1:])} 放回、下一轮照发，三条各发一次")


REMINDER_RESTART_HEALTH = [("database-unwritable", "库写不进", "试写一行失败")]   # reminders-not-resent-after-restart：服务健康查出的那一样（种类、名字、问题）


def reminder_restart_scheduler(world, database_name, drifts=None):
    """reminders-not-resent-after-restart 的假调度器：restart_surviving_scheduler（同一份临时库上再建一个就是服务停了再起）再套上发消息那几路读写的几样——
    发消息记下发了什么（sent），收件人只有 SESSION_A，服务健康查出 REMINDER_RESTART_HEALTH 那一样，记账漂移判出 drifts（{(种类, 台): 条目}）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.sent = []
    scheduler.inbox_lock, scheduler.inbox_failed = threading.Lock(), set()
    scheduler.send_to_session = lambda task, text, event: scheduler.sent.append(text) or True
    scheduler.inbox_reachable = lambda task: True
    scheduler.health_recipients = lambda: {SESSION_A: {"id": 1, "session": SESSION_A}}
    scheduler.health_problems = lambda: list(REMINDER_RESTART_HEALTH)
    scheduler.booking_drift = {"memory": dict(drifts or {})}
    scheduler.booking_audit_state = {}
    scheduler.drift_recipients = lambda: {SESSION_A: {"id": 1, "session": SESSION_A}}
    return scheduler


def cell_reminders_not_resent_after_restart(world):
    """推给会话的提醒发过一回，服务停了再起（同一份库上另建调度器），同一件不再发；跨过下一道提醒门槛、好了又坏才再发（task_server.py 的 push_suspects、watch_health、push_booking_drift、
    forget_drift、persist_restart_surviving_state、restore_restart_surviving_state；任务 13493 的子任务 13522 的 suspected-stall 同一段、依据一字不差，同一天到了三回，后两回各落在服务重启之后）。
    三种去重状态原先只在内存里，现在都记进库：疑似卡住报过的段（suspect_reported）、服务健康每一样发给了哪几个会话（health_notified）、记账漂移每一件推给每个会话推到了哪道提醒门槛
    （booking_drift_notified）。其余的提醒去重状态本来就在库里：要处置的提醒报到的门槛与推过的几回在任务行（reminded_thresholds、reminder_history）、结局没人取推到第几回在任务行（outcome_repushed）、
    空转进程的报数、分片推不动的回数、停用的卡在换服务不丢的那几份；发不出去过的事件（inbox_failed）是没送到的，重启后再试不算重发。
    三趟：疑似卡住第 1 段发过一回，换服务后同一段不再发，第 2 段发；服务健康一样发过一回，换服务后不再发，好了（这一轮查不出）之后又坏再发；记账漂移判出当场推过一回，换服务后同一道门槛不再推，
    过了门槛（BOOKING_DRIFT_PUSH_SECONDS）再推一回。套在假调度器上，不起服务。
    弄坏开关 suspect-reported-memory-only、health-notified-memory-only、booking-drift-notified-memory-only 下各自那一趟换服务后又发一回，这一格红。"""
    problems = []
    store = SERVER.Store(os.path.join(world.root, "reminder-restart.sqlite3"), bytes(32), dict(os.environ))
    task = store.insert("tasks", {"task_id": 0, "state": "running", "session": SESSION_A, "command_key": "", "inbox": {"token": INBOX_TOKEN_SAMPLE}})
    attempt = store.insert("attempts", {"task_id": task["id"], "number": 1, "state": "running", "session": SESSION_A, "command_key": "", "host": SERVER.LOCAL, "log": "x.log",
                                        "suspect": {"active": True, "segment": 1, "why": "整个单元的 CPU 这 600 秒平均 0.0001 个核"}})
    del store
    before = reminder_restart_scheduler(world, "reminder-restart.sqlite3")
    SERVER.Scheduler.push_suspects(before, task)
    after = reminder_restart_scheduler(world, "reminder-restart.sqlite3")
    SERVER.Scheduler.push_suspects(after, task)
    after.store.update("attempts", dict(after.store.attempts_of(task["id"])[0][0], suspect={"active": True, "segment": 2, "why": "整个单元的 CPU 又静了 600 秒"}))
    SERVER.Scheduler.push_suspects(after, task)
    if len(before.sent) != 1 or len(after.sent) != 1 or "第 2 段" not in after.sent[0].split("\n", 1)[0]:
        problems.append(f"疑似卡住：第 1 段发过一回、换服务后同一段不再发、第 2 段发一回，实际换服务前发 {len(before.sent)} 回、换服务后发 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    before = reminder_restart_scheduler(world, "reminder-health.sqlite3")
    SERVER.Scheduler.watch_health(before)
    after = reminder_restart_scheduler(world, "reminder-health.sqlite3")
    SERVER.Scheduler.watch_health(after)
    after.health_problems = lambda: []
    SERVER.Scheduler.watch_health(after)
    after.health_problems = lambda: list(REMINDER_RESTART_HEALTH)
    SERVER.Scheduler.watch_health(after)
    if len(before.sent) != 1 or len(after.sent) != 1:
        problems.append(f"服务健康：发过一回、换服务后不再发、好了又坏再发一回，实际换服务前发 {len(before.sent)} 回、换服务后共发 {len(after.sent)} 回")
    entry = {"since": 1000.0, "rounds": 3, "booked": 0, "used": 0, "attempts": [], "text": "本机内存记账漂移"}
    before = reminder_restart_scheduler(world, "reminder-drift.sqlite3", {"本机": entry})
    SERVER.Scheduler.push_booking_drift(before, 1000.0)
    after = reminder_restart_scheduler(world, "reminder-drift.sqlite3", {"本机": entry})
    SERVER.Scheduler.push_booking_drift(after, 1000.0 + 1)
    SERVER.Scheduler.push_booking_drift(after, 1000.0 + SERVER.BOOKING_DRIFT_PUSH_SECONDS * 1.5)
    if len(before.sent) != 1 or len(after.sent) != 1 or "再推一回" not in after.sent[0].split("\n", 1)[0]:
        problems.append(f"记账漂移：判出当场推过一回、换服务后同一道门槛不再推、过了门槛再推一回（首行写再推一回），实际换服务前推 {len(before.sent)} 回、换服务后推 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    if problems:
        raise SelftestFailure("提醒换服务后不重发：" + "；".join(problems))
    return "疑似卡住的段、服务健康的会话、记账漂移推到的门槛发过一回，换服务后同一件不再发；新一段、好了又坏、过了门槛才再发"


REMINDER_LEDGER_KIND = "queued-too-long"   # reminder-thresholds-every-doubling 与 reminder-ledger-other-shape-read-afresh：照排队过久那一种算提醒门槛


def reminder_ledger_task(world, database_name, fields=None):
    """那两格的临时库里插一条排着的任务（提交它的会话是 SESSION_A），fields 并进任务行；→ 任务号。"""
    store = SERVER.Store(os.path.join(world.root, database_name), bytes(32), dict(os.environ))
    return store.insert("tasks", dict({"task_id": 0, "state": "queued", "session": SESSION_A, "command_key": ""}, **(fields or {})))["id"]


def queued_reminder_scheduler(world, database_name, waited):
    """那两格的假调度器：restart_surviving_scheduler（同一份临时库上再建一个就是服务停了再起）再备上要处置的提醒读写的两张内存表；
    attention_items 换成照 waited[0]（已排多少秒）与服务的 QUEUE_ATTENTION_MINUTES 现算的一项排队过久（没到门槛的交空表）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.reminded_thresholds, scheduler.reminder_histories = {}, {}
    first = SERVER.QUEUE_ATTENTION_MINUTES * 60

    def items(task_id):
        crossed = SERVER.reminder_threshold(waited[0], first)
        if not crossed:
            return []
        return [SERVER.Scheduler.attention_item(scheduler, REMINDER_LEDGER_KIND, crossed, task_id, f"已排 {waited[0]:.0f} 秒", "自证：一直排着", "",
                                                first * 2 ** (crossed - 1))]

    scheduler.attention_items = items
    return scheduler


def cell_reminder_thresholds_every_doubling(world):
    """提醒门槛照 QUEUE_ATTENTION_MINUTES（30 分钟）起、每翻一倍一道，同一道只报一次（task_server.py 的 reminder_threshold、next_reminder_seconds、unreported_attention；
    文件头 ①「要处置的提醒」）：已排 0、1799 秒没到门槛；1800、3599 秒跨过一道、下一回在 3600 秒；3600、7199 秒两道、下一回 7200 秒；7200 秒三道、下一回 14400 秒。
    套在假调度器上（queued_reminder_scheduler），不起服务：已排 0 到 7300 秒每隔 100 秒问一回 wait 那一侧，只在 1800、3600、7200 秒各报一回，
    报的那一项 reminded_after_seconds 依次是 1800、3600、7200，其间不重发。弄坏开关 attention-reported-every-wait 下每一回都报，这一格红。"""
    first = SERVER.QUEUE_ATTENTION_MINUTES * 60
    wanted = {0: (0, 1800), 1799: (0, 1800), 1800: (1, 3600), 3599: (1, 3600), 3600: (2, 7200), 7199: (2, 7200), 7200: (3, 14400)}
    found = {seconds: (SERVER.reminder_threshold(seconds, first), SERVER.next_reminder_seconds(SERVER.reminder_threshold(seconds, first), first)) for seconds in wanted}
    problems = [] if found == wanted else [f"从 1800 秒起每翻一倍一道门槛：{{已排秒数: (跨过几道, 下一回在几秒)}} 应当 {wanted}，实际 {found}"]
    task_id = reminder_ledger_task(world, "reminder-thresholds.sqlite3")
    waited = [0.0]
    scheduler = queued_reminder_scheduler(world, "reminder-thresholds.sqlite3", waited)
    reported = []
    for seconds in range(0, 7301, 100):
        waited[0] = float(seconds)
        reported += [(seconds, item.get("reminded_after_seconds")) for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    if reported != [(1800, 1800), (3600, 3600), (7200, 7200)]:
        problems.append(f"每隔 100 秒问一回 wait 那一侧，应当只在 1800、3600、7200 秒各报一回，实际（已排秒数, reminded_after_seconds）{reported}")
    if problems:
        raise SelftestFailure("提醒门槛每翻一倍一道：" + "；".join(problems))
    return "门槛 1800、3600、7200 秒（下一回各在 3600、7200、14400 秒）；问到 7300 秒只在这三处各报一回"


def cell_reminder_ledger_other_shape_read_afresh(world):
    """任务行里的提醒记账不是这一版的两栏（task_server.py 的 stored_reminded_thresholds、reminder_history_of、attention_round、unreported_attention；文件头 ①「要处置的提醒」）：
    任务行只有别的写法记的账（attention_reported 记着种类到 2、attention_history 里一回带 tier 键），没有 reminded_thresholds 与 reminder_history；同一份临时库上建假调度器
    （queued_reminder_scheduler，就是服务换上新代码），不起服务：别的写法的账不读、当没报过——已排 3000 秒问 wait 那一侧报一回（reminded_after_seconds=1800），回数从第 1 回起；
    再问不报；排到 3600 秒报一回（reminded_after_seconds=3600）；任务行写下 reminded_thresholds 记着 2；同一份库上再建一个（服务再重启）问 3600 秒不报。问答都不抛错。
    弄坏开关 attention-reported-every-wait 下再问照报，attention-reported-memory-only 下再重启之后照报，这一格红。"""
    other_shape = {"attention_reported": {REMINDER_LEDGER_KIND: 2},
                   "attention_history": {REMINDER_LEDGER_KIND: [{"tier": 2, "repush": 0, "at": time.time() - 600, "basis": "已排 3700 秒", "round": 2}]}}
    task_id = reminder_ledger_task(world, "reminder-ledger.sqlite3", other_shape)
    waited = [3000.0]
    scheduler = queued_reminder_scheduler(world, "reminder-ledger.sqlite3", waited)
    problems = []
    rounds = SERVER.Scheduler.attention_round(scheduler, scheduler.attention_items(task_id)[0])
    asked = [item.get("reminded_after_seconds") for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    again = SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)
    waited[0] = 3600.0
    later = [item.get("reminded_after_seconds") for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    if rounds != (1, None) or asked != [1800] or again or later != [3600]:
        problems.append(f"别的写法的账不读：已排 3000 秒应当第 1 回、报一回 reminded_after_seconds=1800，再问不报，3600 秒报一回 3600；"
                        f"实际（回数, 上一回）{rounds}、头一问 {asked}、再问 {len(again)} 项、3600 秒 {later}")
    found = scheduler.store.task(task_id)
    stored = (found[0] if found is not None and found[1] else {}).get("reminded_thresholds")
    if stored != {REMINDER_LEDGER_KIND: 2}:
        problems.append(f"报过之后任务行应当写下 reminded_thresholds {{{REMINDER_LEDGER_KIND}: 2}}，实际 {stored}")
    restarted = queued_reminder_scheduler(world, "reminder-ledger.sqlite3", waited)
    if SERVER.Scheduler.unreported_attention(restarted, task_id, SESSION_A):
        problems.append("同一份库上再建调度器（服务再重启），已排 3600 秒那一回报过了不该再报，实际又报了")
    if problems:
        raise SelftestFailure("提醒记账不是这一版的两栏：" + "；".join(problems))
    return "别的写法的账不读、当没报过：3000 秒报一回（第 1 回，reminded_after_seconds=1800）、再问不报、3600 秒报一回；任务行写下 reminded_thresholds，再重启不重报"


# previous-swap-running-reminded：草稿目录被占那一项的出路里要有的一句（干跑不占这个仓、真换先占这个仓：换 --scratch 当且仅当两趟里至少有一趟是 --dry-run 时有用，
# 被挡的这一条自己是干跑的同样算）
SWAP_BUSY_SCRATCH_ADVICE = "这一种等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch"


def swap_busy_log_lines(world):
    """scripts/gate/hook-swap.py 退 10 的两种拒绝说明的现行原文（hook_swap_swap.py 的 scratch_busy_lines 与 repository_busy_lines 各出的两行；开跑时刻照此刻现算）。
    → (草稿目录被占的两行, 这个仓的换上被占的两行)。"""
    scratch = os.path.join(world.root, "swap-scratch")
    started = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(time.time() - 12))
    who = f"进程 549548，这一台，主机 sample-host，开跑于 {started}（已 12 秒），占用记录 3 秒前刷新过"
    scratch_busy = [
        f"✗ 草稿目录 {scratch} 正被另一趟 hook-swap.py 占着（{who}）：草稿树 {os.path.join(scratch, 'hook-swap', 'tree')} 只有一棵，并行起会互相删掉、盖掉；这一趟没建树、没写文件",
        "→ 怎么办：等那一趟结束再交，或换一个 --scratch（--dry-run 要并行的几条各给一个草稿目录；真换在同一个仓里一次只许一趟，另一趟真换在跑时换草稿目录也会被拒）"]
    repository_busy = [
        f"✗ 这个仓的换上正被另一趟 hook-swap.py 占着（进程 549548，这一台，主机 sample-host，用的草稿目录 {scratch}，开跑于 {started}（已 12 秒），占用记录 3 秒前刷新过）："
        "结局文件 .git/singlefs-hook-swap-outcome.tsv 按仓只有一份，真换与 --seal-outcome 一次只许一趟；这一趟没读结局文件、没建树、没写文件",
        "→ 怎么办：等那一趟结束再交（换草稿目录没有用；--dry-run 不占这一道，各给一个 --scratch 可以并行）"]
    return scratch_busy, repository_busy


def swap_busy_problems(world, fake, root, swap):
    """previous-swap-running-reminded 的退 10 一段：服务照这一条日志里的拒绝说明认占着的是哪一种（swap_busy_cause），出路只写对这一种成立的。
    草稿目录被占：依据点名草稿目录，出路写等占着的那一趟结束再交、两趟里有一趟是 --dry-run 的可以换一个 --scratch（SWAP_BUSY_SCRATCH_ADVICE）；这个仓的换上被占：依据点名这个仓的换上，出路写换草稿目录没有用、整条不出现 --scratch；
    读不到日志（没有尝试、日志文件不在）、日志里两种拒绝说明都没有：依据写两种都可能，出路叫照日志里的拒绝说明办、不出现 --scratch；两种都出现的取最后出现的。→ 问题列表。"""
    scratch_busy, repository_busy = swap_busy_log_lines(world)
    noise = ["python3 scripts/gate/hook-swap.py --spec new.json --scratch swap-scratch", "草稿目录核过不在主工作区里"]
    advice = lib.anomaly_advice("previous-swap-running", 0, fake.environment)
    problems = [] if "--scratch" not in advice else [f"异常表那一行的出路是退 9、退 10 共用的一句，不该点名 --scratch（只对一种成立），实际「{advice}」"]

    def judged(task_id, name, lines, attempted=True):
        path = os.path.join(world.root, f"swap-busy-{name}.log")
        if lines is not None:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("\n".join(lines) + "\n")
        attempts = [({"number": 1, "state": "ended", "log": path}, True)] if attempted else []
        fake.store = types.SimpleNamespace(attempts_of=lambda _task_id: attempts, select=lambda table, where="1", parameters=(), order="id": [])
        found = dict({"id": task_id, "outcome": "command-failed", "exit_code": 10, "command": swap}, state="finished", cwd=root)
        items = {item["kind"]: item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time())}
        return items.get("previous-swap-running"), sorted(items)

    for what, task_id, name, lines, attempted, wanted in (
            ("草稿目录被占的日志", 931, "scratch", noise + scratch_busy, True, "scratch"),
            ("这个仓的换上被占的日志", 932, "repository", noise + repository_busy, True, "repository"),
            ("两种都出现、草稿目录那一种在后", 933, "scratch-last", repository_busy + scratch_busy, True, "scratch"),
            ("两种都出现、这个仓的换上那一种在后", 934, "repository-last", scratch_busy + repository_busy, True, "repository"),
            ("日志里没有这两种拒绝说明", 935, "unrecognized", noise + ["✗ 别的错误：与占用无关"], True, None),
            ("日志文件不在", 936, "missing", None, True, None),
            ("没有尝试", 937, "none", None, False, None)):
        item, kinds = judged(task_id, name, lines, attempted)
        table_advice = lib.anomaly_advice("previous-swap-running", task_id, fake.environment)
        log_pointer = f"task-server/task log {task_id}"
        if item is None or item["threshold"] != 1 or "退 10" not in item["basis"] or not item["next"].startswith(f"{table_advice}；") or log_pointer not in item["next"] or "workspace-write-blocked" in kinds:
            problems.append(f"{what}：应当只出一项 previous-swap-running（门槛 1、依据写退 10、出路以异常表那一行起头、后面接着点名 {log_pointer}），实际 {kinds}、{item}")
            continue
        if wanted == "scratch" and not ("草稿目录" in item["basis"] and SWAP_BUSY_SCRATCH_ADVICE in item["next"] and "没有用" not in item["next"]):
            problems.append(f"{what}：应当认成草稿目录被占（依据点名草稿目录，出路写「{SWAP_BUSY_SCRATCH_ADVICE}」），实际依据「{item['basis']}」、出路「{item['next']}」")
        if wanted == "repository" and not ("这个仓的换上正被" in item["basis"] and "换草稿目录没有用" in item["next"] and "--scratch" not in item["next"]):
            problems.append(f"{what}：应当认成这个仓的换上被占（依据点名这个仓的换上，出路写换草稿目录没有用、不出现 --scratch），实际依据「{item['basis']}」、出路「{item['next']}」")
        if wanted is None and not ("两种都可能" in item["basis"] and "拒绝说明" in item["next"] and "--scratch" not in item["next"] and "没有用" not in item["next"]):
            problems.append(f"{what}：应当写两种都可能（依据写两种都可能，出路叫照日志里的拒绝说明办、不出现 --scratch、不断言换草稿目录没有用），实际依据「{item['basis']}」、出路「{item['next']}」")
    return problems


def cell_previous_swap_running_reminded(world):
    """换钩子时上一趟还在跑或同一个 --scratch 被占着，提醒发给被挡的这个会话（task_server.py 的 attention_of_record、previous_swap_running_items；scripts/gate/hook-swap.py 退 9 与退 10）：
    出了结局的任务命令是 scripts/gate/hook-swap.py（不带 --feed-only）、结局 command-failed 退 9：出一项 previous-swap-running，出路以异常表那一行起头（退 9 的等那一趟写完成再原样重交），
    后面接这一条日志的指引（attention_item 把 follow 接在异常表那一句后面，点名 task-server/task log <这一条的号>）；不出 workspace-write-blocked；退 7、退 8 的、带 --feed-only 的、命令是别的脚本的不出这一种。
    退 10（同一个 --scratch 或这个仓的换上正被另一趟占着）同样出一项 previous-swap-running：依据写退 10，出路以异常表那一行（退 9、退 10 分开写，不点名 --scratch）起头，
    后面照这一条日志里的拒绝说明认占着的是哪一种（swap_busy_problems，日志样本照 hook-swap.py 两种拒绝说明的现行原文）：草稿目录被占的写等占着的那一趟结束再交、
    两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是）可以换一个 --scratch（SWAP_BUSY_SCRATCH_ADVICE），这个仓的换上被占的写换草稿目录没有用、不出现 --scratch，读不到日志或两种都认不出的写两种都可能、不给换草稿目录这条出路。
    弄坏开关 previous-swap-running-unreported 下一条都不出，swap-scratch-busy-unreported 下退 10 的不出，swap-busy-cause-unread 下退 10 的一律按草稿目录被占写，
    swap-busy-unknown-guessed 下认不出的按草稿目录被占写，这一格红。"""
    fake, root = long_run_split_scheduler(world)
    swap = ["python3", "scripts/gate/hook-swap.py", "--spec", "new.json"]
    problems = []

    def kinds_of(record):
        found = dict(record, state="finished", cwd=root)
        fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [], select=lambda table, where="1", parameters=(), order="id": [])
        return {item["kind"]: item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time())}

    items = kinds_of({"id": 920, "outcome": "command-failed", "exit_code": 9, "command": swap})
    item = items.get("previous-swap-running")
    advice = lib.anomaly_advice("previous-swap-running", 920, fake.environment)
    if (item is None or item["threshold"] != 1 or "退 9" not in item["basis"] or not item["next"].startswith(f"{advice}；")
            or "task-server/task log 920" not in item["next"] or "workspace-write-blocked" in items):
        problems.append(f"换钩子退 9：应当只出一项 previous-swap-running（门槛 1、依据写退 9、出路以异常表那一行「{advice[:60]}…」起头、后面接着点名 task-server/task log 920），实际 {sorted(items)}、{item}")
    problems += swap_busy_problems(world, fake, root, swap)
    for what, record in (("退 7", {"id": 921, "outcome": "command-failed", "exit_code": 7, "command": swap}),
                         ("退 8", {"id": 925, "outcome": "command-failed", "exit_code": 8, "command": swap}),
                         ("带 --feed-only", {"id": 922, "outcome": "command-failed", "exit_code": 9, "command": swap + ["--feed-only"]}),
                         ("退 10 带 --feed-only", {"id": 926, "outcome": "command-failed", "exit_code": 10, "command": swap + ["--feed-only"]}),
                         ("命令是别的脚本", {"id": 923, "outcome": "command-failed", "exit_code": 9, "command": ["python3", "scripts/gate/other.py"]}),
                         ("退 10 而命令是别的脚本", {"id": 927, "outcome": "command-failed", "exit_code": 10, "command": ["python3", "scripts/gate/other.py"]})):
        if "previous-swap-running" in kinds_of(record):
            problems.append(f"{what}：不该出 previous-swap-running")
    if problems:
        raise SelftestFailure("换钩子时上一趟还在跑或草稿目录被占着：" + "；".join(problems))
    return ("换钩子退 9、退 10 各出一项 previous-swap-running；退 9 的出路是等那一趟写完成再原样重交，退 10 的照日志里的拒绝说明分草稿目录被占（两趟里有一趟是 --dry-run 的才可换 --scratch）、这个仓的换上被占（换草稿目录没有用）、"
            "认不出（两种都可能），两种都出现的取最后出现的；退 7、退 8、--feed-only、别的脚本不出这一种")


def cell_attention_line_why_clipped(world):
    """提醒那一行的 why= 超过 ATTENTION_WHY_LIMIT 个字符的按字符截到那么长、尾巴接「…」，不超的原样（task_server_lib.py 的 attention_line）：
    why 写 ATTENTION_WHY_LIMIT 加一百个字符的，那一行拆回来的 why= 恰好 ATTENTION_WHY_LIMIT 个字符、以「…」收尾、前面是原文的前 ATTENTION_WHY_LIMIT 减一个字符；
    恰好 ATTENTION_WHY_LIMIT 个字符的与短的原样。弄坏开关 attention-why-unclipped 下照全文写，这一格红。"""
    limit = lib.ATTENTION_WHY_LIMIT
    long_why = "".join(str(position % 10) for position in range(limit + 100))
    problems = []
    for what, why, wanted in (("超过上限", long_why, long_why[:limit - 1] + "…"), ("恰好上限", long_why[:limit], long_why[:limit]), ("很短", "排队过久", "排队过久")):
        line = lib.attention_line(7, {"kind": "queued-too-long", "basis": "依据", "why": why, "next": "出路"})
        values = dict(word.split("=", 1) for word in shlex.split(line) if "=" in word)
        if values.get("why") != wanted:
            problems.append(f"{what}：why= 应当是 {len(wanted)} 个字符（{'截断接「…」' if wanted != why else '原样'}），实际 {len(values.get('why') or '')} 个字符")
    if problems:
        raise SelftestFailure("提醒那一行的 why= 截断：" + "；".join(problems))
    return f"why= 超过 {limit} 个字符的截到 {limit} 个字符接「…」，恰好 {limit} 个与短的原样"


# gpu-cases-not-run-reported：标记行点名的两条用例（singlefs_verification 里打这一行的两处各取一个名字）
GPU_NOT_RUN_SAMPLE_TESTS = ("given_states_engine::given-states-engine-without-the-gpu-feature", "crash_state_enumeration::some_crash_enumeration_item")


def gpu_not_run_judged(world, name, chunks, record=None, earlier=()):
    """gpu-cases-not-run-reported 的一回：一次尝试的日志照 chunks 分几回写（每写一份经真的 log_content 读一回），record_gpu_not_run_marks 记进尝试，
    这一次记成 earlier 之后的那一次、ended；再问这条出了结局的任务（record，默认退 0 的整条任务）。→ (结局说明前面那几句, gpu-cases-not-run 那一项或 None)。"""
    fake, attempt = peer_log_scheduler(world, f"gpu-not-run-{name}", chunks[0])
    for lines in chunks[1:]:
        SERVER.Scheduler.log_content(fake, attempt)
        append_lines(attempt["log"], lines)
    SERVER.Scheduler.log_content(fake, attempt)
    SERVER.Scheduler.record_gpu_not_run_marks(fake, attempt)
    ended = dict(attempt, state="ended", number=len(earlier) + 1, outcome=(record or {}).get("outcome", "succeeded"))
    found = dict({"id": 77, "outcome": "succeeded", "exit_code": 0, "command": ["cargo", "test", "-p", "singlefs-verification"]}, **(record or {}), state="finished")
    fake.peer_root, fake.budget_notices, fake.idle_short_notices, fake.workspace_write_checks = world.root, {}, {}, {}
    fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [(dict(item), True) for item in (*earlier, ended)])
    preface = SERVER.Scheduler.finished_preface(fake, found, ended, found["outcome"])
    items = [item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time()) if item["kind"] == "gpu-cases-not-run"]
    return preface, (items[0] if items else None)


def cell_gpu_cases_not_run_reported(world):
    """日志里用例「本次未跑」的标记行由服务读出来写进结局（task_server.py 的 gpu_not_run_test、read_log_content、record_gpu_not_run_marks、finished_preface、
    gpu_cases_not_run_items；singlefs_verification::gpu_memory_ledger::gpu_not_run_line 打的 `NOT_RUN_NO_GPU_ADAPTER test=<用例> reason=<为什么>`）：
    退 0 的整条任务日志里两行行首是标记的（分两回读到），结局说明报 2 条、两条都点名，出一项 gpu-cases-not-run（提醒门槛 1、依据报 2 条并点名、出路写不算过与带 gpu 特性重新提交）；
    同一份日志里记号后面接着别的字的一行（更长的词）不算（哪些位置与边界认、哪些不认归 gpu-not-run-line-forms 那一格）；超过 GPU_NOT_RUN_TESTS_SHOWN 条的只点名前几条、余下报条数；
    没有标记的尝试结局说明是空的、不出；分片的两片按片各判（有标记的那一片出、依据写第几片，没有的那一片不出）；前一次尝试有标记、最后一次没有的不出；取消的不出。
    经真的 log_content 读日志，套在假调度器上，不起服务。弄坏开关 gpu-cases-not-run-unread 下标记行不认、一条都不报，gpu-not-run-word-edges-unchecked 下更长的词那一行也算进去，这一格红。"""
    first, second = GPU_NOT_RUN_SAMPLE_TESTS
    marked = [["running 5 tests", f"NOT_RUN_NO_GPU_ADAPTER test={first} reason=不带 gpu 特性而配置开了 GPU", "test given_states ... ok"],
              [f"NOT_RUN_NO_GPU_ADAPTER test={second} reason=显存不够：借不到", "NOT_RUN_NO_GPU_ADAPTER_X test=longer-word-case reason=记号是更长的词",
               "test result: ok. 5 passed; 0 failed"]]
    problems = []
    preface, item = gpu_not_run_judged(world, "two", marked)
    if not ("这一趟有 2 条用例本次未跑" in preface and first in preface and second in preface and "不算过" in preface):
        problems.append(f"两行标记：结局说明应当报 2 条、点名 {first} 与 {second}、写不算过，实际「{preface}」")
    if "longer-word-case" in preface:
        problems.append(f"记号后面接着别的字的那一行（更长的词）不该算，实际「{preface}」")
    if (item is None or item["threshold"] != 1 or "2 条用例本次未跑" not in item["basis"] or first not in item["basis"] or second not in item["basis"]
            or "不算过" not in item["next"] or "gpu 特性" not in item["next"]):
        problems.append(f"两行标记：应当出一项 gpu-cases-not-run（门槛 1、依据报 2 条并点名、出路写不算过与带 gpu 特性重新提交），实际 {item}")
    many = [f"NOT_RUN_NO_GPU_ADAPTER test=many-case-{index} reason=不带 gpu 特性" for index in range(SERVER.GPU_NOT_RUN_TESTS_SHOWN + 3)]
    preface, _ = gpu_not_run_judged(world, "many", [many])
    last_shown, first_hidden = f"many-case-{SERVER.GPU_NOT_RUN_TESTS_SHOWN - 1}", f"many-case-{SERVER.GPU_NOT_RUN_TESTS_SHOWN}"
    if f"有 {SERVER.GPU_NOT_RUN_TESTS_SHOWN + 3} 条" not in preface or last_shown not in preface or first_hidden in preface or "另 3 条只报条数" not in preface:
        problems.append(f"{SERVER.GPU_NOT_RUN_TESTS_SHOWN + 3} 行标记：应当报 {SERVER.GPU_NOT_RUN_TESTS_SHOWN + 3} 条、点名到 {last_shown}、另 3 条只报条数，实际「{preface}」")
    preface, item = gpu_not_run_judged(world, "none", [["running 1 test", "test plain ... ok", "test result: ok. 1 passed"]])
    if preface or item is not None:
        problems.append(f"没有标记的尝试：结局说明应当是空的、不出，实际「{preface}」、{item}")
    shard = {"id": 702, "parent_id": 700, "role": "shard", "shard_index": 2}
    _, marked_shard = gpu_not_run_judged(world, "shard-marked", marked, dict(shard))
    _, clean_shard = gpu_not_run_judged(world, "shard-clean", [["test plain ... ok"]], dict(shard, id=703, shard_index=3))
    if marked_shard is None or "第 2 片" not in marked_shard["basis"] or clean_shard is not None:
        problems.append(f"分片的两片：有标记的第 2 片应当出、依据写第 2 片，没有的第 3 片不出，实际 {marked_shard}、{clean_shard}")
    earlier = ({"number": 1, "state": "ended", "outcome": "memory-cap-hit", SERVER.NOT_RUN_MARKS_COLUMN: {"count": 1, "tests": [first]}},)
    _, retried = gpu_not_run_judged(world, "retried-clean", [["test plain ... ok"]], earlier=earlier)
    if retried is not None:
        problems.append(f"前一次撞上限的尝试有标记、最后一次没有：不该出，实际 {retried}")
    preface, cancelled = gpu_not_run_judged(world, "cancelled", marked, {"outcome": "cancelled", "exit_code": None})
    if preface or cancelled is not None:
        problems.append(f"取消的任务：结局说明不写、不出，实际「{preface}」、{cancelled}")
    if problems:
        raise SelftestFailure("日志里的本次未跑标记写进结局：" + "；".join(problems))
    return (f"两行标记报 2 条并点名、出 gpu-cases-not-run；记号后面接着别的字的一行不算；{SERVER.GPU_NOT_RUN_TESTS_SHOWN + 3} 行的点名前 {SERVER.GPU_NOT_RUN_TESTS_SHOWN} 条、余下报条数；"
            "没有标记的说明不变、不出；分片按片各判；最后一次没有标记的、取消的不出")


# 带 --test-threads 1 --nocapture 的 cargo test 打出来的原样一行：「test <名> ... 」起头，后面直接接记号，test= 点名的用例与行首那个名字不是同一个
GPU_NOT_RUN_LINE_AFTER_TEST_START = ("test a_file_read_through_the_session_counts_the_damaged_copy_and_the_next_change_relocates_the_unit ... NOT_RUN_NO_GPU_ADAPTER "
                                     "test=given_states_engine::unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired::file-read-relocated "
                                     "reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（录入段照录、写进库），带 gpu 特性那一趟从库里读回在卡上判（cargo … --features gpu）")


def gate_library_marker_sample():
    """门禁那一份库（scripts/gate/lib_libtest_result_lines.py）里的记号样本日志与它点名的七条用例：按文件路径导入、取同一份内容，两边的自证对同一份日志认同样的行。"""
    import importlib.util
    path = os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "lib_libtest_result_lines.py")
    specification = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_the_marker_sample", path)
    gate_library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(gate_library)
    return gate_library.NOT_RUN_MARKER_SAMPLE_LOG, gate_library.NOT_RUN_MARKER_SAMPLE_TESTS


def cell_gpu_not_run_line_forms(world):
    """日志里「本次未跑」标记行的认法（task_server.py 的 gpu_not_run_test）：NOT_RUN_NO_GPU_ADAPTER 整词、行内任一处都认——前面是行首、空白或别的非标识符字符，后面是空白（含制表符）或行尾，
    ANSI 颜色码先去掉，点名的用例取记号行自己 `test=` 的那一段、不取行首那个测试名；更长或更短的词不认。
    喂三份日志，都经真的 log_content 读、套在假调度器上、不起服务：
    门禁那一份库的样本日志（gate_library_marker_sample：行首、行首带空白、接在结果行起头后面、接在别的输出后面、前面有颜色码、结果行起头与记号之间隔着字、后面接制表符七种，
    另有一行 CANNOT_RUN_ANYTHING_HERE）分两回写：认 7 条、按 test= 逐条点名、不认那一行；一行带 `test <名> ... ` 起头而 test= 点名另一条的真日志原样：认 1 条、点名取 test= 那一个；
    前面紧贴字母、下划线、数字，后面紧贴别的字、冒号、右括号的，更短的词、别的名字、小写的一共 9 行不认，前面是非 ASCII 字或左括号、没写 test= 的、后面紧贴颜色码的 4 行认。
    弄坏开关 gpu-not-run-line-start-only 下只认行首，gpu-not-run-word-edges-unchecked 下不判两侧词界，gpu-not-run-color-codes-kept 下不先去颜色码，这一格红。"""
    sample_log, sample_tests = gate_library_marker_sample()
    sample_lines = sample_log.removesuffix("\n").split("\n")
    middle = len(sample_lines) // 2
    problems = []
    preface, item = gpu_not_run_judged(world, "forms", [sample_lines[:middle], sample_lines[middle:]])
    if f"这一趟有 {len(sample_tests)} 条用例本次未跑" not in preface or "、".join(sample_tests) not in preface:
        problems.append(f"样本日志里七种形态共 {len(sample_tests)} 条应当都算、按出现次序点名 {'、'.join(sample_tests)}，实际「{preface}」")
    if SERVER.GPU_NOT_RUN_UNNAMED in preface:
        problems.append(f"样本日志里 CANNOT_RUN_ANYTHING_HERE 那一行（更长的词）不该算，实际「{preface}」")
    if item is None or f"{len(sample_tests)} 条用例本次未跑" not in item["basis"]:
        problems.append(f"样本日志：gpu-cases-not-run 的依据应当报 {len(sample_tests)} 条，实际 {item}")
    real_name = "given_states_engine::unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired::file-read-relocated"
    preface, _ = gpu_not_run_judged(world, "forms-real-line", [["running 1 test", GPU_NOT_RUN_LINE_AFTER_TEST_START, "ok"]])
    if "这一趟有 1 条用例本次未跑" not in preface or real_name not in preface or "a_file_read_through_the_session" in preface:
        problems.append(f"带 test <名> ... 起头的真日志那一行应当算 1 条、点名取 test= 写的 {real_name}、不取行首的测试名，实际「{preface}」")
    refused = ("PRENOT_RUN_NO_GPU_ADAPTER test=refused-left-letters reason=前面紧贴字母", "x_NOT_RUN_NO_GPU_ADAPTER test=refused-left-underscore reason=前面紧贴下划线",
               "9NOT_RUN_NO_GPU_ADAPTER test=refused-left-digit reason=前面紧贴数字", "NOT_RUN_NO_GPU_ADAPTER_X test=refused-right-longer reason=后面接着别的字",
               "NOT_RUN_NO_GPU_ADAPTER:test=refused-right-colon reason=后面紧贴冒号", "（NOT_RUN_NO_GPU_ADAPTER） test=refused-right-bracket reason=后面紧贴右括号",
               "NOT_RUN_NO_GPU test=refused-shorter reason=更短的词", "NOT_RUN_ANYTHING_HERE test=refused-other-name reason=别的名字",
               "not_run_no_gpu_adapter test=refused-lowercase reason=小写")
    accepted = ("[NOT_RUN_NO_GPU_ADAPTER test=accepted-left-bracket reason=前面是左括号", "卡NOT_RUN_NO_GPU_ADAPTER test=accepted-left-non-ascii reason=前面是非 ASCII 的字",
                "NOT_RUN_NO_GPU_ADAPTER", "NOT_RUN_NO_GPU_ADAPTER\x1b[0m test=accepted-color-after reason=后面紧贴颜色码")
    preface, _ = gpu_not_run_judged(world, "forms-edges", [["running 13 tests", *refused, *accepted, "test result: ok. 13 passed; 0 failed"]])
    if "这一趟有 4 条用例本次未跑" not in preface or any(name not in preface for name in ("accepted-left-bracket", "accepted-left-non-ascii", "accepted-color-after", SERVER.GPU_NOT_RUN_UNNAMED)):
        problems.append(f"前面是非 ASCII 字或左括号、没写 test= 的、后面紧贴颜色码的 4 行应当算（没写 test= 的写「{SERVER.GPU_NOT_RUN_UNNAMED}」），实际「{preface}」")
    if "refused-" in preface:
        problems.append(f"前后紧贴别的字的、更短的词、别的名字、小写的 9 行不该算，实际「{preface}」")
    if problems:
        raise SelftestFailure("本次未跑标记行的认法：" + "；".join(problems))
    return (f"样本日志七种形态 {len(sample_tests)} 条都认、点名取 test= 那一段、CANNOT_RUN_ 那一行不认；带 test <名> ... 起头的真日志那一行点名取 test=；"
            "前后紧贴别的字的、更短的词、别的名字、小写的 9 行不认，前面是非 ASCII 字或左括号、没写 test= 的、后面紧贴颜色码的 4 行认")


def cell_gpu_not_run_marks_per_line(world):
    """一行里有几处记号就认几处、各点各自那一截里 test= 的名字（task_server.py 的 gpu_not_run_test、read_log_content；一处的范围从这一处记号起到同一行下一处 NOT_RUN_ 之前，
    门禁那一份库的 marker_pieces）：两条用例的输出挤在一行（--nocapture、几条同时跑时出得来），两处记号都数、各点各的名；
    同一行里夹着别的 NOT_RUN_<原因>（库认的一族，服务只认 NOT_RUN_NO_GPU_ADAPTER 一个名字）的，别的名字只划截、不算一条，排在它后面的那一处照数；没写 test= 的那一处记 GPU_NOT_RUN_UNNAMED、
    不取同一行别处的 test=。喂两份日志，都经真的 log_content 读、套在假调度器上、不起服务：两处记号挤在一行的日志（另有一行是库一族里别的名字：库认 3 处，服务认 2 处）；
    夹着别的名字的日志（一行夹着别的名字的两处、一行里有 test= 的一处后面接着没写 test= 的一处、一行从行中间起头的两处，后一处前面紧贴非 ASCII 字），共 6 条。
    弄坏开关 gpu-not-run-first-mark-only 下一行只算先出现的那一处，这一格红；gpu-not-run-line-start-only 下行中间起头的那一行不认，这一格红；gpu-cases-not-run-unread 下一条都不认，这一格也红。"""
    same_line = ["NOT_RUN_NO_GPU_ADAPTER test=gpu_case_alpha reason=样本：同一行里先打的记号 NOT_RUN_NO_GPU_ADAPTER test=gpu_case_beta reason=样本：同一行里后打的记号",
                 "NOT_RUN_OUT_OF_GPU_MEMORY test=gpu_case_gamma reason=样本：同一族的另一个原因"]
    squeezed = ["NOT_RUN_NO_GPU_ADAPTER test=between_before reason=样本 NOT_RUN_OUT_OF_GPU_MEMORY test=between_other reason=夹在中间的别的名字 NOT_RUN_NO_GPU_ADAPTER test=between_after reason=样本",
                "NOT_RUN_NO_GPU_ADAPTER test=named_then_bare reason=样本 NOT_RUN_NO_GPU_ADAPTER",
                "进度 42% NOT_RUN_NO_GPU_ADAPTER test=mid_line_before reason=样本：行中间 卡NOT_RUN_NO_GPU_ADAPTER test=mid_line_after reason=样本：前面紧贴非 ASCII 字"]
    problems = []
    library_pieces = sum(len(SERVER.gate_marker_library.marker_pieces(line)) for line in same_line)
    if library_pieces != 3:
        problems.append(f"两处记号挤在一行的日志：门禁那一份库应当认 3 处记号（一行两处、另一行一处别的名字），实际 {library_pieces}")
    preface, item = gpu_not_run_judged(world, "marks-same-line", [["running 3 tests", *same_line, "test result: ok. 3 passed; 0 failed"]])
    if "这一趟有 2 条用例本次未跑" not in preface or "gpu_case_alpha、gpu_case_beta" not in preface or "gpu_case_gamma" in preface or "日志里 2 处 NOT_RUN_NO_GPU_ADAPTER 记号" not in preface:
        problems.append(f"两处记号挤在一行的日志：应当认 2 条、按出现的次序点名 gpu_case_alpha 与 gpu_case_beta、不点名别的名字那一行的 gpu_case_gamma、写 2 处记号，实际「{preface}」")
    if item is None or "2 条用例本次未跑" not in item["basis"] or "gpu_case_beta" not in item["basis"]:
        problems.append(f"两处记号挤在一行的日志：gpu-cases-not-run 的依据应当报 2 条并点名 gpu_case_beta，实际 {item}")
    preface, _ = gpu_not_run_judged(world, "marks-squeezed", [["running 3 tests", *squeezed, "test result: ok. 3 passed; 0 failed"]])
    named = "、".join(("between_before", "between_after", "named_then_bare", SERVER.GPU_NOT_RUN_UNNAMED, "mid_line_before", "mid_line_after"))
    if "这一趟有 6 条用例本次未跑" not in preface or named not in preface or "between_other" in preface:
        problems.append(f"夹着别的名字的日志：应当认 6 条、按出现的次序点名 {named}（夹在中间的别的名字不算一条、没写 test= 的那一处不取同一行别处的 test=），实际「{preface}」")
    if problems:
        raise SelftestFailure("一行里的几处本次未跑记号：" + "；".join(problems))
    return ("一行两处记号认 2 条（库认 3 处、服务只认自己那个名字的）、各点各的 test=；夹着别的名字的三处、行中间起头两处、后一处没写 test= 的共 6 条，别的名字只划截、不算一条")


# ── 异常表、再推、按会话号找收件 socket、交任务的二次确认、收尾协议单测（task_server_lib.py 文件头「异常表」；task_server.py 文件头 ⑥ ⑦ ②「内存」）──
ADVICE_SAMPLE_TASK = 4242          # anomaly-advice-in-texts 拼正文用的任务号（不进库）

UNCLAIMED_REPUSH_SAMPLE_SECONDS = "2"   # unclaimed-outcome-repushed：结局没人取再推的门槛（秒；TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS）

CLEANUP_PROTOCOL_TESTS = 25   # task-server/tests/test_cleanup_protocol.py 的单测条数：跑到的少了（漏载、跳过）也判红；那份文件加减单测时一起改这个数


def cell_anomaly_table_matches_kinds(world):
    """异常表（lib.ANOMALY_TABLE_TEXT）与代码里分别处置的种类逐行对得上：ATTENTION_KINDS 每一种是「提醒照跑」（BOUNCING_ATTENTION_KINDS 那几种是「打回」），SUBMIT_REFUSAL_KINDS 是「提交时拒」，
    SUBMIT_WARNING_KINDS 是「提醒照跑」，错误类型表（OUTCOME_TYPES）里 succeeded 之外每一种是「打回」或「服务自己修」，SERVICE_ANOMALY_KINDS 是「提醒照跑」或「服务自己修」；
    表里不多不少、种类不重、动作在 ANOMALY_ACTIONS 里、修改意见不是那一句通用的；task_server.py 的 ANOMALY_REPUSHED_KINDS 都是表里的提醒。静态核，不起服务。"""
    rows = lib.anomaly_rows(lib.ANOMALY_TABLE_TEXT)
    table = {row.kind: row for row in rows}
    expected = {**{kind: ("提醒照跑",) for kind in lib.ATTENTION_KINDS + lib.SUBMIT_WARNING_KINDS}, **{kind: ("打回",) for kind in lib.BOUNCING_ATTENTION_KINDS}, **{kind: ("提交时拒",) for kind in lib.SUBMIT_REFUSAL_KINDS},
                **{kind: ("打回", "服务自己修") for kind in lib.OUTCOME_TYPES if kind != "succeeded"},
                **{kind: ("提醒照跑", "服务自己修") for kind in lib.SERVICE_ANOMALY_KINDS}}
    problems = []
    duplicated = sorted({row.kind for row in rows if [other.kind for other in rows].count(row.kind) > 1})
    missing, extra = sorted(set(expected) - set(table)), sorted(set(table) - set(expected))
    wrong = sorted(f"{kind}（{table[kind].action}）" for kind in set(expected) & set(table) if table[kind].action not in expected[kind])
    unknown = sorted(row.kind for row in rows if row.action not in lib.ANOMALY_ACTIONS)
    generic = sorted(row.kind for row in rows if row.advice == lib.ANOMALY_GENERIC_ADVICE or "改小命令重交" in row.advice)
    unrepushable = sorted(set(SERVER.ANOMALY_REPUSHED_KINDS) - set(lib.ATTENTION_KINDS))
    for name, found in (("表里重复的种类", duplicated), ("代码里有、表里没有的", missing), ("表里有、代码里没有的", extra), ("动作与代码里的处置对不上的", wrong),
                        ("动作不是四样之一的", unknown), ("修改意见是通用句的", generic), ("再推的种类不是表里的提醒", unrepushable)):
        if found:
            problems.append(f"{name}：{'、'.join(found)}")
    if not rows or problems:
        raise SelftestFailure(f"异常表与代码里的种类对不上（表里读到 {len(rows)} 行）：" + "；".join(problems or ["一行都没读到"]))
    return f"异常表 {len(rows)} 行，与提醒、提交时拒与警告、结局、服务自己的几种逐行对得上，动作都在四样里、修改意见没有通用句"


def cell_anomaly_advice_in_texts(world):
    """每一种异常推出去的正文里带它那一行的修改意见：十二种提醒照服务拼正文的那一段（Scheduler.attention_item、attention_text，套在假调度器上，不起服务）各拼一回，
    「出路：」那一行以异常表那一行的修改意见起头；同一种再推的那一回「此刻还差：」那一行同样带它；疑似卡住的正文（suspect_text）带 suspected-stall 那一行的；
    错误类型表里 succeeded 之外每一种结局的打回理由（lib.reason_fields，发给会话的结局正文「出路：」照它）next 带那一种那一行的。
    弄坏开关 attention-next-generic 下一律退回通用句，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: None)   # attention_text 读日志路径：库里没有这一条，写「无（还没起过）」
    fake.reminder_histories, fake.attention_lock, fake.anomaly_repush_seconds = {}, threading.Lock(), SERVER.ANOMALY_REPUSH_SECONDS
    task = {"id": ADVICE_SAMPLE_TASK}
    problems = []
    for kind in lib.ATTENTION_KINDS:
        advice = table_advice(kind, ADVICE_SAMPLE_TASK)
        item = SERVER.Scheduler.attention_item(fake, kind, 1, ADVICE_SAMPLE_TASK, "依据的数", "原因", "这一回的具体数")
        text = SERVER.Scheduler.attention_text(fake, task, item)
        if f"出路：{advice}" not in text:
            problems.append(f"{kind} 的正文「出路：」应当以「{advice[:80]}…」起头，实际「{text[:300]}」")
        fake.reminder_histories[(ADVICE_SAMPLE_TASK, kind)] = [{"threshold": 1, "repush": 0, "at": time.time() - 60, "basis": "头一回的依据"}]
        repushed = SERVER.Scheduler.attention_text(fake, task, dict(item, repush=1))
        if f"此刻还差：{advice}" not in repushed or "之前推过：" not in repushed or "第 1 回再推" not in repushed:
            problems.append(f"{kind} 再推的正文应当写「之前推过：…」与「此刻还差：{advice[:60]}…」、第一行带「第 1 回再推」，实际「{repushed[:400]}」")
    suspect = SERVER.Scheduler.suspect_text(fake, task, task, {"number": 1, "host": "local", "log": ""}, {"segment": 1, "why": "CPU 满窗口不涨"})
    if table_advice("suspected-stall", ADVICE_SAMPLE_TASK) not in suspect:
        problems.append(f"疑似卡住的正文应当带 suspected-stall 那一行的修改意见，实际「{suspect[:300]}」")
    outcomes = [kind for kind in lib.OUTCOME_TYPES if kind != "succeeded"]
    for kind in outcomes:
        fields = dict(lib.reason_fields({"id": ADVICE_SAMPLE_TASK, "outcome": kind, "exit_code": 1}, [], ""))
        wanted = table_advice(kind, ADVICE_SAMPLE_TASK).split("<unit=")[0]   # launcher-lost 那一行的 <unit= 点名的单元> 由 reason_fields 填上单元名，只比它前面那一截
        if wanted not in fields.get("next", "") or lib.BOUNCE_NEW_TASK_REMINDER not in fields.get("next", ""):
            problems.append(f"结局 {kind} 的 next= 应当带异常表那一行的修改意见「{wanted[:80]}…」与「{lib.BOUNCE_NEW_TASK_REMINDER}」，实际「{fields.get('next', '')[:300]}」")
    if problems:
        raise SelftestFailure("推出去的正文带不带异常表的修改意见：" + "；".join(problems))
    return f"{len(lib.ATTENTION_KINDS)} 种提醒（头一回与再推）、疑似卡住与 {len(outcomes)} 种结局的正文都带异常表那一行的修改意见"


HEADLINE_FACTS = re.compile(r"^task-server：(结局|提醒|疑似卡住)（([^）]*)）任务 (\d+) (.+?)（([a-z][a-z-]*)([^）]*)）$")   # 首行：类别、任务此刻的状态、任务号、中文名、英文种类名、括号里其余的


def headline_facts(line):
    """只看推给会话的消息首行，判三件事 → (是结局还是提醒还是疑似卡住, 任务此刻在排在跑还是已结束, 第几回, 上一回的日期或 None, 中文名, 英文种类名)；首行不合写法交 None。"""
    found = HEADLINE_FACTS.match(line)
    if found is None:
        return None
    category, state, _, name, english, rest = found.groups()
    after = "任务已结束" if "任务已结束" in state else "任务还在排" if "任务还在排" in state else "任务还在跑" if "任务还在跑" in state else ""
    rounds, previous = re.search(r"第 (\d+) 回(?!再推)", rest), re.search(r"上一回在 (\d{4}-\d{2}-\d{2})", rest)
    return category, after, int(rounds.group(1)) if rounds else None, previous.group(1) if previous else None, name, english


def cell_inbox_headlines_tell_kind_state_round(world):
    """推给会话的消息首行说清是什么（task_server.py 的 outcome_headline、reminder_headline、suspect_headline、attention_text、suspect_text；起因是别的会话把一条 suspected-stall 读成了结局）：
    四类各造一条，只看首行判得出三件事——是结局、提醒还是疑似卡住；任务此刻在排、在跑还是已结束；第几回（重推的另有上一回在哪一天）：
    结局「结局（任务已结束）…（succeeded，退 0；第 1 回）」；排着的任务的提醒「提醒（任务还在排，不是结局）…」第 1 回；在跑的任务第 2 回再推的提醒第 3 回、上一回在三天前那一天；
    疑似卡住「疑似卡住（任务还在跑、服务没停它）…」；结局没人取而再推的「结局（任务已结束，这条结局还没人取）…」第 3 回、上一回在两天前那一天。每个种类带中文名、英文种类名在括号里；
    错误类型表的每一种结局、每一种提醒、疑似卡住、结局没人取在 lib.KIND_NAMES 里都有中文名。正文里确认标识那几行（长跑已确认）在依据与出路之后；机器读的最后一行不动
    （TASK_SERVER_ATTENTION、TASK_SERVER_SUSPECT 起头）。套在假调度器上，不起服务。
    弄坏开关 inbox-headline-state-omitted 下首行不写类别与状态，inbox-headline-round-omitted 下不写第几回，inbox-headline-name-omitted 下不写中文名，
    inbox-flag-lines-before-evidence 下确认标识那几行在依据与出路之前，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: None)
    fake.reminder_histories, fake.attention_lock, fake.anomaly_repush_seconds = {}, threading.Lock(), SERVER.ANOMALY_REPUSH_SECONDS
    fake.flag_lines = lambda found: ["长跑已确认：自证里的一句理由"]
    now = time.time()
    day = lambda seconds: time.strftime("%Y-%m-%d", time.localtime(seconds))
    problems = []

    def check(what, line, expected):
        facts = headline_facts(line)
        if facts != expected:
            problems.append(f"{what}：首行应当判得出 {expected}，实际 {facts}（首行「{line}」）")

    check("结局", SERVER.outcome_headline(701, "succeeded", 0), ("结局", "任务已结束", 1, None, "成功", "succeeded"))
    queued_item = SERVER.Scheduler.attention_item(fake, "queued-too-long", 1, 702, "已排 30 分钟", "原因")
    queued_text = SERVER.Scheduler.attention_text(fake, {"id": 702, "state": "queued"}, queued_item)
    check("排着的任务的提醒", queued_text.split("\n", 1)[0], ("提醒", "任务还在排", 1, None, "排队过久", "queued-too-long"))
    fake.reminder_histories[(703, "fills-any-cap")] = [{"threshold": 1, "repush": 0, "at": now - 5 * 86400, "round": 1, "basis": "头一回"},
                                                       {"threshold": 1, "repush": 1, "at": now - 3 * 86400, "round": 2, "basis": "第 1 回再推"}]
    again_item = dict(SERVER.Scheduler.attention_item(fake, "fills-any-cap", 1, 703, "三次峰值贴着上限", "原因"), repush=2)
    again_text = SERVER.Scheduler.attention_text(fake, {"id": 703, "state": "running"}, again_item)
    check("在跑的任务再推的提醒", again_text.split("\n", 1)[0], ("提醒", "任务还在跑", 3, day(now - 3 * 86400), "按给的上限开并行", "fills-any-cap"))
    suspect_text = SERVER.Scheduler.suspect_text(fake, {"id": 704, "state": "running"}, {"id": 704}, {"id": 9, "number": 1, "host": SERVER.LOCAL, "log": "x.log"},
                                                 {"segment": 2, "why": "整个单元的 CPU 这 600 秒平均 0.0001 个核"})
    check("疑似卡住", suspect_text.split("\n", 1)[0], ("疑似卡住", "任务还在跑", 1, None, "疑似卡住", "suspected-stall"))
    check("结局没人取再推", SERVER.outcome_headline(705, "succeeded", 0, 3, now - 2 * 86400, unclaimed=True),
          ("结局", "任务已结束", 3, day(now - 2 * 86400), "成功", "succeeded"))
    if "第 2 段" not in suspect_text.split("\n", 1)[0] or "没处置、第 2 回再推" not in again_text.split("\n", 1)[0] or "还没人取" not in SERVER.outcome_headline(705, "succeeded", 0, 3, now, unclaimed=True):
        problems.append("疑似卡住应写第几段，再推的写没处置、第几回再推，没人取的写这条结局还没人取")
    for what, text in (("提醒", again_text), ("疑似卡住", suspect_text)):
        lines = text.split("\n")
        flag = lines.index("长跑已确认：自证里的一句理由") if "长跑已确认：自证里的一句理由" in lines else None
        advice = next((index for index, line in enumerate(lines) if line.startswith("出路：")), None)
        if what == "提醒" and (flag is None or advice is None or flag < advice or any(line.startswith("依据：") for line in lines[flag:])):
            problems.append(f"提醒：确认标识那一行应当在依据与出路之后，实际各行 {[line[:20] for line in lines]}")
    if not again_text.split("\n")[-1].startswith(f"{lib.ATTENTION_MARK} task=703 ") or not suspect_text.split("\n")[-1].startswith(f"{lib.SUSPECT_MARK} task=704 "):
        problems.append("每条消息最末那一行机器读的行（TASK_SERVER_ATTENTION、TASK_SERVER_SUSPECT）应当一个字不动、仍在最后")
    kinds = [*lib.OUTCOME_TYPES, *lib.ATTENTION_KINDS, "suspected-stall", "unclaimed-outcome"]
    unnamed = sorted(kind for kind in kinds if kind not in lib.KIND_NAMES or lib.KIND_NAMES[kind] == kind or not lib.KIND_NAMES[kind].strip())
    if unnamed:
        problems.append(f"这些种类在 lib.KIND_NAMES 里没有中文名：{'、'.join(unnamed)}")
    if problems:
        raise SelftestFailure("推给会话的消息首行：" + "；".join(problems))
    return "结局、提醒（在排、在跑再推）、疑似卡住、结局没人取再推四类：只看首行就判得出类别、任务在排在跑还是已结束、第几回；中文名在前、英文种类名在括号里；确认标识那几行在依据与出路之后"


def cell_anomaly_repushed_with_history(world):
    """门槛不随时长涨的提醒同一道门槛推过之后还在（task_server.py 文件头 ⑥「没处置时」；异常表 fills-any-cap 那一行）：同键三次峰值都贴着上限、再交一条一直跑着，
    收件 socket 先收到一条「（fills-any-cap；第 1 回）」，同一道门槛报过满再推的门槛（自证里 2 秒）再收到一条「（fills-any-cap，没处置、第 1 回再推；第 2 回」，正文写「之前推过：…」与「此刻还差：<那一行的修改意见>」。
    弄坏开关 anomaly-not-repushed 下不再推，inbox-headline-state-omitted 下首行不写类别与状态（「提醒（任务还在跑，不是结局）」那一段）、照首行认不到那一条，这一格红。"""
    key = "selftest-anomaly-repush"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-anomaly-repush")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: key}))
    states = ("任务还在排", "任务还在跑")   # 这一条交的时候就判得出（同键三次峰值贴着上限），排着时就可能推，起了以后也可能推
    first_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}；" for state in states)
    again_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}，没处置、第 1 回再推；第 2 回" for state in states)
    problems = inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
    again = inbox.wait_for(world, again_head)
    problems += inbox_problems(again, again_head, "同一道门槛没处置再推")
    text = again[0]["text"] if again else ""
    advice = table_advice("fills-any-cap", task_id)
    if again and ("之前推过：" not in text or f"此刻还差：{advice}" not in text):
        problems.append(f"再推的正文应当写「之前推过：…」与「此刻还差：{advice[:60]}…」，实际「{text[:400]}」")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("没处置的提醒再推：" + "；".join(problems), world.server_log_lines())
    return f"fills-any-cap 头一回推过、满 {REPUSH_SAMPLE_SECONDS} 秒还在，再推了一回（写了之前推过什么、此刻还差什么）"


def cell_anomaly_acknowledged_not_repushed(world):
    """出路是照跑的提醒记了照跑之后不再推（task_server.py 文件头 ⑥「没处置时」，ACKNOWLEDGED_KINDS）：同键三次峰值贴着上限、再交一条一直跑着，收到 fills-any-cap 头一回之后
    提交它的会话 confirm --why（还没跑满两小时的界）退 0、回话写「已记照跑」；之后过了再推的时限（自证里 2 秒）两倍多，没再收到「没处置、第 1 回再推」那一条。
    弄坏开关 anomaly-acknowledgement-ignored 下照推，这一格红。"""
    key = "selftest-anomaly-acknowledged"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-anomaly-acknowledged")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: key}))
    states = ("任务还在排", "任务还在跑")   # 这一条交的时候就判得出（同键三次峰值贴着上限），排着时就可能推，起了以后也可能推
    first_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}；" for state in states)
    again_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}，没处置、第 1 回再推；第 2 回" for state in states)
    problems = inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    if confirmed.returncode != 0 or "已记照跑" not in confirmed.stdout + confirmed.stderr:
        problems.append(f"没满两小时的界、开着 fills-any-cap 时 confirm --why 应当退 0、回「已记照跑」，实际退 {confirmed.returncode}：{(confirmed.stdout + confirmed.stderr)[-300:]}")
    time.sleep(3 * float(REPUSH_SAMPLE_SECONDS))
    if inbox.headed(again_head):
        problems.append(f"记了照跑之后不该再推，实际收到「{again_head}」")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("记了照跑的提醒不再推：" + "；".join(problems), world.server_log_lines())
    return f"fills-any-cap 头一回推过、confirm --why 记了照跑，过了 {3 * float(REPUSH_SAMPLE_SECONDS):.0f} 秒没再推"


PROGRESS_PUSH_SAMPLE_SECONDS = "2"   # running-progress-pushed：在跑的任务每满这么多秒发一条进度（TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS）


PROGRESS_LOG_LINE_ROUNDS = 6   # running-progress-pushed：等日志末行出现在进度正文里至多等到第几回


def cell_running_progress_pushed(world):
    """在跑进度（task_server.py 文件头 ⑥「在跑进度」）：一条任务在跑满时限（自证里 2 秒）之后，提交它的会话收到一条首行「进度（任务还在跑，不是结局）任务 N 起跑以来」的消息，
    写第 1 回、它在哪一台与日志末行；再满一回时限收到第 2 回；放它跑完，结局照发。弄坏开关 progress-not-pushed 下不发，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS": PROGRESS_PUSH_SAMPLE_SECONDS})
    release = os.path.join(world.root, "release-progress")
    task_id = world.submit(["bash", "-c", "echo progress-line-sample; " + BLOCK_SCRIPT, "block", release], extra=inbox.environment())
    world.running(task_id)
    head = f"{SERVER.INBOX_HEADLINE}进度（任务还在跑，不是结局）任务 {task_id} 起跑以来"
    found = inbox.wait_for(world, head, count=2)
    problems = []
    if len(found) < 2:
        problems.append(f"在跑满 {PROGRESS_PUSH_SAMPLE_SECONDS} 秒两回应当收到两条首行「{head}」的进度，实际 {len(found)} 条")
    else:
        first, second = found[0]["text"], found[1]["text"]
        if "第 1 回" not in first.splitlines()[0] or "第 2 回" not in second.splitlines()[0]:
            problems.append(f"两条的首行应当依次写第 1 回、第 2 回，实际「{first.splitlines()[0]}」「{second.splitlines()[0]}」")
        # 日志末行不钉在第 1 回上：时限从尝试起了那一刻算，机器忙时命令的第一行可能晚于第 1 回才写进日志（load 139 时第 1 回读到「日志还是空的」）；
        # 接着等后面的回数，至多到第 PROGRESS_LOG_LINE_ROUNDS 回，有一回带上那一行就算。
        for wanted_count in range(3, PROGRESS_LOG_LINE_ROUNDS + 1):
            if any("progress-line-sample" in message["text"] for message in found):
                break
            found = inbox.wait_for(world, head, count=wanted_count)
        if f"任务 {task_id} 在 " not in first or not any("progress-line-sample" in message["text"] for message in found):
            problems.append(f"进度正文应当写这一次尝试在哪一台、收到的 {len(found)} 回里有一回带日志末行（progress-line-sample），实际第 1 回「{first[:400]}」、最后一回「{found[-1]['text'][:400]}」")
    with open(release, "w", encoding="utf-8"):
        pass
    world.finished(task_id)
    if problems:
        raise SelftestFailure("在跑进度：" + "；".join(problems), world.server_log_lines())
    return f"在跑的任务每满 {PROGRESS_PUSH_SAMPLE_SECONDS} 秒给提交它的会话发一条进度，写第几回、在哪一台与日志末行"


def cell_unclaimed_outcome_repushed(world):
    """结局没人取（task_server.py 文件头 ⑥「结局没人取」；异常表 unclaimed-outcome）：会话还没登进会话登记时任务跑完，结局那一条发不出去（写 inbox-unreachable 告警）；
    之后会话登上（演会话重启、换了 socket），结局满再推的门槛（自证里 2 秒）没人取，服务再推一回到新的 socket：第一行「结局还没人取（succeeded」，写此刻还差什么（那一行的修改意见）
    与那条结局的末行；推出去了记成已取，status 不再列成没人取。弄坏开关 unclaimed-outcome-not-repushed 下不再推，这一格红。"""
    inbox = FakeInbox(world, "a", register=False)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS": UNCLAIMED_REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", "exit 0", "unclaimed-repush"], extra=inbox.environment())
    world.finished(task_id)
    patience = world.patience(INBOX_WAIT_SECONDS)
    while patience.waiting() and not any(alert.get("kind") == "inbox-unreachable" and alert.get("task") == task_id for alert in world.alerts()):
        time.sleep(POLL_SECONDS)
    world.register_session(SESSION_A, inbox.path, "inbox-a")
    head = unclaimed_head(task_id, "succeeded")
    found = inbox.wait_for(world, head)
    problems = inbox_problems(found, head, "结局没人取再推")
    text = found[0]["text"] if found else ""
    advice = table_advice("unclaimed-outcome", task_id)
    if found and (f"此刻还差：{advice}" not in text or f"{lib.RESULT_MARK} task={task_id} type=succeeded" not in text):
        problems.append(f"再推的正文应当写「此刻还差：{advice[:60]}…」并带那条结局的末行，实际「{text[:400]}」")
    patience = world.patience(INBOX_COLLECTED_SECONDS)
    listed = True
    while patience.waiting() and listed:
        listed = any(item.get("id") == task_id for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", []))
        time.sleep(POLL_SECONDS)
    if listed:
        problems.append(f"再推出去之后任务 {task_id} 应当记成已取、status 不再列成完成了没人取，实际还列着")
    if problems:
        raise SelftestFailure("结局没人取再推：" + "；".join(problems), world.server_log_lines())
    return f"结局头一回发不出去、会话登上之后满 {UNCLAIMED_REPUSH_SAMPLE_SECONDS} 秒再推到了新的 socket，记成已取"


def cell_inbox_socket_follows_session(world):
    """按会话号找收件 socket（task_server.py 文件头 ⑥「收件地址」）：交任务时会话的收件 socket 是旧的那一个；任务跑着时那个会话重启（旧 socket 关了、
    会话登记里换成新的一份），另有一份同一会话号、更晚启动而进程已经不在的登记点着旧 socket（不该认）；任务跑完，结局那一条照新的 socket 收到（令牌照交任务时的）。
    弄坏开关 inbox-socket-from-submit-only 下照交任务时的旧 socket 发、发不出去，这一格红。"""
    old = FakeInbox(world, "old")
    release = os.path.join(world.root, "release-socket-follows")
    start_inbox_server(world)
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=old.environment())
    world.running(task_id)
    old.close()
    os.remove(os.path.join(world.sessions, "inbox-old.json"))
    gone = subprocess.Popen(["true"])   # 一个已经退了的进程号：那一份登记点着旧 socket、启动更晚，进程不在了不该认
    gone.wait()
    world.register_session(SESSION_A, old.path, "gone-process", pid=gone.pid, started_milliseconds=int(time.time() * 1000) + 60000)
    new = FakeInbox(world, "new", token=old.token)
    open(release, "w").close()
    world.finished(task_id)
    head = outcome_head(task_id, "succeeded")
    problems = inbox_problems(new.wait_for(world, head), head, "会话重启之后的结局")
    if problems:
        raise SelftestFailure("按会话号找收件 socket：" + "；".join(problems), world.server_log_lines())
    return "会话重启换了收件 socket（旧的那一份进程已不在）：结局照会话号找到的新 socket 收到，令牌照交任务时的"


def cell_cleanup_protocol_unit_tests(world):
    """收尾协议的离线单测（task-server/tests/test_cleanup_protocol.py；unittest，在这一格自己的进程里跑，不起服务）：恰好 CLEANUP_PROTOCOL_TESTS 条、全过才绿；
    它的假调度器照 Scheduler.__init__ 的表名现建（scheduler_tables）。弄坏开关 cleanup-tests-tables-hand-listed 下照手抄的旧名单建、收尾那几条红，这一格红。"""
    module = lib.load_module("test_cleanup_protocol", os.path.join(lib.SCRIPT_DIRECTORY, "tests", "test_cleanup_protocol.py"))
    suite = unittest.defaultTestLoader.loadTestsFromModule(module)
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    bad = [test.id() for test, _ in result.failures + result.errors] + [test.id() for test, _ in result.skipped]
    if result.testsRun != CLEANUP_PROTOCOL_TESTS or bad or suite.countTestCases() != CLEANUP_PROTOCOL_TESTS:
        raise SelftestFailure(f"收尾协议单测应当跑 {CLEANUP_PROTOCOL_TESTS} 条、全过，实际载入 {suite.countTestCases()} 条、跑了 {result.testsRun} 条，不对或跳过的 {bad}",
                              stream.getvalue().splitlines()[-40:])
    return f"收尾协议单测 {result.testsRun} 条全过（假调度器的表照 Scheduler.__init__ 现建）"


def cell_recheck_judges_disk(world):
    """派第二台的那一条起之前复核现读第二台（task_server.py 的 Resources.recheck）：排队那一轮第二台的报告带着够的盘余量、照派；复核读到的报告不带盘余量
    （同真的第二台探测），服务现量（假 df）：
    不到下限的这一条不起、排队原因写「第二台盘只剩」；过了下限而放不下这条命令量过的编译目录占盘（30G 加下限）的不起、排队原因写「编译目录占盘」；
    复核记 fits=0、一次都没起，假 df 放够之后照派第二台跑完。弄坏开关 recheck-disk-unchecked 下复核不判盘、照起，这一格红。"""
    floor = SERVER.DISK_FLOOR_BYTES
    world.seed_attempt({"command_key": "selftest-recheck-build", "outcome": "succeeded", "memory_peak": GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    world.start_server()
    problems = []
    cases = (("盘余量不到下限", "selftest-recheck-floor", floor - GIBIBYTE, "第二台盘只剩"),
             ("编译目录占盘放不下", "selftest-recheck-build", floor + 10 * GIBIBYTE, "编译目录占盘"))
    for name, key, shown, wanted in cases:
        # 本机放不下（空 1G）：只派得到第二台；排队那一轮第二台报告里的盘余量够，复核那一份不带、df 演 shown
        world.resources(local=(1, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor,
                        peer_df=f"     Avail\n{shown}\n", recheck_peer=(100, 100))
        task_id = world.submit(["bash", "-c", "exit 0", name], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        rechecks = []
        while patience.waiting() and not rechecks:
            rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={task_id} host=peer fits=")]
            time.sleep(POLL_SECONDS)
        world.next_pass()
        held = world.summary(task_id)
        waiting = held.get("waiting_for") or ""
        if rechecks[:1] != [f"recheck task={task_id} host=peer fits=0"] or held["attempts"] or world.calls_of(task_id, "peer") or wanted not in waiting:
            problems.append(f"{name}（复核时 df 报 {SERVER.gibibytes_text(shown)}）：复核应当记 fits=0、一次都不起、排队原因写「{wanted}」，实际复核 {rechecks[:1]}、"
                            f"尝试 {len(held['attempts'])} 次、在等「{waiting[:300]}」")
        world.resources(local=(1, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor,
                        peer_df=f"     Avail\n{10 * floor}\n", recheck_peer=(100, 100))
        done = world.finished(task_id)
        if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
            problems.append(f"{name}：df 放够之后应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("起之前复核判盘余量：" + "；".join(problems), world.server_log_lines())
    return "起之前复核现量第二台的盘：不到下限、放不下编译目录占盘的都不起，排队原因写盘；df 放够之后照派第二台"


def cell_recheck_judges_nfs(world):
    """派第二台的那一条判过放得下、起之前复核现读之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=launch-recheck），这时第二台没挂上共享根（假 ssh 的 stat -f 退 1，
    NFS 复核记下，/health 的 nfs 一栏挂载那一项不过）；放开之后复核那一份过 peer_view（task_server.py 的 launch_recheck）：记 fits=0、不在第二台起，
    排队原因写「没挂上共享根」；挂载好了之后照派第二台跑完。弄坏开关 recheck-nfs-unchecked 下复核那一份不过 peer_view、照派第二台，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G：只派得到第二台
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"launch-recheck:{pause}"})
    open(os.path.join(pause, "arm-launch-recheck"), "w").close()
    task_id = world.submit(["bash", "-c", "exit 0", "recheck-nfs"], cwd=world.repo)
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-launch-recheck"), "判过放得下、起之前复核现读之前停住")
        with open(mount_exit, "w", encoding="utf-8") as handle:
            handle.write("1\n")
        health = health_until(world, lambda health: any(entry.get("mount") != lib.NFS_PASSED for entry in (health.get("nfs") or {}).get("roots", [])),
                              STATE_TIMEOUT_SECONDS)
        if not any(entry.get("mount") != lib.NFS_PASSED for entry in (health.get("nfs") or {}).get("roots", [])):
            raise SelftestFailure(f"假 ssh 的挂载那一项退 1 之后 /health 的 nfs 一栏应当判挂载不过，实际 {health.get('nfs')}", world.server_log_lines())
    finally:
        open(os.path.join(pause, "release-launch-recheck"), "w").close()
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    rechecks = []
    while patience.waiting() and not rechecks:
        rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={task_id} host=peer fits=")]
        time.sleep(POLL_SECONDS)
    world.next_pass()
    held = world.summary(task_id)
    waiting = held.get("waiting_for") or ""
    if rechecks[:1] != [f"recheck task={task_id} host=peer fits=0"] or held["attempts"] or world.calls_of(task_id, "peer") or "没挂上共享根" not in waiting:
        problems.append(f"复核这一刻第二台没挂上共享根：复核应当记 fits=0、不在第二台起、排队原因写「没挂上共享根」，实际复核 {rechecks[:1]}、尝试 {len(held['attempts'])} 次、"
                        f"在等「{waiting[:300]}」")
    os.remove(mount_exit)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
        problems.append(f"挂载好了之后应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("起之前复核过 NFS 判法：" + "；".join(problems), world.server_log_lines())
    return "起之前复核那一刻第二台没挂上共享根：复核记 fits=0、不在第二台起、排队原因写明；挂载好了照派第二台"


def submitted_sheet(world, command, cwd=None):
    """交一条、不确认：→ (确认单那几行的文字, 任务号)；交不成判红。"""
    completed = world.client(["submit", "--cwd", cwd or world.work, "--", *command])
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"交 {command} 应当退 0、打任务号，实际退 {completed.returncode}", [completed.stdout, completed.stderr] + world.server_log_lines())
    return completed.stdout, int(found.group(1))


def cell_confirmation_scope_text(world):
    """确认单的范围（task_server.py 文件头 ⑦）：命令里没点、脚本里面才展开的写「范围由脚本里面定，服务算不出」，不写「整包、不指定范围」；合入后验证另打一行崩溃用例与域；别的 cargo 照写整包。
    确认单上 --crash-expand-up-to 的值与点名的 crash-case 项数照打。弄坏开关 scope-inside-script-unnamed 下照写整包，crash-scope-unreported 下不打崩溃用例与域，confirm-skipped 下没有确认单，这一格红。"""
    problems = []
    for named, whole, expected in (([], True, lib.WHOLE_SCOPE_TEXT), ([], False, lib.SCOPE_INSIDE_SCRIPT_TEXT), ([("--item", "alpha")], False, "--item 1")):
        if lib.named_scope_text(named, whole) != expected:
            problems.append(f"named_scope_text({named}, {whole}) 应当是「{expected}」，实际「{lib.named_scope_text(named, whole)}」")
    merge = ["bash", FAKE_MERGE_ENTRY, "--skip-apply"]
    for command, words in ((["bash", "-c", "exit 0"], None), ([*merge, "patch"], [lib.SCOPE_INSIDE_SCRIPT_TEXT]),
                           ([*merge, "--crash-expand-up-to", "6", "patch"], ["--crash-expand-up-to 6", lib.SCOPE_INSIDE_SCRIPT_TEXT]),
                           ([*merge, "--item", "crash-case:alpha", "--item=crash-case:beta", "--item", "unit:gamma", "patch"], ["2 条", "由脚本定"])):
        text = lib.crash_scope_text(command, world.repo)
        if (words is None and text) or (words is not None and not all(word in text for word in words)):
            problems.append(f"crash_scope_text({command}) 应当{'是空串' if words is None else '写 ' + '、'.join(words)}，实际「{text}」")
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    patch = os.path.join(world.root, "patch-scope")
    os.makedirs(patch)
    world.start_server()
    for what, command, cwd, expected, unexpected in (("脚本自己定范围的", ["bash", "-c", "exit 0", "scope-sample"], None, [lib.SCOPE_INSIDE_SCRIPT_TEXT], [lib.WHOLE_SCOPE_TEXT, "崩溃用例与域"]),
                                                      ("合入后验证", [*merge, patch], world.repo, [f"项数 0：{lib.SCOPE_INSIDE_SCRIPT_TEXT}", f"崩溃用例与域：{lib.SCOPE_INSIDE_SCRIPT_TEXT}"], [lib.WHOLE_SCOPE_TEXT]),
                                                      ("别的 cargo", ["cargo", "build", "-p", "selftest-scope"], None, [lib.WHOLE_SCOPE_TEXT], [lib.SCOPE_INSIDE_SCRIPT_TEXT])):
        sheet, task_id = submitted_sheet(world, command, cwd)
        world.client(["cancel", str(task_id)])
        if not all(word in sheet for word in expected) or any(word in sheet for word in unexpected):
            problems.append(f"{what}的确认单应当写 {expected}、不写 {unexpected}，实际「{sheet.strip()[:600]}」")
    if problems:
        raise SelftestFailure("确认单的范围：" + "；".join(problems), world.server_log_lines())
    return "脚本里面才展开的写算不出、不写整包；合入后验证打崩溃用例与域，命令里写了的照打"
