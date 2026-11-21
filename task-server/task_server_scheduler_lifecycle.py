"""task_server.py 拆出的一份：Scheduler 的一部分方法：操作与告警记录、换服务不丢的状态、暂停、主循环、第二台健康、对账、判卡住、跑满上限、疑似卡住、要处置的提醒项（Scheduler 在 task_server.py，经继承取到它们）。"""
import os
import sys
import task_server_lib as lib
import contextlib
import dataclasses
import json
import re
import shlex
import signal
import sqlite3
import subprocess
import threading
import time
import traceback
from typing import NamedTuple
from task_server_constants import (
    ACKNOWLEDGED_KINDS, ACTIVE_ATTEMPT_STATES, ATTENTION_HISTORY_KEPT, ENDED_TAMPERED_FIELDS, FILLS_CAP_FRACTION, FILLS_CAP_RUNS, GPU_NOT_RUN_MARK,
    HOOK_SWAP_ENTRY, INBOX_HEADLINE, LOCAL, LONG_RUN_IDLE_CORE_FRACTION, LONG_RUN_IDLE_QUARTER_BREAK, LONG_RUN_SPLIT_EXEMPT_FILE,
    NOT_RUN_MARKS_COLUMN, PATCH_WRITER_CHECK_SECONDS, PATCH_WRITER_OUTCOME_NAME, PATCH_WRITER_READ_ONLY_ARGUMENTS, PATCH_WRITER_VERDICTS, PEER,
    PEER_REPASS_SECONDS, PEER_SCOPE_LEFTOVERS_MARK, PREVIOUS_SWAP_RUNNING_EXIT_CODE, PRUNE_SERVER_CHECK_SECONDS, RESIDENT_RETENTION_SECONDS,
    RESIDENT_TASK_TABLES, RESTART_SURVIVING_STATES, SIGNAL_EXIT_CODES, SPARED_ACTIVITY, SPARED_CPU_BUSY, SPARED_CPU_UNREAD, SPARED_IO_WAIT,
    STALL_AT_CAP_FRACTION, STALL_CPU_CORES, STOP_CHILD_GRACE_SECONDS, STOP_JOIN_SECONDS, SWAP_SCRATCH_BUSY_EXIT_CODE, UNIT_STOP_SECONDS,
    WORKSPACE_BLOCKED_EXIT_CODE, WORKSPACE_WRITERS, WORKSPACE_WRITER_INTERRUPTED_OUTCOMES, WRITER_PATCH_ENTRY, append_log_lines, append_private_line,
    association_id, gpu_not_run_text, log_tail, read_json_file, say, task_scope_text,
)
from task_server_store import StoreRefused
from task_server_machines import (
    IoWaitWindow, MACHINE_ORDER, SuspectWindow, activity_text, activity_window_after, descendant_processes, io_pressure_text, next_reminder_seconds,
    process_is_zombie, recorded_text, reminder_threshold, unit_processes, unit_states,
)


# scripts/gate/hook-swap.py 退 10 的两种拒绝说明（hook_swap_swap.py 的 scratch_busy_lines、repository_busy_lines）各自起头的那一句：swap_busy_cause 照日志里有哪一句认占着的是哪一种
SWAP_BUSY_SCRATCH_LINE = re.compile(r"✗ 草稿目录 .+ 正被另一趟 hook-swap\.py 占着（")
SWAP_BUSY_REPOSITORY_LINE = re.compile(r"✗ 这个仓的换上正被另一趟 hook-swap\.py 占着（")
SWAP_BUSY_SCRATCH, SWAP_BUSY_REPOSITORY = "scratch", "repository"
# hook-swap.py --read-outcome 的退出码（它文件头「结局文件」）：0、1 不在半截上，3 的输出里「→ 怎么办」那一行点名开始行记的反向规格
HOOK_SWAP_VERDICTS = {0: "换完了", 1: "已退回", 2: "读不了（没有结局文件或不是结局文件）", 3: "没换完", 4: "完成或退回之后文件又被改了"}
HOOK_SWAP_UNFINISHED_VERDICT = 3
# 反向规格写在 <草稿目录>/hook-swap/reverse-<时刻>.json（hook-swap.py 文件头「撤回一次换上」）：换钩子退 3 的输出里点名的反向规格在哪个 --scratch 下，结局文件就是哪一趟的
HOOK_SWAP_REVERSE_SPEC_PARTS = ("hook-swap", "reverse-")


class InterruptedWriter(NamedTuple):
    """writer_interruption 判出可能写了一半的一条写工作区工具（workspace_write_judged 把几条并成一项）。"""
    writer: str          # 写工具那个词（写进 why）
    basis: str           # 接在依据后面的一段（结局文件判了什么、补丁目录算不出），没有是空串
    check: str           # 出路里要核什么（不带 git status 那一句）
    log_pointed: bool    # 出路末尾要不要指到这条任务的日志


class PatchOutcomeReadBack(NamedTuple):
    """writer_interruption 判出要读结局文件的那一条 apply-writer-patch.py 或 hook-swap.py：workspace_write_judged 起 read_back，退 0、1 的不报，
    别的照 unread 报、依据接上它打的末几行（换钩子退 3 而结局文件归这条任务里另一条换钩子的不报，owner_marks）。
    起它的那一处写在 workspace_write_judged 里：算键库按（文件, 方法限定名, 命令那个词）在 scripts/gate/lib_function_units_registry.py 的 UNKEYED_PROGRAM_WORDS
    登记这一处不拒算，起法写在别的方法里对不上那一行、拒算 subprocess-path-unfolded。"""
    read_back: list               # [解释器, 写工具, --read-outcome, …]：打补丁的接结局文件、带 --root 的接上它与它的值；换钩子的接它自己的 --root、--scratch 与值
    directory: str                # 起它的当前目录
    label: str                    # 依据里读的是哪一份（「结局文件 <路径>」，或换钩子的「结局文件（hook-swap.py --read-outcome …）」）
    verdicts: dict                # --read-outcome 退出码 → 判了什么（PATCH_WRITER_VERDICTS、HOOK_SWAP_VERDICTS）
    owner_marks: tuple            # 换钩子：它的反向规格路径的开头（HOOK_SWAP_REVERSE_SPEC_PARTS，绝对与解开链接两种写法），退 3 的输出里点名的反向规格认这条换钩子；打补丁的是空的
    unread: InterruptedWriter     # 读出来不是打全了、已退回时报的那一条（依据由读的结果补上）


