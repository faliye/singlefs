"""task_server_selftest.py 拆出的一份：暂停与恢复、换服务不丢的状态、尝试的起与收尾、编译目录池的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import json
import re
import shutil
import signal
import sqlite3
import subprocess
import threading
import time
import unittest.mock
import task_server_lib as lib
from task_server_selftest_fakes import GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, POLL_SECONDS, SERVER, SESSION_A, SESSION_B, STATE_TIMEOUT_SECONDS
from task_server_selftest_world import (
    KEEP_TEMPORARY_EXTRA, SCENE_COMMAND_EXIT, SelftestFailure, blocking_command, build_slot_tree, cargo_targets, child_roles, describe,
    fake_cargo_extra, first_attempt, health_until, raw_column_update, read_text, restart_surviving_scheduler, scene_of, verified_row, wait_path,
)


def cell_resume_declaration_first_word_only(world):
    """续跑声明的判法（task_server_lib.py 的 resumable_command、declaration_matches）：只看第一个非选项参数，后面的参数对上词不算；* 照旧。静态喂文件头，不起服务。
    弄坏开关 resume-declaration-any-word 下非选项参数有一个对上就算，这一格红。"""
    script = os.path.join(world.work, "declared-resume.sh")
    with open(script, "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} segment\nexit 0\n")
    everything = os.path.join(world.work, "declared-resume-all.sh")
    with open(everything, "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} *\nexit 0\n")
    cases = ((script, ["segment"], True), (script, ["--quick", "segment"], True), (script, ["other", "segment"], False), (script, [], False), (everything, ["other"], True))
    problems = []
    for path, arguments, expected in cases:
        found = lib.resumable_command(["bash", path, *arguments], world.work)
        if found != expected:
            problems.append(f"{os.path.basename(path)} {arguments} 应当 {expected}，实际 {found}")
    if problems:
        raise SelftestFailure("续跑声明：" + "；".join(problems))
    return f"{len(cases)} 条：只看第一个非选项参数，* 照旧"


DRAINED_FRAGMENT_PEAK_BYTES = 15 * GIBIBYTE   # 那一格占着的小任务：上限与记账 15G


def cell_ended_attempt_settled_after_restart(world):
    """最新尝试已记成 ended（memory-cap-hit）、任务却还 running（任务 2461：定去向半途抛错留下的孤儿，本机因此两小时算有任务在跑）：
    服务起来对账时补收尾，照撞上限的规矩上限 ×1.5 重排、跑完。"""
    world.resources(local=(50, 50))
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "orphan"], extra={lib.KEY_VARIABLE: "selftest-orphan"})
    world.finished(task_id)
    world.stop_server()
    world.rewrite_signed_row("tasks", task_id, lambda record: record.update(state="running", outcome="", exit_code=None, finished_at=None))
    attempt_ids = [record["id"] for record, _ in world.signed_rows("attempts") if record.get("task_id") == task_id]
    world.rewrite_signed_row("attempts", attempt_ids[-1],
                             lambda record: record.update(outcome="memory-cap-hit", exit_code=250, memory_cap=GIBIBYTE, memory_cap_text="1024M"))
    world.start_server()
    summary = world.finished(task_id)
    attempts = summary["attempts"]
    if summary["task"].get("outcome") != "succeeded" or len(attempts) != 2 or attempts[1].get("memory_cap_text") != "1536M":
        raise SelftestFailure(f"服务起来应当补收尾：按撞上限 ×1.5（1536M）重排、第 2 次跑完 succeeded，实际 {describe(summary)}", world.server_log_lines())
    return "最新尝试 ended、任务 running 的孤儿：对账补收尾，1536M 重排、跑完"


def cell_attempt_claim_atomic(world):
    """TS-03：在插了尝试、改任务之前（一个事务里）服务当场退出：重启之后库里没有那一次尝试，照常起一次、命令只跑一次；同一任务同一次号再插一行被库拒。"""
    problems = []
    world.stop_server()
    arm = os.path.join(world.root, "arm-crash-inside-claim")
    counter = os.path.join(world.root, "count-crash-inside-claim")
    release = os.path.join(world.root, "release-crash-inside-claim")
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"attempt-claimed:{arm}"})
    task_id = world.submit(blocking_command(release, counter))
    open(arm, "w").close()
    world.resources(local=(30, 30))
    code = world.wait_server_exit()
    if code != 70:
        raise SelftestFailure(f"自证造的崩溃应当让服务退 70，实际退 {code}", world.server_log_lines())
    try:
        world.start_server()
        summary = world.running(task_id)
        numbers = [attempt.get("number") for attempt in summary["attempts"]]
        if numbers != [1] or summary["task"].get("attempt_count") != 1:
            problems.append(f"重启之后任务应当 running、只有第 1 次尝试、尝试次数记 1，实际尝试号 {numbers}、尝试次数 {summary['task'].get('attempt_count')}")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded" or read_text(counter) != "x" or len(summary["attempts"]) != 1:
        problems.append(f"应当只起一次、命令只跑一次、succeeded，实际命令跑了 {len(read_text(counter))} 次、{describe(summary)}")
    try:
        world.insert_signed_row("attempts", {"task_id": task_id, "number": 1, "state": "ended", "session": "", "command_key": "selftest-duplicate"})
        problems.append(f"同一任务 {task_id} 同一次号 1 再插一行尝试应当被库拒（唯一约束），实际插进去了")
    except sqlite3.IntegrityError:
        pass
    if problems:
        raise SelftestFailure("插尝试与改任务：" + "；".join(problems), world.server_log_lines())
    return "插尝试与改任务一个事务：事务里崩了重启照常起一次；同一次号再插被库拒"


def cell_start_timeout_read_back(world):
    """TS-06：systemd-run 起了单元却不回执（假 systemd-run 交给真的起完就一直睡，起单元的时限缩到 1 秒）：任务照单元名读回、照常 running、账照记，
    取消照常停它；systemd-run 什么都没起也不回执：尝试留在 starting、写告警 start-uncertain，过了宽限读回单元一直不在才记 unit-start-failed、命令没跑。"""
    problems = []
    release = os.path.join(world.root, "release-timeout")
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-delayed"), "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS": "1"})
    try:
        task_id = world.submit(blocking_command(release))
        summary = world.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished" or
                                 any(attempt.get("state") == "running" and world.unit_active(attempt.get("unit", "")) for attempt in summary["attempts"]),
                                 "照单元名读回成在跑")
        if summary["task"].get("state") != "running":
            problems.append(f"回执超时而单元起了：任务应当照读回的单元记 running，实际 {describe(summary)}")
        else:
            cancelled = world.client(["cancel", str(task_id)])
            finished = world.finished(task_id)
            if cancelled.returncode != 0 or finished["task"].get("outcome") != "cancelled" or world.unit_active(summary["attempts"][0].get("unit", "")):
                problems.append(f"回执超时而单元起了：取消应当照常停它、记 cancelled，实际退 {cancelled.returncode}、{describe(finished)}")
    finally:
        open(release, "w").close()
    world.stop_server()
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-silent"), "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS": "1"})
    marker = os.path.join(world.root, "timeout-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    summary = world.finished(task_id)
    if (summary["task"].get("outcome") != "unit-start-failed" or os.path.exists(marker)
            or not any(alert.get("kind") == "start-uncertain" and alert.get("task") == task_id for alert in world.alerts())):
        problems.append(f"回执超时而什么都没起：应当先写告警 start-uncertain、读回确认没起才记 unit-start-failed、命令没跑，实际 {describe(summary)}、"
                        f"命令{'跑了' if os.path.exists(marker) else '没跑'}、告警 {[alert.get('kind') for alert in world.alerts()]}")
    if problems:
        raise SelftestFailure("起单元回执超时：" + "；".join(problems), world.server_log_lines())
    return "回执超时而单元起了：照单元名读回 running、取消照常停；什么都没起：告警 start-uncertain，读回确认没起才记 unit-start-failed"


def cell_failed_start_not_counted(world):
    """本机放不下、都去第二台：排在前面的 PEER_STARTS_PER_PASS 条 systemd-run 起不来（假 systemd-run 认工作目录 refuse-start），后面同样条数的放得下：
    没起成的不占这一轮在第二台的名额，后面那几条与它们同一轮起（数服务记的「哪一轮起的」）。弄坏开关 failed-start-counted 下后面那几条挪到下一轮，这一格红。"""
    count = SERVER.PEER_STARTS_PER_PASS
    key = "selftest-failed-start"
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64, hold=True)
    world.seed_peak(key, 2 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-selective")})
    refusing = os.path.join(world.repo, "refuse-start")
    os.makedirs(refusing)
    refused = [world.submit(["bash", "-c", "exit 0", "refused", "fake-peer-run"], extra={lib.KEY_VARIABLE: key}, cwd=refusing) for _ in range(count)]
    started = [world.submit(["bash", "-c", "exit 0", "started", "fake-peer-run"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo) for _ in range(count)]
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64)
    world.api("POST", "/wake")
    outcomes = {task_id: world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in refused + started}
    views = {}
    for line in world.scheduler_reads():
        found = re.match(r"^start task=(\d+) host=(\w+) view=(\d+)$", line)
        if found and int(found.group(1)) in outcomes:
            views[int(found.group(1))] = (found.group(2), int(found.group(3)))
    first = views.get(refused[0], ("", -1))
    if (any(outcomes[task_id] != "unit-start-failed" for task_id in refused) or any(outcomes[task_id] != "succeeded" for task_id in started)
            or any(views.get(task_id) != first for task_id in refused + started) or first[0] != "peer"):
        raise SelftestFailure(f"前 {count} 条在第二台起不来、后 {count} 条放得下：起不来的记 unit-start-failed、后面那几条 succeeded，{2 * count} 条都在第二台同一轮起；"
                              f"实际结局 {outcomes}、各条在哪台哪一轮起 {views}", world.server_log_lines())
    return f"第二台一轮的名额 {count} 条：前 {count} 条起不来不占名额，后 {count} 条与它们同一轮起"


# ── 暂停（task_server.py 文件头 ⑤「暂停」；control pause / resume）──
PAUSE_REASON_SAMPLE = "自证：换二进制要一段空档"
PAUSED_WATCH_SECONDS = 1.5   # 暂停着看这么久排着的任务有没有被起（排队一轮 0.1 秒：该起的早起了）
# 第 1 次尝试等放行文件在了退 254（被总上限挤掉，同一上限重排），第 2 次退 0：暂停着第 1 次跑完，重排回队的那一条不该被起
RETRY_AFTER_RELEASE_SCRIPT = ('for round in $(seq 1 2000); do [[ -e "$1" ]] && break; sleep 0.1; done; '
                              '[[ "$TASK_SERVER_ATTEMPT" == 1 ]] && exit 254; exit 0')


def pause_by_api(world, reason, allow_session=""):
    """POST /pause（这一格不是测 control 的，经接口直接暂停；给了 allow_session 就放行那个会话）：回 200 且 paused 为真才交回话。"""
    status, reply = world.api("POST", "/pause", {"reason": reason, "allow_session": allow_session} if allow_session else {"reason": reason})
    if status != 200 or not reply.get("paused"):
        raise SelftestFailure(f"POST /pause 应当回 200、paused 为真，实际 HTTP {status}：{reply}", world.server_log_lines())
    return reply


def pause_health(world):
    """此刻 /health 的整份回话（读的是服务存的暂停态与计数）。"""
    return lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)[1]


def held_in_queue(world, task_id):
    """这条任务此刻排着没起：state 是 queued、一次尝试都没有。"""
    summary = world.summary(task_id)
    return summary["task"].get("state") == "queued" and not summary["attempts"]


def cell_pause_holds_new_attempts(world):
    """暂停着交的任务排着不起：普通任务与分片的计划那一步（子任务也是新尝试）都留在队里，一次尝试都没有，命令一行没跑。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    marker = os.path.join(world.root, "paused-ran")
    plain_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    sharded_id = world.submit(world.write_spec("spec-paused.json", {"shards": [{}]}), cwd=world.repo)
    time.sleep(PAUSED_WATCH_SECONDS)
    plain, sharded = world.summary(plain_id), world.summary(sharded_id)
    children = sharded.get("children", [])
    if (plain["task"].get("state") != "queued" or plain["attempts"] or os.path.exists(marker) or world.sharded_calls("plan")
            or child_roles(sharded) != ["plan"] or any(child["task"].get("state") != "queued" or child["attempts"] for child in children)):
        raise SelftestFailure(f"暂停着交的任务应当排着不起：普通任务应当 queued、没有尝试、命令没跑，分片的只有计划那一个子任务、也是 queued 没有尝试，"
                              f"实际普通任务 {plain['task'].get('state')}、尝试 {len(plain['attempts'])} 次、命令{'跑了' if os.path.exists(marker) else '没跑'}，"
                              f"分片的子任务 {child_roles(sharded)}、状态 {[child['task'].get('state') for child in children]}、计划跑了 {len(world.sharded_calls('plan'))} 回",
                              world.server_log_lines())
    return f"暂停着交的普通任务与分片的计划子任务排了 {PAUSED_WATCH_SECONDS} 秒都没起：没有尝试、命令没跑"


