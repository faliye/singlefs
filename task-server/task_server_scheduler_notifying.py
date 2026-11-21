"""task_server.py 拆出的一份：Scheduler 的一部分方法：给会话发消息、失控进程、记账审计、服务健康、读尝试的日志与内存与进程读数、尝试收尾（Scheduler 在 task_server.py，经继承取到它们）。"""
import os
import sys
import task_server_lib as lib
import contextlib
import json
import sqlite3
import subprocess
import threading
import time
import traceback
from task_server_constants import (
    ACTIVE_ATTEMPT_STATES, ANOMALY_REPUSHED_KINDS, BOOKED_FROM_CAP, BOOKED_WHILE_YOUNG, BOOKED_WITHOUT_SCOPE, BOOKING_DRIFT_BYTES,
    BOOKING_DRIFT_PUSH_SECONDS, BOOKING_DRIFT_RATIO, BOOKING_DRIFT_ROUNDS, BOOKING_DRIFT_ROWS_SHOWN, CLEANUP_RETRY_SECONDS, CPU_GATE_THREADS_COLUMN,
    CPU_MEASURED_AFTER_SECONDS, EXTERNAL_KILL_LOG_AFTER_SECONDS, EXTERNAL_KILL_LOG_SECONDS,
    GPU_CARDS_FAILED_READ_SECONDS, GPU_CARD_FAILED_NEXT_STEP, GPU_CARD_FAILURE_CHECK_SECONDS, GPU_HEALTH_KIND_PREFIXES, GPU_LINES_UNPARSED_KIND,
    GPU_LINES_UNPARSED_NEXT_STEP, GPU_NOT_RUN_BREAK_SWITCHES, GPU_NOT_RUN_TESTS_SHOWN, HEALTH_NEXT_STEP, INBOX_HEADLINE, LEFTOVER_STOP_SECONDS,
    LOCAL, LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, LOG_SCAN_BYTES, NOT_RUN_MARKS_COLUMN, PEAK_SEGMENT_CORES_COLUMN, PEER, PEER_CLEANUP_LOCKS,
    PEER_CLEANUP_LOCKS_GUARD, PEER_CPU_LINE, PEER_EXTERNAL_KILL_LINE, PEER_IO_WAIT_LINE, PEER_MEMORY_LINE, PEER_MEMORY_STAT_LINE,
    PEER_SCOPE_LEFTOVERS_MARK, PEER_SCOPE_LEFTOVERS_SHOWN, PEER_THREADS_LINE, PROCESS_LIST_SECONDS, PROGRESS_PUSH_LISTED_ATTEMPTS,
    PROGRESS_PUSH_TAIL_CHARACTERS, RECORD_WINDOW, RUNAWAY_MINIMUM_CPU_SECONDS, RUNAWAY_MINIMUM_FRACTION, RUNAWAY_REPORTS_BEFORE_STOP,
    RUNAWAY_STOP_GRACE_SECONDS, RUNNING_BOOKED_SUM_TEXT, SIGNAL_EXIT_CODES, UNIT_UNREADABLE_HEALTH_SHOWN, UNIT_UNREADABLE_HEALTH_TEXT,
    WORKSPACE_WRITER_NO_RETRY_OUTCOMES, append_log_lines, gpu_block_not_judgeable_next_step, gpu_not_run_test, gpu_not_run_text, headline_class, keyed_lock, log_tail, outcome_headline,
    own_unit_state, post_to_inbox, read_json_file, reminder_headline, say, suspect_headline,
)
from task_server_store import StoreRefused
from task_server_machines import (
    ThreadReading, attention_event, cgroup_processes, descendant_processes, gibibytes_text, process_run_state, reminder_threshold, signal_name,
    task_id_ranges, thread_reading_of, unit_processes,
)