class SchedulerLifecycle:
    """Scheduler 的一部分方法：操作与告警记录、换服务不丢的状态、暂停、主循环、第二台健康、对账、判卡住、跑满上限、疑似卡住、要处置的提醒项（Scheduler 在 task_server.py，经继承取到它们）。"""

    def alert(self, kind, task_id, line, next_step):
        record = {"time": round(time.time(), 3), "kind": kind, "task": task_id, "line": line, "next_step": next_step}
        self.store.insert("alerts", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.alerts_log, lib.canonical_json(record))
        say(f"  ✗ 告警（{kind}）任务 {task_id}：{line}")
        say(f"     → 怎么办：{next_step}")

    # ── 换服务不丢的调度判断（RESTART_SURVIVING_STATES）：内存表照旧是判的那一份，每次改了整份记进库（Store.write_restart_surviving_state），服务起来时读回 ──
    def restart_surviving_record(self, name):
        """这一份内存表写进库的正文：键是元组与整数的摊成按键排好的列表，没变的两次取出来逐字相同。"""
        if name == "runaway_notified":
            return [[host, pid, dict(entry)] for (host, pid), entry in sorted(list(self.runaway_notified.items()))]
        if name == "shard_advance":
            with self.attention_lock:
                return {"failures": [[parent_id, dict(failure)] for parent_id, failure in sorted(self.advance_failures.items())],
                        "episodes": [[parent_id, episode] for parent_id, episode in sorted(self.advance_episodes.items())]}
        if name == "suspect_reported":
            with self.suspect_lock:
                return [[attempt_id, segment] for attempt_id, segment in sorted(self.suspect_reported.items())]
        if name == "health_notified":
            return [[kind, sorted(sessions)] for kind, sessions in sorted(list(self.health_notified.items()))]
        if name == "booking_drift_notified":
            with self.booking_drift_lock:
                return [[key[0], key[1], session, crossed] for (key, session), crossed in sorted(self.booking_drift_notified.items(), key=lambda entry: (entry[0][0][0], str(entry[0][0][1]), entry[0][1]))]
        if name == "gpu_alerted":
            return sorted(self.gpu_cards_failed_alerted)
        raise AssertionError(f"RESTART_SURVIVING_STATES 里的 {name} 在这里没有一支：新加一份要在这里与 adopt_restart_surviving_record 各写一支")

    def adopt_restart_surviving_record(self, name, state):
        """库里读回的这一份并进内存表（restore_restart_surviving_state 调）：写法不对抛 ValueError，内存表一项都不动。"""
        def number(value):
            return isinstance(value, (int, float)) and not isinstance(value, bool)

        def whole(value):
            return isinstance(value, int) and not isinstance(value, bool)

        if name == "runaway_notified":
            if not isinstance(state, list):
                raise ValueError("空转进程的账应当是列表")
            restored = {}
            for row in state:
                if not (isinstance(row, list) and len(row) == 3 and row[0] in MACHINE_ORDER and whole(row[1]) and isinstance(row[2], dict)
                        and number(row[2].get("reported_at")) and whole(row[2].get("count")) and isinstance(row[2].get("alerted"), bool)):
                    raise ValueError(f"空转进程的账应当每行是 [台, 进程号, {{reported_at, count, alerted…}}]，有一行是 {str(row)[:120]}")
                restored[(row[0], row[1])] = row[2]
            self.runaway_notified.update(restored)
            return
        if name == "shard_advance":
            failures, episodes = (state.get("failures"), state.get("episodes")) if isinstance(state, dict) else (None, None)
            if not (isinstance(failures, list) and isinstance(episodes, list)):
                raise ValueError("分片父任务推不动的账应当是 {failures, episodes}")
            restored_failures, restored_episodes = {}, {}
            for row in failures:
                if not (isinstance(row, list) and len(row) == 2 and whole(row[0]) and isinstance(row[1], dict)
                        and whole(row[1].get("episode")) and whole(row[1].get("count")) and isinstance(row[1].get("why"), str)):
                    raise ValueError(f"推不动的这一回应当每行是 [父任务号, {{episode, why, count}}]，有一行是 {str(row)[:120]}")
                restored_failures[row[0]] = row[1]
            for row in episodes:
                if not (isinstance(row, list) and len(row) == 2 and whole(row[0]) and whole(row[1])):
                    raise ValueError(f"推不动开始过几回应当每行是 [父任务号, 回数]，有一行是 {str(row)[:120]}")
                restored_episodes[row[0]] = row[1]
            with self.attention_lock:
                self.advance_failures.update(restored_failures)
                self.advance_episodes.update(restored_episodes)
            return
        if name == "suspect_reported":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 2 and whole(row[0]) and (row[1] is None or whole(row[1])) for row in state)):
                raise ValueError("疑似卡住报过的段应当每行是 [尝试号, 段号]")
            with self.suspect_lock:
                self.suspect_reported.update({row[0]: row[1] for row in state})
            return
        if name == "health_notified":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 2 and isinstance(row[0], str) and isinstance(row[1], list)
                                                    and all(isinstance(session, str) for session in row[1]) for row in state)):
                raise ValueError("服务健康发过的账应当每行是 [种类, [会话…]]")
            self.health_notified.update({row[0]: set(row[1]) for row in state})
            return
        if name == "booking_drift_notified":
            if not (isinstance(state, list) and all(isinstance(row, list) and len(row) == 4 and row[0] == "memory" and isinstance(row[1], str)
                                                    and isinstance(row[2], str) and whole(row[3]) for row in state)):
                raise ValueError("记账漂移推过的账应当每行是 [memory, 台, 会话, 提醒门槛]")
            with self.booking_drift_lock:
                self.booking_drift_notified.update({((row[0], row[1]), row[2]): row[3] for row in state})
            return
        if name == "gpu_alerted":
            if not (isinstance(state, list) and all(isinstance(kind, str) for kind in state)):
                raise ValueError("GPU 服务写过告警的种类应当是字符串列表")
            self.gpu_cards_failed_alerted.update(state)
            return
        raise AssertionError(f"RESTART_SURVIVING_STATES 里的 {name} 在这里没有一支：新加一份要在这里与 restart_surviving_record 各写一支")

    def persist_restart_surviving_state(self, name):
        """这一份内存表记进库（改了它的地方改完就调）：与上次写进库的逐字相同就不写；库写不进的只记在内存里、打一行，下一次调再试。
        弄坏开关（RESTART_SURVIVING_STATES 里这一份的只记内存）下不写。调用方不拿着 attention_lock（shard_advance 那一份取内存表时要拿它）。"""
        if lib.broken(RESTART_SURVIVING_STATES[name], self.environment):
            return
        with self.restart_surviving_lock:
            state = self.restart_surviving_record(name)
            text = lib.canonical_json(state)
            if self.restart_surviving_written.get(name) == text:
                return
            try:
                self.store.write_restart_surviving_state(name, state)
            except sqlite3.Error as error:
                say(f"  ! 换服务不丢的那一份 {name} 没记进库（{type(error).__name__}: {error}），只记在内存里：下一次改动再写，换服务之前没写上的会丢")
                return
            self.restart_surviving_written[name] = text

    def restore_restart_surviving_state(self):
        """服务起来时（Scheduler.__init__ 末尾）把库里那几份（RESTART_SURVIVING_STATES）读回内存表：
        空转进程报过几次、停没停（watch_runaway_processes）与分片父任务推不动的这一回、开始过几回（note_advance_failure）接着数。
        签名对不上、读不出或写法不对的那一份不认、写告警 record-tampered，那一份照空的起。弄坏开关（这一份的只记内存）下不读。"""
        for name, switch in RESTART_SURVIVING_STATES.items():
            if lib.broken(switch, self.environment):
                continue
            state, valid = self.store.restart_surviving_state(name)
            problem = "" if valid else "签名对不上或读不出"
            if valid and state is not None:
                try:
                    self.adopt_restart_surviving_record(name, state)
                except ValueError as error:
                    problem = str(error)
            if problem:
                self.alert("record-tampered", 0, f"库里换服务不丢的那一份 {name} 不认（{problem}），这一份照空的起",
                           "查是谁直接写了库（只有 task-server 读写它）；这一份从零起：空转进程与推不动的回数从头数")

    def tampered(self, task_id, record, what):
        """校验不过（文件头「库」）：不起、不认，任务记 record-tampered 收尾，写告警。重签写回的那一行自己核得过，之后读到它不再判出校验不过、不再告警：
        关联列照正文写（Store.write_row），正文的 task_id 照 parent_id 写（association_id：parent_id 不是正整数的写 0，判不出它归哪一件就不挂在任何一件名下）。
        弄坏开关 tampered-rows-left-unrepaired 下正文的 task_id 不动、关联列不写，关联列或 parent_id 对不上的那一行重签之后照样核不过，每读一次告警一次。
        几个线程同时读到同一行核不过（排队那一轮与 status 这类）：拿 tamper_lock 现读，已由别的线程记成 record-tampered、核得过的不再告警、不再写。"""
        unrepaired = lib.broken("tampered-rows-left-unrepaired", self.environment)
        with self.tamper_lock:
            current = self.store.task(task_id)
            if not unrepaired and current is not None and current[1] and current[0].get("outcome") == "record-tampered":
                return
            if not lib.broken("tamper-unalerted", self.environment):
                self.alert("record-tampered", task_id, f"库里任务 {task_id} 的{what}校验不过（绕过服务直接写进库，或写进之后被改过），不起、不认",
                           "查是谁直接写了库（只有 task-server 读写它）；任务照 task-server/task submit 重新提交")
            record = dict(record, id=task_id, state="finished", outcome="record-tampered", exit_code=lib.final_exit_code("record-tampered", None),
                          finished_at=round(time.time(), 3), note=f"{what}校验不过，一次都不起")
            if not unrepaired:
                record["task_id"] = association_id(record.get("parent_id"))
            for name in ("session", "command_key"):
                record[name] = record.get(name) if isinstance(record.get(name), str) else ""
            self.store.update("tasks", record)
        self.forget_prediction(task_id)
        if isinstance(record.get("parent_id"), int) and record["parent_id"] != task_id:
            self.advance_parent(record)

    def selftest_read(self, line):
        """只在自证里：排队读预测与余量的每一次（与 stop_running 停了哪几次尝试）往 TASK_SERVER_SELFTEST_WORLD/scheduler-reads 记一行（task_server_lib.py 文件头「只在自证里认的变量」）。"""
        if not self.reads_log:
            return
        with self.reads_lock, open(self.reads_log, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")

    # ── 暂停（文件头 ⑤「暂停」）──
    def pause_in_force(self):
        """排队线程此刻要照着不起新尝试的暂停态（None 是不用）；弄坏开关 pause-queue-ignored 下排队线程当没暂停。"""
        return None if lib.broken("pause-queue-ignored", self.environment) else self.pause

    def allowed_during_pause(self, task, pause):
        """暂停着，这条任务是不是放行的会话交的（文件头 ⑤「暂停」的放行一段）：放行的会话是空串谁都不是，会话是空串的（人手任务）也不匹配。
        弄坏开关 pause-allow-ignored 下谁都不放行，pause-allow-matches-any-session 下放行了会话就谁都放行。"""
        allowed = pause.get("allow_session", "")
        if lib.broken("pause-allow-ignored", self.environment):
            return False
        if allowed and lib.broken("pause-allow-matches-any-session", self.environment):
            return True
        return bool(allowed) and task.get("session", "") == allowed

    def pause_holds(self, task):
        """这条任务此刻要不要留在队里不起：排队线程要照着暂停、而它又不是放行的会话交的 → 那份暂停态，不然 None。"""
        pause = self.pause_in_force()
        if pause is None or self.allowed_during_pause(task, pause):
            return None
        return pause

    def running_outside_allowed(self, pause, running):
        """暂停着在跑的任务里不是放行会话的有几个（等它为 0 才做空档的活）；没放行谁，在跑的都是别的会话的。弄坏开关 pause-other-running-miscounted 下把放行会话的也数进去。"""
        allowed = pause.get("allow_session", "")
        if not allowed or lib.broken("pause-other-running-miscounted", self.environment):
            return running
        return self.store.count_outside_session("tasks", "running", allowed)

    def pause_text(self, pause, running, other_running):
        """暂停态的一句话（/health 的 pause.text，control pause / check 照打）：暂停中（原因：…，起于…）、在跑 N 个；放行了会话的另写放行会话与别的会话在跑几个。
        弄坏开关 pause-allow-unreported 下不写放行的会话。"""
        started = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(pause["since"])) if pause["since"] else "不知道什么时候"
        text = f"暂停中（原因：{pause['reason'] or '没写'}，起于 {started}）"
        allowed = pause.get("allow_session", "")
        if allowed and not lib.broken("pause-allow-unreported", self.environment):
            return f"{text}、放行会话 {allowed}、在跑 {running} 个、别的会话在跑 {other_running} 个"
        return f"{text}、在跑 {running} 个"

    def pause_reply(self, changed, pause, allow_before="", allow_changed=False):
        """接口 POST /pause、/resume 的回话：pause 是这一回涉及的那一份暂停态（没有交 None）；running 是此刻 state 为 running 的任务数，
        other_running 是其中不是放行会话的；allow_session 是这一份里放行的会话，allow_before 是换之前的，allow_changed 是这一回换了没有。"""
        running = self.store.count("tasks", "running")
        current = self.pause
        return {"paused": current is not None, "changed": changed, "reason": pause["reason"] if pause else "", "since": pause["since"] if pause else None,
                "running": running, "other_running": self.running_outside_allowed(current, running) if current is not None else running,
                "allow_session": pause.get("allow_session", "") if pause else "", "allow_before": allow_before, "allow_changed": allow_changed}

    def pause_service(self, reason, allow_session, session, process_id):
        """暂停（接口 POST /pause）：拿 start_gate 改库里的暂停态、再改排队线程读的那一份，之后一条新尝试都不起（放行的会话交的除外）；
        已经暂停着的不改原因与起点，changed 为 false，给了不同的 allow_session 就换放行的会话（allow_changed 为 true），不给的不动放行。
        暂停态校验不过的（valid 为 False）不换放行。弄坏开关 pause-start-ungated 下不拿锁，pause-state-memory-only 下不写库，
        pause-allow-change-ignored 下已暂停着不换放行的会话，resume-keeps-allowed-session 下没给 allow_session 的沿用上一次 resume 时的放行。"""
        gate = contextlib.nullcontext() if lib.broken("pause-start-ungated", self.environment) else self.start_gate
        allow_before, allow_changed = "", False
        with gate:
            if self.pause is not None:
                allow_before = self.pause["allow_session"]
                if (not allow_session or allow_session == allow_before or not self.pause["valid"]
                        or lib.broken("pause-allow-change-ignored", self.environment)):
                    return self.pause_reply(False, self.pause, allow_before)
                self.store.write_pause(self.pause["reason"], self.pause["since"], allow_session)
                self.pause = dict(self.pause, allow_session=allow_session)
                allow_changed = True
            else:
                if not allow_session and lib.broken("resume-keeps-allowed-session", self.environment):
                    allow_session = self.kept_allow_session
                since = round(time.time(), 3)
                if not lib.broken("pause-state-memory-only", self.environment):
                    self.store.write_pause(reason, since, allow_session)
                self.pause = {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}
        if allow_changed:
            self.operation("pause", 0, session, process_id, f"换放行会话 {allow_before or '没有'} → {allow_session}")
        else:
            self.operation("pause", 0, session, process_id, reason + (f"（放行会话 {allow_session}）" if allow_session else ""))
        self.wake()
        return self.pause_reply(not allow_changed, self.pause, allow_before, allow_changed)

    def resume_service(self, session, process_id):
        """放开暂停（接口 POST /resume）：清库里的暂停态（放行的会话在里面，一并没了）、再清排队线程读的那一份，叫醒排队线程；本来没暂停的 changed 为 false。
        弄坏开关 resume-leaves-cache-paused 下只清库，resume-keeps-allowed-session 下记下放行的会话、下一次不带 --allow-session 的 pause 照它放行。"""
        with self.start_gate:
            was = self.pause
            if was is None:
                return self.pause_reply(False, None)
            self.store.clear_pause()
            if not lib.broken("resume-leaves-cache-paused", self.environment):
                self.pause = None
            if lib.broken("resume-keeps-allowed-session", self.environment):
                self.kept_allow_session = was["allow_session"]
        self.operation("resume", 0, session, process_id, was["reason"] + (f"（放行会话 {was['allow_session']}）" if was["allow_session"] else ""))
        self.wake()
        return self.pause_reply(True, was)

    def waiting_text(self, task):
        """这条任务在等什么：排着而服务暂停着，是暂停的原因（放行会话交的除外，它写自己真在等什么）；不然是上一轮判的排队原因（status、列表与 wait 的排队过久提醒读它）。
        弄坏开关 pause-waiting-unreported 下不写暂停，pause-allow-waiting-paused 下放行会话排着的也写暂停。"""
        pause = self.pause
        if (pause is not None and task.get("state") == "queued" and not lib.broken("pause-waiting-unreported", self.environment)
                and (lib.broken("pause-allow-waiting-paused", self.environment) or not self.allowed_during_pause(task, pause))):
            return f"服务暂停中（原因：{pause['reason'] or '没写'}）"
        reason = self.waiting_reasons.get(task["id"], "")
        return reason

    def start_attempt_unless_paused(self, task, placement):
        """起这一次尝试，除非此刻暂停着而它不是放行的会话交的 → (尝试, 因暂停没起)。拿 start_gate 判过没暂停才起，pause_service 同一把锁：pause 返回之后一条新的都不起，
        已经开始的在它返回之前都记下了。弄坏开关 pause-start-ungated 下判了不拿锁。"""
        if lib.broken("pause-start-ungated", self.environment):
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False
        with self.start_gate:
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False

    # ── 主循环 ──
    def wake(self):
        self.wake_event.set()

    def notify_finished(self, task_id):
        """记下结局之后叫醒在等这条任务的 wait（文件头「通知」）与发消息那一路（文件头 ⑥：任务号进 inbox_pending）；弄坏开关 wait-notify-skipped 时不叫 wait，wait 只能等兜底那一轮。"""
        with self.inbox_lock:
            self.inbox_pending.add(task_id)
        self.inbox_event.set()
        if lib.broken("wait-notify-skipped", self.environment):
            return
        for listener in list(self.finished_listeners):
            with contextlib.suppress(Exception):
                listener(task_id)

    def loop(self):
        while not self.stopping.is_set():
            self.last_tick = time.time()
            try:
                if not (self.pause is not None and lib.broken("pause-halts-reconcile", self.environment)):   # 暂停着对账照做（在跑的照跑到结局）；弄坏开关下对账也停
                    self.reconcile()
                self.expire_unconfirmed()   # 交了满时限没确认的取消（文件头 ⑦）
                self.place_queued()
                self.sweep_scenes()   # 取走结局满一段时间的现场删掉（task_server_lib.py 文件头「现场」）
                self.start_build_pool_sweep(periodic=True)   # 树不在了的编译目录组（task_server_lib.py 文件头「编译目录」）：到点在后台扫一回
                if not lib.broken("health-judges-head", self.environment):
                    self.starved_text = self.starved_problem(time.time())   # /health 只读这一份（文件头「接口」）
                self.progress_text = self.progress_problem(time.time())   # 看进展：/health 同样只读这一份
            except Exception:   # 一轮出事不停服务：打出来、下一轮再判
                say("  ! 这一轮排队出了异常（下一轮再判）：\n" + traceback.format_exc())
            self.wake_event.wait(PEER_REPASS_SECONDS if self.repass_soon else self.tick_seconds)
            self.wake_event.clear()

    def stop(self):
        self.stopping.set()
        self.wake()
        self.inbox_event.set()
        if lib.broken("probe-children-left", self.environment):   # 弄坏开关：照改前只等排队线程、不管自己起的子进程
            self.thread.join(timeout=30)
        else:   # 文件头 ③「停服务」：各后台线程合计至多等 STOP_JOIN_SECONDS，再停还在的子进程
            deadline = time.monotonic() + STOP_JOIN_SECONDS
            # 第二台的余量探测线程同样等（它起的 ssh 在停子进程之前收完；弄坏开关 peer-probe-thread-unjoined 下不等，停完子进程之后它还能再起一个 ssh 留在服务单元里）
            probe = () if lib.broken("peer-probe-thread-unjoined", self.environment) else (self.peer_probe_thread,)
            for thread in (self.thread, self.nfs_thread, self.inbox_thread, self.runaway_thread, self.booking_audit_thread) + probe:
                if thread.is_alive():
                    thread.join(timeout=max(0.0, deadline - time.monotonic()))
        if lib.broken("no-detach", self.environment):
            for child in self.children:   # 弄坏开关 no-detach：起成服务的子进程的尝试随服务一起停（单元的 cgroup 被停时就是这样）
                with contextlib.suppress(ProcessLookupError):
                    child.send_signal(signal.SIGTERM)
        if not lib.broken("probe-children-left", self.environment):
            self.stop_own_children()

    def stop_own_children(self):
        """停服务时还在的、服务自己起的子进程（文件头 ③「停服务」）：照 /proc 的父进程号从自己往下找，逐个按进程号先 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL；
        弄坏开关 no-detach 下起成子进程的尝试（self.children 与它们的子孙）不算在内。"""
        kept = set()
        for child in self.children:
            kept |= {child.pid} | descendant_processes(child.pid)
        leftover = descendant_processes(os.getpid()) - kept
        if not leftover:
            return
        say(f"  · 停服务：自己起的子进程还有 {len(leftover)} 个（{'、'.join(str(pid) for pid in sorted(leftover))}），逐个停")
        for signal_number in (signal.SIGTERM, signal.SIGKILL):
            for process_id in sorted(leftover):
                with contextlib.suppress(ProcessLookupError, PermissionError):
                    os.kill(process_id, signal_number)
            deadline = time.monotonic() + STOP_CHILD_GRACE_SECONDS
            while time.monotonic() < deadline:
                leftover = {pid for pid in leftover if os.path.exists(f"/proc/{pid}") and not process_is_zombie(pid)}
                if not leftover:
                    return
                time.sleep(0.05)

    # ── NFS 共享与第二台用不用得了（task_server_lib.py 文件头「NFS 共享」；本文件头 ②「机器」）──
    def nfs_loop(self):
        """每 nfs_recheck_seconds 复核一次 NFS 共享的前三项（两向探针只在起来时做，结论照留）：第二台没挂上的根记进 mount_problem，peer_view 当第二台此刻用不了；
        随后读一次第二台的余量（队空时排队线程不读），第二台用不了照 note_peer 告警。"""
        while not self.stopping.wait(self.nfs_recheck_seconds):
            try:
                report = lib.nfs_share_report(self.configuration.path, self.environment, probe=False)
            except Exception:   # 复核出事不停服务：打出来、下一回再判
                say("  ! 这一回 NFS 复核出了异常（下一回再判）：\n" + traceback.format_exc())
                continue
            with self.nfs_lock:
                probes = {entry["root"]: entry.get("probe") for entry in self.nfs.get("roots", [])}
            for entry in report["roots"]:
                entry["probe"] = probes.get(entry["root"], entry.get("probe"))
            for slow in report.get("slow") or []:   # 第二台一次超时、复查期里回了（task_server_lib.py 文件头「NFS 共享」的挂载）：只记进服务日志与 /health，不推提醒、照派第二台
                say(f"  · 第二台慢（超时、复查通了）：{slow}")
            if report.get("slow"):
                self.nfs_slow_last = {"at": round(time.time(), 3), "slow": list(report["slow"])}
            unmounted = next((entry for entry in report["roots"] if entry.get("mount") != lib.NFS_PASSED), None)
            self.mount_problem = (f"第二台没挂上共享根 {unmounted['root']}（{unmounted['mount']}；{lib.NFS_README}）" if unmounted else "")
            with self.nfs_lock:
                self.nfs = dict(report, probe_checked_at=self.nfs.get("probe_checked_at", self.nfs.get("checked_at")), slow_last=self.nfs_slow_last)
            try:   # 队空时排队线程不读余量：第二台用不了也在这里认出来、告警（note_peer）
                self.peer_view(self.resources.snapshot())
            except Exception:
                say("  ! 这一回读第二台的余量出了异常（下一回再判）：\n" + traceback.format_exc())

    def nfs_snapshot(self):
        with self.nfs_lock:
            return json.loads(json.dumps(self.nfs))

    def peer_view(self, snapshot):
        """这一轮读的余量照 NFS 复核改：第二台没挂上共享根的当它此刻用不了；顺带记第二台用不用得了（note_peer）。"""
        if self.mount_problem and snapshot.reachable.get(PEER):   # 共享根由本机导出、第二台挂载（文件头「NFS 共享」）：挂载复核只判第二台
            snapshot = dataclasses.replace(snapshot, reachable=dict(snapshot.reachable, **{PEER: False}), down=dict(snapshot.down, **{PEER: self.mount_problem}),
                                           down_kind={host: kind for host, kind in snapshot.down_kind.items() if host != PEER},
                                           reports={host: report for host, report in snapshot.reports.items() if host != PEER})
        if snapshot.expected.get(PEER):
            available = bool(snapshot.reachable.get(PEER)) and PEER in snapshot.reports
            self.note_peer(available, "" if available else (snapshot.down.get(PEER) or "第二台的余量查不成"), kind="" if available else snapshot.down_kind.get(PEER, ""))
        return self.unstarted_slices_filled(snapshot)

    def unstarted_slices_filled(self, snapshot):
        """slice 还没起的台（重启之后还没起过内存包装；报告里 memory_peaks_lib.SLICE_STARTED_KEY 为假，slice_total 是按整机内存估的）：总上限换成多机配置
        SLICE_MEMORY_CAP_BY_HOST 里那台那一项（包装第一次起就把 slice 设成它），slice 放得下照它减 slice 里已用的；两台同一段。配置读不到的照报告原样。
        弄坏开关 unstarted-slice-estimated 下照原样用估的。"""
        if lib.broken("unstarted-slice-estimated", self.environment):
            return snapshot
        unstarted = [host for host, report in snapshot.reports.items() if (report.get("memory") or {}).get(self.peaks.SLICE_STARTED_KEY) is False]
        if not unstarted:
            return snapshot
        caps, _ = self.configured_slice_caps()
        reports = dict(snapshot.reports)
        for host in unstarted:
            total = int((caps or {}).get(host) or 0)
            if total > 0:
                memory = dict(reports[host]["memory"], slice_total=total)
                memory["slice_room"] = total - int(memory.get("slice_used") or 0)
                reports[host] = dict(reports[host], memory=memory)
        return dataclasses.replace(snapshot, reports=reports)

    def note_peer(self, available, why, task_id=None, kind=""):
        """第二台用不用得了（本文件头 ②「机器」的告警）：从用得了变成用不了、或用不了的原因变了，写一条告警 peer-unavailable；/health 的 peer 一栏读 peer_status（why 照最新的写）。
        kind 给了的按它去重（盘余量不到下限：余量的数每轮在变，不每轮另写一条），没给的按原因的字面去重。弄坏开关 peer-unavailable-silent 下不记、不告警。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        with self.peer_status_lock:
            status = self.peer_status
            if available:
                if not status["available"]:
                    say(f"  · 第二台又用得了（之前：{status['why']}）")
                status.update(available=True, why="", since=None)
                self.peer_alerted = ""
                return
            if status["available"]:
                status.update(available=False, since=round(time.time(), 3))
            status["why"] = why
            if self.peer_alerted == (kind or why):
                return
            self.peer_alerted = kind or why
        self.alert("peer-unavailable", task_id, f"第二台此刻用不了：{why}；派得到第二台的任务这时只派本机",
                   f"照那一句修第二台（连法、多机配置的双机开关、NFS 挂载照 {lib.NFS_README}），修好之后服务下一轮自己认回；用不了满 {self.peer_unavailable_notify_seconds:.0f} 秒服务发消息给有任务在排、在跑的会话")

    def note_peer_run_failure(self, task, attempt):
        """那一趟没跑成（peer-run-failed：树建不成、ssh 被重置、那边的准备没过）：每一次写一条告警 peer-unavailable，/health 的 peer 一栏记最近一次。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        why = f"任务 {task['id']} 第 {attempt.get('number')} 次在第二台那一趟没跑成（multi-host-run.sh 退 255）：{lib.log_last_line(log_tail(attempt.get('log', '')))}"
        with self.peer_status_lock:
            self.peer_status["last_run_failure"] = {"time": round(time.time(), 3), "task": task["id"], "why": why}
        self.alert("peer-unavailable", task["id"], why, f"看 task-server/task log {task['id']}；连不上、树建不成照那一句修第二台，服务照重试的规矩重排")

    def peer_health(self):
        with self.peer_status_lock:
            return dict(self.peer_status)

    def check_prune_server(self, task, attempt):
        """命令退 75（prune-server 不可用，task_server_lib.py 文件头「错误类型」）：另起一个线程在仓根跑一次 python3 prune-server/control check
        （至多 PRUNE_SERVER_CHECK_SECONDS 秒），不通（退非 0、起不来、超时、那份文件不在）写一条告警 prune-server-unavailable；只报，不起、不停 prune-server。
        自证里 check 换成 TASK_SERVER_PRUNE_SERVER_CHECK 那一份可执行文件。"""
        fake = lib.selftest_override("TASK_SERVER_PRUNE_SERVER_CHECK", self.environment)
        command = [fake] if fake else [sys.executable, "-B", lib.PRUNE_SERVER_CONTROL, "check"]
        shown = "python3 prune-server/control check"

        def run():
            if not fake and not os.path.isfile(lib.PRUNE_SERVER_CONTROL):
                code, text = None, f"仓里没有 {lib.PRUNE_SERVER_CONTROL}"
            else:
                try:
                    completed = subprocess.run(command, cwd=lib.REPOSITORY_ROOT, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                               timeout=PRUNE_SERVER_CHECK_SECONDS)
                    code, text = completed.returncode, completed.stderr + completed.stdout
                except (OSError, subprocess.TimeoutExpired) as error:
                    code, text = None, f"{type(error).__name__}: {error}"
            if code == 0:
                say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次命令退 75，{shown} 通（只报，不另做什么）")
                return
            self.alert(lib.PRUNE_SERVER_ALERT_KIND, task["id"],
                       f"任务 {task['id']} 第 {attempt.get('number')} 次命令退 75（prune-server 不可用）；{shown} {'退 ' + str(code) if code is not None else '没跑成'}："
                       f"{lib.last_line(text, '没有输出')[:300]}",
                       "照 check 那一句修 prune-server（task-server 不起、不停它：python3 prune-server/control start / install 由主 agent 起）；修好之后重交这条任务")
        threading.Thread(target=run, name=f"prune-server-check-{task['id']}", daemon=True).start()

    # ── ① 对账 ──
    def attempt_paths(self, attempt):
        directory = attempt["directory"]
        return os.path.join(directory, "result.json"), os.path.join(directory, "started"), os.path.join(directory, "scope")

    def settle_ended_attempts(self):
        """任务还 running、它最新那一次尝试却已经 ended 的（上一次定去向半途抛错，或服务在记尝试与定去向之间没了）：补定去向（settle_attempt）。
        任务 2461 就是这样留了两小时的孤儿：本机因此一直算有任务在跑。弄坏开关 ended-attempts-unsettled 下不补（改前的样子）。"""
        if lib.broken("ended-attempts-unsettled", self.environment):
            return
        for task, valid in self.store.select("tasks", "state = 'running'"):
            if not valid:
                continue
            attempts = [record for record, record_valid in self.store.attempts_of(task["id"]) if record_valid]
            if not attempts or attempts[-1].get("state") != "ended" or int(attempts[-1].get("number") or 0) != int(task.get("attempt_count") or 0):
                continue
            attempt = attempts[-1]
            if self.finishing_now(attempt.get("id")):   # 收尾线程记了 ended、正在定去向（finish_off_loop）：不是半途断了的，不补
                continue
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试结局 {attempt.get('outcome')} 已记、任务没收尾：现在补收尾")
            self.settle_attempt(task, attempt, None)

    def reconcile(self):
        self.settle_ended_attempts()
        self.forget_settled_entries(time.time())
        unrepaired = lib.broken("tampered-rows-left-unrepaired", self.environment)
        for attempt, valid in self.store.select("attempts", "state IN ('starting', 'running')"):
            if self.finishing_now(attempt.get("id")):   # 收尾在后台线程里做着（finish_off_loop）：这一轮不看它
                continue
            owner = association_id(attempt.get("task_id"))
            found = self.store.task(owner) if owner else None
            if not unrepaired and (not valid or (found is not None and not found[1])):   # 这一行或它那条任务核不过：收尾这一次（end_tampered_attempt）
                self.end_tampered_attempt(attempt, found, "尝试记录" if not valid else "任务记录")
                continue
            if found is None:
                continue
            task, task_valid = found
            if task.get("state") == "finished":
                continue
            if not valid or not task_valid:   # 弄坏开关 tampered-rows-left-unrepaired：只给任务记 record-tampered，尝试行留在在跑里
                self.tampered(task["id"], task, "尝试记录" if not valid else "任务记录")
                continue
            result_path, started_path, _ = self.attempt_paths(attempt)
            if attempt["state"] == "starting" and lib.broken("start-twice-after-restart", self.environment) and not attempt.get("restarted"):
                attempt["restarted"] = True   # 弄坏开关：不看结果、单元与起跑标记，再起一份
                self.launch(task, attempt)
                continue
            # 弄坏开关 unit-unreadable-exit-file-unread：读不到单元状态的那一段不认 result.json（核到没了也照没留结果收）
            exit_file_unread = lib.broken("unit-unreadable-exit-file-unread", self.environment) and attempt["id"] in self.unit_unreadable
            result = None if exit_file_unread else read_json_file(result_path)
            if result is not None:
                self.finish_off_loop(task, attempt, result)
                continue
            now = time.monotonic()
            if now - self.unit_checked_at.get(attempt["id"], 0.0) < self.unit_check_seconds:
                continue
            self.unit_checked_at[attempt["id"]] = now
            if attempt.get("unit"):
                states = unit_states(attempt["unit"], self.environment)
                if states is None:   # 判不出单元在不在（文件头 ①「读不到单元状态」）：不杀、不收尾、不记结局，不经 systemd 核到都没了才照没留结果收尾
                    # 弄坏开关 unit-unreadable-lost-unverified：读不到就当场照没留结果收尾（不核进程，收尾照常经 systemctl 回读、停留下的进程）
                    if lib.broken("unit-unreadable-lost-unverified", self.environment) or self.unit_unreadable_gone(attempt, now):
                        self.finish_off_loop(task, attempt, None if exit_file_unread else read_json_file(result_path))
                    continue
                self.forget_unit_unreadable(attempt["id"])
                load_state, active_state = states
            else:   # 弄坏开关 no-detach：起成子进程的那几次，看进程号在不在
                alive = any(child.pid == attempt.get("child_pid") and child.poll() is None for child in self.children)
                load_state, active_state = ("loaded", "active") if alive else ("not-found", "inactive")
            if active_state in lib.RUNNING_UNIT_STATES:
                self.unit_gone_since.pop(attempt["id"], None)
                self.unit_stopped.discard(attempt["id"])
                self.check_stall(task, attempt)
                if attempt["state"] == "starting":
                    attempt["state"] = "running"
                    self.store.update("attempts", attempt)
                continue
            if attempt["state"] == "starting" and load_state == "not-found" and not os.path.exists(started_path):
                self.resume_starting(task, attempt, now)
                continue
            gone_since = self.unit_gone_since.setdefault(attempt["id"], now)
            if now - gone_since < self.lost_grace_seconds or lib.broken("lost-not-recorded", self.environment):
                continue
            self.finish_off_loop(task, attempt, read_json_file(result_path))
        self.drive_sharded_parents()

    def unit_unreadable_gone(self, attempt, now):
        """对账读不到这一次的单元状态（unit_states 交 None）、又没留 result.json 时调（文件头 ①「读不到单元状态」）：记下从哪一刻（now，monotonic）起连着读不到，
        照不经 systemd 最近一回核到的（unit_presence_reading，后台线程核）定：核到这一次在两台上的 cgroup 都读得到且空、记下的进程号都不在，连着满 lost_grace_seconds
        交 True（由对账照没留结果收尾、记 launcher-lost；attempt 记上 unit_unreadable_gone，stop_leftovers 照它认）；还在的、核不出的、还没核完的交 False
        （不杀、不收尾、不记结局，尝试照记 running，不设到点就收尾的时限）。头一回读不到、之后每跨过一道提醒门槛（同 unit-unreadable 提醒），与照没留结果收尾的那一刻，各往服务日志打一行 TASK_UNIT_UNREADABLE。"""
        entry = self.unit_unreadable.setdefault(attempt["id"], {"since": now, "task": attempt.get("task_id"), "number": attempt.get("number"),
                                                                "host": attempt.get("host"), "unit": attempt.get("unit")})
        reading = self.unit_presence_reading(attempt, now)
        verdict, detail = (reading[0], reading[1]) if reading is not None else (lib.PRESENCE_UNPROBED, lib.PRESENCE_UNPROBED_TEXT)
        crossed = reminder_threshold(now - entry["since"], self.unit_unreadable_attention_seconds)
        if crossed > self.unit_unreadable_noted.get(attempt["id"], -1):
            self.unit_unreadable_noted[attempt["id"]] = crossed
            self.note_unit_unreadable(attempt, now - entry["since"], verdict, detail)
        if verdict != lib.PRESENCE_GONE:
            self.unit_gone_since.pop(attempt["id"], None)
            return False
        if now - self.unit_gone_since.setdefault(attempt["id"], now) < self.lost_grace_seconds:
            return False
        attempt["unit_unreadable_gone"] = detail
        self.note_unit_unreadable(attempt, now - entry["since"], verdict, f"核到都不在，照没留结果收尾：{detail}")
        return True

    def note_unit_unreadable(self, attempt, seconds, verdict, detail):
        """读不到单元状态的那一段往服务日志打的一行（文件头 ①「读不到单元状态」）。"""
        say(f"TASK_UNIT_UNREADABLE time={time.time():.6f} attempt={attempt['id']} task={attempt.get('task_id')} number={attempt.get('number')} "
            f"host={attempt.get('host')} unit={attempt.get('unit')} seconds={seconds:.0f} presence={verdict} detail={detail}")

    def forget_unit_unreadable(self, attempt_id):
        """清掉一次尝试读不到单元状态的那一段记账（对账读到了单元状态、收尾、核不过而记 ended 时；文件头 ①「读不到单元状态」）。"""
        self.unit_unreadable.pop(attempt_id, None)
        self.unit_unreadable_noted.pop(attempt_id, None)
        with self.unit_presence_lock:
            self.unit_presence.pop(attempt_id, None)

    def unit_presence_reading(self, attempt, now):
        """不经 systemd 最近一回核到的 (lib.PRESENCE_*, 说明, 核完的时刻)，还没核完交 None；上一回比 unit_check_seconds 旧、又没在核的，另起一个后台线程再核
        （probe_unit_presence；第二台那一趟带 ssh，排队线程不等它）。"""
        key = attempt["id"]
        with self.unit_presence_lock:
            reading = self.unit_presence.get(key)
            if key not in self.unit_presence_probing and (reading is None or now - reading[2] >= self.unit_check_seconds):
                self.unit_presence_probing.add(key)
                threading.Thread(target=self.probe_unit_presence, args=(dict(attempt),), name=f"unit-presence-{key}", daemon=True).start()
        return reading

    def unit_presence_now(self, attempt_id):
        """提醒与 /health 读的：(lib.PRESENCE_*, 说明)，还没核完的交 PRESENCE_UNPROBED。"""
        with self.unit_presence_lock:
            reading = self.unit_presence.get(attempt_id)
        return (reading[0], reading[1]) if reading is not None else (lib.PRESENCE_UNPROBED, lib.PRESENCE_UNPROBED_TEXT)

    def probe_unit_presence(self, attempt):
        """unit_presence_reading 起的后台线程：核一回（attempt_presence），写进 unit_presence；这一段已清掉（读到了、收尾了）的不写回。
        核法自己出事照核不出办：不收尾，下一回再核。"""
        try:
            verdict, detail = self.attempt_presence(attempt)
        except Exception as error:
            verdict, detail = lib.PRESENCE_UNVERIFIED, f"核进程出了异常（{type(error).__name__}: {error}）"
        with self.unit_presence_lock:
            self.unit_presence_probing.discard(attempt["id"])
            if attempt["id"] in self.unit_unreadable:
                self.unit_presence[attempt["id"]] = (verdict, detail, time.monotonic())

    def attempt_presence(self, attempt):
        """不经 systemd 核这一次尝试还有没有进程（文件头 ①「读不到单元状态」）→ (lib.PRESENCE_*, 说明)。本机读单元那一层（attempt_runner 与它起的包装都在本机的单元里，
        派到第二台的也是）：起跑标记记的 runner 进程号、单元的 cgroup、attempt_runner 写的 scope 文件点名的目录、带这一次尝试标签的内存包装 scope；
        派到第二台、清理回执 pending 的另读第二台那一趟（multi-host-run.sh 记的进程号文件与 scope 单元名文件）。两台跑同一段（Machine.attempt_presence_readings），
        判在 lib.attempt_presence_verdict。起跑标记不在或读不了、scope 文件读不了、清理回执读不到或认不出的，那一台照核不出算。
        弄坏开关 unit-unreadable-absent-cgroup-unconfirmed 下两台都把不在的 cgroup 目录算核不出（请求里带 absent_cgroup_unconfirmed）。"""
        _, started_path, scope_path = self.attempt_paths(attempt)
        label = attempt.get("label") or ""
        common = {"label_entry": f"{lib.ATTEMPT_LABEL_VARIABLE}={label}" if label else "", "label_scope_prefix": lib.MEMORY_CAP_SCOPE_PREFIX,
                  "tree_depth": lib.ATTEMPT_PRESENCE_TREE_DEPTH, "absent_cgroup_unconfirmed": lib.broken("unit-unreadable-absent-cgroup-unconfirmed", self.environment)}
        runner, problem = recorded_text(started_path, "起跑标记")
        if problem or not runner.isdigit():
            return lib.attempt_presence_verdict([(LOCAL, None, problem or f"起跑标记 {started_path} 不在或不是进程号（attempt_runner 还没起，或单元没起成），核不出")])
        scope, problem = recorded_text(scope_path, "scope 文件")
        if problem:
            return lib.attempt_presence_verdict([(LOCAL, None, problem)])
        # scope 文件只认真的 cgroup 目录（同 stop_leftovers：自证里 attempt_runner 读的是假 scope 目录，那一份不是 cgroup）
        recorded_scope = [scope] if scope.startswith("/sys/fs/cgroup/") and scope.endswith(".scope") else []
        local_request = dict(common, process_ids=[runner], process_id_files=[], cgroup_directories=recorded_scope, cgroup_names=[attempt["unit"]], cgroup_name_files=[])
        parts = [(LOCAL, *self.machines[LOCAL].attempt_presence_readings(local_request))]
        if attempt.get("host") != LOCAL:
            parts.append(self.peer_presence_part(attempt, common))
        return lib.attempt_presence_verdict(parts)

    def peer_presence_part(self, attempt, common):
        """派到第二台的那一次在第二台那一趟的读数（attempt_presence）：→ (台, 读数或 None, 读不成的那一句)。清理回执 not-started（那一趟没起过）、confirmed（已收干净）的
        第二台没有这一次的进程；pending 的照回执里的进程号文件（runs/<标签>.pid）与它旁边的 scope 单元名文件（runs/<标签>.scope，名字不带 .scope）在第二台上核。"""
        host = attempt["host"]
        path = attempt.get("peer_cleanup_file") or ""
        receipt = read_json_file(path) if path else None
        status = (receipt or {}).get("status")
        if receipt is None:
            return host, None, f"清理回执 {path or '（这一次没有）'} 读不到，第二台那一趟核不出"
        if status in ("not-started", "confirmed"):
            return host, {"processes": {}, "cgroups": {}, "files": {}, "problems": [], "note": f"清理回执 {status}，第二台没有这一次的进程"}, ""
        pid_file = receipt.get("pid_file")
        if status != "pending" or not isinstance(pid_file, str) or not pid_file.startswith("/") or not pid_file.endswith(".pid"):
            return host, None, f"清理回执 {path} 认不出（状态 {status}、进程号文件 {pid_file}），第二台那一趟核不出"
        request = dict(common, process_ids=[], process_id_files=[pid_file], cgroup_directories=[], cgroup_names=[],
                       cgroup_name_files=[pid_file[:-len(".pid")] + ".scope"], cgroup_name_suffix=".scope")
        return (host, *self.machines[host].attempt_presence_readings(request))

    def end_tampered_attempt(self, attempt, found, what):
        """对账判出在跑的这一次核不过（尝试行或它那条任务的行校验不过；task_server_lib.py 文件头「库」）：它那条任务还没结局的照 tampered 记 record-tampered、
        告警一条；任务已有结局、或库里没有它的，告警写这一行。这一行记 ended、结局 record-tampered，正文只留认得出是哪一次的几样（ENDED_TAMPERED_FIELDS），
        路径、峰值与记账都不留（正文不可信，服务不照它删、不照它记账），关联列与正文的 task_id 照 association_id 写、重签：之后它核得过、
        不在在跑的尝试里（Store.running_attempts），不再占内存、盘与线程。同一任务同一次号已有一行的（关联列被挪过来顶了号），task_id 写 0。
        记完之后后台线程照 stop_tampered_unit 停它的单元、回读、报 tampered-unit-stop（不算占用的单元留着跑，这台的内存会被多派）。
        弄坏开关 tampered-unit-left-running 下不停。"""
        owner = association_id(attempt.get("task_id"))
        if found is not None and found[0].get("state") != "finished":
            self.tampered(found[0]["id"], found[0], what)
        elif not lib.broken("tamper-unalerted", self.environment):
            self.alert("record-tampered", owner, f"库里在跑的尝试记录（行 {attempt['id']}）或它那条任务的记录校验不过：服务不认这一次，记 ended、不再算在跑",
                       f"查是谁直接写了库（只有 task-server 读写它）；它的单元 {attempt.get('unit') or '（没记）'} 服务接着停，结果另报 tampered-unit-stop")
        ended = {name: attempt[name] for name in ENDED_TAMPERED_FIELDS if isinstance(attempt.get(name), (str, int, float)) and not isinstance(attempt.get(name), bool)}
        ended.update(id=attempt["id"], task_id=owner, state="ended", outcome="record-tampered", ended_at=round(time.time(), 3),
                     note=f"{what}校验不过：服务不认这一次，记 ended、不再算在跑（单元由服务停，结果另报 tampered-unit-stop）")
        for name in ("session", "command_key"):
            ended[name] = ended.get(name) if isinstance(ended.get(name), str) else ""
        try:
            self.store.update("attempts", ended)
        except sqlite3.IntegrityError:   # (task_id, number) 唯一：那一号已有一行
            self.store.update("attempts", dict(ended, task_id=0))
        self.unit_checked_at.pop(attempt["id"], None)
        self.unit_gone_since.pop(attempt["id"], None)
        self.forget_unit_unreadable(attempt["id"])
        if not lib.broken("tampered-unit-left-running", self.environment):
            task = found[0] if found is not None and found[1] else None   # 任务行也核不过的不照它记的会话与令牌发消息，只写告警
            threading.Thread(target=self.stop_tampered_unit, args=(attempt.get("unit"), owner, task), daemon=True).start()

    def stop_tampered_unit(self, unit, owner, task):
        """停核不过的那一次的单元、回读，报 tampered-unit-stop（end_tampered_attempt 起的后台线程，排队线程不等 systemctl）。行的正文不可信，只停这两样都合的：
        名字是这个服务起的单元（singlefs-task-<库的标>-<任务号>-<次号>.service），又不是校验过的在跑尝试的单元；别的不动、照实报，由人核。
        第二台的那一趟这里停的是本机包装单元，第二台那边由包装收到 TERM 时清（multi-host-run.sh）。"""
        name = unit if isinstance(unit, str) else ""
        if not re.fullmatch(rf"singlefs-task-{re.escape(self.store.tag)}-[0-9]+-[0-9]+\.service", name):
            line = f"核不过的尝试记的单元 {name or '（没记）'} 不是这个服务起的单元名：没停"
        elif any(record.get("unit") == name for record in self.store.running_attempts()):
            line = f"核不过的尝试记的单元 {name} 是另一次校验过的在跑尝试的：没停"
        else:
            with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                subprocess.run(["systemctl", "--user", "stop", name], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=UNIT_STOP_SECONDS + 30)
            states, processes = unit_states(name, self.environment), unit_processes(name, self.environment)
            if states is None or states[1] in lib.RUNNING_UNIT_STATES or processes is None or processes:
                line = (f"核不过的尝试的单元 {name} 停了之后回读没停干净：状态 {states[1] if states else '读不到'}、"
                        f"cgroup 里进程 {'读不到' if processes is None else len(processes)} 个")
            else:
                line = f"核不过的尝试的单元 {name} 已停（回读不在跑、cgroup 里没有进程）"
        next_step = lib.anomaly_advice("tampered-unit-stop", owner, self.environment)
        self.alert("tampered-unit-stop", owner, line, next_step)
        if task is not None and (task.get("inbox") or {}).get("token"):   # 人手交的没有令牌、没有会话可发
            self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}任务 {owner} 核不过的尝试：{line}", f"出路：{next_step}"]), ("tampered-unit-stop", owner, name))

    def forget_settled_entries(self, now):
        """按任务号记在内存里的那几张表（RESIDENT_TASK_TABLES）与按尝试号记的疑似卡住（suspect_reported、inbox_failed 里的 suspect 事件）随跑过的任务涨：
        对账每一轮问一次、至多隔 resident_prune_seconds 清一回。删掉任务有了结局并被取走（collected）、结局满 RESIDENT_RETENTION_SECONDS 没人取、
        或库里已没有的那几条任务的项；按尝试号记的删掉不在在跑的尝试里（Store.running_attempts）的项。库里另存了一份的（报到的提醒门槛、结局再推到第几回）
        删了照库里那份接着算；删了之后同一件再判出同一样的，照新的一件再告警、再报一次。弄坏开关 resident-entries-kept 下一项都不删。"""
        if lib.broken("resident-entries-kept", self.environment) or now - self.resident_pruned_at < self.resident_prune_seconds:
            return
        self.resident_pruned_at = now

        def task_of_event(event):   # inbox_failed 的事件归哪条任务：("outcome", 号)、(号, 种类, 提醒门槛)；("suspect", 尝试号, 段号) 归尝试，交 None
            if not isinstance(event, tuple) or not event or event[0] == "suspect":
                return None
            return event[1] if event[0] == "outcome" and len(event) > 1 else event[0]

        def task_of_key(key, position):
            return key[position] if position is not None and isinstance(key, tuple) and len(key) > position else (key if position is None else None)

        named = {task_of_key(key, position) for name, position in RESIDENT_TASK_TABLES.items() for key in list(vars(self)[name])}
        named |= {task_of_event(event) for event in list(self.inbox_failed)}
        candidates = sorted(task_id for task_id in named if association_id(task_id))
        present = {}
        for start in range(0, len(candidates), 500):
            chunk = candidates[start:start + 500]
            present.update((record["id"], record) for record, _ in self.store.select("tasks", f"id IN ({', '.join('?' * len(chunk))})", tuple(chunk)))
        settled = set()
        for task_id in candidates:
            record = present.get(task_id)
            if record is None or (record.get("state") == "finished" and (record.get("collected")
                                                                           or now - float(record.get("finished_at") or now) >= RESIDENT_RETENTION_SECONDS)):
                settled.add(task_id)
        running = {attempt.get("id") for attempt in self.store.running_attempts()}
        for name, position in RESIDENT_TASK_TABLES.items():
            container = vars(self)[name]
            for key in list(container):
                if task_of_key(key, position) in settled:
                    if isinstance(container, dict):
                        container.pop(key, None)
                    else:
                        container.discard(key)
        self.persist_restart_surviving_state("shard_advance")   # advance_failures、advance_episodes 删掉的项，库里那一份跟着删（一项没删的与上次写的相同，不重写）
        with self.suspect_lock:
            for attempt_id in list(self.suspect_reported):
                if attempt_id not in running:
                    self.suspect_reported.pop(attempt_id, None)
        self.persist_restart_surviving_state("suspect_reported")
        for event in list(self.inbox_failed):
            suspect_gone = isinstance(event, tuple) and len(event) > 1 and event[0] == "suspect" and event[1] not in running
            if suspect_gone or task_of_event(event) in settled:
                self.inbox_failed.discard(event)

    def resume_starting(self, task, attempt, now):
        """starting、单元不在、起跑标记也没有的那一次（文件头 ①）：拿这条任务的锁、现读任务再定——取消了的照取消收尾（不起）；
        起单元的回执超时过（start_uncertain_at，TS-06）的过了 start_uncertain_grace_seconds 还是这样，确认没起、记 unit-start-failed；
        别的是服务在写了单元名与起单元之间没了：起它一次。"""
        with self.store.task_lock(task["id"]):
            found = self.store.task(task["id"])
            current = found[0] if found is not None and found[1] else task
            if current.get("cancel_requested") or current.get("state") == "finished":
                say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试还没起、任务已取消：不起，照取消收尾")
                self.finish_off_loop(current, attempt, None)
                return
            uncertain_at = attempt.get("start_uncertain_at")
            if uncertain_at:
                if time.time() - float(uncertain_at) < self.start_uncertain_grace_seconds:
                    return
                why = (f"systemd-run 回执超时之后照单元名 {attempt.get('unit')} 读回 {self.start_uncertain_grace_seconds:.0f} 秒，单元一直不在、起跑标记也没有："
                       "确认没起")
                attempt.update(state="ended", outcome="unit-start-failed", ended_at=round(time.time(), 3), note=why)
                self.store.update("attempts", attempt)
                self.unit_checked_at.pop(attempt["id"], None)
                self.finalize(current, "unit-start-failed", None, why)
                return
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的单元名写进库之后、起单元之前服务没了：现在起它（只这一次）")
            self.launch(current, attempt)

    def check_stall(self, task, attempt):
        """日志距上一次增长满 stall_seconds、这段时间 CPU 也几乎没涨就停掉这一条（文件头「卡住」）。两台同一段：日志里除起法那一层的进度行之外的字节数（log_content），
        CPU 照那一台的接口读（Machine.cpu_usec：本机读尝试目录的 cpu_usec，第二台读日志里最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec）。
        这一回没停它的，再判疑似卡住（check_suspect：只通知、不停）。
        人手交的跑满两小时另记一行进日志（long_run_sessionless_note）。
        头一步判四小时硬上限（judge_run_hard_limit，文件头 ⑦「四小时硬上限」）：不看宽限，重排之后刚起的那一次累计到了照停。"""
        if self.judge_run_hard_limit(task, attempt):
            return
        usage = self.fresh_cpu_usec(attempt)
        if usage is not None:   # CPU 闸的峰值段核数（note_peak_segment）：每一轮记一回累计读数，宽限里的也记，头一段从起跑那一会儿量起
            self.note_peak_segment(attempt, time.time(), usage)
        self.note_threads(attempt)   # CPU 闸的可跑线程数与线程总数远超登记数（文件头 ②「CPU 闸」、①「要处置的提醒」的 threads-over-registered）：每一轮读一回，宽限里的也读
        if attempt.get("stalled"):
            return
        self.long_run_sessionless_note(task, attempt)
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        if elapsed < self.stall_grace_seconds and not lib.broken("stall-grace-skipped", self.environment):
            self.log_growth.pop(attempt["id"], None)   # 宽限里不判、不量：宽限过了从那一刻起量窗口
            self.cpu_windows.pop(attempt["id"], None)
            self.io_waits.pop(attempt["id"], None)
            self.activity_windows.pop(attempt["id"], None)
            self.selftest_read(f"stall-grace task={task['id']} attempt={attempt['number']} elapsed={elapsed:.3f}")   # 自证：宽限里判过一次（stall-waits-for-grace 照它放行）
            return
        size = self.log_content(attempt)
        now = time.monotonic()
        cpu = self.machines[attempt["host"]].cpu_usec(attempt)
        if not self.judge_stall_stop(task, attempt, size, cpu, now):
            self.check_suspect(task, attempt, cpu, now)
            self.note_attempt_progress(attempt, now)

    def fresh_cpu_usec(self, attempt):
        """check_stall 每一轮开头读的这一次尝试累计 CPU 微秒：先把日志读到底（log_content：第二台的 CPU、线程数都在日志里，读到的是这一刻之前的最近一行），
        再照那一台的接口读（Machine.cpu_usec）；宽限里不读日志的轮、第二台的尝试同样是现读的。弄坏开关 stall-cpu-read-before-log 下不先读日志（第二台交上一回解析的旧读数）。"""
        if not lib.broken("stall-cpu-read-before-log", self.environment):
            self.log_content(attempt)
        return self.machines[attempt["host"]].cpu_usec(attempt)

    def note_attempt_progress(self, attempt, now):
        """日志不涨时读一回这一次尝试的进展（文件头 ①「要处置的提醒」的 quiet-log-busy；check_stall 每一轮调，now 是 time.monotonic()）：日志（log_content 算的那部分）
        满 stall_seconds 没涨、距上一回读满 stall_seconds 的一半才读（第二台每读一回起一次 ssh），照那一台的接口读（Machine.attempt_progress_readings：
        这一趟进程树里打开写着的文件与大小）。写着的文件比上一回读的大了（子进程的输出另存在运行目录里的 record.log 这类），
        记进 progress_noted（时刻与给人看的那半句：哪份文件从多少涨到多少字节）；quiet_log_items 与进度推送只读它，不另去读。
        上一回没读过的这一回只记大小（头一回读不出涨没涨）。弄坏开关 open-file-growth-ignored 下不看文件涨没涨。"""
        seen = self.log_growth.get(attempt["id"])
        if seen is None or now - seen[1] < self.stall_seconds:
            return
        last = self.progress_read_at.get(attempt["id"])
        if last is not None and now - last < self.stall_seconds / 2:
            return
        self.progress_read_at[attempt["id"]] = now
        readings, _ = self.machines[attempt["host"]].attempt_progress_readings(attempt)
        if readings is None:
            return
        files = {path: size for path, size in (readings.get("files") or {}).items() if isinstance(size, int) and not isinstance(size, bool)}
        previous = self.progress_files.get(attempt["id"])
        self.progress_files[attempt["id"]] = files
        pieces = []
        if previous is not None and not lib.broken("open-file-growth-ignored", self.environment):
            grown = sorted((path for path, size in files.items() if path in previous and size > previous[path]), key=lambda path: previous[path] - files[path])
            if grown:
                more = f"（另有 {len(grown) - 1} 份在涨）" if len(grown) > 1 else ""
                pieces.append(f"子进程写着的 {grown[0]} 从 {previous[grown[0]]} 涨到 {files[grown[0]]} 字节{more}")
        if pieces:
            self.progress_noted[attempt["id"]] = (now, "；".join(pieces))

    def judge_stall_stop(self, task, attempt, size, cpu, now):
        """文件头 ①「卡住」那几条停的规矩：日志与 CPU 都满窗口不涨、日志满窗口不涨而此刻内存贴着上限（CPU 在涨也停）；停了交 True。
        停的那一刻内存此刻或峰值到了上限的 STALL_AT_CAP_FRACTION 的记 memory-cap-hit、照重试规矩重排（顶在上限上卡住）。内存照那一台的接口读（Machine.memory_now）。
        日志与 CPU 都满窗口不涨、而这个窗口里每一轮都有进程在 D 状态的不停（卡在 IO 上；note_io_wait 记，D 状态照那一台的接口读 Machine.io_blocked）。"""
        if lib.broken("stall-ignored", self.environment):
            return False
        seen = self.log_growth.get(attempt["id"])
        if seen is None:
            # 头一回量（服务刚起来对账、或宽限刚过）：日志上一次涨的时刻取日志文件的修改时刻，不当成此刻（服务重启不把「日志多久没涨」清零）；
            # CPU 的窗口照旧从此刻起量（卡住判法不变）。读不到修改时刻、弄坏开关 stall-clock-reset-on-restart 下取此刻
            self.log_growth[attempt["id"]] = (size, self.log_changed_at(attempt, now), now, cpu, "")
            self.io_waits.pop(attempt["id"], None)
            self.note_window_head(task, attempt, now)
            return False
        if seen[0] != size:
            self.log_growth[attempt["id"]] = (size, now, now, cpu, "")
            self.io_waits.pop(attempt["id"], None)   # 日志涨了：卡在 IO 上那一段从头算
            self.note_window_head(task, attempt, now)
            return False
        self.note_io_wait(attempt, seen[2], now)   # 日志没涨的每一轮读一次 D 状态（文件头 ①「卡在 IO 上」）
        self.note_activity(task, attempt, seen[2])   # 日志没涨的每一轮读一次进程内存（文件头 ①「卡住」的有进展）
        if now - seen[2] < self.stall_seconds:
            return False
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        # 此刻贴着上限（任务 7723：上限 6144M，memory.current 6442393600、日志 37702 秒没涨，CPU 平均 2.526 核在回收里打转、跑了 37775 秒）：CPU 在涨、读不到都不算在算，
        # 往下走顶在上限上卡住那一支（记 memory-cap-hit、放大重排）。弄坏开关 at-cap-busy-cpu-kept-running 下不认，at-cap-fraction-ignored 下读得到就算贴着
        current, peak = self.machines[attempt["host"]].memory_now(attempt)
        cap = int(attempt.get("memory_cap") or 0)
        pinned_at_cap = (current is not None and cap > 0 and not lib.broken("at-cap-busy-cpu-kept-running", self.environment)
                         and (current >= STALL_AT_CAP_FRACTION * cap or lib.broken("at-cap-fraction-ignored", self.environment)))
        # 有进展（进程内存或内存峰值这一窗口在涨）：不停，从此刻再看一个窗口；贴着上限的不算（那是在回收里打转）。
        # 弄坏开关 progress-ignored-for-stall 下不看
        progress_ignored = lib.broken("progress-ignored-for-stall", self.environment)
        activity = "" if pinned_at_cap or progress_ignored else activity_text(self.activity_windows.get(attempt["id"]), seen[2])
        if activity:
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的日志 {now - seen[1]:.0f} 秒没涨，而{activity}：有进展，不停，再看一个窗口")
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_ACTIVITY)
            self.note_window_head(task, attempt, now)
            return False
        if not pinned_at_cap and (seen[3] is None or cpu is None) and not lib.broken("cpu-unread-counted-as-idle", self.environment):
            # 窗口开头或此刻读不到 CPU（刚起、第二台那一趟还在起、换上读法之前起的尝试没有 cpu_usec）：读不到不等于不涨，不停，从此刻再看一个窗口
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_CPU_UNREAD)
            self.note_window_head(task, attempt, now)
            return False
        cores = (cpu - seen[3]) / 1e6 / (now - seen[2]) if cpu is not None and seen[3] is not None else None
        if (not pinned_at_cap and cores is not None and cores >= STALL_CPU_CORES
                and not lib.broken("cpu-ignored-for-stall", self.environment)):
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_CPU_BUSY)   # 日志不涨而 CPU 在涨：在算，不停，从此刻再看一个窗口
            self.note_window_head(task, attempt, now)
            return False
        waited = self.io_waits.get(attempt["id"])
        if (not pinned_at_cap and waited is not None and waited.polls > 0 and waited.every_poll_blocked
                and not lib.broken("io-wait-ignored", self.environment)):
            # 日志与 CPU 都不涨而这个窗口每一轮都有进程在 D 状态：在等盘或 NFS，不判 stalled、不停，从此刻再看一个窗口（满 io_wait_attention_seconds 由 io_wait_items 报 io-wait）
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨，"
                f"而这 {waited.polls} 轮每一轮都有进程在 D 状态（最近一轮 {waited.blocked_now} 个）：在等 IO，不停，再看一个窗口")
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_IO_WAIT)
            self.note_window_head(task, attempt, now)
            return False
        tail = log_tail(attempt.get("log", "")) if not pinned_at_cap and not progress_ignored else ""
        if "CRASH_AMPLIFICATION_PLACEMENT judge=gpu" in tail or "CRASH_AMPLIFICATION_DECISION where=local" in tail:
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨，"
                "而在等 gpu-verification-server 判待判块：不停，再看一个窗口")
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, SPARED_ACTIVITY)
            self.note_window_head(task, attempt, now)
            return False
        attempt["stalled"] = True
        attempt["stalled_after"] = round(now - seen[1], 3)
        peak_at_cap = peak is not None and cap > 0 and peak >= STALL_AT_CAP_FRACTION * cap
        if (pinned_at_cap or peak_at_cap) and not lib.broken("stalled-at-cap-not-retried", self.environment):
            # 顶在上限上卡在回收里：不记 stalled，停掉之后照撞上限办（finish_attempt 记 memory-cap-hit、照重试规矩乘 1.5 重排）
            attempt["stalled_at_cap"] = True
            measured = "、".join(text for text in (f"此刻内存 {lib.mebibytes_text(current)}" if current is not None else "",
                                                   f"内存峰值 {lib.mebibytes_text(peak)}" if peak is not None else "") if text)
            busy = f"，CPU 这 {now - seen[2]:.0f} 秒平均 {cores:.2f} 个核（在回收里打转，不算在算）" if pinned_at_cap and cores is not None and cores >= STALL_CPU_CORES else ""
            attempt["stall_note"] = (f"顶在上限上卡住：{measured}、上限 {attempt.get('memory_cap_text') or lib.mebibytes_text(cap)}"
                                     f"（不低于 {STALL_AT_CAP_FRACTION:.0%}），日志 {now - seen[1]:.0f} 秒没涨{busy}")
            self.store.update("attempts", attempt)
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试{attempt['stall_note']}：停掉它，按撞上限办")
            threading.Thread(target=self.stop_running, args=(task["id"],), kwargs=self.judged_attempt_only(attempt), daemon=True).start()
            return True
        attempt["stall_note"] = f"日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨"   # 判卡住的依据记进尝试，结局说明接上它
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的{attempt['stall_note']}：停掉它，记 stalled")
        threading.Thread(target=self.stop_running, args=(task["id"],), kwargs=self.judged_attempt_only(attempt), daemon=True).start()
        return True

    def judged_attempt_only(self, attempt):
        """判卡住（judge_stall_stop）起的后台线程交给 stop_running 的那几个参数：只停判它的那一次（attempt_id）。
        线程起来到读库之间那一次自己退了、照重试规矩重排起了下一次的，下一次不归它停。弄坏开关 stall-stop-by-task 下不给，按任务号停在跑的全部。"""
        return {} if lib.broken("stall-stop-by-task", self.environment) else {"attempt_id": attempt["id"]}

    def run_hard_limit_verdict(self, task, attempt):
        """这一次尝试要不要照四小时硬上限停（文件头 ⑦「四小时硬上限」）→ 要记进尝试的 {seconds, how} 或 None：带 --full-gate 的不判；累计（run_limit_accumulated）到了 run_hard_limit_seconds 的停；
        子任务另看父任务整件满界了没有（phase=lib.RUN_HARD_LIMIT_PHASE）。各弄坏开关见文件头 ⑦ 那一段。"""
        environment = self.environment
        if lib.broken("run-hard-limit-skipped", environment):
            return None
        # 提交门禁（交的时候带 --full-gate、弹窗由用户确认过的那几道）不受四小时上限管：它判的是整棵树，本来就要跑这么久，拆不了。
        # 别的任务实际用时满四小时一律停，逼提交它的会话拆小再交。
        if self.full_gate_of(task) and not lib.broken("run-hard-limit-full-gate-judged", environment):
            return None
        if self.long_run_confirmed_of(task) and lib.broken("run-hard-limit-confirmed-spared", environment):
            return None
        if lib.broken("run-hard-limit-writers-spared", environment) and self.modifies_workspace(task):
            return None
        total, current, earlier, _ = self.run_limit_accumulated(task, attempt)
        spared_child = task.get("parent_id") and lib.broken("run-hard-limit-children-spared", environment)
        if lib.over_run_hard_limit(total, self.run_hard_limit_seconds, environment) and not spared_child:
            each = "、".join(f"第 {number} 次 {seconds:.0f} 秒" for number, seconds in sorted(earlier)) + ("、" if earlier else "") + f"第 {attempt.get('number')} 次 {current:.0f} 秒（在跑）"
            return {"seconds": round(total, 3), "how": f"各次尝试在跑的时长累计 {total:.0f} 秒（{each}）"}
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1] and parent[0].get("phase") == lib.RUN_HARD_LIMIT_PHASE:
                return {"seconds": round(total, 3), "how": f"父任务 {task['parent_id']} 整件满界：{(parent[0].get('run_hard_limit_stop') or {}).get('how') or '没记'}"}
        return None

    def judge_run_hard_limit(self, task, attempt):
        """四小时硬上限判这一次尝试（文件头 ⑦「四小时硬上限」）→ 这一轮归它管了没有（管了 check_stall 不再往下判）：判停过的接着停，这一轮判到的记下、停这一次。"""
        if attempt.get("run_hard_limit_stop"):
            self.stop_over_limit_attempt(task, attempt)
            return True
        record = self.run_hard_limit_verdict(task, attempt)
        if record is None:
            return False
        self.note_run_hard_limit_stop(task, attempt, record)
        return True

    def note_run_hard_limit_stop(self, task, attempt, record):
        """这一次尝试判了四小时硬上限：record 记进尝试的 run_hard_limit_stop（结局说明照它写），停这一次。"""
        attempt["run_hard_limit_stop"] = record
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试{record['how']}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限：停这一次，打回")
        self.stop_over_limit_attempt(task, attempt)

    def settle_run_hard_limit(self, task, attempt, exit_code, counts):
        """照四小时硬上限停的那一次尝试停下来了（结局 lib.RUN_HARD_LIMIT_OUTCOME）：任务不重排，照 over-four-hours 收尾（文件头 ⑦「四小时硬上限」）。"""
        record = attempt.get("run_hard_limit_stop") or {}
        self.finalize(task, lib.RUN_HARD_LIMIT_OUTCOME, exit_code,
                      (f"{self.finished_preface(task, attempt, lib.RUN_HARD_LIMIT_OUTCOME)}{record.get('how') or '没记'}，到了 {self.run_hard_limit_seconds:.0f} 秒的硬上限："
                       "服务停了这一次尝试、不重排，--why 放不过"), outcome_counts=counts)

    def parent_run_hard_limit(self, parent, children):
        """分片的父任务整件判不判四小时硬上限（文件头 ⑦「四小时硬上限」）→ 要记进父任务的 {seconds, how} 或 None。整件不另判（用户定「按每一片自己的累计用时算」）：
        各片、计划、合并各按自己的尝试累计（run_hard_limit_verdict），一片满界停那一片、记 over-four-hours，别的片照跑，收齐了整件照那一片的结局收尾（drive_parent_phase
        的「一步没成」）；排队把片一片片串起来的挂钟不算进谁的账。交 None。
        弄坏开关 shard-whole-by-wall-clock 下整件按有片在跑的挂钟算：有片在跑的时段取并集（lib.covered_seconds），片之和不算、全都在排的时段不算，撞了内存上限的那一片
        只从它最后一次撞上限之后那一次起算（同 run_limit_accumulated），带 --full-gate 的不判；run-hard-limit-skipped 下不算，long-run-cap-hits-counted 下撞上限的那几次也算。"""
        environment = self.environment
        if not lib.broken("shard-whole-by-wall-clock", environment) or lib.broken("run-hard-limit-skipped", environment):
            return None
        if self.full_gate_of(parent) and not lib.broken("run-hard-limit-full-gate-judged", environment):
            return None
        now = time.time()
        intervals = []
        for child in children:
            attempts = [record for record, valid in self.store.attempts_of(child["id"]) if valid and record.get("started_at")]
            hits = [int(record.get("number") or 0) for record in attempts if record.get("state") == "ended" and record.get("outcome") in lib.RECORD_CAP_HIT_OUTCOMES]
            floor = max(hits) if hits and not lib.broken("long-run-cap-hits-counted", environment) else 0
            intervals += [(float(record["started_at"]), float(record.get("ended_at") or now)) for record in attempts if int(record.get("number") or 0) > floor]
        seconds = lib.covered_seconds(intervals, environment)
        if not lib.over_run_hard_limit(seconds, self.run_hard_limit_seconds, environment):
            return None
        return {"seconds": round(seconds, 3), "how": f"分片的整件有片在跑的挂钟时长 {seconds:.0f} 秒（片之和不算）"}

    def stop_over_limit_attempt(self, task, attempt):
        """停照四小时硬上限判停的这一次尝试（只停这一次，judged_attempt_only）：后台线程起 stop_running，这一次还在停的不再起；没停成的（stop_running 退了、单元还在跑）下一轮 check_stall 再起，
        服务重启之后 check_stall 照尝试记的 run_hard_limit_stop 接着停。弄坏开关 hard-limit-stop-once 下线程退了也不让再起。"""
        attempt_id = attempt["id"]
        if attempt_id in self.hard_limit_stops:
            return
        self.hard_limit_stops.add(attempt_id)

        def stop():
            try:
                self.stop_running(task["id"], **self.judged_attempt_only(attempt))
            finally:
                if not lib.broken("hard-limit-stop-once", self.environment):
                    self.hard_limit_stops.discard(attempt_id)
        threading.Thread(target=stop, daemon=True).start()

    def long_run_lines(self, task):
        """这条任务（分片的父任务连同各子任务）长跑的几行（task status、列表、推给会话的正文与日志头都打）：确认过的「长跑已确认：<why>」，在跑而满界没确认的「长跑待确认：…」。
        弄坏开关 long-run-confirmed-flag-hidden 下不打确认过的那一行。"""
        records = [task]
        if task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                records += self.children_of(task["id"])
        lines = []
        for record in records:
            part = f"（子任务 {record['id']}）" if record.get("id") != task.get("id") else ""
            flag = record.get("long_run_confirmed")
            if flag and not lib.broken("long-run-confirmed-flag-hidden", self.environment):
                lines.append(f"长跑已确认{part}：{flag.get('why')}")
            elif not flag and record.get("state") == "running":
                attempts = [attempt for attempt, valid in self.store.attempts_of(record["id"]) if valid]
                if self.long_run_unconfirmed_items(record, attempts):
                    lines.append(f"长跑待确认{part}：累计跑满 {self.run_limit_seconds:.0f} 秒没确认，服务照跑、满 {self.run_hard_limit_seconds:.0f} 秒的硬上限照停；范围对、确是要跑这么久的 {lib.long_run_confirm_command(record['id'])}，"
                                 f"不对的 task-server/task cancel {record['id']}、改了范围重交")
        return lines

    def full_gate_of(self, task):
        """这条任务（子任务照父任务）带没带全量门禁标识（task submit --full-gate）。"""
        if task.get("full_gate"):
            return True
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            return parent is not None and parent[1] and bool(parent[0].get("full_gate"))
        return False

    def long_run_confirmed_of(self, task):
        """这条任务确认过要跑过两小时：自己确认过，或（子任务）父任务确认过（文件头「跑满两小时提醒」）。
        弄坏开关 long-run-why-children-reminded 下子任务不看父任务。"""
        if task.get("long_run_confirmed"):
            return True
        if task.get("parent_id") and not lib.broken("long-run-why-children-reminded", self.environment):
            parent = self.store.task(task["parent_id"])
            return parent is not None and parent[1] and bool(parent[0].get("long_run_confirmed"))
        return False

    def whole_package_of(self, task):
        """这条任务（子任务照父任务）带的「整包已确认」标识：确认文件的路径或空串（文件头 ②「提交时拒」⑥）。"""
        if task.get("whole_package_confirmed"):
            return task["whole_package_confirmed"]
        if task.get("parent_id"):
            parent = self.store.task(task["parent_id"])
            if parent is not None and parent[1]:
                return parent[0].get("whole_package_confirmed") or ""
        return ""

    def flag_lines(self, task):
        """确认标识的行（task status、列表、推给会话的正文与日志头都打）：「整包已确认：<文件>」、「全量门禁」与长跑那几行（long_run_lines），没有的不打。"""
        whole = self.whole_package_of(task)
        return ([f"整包已确认：{whole}"] if whole else []) + (["全量门禁"] if task.get("full_gate") else []) + self.long_run_lines(task)

    def run_limit_accumulated(self, task, attempt):
        """这条任务（分片的子任务按自己的）各次尝试在跑的时长累加（文件头 ①「跑满两小时提醒」）：撞了内存上限重来的计时归零、只从最后一次撞上限之后那一次起算
        （结束了的取记下的 seconds，这一次取此刻已跑的；排队等的不算）。→ (累计秒, 这一次已跑秒, 算进去的前几次 [(第几次, 秒)…], 撞过上限的次数)。
        弄坏开关 long-run-cap-hits-counted 下撞上限的那几次也累加。"""
        current = max(0.0, time.time() - float(attempt.get("started_at") or time.time()))
        ended = sorted((int(record.get("number") or 0), float(record.get("seconds") or 0), record.get("outcome")) for record, valid in self.store.attempts_of(task["id"])
                       if valid and record.get("state") == "ended" and record.get("id") != attempt.get("id"))
        cap_hit_numbers = [number for number, _, outcome in ended if outcome in lib.RECORD_CAP_HIT_OUTCOMES]
        if cap_hit_numbers and not lib.broken("long-run-cap-hits-counted", self.environment):
            ended = [entry for entry in ended if entry[0] > max(cap_hit_numbers)]
        earlier = [(number, seconds) for number, seconds, _ in ended]
        return current + sum(seconds for _, seconds in earlier), current, earlier, len(cap_hit_numbers)

    def run_limit_exempt(self, task):
        """满界不提醒的（文件头 ①「跑满两小时提醒」）：确认过的（long_run_confirmed_of）、交的时候带了全量门禁标识的（full_gate_of，full_gate_refusal 核过）。
        弄坏开关 long-run-unconfirmed-silent 下一律不提醒，long-run-confirmed-still-reminded 下确认过的照提醒，full-gate-flag-ignored 下带了也照提醒，
        whole-package-confirmed-exempts-two-hours 下整包已确认的不提醒。不指定范围的提交时门禁没带全量标识的提交时就被拒（whole_package_refusal），走不到这里。"""
        if lib.broken("long-run-unconfirmed-silent", self.environment):
            return True
        if self.long_run_confirmed_of(task) and not lib.broken("long-run-confirmed-still-reminded", self.environment):
            return True
        if self.whole_package_of(task) and lib.broken("whole-package-confirmed-exempts-two-hours", self.environment):
            return True
        # 提交门禁（带 --full-gate 的）同样不推两小时的拆分提醒：理由同四小时上限那一处（run_hard_limit_verdict）
        return bool(self.full_gate_of(task)) and not lib.broken("full-gate-flag-ignored", self.environment)

    def long_run_unconfirmed_items(self, record, attempts):
        """跑满两小时没确认（文件头 ①「要处置的提醒」的 long-run-unconfirmed）：在跑的那一次尝试连同算进去的前几次累计满 run_limit_seconds、又不在 run_limit_exempt 里的交一项，
        提醒门槛是 1 加撞过上限的次数（撞了上限归零，再满界算跨过一道新门槛）；依据写累计与各次用时、命令、范围声明、日志末几行。只提醒、照跑。"""
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        if not active or self.run_limit_exempt(record):
            return []
        attempt = active[-1]
        total, current, earlier, cap_hits = self.run_limit_accumulated(record, attempt)
        if total < self.run_limit_seconds:
            return []
        each = "、".join(f"第 {number} 次 {seconds:.0f} 秒" for number, seconds in sorted(earlier)) + ("、" if earlier else "") + f"第 {attempt['number']} 次 {current:.0f} 秒（在跑）"
        scope = lib.scope_declaration_segments(record.get("environment") or {}, record.get("command") or [])
        tail = [line for line in lib.log_lines(log_tail(attempt.get("log", ""))) if line.strip()][-3:]
        return [self.attention_item(
            "long-run-unconfirmed", 1 + cap_hits, record["id"],
            (f"累计跑了 {total:.0f} 秒（{total / 60:.0f} 分钟），满 {self.run_limit_seconds:.0f} 秒的界：各次 {each}；命令：{shlex.join(record.get('command') or [])}；"
             f"范围声明：{'；'.join(scope) if scope else '没带 SINGLEFS_RUN_SCOPE'}；日志末几行：{' ⏎ '.join(tail) if tail else '日志读不到'}"),
            f"实际用时过了两小时：建议现在就把这一件拆小（点名范围：--item、--check 少几格、点名测试目标，或照服务分片），拆完重交；此刻服务不停它、不改任务；"
            f"累计满 {self.run_hard_limit_seconds:.0f} 秒的硬上限服务直接停掉这一次、记 over-four-hours，那时只能拆了再交")]

    def long_run_sessionless_note(self, task, attempt):
        """没有会话的（人手 --manual）跑满两小时没确认（文件头 ①「要处置的提醒」的 long-run-unconfirmed）：不推，往这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的（不算日志在涨），
        每次尝试一行（记 long_run_noted）。弄坏开关 long-run-sessionless-unnoted 下不写。"""
        if task.get("session") or attempt.get("long_run_noted") or lib.broken("long-run-sessionless-unnoted", self.environment):
            return
        items = self.long_run_unconfirmed_items(task, [attempt])
        if not items:
            return
        attempt["long_run_noted"] = True
        self.store.update("attempts", attempt)
        append_log_lines(attempt.get("log", ""), [f"{lib.LONG_RUN_LOG_MARK} 长跑待确认：{items[0]['basis']}；出路：{items[0]['next']}"])

    def note_io_wait(self, attempt, window_started_at, now):
        """日志没涨的这一轮读一次这一次尝试此刻在 D 状态的进程数（Machine.io_blocked），记进 io_waits（文件头 ①「卡在 IO 上」）：
        窗口（window_started_at，log_growth 记的那一个）换了从头记这个窗口里是不是每一轮都有；从哪一刻起每一轮都有跨窗口接着算，有一轮没有或读不到就清掉。"""
        blocked = self.machines[attempt["host"]].io_blocked(attempt)
        previous = self.io_waits.get(attempt["id"])
        fresh = previous is None or previous.window_started_at != window_started_at
        every = (True if fresh else previous.every_poll_blocked) and bool(blocked)
        since = None
        if blocked:
            since = previous.blocked_since if previous is not None and previous.blocked_since is not None else now
        self.io_waits[attempt["id"]] = IoWaitWindow(window_started_at, (0 if fresh else previous.polls) + 1, every, blocked, since)

    def note_window_head(self, task, attempt, window_started_at):
        """判卡住开新窗口的那一轮（log_growth 记下新的窗口开头 window_started_at：头一回量、日志涨了、上一个窗口满了没停）就读这个窗口的头一份进程内存与峰值（note_activity），
        不等到窗口里日志不涨的下一轮才记：一轮排队比卡住窗口还长时，到判窗口那一轮窗口里至少有头尾两份读数，在涨的才看得出涨（activity_text）。
        弄坏开关 stall-window-head-unread 下开窗那一轮不读，头一份读数在下一轮才记（那一轮窗口里只有一份，涨多少都当没进展）。"""
        if not lib.broken("stall-window-head-unread", self.environment):
            self.note_activity(task, attempt, window_started_at)

    def note_activity(self, task, attempt, window_started_at):
        """开新窗口的那一轮（note_window_head）与窗口里日志没涨的每一轮读一次这一次尝试的进程内存与峰值（Machine.memory_now），
        记进 activity_windows（activity_window_after；文件头 ①「卡住」的有进展）。"""
        memory, peak = self.machines[attempt["host"]].memory_now(attempt)
        self.activity_windows[attempt["id"]] = activity_window_after(self.activity_windows.get(attempt["id"]), window_started_at, memory, peak)

    def check_suspect(self, task, attempt, cpu, now):
        """疑似卡住（文件头 ①「疑似卡住」）：整个单元的 CPU 满 stall_seconds 平均不到 STALL_CPU_CORES 个核，
        不管日志涨不涨：不停，记进尝试的 suspect、写告警、叫醒在等的 wait（它打 lib.SUSPECT_MARK 那一行、退 lib.SUSPECT_EXIT）。
        一段只报一次；疑似之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核），这一段结束，从此刻再量下一段。读不到 CPU 不当不涨：窗口作废重量。
        弄坏开关 suspect-unreported 下不判。"""
        if lib.broken("suspect-unreported", self.environment):
            return
        if cpu is None:
            self.cpu_windows.pop(attempt["id"], None)
            return
        window = self.cpu_windows.get(attempt["id"])
        if window is None or cpu < window.last_cpu:
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
            return
        if window.suspected:
            if (cpu - window.last_cpu) / 1e6 / max(now - window.last_at, 1e-6) >= STALL_CPU_CORES:   # 这一段结束：CPU 又涨了
                self.end_suspect(task, attempt, "CPU 又涨了")
                self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
                return
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu)
            return
        if now - window.started_at < self.stall_seconds:
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu)
            return
        cores = (cpu - window.started_cpu) / 1e6 / max(now - window.started_at, 1e-6)
        if cores >= STALL_CPU_CORES:   # 在算：从此刻再量一个窗口
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, False)
            return
        self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu, suspected=True)
        previous = attempt.get("suspect") or {}
        attempt["suspect"] = {"active": True, "segment": int(previous.get("segment") or 0) + 1, "at": round(time.time(), 3),
                              "window_seconds": round(now - window.started_at, 3), "cores": round(cores, 6), "threshold_cores": STALL_CPU_CORES,
                              "why": f"整个单元的 CPU 这 {now - window.started_at:.0f} 秒平均 {cores:.4f} 个核，不到 {STALL_CPU_CORES} 个核（日志涨不涨都不看）"}
        # 先写告警、再记进尝试：wait 那一侧读到 suspect 时告警已在
        self.alert("suspected-stall", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试疑似卡住：{attempt['suspect']['why']}；服务不停它",
                   f"提交它的会话自己查（task-server/task log {task['id']}、task-server/task status {task['id']}），还在算就再起 task-server/task wait {task['id']} 接着等，"
                   f"真卡住了 task-server/task cancel {task['id']}")
        self.store.update("attempts", attempt)
        self.notify_finished(task["id"])   # 叫醒在等的 wait：它照 suspect 打一行、退出

    def unreported_suspect(self, task_id, session):
        """wait 那一侧调：这条任务在跑的那一次正疑似卡住、这一段还没报给提交它的会话，交那一段（连同第几次、哪台、日志），不然交 None。
        报给提交它的会话就记下这一段报过了（记在服务的内存里：只有排队线程写尝试记录，不和它抢着写）；别的会话来等照样看得到、不记。
        弄坏开关 suspect-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return None
        for attempt, valid in self.store.attempts_of(task_id):
            suspect = attempt.get("suspect") or {}
            if not valid or attempt.get("state") not in ACTIVE_ATTEMPT_STATES or not suspect.get("active"):
                continue
            with self.suspect_lock:
                if self.suspect_reported.get(attempt["id"]) == suspect.get("segment") and not lib.broken("suspect-reported-every-wait", self.environment):
                    continue
                recorded = found[0].get("session") == session
                if recorded:
                    self.suspect_reported[attempt["id"]] = suspect.get("segment")
            if recorded:
                self.persist_restart_surviving_state("suspect_reported")
            return dict(suspect, attempt=attempt.get("number"), host=attempt.get("host"), log=attempt.get("log"))

    def end_suspect(self, task, attempt, why):
        suspect = dict(attempt.get("suspect") or {}, active=False, ended_at=round(time.time(), 3), ended_because=why)
        attempt["suspect"] = suspect
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的疑似卡住（第 {suspect.get('segment')} 段）结束：{why}")

    def attention_items(self, task_id):
        """这条任务（分片的连同子任务）此刻要处置的提醒（文件头 ①「要处置的提醒」）：每项 {kind, threshold, task, basis, why, next}（时长判的另有 reminded_after_seconds），task 是提醒说的那一条
        （子任务的是子任务号）。不改任务；十八种判法（lib.ATTENTION_KINDS 一种一个）：十七种在 attention_of_record（预算贴着整台读 budget_check 排队时记下的那一项，在每一台上都起不来读 note_idle_short 记下的那一项；后者开着的那条在排的任务不报前者，「在每一台上都起不来」那一判还没对它判过的在排任务也不报前者），分片推不动在下面（改主工作区的那一种另起一次 apply-writer-patch.py --read-outcome，workspace_write_item），提醒门槛按 reminder_threshold。
        弄坏开关 attention-shard-advance-unreported 下不报分片父任务推不动（别几种的开关在 attention_of_record）。"""
        found = self.store.task(task_id)
        if found is None or not found[1]:
            return []
        records = [found[0]]
        if found[0].get("sharded_entry"):
            try:
                records += self.children_of(task_id)
            except StoreRefused:   # 名下有校验不过的子任务行：不认那几行（summary 与 cancel 已写告警）
                pass
        now = time.time()
        items = []
        for record in records:
            items += self.attention_of_record(record, now)
        failure = dict(self.advance_failures.get(task_id) or {})
        if failure and found[0].get("state") == "sharded" and not lib.broken("attention-shard-advance-unreported", self.environment):
            items.append(self.attention_item("shard-advance-failed", int(failure.get("episode") or 1), task_id,
                                             f"推下一步连着出错 {failure.get('count')} 轮：{failure.get('why')}",
                                             "服务每一轮对账都重推，一直推不动整件就停在原地"))
        return items

    def attention_item(self, kind, threshold, task_id, basis, why, follow="", reminded_after=None):
        """要处置的提醒的一项（文件头 ①「要处置的提醒」）：threshold 是跨过的提醒门槛；时长判的那几种给 reminded_after（这一回跨过的那道门槛，秒），记成 reminded_after_seconds。
        next 是异常表里这一种的修改意见（lib.anomaly_advice），follow 给了的接在后面（这一回的具体数）。每一种都只经这里拼 next，不另写出路句。"""
        advice = lib.anomaly_advice(kind, task_id, self.environment)
        item = {"kind": kind, "threshold": threshold, "task": task_id, "basis": basis, "why": why, "next": f"{advice}；{follow}" if follow else advice}
        if reminded_after is not None:
            item["reminded_after_seconds"] = round(reminded_after)
        return item

    def quiet_log_items(self, record, attempts):
        """日志长期不涨而服务没停它（文件头 ①「要处置的提醒」的 quiet-log-busy）：在跑的那一次尝试的日志（log_content 算的那部分）距上一次增长
        满 quiet_attention_seconds 交一项，不然交空表。日志多久没涨照判卡住记的那一份（log_growth，宽限里还没量的不报）。
        只在服务最近一个窗口因 CPU 在涨或读不到 CPU 没停它时报（log_growth 记的缘由是 SPARED_CPU_BUSY、SPARED_CPU_UNREAD）：按 D 状态免停的由
        io_wait_items 报，还没判过一个窗口的不报。弄坏开关 quiet-log-busy-any-exemption 下不看缘由。
        日志不涨而有进展的不算：多久没涨从日志上一次涨与最近一回读到进展（progress_noted：子进程写着的文件在涨；note_attempt_progress 记）里晚的那一刻起算，
        读到进展之后不满门槛的不报；弄坏开关 quiet-log-busy-progress-ignored 下不看进展、只从日志上一次涨起算。"""
        task_id = record["id"]
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        seen = self.log_growth.get(active[-1]["id"]) if active else None
        if seen is None:
            return []
        if seen[4] not in (SPARED_CPU_BUSY, SPARED_CPU_UNREAD) and not lib.broken("quiet-log-busy-any-exemption", self.environment):
            return []
        now = time.monotonic()
        noted = None if lib.broken("quiet-log-busy-progress-ignored", self.environment) else self.progress_noted.get(active[-1]["id"])
        quiet = now - (max(seen[1], noted[0]) if noted is not None else seen[1])
        crossed = reminder_threshold(quiet, self.quiet_attention_seconds)
        if not crossed:
            return []
        since_progress = f"，上一回读到进展（{noted[1]}）是 {now - noted[0]:.0f} 秒前" if noted is not None else ""
        return [self.attention_item(
            "quiet-log-busy", crossed, task_id,
            (f"第 {active[-1].get('number')} 次尝试的日志 {now - seen[1]:.0f} 秒（{(now - seen[1]) / 60:.0f} 分钟）没涨{since_progress}，"
             f"日志与进展满 {next_reminder_seconds(crossed, self.quiet_attention_seconds):.0f} 秒没动再提醒一回"),
            (f"日志不涨而{'读不到 CPU' if seen[4] == SPARED_CPU_UNREAD else 'CPU 在涨'}的服务不停它：日志与 CPU 都不涨才停"),
            (f"还在算就再起 task-server/task wait {task_id}，"
             f"在空转或在回收里打转 task-server/task cancel {task_id}、改了重交"),
            self.quiet_attention_seconds * 2 ** (crossed - 1))]

    def io_wait_items(self, record, attempts):
        """卡在 IO 上而服务没停它（文件头 ①「要处置的提醒」的 io-wait）：在跑的那一次尝试日志没涨的每一轮都读到有进程在 D 状态（io_waits 的 blocked_since），
        连着满 io_wait_attention_seconds 交一项，不然交空表。依据写最近一轮 D 状态的进程数与本机此刻 /proc/pressure/io 的 some avg60。"""
        task_id = record["id"]
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        waited = self.io_waits.get(active[-1]["id"]) if active and not active[-1].get("stalled") else None   # 已判停的那一次在等停下来，不再报
        if waited is None or waited.blocked_since is None:
            return []
        blocked_for = time.monotonic() - waited.blocked_since
        crossed = reminder_threshold(blocked_for, self.io_wait_attention_seconds)
        if not crossed:
            return []
        host = lib.HOST_NAMES.get(active[-1].get("host"), active[-1].get("host"))
        return [self.attention_item(
            "io-wait", crossed, task_id,
            (f"第 {active[-1].get('number')} 次尝试（{host}）日志没涨，连着 {blocked_for:.0f} 秒（{blocked_for / 60:.0f} 分钟）每一轮都有进程在 D 状态"
             f"（不可中断的 IO 等待），最近一轮 {waited.blocked_now} 个；本机此刻 /proc/pressure/io {io_pressure_text()}；"
             f"连着满 {next_reminder_seconds(crossed, self.io_wait_attention_seconds):.0f} 秒再提醒一回"),
            "日志不涨而每一轮都有进程在等 IO（D 状态）：多半在等盘或 NFS，服务不因为日志不涨判它卡住、不停它，IO 缓过来它自己接着跑",
            (f"这一次在{host}：等得起再起 task-server/task wait {task_id}，"
             f"盘与 NFS 修不好 task-server/task cancel {task_id}、IO 缓过来再重交"),
            self.io_wait_attention_seconds * 2 ** (crossed - 1))]

    def unit_unreadable_items(self, record, attempts):
        """读不到单元状态（文件头 ①「要处置的提醒」的 unit-unreadable）：在跑的那几次尝试里对账连着读不到单元状态满 unit_unreadable_attention_seconds 的各交一项
        （提醒门槛按连着读不到多久），依据写尝试派在哪台、本机读不到哪个单元的状态、不经 systemd 最近一回核到的；不然交空表。"""
        task_id, now, items = record["id"], time.monotonic(), []
        for attempt in attempts:
            entry = self.unit_unreadable.get(attempt.get("id")) if attempt.get("state") in ACTIVE_ATTEMPT_STATES else None
            if entry is None:
                continue
            elapsed = now - entry["since"]
            crossed = reminder_threshold(elapsed, self.unit_unreadable_attention_seconds)
            if not crossed:
                continue
            verdict, detail = self.unit_presence_now(attempt["id"])
            host = lib.HOST_NAMES.get(attempt.get("host"), attempt.get("host"))
            items.append(self.attention_item(
                "unit-unreadable", crossed, task_id,
                (f"第 {attempt.get('number')} 次尝试（派在{host}）：本机读不到它的单元 {attempt.get('unit')} 的状态（systemctl --user show 起不来、超时或没打 ActiveState），"
                 f"连着 {elapsed:.0f} 秒；不经 systemd 最近一回核到{lib.PRESENCE_WORDS[verdict]}：{detail}；"
                 f"读不到的连着满 {next_reminder_seconds(crossed, self.unit_unreadable_attention_seconds):.0f} 秒再提醒一回"),
                "读不到单元状态时服务判不出它在不在：不杀、不收尾、不记结局，尝试照记 running；核到 cgroup 读得到且空、记下的进程号都不在才记 launcher-lost，核不出就一直等",
                f"这一次的单元是 {attempt.get('unit')}", self.unit_unreadable_attention_seconds * 2 ** (crossed - 1)))
        return items

    def cores_short_of_threads(self, cores, threads):
        """到此刻平均在用 cores 个核算不算没用满 threads 线程（LONG_RUN_IDLE_CORE_FRACTION）。弄坏开关 long-run-idle-quarter-threshold 下界是四分之一。"""
        fraction = LONG_RUN_IDLE_QUARTER_BREAK if lib.broken("long-run-idle-quarter-threshold", self.environment) else LONG_RUN_IDLE_CORE_FRACTION
        return cores < threads * fraction

    def long_run_idle_items(self, record, attempts, now):
        """跑得久而核没用起来（文件头 ①「要处置的提醒」的 long-run-idle-cores）：在跑的那一次尝试跑满 long_run_attention_seconds、到此刻平均在用核数（Machine.cpu_usec 除以已跑秒数，
        同 running_progress）没用满给的线程数（threads_of；cores_short_of_threads），交一项，提醒门槛是尝试的次数（每次尝试只报一次，不随时长再报）；读不到 CPU 的不报。只提醒、照跑。"""
        active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
        if not active:
            return []
        attempt = active[-1]
        elapsed = now - float(attempt.get("started_at") or now)
        if elapsed < self.long_run_attention_seconds:
            return []
        machine = self.machines.get(attempt.get("host"))
        usage = machine.cpu_usec(attempt) if machine is not None else None
        threads = int(self.threads_of(record) or 0)
        if usage is None or threads <= 0:
            return []
        cores = usage / 1e6 / elapsed
        if not self.cores_short_of_threads(cores, threads):
            return []
        peak = machine.memory_now(attempt)[1]
        task_id = record["id"]
        return [self.attention_item(
            "long-run-idle-cores", max(1, int(attempt.get("number") or 1)), task_id,
            (f"第 {attempt.get('number')} 次尝试已跑 {elapsed:.0f} 秒（{elapsed / 60:.0f} 分钟），给 {threads} 线程、平均只用 {cores:.1f} 核"
             f"（不到 {threads * LONG_RUN_IDLE_CORE_FRACTION:.0f} 核），进程内存峰值 {lib.size_text(peak)}"),
            f"跑满 {self.long_run_attention_seconds / 60:.0f} 分钟而核没用满：资源运用是否合理、脚本能不能并行或切片？（服务照跑，不停它）",
            f"照跑就再起 task-server/task wait {task_id}")]

    def fills_any_cap_items(self, record):
        """按给的上限开并行（文件头 ①「要处置的提醒」的 fills-any-cap）：这条命令键最近 FILLS_CAP_RUNS 次跑成的尝试（只读进程内存口径的行，current_metric）
        峰值都不低于各自上限的 FILLS_CAP_FRACTION，交一项（每条任务只报一次）；不到这么多次的不报。撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，
        照上限一档档撞上去的（任务 9616 那条静态核 6.0、9.0、13.5 GiB）不是按上限开并行。上限本来就照记下的峰值定的那几次（尝试记的 prediction 一栏是 lib.RECORD_HISTORY）也不算：
        峰值贴着上限是定法的结果，命令未必读上限。弄坏开关 fills-any-cap-counts-cap-hits 下撞了上限的也算进那几次，fills-any-cap-counts-record-caps 下照记录定的上限也算。"""
        counted = lib.RECORD_COMPLETED_OUTCOMES + (lib.RECORD_CAP_HIT_OUTCOMES if lib.broken("fills-any-cap-counts-cap-hits", self.environment) else ())
        runs = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (record.get("command_key"),), order="id DESC LIMIT 50"):
            if not valid or not self.current_metric(attempt):
                continue
            if attempt.get("prediction") == lib.RECORD_HISTORY and not lib.broken("fills-any-cap-counts-record-caps", self.environment):
                continue
            outcome, peak, cap = attempt.get("outcome"), attempt.get("memory_peak"), attempt.get("memory_cap")
            if outcome not in counted or not isinstance(peak, int) or not isinstance(cap, int) or cap <= 0:
                continue
            runs.append((float(attempt.get("ended_at") or 0), peak, cap))
        runs = sorted(runs, reverse=True)[:FILLS_CAP_RUNS]
        if len(runs) < FILLS_CAP_RUNS or any(peak < cap * FILLS_CAP_FRACTION for _, peak, cap in runs):
            return []
        human = self.host_dispatch.human
        task_id = record["id"]
        shown = "、".join(f"上限 {human(cap)} 峰值 {human(peak)}" for _, peak, cap in runs)
        return [self.attention_item(
            "fills-any-cap", 1, task_id, f"这条命令键最近 {FILLS_CAP_RUNS} 次：{shown}（每次都不低于上限的 {FILLS_CAP_FRACTION:.0%}）",
            "给多少就用满多少：命令多半照服务交的上限（TASK_SERVER_MEMORY_CAP_BYTES）定同时起的个数，记录跟着上限涨，起步一次比一次大",
            f"照跑就再起 task-server/task wait {task_id}")]

    def repeated_cap_hits_item(self, record, state, attempts):
        """连着撞上限还在重排（文件头 ①「要处置的提醒」的 repeated-cap-hits）：在排、或下一次在跑，末尾连着 lib.LOOP_LEAK_REMINDER_CAP_HITS 次已收尾的尝试结局都是
        memory-cap-hit 的交一项（每条任务只报一次），不然交空表。在跑而一次在跑、在起的尝试都没有的（刚收尾、settle_attempt 还没定重排还是打回）不判：
        那一刻打回的照结局的 loop_leak= 报，不另报这一条。项另带 loop_leak（lib.loop_leak_check_text），attention_text 与 lib.attention_line 照它另写一段。"""
        if state == "running" and not any(attempt.get("state") in ACTIVE_ATTEMPT_STATES for attempt in attempts):
            return []
        cap_hits = lib.trailing_cap_hits(attempts)
        if len(cap_hits) < lib.LOOP_LEAK_REMINDER_CAP_HITS:
            return []
        task_id = record["id"]
        numbers = f"第 {cap_hits[0].get('number')}–{cap_hits[-1].get('number')} 次"
        item = self.attention_item(
            "repeated-cap-hits", 1, task_id, f"末尾连着 {len(cap_hits)} 次尝试撞了内存上限（{numbers}），服务照乘 1.5 重排、还没到两台最大",
            "要的内存一次比一次多的，多半是代码在循环里越攒越多或把全量整份读进内存，不是范围大；放大上限只是推迟到撞两台最大才打回",
            f"服务照旧乘 1.5 重排，照跑就再起 task-server/task wait {task_id}")
        return [dict(item, loop_leak=lib.loop_leak_check_text(cap_hits))]

    def current_metric(self, attempt):
        """这一次尝试的峰值是不是这一版的口径（task_server_lib.py 文件头「资源记录」的峰值口径一栏）：peak_metric 是「进程内存」的才算（task_server_lib.py 的 current_peak_metric，control key-stats 同用）。
        弄坏开关 old-metric-peaks-read 下照改前不看这一栏，旧行也算。"""
        return lib.current_peak_metric(attempt) or lib.broken("old-metric-peaks-read", self.environment)

    def attention_of_record(self, record, now):
        """一条任务记录（父任务或子任务）此刻要处置的提醒：排队过久、日志长期不涨而服务没停它、卡在 IO 上而服务没停它、
        读不到单元状态、清理没收尾。
        弄坏开关 attention-queue-unreported、attention-quiet-busy-unreported、attention-io-wait-unreported、
        attention-cleanup-unreported、repeated-cap-hits-silent、unit-unreadable-unreminded 各关一种。"""
        task_id, state = record["id"], record.get("state")
        attempts = [attempt for attempt, valid in self.store.attempts_of(task_id) if valid]
        items = []
        if state == "queued" and not lib.broken("attention-queue-unreported", self.environment):
            since = max([float(record.get("submitted_at") or now)] + [float(attempt.get("ended_at") or 0) for attempt in attempts])   # 重排的从上一次尝试结束起算
            waited = now - since
            crossed = reminder_threshold(waited, self.queue_attention_seconds)
            why = self.waiting_text(record)
            # 排队原因还没判出来（服务刚起来、这一轮还没判到它）的这一轮不报，等判出来那一轮再报（文件头 ①「要处置的提醒」）；弄坏开关 attention-queue-reasonless-sent 下照改前照报
            if crossed and (why or lib.broken("attention-queue-reasonless-sent", self.environment)):
                items.append(self.attention_item(
                    "queued-too-long", crossed, task_id,
                    f"已排 {waited:.0f} 秒（{waited / 60:.0f} 分钟），排满 {next_reminder_seconds(crossed, self.queue_attention_seconds):.0f} 秒再提醒一回",
                    why or "排队原因此刻读不到（这一轮还没判到它）",
                    f"等得起再起 task-server/task wait {task_id}", self.queue_attention_seconds * 2 ** (crossed - 1)))
        if state == "running" and not lib.broken("attention-quiet-busy-unreported", self.environment):
            items += self.quiet_log_items(record, attempts)
        if state == "running" and not lib.broken("attention-io-wait-unreported", self.environment):
            items += self.io_wait_items(record, attempts)
        if state == "running" and not lib.broken("unit-unreadable-unreminded", self.environment):
            items += self.unit_unreadable_items(record, attempts)
        # 预算贴着整台（budget_check 记的那一项）。在跑的照报；在排的两条都满足才报：
        # 一是排队那一轮已对它判过「在每一台上都起不来」（idle_short_judged 里有它：note_idle_short_judged 在 note_idle_short 记完、或判出不是之后才记；先读这一项、后读 idle_short_notices，
        # 次序反过来会读到还没记的 idle_short_notices 而先报）：交的时候（confirmation 的 budget_check）就记下的这一项因此不抢在「每台都起不来」之前；
        # 不看 waiting_reasons：排在排满时限的任务后面、这一轮判不了放不放得下、有一台判不成的这几种也给了它排队原因，那时「每台都起不来」还没判；
        # 二是 booking-beyond-idle-machine 没开着：它起不来，「确要这么多就照跑」的出路走不通，两种说的是同一件事（要的就是整台），前者的出路（等整机空下来、不要的撤钉值或收小）
        # 已经包住后者；起得来之后（不再在排或那一项判没了）照报。
        # 弄坏开关 oversized-reported-before-queue-judged 下没判过也报，oversized-under-idle-short-reported 下两种照报
        if state in ("queued", "running") and task_id in self.budget_notices:
            if state == "running":
                reportable = True
            else:
                judged = task_id in self.idle_short_judged or lib.broken("oversized-reported-before-queue-judged", self.environment)
                reportable = judged and (task_id not in self.idle_short_notices or lib.broken("oversized-under-idle-short-reported", self.environment))
            if reportable:
                items.append(dict(self.budget_notices[task_id]))
        if state == "queued" and task_id in self.idle_short_notices:   # 排队那一轮判出的在每一台上都起不来（note_idle_short）
            items.append(dict(self.idle_short_notices[task_id]))
        if state == "running" and not lib.broken("long-run-idle-silent", self.environment):
            items += self.long_run_idle_items(record, attempts, now)
        if state == "running":
            items += self.threads_over_registered_items(record, attempts)
        if state == "running":
            items += self.long_run_unconfirmed_items(record, attempts)
        if state in ("queued", "running") and not lib.broken("fills-any-cap-silent", self.environment):
            items += self.fills_any_cap_items(record)
        if state in ("queued", "running") and not lib.broken("repeated-cap-hits-silent", self.environment):
            items += self.repeated_cap_hits_item(record, state, attempts)
        if not lib.broken("attention-cleanup-unreported", self.environment):
            for attempt in attempts:
                if attempt.get("cleanup_pending") and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                    left_children = PEER_SCOPE_LEFTOVERS_MARK in attempt["cleanup_pending"]   # 第二台停了 scope 还留着进程（confirm_peer_cleanup）
                    items.append(self.attention_item(
                        "cleanup-pending", max(1, int(attempt.get("number") or 1)), task_id,
                        f"第 {attempt.get('number')} 次尝试的命令已退出，清理没收尾：{attempt['cleanup_pending']}",
                        ("命令退出之后它那一层内存包装的 scope 里还有进程，停了 scope 也没清掉；服务保留这一次的资源占用、每隔几秒重查" if left_children
                         else "服务保留这一次的资源占用、每隔几秒重查，查清之前任务不记结局"),
                        ("这条命令退出后还留着子进程（依据里列着第二台 scope 里留下的进程）" if left_children
                         else "留下的也可能是第二台的远端连接：清理回执没收尾的看日志里 multi-host-run.sh ⑥ 那几行")))
        if state == "finished" and not record.get("parent_id") and not lib.broken("workspace-write-interrupted-unreported", self.environment):
            item = self.workspace_write_item(record, attempts)
            if item is not None:
                items.append(item)
        if state == "finished" and not lib.broken("long-run-split-silent", self.environment):
            items += self.long_run_split_items(record, attempts)
        if state == "finished":
            items += self.gpu_cases_not_run_items(record, attempts)
        if state == "finished" and not record.get("parent_id") and not lib.broken("workspace-write-blocked-unreported", self.environment):
            items += self.workspace_write_blocked_items(record, attempts)
        if state == "finished" and not record.get("parent_id") and not lib.broken("previous-swap-running-unreported", self.environment):
            items += self.previous_swap_running_items(record, attempts)
        return items

    def settled_run_seconds(self, attempts):
        """一条出了结局的任务（分片的一片按自己的）服务记的用时（.claude/rules/long-run-split.md「判据」）：收尾了的各次尝试记下的 seconds 相加，
        撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算（撞上限的那几次和更早的不算；同 run_limit_accumulated）。→ (秒, 算进去的几次, 撞过上限的次数)。
        弄坏开关 long-run-split-cap-hits-counted 下撞上限的那几次也累加。"""
        ended = sorted((int(attempt.get("number") or 0), float(attempt.get("seconds") or 0), attempt.get("outcome"))
                       for attempt in attempts if attempt.get("state") == "ended")
        cap_hit_numbers = [number for number, _, outcome in ended if outcome in lib.RECORD_CAP_HIT_OUTCOMES]
        if cap_hit_numbers and not lib.broken("long-run-split-cap-hits-counted", self.environment):
            ended = [entry for entry in ended if entry[0] > max(cap_hit_numbers)]
        return sum(seconds for _, seconds, _ in ended), len(ended), len(cap_hit_numbers)

    def long_run_split_exempt(self, record):
        """这条任务在不在 .claude/long-run-split-exempt（用户点名豁免的项）里：表里每行 `<项名>  # <理由>` 的项名，等于这条命令的某个词（--item 后面的项、--test 后面的测试名、
        `--item=项` 这种带等号的等号后面、逗号分隔的每一段）就算。表读不到当没有。弄坏开关 long-run-split-exempt-ignored 下不看这张表。"""
        if lib.broken("long-run-split-exempt-ignored", self.environment):
            return False
        path = lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_SPLIT_EXEMPT", self.environment) or os.path.join(self.peer_root, LONG_RUN_SPLIT_EXEMPT_FILE)
        try:
            with open(path, encoding="utf-8") as handle:
                names = {line.split("#", 1)[0].strip() for line in handle if line.strip() and not line.lstrip().startswith("#")} - {""}
        except OSError:
            return False
        words = set()
        for word in record.get("command") or []:
            words.add(word)
            for part in re.split(r"[=,]", word):
                words.add(part)
        return bool(names & words)

    def long_run_split_items(self, record, attempts):
        """出结局时用时超过 lib.LONG_RUN_SPLIT_SECONDS 的要拆（文件头 ①「要处置的提醒」的 long-run-split；.claude/rules/long-run-split.md）：任务（分片的一片按自己的，
        分片的父任务没有自己的尝试、不判整件的挂钟）settled_run_seconds 超过界、又不在豁免表（long_run_split_exempt）里的交一项，只推一回。
        依据写这是按片或按整条任务的用时判的（日志里取不出那一项的起止，拿片或整条任务的用时代替）。弄坏开关 long-run-split-silent 下不报。"""
        seconds, counted, cap_hits = self.settled_run_seconds(attempts)
        if seconds <= lib.LONG_RUN_SPLIT_SECONDS or self.long_run_split_exempt(record):
            return []
        what = task_scope_text(record)
        return [self.attention_item(
            "long-run-split", 1, record["id"],
            (f"{what}服务记的用时 {seconds:.0f} 秒（{seconds / 3600:.1f} 小时），超过 {lib.LONG_RUN_SPLIT_SECONDS} 秒；这是按{'片' if record.get('role') == 'shard' else '整条任务'}的用时判的，"
             f"不是日志里某一项的起止（取不出那一项的起止，拿片或整条任务的用时代替）；算了 {counted} 次尝试，撞内存上限重来的 {cap_hits} 次从最后一次撞上限之后那一次起算"),
            "规则 .claude/rules/long-run-split.md「判据」：一项一次运行的用时超过 7200 秒就要拆；撞没撞四小时上限、结局是绿是红、填没填过 --why、确认没确认长跑都不改这条判据",
            "这一件已经跑完、结局照常用；不豁免的由用户点名，登记在 .claude/long-run-split-exempt")]

    def gpu_cases_not_run_items(self, record, attempts):
        """出了结局的任务（分片的一片按自己的，分片的父任务没有自己的尝试；取消的不判）最后那一次尝试的日志里有用例「本次未跑」的标记行
        （文件头 ①「要处置的提醒」的 gpu-cases-not-run；finish_attempt 经 record_gpu_not_run_marks 记的 NOT_RUN_MARKS_COLUMN）：交一项，提醒门槛 1、只推一回，
        依据同结局说明那一段（gpu_not_run_text）。任务的退出码与结局种类不动。"""
        ended = [attempt for attempt in attempts if attempt.get("state") == "ended"]
        if record.get("outcome") == "cancelled" or not ended:
            return []
        last = max(ended, key=lambda attempt: int(attempt.get("number") or 0))
        marks = last.get(NOT_RUN_MARKS_COLUMN)
        if not marks:
            return []
        return [self.attention_item(
            "gpu-cases-not-run", 1, record["id"],
            f"{task_scope_text(record)}第 {last.get('number')} 次尝试（结局 {record.get('outcome')}，退 {record.get('exit_code')}）：{gpu_not_run_text(marks)}",
            (f"日志里带 {GPU_NOT_RUN_MARK} 记号的一处是那条用例的判段没执行、拿占位结局就返回（不带 gpu 特性而配置开了 GPU，或卡、显存借不到），"
             "测试进程照报 ok；结局种类与退出码照命令的，这几条不在「过」里"),
            "结局里别的用例照常用")]

    def workspace_write_blocked_items(self, record, attempts):
        """换钩子被上一趟没写完成的挡住（文件头 ①「要处置的提醒」的 workspace-write-blocked）：命令是 scripts/gate/hook-swap.py（不带 --feed-only；命令那一层与包在里面的都认，
        hook_swap_command）、最后一次尝试结局 command-failed 退 7（hook-swap.py 退出码 7：上一趟换上没写完成，拒绝再换、一个字节没写）的，交一项报给被挡的这个会话，
        依据点名上一趟疑似是哪条任务（库里更早的、改主工作区的换钩子命令里，结局是被杀这类没正常结束的，取最近的一条；换钩子在命令那一层的与包在 bash -c 串、
        命令位置上起的脚本正文里的都认，writer_is_hook_swap）；日志里的拒绝说明逐份点名状态。弄坏开关 workspace-write-blocked-unreported 下不报，
        wrapped-previous-swap-unnamed 下找上一趟只认命令那一层的换钩子。"""
        if record.get("outcome") != "command-failed" or record.get("exit_code") != WORKSPACE_BLOCKED_EXIT_CODE or not self.hook_swap_command(record):
            return []
        earlier = None
        for other, valid in self.store.select("tasks", "state = 'finished' AND id < ?", (record["id"],), order="id DESC LIMIT 100"):
            if (valid and not other.get("parent_id") and self.modifies_workspace(other)
                    and (other.get("outcome") in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (other.get("outcome") == "command-failed" and other.get("exit_code") in SIGNAL_EXIT_CODES))
                    and (self.hook_swap_on_command_level(other)
                         or (not lib.broken("wrapped-previous-swap-unnamed", self.environment) and self.writer_is_hook_swap(other)))):
                earlier = other
                break
        who = (f"上一趟疑似是任务 {earlier['id']}（结局 {earlier.get('outcome')}，退 {earlier.get('exit_code')}）" if earlier is not None
               else "上一趟是哪条任务服务在库里没查到（可能不是经服务起的，或更早）")
        return [self.attention_item(
            "workspace-write-blocked", 1, record["id"],
            f"换钩子的命令退 7：拒绝再换、一个字节没写；{who}",
            "hook-swap.py 起手先读结局文件：上一趟换上没写完成（被杀或回滚不成，几份文件停在半截），之后所有会话的换上都被挡，直到有人补写完成或退回",
            f"这一条的日志（task-server/task log {record['id']}）里拒绝说明逐份点名状态与怎么退回")]

    def hook_swap_command(self, record):
        """这条任务的命令起的是换钩子 scripts/gate/hook-swap.py、又不带 --feed-only（只喂不换）：命令那一层是它的算（hook_swap_on_command_level）；
        那一层没有的，包在 bash -c 串、命令位置上起的脚本正文里有一条是它的也算（wrapped_hook_swap；那一条带 --feed-only 的不算）。
        弄坏开关 wrapped-hook-swap-unread 下只看命令那一层。"""
        if "--feed-only" in (record.get("command") or []):
            return False
        if self.hook_swap_on_command_level(record):
            return True
        return not lib.broken("wrapped-hook-swap-unread", self.environment) and self.wrapped_hook_swap(record)

    def hook_swap_on_command_level(self, record):
        """命令那一层不带 - 的词照任务的工作目录折成相对本仓根的路径，有一个是 scripts/gate/hook-swap.py。"""
        root = os.path.realpath(self.peer_root)
        return HOOK_SWAP_ENTRY in self.repository_paths(record.get("command") or [], os.path.realpath(record.get("cwd") or root))

    def wrapped_hook_swap(self, record):
        """包在里面认到的写工作区工具（wrapped_workspace_commands；命令那一层判出写、只读或只喂的不往里看，同 workspace_writer_commands）里有一条是换钩子、又不带 --feed-only：
        带只读参数（--dry-run 这类）的也算，同命令那一层：hook-swap.py 的 --dry-run 一样先读结局文件、先占草稿目录，退 7、9、10 的成因与真换相同；
        弄坏开关 feed-only-counted-as-writer 下只喂的那一条当写的算。弄坏开关 wrapped-read-only-hook-swap-unread 下只认写的那几条（writer_is_hook_swap），带只读参数的不算。"""
        if lib.broken("wrapped-read-only-hook-swap-unread", self.environment):
            return self.writer_is_hook_swap(record)
        if self.writer_command_verdict(record["command"], self.command_paths_in_repository(record)) is not None:
            return False
        return any(self.names_hook_swap(writer) and (writes or "--feed-only" not in writer.words)
                   for writer, writes in self.wrapped_workspace_commands(record))

    def names_hook_swap(self, writer):
        """认到的这一条写工作区工具（WorkspaceWriter）的词照任务的工作目录折成相对本仓根的路径，有一个是 scripts/gate/hook-swap.py。"""
        return HOOK_SWAP_ENTRY in self.repository_paths(writer.words, os.path.realpath(writer.cwd))

    def writer_is_hook_swap(self, record):
        """这条任务里认到的写工作区工具（workspace_writer_commands：命令那一层或包在里面的，只读、只喂的不算）有一条是换钩子（names_hook_swap）。"""
        return any(self.names_hook_swap(writer) for writer in self.workspace_writer_commands(record))

    def previous_swap_running_items(self, record, attempts):
        """换钩子时上一趟还在跑（文件头 ①「要处置的提醒」的 previous-swap-running）：命令是 scripts/gate/hook-swap.py（不带 --feed-only；命令那一层与包在里面的都认，hook_swap_command）、
        最后一次尝试结局 command-failed 退 9（hook-swap.py 退出码 9：上一趟没写完成而起它的进程还在这一台上，拒绝再换、一个字节没写）或退 10（退出码 10：同一个 --scratch
        或这个仓的真换正被另一趟 hook-swap.py 占着，没建树、没写文件）的，交一项报给被挡的这个会话。退 9 的出路是等那一趟写完成再原样重交（异常表那一行）；
        退 10 的出路跟着成因（swap_busy_cause 读最后一次尝试的日志认）：草稿目录被占的等占着的那一趟结束再交、两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是）
        可以换一个 --scratch（干跑不占这个仓，真换先占这个仓：两趟真换换了草稿目录照样被仓占着），这个仓的换上被占的只有等那一趟结束再交，
        认不出的写两种都可能、叫照日志里的拒绝说明办；日志里的拒绝说明点名那一趟的进程号与起了多少秒。
        弄坏开关 previous-swap-running-unreported 下都不报（attention_of_record），swap-scratch-busy-unreported 下退 10 的不报。"""
        code = record.get("exit_code")
        if record.get("outcome") != "command-failed" or code not in (PREVIOUS_SWAP_RUNNING_EXIT_CODE, SWAP_SCRATCH_BUSY_EXIT_CODE) or not self.hook_swap_command(record):
            return []
        if code == SWAP_SCRATCH_BUSY_EXIT_CODE:
            if lib.broken("swap-scratch-busy-unreported", self.environment):
                return []
            return [self.swap_busy_item(record, self.swap_busy_cause(attempts))]
        return [self.attention_item(
            "previous-swap-running", 1, record["id"],
            "换钩子的命令退 9：上一趟换上还没写完成、起它的进程还在这一台上跑，拒绝再换、一个字节没写",
            "hook-swap.py 起手先读结局文件：上一趟还在换或在验，它写完成之前别的换上都被挡；这一趟没动任何文件，不用退回、不用补写",
            f"那一趟的进程号与起了多少秒在这一条的日志（task-server/task log {record['id']}）里的拒绝说明")]

    def swap_busy_cause(self, attempts):
        """换钩子退 10 占着的是哪一种 → SWAP_BUSY_SCRATCH（同一个 --scratch 被占）、SWAP_BUSY_REPOSITORY（这个仓的真换被占）或 None（认不出）。
        读最后一次尝试的日志（log_tail）：有行里出现 SWAP_BUSY_SCRATCH_LINE 的是草稿目录被占，出现 SWAP_BUSY_REPOSITORY_LINE 的是这个仓的换上被占，两种都出现的取最后出现的；
        没有尝试、日志读不到或两种都没有的是 None。弄坏开关 swap-busy-cause-unread 下不读日志、一律是 SWAP_BUSY_SCRATCH，swap-busy-unknown-guessed 下认不出的也是 SWAP_BUSY_SCRATCH。"""
        if lib.broken("swap-busy-cause-unread", self.environment):
            return SWAP_BUSY_SCRATCH
        cause = None
        if attempts:
            last = max(attempts, key=lambda attempt: int(attempt.get("number") or 0))
            for line in log_tail(last.get("log") or "").splitlines():
                if SWAP_BUSY_REPOSITORY_LINE.search(line):
                    cause = SWAP_BUSY_REPOSITORY
                elif SWAP_BUSY_SCRATCH_LINE.search(line):
                    cause = SWAP_BUSY_SCRATCH
        if cause is None and lib.broken("swap-busy-unknown-guessed", self.environment):
            return SWAP_BUSY_SCRATCH
        return cause

    def swap_busy_item(self, record, cause):
        """换钩子退 10 的那一项 previous-swap-running：依据、原因、出路都跟着 swap_busy_cause 认出的成因写；出路只写对这一种成立的；认不出的不给换草稿目录这条出路（两种里只有一种有用），
        叫照日志里的拒绝说明办。异常表那一行的出路写退 10 的共同部分（等占着的那一趟结束再交），后面接这里的 follow。"""
        task_id = record["id"]
        if cause == SWAP_BUSY_SCRATCH:
            return self.attention_item(
                "previous-swap-running", 1, task_id,
                "换钩子的命令退 10：同一个 --scratch 正被另一趟 hook-swap.py 占着（日志里有以「✗ 草稿目录」起头的拒绝说明），拒绝再换、没建树、没写文件",
                "hook-swap.py 起手先占草稿目录：同一个 --scratch 的草稿树只有一棵，并行起会互相删掉；这一趟没动任何文件，不用退回、不用补写",
                f"这一种等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch"
                f"（干跑不占这个仓、真换先占这个仓；要并行的几条各给一个草稿目录；两趟真换换了草稿目录照样被这个仓的换上挡住）；"
                f"占着的那一趟的进程号与开跑时间在这一条的日志（task-server/task log {task_id}）里的拒绝说明")
        if cause == SWAP_BUSY_REPOSITORY:
            return self.attention_item(
                "previous-swap-running", 1, task_id,
                "换钩子的命令退 10：这个仓的换上正被另一趟 hook-swap.py 占着（日志里有以「✗ 这个仓的换上」起头的拒绝说明），拒绝再换、没读结局文件、没建树、没写文件",
                "真换在同一个仓里一次只许一趟：结局文件按仓只有一份，换草稿目录也占不到；这一趟没动任何文件，不用退回、不用补写",
                f"这一种只有等那一趟结束再交，换草稿目录没有用；占着的那一趟的进程号、用的草稿目录与开跑时间在这一条的日志（task-server/task log {task_id}）里的拒绝说明")
        return self.attention_item(
            "previous-swap-running", 1, task_id,
            "换钩子的命令退 10：同一个 --scratch 被占，还是这个仓的换上被占，服务没从日志里认出来（读不到日志，或日志里没有以「✗ 草稿目录」或「✗ 这个仓的换上」起头的拒绝说明），两种都可能；拒绝再换、没写文件",
            "hook-swap.py 退 10 有两种成因：同一个 --scratch 的草稿目录被占，或这个仓的真换被占；这一趟没动任何文件，不用退回、不用补写",
            f"占着的是哪一种、能不能换草稿目录，看这一条的日志（task-server/task log {task_id}）里的拒绝说明与它的「→ 怎么办」那一行，照它办")

    def workspace_write_item(self, record, attempts):
        """改主工作区的命令（modifies_workspace）没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一、或命令被信号杀（command-failed 而退出码在 SIGNAL_EXIT_CODES 里）时，
        可能写了一半，交一项 workspace-write-interrupted（点名要核的文件或补丁目录）；正常结束的、不改工作区的交 None。用户：「这个问题很严重 因为可能会导致补丁没有正常写入」。
        apply-writer-patch.py 有结局文件（补丁目录里的 apply-outcome.tsv）的照它判：起 --read-outcome，打全了、已退回的不报，没打完、打完之后又被改了、读不了的报（连同它打的那几行）；
        结局文件不在的不报（它在动手换之前写开始与文件行，不在就是还没开始换）；补丁目录从命令里算不出的照可能写了一半报；带 --dry-run 这类只读参数的不报。
        hook-swap.py 同样起它的 --read-outcome（hook_swap_read_back）：换完了、已退回的不报，没换完而结局文件归同一条命令里另一条换钩子的不报，别的报。
        一条命令里认到几个写工具的逐个判，有一个没打全就报（workspace_write_judged）。结了局的任务只核一次（workspace_write_checks）。
        弄坏开关 apply-outcome-ignored 下不读结局文件，一律照可能写了一半报。"""
        task_id = record["id"]
        if task_id in self.workspace_write_checks:
            return self.workspace_write_checks[task_id]
        outcome, exit_code = record.get("outcome"), record.get("exit_code")
        interrupted = outcome in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (outcome == "command-failed" and exit_code in SIGNAL_EXIT_CODES)
        if outcome == lib.RUN_HARD_LIMIT_OUTCOME and (not attempts or lib.broken("run-hard-limit-writer-unreminded", self.environment)):
            interrupted = False   # 自己一次尝试都没起的（分片的父任务照片收尾），主工作区没动（文件头「四小时硬上限」）
        item = None
        if interrupted and self.modifies_workspace(record):
            item = self.workspace_write_judged(record, attempts, outcome, exit_code)
        self.workspace_write_checks[task_id] = item
        return item

    def workspace_write_judged(self, record, attempts, outcome, exit_code):
        """workspace_write_item 判出没正常结束的改主工作区命令：认到的每一条写工作区工具（workspace_writer_commands；包在 bash -c 串、命令位置上起的脚本正文里的取包着的那几条，
        不取 bash 与 -c 的整串）逐条判（writer_interruption），有一条可能写了一半就报，写工具、依据与要核的路径把可能写了一半的那几条都写上；
        要读结局文件的那几条（PatchOutcomeReadBack）在这里起 --read-outcome（打补丁的 apply-writer-patch.py、换钩子的 hook-swap.py；同一条读法只起一次），
        退 0、1 的不报，别的报、依据带它打的末几行。换钩子退 3 时，输出里点名的反向规格是这条任务里另一条换钩子的（owner_marks 认得出、又不是这一条的），这一条不报：
        结局文件按仓只有一份、记的是最后写开始行的那一趟，真换起手先读它，上一趟停在半截就拒绝再换、不写开始行，所以结局文件归另一条的，这一条要么换完了、要么还没动手。
        认不到的（弄坏开关 merge-and-verify-counted-as-writer 下的合入后验证）照命令那一层的词（command_workspace_writer）。
        → 一项 workspace-write-interrupted，或 None（每一条都打全了、换完了、已退回、还没开始换或是只读的用法）。弄坏开关 wrapped-writer-words-unused 下一律照命令那一层的词，
        swap-outcome-owner-ignored 下换钩子退 3 不看反向规格是谁的、一律报。"""
        task_id, cwd = record["id"], record.get("cwd") or self.peer_root
        found = [] if lib.broken("wrapped-writer-words-unused", self.environment) else self.workspace_writer_commands(record)
        number = max([int(attempt.get("number") or 0) for attempt in attempts] or [0])
        basis = f"第 {number} 次尝试结局 {outcome}（退 {exit_code}）" + ("，命令被信号杀" if outcome == "command-failed" else "")
        status_hint = f"git -C {shlex.quote(cwd)} status --short 与 git -C {shlex.quote(cwd)} diff --stat 看主工作区此刻改了哪几份"
        ones = [self.writer_interruption(writer) for writer in found or [self.command_workspace_writer(record)]]
        task_marks = [mark for one in ones if isinstance(one, PatchOutcomeReadBack) for mark in one.owner_marks]
        owner_ignored = lib.broken("swap-outcome-owner-ignored", self.environment)
        judged, readings = [], {}
        for one in ones:
            if isinstance(one, PatchOutcomeReadBack):
                read_back = one.read_back
                reading_key = (tuple(read_back), one.directory)
                if reading_key not in readings:
                    try:
                        completed = subprocess.run(read_back, cwd=one.directory, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                                   timeout=PATCH_WRITER_CHECK_SECONDS)
                        readings[reading_key] = (completed.returncode, (completed.stdout + completed.stderr).strip().splitlines())
                    except (OSError, subprocess.TimeoutExpired) as error:
                        readings[reading_key] = (2, [f"--read-outcome 起不来或没在 {PATCH_WRITER_CHECK_SECONDS} 秒里退：{type(error).__name__}: {error}"])
                verdict, lines = readings[reading_key]
                if verdict in (0, 1):
                    continue   # 打全了、换完了、已退回：主工作区不在半截上
                text = "\n".join(lines)
                if (verdict == HOOK_SWAP_UNFINISHED_VERDICT and one.owner_marks and not owner_ignored
                        and any(mark in text for mark in task_marks) and not any(mark in text for mark in one.owner_marks)):
                    continue   # 结局文件是这条任务里另一条换钩子的：这一条换完了或还没动手
                one = one.unread._replace(basis=f"；{one.label} 判{one.verdicts.get(verdict, f'退 {verdict}')}："
                                                + " / ".join(line.strip() for line in lines[-6:]))
            if one is not None:
                judged.append(one)
        if not judged:
            return None
        writers = "、".join(dict.fromkeys(one.writer for one in judged))
        ending = status_hint + (f"；task-server/task log {task_id} 看它停在哪一步" if any(one.log_pointed for one in judged) else "")
        return self.attention_item("workspace-write-interrupted", 1, task_id, basis + "".join(one.basis for one in judged),
                                   f"这条命令改主工作区（{writers}），没正常结束，可能写了一半", "；".join(one.check for one in judged) + "；" + ending)

    def writer_interruption(self, writer):
        """workspace_write_judged 的一条写工作区工具（WorkspaceWriter）没正常结束 → InterruptedWriter、PatchOutcomeReadBack（要读结局文件），或 None（不在半截上）。
        认写工具照任务的工作目录（writer.cwd）。
        apply-writer-patch.py：带 PATCH_WRITER_READ_ONLY_ARGUMENTS 之一的交 None；不带补丁目录的（--finish-pending、--rollback-pending）叫看 --pending-status；
        补丁目录（与 --root 的值）取代入之后的词（writer.expanded），相对的照这一条起时的当前目录（writer.directory）拼：有一样算不出的照可能写了一半报、不读结局文件；
        算得出的读结局文件：不在交 None（在动手换之前写，不在就是还没开始换），在的交 PatchOutcomeReadBack（--read-outcome 由 workspace_write_judged 起）。
        hook-swap.py 照点名它后面的路径报，--root、--scratch 的值算得出的交 PatchOutcomeReadBack（hook_swap_read_back）。
        别的写工具照点名它后面的路径报。点名的路径代入得出的写代入之后的样子，代入不了的写原词。弄坏开关 apply-outcome-ignored 下不读结局文件、在不在都报。"""
        root = os.path.realpath(self.peer_root)
        command, cwd = list(writer.words), writer.cwd
        positions = [index for index, word in enumerate(command) if not word.startswith("-")
                     and os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), word)), root) in WORKSPACE_WRITERS]
        position = positions[0] if positions else 0
        tool = command[position] if positions else (command[0] if command else "")
        after, after_expanded = command[position + 1:], list(writer.expanded)[position + 1:]
        chosen = [index for index, word in enumerate(after) if not word.startswith("-") and (index == 0 or after[index - 1] != "--root")]
        named = [after[index] for index in chosen]
        shown = [after_expanded[index] if after_expanded[index] is not None else after[index] for index in chosen]
        relative_tool = os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), tool)), root)
        if relative_tool != WRITER_PATCH_ENTRY:
            named_paths = InterruptedWriter(tool, "", f"要核的是{('这几处：' + '、'.join(shown)) if shown else '它改的文件'}", True)
            if relative_tool == HOOK_SWAP_ENTRY:
                return self.hook_swap_read_back(writer, tool, after, after_expanded, named_paths)
            return named_paths
        quoted = shlex.quote(tool)
        if any(word in PATCH_WRITER_READ_ONLY_ARGUMENTS for word in after):
            return None   # 只读的用法（--dry-run、--pending-status 这类）不写主工作区
        if not named:   # --finish-pending、--rollback-pending：照记下的暂存换，看暂存还在不在
            return InterruptedWriter(tool, "", f"python3 {quoted} --pending-status 看有没有没换完的，照它打的出路接着换完或退回", False)
        directory, patch = writer.directory, after_expanded[chosen[0]]
        root_value = after_expanded[after.index("--root") + 1] if "--root" in after[:-1] else ""
        if (patch is None or root_value is None
                or (directory is None and not (os.path.isabs(patch) and (not root_value or os.path.isabs(root_value))))):
            return InterruptedWriter(
                tool, f"；补丁目录 {named[0]} 从这条命令里算不出（变量、位置参数、命令替换或 cd 到算不出的目录），没读结局文件",
                f"照这条命令的日志认出补丁目录 {named[0]} 的真路径，python3 {quoted} --read-outcome <补丁目录>/{PATCH_WRITER_OUTCOME_NAME} 逐个文件看已换上与没换上，"
                f"再照 python3 {quoted} --pending-status 的出路接着换完或退回", True)
        outcome_file = os.path.join(directory or "", patch, PATCH_WRITER_OUTCOME_NAME)
        if not os.path.exists(outcome_file) and not lib.broken("apply-outcome-ignored", self.environment):
            return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动
        unread = InterruptedWriter(tool, "", f"python3 {quoted} --read-outcome {shlex.quote(outcome_file)} 逐个文件看已换上与没换上，"
                                             f"再照 python3 {quoted} --pending-status 的出路接着换完或退回；补丁目录 {shown[0]}", False)
        if lib.broken("apply-outcome-ignored", self.environment):
            return unread
        read_back = [sys.executable, os.path.join(cwd, tool) if not os.path.isabs(tool) else tool, "--read-outcome", outcome_file]
        if "--root" in after[:-1]:
            read_back += ["--root", root_value]
        return PatchOutcomeReadBack(read_back, directory or cwd, f"结局文件 {outcome_file}", PATCH_WRITER_VERDICTS, (), unread)

    def hook_swap_read_back(self, writer, tool, after, after_expanded, named_paths):
        """writer_interruption 的一条 hook-swap.py（tool 是它那个词，after 与 after_expanded 是它后面的词与代入之后的样子）→ PatchOutcomeReadBack，或 named_paths（照点名的路径报）。
        读法：hook-swap.py --read-outcome，带上这一条自己的 --root 与 --scratch（结局文件在 --root 的 git-dir 下，仓根不是 git 工作树的根时在 --scratch 下，hook-swap.py 文件头「结局文件」），
        在这一条起时的当前目录（writer.directory）里起；脚本路径照任务的工作目录拼，同打补丁的那一条。--root、--scratch 的值代入不了，或是相对的而当前目录算不出，交 named_paths。
        owner_marks 是 --scratch 下反向规格路径的开头（HOOK_SWAP_REVERSE_SPEC_PARTS）；不带 --scratch 的是空的（退 3 照报）。
        弄坏开关 swap-outcome-unread 下不读，一律交 named_paths。"""
        if lib.broken("swap-outcome-unread", self.environment):
            return named_paths
        values = {}
        for option in ("--root", "--scratch"):
            if option in after[:-1]:
                value = after_expanded[after.index(option) + 1]
                if value is None or (writer.directory is None and not os.path.isabs(value)):
                    return named_paths
                values[option] = value
        directory = writer.directory or writer.cwd
        read_back = [sys.executable, os.path.join(writer.cwd, tool) if not os.path.isabs(tool) else tool, "--read-outcome"]
        for option, value in values.items():
            read_back += [option, value]
        marks = ()
        if "--scratch" in values:
            scratch = os.path.join(directory, values["--scratch"])
            marks = tuple(dict.fromkeys(os.path.join(form, *HOOK_SWAP_REVERSE_SPEC_PARTS) for form in (os.path.abspath(scratch), os.path.realpath(scratch))))
        label = "结局文件（" + " ".join([tool] + read_back[2:]) + "）"
        return PatchOutcomeReadBack(read_back, directory, label, HOOK_SWAP_VERDICTS, marks, named_paths)

    def stored_reminded_thresholds(self, task_id):
        """库里这一条任务的行记着的报到的提醒门槛（正文的 reminded_thresholds，{种类: 报到的最高门槛}；文件头 ①「要处置的提醒」）：行不在、校验不过、写法不对的那几项交空。
        弄坏开关 attention-reported-memory-only 下不读库、交空（改前的样子）。"""
        if lib.broken("attention-reported-memory-only", self.environment):
            return {}
        found = self.store.task(task_id)
        stored = found[0].get("reminded_thresholds") if found is not None and found[1] else None
        if not isinstance(stored, dict):
            return {}
        return {kind: threshold for kind, threshold in stored.items() if isinstance(kind, str) and isinstance(threshold, int) and threshold > 0}

    def reminder_history_of(self, task_id, kind):
        """这条任务这一种提醒报过的那几回 [{threshold, repush, at, basis, round}…]（旧的在前）：内存里有的照内存，没有的读库里任务行的 reminder_history（服务重启之后）；都没有交 []。"""
        with self.attention_lock:
            remembered = list(self.reminder_histories.get((task_id, kind)) or [])
        if remembered or lib.broken("attention-reported-memory-only", self.environment):
            return remembered
        found = self.store.task(task_id)
        stored = found[0].get("reminder_history") if found is not None and found[1] else None
        entries = stored.get(kind) if isinstance(stored, dict) else None
        return [entry for entry in entries if isinstance(entry, dict)] if isinstance(entries, list) else []

    def attention_round(self, item):
        """这一项（attention_items 的项）这一次推出去是第几回、上一回是什么时刻 → (回数, 时刻或 None)：这条任务这一种提醒报过的最后一回（reminder_history_of）的回数加一，没报过的是第 1 回。"""
        history = self.reminder_history_of(item["task"], item["kind"])
        if not history:
            return 1, None
        return int(history[-1].get("round") or len(history)) + 1, history[-1].get("at")

    def attention_repush(self, item):
        """提醒门槛不随时长涨的提醒（ANOMALY_REPUSHED_KINDS）同一道门槛报过之后还在（文件头 ⑥「没处置时」）：这道门槛头一回报出去满 anomaly_repush_seconds 再推第 1 回、
        之后每翻一倍再推（reminder_threshold），→ 此刻该推第几回（不该推交 0）；报过的那几回照 reminder_history_of。弄坏开关 anomaly-not-repushed 下交 0。"""
        if lib.broken("anomaly-not-repushed", self.environment):
            return 0
        history = [entry for entry in self.reminder_history_of(item["task"], item["kind"]) if entry.get("threshold") == item["threshold"]]
        if not history:
            return 0
        first = min(float(entry.get("at") or 0) for entry in history)
        due = reminder_threshold(time.time() - first, self.anomaly_repush_seconds)
        done = max(int(entry.get("repush") or 0) for entry in history)
        return due if due > done else 0

    def anomaly_acknowledged(self, item):
        """这一项（attention_items 的项）那条任务行里记了照跑（anomaly_acknowledged 有这一种；confirm --why 记的）。弄坏开关 anomaly-acknowledgement-ignored 下一律当没记。"""
        if item["kind"] not in ACKNOWLEDGED_KINDS or lib.broken("anomaly-acknowledgement-ignored", self.environment):
            return False
        found = self.store.task(item["task"])
        return bool(found and found[1] and item["kind"] in (found[0].get("anomaly_acknowledged") or {}))

    def acknowledge_anomalies(self, task_id, session, process_id, stated):
        """confirm --why 时这条任务（分片的连同子任务）此刻开着的、出路是照跑的提醒（ACKNOWLEDGED_KINDS）逐条记进那条任务行的 anomaly_acknowledged，→ 记了的种类（排过序）。"""
        kinds = set()
        # 开着的提醒，连同库里记着报过的（budget_notices 只在内存：服务换上之后在跑的那一条就没有开着的 oversized-budget，报过的照样认）
        reported = [] if lib.broken("anomaly-acknowledge-open-only", self.environment) else [
            {"task": task_id, "kind": kind} for kind in self.stored_reminded_thresholds(task_id) if kind in ACKNOWLEDGED_KINDS]
        for item in self.attention_items(task_id) + reported:
            if item["kind"] not in ACKNOWLEDGED_KINDS:
                continue
            with self.store.task_lock(item["task"]):
                found = self.store.task(item["task"])
                if not (found and found[1]):
                    continue
                marks = dict(found[0].get("anomaly_acknowledged") or {})
                marks.setdefault(item["kind"], {"why": stated, "at": round(time.time(), 3)})
                self.store.change("tasks", item["task"], {"anomaly_acknowledged": marks})
            kinds.add(item["kind"])
        if kinds:
            self.operation("anomaly-acknowledge", task_id, session, process_id, f"{'、'.join(sorted(kinds))}：{stated[:200]}")
        return sorted(kinds)

    def reminded_threshold_of(self, item):
        """这一项（attention_items 的项）报给提交它的会话的最高提醒门槛：内存里那一份与库里那一份取大的，读到库里的并进内存（服务重启之后头一次读库）。"""
        key = (item["task"], item["kind"])
        with self.attention_lock:
            remembered = self.reminded_thresholds.get(key, 0)
        if remembered >= item["threshold"]:
            return remembered
        stored = self.stored_reminded_thresholds(item["task"]).get(item["kind"], 0)
        with self.attention_lock:
            self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), stored)
            return self.reminded_thresholds[key]

    def note_attention_reported(self, items):
        """这几项报给了提交它的会话：记进内存，再逐条任务记进库里那一行的 reminded_thresholds 与 reminder_history（Store.change 只并进这两栏、门槛只升不降；文件头 ①「要处置的提醒」）。
        库写不进（行不在、校验不过、库出错）的说一句、只记内存。弄坏开关 attention-reported-memory-only 下只记内存（改前的样子）。"""
        reached_by_task, logged = {}, {}
        now = round(time.time(), 3)
        rounds = {(item["task"], item["kind"]): self.attention_round(item)[0] for item in items}
        with self.attention_lock:
            for item in items:
                key = (item["task"], item["kind"])
                self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), item["threshold"])
                reached_by_task.setdefault(item["task"], {})[item["kind"]] = self.reminded_thresholds[key]
                # 这一回报了什么（reminder_history：再推的正文照它写前几次推了什么，attention_repush 照它算再推到第几回）
                entry = {"threshold": item["threshold"], "repush": int(item.get("repush") or 0), "at": now, "basis": str(item.get("basis") or "")[:200], "round": rounds[key]}
                self.reminder_histories[key] = (self.reminder_histories.get(key, []) + [entry])[-ATTENTION_HISTORY_KEPT:]
                logged.setdefault(item["task"], {}).setdefault(item["kind"], []).append(entry)
        if lib.broken("attention-reported-memory-only", self.environment):
            return
        for task_id, reached in reached_by_task.items():
            def merged(record, reached=reached, entries=logged.get(task_id, {})):
                stored = record.get("reminded_thresholds") if isinstance(record.get("reminded_thresholds"), dict) else {}
                history = record.get("reminder_history") if isinstance(record.get("reminder_history"), dict) else {}
                return {"reminded_thresholds": dict(stored, **{kind: max(stored[kind] if isinstance(stored.get(kind), int) else 0, threshold) for kind, threshold in reached.items()}),
                        "reminder_history": dict(history, **{kind: (list(history.get(kind) or []) + added)[-ATTENTION_HISTORY_KEPT:] for kind, added in entries.items()})}
            try:
                self.store.change("tasks", task_id, merged)
            except (StoreRefused, sqlite3.Error) as error:
                say(f"  ! 任务 {task_id} 报到的提醒门槛没记进库（{error}），只记在内存里：服务重启之后这几道门槛会再报一次")

    def unreported_attention(self, task_id, session):
        """wait 那一侧调：这条任务此刻要处置的提醒里还没报给提交它的会话、或又跨过一道提醒门槛的，交这些项（attention_items 的项）；没有交空表。
        报给提交它的会话就记下这一种报到了这道门槛（内存里与库里各一份，note_attention_reported；服务重启后照库认、不再报）；别的会话来等照样看得到、不记。
        内存里那一份的判与记在同一把锁里：两个 wait 一起来只报一次。弄坏开关 attention-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return []
        owner = found[0].get("session") == session
        reported, noted = [], []
        for item in self.attention_items(task_id):
            key = (item["task"], item["kind"])
            self.reminded_threshold_of(item)   # 先把库里那一份并进内存
            with self.attention_lock:
                if self.reminded_thresholds.get(key, 0) >= item["threshold"] and not lib.broken("attention-reported-every-wait", self.environment):
                    continue
                if owner:
                    self.reminded_thresholds[key] = max(self.reminded_thresholds.get(key, 0), item["threshold"])
                    noted.append(item)
            reported.append(item)
        if noted:
            self.note_attention_reported(noted)
        return reported

    # ── ⑥ 发消息给会话（文件头 ⑥）──
    def inbox_loop(self):
        """发消息那一路：每 inbox_round_seconds 一轮（notify_finished 当场叫醒）：先发记下了结局的，再扫在排、在跑的任务的疑似卡住与要处置的提醒，最后盯服务健康。
        一轮出事不停服务：打出来、下一轮再判。"""
        while not self.stopping.is_set():
            self.inbox_event.clear()
            with self.inbox_lock:
                self.inbox_sockets.clear()   # 收件 socket 每一轮照会话登记现找（inbox_socket_of）：会话重启换了 socket 的下一轮就照新的发
            try:
                self.push_outcomes()
                self.repush_unclaimed_outcomes()
                self.push_reminders()
                self.watch_health()
            except Exception:
                say("  ! 这一轮发消息出了异常（下一轮再判）：\n" + traceback.format_exc())
            self.inbox_event.wait(self.inbox_round_seconds)