def cell_pause_lets_running_finish(world):
    """在跑的照跑到结局：跑着的第 1 次尝试暂停之后才放行、退 254，暂停着对账照做（结局写回、重排回队），回队的那一条不起第 2 次。"""
    world.resources(local=(30, 30))
    world.start_server()
    release = os.path.join(world.root, "pause-release")
    task_id = world.submit(["bash", "-c", RETRY_AFTER_RELEASE_SCRIPT, "retry-after-release", release])
    world.running(task_id)
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    open(release, "w").close()
    world.wait_for(task_id, lambda found: found["task"].get("state") == "queued" and [attempt.get("outcome") for attempt in found["attempts"]] == ["squeezed-out"],
                   "把暂停之前就在跑的第 1 次尝试的结局写回、重排回队（暂停着对账照做）")
    time.sleep(PAUSED_WATCH_SECONDS)
    later = world.summary(task_id)
    if later["task"].get("state") != "queued" or len(later["attempts"]) != 1:
        raise SelftestFailure(f"暂停着重排回队的那一条不该起第 2 次尝试，实际 {later['task'].get('state')}、尝试 {len(later['attempts'])} 次（{describe(later)}）",
                              world.server_log_lines())
    return "暂停之前在跑的第 1 次尝试照跑到结局（squeezed-out 写回、重排回队），回队的那一条暂停着不起第 2 次"


def cell_pause_survives_restart(world):
    """暂停态记在库里：服务重启后照旧暂停（再 pause 回 changed 为假，原因与起点还是原来的）；库里的暂停记录被绕过服务改过的，按暂停算、说明写校验不过。"""
    world.resources(local=(30, 30))
    world.start_server()
    first = pause_by_api(world, PAUSE_REASON_SAMPLE)
    world.stop_server()
    world.start_server()
    status, again = world.api("POST", "/pause", {"reason": "重启之后又暂停"})
    if status != 200 or again.get("changed") or again.get("reason") != PAUSE_REASON_SAMPLE or again.get("since") != first.get("since"):
        raise SelftestFailure(f"服务重启后应当还暂停着：再 pause 应当回 changed 为假、原因「{PAUSE_REASON_SAMPLE}」、起点 {first.get('since')} 不变，实际 HTTP {status}：{again}",
                              world.server_log_lines())
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE meta SET value = replace(value, ?, ?) WHERE name = ?", (PAUSE_REASON_SAMPLE, "绕过服务改过的原因", SERVER.PAUSE_META_NAME))
    world.start_server()
    status, health = lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)
    if status != 200 or not health.get("paused") or "校验不过" not in ((health.get("pause") or {}).get("reason") or ""):
        raise SelftestFailure(f"暂停记录被改过时应当按暂停算、原因写校验不过，实际 HTTP {status}：paused {health.get('paused')}、pause {health.get('pause')}", world.server_log_lines())
    return "重启后再 pause 回 changed 为假、原因与起点不变；暂停记录被改过的按暂停算、写校验不过"


def cell_resume_releases_queue(world):
    """resume 放开：暂停着交的任务在 resume 之后起来、跑完，resume 回话与 /health 都不是暂停；再 resume 回 changed 为假。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    task_id = world.submit(["bash", "-c", "exit 0"])
    status, reply = world.api("POST", "/resume", {})
    health = lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)[1]
    if status != 200 or reply.get("paused") or not reply.get("changed") or health.get("paused"):
        raise SelftestFailure(f"resume 应当回 200、paused 为假、changed 为真，/health 的 paused 也为假，实际 HTTP {status}：{reply}、/health paused {health.get('paused')}",
                              world.server_log_lines())
    summary = world.finished(task_id)
    status, again = world.api("POST", "/resume", {})
    if summary["task"].get("outcome") != "succeeded" or status != 200 or again.get("changed"):
        raise SelftestFailure(f"resume 之后排着的任务应当起来、跑完 succeeded，再 resume 回 changed 为假，实际 {describe(summary)}、再 resume HTTP {status}：{again}",
                              world.server_log_lines())
    return "resume 之后暂停着交的任务起来、跑完 succeeded；再 resume 回 changed 为假"


def cell_pause_reported(world):
    """control pause / check / resume 与 status 的字样：pause 回读是暂停才退 0、已暂停着的说原来就暂停着；check 退 0、另打「暂停中（原因：…，起于…）、在跑 N 个」；
    排着的任务 status 写「在等：服务暂停中（原因：…）」；resume 之后这几句都没了；pause、resume 各写一条操作日志。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    problems = []
    paused = world.control(["pause", "--reason", PAUSE_REASON_SAMPLE])
    if paused.returncode != 0 or "暂停了" not in paused.stdout or PAUSE_REASON_SAMPLE not in paused.stdout:
        problems.append(f"control pause 应当退 0、说暂停了并点名原因，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-200:]}」、stderr「{paused.stderr.strip()[-200:]}」")
    task_id = world.submit(["bash", "-c", "exit 0"])
    shown = world.client(["status", str(task_id)])
    if f"在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）" not in shown.stdout:
        problems.append(f"暂停着排着的任务 status 应当写「在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）」，实际「{shown.stdout.strip()[:300]}」")
    checked = world.control(["check"])
    if checked.returncode != 0 or f"暂停中（原因：{PAUSE_REASON_SAMPLE}，起于 " not in checked.stdout or "）、在跑 0 个" not in checked.stdout:
        problems.append(f"暂停着 control check 应当退 0、另打「暂停中（原因：{PAUSE_REASON_SAMPLE}，起于 …）、在跑 0 个」，实际退 {checked.returncode}、"
                        f"stdout「{checked.stdout.strip()[-300:]}」、stderr「{checked.stderr.strip()[-200:]}」")
    again = world.control(["pause", "--reason", "别的原因"])
    if again.returncode != 0 or "原来就暂停着" not in again.stdout or PAUSE_REASON_SAMPLE not in again.stdout:
        problems.append(f"已经暂停着再 control pause 应当退 0、说原来就暂停着、原因还是原来的，实际退 {again.returncode}、stdout「{again.stdout.strip()[-200:]}」")
    resumed = world.control(["resume"])
    if resumed.returncode != 0 or "放开了" not in resumed.stdout:
        problems.append(f"control resume 应当退 0、说放开了，实际退 {resumed.returncode}、stdout「{resumed.stdout.strip()[-200:]}」、stderr「{resumed.stderr.strip()[-200:]}」")
    after_check = world.control(["check"])
    after_status = world.client(["status", str(task_id)])
    if "暂停中" in after_check.stdout or "服务暂停中" in after_status.stdout:
        problems.append(f"resume 之后 control check 与 status 都不该再写暂停，实际 check「{after_check.stdout.strip()[-200:]}」、status「{after_status.stdout.strip()[:200]}」")
    notes = {operation.get("action"): operation.get("note") for operation in world.operations() if operation.get("action") in ("pause", "resume")}
    if notes.get("pause") != PAUSE_REASON_SAMPLE or "resume" not in notes:
        problems.append(f"操作日志里应当有 pause（备注原因「{PAUSE_REASON_SAMPLE}」）与 resume 各一条，实际 {notes}")
    if problems:
        raise SelftestFailure("暂停的字样：" + "；".join(problems), world.server_log_lines())
    return "control pause 回读退 0、已暂停的说原来就暂停着；check 退 0 另打暂停中；status 写在等服务暂停中；resume 之后都没了；操作日志有 pause 与 resume"