class SchedulerNotifying:
    """Scheduler 的一部分方法：给会话发消息、失控进程、记账审计、服务健康、读尝试的日志与内存与进程读数、尝试收尾（Scheduler 在 task_server.py，经继承取到它们）。"""

    def task_log_path(self, task):
        """这条任务最后一次尝试的日志（分片的父任务自己没有尝试，取子任务里最后起的那一次）；一次都没起交空串。"""
        attempts = [attempt for attempt, valid in self.store.attempts_of(task["id"]) if valid]
        if not attempts and task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                attempts = [attempt for child in self.children_of(task["id"]) for attempt, valid in self.store.attempts_of(child["id"]) if valid]
        return max(attempts, key=lambda attempt: float(attempt.get("started_at") or 0)).get("log", "") if attempts else ""

    def inbox_socket_of(self, task):
        """这条任务此刻该往哪个收件 socket 发（文件头 ⑥「收件地址」）→ (路径, 找不到时的那一句)：按任务记的会话号读会话登记（lib.session_inbox_socket：
        会话号相同、进程还活着的那一份的 messagingSocketPath），一轮里同一个会话只读一次（inbox_sockets，inbox_loop 每一轮开头清掉）；令牌照交任务时带的。
        弄坏开关 inbox-socket-from-submit-only 下照交任务时记的那一个 socket（会话重启换了 socket 就发不到）。"""
        if lib.broken("inbox-socket-from-submit-only", self.environment):
            submitted = (task.get("inbox") or {}).get("socket", "")
            return submitted, ("" if submitted else "交任务时没记收件 socket")
        session = task.get("session") or ""
        with self.inbox_lock:
            known = self.inbox_sockets.get(session)
        if known is None:
            known = lib.session_inbox_socket(session, self.environment)
            with self.inbox_lock:
                self.inbox_sockets[session] = known
        return known

    def inbox_reachable(self, task):
        """这条任务记着会话令牌、此刻照会话号找得到收件 socket：提醒、疑似卡住与服务健康只发给这样的（找不到的不算一回发失败，wait 照取；结局另照 push_outcomes 发、发不出写告警）。"""
        return bool((task.get("inbox") or {}).get("token")) and bool(self.inbox_socket_of(task)[0])

    def send_to_session(self, task, text, event):
        """往提交 task 的会话的收件 socket（inbox_socket_of 照会话号现找的，令牌是任务记着的）发 text → 发出去了没有。发不出去（找不到活着的会话、socket 不在、连不上、
        会话已关）写一行告警：服务日志、告警记录与这条任务最后一次尝试的日志里 lib.INBOX_LOG_MARK 起头的一行；event 记进 inbox_failed，不重试到死；任务照旧。
        弄坏开关 inbox-not-sent 下一条都不发、交 False。"""
        if lib.broken("inbox-not-sent", self.environment):
            return False
        inbox = task.get("inbox") or {}
        headline = text.splitlines()[0]
        socket_path, missing = self.inbox_socket_of(task)
        problem = missing or post_to_inbox(socket_path, inbox.get("token", ""), text, self.environment)
        session = task.get("session")
        if not problem:
            say(f"  · 发给会话 {session}：{headline}")
            return True
        if event is not None:
            with self.inbox_lock:
                self.inbox_failed.add(event)
        line = f"往会话 {session} 的收件 socket {socket_path or '（照会话号没找到）'} 发「{headline}」没发出去（{problem}）；不再重试，任务照旧"
        next_step = f"会话已关的不用管；会话还在的用 task-server/task wait {task['id']} 取结局、task-server/task status {task['id']} 看在等什么"
        append_log_lines(self.task_log_path(task), [f"{lib.INBOX_LOG_MARK} ✗ {line}；→ 怎么办：{next_step}"])
        try:
            self.alert("inbox-unreachable", task["id"], line, next_step)
        except Exception:   # 库写不进（服务健康里的那一样）时告警记录写不进：服务日志照打
            say(f"  ✗ 告警（inbox-unreachable）任务 {task['id']}：{line}")
            say(f"     → 怎么办：{next_step}")
        return False

    def push_outcomes(self):
        """记下了结局的（inbox_pending）：发给提交它的会话，先发还没报的提醒（改主工作区的没正常结束这类，push_attention），再发结局；
        结局发出去了记成已取（同 wait 把结局交给提交它的会话）。结局只发一次（处理完的任务号不再放回：发出去记了已取的、发不出去写了告警的，同文件头 ⑥ 不重试）；
        会话已经用 wait 取了的不发；子任务不发（父任务的结局发）。处理某一条时抛错（读库、summary、push_attention、记已取），那一条与排在它后面还没处理的号放回
        inbox_pending，异常照旧交给 inbox_loop，下一轮接着发（unsent 记还没处理完的）。
        弄坏开关 pushed-outcome-not-collected 下发出去了也不记已取；inbox-pending-dropped-on-error 下抛错时照改前不放回。"""
        with self.inbox_lock:
            pending, self.inbox_pending = self.inbox_pending, set()
        unsent = sorted(pending)
        try:
            while unsent:
                task_id = unsent[0]
                found = self.store.task(task_id)
                task = found[0] if found is not None and found[1] else None
                if (task is not None and not task.get("parent_id") and (task.get("inbox") or {}).get("token")
                        and task.get("state") == "finished" and not task.get("collected")):
                    self.push_attention(task, final=True)
                    if (self.send_to_session(task, self.outcome_text(self.summary(task_id)), ("outcome", task_id))
                            and not lib.broken("pushed-outcome-not-collected", self.environment)):
                        self.mark_collected(task_id)
                unsent.pop(0)
        finally:
            if unsent and not lib.broken("inbox-pending-dropped-on-error", self.environment):
                with self.inbox_lock:
                    self.inbox_pending.update(unsent)

    def repush_unclaimed_outcomes(self):
        """结局没人取（文件头 ⑥「结局没人取」；异常表 unclaimed-outcome；首行 outcome_headline 写第 N 回与上一回在哪一天，上一回的时刻记在任务行的 outcome_repushed_at）：父任务记着会话令牌、结局满 unclaimed_warning_seconds 还没交给提交它的会话（collected 为假：
        发结局的那一条没送到，也没起 wait 取）的再推一回，之后每翻一倍再推（门槛照 reminder_threshold）；正文写这条结局还没人取、它是什么（outcome_text 整段接在后面）。
        推出去了记成已取（同 push_outcomes）；推到第几回记进任务行的 outcome_repushed（内存里另留一份 unclaimed_repushed），服务重启后不重推同一回。
        收件 socket 照会话号现找（send_to_session），会话重启换了 socket 的这一回照新的推。弄坏开关 unclaimed-outcome-not-repushed 下不再推。"""
        if lib.broken("unclaimed-outcome-not-repushed", self.environment):
            return
        now = time.time()
        for task, valid in self.store.select("tasks", "state = 'finished' AND json_extract(record, '$.collected') = 0"):
            if not valid or task.get("parent_id") or task.get("collected") or not (task.get("inbox") or {}).get("token"):
                continue
            waited = now - float(task.get("finished_at") or now)
            crossed = reminder_threshold(waited, self.unclaimed_warning_seconds)
            done = max(self.unclaimed_repushed.get(task["id"], 0), int(task.get("outcome_repushed") or 0))
            if crossed <= done:
                continue
            self.unclaimed_repushed[task["id"]] = crossed
            previous_at = task.get("outcome_repushed_at") or task.get("finished_at")   # 上一回推出去的时刻：头一回再推的上一回是结局那一条（记结局的时候发的）
            with contextlib.suppress(StoreRefused, sqlite3.Error):
                self.store.change("tasks", task["id"], {"outcome_repushed": crossed, "outcome_repushed_at": round(now, 3)})
            reply = self.summary(task["id"])
            text = "\n".join([outcome_headline(task["id"], task.get("outcome"), task.get("exit_code"), crossed + 1, previous_at, unclaimed=True),
                              f"依据：结局记下 {waited:.0f} 秒（{waited / 60:.0f} 分钟）了，还没交给提交它的会话（发结局的那一条没送到，也没起 task wait 取）",
                              f"此刻还差：{lib.anomaly_advice('unclaimed-outcome', task['id'], self.environment)}（之后每翻一倍再推一回）",
                              "这条结局是：", self.outcome_text(reply)])
            if (self.send_to_session(task, text, ("unclaimed-outcome", task["id"], crossed))
                    and not lib.broken("pushed-outcome-not-collected", self.environment)):
                self.mark_collected(task["id"])

    def push_reminders(self):
        """在排、在跑（连同分片的父任务）而此刻照会话号找得到收件 socket 的任务（inbox_reachable）：疑似卡住与要处置的提醒里还没报的，逐项发给提交它的会话。"""
        for task, valid in self.store.select("tasks", "state != 'finished'"):
            if not valid or task.get("parent_id") or not self.inbox_reachable(task):
                continue
            self.push_suspects(task)
            self.push_attention(task)
            self.push_progress(task, time.time())

    def push_progress(self, task, now):
        """在跑进度（文件头 ⑥「在跑进度」）：这条任务（分片的连同子任务）有尝试在跑，从最早那一次起跑（发过的从上一回发）满 progress_push_seconds（58 分钟），
        给提交它的会话发一条：已跑多久、分片的各片几片结束几片在跑几片在排、每一次在跑的尝试在哪一台、已跑多久、平均在用几核、日志多久没涨与日志末行。
        只报、不停、不要会话处置；发没发出去都记这一回的时刻（发不出去的不每轮重发）。弄坏开关 progress-not-pushed 下不发。"""
        if lib.broken("progress-not-pushed", self.environment):
            return
        try:
            children = self.children_of(task["id"])
        except StoreRefused:
            return
        names = {task["id"]} | {child["id"] for child in children}
        running = sorted((attempt for attempt in self.store.running_attempts() if attempt.get("task_id") in names and attempt.get("started_at")),
                         key=lambda attempt: float(attempt["started_at"]))
        if not running:
            return
        first = float(running[0]["started_at"])
        last = max(first, self.progress_pushed.get(task["id"], 0.0), float(task.get("progress_pushed_at") or 0.0))
        if now - last < self.progress_push_seconds:
            return
        rounds = int(task.get("progress_pushed_rounds") or 0) + 1
        self.progress_pushed[task["id"]] = now
        with contextlib.suppress(StoreRefused, sqlite3.Error):
            self.store.change("tasks", task["id"], {"progress_pushed_at": round(now, 3), "progress_pushed_rounds": rounds})
        lines = [f"{INBOX_HEADLINE}{headline_class('进度', '任务还在跑，不是结局')}任务 {task['id']} 起跑以来 {(now - first) / 60:.0f} 分钟"
                 f"（第 {rounds} 回，每 {self.progress_push_seconds / 60:.0f} 分钟报一回；只报进度，不用处置）"]
        if children:
            finished = sum(1 for child in children if child.get("state") == "finished")
            started = {attempt.get("task_id") for attempt in running} - {task["id"]}
            lines.append(f"子任务 {len(children)} 条：{finished} 条已结束、{len(started)} 条在跑、{len(children) - finished - len(started)} 条在排")
        for attempt in running[:PROGRESS_PUSH_LISTED_ATTEMPTS]:
            lines.append(self.progress_attempt_line(task, attempt, now))
        if len(running) > PROGRESS_PUSH_LISTED_ATTEMPTS:
            lines.append(f"· 另有 {len(running) - PROGRESS_PUSH_LISTED_ATTEMPTS} 次在跑的没列")
        lines.append(f"要看细的：task-server/task status {task['id']}、task-server/task log {task['id']}")
        self.send_to_session(task, "\n".join(lines), None)

    def progress_attempt_line(self, task, attempt, now):
        """在跑进度那一条里一次在跑的尝试那一行（push_progress）：在哪一台、起跑以来多久、
        平均在用几核、日志多久没涨（日志不涨时读到过进展的接着写读到的是哪份文件在涨与多少秒前读的：running_progress 的 progress_note）、日志末行。
        弄坏开关 progress-line-progress-unnamed 下不写读到的进展。"""
        progress = self.running_progress(task, attempt, now)
        cores, quiet = progress.get("cores"), progress.get("log_quiet_seconds")
        tail = [line.strip() for line in log_tail(attempt.get("log", "")).splitlines() if line.strip()]
        note, age = progress.get("progress_note"), progress.get("progress_age_seconds")
        note_text = (f"（日志不涨而有进展：{note}，{age:.0f} 秒前读到）" if note and age is not None and not lib.broken("progress-line-progress-unnamed", self.environment)
                     else "")
        return (f"· 任务 {attempt.get('task_id')} 在 {attempt.get('host')}：起跑以来 {progress['elapsed_seconds'] / 60:.0f} 分钟、"
                f"平均在用 {'量不到' if cores is None else f'{cores:.1f}'} 核、日志 {'量不到多久' if quiet is None else f'{quiet:.0f} 秒'}没涨{note_text}；"
                f"日志末行：{tail[-1][:PROGRESS_PUSH_TAIL_CHARACTERS] if tail else '（日志还是空的）'}")

    def push_attention(self, task, final=False):
        """这条任务（分片的连同子任务）要处置的提醒里还没报给提交它的会话、也没发失败过的，发给它；记账与 wait 同一份 reminded_thresholds
        （同一种同一道门槛只报一次：这里发过的 wait 不再报，wait 报过的这里不再发）。说的是子任务的那几项按 (种类, 提醒门槛, 原因) 合成一条（attention_groups，
        文件头 ⑥「要处置的提醒」）：发出去了每一项都记报过，发不出去每一项都记发失败过。合成的那一组头一轮先压一轮（attention_held）：各片各记提交时刻、相差几毫秒，
        同一刻跨过门槛的兄弟片跨过一轮的边界会拆成两条，下一轮连同新跨过门槛的一起发（组里有一项压过就发）；final（发结局之前那一次，push_outcomes）不压，没有下一轮了。
        弄坏开关 inbox-reported-every-round 下报过的照发。"""
        per_shard = lib.broken("inbox-attention-per-shard", self.environment)
        pending = []
        for item in self.attention_items(task["id"]):
            reported = self.reminded_threshold_of(item) >= item["threshold"] and not lib.broken("inbox-reported-every-round", self.environment)
            if reported and item["kind"] in ANOMALY_REPUSHED_KINDS and not self.anomaly_acknowledged(item):   # 门槛不随时长涨的：同一道门槛推过之后一直还在，满时限再推（attention_repush）；记了照跑的不再推
                repush = self.attention_repush(item)
                if repush:
                    item, reported = dict(item, repush=repush), False
            with self.inbox_lock:
                failed = attention_event(item) in self.inbox_failed
            if not (reported or failed):
                pending.append(item)
        for group in self.attention_groups(task, pending):
            events = [attention_event(item) for item in group]
            if group[0].get("task") != task["id"] and not per_shard and not final:
                with self.attention_lock:
                    held = any(event in self.attention_held for event in events)
                    self.attention_held.update(events)
                if not held:
                    continue
            text = self.attention_text(task, group[0]) if len(group) == 1 else self.merged_attention_text(task, group)
            sent = self.send_to_session(task, text, events[0])
            with self.attention_lock:
                self.attention_held.difference_update(events)
            if sent:
                self.note_attention_reported(group)
            if not sent:
                with self.inbox_lock:
                    self.inbox_failed.update(events)

    def attention_groups(self, task, items):
        """发给会话的提醒怎么合（文件头 ⑥「要处置的提醒」）：说的是父任务自己的一项一组；说的是子任务的，种类、提醒门槛、原因都相同的合成一组（分片排久那一种，
        几十片同一刻跨过同一道门槛、原因逐字相同），组里照任务号从小到大。→ [[项…]…]，次序照每组头一项在 items 里的次序。task wait 那一侧不经这里、照旧逐项报。
        弄坏开关 inbox-attention-per-shard 下照改前一项一组（逐片各发一条）。"""
        if lib.broken("inbox-attention-per-shard", self.environment):
            return [[item] for item in items]
        groups = {}
        for index, item in enumerate(items):
            key = ("own", index) if item.get("task") == task["id"] else ("children", item.get("kind"), item.get("threshold"), item.get("repush", 0), item.get("why"))
            groups.setdefault(key, []).append(item)
        return [sorted(group, key=lambda item: int(item.get("task") or 0)) for group in groups.values()]

    def merged_attention_text(self, task, group):
        """几片子任务合成的那一条提醒的正文（attention_groups）：第一行「task-server：提醒（…，不是结局）任务 <父任务号> <中文名>（<种类>，子任务 <号的区间>，共 <几> 片；第 <N> 回）」，
        再写依据（照号最小的那一片）、原因、出路（照号最小的那一片，别的片换成那一片的号；那一片的项带 loop_leak 的另起一行写那一段）、日志（那一片的），
        末尾是那一片的 TASK_SERVER_ATTENTION 一行原样。"""
        first = group[0]
        found = self.store.task(first["task"])
        log = self.task_log_path(found[0]) if found is not None and found[1] else ""
        repushed = f"，没处置、第 {first['repush']} 回再推" if first.get("repush") else ""
        round_number, previous_at = self.attention_round(first)
        subject = found[0] if found is not None and found[1] else None
        return "\n".join([reminder_headline(task, first, round_number, previous_at, f"{repushed}，子任务 {task_id_ranges([item['task'] for item in group])}，共 {len(group)} 片", subject),
                          *self.repush_lines(first),
                          f"依据：{first.get('basis')}（子任务 {first['task']}；同一道提醒门槛、同一原因的各片合成这一条）", f"原因：{first.get('why')}",
                          f"出路：{first.get('next')}（以子任务 {first['task']} 为例，别的片换成那一片的号）",
                          *([first["loop_leak"]] if first.get("loop_leak") else []), f"日志：{log or '无（还没起过）'}",
                          lib.attention_line(task["id"], first)])

    def push_suspects(self, task):
        """这条任务（分片的连同子任务）在跑的尝试正疑似卡住、这一段还没报的，发给提交它的会话；记账与 wait 同一份 suspect_reported（一段只报一次）。
        弄坏开关 inbox-reported-every-round 下报过的照发。"""
        records = [task]
        if task.get("sharded_entry"):
            with contextlib.suppress(StoreRefused):
                records += self.children_of(task["id"])
        for record in records:
            for attempt, valid in self.store.attempts_of(record["id"]):
                suspect = attempt.get("suspect") or {}
                if not valid or attempt.get("state") not in ACTIVE_ATTEMPT_STATES or not suspect.get("active"):
                    continue
                event = ("suspect", attempt["id"], suspect.get("segment"))
                with self.suspect_lock:
                    reported = self.suspect_reported.get(attempt["id"]) == suspect.get("segment") and not lib.broken("inbox-reported-every-round", self.environment)
                with self.inbox_lock:
                    failed = event in self.inbox_failed
                if reported or failed:
                    continue
                if self.send_to_session(task, self.suspect_text(task, record, attempt, suspect), event):
                    with self.suspect_lock:
                        self.suspect_reported[attempt["id"]] = suspect.get("segment")
                    self.persist_restart_surviving_state("suspect_reported")

    def outcome_text(self, reply):
        """结局那一条的正文：第一行「task-server：结局（任务已结束）任务 <号> <中文名>（<种类>，退 <码>；第 1 回）」（outcome_headline），再写原因、出路、确认标识那几行（flag_lines）、
        日志（照 wait 打的 TASK_SERVER_REASON 的 why、next 与末行的 log），末尾是 wait 的末行原样。
        连着撞上限之后打回的（理由行带 loop_leak=，task_server_lib.py 文件头「打回的理由」）在「出路：」之后另起一行写那一段。"""
        task = reply["task"]
        fields = dict(lib.reply_reason_fields(reply))
        attempts = lib.outcome_attempts(reply)
        flags_first = lib.broken("inbox-flag-lines-before-evidence", self.environment)
        return "\n".join([outcome_headline(task["id"], task.get("outcome"), task.get("exit_code")), *(self.flag_lines(task) if flags_first else []),
                          f"原因：{fields.get('why')}", f"出路：{fields.get('next')}", *([fields["loop_leak"]] if fields.get("loop_leak") else []), *([] if flags_first else self.flag_lines(task)),
                          f"日志：{(attempts[-1].get('log') if attempts else '') or '无（一次都没起）'}", lib.reply_result_line(reply, self.environment)])

    def attention_text(self, task, item):
        """要处置的提醒那一条的正文：第一行「task-server：提醒（任务还在排 / 还在跑，不是结局）任务 <号> <中文名>（<种类>；第 <N> 回）」（reminder_headline；重推的接「上一回在 <日期>」），
        再写依据、原因、出路、确认标识那几行（flag_lines）、日志（照 wait 打的 TASK_SERVER_ATTENTION），末尾是那一行原样；
        项带 loop_leak 的（repeated-cap-hits）在「出路：」之后另起一行写那一段。"""
        part = f"，子任务 {item['task']}" if item.get("task") != task["id"] else ""
        repushed = f"，没处置、第 {item['repush']} 回再推" if item.get("repush") else ""
        found = self.store.task(item["task"])
        log = self.task_log_path(found[0]) if found is not None and found[1] else ""
        round_number, previous_at = self.attention_round(item)
        flags_first = lib.broken("inbox-flag-lines-before-evidence", self.environment)
        subject = found[0] if found is not None and found[1] else None
        return "\n".join([reminder_headline(task, item, round_number, previous_at, f"{repushed}{part}", subject), *(self.flag_lines(task) if flags_first else []), *self.repush_lines(item),
                          f"依据：{item.get('basis')}", f"原因：{item.get('why')}",
                          f"出路：{item.get('next')}", *([item["loop_leak"]] if item.get("loop_leak") else []), *([] if flags_first else self.flag_lines(task)), f"日志：{log or '无（还没起过）'}",
                          lib.attention_line(task["id"], item)])

    def repush_lines(self, item):
        """再推的那一回（item 带 repush）正文另写的两行：之前推过什么（reminder_history_of 里这一种最近几回：隔了多久、当时的依据），此刻还差什么（这一行的修改意见）；
        不是再推的交 []。"""
        if not item.get("repush"):
            return []
        now = time.time()
        history = self.reminder_history_of(item["task"], item["kind"])[-3:]
        shown = "；".join(f"{(now - float(entry.get('at') or now)) / 60:.0f} 分钟前报过"
                         + (f"（第 {entry['repush']} 回再推）" if entry.get("repush") else "") + f"：{entry.get('basis')}" for entry in history)
        return [f"之前推过：{shown or '库里读不到推过的记录'}",
                f"此刻还差：{item.get('next')}（满 {self.anomaly_repush_seconds * 2 ** (item['repush'] - 1):.0f} 秒还没人处置，服务再推这一回）"]

    def suspect_text(self, task, record, attempt, suspect):
        """疑似卡住那一条的正文：第一行「task-server：疑似卡住（任务还在跑、服务没停它）任务 <号> 疑似卡住（suspected-stall，第 <段> 段；第 1 回）」（suspect_headline），再写依据、出路、日志（照 wait 打的 TASK_SERVER_SUSPECT），末尾是那一行原样。"""
        part = f"，子任务 {record['id']}" if record["id"] != task["id"] else ""
        shown = dict(suspect, attempt=attempt.get("number"), host=attempt.get("host"), log=attempt.get("log"))
        return "\n".join([suspect_headline(task, suspect.get("segment"), part), f"依据：{suspect.get('why')}",
                          f"出路：任务照跑、服务不停它；{lib.anomaly_advice('suspected-stall', task['id'], self.environment)}（结局照发）",
                          f"日志：{attempt.get('log') or '无'}", lib.suspect_line(task["id"], shown)])

    def runaway_loop(self):
        """任务之外的进程空转（文件头 ⑥）单独一个线程：读第二台要过 ssh，不挡发消息那一路。一轮出事不停服务：打出来、下一轮再判。"""
        while not self.stopping.wait(self.runaway_scan_seconds):
            try:
                self.watch_runaway_processes()
            except Exception:
                say("  ! 这一轮扫任务之外的进程出了异常（下一轮再判）：\n" + traceback.format_exc())

    def process_listings(self):
        """两台此刻的进程表（lib.PROCESS_LIST_COLUMNS 那几列）→ [(台, 输出)]；读不到的那台不在里面。自证里只读 TASK_SERVER_SELFTEST_PROCESS_LIST 点名的文件（当本机的）。"""
        override = lib.selftest_override("TASK_SERVER_SELFTEST_PROCESS_LIST", self.environment)
        if override or lib.fast(self.environment):
            with contextlib.suppress(OSError):
                if override:
                    with open(override, encoding="utf-8") as handle:
                        return [(LOCAL, handle.read())]
            return []
        listings = []
        with contextlib.suppress(OSError, subprocess.TimeoutExpired):
            completed = subprocess.run(["ps", "-U", str(os.getuid()), "-o", lib.PROCESS_LIST_COLUMNS], capture_output=True, text=True, env=self.environment,
                                       stdin=subprocess.DEVNULL, timeout=PROCESS_LIST_SECONDS)
            if completed.returncode == 0:
                listings.append((LOCAL, completed.stdout))
        peer = self.resources.peer
        if peer is not None and peer.available and not self.resources.peer_cooling_down():   # 第二台上一回读不成还在冷却里的不去连
            code, output, _ = peer.run(f'ps -U "$(id -u)" -o {lib.PROCESS_LIST_COLUMNS}', timeout=PROCESS_LIST_SECONDS)
            if code == 0:
                listings.append((PEER, output))
        return listings

    def registered_session_processes(self):
        """本机此刻活着的 Claude Code 进程 → {进程号: 会话号}；登记读不了交空表。"""
        try:
            registry = lib.session_registry_library()
            directory = lib.selftest_override(lib.SESSIONS_DIRECTORY_OVERRIDE, self.environment) or registry.registry_directory({})
            return {entry.pid: entry.session_id for entry in registry.session_registry_entries(directory) or [] if entry.is_alive}
        except Exception:   # 载不进、读的时候抛错：这一回当没有
            return {}

    def runaway_protected_pids(self, listing):
        """本机不碰的进程号：登记的 Claude Code 进程与它们的祖先、服务自己与它的祖先。"""
        parents = lib.process_parents(listing)
        protected = set()
        for pid in [*self.registered_session_processes(), os.getpid()]:
            while pid and pid not in protected:
                protected.add(pid)
                pid = parents.get(pid, 0)
        return protected

    def runaway_owner(self, pid, listing):
        """本机这个进程属于哪个会话：顺着父进程找到登记的 Claude Code 进程，交会话号；找不到交空串。"""
        parents = lib.process_parents(listing)
        sessions = self.registered_session_processes()
        seen = set()
        pid = parents.get(pid, 0)   # 从父进程起找：会话进程自己空转的不算它名下的进程，不停它
        while pid in parents and pid not in seen:
            if pid in sessions:
                return sessions[pid]
            seen.add(pid)
            pid = parents[pid]
        return sessions.get(pid, "")

    def watch_runaway_processes(self):
        """任务之外的进程空转（文件头 ⑥）：两台各扫一次，判中的写告警、发给所属会话，只报不停。弄坏开关 runaway-process-unreported 下不报。"""
        if lib.broken("runaway-process-unreported", self.environment):
            return
        now, present = time.time(), set()
        for host, listing in self.process_listings():
            for found in lib.runaway_processes(listing, RUNAWAY_MINIMUM_CPU_SECONDS, RUNAWAY_MINIMUM_FRACTION, self.environment):
                key = (host, found["pid"])
                present.add(key)
                entry = self.runaway_notified.get(key)
                # 上次报它之后才起的同号进程是另一个（换服务的空档里原来那个退了、号被复用）：从零数，不接着原来那个的报数停它；
                # elapsed 是那一台量的时长，与两台的钟对不对得上无关。弄坏开关 runaway-reused-pid-carried 下照号接着数
                if (entry is not None and entry["reported_at"] and now - found["elapsed"] > entry["reported_at"]
                        and not lib.broken("runaway-reused-pid-carried", self.environment)):
                    entry = None
                if entry is None:
                    entry = self.runaway_notified[key] = {"reported_at": 0.0, "count": 0, "alerted": False}
                if entry.get("stopped") or now - entry["reported_at"] < self.runaway_renotify_seconds:
                    continue   # 同一个进程只停一次：停不掉的那一条告警写了回读还在
                owner = self.runaway_owner(found["pid"], listing) if host == LOCAL else ""
                targets = self.runaway_targets(owner)
                label = lib.HOST_NAMES[host]
                if not owner:
                    protected = host == LOCAL and found["pid"] in self.runaway_protected_pids(listing)
                    stoppable = (lib.runaway_unowned_stoppable(found, self.environment) and not protected) or lib.broken("runaway-protected-stopped", self.environment)
                    if stoppable and not lib.broken("runaway-unowned-not-stopped", self.environment):
                        stopped = self.stop_runaway_process(host, found["pid"])
                        entry["stopped"] = True
                        self.alert("runaway-process-stopped", 0, f"{label}进程 {found['pid']}（{found['command']}）认不出发起的会话，服务直接按号终止：{'回读已不在' if stopped else '回读还在'}",
                                   "看是谁留下的，改掉那个写法；回读还在的按进程号查它为什么停不掉")
                    elif not entry["alerted"]:
                        entry["alerted"] = True
                        self.alert("runaway-process", 0, f"{label}进程 {found['pid']}（{found['command']}）在空转，认不出发起的会话，属于不碰的那一类（cgroup {found['cgroup']}）",
                                   "看它是不是该这么忙；是系统服务的查那个服务")
                    entry["reported_at"] = now
                    continue
                if entry["count"] >= RUNAWAY_REPORTS_BEFORE_STOP and not lib.broken("runaway-process-not-stopped", self.environment):
                    stopped = self.stop_runaway_process(host, found["pid"])
                    entry["stopped"] = True
                    line = f"本机进程 {found['pid']}（{found['command']}）报了 {entry['count']} 次还在，服务按号停了：{'回读已不在' if stopped else '回读还在'}"
                    self.alert("runaway-process-stopped", 0, line, "看是哪条命令留下的，改掉那个写法；回读还在的按进程号查它为什么停不掉")
                    for task in targets:
                        self.send_to_session(task, f"{INBOX_HEADLINE}任务之外的进程空转 已停 本机进程 {found['pid']}\n{line}", None)
                    continue
                stop = f"python3 .claude/singlefs-ai-sop/scripts/proc.py stop {found['pid']}"
                basis = (f"已跑 {found['elapsed']} 秒、累计 CPU {found['cpu']} 秒（占 {found['cpu'] / max(found['elapsed'], 1):.0%}），"
                         f"命令名 {found['command']}、cgroup {found['cgroup']}，不在任务 slice 里；是你这个会话发起的")
                after = f"报满 {RUNAWAY_REPORTS_BEFORE_STOP} 次还在，服务按号停掉；还在的每 {self.runaway_renotify_seconds:.0f} 秒再报一次"
                next_step = f"看它在做什么：用不上的现在就按进程号停（{label}上跑 {stop}）。{after}"
                if not entry["alerted"]:
                    entry["alerted"] = True
                    self.alert("runaway-process", 0, f"{label}进程 {found['pid']}：{basis}", next_step)
                entry["reported_at"] = now
                entry["count"] += 1
                for task in targets:
                    self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}任务之外的进程空转 {label}进程 {found['pid']}", f"依据：{basis}", f"出路：{next_step}"]), None)
        for key in [key for key in self.runaway_notified if key not in present]:
            del self.runaway_notified[key]
        self.persist_restart_surviving_state("runaway_notified")

    def runaway_targets(self, owner):
        """任务之外的进程空转发给谁 → [任务…]（send_to_session 照任务记的地址发）：谁的进程提醒谁，只发所属会话，地址取它最近一条找得到收件 socket 的任务；认不出所属或没有地址的交空表。
        只挑带收件地址的那几行（分片的子任务没有 inbox，不占那 20 行：同一会话最近交了不少于 20 片的分片任务时照样找得到它的父任务）。
        弄坏开关 runaway-sent-to-every-session 下发给服务健康的那几个会话，runaway-targets-counting-children 下子任务也占那 20 行。"""
        if lib.broken("runaway-sent-to-every-session", self.environment):
            return list(self.health_recipients().values())
        if not owner:
            return []
        where = "session = ?" if lib.broken("runaway-targets-counting-children", self.environment) else "session = ? AND json_extract(record, '$.inbox') IS NOT NULL"
        for task, valid in self.store.select("tasks", where, (owner,), order="id DESC LIMIT 20"):
            if valid and self.inbox_reachable(task):
                return [task]
        return []

    def stop_runaway_process(self, host, pid):
        """在那一台上按进程号停（两台同一个次序与判法：先 TERM，过宽限还在发 KILL）→ 回读 /proc 已不在（僵尸算不在）。本机 lib.stop_process_here，第二台经 ssh 跑 lib.stop_process_script。"""
        grace = 1 if lib.fast(self.environment) else int(RUNAWAY_STOP_GRACE_SECONDS)
        if host == PEER:
            return self.resources.peer.run(lib.stop_process_script(pid, grace), timeout=2 * grace + PROCESS_LIST_SECONDS)[0] == 0
        return lib.stop_process_here(pid, grace)

    def unit_unreadable_report(self):
        """/health 报的读不到单元状态（文件头 ①「读不到单元状态」）：→ ([{attempt, task, number, host, unit, seconds, presence, detail}…], 那一句或空串)。
        只读对账记下的那一段（unit_unreadable）与不经 systemd 最近一回核到的（unit_presence），不现读 systemd。"""
        now, entries = time.monotonic(), []
        for attempt_id, entry in sorted(dict(self.unit_unreadable).items()):
            verdict, detail = self.unit_presence_now(attempt_id)
            entries.append({"attempt": attempt_id, "task": entry["task"], "number": entry["number"], "host": entry["host"], "unit": entry["unit"],
                            "seconds": round(now - entry["since"], 1), "presence": verdict, "detail": detail})
        if not entries:
            return [], ""
        shown = "；".join(f"任务 {entry['task']} 第 {entry['number']} 次（派在{lib.HOST_NAMES.get(entry['host'], entry['host'])}，单元 {entry['unit']}，"
                         f"连着 {entry['seconds']:.0f} 秒，不经 systemd 核到{lib.PRESENCE_WORDS[entry['presence']]}）" for entry in entries[:UNIT_UNREADABLE_HEALTH_SHOWN])
        more = f"，另有 {len(entries) - UNIT_UNREADABLE_HEALTH_SHOWN} 次" if len(entries) > UNIT_UNREADABLE_HEALTH_SHOWN else ""
        return entries, (f"{UNIT_UNREADABLE_HEALTH_TEXT}（systemctl --user show 起不来、超时或没打 ActiveState），{len(entries)} 次：{shown}{more}；"
                         "服务不杀、不收尾、不记结局，照记 running（出路：在本机核用户级 systemd，systemctl --user show -p ActiveState <单元> 答得上之后服务照常判）")

    # ── 记账对账（文件头 ②「内存」的记账对账）──
    def booking_audit_loop(self):
        """每 booking_audit_seconds 一轮：读最近一份余量（第二台取探测线程那一份，不连第二台），逐台对内存记账（audit_bookings）；一轮出事不停服务。"""
        while not self.stopping.wait(self.booking_audit_seconds):
            try:
                self.audit_bookings(self.resources.snapshot(), time.time())
            except Exception:
                say("  ! 这一轮记账对账出了异常（下一轮再判）：\n" + traceback.format_exc())

    def audit_bookings(self, snapshot, now):
        """记账对账一轮：内存（audit_memory_bookings），判出的推给被挡住、还在排的任务所在的会话（push_booking_drift）。只报不停任务。弄坏开关 booking-audit-skipped 下不对账。"""
        if lib.broken("booking-audit-skipped", self.environment):
            return
        self.audit_memory_bookings(snapshot, now)
        self.refresh_booking_ledger(snapshot, now)
        self.push_booking_drift(now)

    @staticmethod
    def running_booking_text(basis, used, cap):
        """在跑记账为什么记这么多的那一句（booking_row 与 ledger_row 共用；basis 是 running_booking 交的 BOOKED_FROM_* 之一）：写清取的是此刻实测、实测到了上限，
        还是报告里没有它的 scope 记整份上限。"""
        human = gibibytes_text
        if basis == BOOKED_WITHOUT_SCOPE:
            return f"报告里没有它的 scope（刚起、读不到用量或这一轮这台没报告），记整份上限 {human(cap)}"
        if basis == BOOKED_WHILE_YOUNG:
            return f"起跑不满 {CPU_MEASURED_AFTER_SECONDS:.0f} 秒，照定的规矩记整份上限 {human(cap)}（满了之后照实测记；不算漂移）"
        if basis == BOOKED_FROM_CAP:
            return f"此刻实测 {human(int(used or 0))} 到了它的上限，记上限 {human(cap)}"
        return f"照此刻实测的用量 {human(int(used or 0))} 记"

    def booking_row(self, attempt, usage):
        """一条在跑尝试的记账明细 {label, task, booked, used, cap, why}：记多少（running_booking）、实占多少（报告里它的 scope 回收不掉的用量，没有 scope 是 None）、
        为什么记这么多（running_booking_text）。"""
        label, cap = attempt.get("label") or "", int(attempt.get("memory_cap") or 0)
        used = usage.get(label)
        booked, basis = self.running_booking(attempt, usage)
        return {"label": label, "task": attempt.get("task_id"), "booked": booked, "used": used, "cap": cap, "basis": basis, "why": self.running_booking_text(basis, used, cap)}

    def audit_memory_bookings(self, snapshot, now):
        """内存：逐台比「这台在跑尝试的记账之和」与对账的基准「报告里各 scope 回收不掉的用量之和」；
        不少于 BOOKING_DRIFT_RATIO 倍且多出 BOOKING_DRIFT_BYTES、连着 BOOKING_DRIFT_ROUNDS 轮，判记账漂移，写进 booking_drift['memory'][台]（逐条列各尝试记多少、实占多少、
        为什么记这么多）；这一轮对得上了的清掉。读不到这台的报告、这台没有在跑的不判（连数清零）。有 scope 的照实测记（文件头 ②「内存」），
        多出来的只来自报告里没有它的 scope、记整份上限的那几条。"""
        for host in self.machines:
            key, report = ("memory", host), snapshot.reports.get(host)
            attempts = [attempt for attempt in self.booking_attempts() if attempt.get("host") == host] if report is not None else []
            if not attempts:
                self.forget_drift(key)
                continue
            usage = self.report_scope_usage(report)
            rows = [self.booking_row(attempt, usage) for attempt in attempts]
            # 起跑不满两分钟、照规矩记整份上限的那几条（BOOKED_WHILE_YOUNG）不是漂移：对账时它们的记账照实占算（任务 15822 第 2 次尝试起跑 103 秒时被报成漂移）；
            # 弄坏开关 booking-audit-young-counted 下照整份上限算
            young_counted = lib.broken("booking-audit-young-counted", self.environment)
            booked = sum(int(row["used"] or 0) if row["basis"] == BOOKED_WHILE_YOUNG and not young_counted else row["booked"] for row in rows)
            used = sum(usage.values())
            expected = used
            if not (booked >= BOOKING_DRIFT_RATIO * expected and booked - expected >= BOOKING_DRIFT_BYTES):
                self.forget_drift(key)
                continue
            with self.booking_drift_lock:
                rounds = self.booking_audit_state.get(key, 0) + 1
                self.booking_audit_state[key] = rounds
                if rounds < BOOKING_DRIFT_ROUNDS:
                    continue
                known = self.booking_drift["memory"].get(host)
                shown = "；".join(f"任务 {row['task']} 尝试 {row['label']} 记 {gibibytes_text(row['booked'])}、实占 {'读不到' if row['used'] is None else gibibytes_text(row['used'])}：{row['why']}" for row in rows)
                text = (f"{self.machines[host].label}在跑的内存记账之和 {gibibytes_text(booked)}，实测各 scope 回收不掉的用量之和 {gibibytes_text(used)}"
                        f"（对账的基准；连着 {rounds} 轮不少于基准的 {BOOKING_DRIFT_RATIO:g} 倍、多出 {gibibytes_text(BOOKING_DRIFT_BYTES)} 以上）；逐条：{shown}")
                self.booking_drift["memory"][host] = {"since": known["since"] if known else now, "rounds": rounds, "booked": booked, "used": used, "expected": expected,
                                                      "attempts": rows, "text": text}
            if known is None:
                say(f"  ! 记账漂移：{text}")

    def forget_drift(self, key):
        """这一项（("memory", 台)）这一轮对得上、或不再判：连数清零、判出的记录删掉、推过的记录忘掉（再漂移再推）。"""
        with self.booking_drift_lock:
            self.booking_audit_state.pop(key, None)
            if self.booking_drift[key[0]].pop(key[1], None) is not None:
                say(f"  · 记账漂移（{key[0]} {key[1]}）好了")
            for notified in [notified for notified in self.booking_drift_notified if notified[0] == key]:
                del self.booking_drift_notified[notified]
        self.persist_restart_surviving_state("booking_drift_notified")

    def ledger_row(self, attempt, usage):
        """一条在跑尝试在记账明细里的一行（refresh_booking_ledger）：{task, number, label, cap, used, booked, basis, at_cap, text}。
        basis 说取的是哪一个：stopped（单元已确认停了、收尾没做完，不计），或 running_booking 交的 no-scope（报告里没有它的 scope，记整份上限）、
        measured（照报告里它的 scope 此刻回收不掉的用量）、cap（实测到了它的上限、记上限）；at_cap 为真是记的就是上限。
        text 照 running_booking_text 写清取的是哪一个与几个数。"""
        label, cap = attempt.get("label") or "", int(attempt.get("memory_cap") or 0)
        row = {"task": attempt.get("task_id"), "number": attempt.get("number"), "label": label, "cap": cap, "used": usage.get(label),
               "booked": 0, "basis": "stopped", "at_cap": False}
        if attempt.get("id") in self.unit_stopped:
            row["text"] = "单元已确认停了、收尾还没做完，不计"
            return row
        row["booked"], row["basis"] = self.running_booking(attempt, usage)
        row["at_cap"] = row["booked"] >= cap
        row["text"] = self.running_booking_text(row["basis"], row["used"], cap)
        return row

    def refresh_booking_ledger(self, snapshot, now):
        """在跑记账的逐条明细（/health 的 booking_ledger 一栏；control bookings 打出来）：逐台列这台每一条在跑的尝试——上限、此刻实占（报告里它的 scope 回收不掉的用量）、
        记多少、为什么记这么多（ledger_row）——与这台的合计：总额、余量、在跑上限之和、各 scope 实占合计、记账合计、照公式（总额 − 余量 − 记账合计）还放得下多少、
        实测 slice 还放得下多少。这台这一轮没有余量报告的只记一句。弄坏开关 bookings-ledger-unreported 下留空表。"""
        ledger = {}
        if not lib.broken("bookings-ledger-unreported", self.environment):
            human = gibibytes_text
            running = self.store.running_attempts()
            for host in self.machines:
                label, report = self.machines[host].label, snapshot.reports.get(host)
                if report is None:
                    ledger[host] = {"at": round(now, 1), "read": False, "rows": [], "title": f"{label}：这一轮没有余量报告（探测没回报告），记不出",
                                    "total": f"{label}：合计记不出"}
                    continue
                usage = self.report_scope_usage(report)
                rows = sorted((self.ledger_row(attempt, usage) for attempt in running if attempt.get("host") == host), key=lambda row: -row["booked"])
                margin = self.host_margin(host, snapshot)[0]
                slice_total, booked = snapshot.total(host), sum(row["booked"] for row in rows)
                caps, used = sum(row["cap"] for row in rows), sum(usage.values())
                room = None if margin is None else snapshot.total(host) - margin - booked   # 文件头 ②「内存」的公式
                slice_room = (report.get("memory") or {}).get("slice_room")
                title = (f"{label}：slice 总额 {human(slice_total)}、余量 {'量不到' if margin is None else human(margin)}、在跑 {len(rows)} 条的上限之和 {human(caps)}、"
                         f"各 scope 实占合计 {human(used)}")
                total = (f"{label}合计：记账 {human(booked)}（实占 {human(used)}），照公式（总额减余量减记账合计）放得下 "
                         f"{'量不到余量，算不出' if room is None else human(max(room, 0))}"
                         f"{'' if not isinstance(slice_room, int) else '；实测 slice 还放得下 ' + human(max(slice_room, 0))}")
                ledger[host] = {"at": round(now, 1), "read": True, "slice_total": slice_total, "margin": margin, "caps": caps, "used": used, "booked": booked,
                                "room": room, "slice_room": slice_room if isinstance(slice_room, int) else None, "rows": rows, "title": title, "total": total}
        with self.booking_drift_lock:
            self.booking_ledger = ledger

    def booking_ledger_snapshot(self):
        """/health 的 booking_ledger 一栏：{台: refresh_booking_ledger 记的那一台的明细}；记账对账还没做过一轮、或弄坏开关 bookings-ledger-unreported 下是空表。"""
        with self.booking_drift_lock:
            return json.loads(json.dumps(self.booking_ledger))

    def booking_drift_snapshot(self):
        """/health 的 booking_drift 一栏：{memory: {台: 明细}}，没有漂移的是空表。"""
        with self.booking_drift_lock:
            return json.loads(json.dumps({"memory": self.booking_drift["memory"]}))

    def drift_recipients(self):
        """被内存记账漂移挡住、还在排的任务所在的会话：{会话: 它那条任务（分片的子任务取父任务，收件地址在父任务上）}。看排队原因里写着 RUNNING_BOOKED_SUM_TEXT 的；
        照会话号找不到收件 socket 的不算。"""
        recipients = {}
        for task, valid in self.store.select("tasks", "state = 'queued'"):
            if not valid or RUNNING_BOOKED_SUM_TEXT not in self.waiting_reasons.get(task["id"], ""):
                continue
            found = self.store.task(task["parent_id"]) if task.get("parent_id") else (task, True)
            owner = found[0] if found is not None and found[1] else None
            if owner is not None and self.inbox_reachable(owner):
                recipients[owner.get("session", "")] = owner
        return recipients

    def push_booking_drift(self, now):
        """判出的记账漂移推给被挡住、还在排的任务所在的会话：同一件事对同一个会话判出来当场推一回，之后照 reminder_threshold（门槛从 BOOKING_DRIFT_PUSH_SECONDS 起、每翻一倍一道）
        每跨过一道门槛再推一回，同一道门槛不重推；好了再漂移再推（forget_drift 忘掉推过的）。"""
        with self.booking_drift_lock:
            current = [(("memory", host), entry) for host, entry in self.booking_drift["memory"].items()]
        for key, entry in current:
            crossed = reminder_threshold(now - entry["since"], BOOKING_DRIFT_PUSH_SECONDS)
            again = f"（判出满 {now - entry['since']:.0f} 秒还在，再推一回）" if crossed else ""
            for session, owner in self.drift_recipients().items():
                with self.booking_drift_lock:
                    if crossed <= self.booking_drift_notified.get((key, session), -1):
                        continue
                    self.booking_drift_notified[(key, session)] = crossed
                self.persist_restart_surviving_state("booking_drift_notified")   # 先记再发：发的当中服务没了，下一回起来不再推同一道门槛
                self.send_to_session(owner, "\n".join([f"{INBOX_HEADLINE}记账漂移 内存记账{again}", f"问题：{self.drift_text_shown(entry)}",
                                                       "出路：服务只报不停任务；挡着你排队的是上面这几条在跑的尝试多记的账：它们跑完或 task-server/task cancel 就放掉；"
                                                       "账一直不回落的把这一条交给维护 task-server 的会话（task-server/control check 也列着）"]), None)

    @staticmethod
    def drift_text_shown(entry):
        """推给会话的正文：逐条列的尝试至多 BOOKING_DRIFT_ROWS_SHOWN 条，多的写还有几条（/health 与 control check 列全）。"""
        rows = entry["attempts"]
        if len(rows) <= BOOKING_DRIFT_ROWS_SHOWN:
            return entry["text"]
        return f"{entry['text'].split('；逐条：')[0]}；逐条：" + "；".join(
            f"任务 {row['task']} 尝试 {row['label']} 记 {gibibytes_text(row['booked'])}、实占 {'读不到' if row['used'] is None else gibibytes_text(row['used'])}：{row['why']}"
            for row in rows[:BOOKING_DRIFT_ROWS_SHOWN]) + f"；还有 {len(rows) - BOOKING_DRIFT_ROWS_SHOWN} 条"

    def health_recipients(self):
        """名下有在排、在跑任务（分片的父任务也算）、此刻照会话号找得到收件 socket 的会话（inbox_reachable）：{会话: 它最新那一条这样的任务}（服务健康发给它们）。"""
        recipients = {}
        for task, valid in self.store.select("tasks", "state != 'finished'"):
            if valid and not task.get("parent_id") and self.inbox_reachable(task):
                recipients[task.get("session", "")] = task   # 照任务号从小到大：留最新那一条的地址
        return recipients

    def health_problems(self):
        """服务自己盯的健康（文件头 ⑥「服务健康」）：库写不写得进（试写一行）、派活判排头能起却一直没起成与看进展
        （/health 的 problem 读的那两句，排队线程每一轮算好的）、自己的单元在不在（is-active 的输出）、NFS 共享前三项（nfs_loop 复核的那一份）、
        第二台用不了满 peer_unavailable_notify_seconds（note_peer 记的）。→ [(种类, 名字, 那一句)]。"""
        problems = []
        writable, problem = self.store.health_write()
        if not writable:
            problems.append(("database-unwritable", "库写不进", f"库写不进：{problem}"))
        if self.starved_text:
            problems.append(("head-not-started", "派活判排头能起却一直没起成", self.starved_text))
        if self.progress_text:
            problems.append(("no-progress", "看进展：放得下却一直一条都没起", self.progress_text))
        state = own_unit_state(self.environment)
        if state not in lib.RUNNING_UNIT_STATES:
            problems.append(("unit-missing", "单元不在", f"单元 {lib.UNIT_NAME} 不在（is-active 打的是「{state}」）"))
        nfs = self.nfs_snapshot()
        if not nfs.get("ok", True):
            problems.append(("nfs", "NFS 共享判不过", f"NFS 共享判不过：{nfs.get('problem')}（出路：{nfs.get('next_step')}）"))
        peer = self.peer_health()
        since = peer.get("since")
        if peer.get("available") is False and isinstance(since, (int, float)) and time.time() - since >= self.peer_unavailable_notify_seconds:
            problems.append(("peer-unavailable", "第二台用不了", f"第二台用不了已 {time.time() - since:.0f} 秒：{peer.get('why')}（派得到第二台的这时只派本机）"))
        problems.extend(self.gpu_card_failures())
        return problems

    def gpu_cards_failed_command(self):
        """读卡坏起的那一条：自证里是 TASK_SERVER_SELFTEST_GPU_CARDS_FAILED 那一份（没给就不读，交 None），别处是 gpu-verification-server/control cards-failed。"""
        fake = lib.selftest_override("TASK_SERVER_SELFTEST_GPU_CARDS_FAILED", self.environment)
        if fake:
            return [fake]
        if lib.in_selftest(self.environment):
            return None
        return [sys.executable, "-B", lib.GPU_SERVICE_CONTROL, "cards-failed"]

    def gpu_card_failures(self):
        """GPU 服务卡坏与判不了的块（文件头 ⑥「服务健康」）：上一回 gpu-verification-server/control cards-failed 读到的每一张坏卡一样
        （种类 gpu-card-failed:<台>:<进程号>:<卡号>，出路 GPU_CARD_FAILED_NEXT_STEP）、每一台每一条判不了的流一样（种类 gpu-block-not-judgeable:<台>:<族>:<流键头>，
        出路照这条流各块的 claim= 写，gpu_block_not_judgeable_next_step），有认不出的行再一样（GPU_LINES_UNPARSED_KIND）；
        到时候了另起一条线程再读（自证给了替身的每一轮都读），这一回不等它。
        弄坏开关 gpu-card-failure-unwatched 下不读、交空；gpu-block-not-judgeable-claim-ignored 下判不了的出路不看认领，一律写放了。→ [(种类, 名字, 那一句, 出路)]。"""
        if lib.broken("gpu-card-failure-unwatched", self.environment):
            return []
        command = self.gpu_cards_failed_command()
        if command is None:
            return []
        period = 0 if lib.selftest_override("TASK_SERVER_SELFTEST_GPU_CARDS_FAILED", self.environment) else GPU_CARD_FAILURE_CHECK_SECONDS
        with self.gpu_cards_failed_lock:
            due = not self.gpu_cards_failed_reading and time.time() - self.gpu_cards_failed_read_at >= period
            if due:
                self.gpu_cards_failed_reading = True
            found = list(self.gpu_cards_failed)
            unparsed = list(self.gpu_lines_unparsed)
        if due:
            threading.Thread(target=self.read_gpu_cards_failed, name="task-server-gpu-cards-failed", daemon=True).start()
        problems = []
        for kind, line, _, claims in found:
            if claims is None:
                problems.append((kind, "GPU 服务卡坏", line, GPU_CARD_FAILED_NEXT_STEP))
                continue
            let_go, kept = claims
            if lib.broken("gpu-block-not-judgeable-claim-ignored", self.environment):
                let_go, kept = let_go + kept, 0
            problems.append((kind, "GPU 服务判不了的块", line, gpu_block_not_judgeable_next_step(let_go, kept)))
        if unparsed:
            problems.append((GPU_LINES_UNPARSED_KIND, "GPU 服务的卡坏行认不出",
                             f"gpu-verification-server/control cards-failed 有 {len(unparsed)} 行认不出（卡号或判不了那一块的族、流键头、块号、认领取不到，或认领的值认不得），"
                             f"头一行：{unparsed[0][:300]}", GPU_LINES_UNPARSED_NEXT_STEP))
        return problems

    def read_gpu_cards_failed(self):
        """起一回读卡坏的那一条（gpu_card_failures 另起的线程里）：退 0 的照标准输出换上读到的（lib.gpu_service_report），没读成的台与认不出的行所在的台
        留着那一台上一回读到的（lib.gpu_service_problems_merged），记下读完过一回、这一回读成了哪几台（lib.gpu_service_hosts_read，并进 gpu_hosts_read）；
        起不了、超时、退非 0 的整份留着上一回的、往服务日志记一行。命令在这里照字面现拼（与 gpu_cards_failed_command 同一条；自证里不给替身时
        gpu_card_failures 不起这一条线程），不经线程参数带进来：算键的闭包认得出本函数里的字面列表与替身，认不出线程参数与方法的返回值。"""
        found, unread, unparsed, hosts_read = None, set(), [], set()
        fake = lib.selftest_override("TASK_SERVER_SELFTEST_GPU_CARDS_FAILED", self.environment)
        command = [fake] if fake else [sys.executable, "-B", lib.GPU_SERVICE_CONTROL, "cards-failed"]
        try:
            completed = subprocess.run(command, cwd=lib.REPOSITORY_ROOT, capture_output=True, text=True, errors="replace", env=self.environment,
                                       stdin=subprocess.DEVNULL, timeout=GPU_CARDS_FAILED_READ_SECONDS)
            if completed.returncode == 0:
                found, unread, unparsed = lib.gpu_service_report(completed.stdout, self.environment)
                unread |= {line.split(" ", 2)[1].partition("=")[2] for line in unparsed if len(line.split(" ", 2)) > 1}
                hosts_read = lib.gpu_service_hosts_read(completed.stdout) - unread
                if unparsed:
                    say(f"  ✗ 读 GPU 服务的卡坏：{len(unparsed)} 行认不出（卡号或判不了那一块的族、流键头、块号、认领取不到，或认领的值认不得），"
                        f"那几台留着上一回读到的；头一行：{unparsed[0][:300]}")
            else:
                say(f"  · 读 GPU 服务的卡坏：{' '.join(command)} 退 {completed.returncode}：{lib.last_line(completed.stderr, '没有标准错误')[:300]}（留着上一回读到的）")
        except (OSError, subprocess.TimeoutExpired) as error:
            say(f"  · 读 GPU 服务的卡坏：{' '.join(command)} 没跑成（{type(error).__name__}: {error}；留着上一回读到的）")
        with self.gpu_cards_failed_lock:
            if found is not None:
                self.gpu_cards_failed = lib.gpu_service_problems_merged(self.gpu_cards_failed, found, unread, self.environment)
                self.gpu_lines_unparsed = unparsed
                self.gpu_hosts_read |= hosts_read
                self.gpu_cards_failed_known = True
            self.gpu_cards_failed_read_at = time.time()
            self.gpu_cards_failed_reading = False

    def gpu_health_settled(self, kind):
        """读 GPU 服务得来的一样（GPU_HEALTH_KIND_PREFIXES）这一回没查出，能不能当成好了、忘掉（watch_health；文件头 ⑥「服务健康」）：
        服务起来之后还没读完过一回的不能；判不了的流不能（GPU 服务重起之后这条流的块还没再试，cards-failed 读不到它，不是好了；再试又判不了也不重发）；
        卡坏的要它那一台自服务起来读成过（gpu_hosts_read）：一直没读成的那一台，记着发过的不当成好了；认不出的行那一样读完过一回就能。
        弄坏开关 gpu-health-forgotten-before-the-first-read 下一律能；gpu-block-not-judgeable-forgotten-between-reads 下判不了的流照卡坏的办；
        gpu-health-unread-host-taken-as-healed 下不看那一台读成过没有。"""
        if lib.broken("gpu-health-forgotten-before-the-first-read", self.environment):
            return True
        with self.gpu_cards_failed_lock:
            known, hosts_read = self.gpu_cards_failed_known, set(self.gpu_hosts_read)
        if not known:
            return False
        if kind == GPU_LINES_UNPARSED_KIND:
            return True
        if kind.startswith(f"{lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:") and not lib.broken("gpu-block-not-judgeable-forgotten-between-reads", self.environment):
            return False
        if lib.broken("gpu-health-unread-host-taken-as-healed", self.environment):
            return True
        return any(kind.startswith(f"{alert_kind}:{host}:") for host in hosts_read
                   for alert_kind in (lib.GPU_CARD_FAILED_ALERT_KIND, lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND))

    def watch_health(self):
        """服务健康（文件头 ⑥）：查出一样，往每个名下有在排、在跑任务的会话发一条「task-server：服务健康 <哪一样>」；同一样对同一个会话只发一次，
        这一轮查不出了（好了）就忘掉，再坏再发；读 GPU 服务得来的那几样照 gpu_health_settled 判能不能忘掉。没有要发给的会话时不查（试写库、起 systemctl 都省掉），记下的也忘掉。
        出路照那一样自带的（gpu_card_failures 交的第四项），没带的写 HEALTH_NEXT_STEP。
        弄坏开关 health-unwatched 下不查不发，health-alerted-every-round 下每一轮照发。"""
        if lib.broken("health-unwatched", self.environment):
            return
        recipients = self.health_recipients()
        if not recipients:
            self.health_notified.clear()
            self.persist_restart_surviving_state("health_notified")
            return
        problems = self.health_problems()
        found = {kind for kind, *_ in problems}
        for kind in [kind for kind in self.health_notified if kind not in found]:
            if kind.startswith(GPU_HEALTH_KIND_PREFIXES) and not self.gpu_health_settled(kind):
                continue   # 不知道好没好，不当成好了（gpu_health_settled）
            say(f"  · 服务健康：{kind} 好了")
            del self.health_notified[kind]
        for kind, name, line, *own_next_step in problems:
            alert_kind = next((prefix for prefix in (lib.GPU_CARD_FAILED_ALERT_KIND, lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND) if kind.startswith(f"{prefix}:")), None)
            next_step = own_next_step[0] if own_next_step else HEALTH_NEXT_STEP
            if alert_kind and kind not in self.gpu_cards_failed_alerted:
                self.gpu_cards_failed_alerted.add(kind)
                self.persist_restart_surviving_state("gpu_alerted")   # 先记再写：写的当中服务没了，下一回起来不再写同一条
                self.alert(alert_kind, 0, line, next_step)
            told = self.health_notified.setdefault(kind, set())
            for session, task in recipients.items():
                if session in told and not lib.broken("health-alerted-every-round", self.environment):
                    continue
                told.add(session)   # 发不出去也记下：同一样不对这个会话重试到死（send_to_session 写了告警）
                self.persist_restart_surviving_state("health_notified")   # 先记再发：发的当中服务没了，下一回起来不再发同一条
                self.send_to_session(task, "\n".join([f"{INBOX_HEADLINE}服务健康 {name}", f"问题：{line}", f"出路：{next_step}"]), None)
        self.persist_restart_surviving_state("health_notified")

    def selftest_advance_fault(self):
        """只在自证里：TASK_SERVER_SELFTEST_ADVANCE_FAILS=<文件>，那份文件在时 drive_sharded_parents 每推一个父任务先抛一个错（演分片父任务推不动）。"""
        arm = lib.selftest_override("TASK_SERVER_SELFTEST_ADVANCE_FAILS", self.environment)
        if arm and os.path.exists(arm):
            raise RuntimeError("自证造的推不动（TASK_SERVER_SELFTEST_ADVANCE_FAILS 点名的文件在）")

    def note_advance_failure(self, parent_id, why):
        """分片父任务这一轮推不动（drive_sharded_parents）：记进内存、整份记进库（换服务后照库接着数），attention_items 读。
        连着推不动是同一回（count 加一），推通之后再推不动是新的一回（提醒门槛加一）。"""
        with self.attention_lock:
            current = self.advance_failures.get(parent_id)
            if current is None:
                episode = self.advance_episodes.get(parent_id, 0) + 1
                self.advance_episodes[parent_id] = episode
                self.advance_failures[parent_id] = {"episode": episode, "why": why, "count": 1}
            else:
                current.update(why=why, count=current["count"] + 1)
        self.persist_restart_surviving_state("shard_advance")

    def note_advance_recovered(self, parent_id):
        with self.attention_lock:
            self.advance_failures.pop(parent_id, None)
        self.persist_restart_surviving_state("shard_advance")

    def log_changed_at(self, attempt, now):
        """日志文件的修改时刻换到 time.monotonic() 的钟上（不晚于 now）；读不到交 now。"""
        if lib.broken("stall-clock-reset-on-restart", self.environment):
            return now
        try:
            age = time.time() - os.path.getmtime(attempt.get("log", ""))
        except OSError:
            return now
        return now - max(0.0, age)

    def scope_cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU：attempt_runner 每次采样写进尝试目录的 cpu_usec（这一条子孙进程所在各 scope 的合计，里层包装各开的 scope 也在内）；
        读不到交 None。弄坏开关 outer-scope-cpu-only 下只读外层那一个 scope（改前的样子）。"""
        try:
            if lib.broken("outer-scope-cpu-only", self.environment):
                with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                    scope = handle.read().strip()
                with open(os.path.join(scope, "cpu.stat"), encoding="utf-8") as handle:
                    return next(int(line.split()[1]) for line in handle if line.startswith("usage_usec "))
            with open(os.path.join(attempt.get("directory", ""), "cpu_usec"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError, StopIteration, IndexError):
            return None

    def scope_thread_reading(self, attempt):
        """这一次尝试最近一轮的线程读数（本机的 Machine.thread_reading；文件头 ②「CPU 闸」的可跑线程数）：attempt_runner 每轮采样写进尝试目录的 lib.THREADS_FILE_NAME
        （JSON：sample、runnable、total、top_threads、top_name），读不到、写坏的交 None。"""
        try:
            with open(os.path.join(attempt.get("directory", ""), lib.THREADS_FILE_NAME), encoding="utf-8") as handle:
                return thread_reading_of(json.load(handle))
        except (OSError, ValueError):
            return None

    def scope_memory_peak(self, attempt):
        """这一次尝试进程内存（anon 加 shmem）见过的最大，字节：attempt_runner 每轮采、涨了就写进尝试目录的 memory_peak（文件头 ①「顶在上限上卡住」）；
        读不到（还没采到）交 None。弄坏开关 memory-peak-with-page-cache 下照改前读外层 scope（尝试目录的 scope 文件记着）的 memory.peak（含页缓存）。"""
        try:
            if lib.broken("memory-peak-with-page-cache", self.environment):
                with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                    scope = handle.read().strip()
                with open(os.path.join(scope, "memory.peak"), encoding="utf-8") as handle:
                    return int(handle.read().strip())
            with open(os.path.join(attempt.get("directory", ""), lib.MEMORY_PEAK_FILE_NAME), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError):
            return None

    def scope_memory_current(self, attempt):
        """这一次尝试外层 scope（内存包装给它开的、尝试目录的 scope 文件记着）此刻进程用的内存（memory.stat 的 anon 加 shmem，memory_peaks_lib 的 process_memory_bytes），
        字节；读不到（还没采到 scope、scope 已收尾）交 None（本机的 Machine.memory_now；judge_stall_stop 判贴着上限用）。
        弄坏开关 memory-peak-with-page-cache 下照改前读 memory.current（含页缓存）。"""
        try:
            with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                scope = handle.read().strip()
            if lib.broken("memory-peak-with-page-cache", self.environment):
                with open(os.path.join(scope, "memory.current"), encoding="utf-8") as handle:
                    return int(handle.read().strip())
        except (OSError, ValueError):
            return None
        return self.peaks.process_memory_bytes(scope) or None   # 0 是 scope 已收尾（目录不在）：同读不到

    def scope_blocked_processes(self, attempt):
        """这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数（本机的 Machine.io_blocked；文件头 ①「卡在 IO 上」）：数的是尝试目录 started 记的 runner 进程号
        连同它的子孙（照父进程号往下：单元里的与内存包装、里层包装各开的 scope 里的都在内）与外层 scope（尝试目录的 scope 文件记着）cgroup.procs 里的进程，
        各读 <proc_root>/<号>/stat 的状态字段；进程号与 scope 两样都读不到交 None，读不到状态的那几个进程跳过。"""
        _, started_path, scope_path = self.attempt_paths(attempt)
        members, readable = set(), False
        try:
            with open(started_path, encoding="utf-8") as handle:
                runner = int(handle.read().strip())
            members |= descendant_processes(runner, self.proc_root) | {runner}
            readable = True
        except (OSError, ValueError):
            pass
        try:
            with open(scope_path, encoding="utf-8") as handle:
                scope = handle.read().strip()
        except OSError:
            scope = ""
        listed = cgroup_processes(scope) if scope else None
        if listed is not None:
            members |= {int(word) for word in listed if word.isdigit()}
            readable = True
        if not readable:
            return None
        return sum(1 for member in members if process_run_state(member, self.proc_root) == "D")

    def log_content(self, attempt):
        """→ 日志里算「在涨」的那部分字节数（读不到交 -1）。两台同一段：整份的大小减去起法那一层打的进度行（MULTI_HOST_RUN_STEP、MULTI_HOST_RUN_PEER_CPU、
        MULTI_HOST_RUN_PEER_MEMORY、MULTI_HOST_RUN_PEER_MEMORY_STAT、MULTI_HOST_RUN_PEER_GPU、MULTI_HOST_RUN_PEER_IO_WAIT）与服务自己接的 lib.INBOX_LOG_MARK、lib.LONG_RUN_LOG_MARK 那几行；从上次读到的位置接着读整行，最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec 记进 log_offsets（第二台的 Machine.cpu_usec 读它），
        最近一行 MULTI_HOST_RUN_PEER_MEMORY 的此刻内存与读到过的峰值记进 peer_memory（第二台的 Machine.memory_now 读它），
        这一回读到的几行 MULTI_HOST_RUN_PEER_IO_WAIT 与 MULTI_HOST_RUN_PEER_THREADS 接在还没被读走的后面（peer_io_rounds、peer_thread_rounds；第二台的 Machine.io_blocked、
        thread_reading 读时并起来：merge_peer_io_rounds、merge_peer_thread_rounds），
        read=yes 的 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行记进 peer_memory_stat（弄坏开关 peak-memory-stat-read-at-exit 下记最后一行），
        以 lib.HEARTBEAT_LOG_MARK 起头的心跳行同样不算（弄坏开关 heartbeat-counted-as-log 下算）；以 lib.VERDICT_LIBRARY_PROGRESS_LOG_MARK 起头的判定库进度行算日志在涨
        （命令在等 prune-server 回话、没有别的输出时靠它不被判卡住；弄坏开关 verdict-library-progress-uncounted 下不算）；
        带 GPU_NOT_RUN_MARK 记号的「本次未跑」标记（gpu_not_run_test 认一行里每一处的整词，一处一条）在同一遍里数下、前 GPU_NOT_RUN_TESTS_SHOWN 条点名的用例记进 gpu_not_run_marks（算日志在涨）。
        同一次尝试同一时刻只一个线程在读（LOG_CONTENT_LOCKS 按尝试记录 id；排队线程判卡住与收尾线程都调它）：不会两个线程从同一个位置各读一遍、
        把同一段的 IO 等待与线程读数各接一回。弄坏开关 log-content-unlocked 下不拿锁。"""
        if lib.broken("log-content-unlocked", self.environment):
            return self.read_log_content(attempt)
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            return self.read_log_content(attempt)

    def read_log_content(self, attempt):
        """log_content 拿着那一次尝试的锁做的那一段（判法见 log_content）。"""
        path = attempt.get("log", "")
        try:
            size = os.path.getsize(path)
        except OSError:
            return -1
        offset, noise, cpu = self.log_offsets.get(attempt["id"], (0, 0, None))
        memory = self.peer_memory.get(attempt["id"], (None, None))
        stat_at_peak = self.peer_memory_stat.get(attempt["id"])
        blocked_rounds = []   # 这一回读到的每一行 MULTI_HOST_RUN_PEER_IO_WAIT：blocked，read=no 记 None
        thread_rounds = []   # 这一回读到的每一行 MULTI_HOST_RUN_PEER_THREADS：ThreadReading，read=no 记 None
        not_run_count, not_run_tests = self.gpu_not_run_marks.get(attempt["id"], (0, ()))
        not_run_tests = list(not_run_tests)
        # 用例「本次未跑」的标记行（gpu_not_run_test）：就在这一遍里逐行认，不另读一遍日志。弄坏开关 gpu-cases-not-run-unread 下不认
        marks_read = not lib.broken("gpu-cases-not-run-unread", self.environment)
        marks_broken = tuple(name for name in GPU_NOT_RUN_BREAK_SWITCHES if lib.broken(name, self.environment))
        read_at = time.time()
        # MULTI_HOST_RUN_PEER_GPU 是 multi-host-run.sh 每一轮照打的显存行：服务不读它（卡归 GPU 批处理验证服务），只不算日志在涨
        noise_marks = ("MULTI_HOST_RUN_STEP ", "MULTI_HOST_RUN_PEER_EXTERNAL_KILL ", "MULTI_HOST_RUN_PEER_GPU ", lib.INBOX_LOG_MARK + " ", lib.LONG_RUN_LOG_MARK + " ")
        if not lib.broken("log-noise-marks-incomplete", self.environment):
            # 第二台那一层收尾打的峰值与留着编译目录的几行同样不算日志在涨。弄坏开关 log-noise-marks-incomplete 下算
            noise_marks += tuple(mark + " " for mark in (lib.PEER_MEMORY_PEAK_MARK, lib.PEER_BUILD_PEAK_MARK, lib.PEER_BUILD_KEPT_MARK))
        if not lib.broken("heartbeat-counted-as-log", self.environment):
            noise_marks += (lib.HEARTBEAT_LOG_MARK + " ",)   # 命令自己打的心跳行：不算日志在涨
        if lib.broken("verdict-library-progress-uncounted", self.environment):
            noise_marks += (lib.VERDICT_LIBRARY_PROGRESS_LOG_MARK + " ",)   # 弄坏开关：判定库进度行也当噪声，只等判定库回话、没有别的输出的命令被当成日志不涨
        with contextlib.suppress(OSError), open(path, "rb") as handle:
            while size > offset:   # 一次至多读 LOG_SCAN_BYTES：服务重启之后头一回接着读的是整份日志
                handle.seek(offset)
                chunk = handle.read(min(size - offset, LOG_SCAN_BYTES))
                complete = chunk[:chunk.rfind(b"\n") + 1] or (chunk if len(chunk) == LOG_SCAN_BYTES else b"")   # 一整块都没有换行的照整块往前走
                if not complete:
                    break
                line_end = offset
                for line in complete.splitlines(keepends=True):
                    line_end += len(line)
                    text = line.decode("utf-8", "replace").strip()
                    found = PEER_CPU_LINE.match(text)
                    if found:
                        cpu = int(found.group(2)) if found.group(3) == "yes" else cpu
                    held = PEER_MEMORY_LINE.match(text)
                    if held:   # read=no：这一轮没读到，此刻的不算数、峰值照留
                        memory = (int(held.group(2)), max(int(held.group(3)), memory[1] or 0)) if held.group(4) == "yes" else (None, memory[1])
                    waited = PEER_IO_WAIT_LINE.match(text)
                    if waited:
                        blocked_rounds.append(int(waited.group(2)) if waited.group(3) == "yes" else None)
                    threaded = PEER_THREADS_LINE.match(text)
                    if threaded:
                        thread_rounds.append(ThreadReading(*(int(value) for value in threaded.group(1, 2, 3, 4)), threaded.group(5)) if threaded.group(6) == "yes" else None)
                    stated = PEER_MEMORY_STAT_LINE.match(text)
                    if stated and stated.group(8) == "yes" and (stat_at_peak is None or int(stated.group(2)) > stat_at_peak["current"]
                                                                or lib.broken("peak-memory-stat-read-at-exit", self.environment)):
                        stat_at_peak = dict(zip(("current", *lib.MEMORY_STAT_NAMES), (int(value) for value in stated.groups()[1:7])))
                    killed_line = PEER_EXTERNAL_KILL_LINE.match(text)
                    if killed_line:
                        self.peer_external_kill[attempt["id"]] = (killed_line.group(1), int(killed_line.group(2)), killed_line.group(3))
                    if found or held or waited or threaded or stated or text.startswith(noise_marks):   # 服务自己接的那几行（文件头 ⑥）与命令的心跳行同样不算在涨
                        noise += len(line)
                    for tested in (gpu_not_run_test(line, marks_broken) if marks_read else ()):   # 认原行（不 strip）：词界与颜色码由 gpu_not_run_test 判；一行几处记号数几条
                        not_run_count += 1
                        if len(not_run_tests) < GPU_NOT_RUN_TESTS_SHOWN:
                            not_run_tests.append(tested)
                offset += len(complete)
        self.log_offsets[attempt["id"]] = (offset, noise, cpu)
        if not_run_count:
            self.gpu_not_run_marks[attempt["id"]] = (not_run_count, not_run_tests)
        if memory != (None, None):
            self.peer_memory[attempt["id"]] = memory
        if stat_at_peak is not None:
            self.peer_memory_stat[attempt["id"]] = stat_at_peak
        # 读到的几行接在还没被读走的后面（peer_io_rounds、peer_thread_rounds；PeerMachine.io_blocked、thread_reading 读时并起来）。
        # 弄坏开关 peer-readings-overwritten 下换成这一回读到的（同一轮里第二回读日志把前一回读到的那几行盖掉）
        overwritten = lib.broken("peer-readings-overwritten", self.environment)
        for table, rounds in ((self.peer_io_rounds, blocked_rounds), (self.peer_thread_rounds, thread_rounds)):
            if rounds:
                table[attempt["id"]] = list(rounds) if overwritten else table.get(attempt["id"], []) + rounds
        return size - noise

    def merge_peer_io_rounds(self, attempt):
        """第二台这一次尝试上一次读之后接下来的几行 MULTI_HOST_RUN_PEER_IO_WAIT 并起来（PeerMachine.io_blocked 调）：取最少的（有一轮没有 D 状态的进程或读不到，
        就不算每一轮都在等 IO；有一行 read=no 记 None）记进 peer_io_wait、清掉接着的那几行；一行都没接到的不动。→ peer_io_wait 里那一份。"""
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            rounds = self.peer_io_rounds.pop(attempt.get("id"), None)
            if rounds:
                self.peer_io_wait[attempt["id"]] = None if None in rounds else min(rounds)
            return self.peer_io_wait.get(attempt.get("id"))

    def merge_peer_thread_rounds(self, attempt):
        """第二台这一次尝试上一次读之后接下来的几行 MULTI_HOST_RUN_PEER_THREADS 并起来（PeerMachine.thread_reading 调）：读到了的取最大（文件头 ②「CPU 闸」的可跑线程数：
        段内取最大；采样号、可跑的各取最大，占线程最多的进程取总数最大那一行的；都是 read=no 记 None）记进 peer_threads、清掉接着的那几行；一行都没接到的不动。"""
        with keyed_lock(LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, attempt.get("id")):
            rounds = self.peer_thread_rounds.pop(attempt.get("id"), None)
            if rounds:
                readable = [reading for reading in rounds if reading is not None]
                busiest = max(readable, key=lambda reading: reading.total) if readable else None
                self.peer_threads[attempt["id"]] = None if busiest is None else ThreadReading(
                    max(reading.sample for reading in readable), max(reading.runnable for reading in readable), busiest.total, busiest.top_threads, busiest.top_name)
            return self.peer_threads.get(attempt.get("id"))

    def signal_note(self, task, attempt, result, exit_code):
        """这一次被信号终止时给人看的那一句（文件头 ①「收尾」的信号）：尝试单元自己收到的停的信号（attempt_runner 记的 stopped_signal，它转给了命令）、
        或命令的退出码是 128 + 信号号（SIGNAL_EXIT_CODES），写出信号名，再写服务这一次有没有发过停的信号、是哪一步发的（service_stop_step）；不是信号交空串。
        弄坏开关 signal-exit-unnamed 下一律交空串。"""
        if lib.broken("signal-exit-unnamed", self.environment):
            return ""
        received = int((result or {}).get("stopped_signal") or 0)
        if received:
            head = f"尝试单元收到 {signal_name(received)}（attempt_runner 记的 stopped_signal），转给了命令"
        elif isinstance(exit_code, int) and exit_code in SIGNAL_EXIT_CODES:
            head = f"命令被 {signal_name(exit_code - 128)} 终止（退出码 {exit_code}）"
        else:
            return ""
        step = self.service_stop_step(task, attempt)
        return f"{head}；{'服务这一次发过停的信号：' + step if step else '服务这一次没有发过停的信号'}"

    def service_stop_step(self, task, attempt):
        """服务停这一次尝试是哪一步（signal_note 用）：判卡住、四小时硬上限、取消各照记下的那一样；都没记而 stop_running 对它发过停的写 stop_running；
        都没有交空串。"""
        for marked, step in ((attempt.get("stalled"), "判卡住（judge_stall_stop）"),
                             (attempt.get("run_hard_limit_stop"), "四小时硬上限（judge_run_hard_limit）"), (task.get("cancel_requested"), "取消（task cancel 或撤下这一片）"),
                             (attempt.get("id") in self.service_stopped_attempts, "停尝试的单元（stop_running）")):
            if marked:
                return step
        return ""

    def external_kill_note(self, task, attempt, result, outcome):
        """这一次尝试是不是被不是服务发的 TERM / KILL 杀掉的（文件头 ① 与 task_server_lib.py 文件头「错误类型」killed-externally）→ 说明一句，不是交空串。
        只在照退出码记的结局是 command-failed、服务这一次没有发过停的信号（service_stop_step）、单元自己也没收到停的信号（result 的 stopped_signal）时判；
        这一次尝试进程树里见过的进程号（Machine.process_ids_seen；第二台的进程号服务这一侧拿不到，走 peer_external_kill_note）里有一个在这一台同一时段（尝试开跑前几秒到结束后 EXTERNAL_KILL_LOG_AFTER_SECONDS 秒）
        的 earlyoom 日志（journalctl -t earlyoom）或内核日志（journalctl -k）里被点名杀了的，说明写那一行与进程号；日志读不到、没有对上的交空串，仍记 command-failed，不猜。
        自证里没换假 journalctl（TASK_SERVER_JOURNALCTL）的世界不读真日志。弄坏开关 external-kill-unrecognized 下一律交空串。"""
        environment = self.environment
        if outcome != "command-failed" or lib.broken("external-kill-unrecognized", environment) or (result or {}).get("stopped_signal"):
            return ""
        if self.service_stop_step(task, attempt):
            return ""
        if attempt.get("host") == PEER:
            return self.peer_external_kill_note(attempt)
        if lib.in_selftest(environment) and not environment.get("TASK_SERVER_JOURNALCTL"):
            return ""
        machine = self.machines.get(attempt.get("host"))
        process_ids = machine.process_ids_seen(attempt, result) if machine is not None else None
        if not process_ids:
            return ""
        ended = float((result or {}).get("ended_at") or time.time())
        since = float(attempt.get("started_at") or (result or {}).get("started_at") or ended) - 5.0
        for source, line in self.kill_log_lines(since, ended + EXTERNAL_KILL_LOG_AFTER_SECONDS):
            killed = lib.external_kill_pid(line, environment)
            if killed in process_ids:
                return (f"命令被不是服务发的信号杀掉：这一台的 {source} 日志里「{line.strip()[:200]}」点名的进程 {killed} 是这一次尝试进程树里的"
                        f"（attempt_runner 每轮采样记下的 {len(process_ids)} 个进程号之一）；服务这一次没有发过停的信号")
        return ""

    def peer_external_kill_note(self, attempt):
        """第二台的尝试被外部信号杀掉的说明（external_kill_note 的第二台那一路）：服务这一侧拿不到第二台的进程号，认的是 multi-host-run.sh 在第二台那一侧记下这一趟进程树的进程号、
        命令退非 0 之后读第二台的 earlyoom 与内核日志、点名这些进程号之一被杀时打的 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行（log_content 记进 peer_external_kill）；
        日志读到底再看，没有这一行交空串（仍是 command-failed）；那一行记的是内核「Memory cgroup out of memory」（lib.memory_cgroup_kill_line：某个内存组到了它自己的上限，
        不是整机内存紧张）的同样交空串，照原来的路判。multi-host-run.sh 那一侧不分这两种，服务这一侧按 line= 的内容分。
        弄坏开关 peer-external-kill-line-ignored 下不认这一行，memory-cgroup-kill-counted 下内存组那一种也认。"""
        self.log_content(attempt)
        found = self.peer_external_kill.get(attempt["id"])
        if found is None or lib.broken("peer-external-kill-line-ignored", self.environment):
            return ""
        source, killed, line = found
        if lib.memory_cgroup_kill_line(line, self.environment):   # 某个内存组到了它自己的上限（「Memory cgroup out of memory」）：不是外部，照原来的路判
            return ""
        return (f"命令被不是服务发的信号杀掉：第二台的 {source} 日志里「{line.strip()[:200]}」点名的进程 {killed} 是这一趟进程树里的"
                f"（multi-host-run.sh 在第二台那一侧记下的进程号，命令退非 0 之后读第二台的日志打了 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行）；服务这一次没有发过停的信号")

    def kill_log_lines(self, since, until):
        """[(来源, 一行)]：本机 since 到 until（Unix 秒）之间 earlyoom（journalctl -t earlyoom）与内核（journalctl -k）打的日志，只留 lib.external_kill_pid 认得的杀进程那种行；
        某一路起不来、超时、退非 0（没权限读这份日志这类）的跳过那一路。"""
        found = []
        for source, selector in (("earlyoom", ["-t", "earlyoom"]), ("内核", ["-k"])):
            try:
                completed = subprocess.run([self.journalctl, "--no-pager", "-q", "-o", "cat", f"--since=@{int(since)}", f"--until=@{int(until) + 1}", *selector],
                                           capture_output=True, text=True, errors="replace", env=self.environment, stdin=subprocess.DEVNULL, timeout=EXTERNAL_KILL_LOG_SECONDS)
            except (OSError, subprocess.SubprocessError):
                continue
            if completed.returncode == 0:
                found += [(source, line) for line in completed.stdout.splitlines() if lib.external_kill_pid(line, self.environment) is not None]
        return found

    def finish_attempt(self, task, attempt, result):
        # 有结果不等于收干净。保持 starting/running，现有资源账会继续保留这次占用。
        if attempt.get("cleanup_pending") and time.time() < attempt.get("cleanup_retry_at", 0):
            return
        if attempt.get("host") == LOCAL and not attempt.get("cleanup_pending"):   # 本机的尝试：单元已停（读得到、cgroup.procs 空）收尾慢也不占 CPU 与内存的账；第二台的由取消回读放
            self.release_stopped_booking(attempt)
        self.selftest_pause("attempt-finishing")
        found = self.store.task(task["id"])   # 现读：取消那一侧改过 cancel_requested（不拿对账时读的那一份）
        if found is not None and found[1]:
            task = found[0]
        exit_code = result.get("exit_code") if result else None
        cancelled = bool(task.get("cancel_requested")) and (result is None or bool(result.get("stopped_signal")))
        tail = log_tail(attempt.get("log", ""))
        machine = self.machines[attempt["host"]]
        exit_code, judged_host, runner_exit = machine.command_exit(exit_code, tail)   # 起法那一层打进日志的命令自己的退出码（Machine.command_exit）
        if runner_exit is not None:
            attempt["peer_runner_exit_code"] = runner_exit
        outcome = lib.classify_attempt(exit_code, judged_host, tail, cancelled, self.environment)
        if attempt.get("stalled") and not cancelled and (result is None or result.get("stopped_signal")):
            outcome = "memory-cap-hit" if attempt.get("stalled_at_cap") else "stalled"   # 顶在上限上卡住的按撞上限办（judge_stall_stop）
        if (attempt.get("run_hard_limit_stop") and not cancelled and (result is None or result.get("stopped_signal"))
                and not lib.broken("run-hard-limit-outcome-unmarked", self.environment)):
            outcome = lib.RUN_HARD_LIMIT_OUTCOME   # 照四小时硬上限判停的（judge_run_hard_limit）：settle_attempt 不重排，照结局收尾
        signal_note = self.signal_note(task, attempt, result, exit_code)
        if signal_note:   # 被信号终止的：结局原因句接这一句（task_server_lib.reason_fields），服务日志另记一行
            attempt["signal_note"] = signal_note
            say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试：{signal_note}")
        kill_note = self.external_kill_note(task, attempt, result, outcome)
        if kill_note:   # 被不是服务发的 TERM / KILL 杀掉、earlyoom 或内核日志里有它的进程号：单独一种结局，照 retry_decision 重排（task_server_lib.py 文件头「错误类型」killed-externally）
            outcome = lib.EXTERNAL_KILL_OUTCOME
            attempt["external_kill_note"] = kill_note
            say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次尝试：{kill_note}")
        self.log_content(attempt)   # 把日志读到底：第二台每轮那几行最后一轮之后才到的照算（peer_memory_stat）
        peak_segment = self.finish_peak_segment_cores(attempt, machine, time.time())   # 这一次的峰值段核数（文件头 ②「CPU 闸」）：记 ended 时写进尝试
        gate_threads = self.finish_cpu_gate_threads(attempt, peak_segment)   # 连同可跑线程数的 CPU 闸线程数（文件头 ②「CPU 闸」）：记 ended 时写进尝试，记录读它
        # 脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认
        if outcome == "memory-cap-hit" and not lib.broken("cap-hit-stat-unreported", self.environment):
            # 内存最高那一刻的明细（文件头 ①「内存最高那一刻」）：抄进结局说明，分得清是堆还是页缓存
            attempt["peak_memory_note"] = lib.peak_memory_note(machine.memory_stat_at_peak(attempt, result))
        if outcome == "squeezed-out":   # 被谁挤掉：同一台此刻在跑的别的尝试（打回的理由 by= 那一栏，task_server_lib.reason_fields）
            attempt["squeezed_by"] = "、".join(f"任务 {other.get('task_id')}（上限 {other.get('memory_cap_text')}）"
                                              for other in self.store.running_attempts()
                                              if other.get("host") == attempt.get("host") and other.get("id") != attempt.get("id"))
        # 钉了的命令退 254 记被挤掉、照重排办（文件头 ②「内存」的钉值；任务 10289 钉值等于 slice 总上限，内存包装退 254 写「这一条的结果不算数，重跑」）。弄坏开关 pinned-squeeze-as-cap-hit 下照撞上限办
        squeezed_pinned = attempt.get("prediction") == lib.PINNED_HISTORY and not lib.broken("pinned-squeeze-as-cap-hit", self.environment)
        if outcome == "squeezed-out" and not squeezed_pinned and not lib.broken("squeeze-at-total-not-cap-hit", self.environment):
            # 这一条的上限已经到了那台上限收到的那个数（总上限减那台此刻的余量）：slice 满就是它自己撞了上限（实测：上限与总上限都是 50G 的那一条被内核在 slice 那一层杀，记成被挤掉、原样重排了三次）
            total = self.usable_maxima(self.resources.snapshot()).get(attempt.get("host", ""))   # 上限收到的那个数（booked_at_cap）；余量量不到的是 None，不改判
            if total and int(attempt.get("memory_cap") or 0) >= total:
                outcome = "memory-cap-hit"
        peer_clean, peer_note = self.confirm_peer_cleanup(attempt)
        local_clean, leftover = self.stop_leftovers(attempt, result)
        if leftover:
            attempt["leftover_note"] = leftover
        elif local_clean and attempt.get("cleanup_pending"):
            attempt["leftover_note"] = "清理重查已确认，本机进程已收干净"
        if not peer_clean or not local_clean:
            note = "；".join(part for part in (peer_note, leftover) if part)
            changed = attempt.get("cleanup_pending") != note
            attempt.update(cleanup_pending=note, cleanup_retry_at=time.time() + CLEANUP_RETRY_SECONDS)
            self.unit_stopped.discard(attempt["id"])   # 清理没确认：资源占用保留，在跑记账照数
            self.store.update("attempts", attempt)
            if changed:
                say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} status=pending detail={note}")
                self.alert("cleanup-pending", task["id"], f"任务 {task['id']} 清理尚未确认，保留资源占用：{note}",
                           "服务会重查；检查该尝试日志与远端连接。不要删进程号、退出码或清理状态文件来放行")
            return
        attempt.pop("cleanup_pending", None)
        attempt.pop("cleanup_retry_at", None)
        say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} status=confirmed")
        # 上面 log_content 已读到底：日志里「本次未跑」的标记行记进尝试（结局说明与 gpu-cases-not-run 读它）；在放掉读到哪（log_offsets）之前取走，
        # 别的线程此刻再读只会从读到的位置接着读，不会从头再数一遍
        self.record_gpu_not_run_marks(attempt)
        self.log_growth.pop(attempt.get("id"), None)
        self.log_offsets.pop(attempt.get("id"), None)
        self.hard_limit_stops.discard(attempt.get("id"))
        self.peer_memory.pop(attempt.get("id"), None)
        self.peer_io_wait.pop(attempt.get("id"), None)
        self.peer_threads.pop(attempt.get("id"), None)
        for table in (self.peer_io_rounds, self.peer_thread_rounds):
            table.pop(attempt.get("id"), None)
        self.thread_overruns.pop(attempt.get("id"), None)
        self.io_waits.pop(attempt.get("id"), None)
        self.activity_windows.pop(attempt.get("id"), None)
        for table in (self.progress_read_at, self.progress_files, self.progress_noted):
            table.pop(attempt.get("id"), None)
        self.service_stopped_attempts.pop(attempt.get("id"), None)
        self.running_threads_cache.pop(attempt.get("id"), None)
        self.unit_stopped.discard(attempt.get("id"))
        with self.running_usage_lock:
            self.running_peak_samples.pop(attempt.get("id"), None)
            self.running_peak_cores.pop(attempt.get("id"), None)
            self.running_peak_runnable.pop(attempt.get("id"), None)
            self.running_latest_cores.pop(attempt.get("id"), None)
            self.running_latest_runnable.pop(attempt.get("id"), None)
        self.cpu_windows.pop(attempt.get("id"), None)   # suspect_reported 不删：wait 那一侧可能刚读到这一次还在跑，删了会把报过的那一段再报一遍
        attempt.update(state="ended", exit_code=exit_code, outcome=outcome, ended_at=round((result or {}).get("ended_at") or time.time(), 3),
                       peak_metric=lib.PROCESS_MEMORY_PEAK_METRIC)   # 峰值口径一栏（task_server_lib.py 文件头「资源记录」）：这一版记的都是进程内存
        attempt[PEAK_SEGMENT_CORES_COLUMN] = peak_segment
        attempt[CPU_GATE_THREADS_COLUMN] = gate_threads
        attempt["seconds"] = round(attempt["ended_at"] - attempt.get("started_at", attempt["ended_at"]), 3)
        if result and not lib.broken("resources-unrecorded", self.environment):
            for name in ("memory_peak", "cpu_usec", "cores"):
                attempt[name] = result.get(name)
            attempt["build_peak"] = result.get("build_directory_peak")
            if result.get("seconds") is not None:
                attempt["seconds"] = result["seconds"]
        if not lib.broken("resources-unrecorded", self.environment):
            # 起法那一层打进日志的峰值（Machine.ended_peaks：第二台是 multi-host-run.sh 打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK）：
            # attempt_runner 量不到的那一项照它记
            for name, value in machine.ended_peaks(attempt, tail).items():
                if attempt.get(name) is None:
                    attempt[name] = value
            self.peer_memory_stat.pop(attempt.get("id"), None)
            self.peer_external_kill.pop(attempt.get("id"), None)
        if attempt.get("build_slot"):
            # 组里这一个记下这一次量到的占盘（编译目录占盘那一判照它扣）；第二台那一个留没留着照起法那一层报的（multi-host-run.sh 带 --keep-build-directory 跑完打的
            # PEER_BUILD_KEPT_MARK）：报了的组里记成可接着用，没报的（中途失败、被停，⑥ 连它一起删了）组里删掉这一个（task_server_lib.py 文件头「编译目录」）
            kept = True
            if machine.name != LOCAL:
                kept_label = machine.kept_build_label(tail)
                attempt["build_directory_kept"] = kept = bool(kept_label) and os.path.basename(attempt.get("build_directory") or "") == kept_label
            self.settle_build_slot(attempt["build_slot"], kept, attempt.get("build_peak"))
        if attempt.get("memory_peak") is None and lib.broken("unmeasured-recorded-as-zero", self.environment):
            attempt["memory_peak"] = 0   # 弄坏开关：量不到的记成 0
        self.store.update("attempts", attempt)
        say(f"TASK_RESOURCES_RELEASED time={time.time():.6f} attempt={attempt['id']} outcome={outcome}")
        self.resources.invalidate()
        self.unit_checked_at.pop(attempt["id"], None)
        self.unit_gone_since.pop(attempt["id"], None)
        self.forget_unit_unreadable(attempt["id"])
        try:
            self.settle_attempt(task, attempt, result)
        except Exception as error:   # 尝试已记成 ended：这里出错不能留下「任务 running、尝试 ended」的孤儿，告警、下一轮对账 settle_ended_attempts 补收尾
            say(f"  ✗ 任务 {task['id']} 第 {attempt['number']} 次尝试的结局 {outcome} 已记，定任务去向时出错（{type(error).__name__}: {error}）：下一轮对账补收尾")
            self.alert("settle-failed", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试结局 {outcome} 已记、定去向时出错：{type(error).__name__}: {error}",
                       "服务下一轮对账会补收尾（settle_ended_attempts）；还 running 就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def labelled_scope(self, attempt):
        """本机 slice 下这次尝试的 scope；确认没有交空串，无法查询交 None。"""
        label = attempt.get("label") or ""
        if not label:
            return ""
        directory = self.peaks.slice_directory_of(self.environment.get("RUN_WITH_MEMORY_CAP_SLICE") or self.peaks.DEFAULT_SLICE,
                                                  self.environment.get("RUN_WITH_MEMORY_CAP_CGROUP_ROOT") or "/sys/fs/cgroup")
        if directory is None:
            return None
        if directory:
            try:
                with os.scandir(directory) as entries:
                    list(entries)
            except FileNotFoundError:
                return ""
            except OSError:
                return None
        for name, entry in self.peaks.slice_scopes(directory).items():
            if entry.get(self.peaks.SCOPE_LABEL_KEY) == label and name.endswith(".scope"):
                return os.path.join(directory, name)
        return ""

    def confirm_peer_cleanup(self, attempt):
        """新尝试的远端清理回执。失联、包装被杀或回执读不了都保留占用；恢复连接后重试收尾。
        同一次尝试同一时刻只起一趟 multi-host-run.sh --recover-cleanup（对账与停单元的后台线程、取消那一侧都会来确认）：按尝试记录 id 拿一把锁
        （锁表 PEER_CLEANUP_LOCKS，放下就删），后到的等前一趟放下、再读回执判，读到 confirmed 就不再起。
        弄坏开关 peer-cleanup-recovered-concurrently 下不拿锁，几处同时读到 pending 各起一趟、各往日志追加一份。"""
        path = attempt.get("peer_cleanup_file")
        if not path:   # 升级前的尝试没有此协议；不能凭空推断它写过新回执。
            return True, ""
        if lib.broken("peer-cleanup-recovered-concurrently", self.environment):
            return self.peer_cleanup_confirmed(attempt, path)
        with keyed_lock(PEER_CLEANUP_LOCKS, PEER_CLEANUP_LOCKS_GUARD, attempt.get("id")):
            return self.peer_cleanup_confirmed(attempt, path)

    def peer_cleanup_confirmed(self, attempt, path):
        """confirm_peer_cleanup 拿着那一次尝试的锁做的那一段：读回执，pending 的起一趟 --recover-cleanup、输出追加进那一次的日志，再读回执判。→ (确认了没有, 没确认的那一句)。"""
        record = read_json_file(path)
        if record and record.get("status") in ("not-started", "confirmed"):
            return True, ""
        if record and record.get("status") == "pending":
            try:
                completed = subprocess.run(["bash", self.peer_runner, "--recover-cleanup", path], capture_output=True, text=True,
                                           env=lib.ssh_multiplexed_environment(self.environment, self.environment),   # 服务起的 ssh 复用连接（task_server_lib.py 文件头「主机」）
                                           stdin=subprocess.DEVNULL, timeout=60)
                # 不凭退出码放行：回执必须原子写成 confirmed。
                with open(attempt["log"], "a", encoding="utf-8") as handle:
                    handle.write(completed.stdout + completed.stderr)
            except (OSError, subprocess.TimeoutExpired):
                pass
            record = read_json_file(path)
            if record and record.get("status") == "confirmed":
                return True, ""
        leftovers = [str(line)[:200] for line in ((record or {}).get("scope_leftovers") or [])][:PEER_SCOPE_LEFTOVERS_SHOWN]
        if leftovers:   # multi-host-run.sh 停了第二台这一趟内存包装的 scope 还留着进程（它文件头 ⑥）：说明写留下的进程，提醒换成核脚本的那一句
            return False, f"{PEER_SCOPE_LEFTOVERS_MARK}{'；'.join(leftovers)}（清理回执 {path}）"
        return False, f"远端清理回执未确认（{path}），尚不能放掉这一次的资源占用"

    def stop_leftovers(self, attempt, result):
        """尝试正常结束（结果已写、没被停）之后留下的进程（文件头 ①「留下的进程」）：单元里的由 attempt_runner 退出之前自己停掉、回读，记在 result.json 的
        unit_leftovers（它一退 systemd 就收单元、当场杀掉剩下的，服务那时回读不到）；外层 scope（内存包装开的，单元收尾时不跟着停）由服务回读，
        还有进程就发一次 systemctl --user kill --signal=SIGKILL（只对这一次自己的 scope 名），至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup.procs 空了没有。
        → (确认清空, 说明)。未知与仍有进程都交 False，后续对账重查。"""
        if lib.broken("leftover-processes-kept", self.environment) or not attempt.get("unit"):
            return True, ""
        if result is None and attempt.get("unit_unreadable_gone"):   # 读不到单元状态时不经 systemd 核过这一次的进程都不在（unit_unreadable_gone）：没有要停的
            return True, f"单元状态读不到，不经 systemd 核过这一次的进程都不在：{attempt['unit_unreadable_gone']}"
        found, remaining = {}, {}
        unit_leftovers = (result or {}).get("unit_leftovers") or {}
        if unit_leftovers.get("processes"):
            found[attempt["unit"]] = [str(process) for process in unit_leftovers["processes"]]
        # runner 的回读只是当时的快照；还剩或读不了时重新查 cgroup，不用旧 PID 杀可能复用的进程。
        if not unit_leftovers.get("verified") or unit_leftovers.get("remaining"):
            left = unit_processes(attempt["unit"], self.environment)
            if left is None:
                return False, f"单元 {attempt['unit']} 的进程读不了"
            runner_pid = str((result or {}).get("runner_pid") or "")
            left = [pid for pid in left if pid != runner_pid]
            if left:
                found[attempt["unit"]] = left
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", attempt["unit"]],
                                   capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                current = unit_processes(attempt["unit"], self.environment)
                if current is None:
                    return False, f"停单元 {attempt['unit']} 后无法回读"
                remaining[attempt["unit"]] = [pid for pid in current if pid != runner_pid]
        scope = str((result or {}).get("scope") or "") or self.labelled_scope(attempt)   # 命令比一轮采样还短时按标签找
        if scope is None:
            return False, "无法查询本机 scope，保留资源占用"
        if scope.startswith("/sys/fs/cgroup/") and scope.endswith(".scope"):
            name = os.path.basename(scope)
            left = cgroup_processes(scope)
            if left is None:
                return False, f"scope {name} 的进程读不了"
            if left:
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", name], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                deadline = time.monotonic() + LEFTOVER_STOP_SECONDS
                still_there = cgroup_processes(scope)
                while still_there and time.monotonic() <= deadline:
                    time.sleep(0.2)
                    still_there = cgroup_processes(scope)
                if still_there is None:
                    return False, f"停 scope {name} 后无法回读"
                found[name], remaining[name] = left, still_there
        if not found:
            return True, ""
        counted = "、".join(f"{name} 里 {len(left)} 个（进程号 {' '.join(left[:8])}{' …' if len(left) > 8 else ''}）" for name, left in found.items())
        still = "、".join(f"{name} 还剩 {len(left)} 个" for name, left in remaining.items() if left)
        text = (f"命令退出之后还留着进程：{counted}（命令起的孙进程没收干净）"
                + (f"，回读 {still}，保留资源占用" if still else "，服务已停掉，回读 cgroup 空了"))
        say(f"  · 第 {attempt.get('number')} 次尝试{text}")
        return not bool(still), text

    # ── 收尾不在排队线程里做（文件头 ①「收尾不在排队线程里做」）──
    def finish_off_loop(self, task, attempt, result):
        """收尾一次尝试（finish_attempt）放到后台线程里做：第二台的清理回执（--recover-cleanup 经 ssh，至多 60 秒）、
        留下的进程回读（至多 LEFTOVER_STOP_SECONDS）、记结局时删编译目录（第二台 --remove-build-directory）都在那个线程里，
        排队线程照常对账别的尝试、派活。同一次尝试同一时刻只一个线程在做（finishing，按尝试记录 id；对账与补收尾见到它就跳过），做完放下、叫醒排队。
        弄坏开关 finish-in-scheduler 下照在排队线程里当场做。"""
        if attempt.get("cleanup_pending") and time.time() < attempt.get("cleanup_retry_at", 0):
            return   # 清理没确认的还没到重查时刻（finish_attempt 开头同一判）：不起线程

        def work():
            self.finish_attempt(task, attempt, result)
        if lib.broken("finish-in-scheduler", self.environment):
            work()
            return
        with self.finishing_lock:
            if attempt["id"] in self.finishing:
                return
            self.finishing.add(attempt["id"])
        threading.Thread(target=self.finish_in_background, args=(attempt["id"], work), name=f"task-server-finish-{attempt['id']}", daemon=True).start()

    def finish_in_background(self, attempt_id, work):
        """finish_off_loop 起的那个线程：做 work，出事打出来（尝试没记 ended 的下一轮对账再判，记了 ended 而任务没收尾的由 settle_ended_attempts 补），放下、叫醒排队。"""
        try:
            work()
        except Exception:
            say(f"  ! 尝试（行 {attempt_id}）收尾出了异常（下一轮对账再判）：\n" + traceback.format_exc())
        finally:
            with self.finishing_lock:
                self.finishing.discard(attempt_id)
            self.wake()

    def finishing_now(self, attempt_id):
        """这一次尝试的收尾此刻在后台线程里做着没有（finish_off_loop）。"""
        with self.finishing_lock:
            return attempt_id in self.finishing

    def finished_preface(self, task, attempt, outcome):
        """收尾那一刻结局说明前面接的几句（文件头 ①「留下的进程」与 task_server_lib.py 文件头「续跑」）：留下的进程停掉了的那一句；
        结局不是 succeeded、这一次跑超过 lib.LONG_RUN_SECONDS、命令又没登记续跑（lib.resumable_command）的另写「这条跑了 N 分钟、没有续跑：重交会从头跑」。
        弄坏开关 long-run-not-noted 下不写这一句。
        这一次的日志里有用例「本次未跑」标记行的（文件头 ①「要处置的提醒」的 gpu-cases-not-run）另写那一段（gpu_not_run_text），结局种类与退出码不动。"""
        parts = [attempt["leftover_note"]] if attempt.get("leftover_note") else []
        seconds = float(attempt.get("seconds") or 0)
        limit = float(lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_SECONDS", self.environment) or lib.LONG_RUN_SECONDS)
        if (outcome not in ("succeeded", "cancelled") and seconds > limit and not lib.broken("long-run-not-noted", self.environment)
                and not lib.resumable_command(task.get("command") or [], task.get("cwd"), self.environment)):
            parts.append(f"这条跑了 {seconds / 60:.0f} 分钟、没有续跑：重交会从头跑")
        if attempt.get(NOT_RUN_MARKS_COLUMN) and outcome != "cancelled":   # 日志里有用例「本次未跑」的标记行（record_gpu_not_run_marks 记的）：同时推 gpu-cases-not-run
            parts.append(gpu_not_run_text(attempt[NOT_RUN_MARKS_COLUMN]))
        return "".join(part + "；" for part in parts)

    def record_gpu_not_run_marks(self, attempt):
        """把 log_content 读到的用例「本次未跑」标记行（gpu_not_run_marks 里这一次尝试那一份）记进尝试的 NOT_RUN_MARKS_COLUMN、放掉内存里那一份
        （finish_attempt 记 ended 之前调，日志已读到底）；没有标记行的不记这一栏。"""
        found = self.gpu_not_run_marks.pop(attempt.get("id"), None)
        if found:
            attempt[NOT_RUN_MARKS_COLUMN] = {"count": found[0], "tests": list(found[1])}

    def settle_attempt(self, task, attempt, result=None):
        """尝试的结局已记进库（state=ended、outcome）之后定这条任务的去向（文件头「重试」）：告警、取消、钉值与改工作区的不重排、retry_decision 重排或收尾。
        与记尝试分开写：这一步半途出错时尝试已是 ended、任务还 running，由 reconcile 的 settle_ended_attempts 补收尾，不留孤儿；补收尾时 result 是 None。"""
        outcome, exit_code = attempt.get("outcome", ""), attempt.get("exit_code")
        if outcome == "peer-run-failed":   # 第二台那一趟没跑成：写告警、/health 的 peer 一栏记最近一次（本文件头 ②「机器」）
            self.note_peer_run_failure(task, attempt)
        if outcome == lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME:   # 命令退 75：起一次 prune-server/control check，不通写告警（只报，不起、不停它）
            self.check_prune_server(task, attempt)
        if outcome == "cancelled":
            self.finalize(task, outcome, exit_code, "提交它的会话取消、单元已停")
            return
        counts = dict(task.get("outcome_counts") or {})
        counts[outcome] = counts.get(outcome, 0) + 1
        if outcome == lib.RUN_HARD_LIMIT_OUTCOME:
            self.settle_run_hard_limit(task, attempt, exit_code, counts)
            return
        # 结局与重排的说明前面写明（task status 的说明一行看得见）：顶在上限上卡住的那一次（judge_stall_stop）
        at_cap = f"{attempt.get('stall_note')}；" if (outcome == "memory-cap-hit" and attempt.get("stalled_at_cap")) or (outcome == "stalled" and attempt.get("stall_note")) else ""   # 判卡住的依据写进说明，不再是「没有说明」
        if outcome == "memory-cap-hit" and attempt.get("peak_memory_note"):   # 内存最高那一刻的明细（finish_attempt 记的）：重排与打回的说明同样写
            at_cap += f"{attempt['peak_memory_note']}；"
        leftover = f"{attempt['leftover_note']}；" if attempt.get("leftover_note") else ""   # 命令退出之后留下的进程停掉了（stop_leftovers）：重排的说明同样写
        outcome_preface = at_cap
        if outcome == "memory-cap-hit" and attempt.get("prediction") == lib.PINNED_HISTORY and not lib.broken("pinned-cap-grown", self.environment):
            # 钉值撞了上限不放大（文件头 ②「内存」的钉值）：上限就是配置的总上限，到顶照 memory-cap-at-maximum 打回
            self.finalize(task, "memory-cap-at-maximum", exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, 'memory-cap-at-maximum')}钉值（多机配置的 {lib.PIN_SOURCE}）{attempt.get('memory_cap_text', '')} 也撞了上限：钉值不放大，打回",
                          outcome_counts=counts, maximum_host=attempt.get("host", ""), maximum_cap_text=attempt.get("memory_cap_text", ""))
            return
        if outcome in WORKSPACE_WRITER_NO_RETRY_OUTCOMES and self.modifies_workspace(task) and not lib.broken("workspace-writers-retried", self.environment):
            # 改主工作区的命令（换钩子、打补丁、暂存）跑第二遍结果就不同：凡是会重排的结局一律不重排，打回提交方自己改了重交，不绕过
            #（用户：「改主工作区的命令撞上限不再原样重排、打回让提交方修改 后提交，不能绕过」）
            self.finalize(task, outcome, exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, outcome)}这条命令改主工作区，跑第二遍结果就变（补丁已打上的再打会被拒），服务不重排：改好再重交，"
                          f"或先在副本里跑；下次这条命令照这把键近 {RECORD_WINDOW} 次最大起记，撞过的这一次按 {attempt.get('memory_cap_text', '')} 算、不放大", outcome_counts=counts)
            return
        maxima = self.usable_maxima(self.resources.snapshot()) if outcome == "memory-cap-hit" else {}   # 乘 1.5 照上限收到的那个数比
        fixed_host = PEER if self.peer_only_entry(task) else ""
        if fixed_host:   # 弄坏开关 host-lists-restored：照改前只派第二台的入口不换台
            maxima = {fixed_host: maxima.get(fixed_host)} if maxima else {}
        decision = lib.retry_decision(counts, outcome, int(attempt.get("memory_cap") or 0), attempt.get("host", ""), task.get("next_host", "") or fixed_host,
                                      maxima, self.environment)
        if not decision.retry:
            maximum = ({"maximum_host": attempt.get("host", ""), "maximum_cap_text": attempt.get("memory_cap_text", "")}
                       if decision.outcome == "memory-cap-at-maximum" else {})
            self.finalize(task, decision.outcome, exit_code, (outcome_preface + self.finished_preface(task, attempt, decision.outcome) + decision.why).rstrip("；"),
                          outcome_counts=counts, **maximum)
            return
        # 这一次的编译目录还在组里：下一次起在同一台、它空着就接着用它（start_attempt 的 take_build_slot；task_server_lib.py 文件头「编译目录」）
        number = int(attempt.get("number") or 0)
        try:   # 重排在一个事务里判：任务还 running、这一次是它最新那一次、没被取消（取消那一侧改过的不拿旧字典盖掉）
            task = self.store.change("tasks", task["id"], dict(state="queued", next_cap=decision.cap_bytes, next_host=decision.host or "",
                                                                note=at_cap + leftover + decision.why, outcome_counts=counts),
                                     guard=lambda current: ("cancelled" if current.get("cancel_requested") else
                                                            "" if current.get("state") == "running" and int(current.get("attempt_count") or 0) == number else "state"))
        except StoreRefused as refused:
            if refused.kind == "guard" and str(refused) == "cancelled":
                self.finalize(task, "cancelled", exit_code, f"第 {number} 次尝试 {outcome}，重排之前提交它的会话取消了：不再起", outcome_counts=counts)
            elif refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            return
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试 {outcome}：{at_cap}{decision.why}，上限 {lib.mebibytes_text(decision.cap_bytes)} 重排")
        if task.get("role") == "shard" and lib.broken("shard-retry-restarts-all", self.environment):
            for sibling in self.children_of(task["parent_id"]):   # 弄坏开关：成了的兄弟片也重排、再跑一遍
                if sibling.get("role") == "shard" and sibling["id"] != task["id"] and sibling.get("outcome") == "succeeded":
                    self.store.change("tasks", sibling["id"], dict(state="queued", outcome="", exit_code=None, note="弄坏开关 shard-retry-restarts-all：跟着重排"),
                                      reopen=True)
        self.wake()

    # ── 编译目录组（task_server_lib.py 文件头「编译目录」；文件头 ②「编译目录组」）──
    def server_gives_build_directory(self, task):
        """这一条要不要服务给编译目录：会编（compiles）而调用方没给（环境里的 CARGO_TARGET_DIR / CARGO_BUILD_TARGET_DIR）。
        弄坏开关 target-directory-unset 下一律不给（命令照 cargo 默认往起它的目录下的 target 编）。"""
        return (bool(task.get("compiles")) and not any((task.get("environment") or {}).get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)
                and not lib.broken("target-directory-unset", self.environment))