def cell_pause_not_starved(world):
    """暂停着不报「排头放得下却没起」：本机空 100G、排头一条小的放得下，暂停着排着不起，过了阈值几倍 /health 照回 ok，control check 退 0。"""
    world.resources(local=(100, 100))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    checked = world.control(["check"])
    if not health.get("ok") or checked.returncode != 0 or "放得下" in checked.stdout + checked.stderr:
        raise SelftestFailure(f"暂停着排头放得下而不起是要的：/health 应当一直回 ok、control check 应当退 0 且不提放得下，实际 /health {health.get('ok')}（{health.get('problem')}）、"
                              f"check 退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」", world.server_log_lines())
    return f"暂停着排头放得下而不起，过了阈值 {4 * SERVER.FAST_STARVED_SECONDS:.0f} 倍的时间 /health 照回 ok，control check 退 0"


def cell_pause_waits_for_starting_attempt(world):
    """pause 与起尝试共一把锁：排队线程判过放得下、记尝试之前停住，这时 pause 不回；放行、尝试记下之后 pause 才回，回话的在跑数里有这一条
    （等 control check 报在跑 0 个才做活，不会被正在起的一条挤进来）。"""
    pause_point = os.path.join(world.root, "pause-point")
    os.makedirs(pause_point)
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause_point}"})
    release = os.path.join(world.root, "starting-release")
    task_id = world.submit(blocking_command(release))
    open(os.path.join(pause_point, "arm-start-checked"), "w").close()
    world.resources(local=(30, 30))
    replies = []
    pauser = threading.Thread(target=lambda: replies.append(world.api("POST", "/pause", {"reason": PAUSE_REASON_SAMPLE})))
    try:
        wait_path(world, os.path.join(pause_point, "reached-start-checked"), "停在判过 queued 之后、记尝试之前")
        pauser.start()
        time.sleep(1.0)
        answered_early = bool(replies)
        open(os.path.join(pause_point, "release-start-checked"), "w").close()
        pauser.join(timeout=STATE_TIMEOUT_SECONDS)
        if answered_early or not replies:
            raise SelftestFailure(f"一条尝试正在起（判过放得下、还没记下）时 pause 应当等它记下再回，实际{'没等就回了' if answered_early else '放行之后也没回'}："
                                  f"{replies}", world.server_log_lines())
        status, reply = replies[0]
        if status != 200 or not reply.get("paused") or reply.get("running") != 1:
            raise SelftestFailure(f"pause 等正在起的那一条记下之后才回：应当 200、paused 为真、在跑 1 个，实际 HTTP {status}：{reply}", world.server_log_lines())
        world.running(task_id)
    finally:
        open(release, "w").close()
        if pauser.ident is not None:
            pauser.join(timeout=10)
    return "尝试正在起时 pause 不回，记下之后才回、在跑数里有这一条"


# ── 暂停时放行一个会话（task_server.py 文件头 ⑤「暂停」的放行一段；control pause --allow-session）──
def cell_pause_allows_one_session(world):
    """暂停时放行了一个会话：它交的普通任务、撞上限重排回队的任务、分片的各步（子任务随提交它的会话）照常排、照常起、跑到 succeeded；别的会话交的任务排着不起，一次尝试都没有。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    other_marker = os.path.join(world.root, "other-session-ran")
    allowed_marker = os.path.join(world.root, "allowed-session-ran")
    release = os.path.join(world.root, "allowed-retry-release")
    open(release, "w").close()   # 放行文件早就在：第 1 次尝试一起来就退 254（重排回队），第 2 次退 0
    other_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", other_marker], session=SESSION_B)
    plain_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", allowed_marker], session=SESSION_A)
    retried_id = world.submit(["bash", "-c", RETRY_AFTER_RELEASE_SCRIPT, "retry-after-release", release], session=SESSION_A)
    sharded_id = world.submit(world.write_spec("spec-allowed.json", {"shards": [{}]}), session=SESSION_A, cwd=world.repo)
    finished = {name: world.finished(task_id) for name, task_id in (("普通任务", plain_id), ("撞上限重排回队的任务", retried_id), ("分片的任务", sharded_id))}
    time.sleep(PAUSED_WATCH_SECONDS)
    problems = [f"放行会话交的{name}应当照常起、跑到 succeeded，实际 {describe(summary)}" for name, summary in finished.items() if summary["task"].get("outcome") != "succeeded"]
    if child_roles(finished["分片的任务"]) != ["merge", "plan", "shard"]:
        problems.append(f"放行会话交的分片任务的计划、片、合并三步都应当照常起，实际子任务 {child_roles(finished['分片的任务'])}")
    if not os.path.exists(allowed_marker):
        problems.append("放行会话交的普通任务的命令应当跑了（标记文件没留下）")
    if not held_in_queue(world, other_id) or os.path.exists(other_marker):
        problems.append(f"别的会话交的任务暂停着应当排着不起（queued、没有尝试、命令没跑），实际 {describe(world.summary(other_id))}、命令{'跑了' if os.path.exists(other_marker) else '没跑'}")
    if problems:
        raise SelftestFailure("暂停时放行一个会话：" + "；".join(problems), world.server_log_lines())
    return "放行会话的普通任务、撞上限重排回队的任务、分片的三步暂停着照常起跑完；别的会话的任务排了一阵都没起"


def cell_pause_allow_waiting_text(world):
    """暂停时放行了一个会话：别的会话排着的任务 status 写「在等：服务暂停中（原因：…）」；放行会话排着的任务不写暂停，写它真在等什么（这里是假余量写了 hold）。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    allowed_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    other_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    time.sleep(1)
    allowed_shown = world.client(["status", str(allowed_id)], session=SESSION_A)
    other_shown = world.client(["status", str(other_id)], session=SESSION_B)
    world.resources(local=(30, 30))
    world.finished(allowed_id)
    problems = []
    if "在等：自证的假余量写了 hold" not in allowed_shown.stdout or "服务暂停中" in allowed_shown.stdout:
        problems.append(f"放行会话排着的任务 status 应当写它真在等什么（「在等：自证的假余量写了 hold」）、不写暂停，实际「{allowed_shown.stdout.strip()[:300]}」")
    if f"在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）" not in other_shown.stdout:
        problems.append(f"别的会话排着的任务 status 应当写「在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）」，实际「{other_shown.stdout.strip()[:300]}」")
    if problems:
        raise SelftestFailure("放行时的在等字样：" + "；".join(problems), world.server_log_lines())
    return "放行会话排着的任务 status 写它真在等什么，别的会话排着的写在等服务暂停中"


def cell_pause_allow_switches_session(world):
    """control pause --allow-session 起手放行一个会话；已暂停着再 pause --allow-session 另一个会话退 0、说换了放行的会话、原因与起点不变：之后另一个会话早先交的排着的任务起来，原来放行的会话新交的反而排着；
    再放行同一个会话说原来就暂停着；操作日志有一条换放行会话。"""
    world.resources(local=(30, 30))
    world.start_server()
    problems = []
    paused = world.control(["pause", "--allow-session", SESSION_A, "--reason", PAUSE_REASON_SAMPLE])
    if paused.returncode != 0 or "暂停了" not in paused.stdout or SESSION_A not in paused.stdout:
        problems.append(f"control pause --allow-session 应当退 0、说暂停了并点名放行的会话，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-300:]}」、stderr「{paused.stderr.strip()[-200:]}」")
    started = (pause_health(world).get("pause") or {}).get("since")
    world.finished(world.submit(["bash", "-c", "exit 0"], session=SESSION_A))
    held_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    time.sleep(PAUSED_WATCH_SECONDS)
    if not held_in_queue(world, held_id):
        problems.append(f"换之前另一个会话交的任务应当排着不起，实际 {describe(world.summary(held_id))}")
    switched = world.control(["pause", "--allow-session", SESSION_B])
    if switched.returncode != 0 or "换了放行的会话" not in switched.stdout or SESSION_A not in switched.stdout or SESSION_B not in switched.stdout:
        raise SelftestFailure("已暂停着再 pause --allow-session 另一个会话应当退 0、说换了放行的会话并点名换前换后："
                              f"实际退 {switched.returncode}、stdout「{switched.stdout.strip()[-300:]}」、stderr「{switched.stderr.strip()[-300:]}」" + "；".join(problems), world.server_log_lines())
    summary = world.finished(held_id)
    late_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    time.sleep(PAUSED_WATCH_SECONDS)
    shown = pause_health(world).get("pause") or {}
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"换放行的会话之后，新放行的会话早先交的排着的任务应当起来跑完 succeeded，实际 {describe(summary)}")
    if not held_in_queue(world, late_id):
        problems.append(f"换放行的会话之后，原来放行的会话新交的任务应当排着不起，实际 {describe(world.summary(late_id))}")
    if shown.get("reason") != PAUSE_REASON_SAMPLE or shown.get("since") != started or shown.get("allow_session") != SESSION_B:
        problems.append(f"换放行的会话不改原因与起点：/health 应当是原因「{PAUSE_REASON_SAMPLE}」、起点 {started}、放行会话 {SESSION_B}，实际 {shown}")
    again = world.control(["pause", "--allow-session", SESSION_B])
    if again.returncode != 0 or "原来就暂停着" not in again.stdout or "换了" in again.stdout:
        problems.append(f"再放行同一个会话应当退 0、说原来就暂停着、不说换了，实际退 {again.returncode}、stdout「{again.stdout.strip()[-200:]}」")
    notes = [operation.get("note", "") for operation in world.operations() if operation.get("action") == "pause"]
    if not any("换放行会话" in note and SESSION_A in note and SESSION_B in note for note in notes):
        problems.append(f"操作日志里应当有一条 pause 备注「换放行会话 {SESSION_A} → {SESSION_B}」，实际 {notes}")
    if problems:
        raise SelftestFailure("换放行的会话：" + "；".join(problems), world.server_log_lines())
    return "pause --allow-session 起手放行一个会话；再放行另一个会话退 0、说换了，另一个会话排着的起来、原来放行的会话新交的排着，原因与起点不变；同一个会话再放行说原来就暂停着；操作日志有换放行会话"


def cell_resume_clears_allowed_session(world):
    """resume 把放行一并清掉：放行了一个会话的暂停 resume 之后，不带放行再 pause，回话里没有放行的会话，上一回放行的会话交的任务排着不起。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    status, resumed = world.api("POST", "/resume", {})
    if status != 200 or resumed.get("paused") or not resumed.get("changed"):
        raise SelftestFailure(f"resume 应当回 200、paused 为假、changed 为真，实际 HTTP {status}：{resumed}", world.server_log_lines())
    second = pause_by_api(world, "放开之后又暂停")
    task_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    time.sleep(PAUSED_WATCH_SECONDS)
    problems = []
    if second.get("allow_session") != "":
        problems.append(f"resume 之后不带放行再 pause，回话里不该还有放行的会话，实际 {second}")
    if not held_in_queue(world, task_id):
        problems.append(f"resume 之后不带放行再 pause，上一回放行的会话交的任务应当排着不起，实际 {describe(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("resume 清放行：" + "；".join(problems), world.server_log_lines())
    return "resume 之后不带放行再 pause：回话没有放行的会话，上一回放行的会话交的任务排着不起"


def cell_pause_allow_survives_restart(world):
    """放行的会话记在带签名的暂停态里：服务重启后照旧放行（/health 点名它，它新交的照起）；库里的放行会话被绕过服务改成另一个会话的，按暂停算、不放行谁、说明写校验不过，另一个会话的任务排着不起。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    world.stop_server()
    world.start_server()
    kept = pause_health(world).get("pause") or {}
    if kept.get("allow_session") != SESSION_A:
        raise SelftestFailure(f"服务重启后应当照旧放行会话 {SESSION_A}，实际 /health 的 pause {kept}", world.server_log_lines())
    after_restart = world.finished(world.submit(["bash", "-c", "exit 0"], session=SESSION_A))
    held_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE meta SET value = replace(value, ?, ?) WHERE name = ?",
                           (f'"allow_session":"{SESSION_A}"', f'"allow_session":"{SESSION_B}"', SERVER.PAUSE_META_NAME))
    world.start_server()
    time.sleep(PAUSED_WATCH_SECONDS)
    health = pause_health(world)
    tampered = health.get("pause") or {}
    problems = []
    if after_restart["task"].get("outcome") != "succeeded":
        problems.append(f"重启后放行会话新交的任务应当照起跑完，实际 {describe(after_restart)}")
    if not health.get("paused") or tampered.get("allow_session") != "" or "校验不过" not in (tampered.get("reason") or ""):
        problems.append(f"库里的放行会话被改过时应当按暂停算、不放行谁、原因写校验不过，实际 paused {health.get('paused')}、pause {tampered}")
    if not held_in_queue(world, held_id):
        problems.append(f"放行会话被改成另一个会话之后，它交的任务不该起，实际 {describe(world.summary(held_id))}")
    if problems:
        raise SelftestFailure("放行跨重启：" + "；".join(problems), world.server_log_lines())
    return "重启后照旧放行、放行会话新交的照起；库里的放行会话被改成另一个会话：按暂停算、不放行谁、写校验不过，另一个会话的任务排着"


def cell_pause_allow_reported(world):
    """放行了一个会话的暂停：control pause 点名放行的会话，--allow-session 不带值退 2；control check 与 /health 写「放行会话 <号>、在跑 N 个、别的会话在跑 M 个」，
    别的会话的任务跑完之后 M 变 0，N 里只剩放行会话的。"""
    world.resources(local=(30, 30))
    world.start_server()
    other_release = os.path.join(world.root, "other-release")
    allowed_release = os.path.join(world.root, "allowed-release")
    other_id = world.submit(blocking_command(other_release), session=SESSION_B)
    allowed_id = world.submit(blocking_command(allowed_release), session=SESSION_A)
    try:
        world.running(other_id)
        world.running(allowed_id)
        problems = []
        unfinished = world.control(["pause", "--allow-session"])
        if unfinished.returncode != lib.USAGE_EXIT:
            problems.append(f"--allow-session 不带值应当退 {lib.USAGE_EXIT}，实际退 {unfinished.returncode}")
        paused = world.control(["pause", "--reason", PAUSE_REASON_SAMPLE, "--allow-session", SESSION_A])
        wanted = f"放行会话 {SESSION_A}、在跑 2 个、别的会话在跑 1 个"
        if paused.returncode != 0 or "暂停了" not in paused.stdout or wanted not in paused.stdout:
            problems.append(f"control pause --allow-session 应当退 0、说暂停了、写「{wanted}」，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-300:]}」、"
                            f"stderr「{paused.stderr.strip()[-200:]}」")
        checked = world.control(["check"])
        if checked.returncode != 0 or wanted not in checked.stdout:
            problems.append(f"control check 应当退 0、写「{wanted}」，实际退 {checked.returncode}、stdout「{checked.stdout.strip()[-300:]}」")
        shown = pause_health(world).get("pause") or {}
        if shown.get("allow_session") != SESSION_A or shown.get("other_running") != 1:
            problems.append(f"/health 的 pause 应当带放行会话 {SESSION_A}、别的会话在跑 1 个，实际 {shown}")
        open(other_release, "w").close()
        world.finished(other_id)
        later = world.control(["check"])
        wanted = f"放行会话 {SESSION_A}、在跑 1 个、别的会话在跑 0 个"
        if later.returncode != 0 or wanted not in later.stdout:
            problems.append(f"别的会话的任务跑完之后 control check 应当写「{wanted}」，实际退 {later.returncode}、stdout「{later.stdout.strip()[-300:]}」")
    finally:
        open(other_release, "w").close()
        open(allowed_release, "w").close()
    world.finished(allowed_id)
    if problems:
        raise SelftestFailure("放行时的字样与计数：" + "；".join(problems), world.server_log_lines())
    return "control pause --allow-session 点名放行的会话、不带值退 2；check 与 /health 写放行会话与两种在跑数，别的会话的跑完后别的会话在跑 0 个"


SHARD_ADVANCE_PARENT_ID = 4242     # advance-episodes-survive-restart 的假分片父任务号


def cell_advance_episodes_survive_restart(world):
    """分片父任务推不动的这一回与开始过几回换服务后照接（task_server.py 的 note_advance_failure、note_advance_recovered、restore_restart_surviving_state）：
    推不动一回、推通、再推不动连着两轮（第 2 回、count 2）；服务停了再起（同一份库上另建调度器）：读回的是第 2 回、count 2，再推不动 count 3，推通之后再推不动是第 3 回。
    套在假调度器上，不起服务。弄坏开关 advance-episodes-memory-only 下换服务后从第 1 回数起，这一格红。"""
    parent = SHARD_ADVANCE_PARENT_ID
    before = restart_surviving_scheduler(world, "shard-advance.sqlite3")
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：头一回")
    SERVER.Scheduler.note_advance_recovered(before, parent)
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：第二回头一轮")
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：第二回第二轮")
    after = restart_surviving_scheduler(world, "shard-advance.sqlite3")
    restored = dict(after.advance_failures.get(parent) or {})
    SERVER.Scheduler.note_advance_failure(after, parent, "推不动：第二回第三轮")
    continued = dict(after.advance_failures.get(parent) or {})
    SERVER.Scheduler.note_advance_recovered(after, parent)
    SERVER.Scheduler.note_advance_failure(after, parent, "推不动：第三回")
    third = dict(after.advance_failures.get(parent) or {})
    problems = []
    if restored != {"episode": 2, "why": "推不动：第二回第二轮", "count": 2}:
        problems.append(f"换服务后读回的应当是第 2 回、连着 2 轮，实际 {restored}")
    if continued.get("episode") != 2 or continued.get("count") != 3:
        problems.append(f"换服务后再推不动应当还是第 2 回、count 3，实际 {continued}")
    if third.get("episode") != 3 or after.advance_episodes.get(parent) != 3:
        problems.append(f"推通之后再推不动应当是第 3 回，实际 {third}、开始过 {after.advance_episodes.get(parent)} 回")
    if problems:
        raise SelftestFailure("推不动的回数换服务不丢：" + "；".join(problems))
    return "推不动的这一回（第 2 回、count 2）换服务后照接，推通之后再推不动是第 3 回"


def cell_restart_surviving_tampered_not_adopted(world):
    """库里换服务不丢的那一份被绕过服务改过（task_server.py 的 Store.restart_surviving_state、restore_restart_surviving_state）：分片父任务推不动的一回记进库，
    再直接改库里那一行的正文多记一个父任务开始推不动过一回、签名不动；服务停了再起：那一份不认（一项都不读回）、写一条 record-tampered 告警点名 shard_advance。
    套在假调度器上，不起服务。弄坏开关 restart-surviving-signature-unchecked 下改过的照样读回，这一格红。"""
    database = os.path.join(world.root, "restart-surviving-tampered.sqlite3")
    before = restart_surviving_scheduler(world, os.path.basename(database))
    SERVER.Scheduler.note_advance_failure(before, SHARD_ADVANCE_PARENT_ID, "推不动：头一回")
    meta_name = SERVER.RESTART_SURVIVING_META_PREFIX + "shard_advance"
    with contextlib.closing(sqlite3.connect(database)) as connection:
        row = connection.execute("SELECT value FROM meta WHERE name = ?", (meta_name,)).fetchone()
        if row is None:
            raise SelftestFailure(f"记下推不动的一回之后库里应当有 {meta_name} 那一行，实际没有：改不了它、这一格判不了")
        stored = json.loads(row[0])
        stored["state"]["episodes"].append([SHARD_ADVANCE_PARENT_ID + 1, 1])
        connection.execute("UPDATE meta SET value = ? WHERE name = ?", (lib.canonical_json(stored), meta_name))
        connection.commit()
    after = restart_surviving_scheduler(world, os.path.basename(database))
    problems = []
    if after.advance_failures or after.advance_episodes:
        problems.append(f"签名对不上的那一份不该读回，实际推不动的记着 {after.advance_failures}、开始过的回数 {after.advance_episodes}")
    if not any(kind == "record-tampered" and "shard_advance" in line for kind, _, line in after.alerts):
        problems.append(f"应当写一条 record-tampered 告警点名 shard_advance，实际 {after.alerts}")
    if problems:
        raise SelftestFailure("改过的换服务不丢的那一份：" + "；".join(problems))
    return "库里那一份被绕过服务改过：不读回、写了 record-tampered 告警"


# ── 全量严格审核那一批（records/2026-11-07-task-server全量严格审核报告.md 缺陷一、三、四、五、七、八、九与整包测试的判法）──
TAMPERED_HOLDER_KEY = "selftest-tampered-holder"   # tampered-attempt-ended-once：在跑那一条的命令键（记 10G 峰值，上限与记账 15G）
TAMPERED_WAITER_KEY = "selftest-tampered-waiter"   # 那一格后交的那一条：同样 15G；本机 30G 放不下两条，前一条还算在跑就排着
TAMPERED_FOREIGN_COLUMN = 999999                   # 改到任务行关联列上的号（库里没有这一号的任务）
TAMPERED_SUMMARY_READS = 3                         # 收尾之后再读几回 status：每读一回都不该多一条告警
TAMPERED_WAITER_SECONDS = 30.0                     # 后交的那一条等跑起来的安静时限


def tampered_alerts(world, task_id):
    return [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]


def cell_tampered_attempt_ended_once(world):
    """核不过的行重签之后核得过、只告警一次（缺陷一；task_server.py 的 Store.write_row、Scheduler.tampered、Scheduler.end_tampered_attempt）：
    一条在排的任务只改它的关联列：服务记 record-tampered，那一行重签之后照服务的规则核得过，之后再读 TAMPERED_SUMMARY_READS 回 status 告警仍只一条；
    一条在跑的任务（本机 30G 上记 15G）只改它那一次尝试的关联列：对账记任务 record-tampered、那一次记 ended（核得过、关联列照正文挂回它名下），告警一条；
    之后交的那一条（同样记 15G）不再被它占着的内存挡住、照起。弄坏开关 tampered-rows-left-unrepaired 下关联列不写回、尝试留在在跑里，这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.seed_peak(TAMPERED_HOLDER_KEY, DRAINED_FRAGMENT_PEAK_BYTES)
    world.seed_peak(TAMPERED_WAITER_KEY, DRAINED_FRAGMENT_PEAK_BYTES)
    world.start_server()
    problems = []
    queued = world.submit(["bash", "-c", "exit 0", "tampered-queued"])
    raw_column_update(world, "tasks", TAMPERED_FOREIGN_COLUMN, queued)
    summary = world.finished(queued, timeout=20)
    for _ in range(TAMPERED_SUMMARY_READS):
        world.summary(queued)
    _, valid, column = verified_row(world, "tasks", queued)
    alerts = tampered_alerts(world, queued)
    if summary["task"].get("outcome") != "record-tampered" or not valid or column != 0 or len(alerts) != 1:
        problems.append(f"在排的任务只改了关联列：应当记 record-tampered、重签之后核得过（关联列 0）、再读 {TAMPERED_SUMMARY_READS} 回告警仍只 1 条，"
                        f"实际 {describe(summary)}、核{'得过' if valid else '不过'}（关联列 {column}）、告警 {len(alerts)} 条")
    world.resources(local=(30, 30))
    release = os.path.join(world.root, "release-tampered")
    try:
        holder = world.submit(blocking_command(release), extra={lib.KEY_VARIABLE: TAMPERED_HOLDER_KEY})
        attempt_id = world.running(holder)["attempts"][0]["id"]
        raw_column_update(world, "attempts", 0, attempt_id)
        finished = world.finished(holder, timeout=20)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        record, valid, column = verified_row(world, "attempts", attempt_id)
        while patience.waiting() and not (valid and (record or {}).get("state") == "ended"):
            time.sleep(POLL_SECONDS)
            record, valid, column = verified_row(world, "attempts", attempt_id)
        if finished["task"].get("outcome") != "record-tampered" or not valid or (record or {}).get("state") != "ended" \
                or (record or {}).get("outcome") != "record-tampered" or column != holder:
            problems.append(f"在跑那一次只改了关联列：任务应当记 record-tampered、那一次记 ended 与 record-tampered、核得过、关联列挂回任务 {holder}，"
                            f"实际任务 {describe(finished)}、那一次 {json.dumps(record, ensure_ascii=False)[:300]}、核{'得过' if valid else '不过'}、关联列 {column}")
        for _ in range(TAMPERED_SUMMARY_READS):
            world.summary(holder)
        if len(tampered_alerts(world, holder)) != 1:
            problems.append(f"在跑那一次核不过：告警应当只 1 条，实际 {len(tampered_alerts(world, holder))} 条")
        waiter = world.submit(blocking_command(release), extra={lib.KEY_VARIABLE: TAMPERED_WAITER_KEY})
        try:
            world.wait_for(waiter, lambda summary: any(attempt.get("state") == "running" for attempt in summary["attempts"]), "跑起来",
                           timeout=TAMPERED_WAITER_SECONDS)
        except SelftestFailure:
            problems.append(f"核不过的那一次记了 ended 之后不再占内存：后交的那一条（记 15G）应当在本机 30G 上照起，实际 {describe(world.summary(waiter))}、"
                            f"排队原因「{world.summary(waiter).get('waiting_for')}」")
    finally:
        open(release, "w").close()
    world.finished(waiter)
    if problems:
        raise SelftestFailure("核不过的行收尾：" + "；".join(problems), world.server_log_lines())
    return "关联列被改的任务行与尝试行：重签之后核得过、告警各一条，在跑那一次记 ended、不再占内存，后交的照起"


def cell_tampered_attempt_unit_stopped(world):
    """核不过的在跑尝试，服务停它的单元（task_server.py 的 stop_tampered_unit）：只改那一次尝试的关联列，对账记 ended 之后，它的单元回读不在跑、
    告警写 tampered-unit-stop 与「已停」。行的正文不可信那一半（单元名不合、是别的在跑尝试的不停）静态调 stop_tampered_unit 核：假 systemctl 没被调。
    弄坏开关 tampered-unit-left-running 下单元接着跑、没有这条告警，这一格红。"""
    scheduler = unittest.mock.Mock(environment={})
    scheduler.store.tag = "selftest"
    scheduler.store.running_attempts.return_value = [{"unit": "singlefs-task-selftest-1-1.service"}]
    for name in ("sshd.service", "singlefs-task-other-1-1.service", "singlefs-task-selftest-1-1.service", None):
        SERVER.Scheduler.stop_tampered_unit(scheduler, name, 1, None)
    lines = [call.args[2] for call in scheduler.alert.call_args_list]
    if len(lines) != 4 or not all("没停" in line for line in lines):
        raise SelftestFailure(f"单元名不是这个服务起的、或是校验过的在跑尝试的，应当都不停、照实报「没停」，实际 {lines}")
    world.resources(local=(30, 30))
    world.start_server()
    release = os.path.join(world.root, "release-tampered-unit")
    try:
        holder = world.submit(blocking_command(release))
        attempt = world.running(holder)["attempts"][0]
        unit = attempt.get("unit") or ""
        raw_column_update(world, "attempts", 0, attempt["id"])
        world.finished(holder, timeout=20)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        alerts = []
        while patience.waiting() and not alerts:
            alerts = [alert for alert in world.alerts() if alert.get("kind") == "tampered-unit-stop" and alert.get("task") == holder]
            time.sleep(POLL_SECONDS)
        active = subprocess.run(["systemctl", "--user", "is-active", unit], capture_output=True, text=True).stdout.strip() if unit else ""
        if not unit or not alerts or "已停" not in alerts[0].get("line", "") or active in lib.RUNNING_UNIT_STATES:
            raise SelftestFailure(f"核不过的在跑尝试应当被停掉：单元 {unit or '（没记）'} 回读 {active or '没读'}、告警 {alerts}", world.server_log_lines())
    finally:
        open(release, "w").close()
    return "核不过的在跑尝试记 ended 之后，它的单元被停掉、回读不在跑，告警写已停"


def cell_pause_keeps_queue_positions(world):
    """暂停放行一个会话时别的会话排着的任务排位不丢（task_server.py 的 place_queued_into、positions_in）：假余量 hold，放行会话 A；会话 B 先交一条、A 后交一条：
    两条的 status 排位照全队算（B 那条 1/2、A 那条 2/2）。弄坏开关 pause-queue-positions-filtered 下只照放行的那几条算（B 那条没有排位、A 那条 1/1），这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    other_id = world.submit(["bash", "-c", "exit 0", "paused-other"], session=SESSION_B)
    allowed_id = world.submit(["bash", "-c", "exit 0", "paused-allowed"], session=SESSION_A)
    world.next_pass()
    world.next_pass()
    other, allowed = (world.summary(task_id).get("queue_position") or {} for task_id in (other_id, allowed_id))
    seen = ((other.get("position"), other.get("queued")), (allowed.get("position"), allowed.get("queued")))
    if seen != ((1, 2), (2, 2)):
        raise SelftestFailure(f"暂停放行 A 时两条的排位应当照全队算（B 那条 1/2、A 那条 2/2），实际 {seen}", world.server_log_lines())
    return "暂停放行一个会话：别的会话排着的排位照全队算"


def cell_build_directory_outside_repository(world):
    """会编的 cargo 编在仓外、服务给的编译目录里（这棵树在本机的组里那一个，task_server_lib.py 文件头「编译目录」）：假 cargo 拿到的 CARGO_TARGET_DIR 在
    TASK_SERVER_BUILD_DIRECTORY 下、尝试记着它的上一层、起它的目录下没长 target/；记下结局之后那个编译目录留着（归组、不归任务），wait 末行不带 build=；
    别的会话 task clean 被拒（退 3），本会话的退 0。调用方给的编译目录指到仓里的当场拒；给了仓外的照它用、不另给、不删。"""
    world.start_server()
    path = fake_cargo_extra(world)["PATH"]
    task_id = world.submit(["cargo", "build"], extra={"PATH": path})
    summary = world.finished(task_id)
    targets = cargo_targets(world)
    target = targets[-1] if targets else ""
    attempt = summary["attempts"][0] if summary["attempts"] else {}
    problems = []
    if summary["task"].get("outcome") != "succeeded" or not target:
        problems.append(f"假 cargo 应当跑完、记下给它的 CARGO_TARGET_DIR，实际 {describe(summary)}、拿到的 {targets}")
    elif not lib.inside(target, world.builds) or lib.inside_repository(target):
        problems.append(f"这一次尝试的编译目录应当在 {world.builds} 下（仓外），实际 {target}")
    elif not os.path.isdir(os.path.dirname(target)):
        problems.append(f"记下结局之后编译目录 {os.path.dirname(target)} 应当留着（归这棵树的组，树删了才删），实际不在")
    else:
        line = world.client(["wait", str(task_id)]).stdout.strip().splitlines()[-1:]
        if not line or " build=" in line[0]:
            problems.append(f"wait 的末行不该带 build=（编译目录归树、不归任务），实际 {line}")
        refused = world.client(["clean", str(task_id)], session=SESSION_B)
        if refused.returncode != 3:
            problems.append(f"别的会话 task clean 应当被拒（退 3），实际退 {refused.returncode}：{refused.stderr.strip()[-200:]}")
        cleaned = world.client(["clean", str(task_id)])
        if cleaned.returncode != 0:
            problems.append(f"同一会话 task clean 应当退 0，实际退 {cleaned.returncode}、输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}")
    if os.path.lexists(os.path.join(world.work, "target")):
        problems.append(f"起它的目录 {world.work} 下长出了 target/（没给编译目录，cargo 照默认编在了起它的目录下）")
    if attempt.get("build_directory") != (os.path.dirname(target) if target else None):
        problems.append(f"尝试记录里的编译目录应当是 {os.path.dirname(target) if target else '?'}，实际 {attempt.get('build_directory')}")
    for refused_extra, refused_command in (({"CARGO_TARGET_DIR": os.path.join(lib.REPOSITORY_ROOT, "target")}, ["cargo", "build", "fake-wrapper-exits=0"]),
                                           ({}, ["cargo", "build", "--target-dir", os.path.join(lib.REPOSITORY_ROOT, "selftest-target"), "fake-wrapper-exits=0"])):
        completed = world.client(["submit", "--", *refused_command], extra=dict(refused_extra, PATH=path))
        if completed.returncode != 3 or "指到仓里" not in completed.stderr:
            problems.append(f"调用方给的编译目录指到仓里（{refused_extra or refused_command[3]}）应当当场拒（退 3、说指到仓里），实际退 {completed.returncode}")
    own = os.path.join(world.root, "own-target")
    kept = world.finished(world.submit(["cargo", "build"], extra={"PATH": path, "CARGO_TARGET_DIR": own}))
    if kept["task"].get("outcome") != "succeeded" or not os.path.isfile(os.path.join(own, "debug", "selftest-artifact")) or kept["attempts"][0].get("build_directory"):
        problems.append(f"调用方给的仓外编译目录应当照它用、不删、不另给，实际 {describe(kept)}、产物{'在' if os.path.isfile(os.path.join(own, 'debug', 'selftest-artifact')) else '不在'}")
    if problems:
        raise SelftestFailure("编译目录：" + "；".join(problems), world.server_log_lines())
    return (f"会编的 cargo 编在仓外的 {os.path.relpath(target, world.root)}、跑完留着、wait 末行不带 build=、别的会话 clean 拒、本会话 clean 退 0，"
            "起它的目录没长 target/；指到仓里的当场拒；调用方给的仓外目录照用不删")


def cell_build_slot_reused_on_same_tree(world):
    """同一棵树上先后两条会编的任务拿到同一个编译目录（task_server_lib.py 文件头「编译目录」；task_server.py 的 take_build_slot）：这一格自己建的树里先后交两条假 cargo，
    后一条的尝试记着前一条那一个、记 build_directory_reused，两次假 cargo 拿到的 CARGO_TARGET_DIR 是同一个；目录的名字不带任何一次尝试的标签（标签里有任务号，
    接着用它的后一条任务的进程在 ps 里会挂着前一条的号）。弄坏开关 build-slot-per-attempt 下每次新开一个、build-directory-named-by-label 下目录照开它的那一次尝试的标签起名，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    tree = build_slot_tree(world, "tree-reused")
    extra = fake_cargo_extra(world)
    first = world.finished(world.submit(["cargo", "build"], extra=extra, cwd=tree))
    second = world.finished(world.submit(["cargo", "build"], extra=extra, cwd=tree))
    directories = [first_attempt(found).get("build_directory") or "" for found in (first, second)]
    targets = cargo_targets(world)
    if (any(found["task"].get("outcome") != "succeeded" for found in (first, second)) or not all(directories) or directories[0] != directories[1]
            or not first_attempt(second).get("build_directory_reused") or targets != [os.path.join(directories[0], "target")] * 2):
        raise SelftestFailure(f"同一棵树上先后两条会编的任务应当拿到同一个编译目录、后一条记 build_directory_reused、两次 CARGO_TARGET_DIR 相同，"
                              f"实际 {describe(first)}、{describe(second)}、编译目录 {directories}、后一条 build_directory_reused="
                              f"{first_attempt(second).get('build_directory_reused')}、假 cargo 拿到 {targets}", world.server_log_lines())
    labels = [first_attempt(found).get("label") or "" for found in (first, second)]
    name = os.path.basename(directories[0])
    if not all(labels) or any(label in name for label in labels):
        raise SelftestFailure(f"编译目录的名字不该带任何一次尝试的标签（带了任务号，后一条接着用时 ps 里认错是哪条任务的），实际目录 {name}、两次尝试的标签 {labels}", world.server_log_lines())
    return f"同一棵树上先后两条会编的任务都编在 {os.path.relpath(directories[0], world.root)}（名字不带任务号），后一条接着用"


def cell_build_slot_separate_per_tree(world):
    """不同的树拿到不同的编译目录（task_server_lib.py 文件头「编译目录」：不同的树从不共用一个）：这一格建两棵树，各交一条假 cargo（先后交，前一条跑完后一条才起，
    组里有空着的），两条的编译目录不同，各自的尝试记着自己那棵树。弄坏开关 build-slot-tree-ignored 下不分树，后一条拿到前一条那一个，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    trees = [build_slot_tree(world, name) for name in ("tree-one-side", "tree-other-side")]
    extra = fake_cargo_extra(world)
    summaries = [world.finished(world.submit(["cargo", "build"], extra=extra, cwd=tree)) for tree in trees]
    directories = [first_attempt(found).get("build_directory") or "" for found in summaries]
    recorded = [first_attempt(found).get("build_tree") or "" for found in summaries]
    if (any(found["task"].get("outcome") != "succeeded" for found in summaries) or not all(directories) or directories[0] == directories[1]
            or recorded != [os.path.realpath(tree) for tree in trees]):
        raise SelftestFailure(f"两棵不同的树应当各拿各的编译目录、尝试各记自己那棵树，实际 {[describe(found) for found in summaries]}、编译目录 {directories}、"
                              f"记的树 {recorded}（应当是 {[os.path.realpath(tree) for tree in trees]}）", world.server_log_lines())
    return "两棵不同的树各拿各的编译目录：" + "、".join(os.path.relpath(directory, world.root) for directory in directories)


def cell_build_slot_concurrent_distinct(world):
    """同一棵树同时两条会编的任务拿到组里不同的两个（task_server.py 的 held_build_slots：占着的不给别的）：两条都编完再占着到放行为止，
    两条在跑的这一会儿编译目录不同；放行之后都跑完。弄坏开关 build-slot-held-shared 下后起的那一条拿到先起那一条占着的那一个，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    tree = build_slot_tree(world, "tree-concurrent")
    release = os.path.join(world.root, "release-build-slot-concurrent")
    extra = fake_cargo_extra(world)
    task_ids = []
    try:
        for name in ("concurrent-one-side", "concurrent-other-side"):
            task_ids.append(world.submit(["bash", "-c", "cargo build && " + HOLD_UNTIL_RELEASED_SCRIPT, name, release], extra=extra, cwd=tree))
        running = [world.running(task_id) for task_id in task_ids]
        directories = [next((attempt.get("build_directory") or "" for attempt in found["attempts"] if attempt.get("state") == "running"), "") for found in running]
    finally:
        open(release, "w").close()
    finished = [world.finished(task_id) for task_id in task_ids]
    problems = []
    if not all(directories) or directories[0] == directories[1]:
        problems.append(f"同一棵树同时在跑的两条应当拿到组里不同的两个编译目录，实际 {directories}")
    failed = [describe(found) for found in finished if found["task"].get("outcome") != "succeeded"]
    if failed:
        problems.append(f"放行之后两条都应当跑完，实际没成的 {failed}")
    if problems:
        raise SelftestFailure("同一棵树同时两条：" + "；".join(problems), world.server_log_lines())
    return "同一棵树同时在跑的两条拿到组里不同的两个：" + "、".join(os.path.relpath(directory, world.root) for directory in directories)


POOL_GONE_SECONDS = 20.0   # build-pool-removed-with-tree：服务重新起来之后至多等这么久看树不在了的那一组删了（起来时那一扫在后台线程里）


def cell_build_pool_removed_with_tree(world):
    """树删了服务起来时扫掉整组（task_server.py 的 sweep_vanished_tree_pools）：两棵树各编过一次，停服务、删掉其中一棵，再起服务：删掉的那棵树的编译目录没了，
    还在的那棵树的照旧留着。弄坏开关 vanished-tree-pool-kept 下不删，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    extra = fake_cargo_extra(world)
    vanishing, staying = build_slot_tree(world, "tree-vanishing"), build_slot_tree(world, "tree-staying")
    summaries = [world.finished(world.submit(["cargo", "build"], extra=extra, cwd=tree)) for tree in (vanishing, staying)]
    gone, kept = (first_attempt(found).get("build_directory") or "" for found in summaries)
    if any(found["task"].get("outcome") != "succeeded" for found in summaries) or not os.path.isdir(gone) or not os.path.isdir(kept) or gone == kept:
        raise SelftestFailure(f"两棵树各编一次应当都跑完、各留一个编译目录，实际 {[describe(found) for found in summaries]}、编译目录「{gone}」「{kept}」", world.server_log_lines())
    world.stop_server()
    shutil.rmtree(vanishing)
    world.start_server()
    patience = world.patience(POOL_GONE_SECONDS)
    while os.path.lexists(gone) and patience.waiting():
        time.sleep(POLL_SECONDS)
    problems = []
    if os.path.lexists(gone):
        problems.append(f"树 {vanishing} 删了、服务重新起来之后 {POOL_GONE_SECONDS:.0f} 秒里它的编译目录 {gone} 应当删了，实际还在")
    if not os.path.isdir(kept):
        problems.append(f"还在的树 {staying} 的编译目录 {kept} 应当照旧留着，实际不在了")
    if problems:
        raise SelftestFailure("树删了扫掉整组：" + "；".join(problems), world.server_log_lines())
    return "树删了之后服务起来时扫掉了它的编译目录，还在的树的照旧留着"


UNPOOLED_TASK_NUMBER = 7001            # unpooled-label-directories-removed：编译根下摆的几个照尝试标签起名的目录用的任务号（这一格的库里没有这些任务）
UNPOOLED_OTHER_TAG = "other-library"   # 同上：别的库标签（这一格的库标签是 8 位十六进制，与它不会相同）
UNPOOLED_HOLDER_SECONDS = 600          # 同上：开着那一个目录的 sleep 睡多久（这一格收尾时对它的进程组发 KILL，不等它睡完）


def cell_unpooled_label_directories_removed(world):
    """本机编译根下不在任何组里、照尝试标签起名的目录，服务起来时那一扫删掉（task_server.py 的 sweep_unpooled_attempt_label_directories）：服务起一次建库、停掉，
    在编译根下摆五个目录再起：不在组里的 task-<这一格的库标签>-<号>-<次> 删了、回读不在；同样名字而有进程开着的（这一格起的 sleep，工作目录在里面）不删、写告警
    build-directory-left；组里记着的同样名字的、别的库标签的、别的名字的都照旧在；那一扫打出汇总行。sleep 自成一个进程组，收尾对整组发 KILL。
    弄坏开关 unpooled-label-directories-kept 下不删，unpooled-label-directory-open-unchecked 下有进程开着的也删，unpooled-label-directories-pool-unread 下组里记着的也删，这一格都红。"""
    world.start_server()
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30)) as connection:
        (tag,) = connection.execute("SELECT value FROM meta WHERE name = 'tag'").fetchone()
    names = {"不在组里的": f"task-{tag}-{UNPOOLED_TASK_NUMBER}-1", "有进程开着的": f"task-{tag}-{UNPOOLED_TASK_NUMBER + 1}-1",
             "组里记着的": f"task-{tag}-{UNPOOLED_TASK_NUMBER + 2}-1", "别的库标签的": f"task-{UNPOOLED_OTHER_TAG}-{UNPOOLED_TASK_NUMBER}-1",
             "别的名字的": "selftest-unpooled-other-name"}
    directories = {role: os.path.join(world.builds, name) for role, name in names.items()}
    for directory in directories.values():
        os.makedirs(os.path.join(directory, "target", "debug"), exist_ok=True)
        with open(os.path.join(directory, "target", "debug", "selftest-artifact"), "w", encoding="utf-8") as handle:
            handle.write("selftest\n")
    tree = os.path.realpath(build_slot_tree(world, "tree-unpooled-pooled"))
    now = round(time.time(), 3)
    world.insert_signed_row("build_slots", {"task_id": 0, "state": "present", "session": "", "command_key": tree, "tree": tree, "host": SERVER.LOCAL,
                                            "directory": directories["组里记着的"], "opened_by": names["组里记着的"], "opened_at": now, "used_at": now, "ready": True})
    holder = subprocess.Popen(["sleep", str(UNPOOLED_HOLDER_SECONDS)], cwd=directories["有进程开着的"], stdin=subprocess.DEVNULL, start_new_session=True)
    problems = []
    try:
        world.start_server()
        server_log = os.path.join(world.root, f"server-{world.server_logs}.log")
        patience = world.patience(POOL_GONE_SECONDS)
        while SERVER.UNPOOLED_SWEEP_SUMMARY_MARK not in read_text(server_log) and patience.waiting():
            time.sleep(POLL_SECONDS)
        alerted = any(alert.get("kind") == "build-directory-left" and directories["有进程开着的"] in (alert.get("line") or "") for alert in world.alerts())
        if SERVER.UNPOOLED_SWEEP_SUMMARY_MARK not in read_text(server_log):
            problems.append(f"服务重新起来之后 {POOL_GONE_SECONDS:.0f} 秒里编译根那一扫应当打出「{SERVER.UNPOOLED_SWEEP_SUMMARY_MARK}」的汇总行，实际没打")
        if os.path.lexists(directories["不在组里的"]):
            problems.append(f"不在组里的 {directories['不在组里的']} 应当删了、回读不在，实际还在")
        for role in ("有进程开着的", "组里记着的", "别的库标签的", "别的名字的"):
            if not os.path.isfile(os.path.join(directories[role], "target", "debug", "selftest-artifact")):
                problems.append(f"{role} {directories[role]} 应当照旧在，实际不在了")
        if not alerted:
            problems.append(f"有进程开着的 {directories['有进程开着的']} 没删成，应当写告警 build-directory-left 点名它，实际告警 {world.alerts()}")
    finally:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(holder.pid, signal.SIGKILL)   # 整组都是这一格自己起的
        holder.wait()
    if problems:
        raise SelftestFailure("编译根下照尝试标签起名、不在组里的目录：" + "；".join(problems), world.server_log_lines())
    return "编译根下不在组里、照尝试标签起名的删了；有进程开着的留着、写了告警；组里记着的、别的库标签的、别的名字的照旧在"


UNPOOLED_RUNNING_TASK_NUMBER = 7101   # unpooled-label-directory-of-running-attempt-kept：编译根下摆的两个照尝试标签起名的目录用的任务号（这一格的库里没有这些任务）


def cell_unpooled_label_directory_of_running_attempt_kept(world):
    """编译根那一扫（task_server.py 的 sweep_unpooled_attempt_label_directories）不删在起、在跑的尝试记着的目录：服务起一次建库、停掉，在编译根下摆两个
    task-<这一格的库标签>-<号>-1，库里记一条在跑的尝试、它的 build_directory 指着其中一个（组里没有那一行、没有进程开着里面的文件，只靠在跑的尝试记着它；
    尝试不挂在任何任务名下，对账照「库里没有它那条任务」不动它），再起：尝试记着的那一个照旧在，没人记着的那一个删了、回读不在，那一扫打出汇总行。
    弄坏开关 unpooled-label-directories-running-unread 下尝试记着的也删，unpooled-label-directories-kept 下一个都不删、不打汇总行，这一格都红。"""
    world.start_server()
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30)) as connection:
        (tag,) = connection.execute("SELECT value FROM meta WHERE name = 'tag'").fetchone()
    directories = {role: os.path.join(world.builds, f"task-{tag}-{number}-1")
                   for role, number in (("在跑的尝试记着的", UNPOOLED_RUNNING_TASK_NUMBER), ("没人记着的", UNPOOLED_RUNNING_TASK_NUMBER + 1))}
    for directory in directories.values():
        os.makedirs(os.path.join(directory, "target", "debug"), exist_ok=True)
        with open(os.path.join(directory, "target", "debug", "selftest-artifact"), "w", encoding="utf-8") as handle:
            handle.write("selftest\n")
    world.insert_signed_row("attempts", {"task_id": 0, "number": 1, "state": "running", "session": "", "command_key": "selftest-unpooled-running",
                                         "build_directory": directories["在跑的尝试记着的"], "started_at": round(time.time(), 3)})
    world.start_server()
    server_log = os.path.join(world.root, f"server-{world.server_logs}.log")
    patience = world.patience(POOL_GONE_SECONDS)
    while SERVER.UNPOOLED_SWEEP_SUMMARY_MARK not in read_text(server_log) and patience.waiting():
        time.sleep(POLL_SECONDS)
    problems = []
    if SERVER.UNPOOLED_SWEEP_SUMMARY_MARK not in read_text(server_log):
        problems.append(f"服务重新起来之后 {POOL_GONE_SECONDS:.0f} 秒里编译根那一扫应当打出「{SERVER.UNPOOLED_SWEEP_SUMMARY_MARK}」的汇总行，实际没打")
    if os.path.lexists(directories["没人记着的"]):
        problems.append(f"没人记着的 {directories['没人记着的']} 应当删了、回读不在，实际还在")
    if not os.path.isfile(os.path.join(directories["在跑的尝试记着的"], "target", "debug", "selftest-artifact")):
        problems.append(f"在跑的尝试记着的 {directories['在跑的尝试记着的']} 应当照旧在（组里没有那一行，那一次尝试还在用它），实际不在了")
    if problems:
        raise SelftestFailure("编译根下照尝试标签起名、在跑的尝试记着的目录：" + "；".join(problems), world.server_log_lines())
    return "编译根下在跑的尝试记着的那一个照旧在，没人记着的删了"


def cell_clean_keeps_build_pool(world):
    """task clean 不删整组（task_server_lib.py 文件头「编译目录」：clean 只删那一条任务自己的现场）：这一格的树里编过一次，task clean 退 0、删了 0 份，
    那个编译目录照旧在。弄坏开关 clean-removes-build-slot 下 clean 照删组里那一个，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    tree = build_slot_tree(world, "tree-cleaned")
    task_id = world.submit(["cargo", "build"], extra=fake_cargo_extra(world), cwd=tree)
    summary = world.finished(task_id)
    build = first_attempt(summary).get("build_directory") or ""
    if summary["task"].get("outcome") != "succeeded" or not os.path.isdir(build):
        raise SelftestFailure(f"假 cargo 应当跑完、留下组里那一个编译目录，实际 {describe(summary)}、编译目录「{build}」", world.server_log_lines())
    cleaned = world.client(["clean", str(task_id)])
    if cleaned.returncode != 0 or "删了 0 份" not in cleaned.stdout or not os.path.isdir(build):
        raise SelftestFailure(f"task clean 应当退 0、删了 0 份、编译目录 {build} 照旧在（归这棵树的组），实际退 {cleaned.returncode}、"
                              f"编译目录{'在' if os.path.isdir(build) else '不在了'}、输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}", world.server_log_lines())
    return "task clean 退 0、删了 0 份，这棵树组里的编译目录照旧在"


def cell_clean_failure_explained(world):
    """task clean 没删成时（现场目录里有一层子目录去掉了写权限，里面的文件删不掉）退 5，逐份写明是哪一份现场、为什么（remove_scenes 交回的原因），操作日志同样写原因；
    把权限改回来再 clean 退 0、删了。提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：现场留到 task clean。"""
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"], extra=KEEP_TEMPORARY_EXTRA)
    summary = world.finished(task_id)
    scene, _ = scene_of(world, summary)
    if not scene or not os.path.isdir(scene):
        raise SelftestFailure(f"退 {SCENE_COMMAND_EXIT} 的那一次应当留下现场（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1），实际 {describe(summary)}、现场「{scene}」",
                              world.server_log_lines())
    locked = os.path.join(scene, "locked-by-selftest")
    os.makedirs(locked)
    with open(os.path.join(locked, "held"), "w", encoding="utf-8") as handle:
        handle.write("selftest\n")
    os.chmod(locked, 0o500)
    problems = []
    try:
        stuck = world.client(["clean", str(task_id)])
        reason = "rm 之后还在"   # remove_scenes 交回的那一句
        noted = [operation.get("note", "") for operation in world.operations() if operation.get("action") == "clean" and operation.get("task") == task_id]
        if stuck.returncode != 5 or f"现场 {scene}" not in stuck.stderr or reason not in stuck.stderr or not os.path.isdir(scene):
            problems.append(f"现场里有删不掉的文件时 task clean 应当退 5、写明没删成的是现场 {scene}、为什么（{reason}），现场照留，"
                            f"实际退 {stuck.returncode}、现场{'在' if os.path.isdir(scene) else '不在'}、stderr「{stuck.stderr.strip()[-400:]}」")
        if not noted or reason not in noted[-1]:
            problems.append(f"没删成的那一次操作日志应当写原因（{reason}），实际 {noted}")
    finally:
        os.chmod(locked, 0o700)
    retried = world.client(["clean", str(task_id)])
    if retried.returncode != 0 or os.path.lexists(scene):
        problems.append(f"权限改回来再 clean 应当退 0、删了现场，实际退 {retried.returncode}、现场{'还在' if os.path.lexists(scene) else '不在'}")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return "现场里有删不掉的文件时 clean 退 5、写明哪一份现场为什么、操作日志带原因；权限改回来再 clean 删了"
