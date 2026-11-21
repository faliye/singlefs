"""agent-watch.py 的一块：单个子 agent 的告警（工具过长、等待循环、无动静、进度文件、例行询问）、后台任务的进程、只看进程的一路、整份报告、hook 检出的读法、--ack 的读写与剪除、整查一轮 run，以及测这几处的自证用例。入口 agent-watch.py 导入并再导出这里的名字。"""
import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo
import os as preflight_os, sys as preflight_sys  # noqa: E402
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'hooks'))
import lib_work_root as work_roots  # noqa: E402  工作根与会话临时目录根的唯一定义
import lib_prompt_scratch_line as prompt_scratch_line  # noqa: E402  派发提示里「本提示的草稿目录」那一行的唯一认法，派发闸、交回闸、完工清理闸调的是同一份
import lib_session_directory as session_directories  # noqa: E402  会话记录目录的唯一取法（来源仓根 TASK_SERVER_SOURCE_REPOSITORY，不经 $HOME；agent-cost.py、agent-handover.py、watch.sh、move-directory-in-use-check.py 同取它）
from agent_watch_transcripts import (
    ACK_CLASS_PREFIX, AgentTranscript, BROKEN_DETECTION, ENTRY_SCRIPT_PATH, EXITED_PROCESS_STATES, KEPT_COPY_ALERT,
    LEFTOVER_BACKGROUND_ALERT, ROOT_FREE_ALERT, ROOT_FREE_SUBJECT, SELFTEST_SECTION_OFF, bash_result, bash_use, claude_instance_pid,
    command_for_display, command_without_heredoc_bodies, find_transcript, format_duration, format_tokens, handback_records,
    kept_copy_report, leftover_background_report, live_sessions, process_table, record_at, root_free_report, scratch_hog_report,
    scratch_hog_roots, session_task_events, stale_session_reason, wait_for_selftest_ready, wait_until, write_transcript
)
from agent_watch_repository import (
    REPOSITORY_ROOT, emit_knowledge_sync_section, knowledge_state_path, print_lines, processes_restart_line, report_deadline_of,
    repository_status_section, take_repository_baseline, timed_report_head
)


WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")

# 禁用命令只报这个窗口里发出的：看门狗每次起都从头读会话记录，窗外的旧命令会让每一次重起当场退出、再也盯不住那个 agent。
BANNED_COMMAND_WINDOW_SECONDS = 600

CPU_SAMPLE_SECONDS = 2               # 子 agent 在等的后台任务，隔这么久取两次 CPU 时间，报此刻占几个核（background_task_processes）

# --processes-only 不盯的命令：claude 本体、IDE、node（常驻的 MCP 一类）、别的看门狗
COMMAND_MARKERS_NOT_WATCHED = ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")

# 「跑满 N 小时要主 agent 问一次」：告警名是「跑满 <已跨过的那一格> 要主 agent 问一次」，--ack 写「<agent 号>:跑满 <那一格>」
ROUTINE_INQUIRY_ALERT_PREFIX = "跑满 "
ROUTINE_INQUIRY_ALERT_SUFFIX = "要主 agent 问一次"
ROUTINE_INQUIRY_NEXT_STEP = ("主 agent 给它发一条例行询问，并写明「把回答写进它草稿目录的 progress.md」（在做什么、还差几步、在等哪个进程、预计多久；"
                             "等的已经结束就接着做或交回），发完再起看门狗")

HOOKS_DIRECTORY = os.path.join(REPOSITORY_ROOT, ".claude", "hooks")

SELFTEST_READY_VARIABLE = "AGENT_WATCH_SELFTEST_READY_FILE"   # 自证拉起的看门狗第一次进两次整查之间的等待前建这个文件（mark_selftest_ready），自证按它等

# 「中途停下没交回」（文件头同名一段）：结束本轮、没交回、一个后台任务都没起过的
STOPPED_MIDWAY_ALERT = "中途停下没交回"
# 「该交接」：只对这两类报、到 --context-tokens（默认 70 万，推的）报一次；别的类型上下文多大都不叫醒（主 agent 的规矩是上下文大小不是停它的理由）
HANDOVER_AGENT_TYPES = {"implementation-writer", "experiment-runner"}
HANDOVER_ALERT = "该交接"
HANDOVER_TOP_OUTPUTS_SHOWN = 3
# 「进度文件不涨」：结束本轮在等自己后台任务的子 agent，派发提示里草稿目录下的 progress.md 超过 --progress-stale-minutes 没改就报，不等整点
# （草稿目录只认派发提示行首「草稿目录：」那一行，认法在 .claude/hooks/lib_prompt_scratch_line.py；没有那一行的取不到，不到提示点名的别处去找）
PROGRESS_FILE_NAME = "progress.md"
PROGRESS_STALE_ALERT = "进度文件不涨"
PROGRESS_TAIL_LINES = 3
# 提示里点名的工作根下的路径：只给弄坏开关 progressanypath 的宽松取法用（平常只认行首「草稿目录：」那一行，progress_file_of）
SCRATCH_PATH_MENTION = re.compile(work_roots.mention_prefix_pattern() + r"[^\s`'\"，。；：、（）()「」<>]+")
# 一个会话一个看门狗（--discover）：锁、确认过的告警与见过没结束的子 agent 都放在这个会话的状态目录里
STATE_ROOT_VARIABLE = "AGENT_WATCH_STATE_ROOT"   # 环境变量只给 --selftest 换成临时的状态目录
STATE_ROOT = os.environ.get(STATE_ROOT_VARIABLE) or work_roots.state_directory("watch-state")
SINGLE_INSTANCE_EXIT = 5
# 钩子已经拒掉的写与判不了的脚本：只记进报告，不叫醒主 agent（主 agent 醒了也没事可做）
# 钩子判不了而照常放行、往检出记录补一行（「放行但必记检出」，.claude/agent-common.md「不做」一节）的 findings 起头：同样只记不叫醒——
# 判不了的事主 agent 醒了没有可处置的（.claude/skills/dispatch/SKILL.md「派出去之后」「读到告警的处置」第一项：钩子已经拒掉的事与判不了的检出只记进调度记录）。
# 钩子里加了新的这类起头而没登进这一张表，自证 selftest_unjudged_prefixes 判红（它从 .claude/hooks/ 的源码里抽出全部这类起头与这张表比）。
# 弄坏开关 unjudgedwakes 去掉这一组（这些检出真碰到都叫醒）
UNJUDGED_DETECTION_PREFIXES = (
    "开会话提示没判、照常放行", "资源提醒没判、照常放行", "弹窗闸没判、照常放行", "定时闸没判、照常放行",
    "完工清理闸没判全、照常放行", "完工清理闸没判、照常放行", "交回正文闸没判全、照常放行", "派发闸没判全、照常放行",
    "快照清单读不了、照常放行", "续做闸没判全", "写闸没判全", "范围声明闸判不了",
    "循环提醒的停止钩子判不了、照常放行", "提交后的索引检查判不了、照常放行", "时刻写法闸没判、照常放行",
    "放置类弹窗闸没判、照常放行", "会话编号收工检查没判、照常放行",
    "bash-command-detector.sh ", "run-scope-guard.sh ", "two-logs-guard.sh ", "判不了 ",
    "project-code-guard.sh 没判成", "项目代码闸没判全")
INFO_DETECTION_PREFIXES = ("写被拒", "项目代码闸没判全") + (() if BROKEN_DETECTION == "unjudgedwakes" else UNJUDGED_DETECTION_PREFIXES)
# 只记进报告、不叫醒主 agent 的三类条件告警（这三类醒来多半只是重起一只）
INFO_ALERT_NAMES = {"等待循环", "同一命令反复且输出不变", PROGRESS_STALE_ALERT}
EVENT_ALERT_NAMES = {"hook 检出"}   # 事件类告警：确认之后到那个子 agent 结束都不再叫醒；别的是条件类，条件消失就作废


def cost_report(agent_ids, session_dir, thresholds):
    lines = []
    totals = {"calls": 0, "fresh": 0, "cache": 0, "output": 0, "rewrite": 0}
    if agent_ids:
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(paths)]
    for agent_id, path in targets:
        if path is None:
            lines.append(f"子 agent {agent_id}：没有会话记录")
            continue
        transcript = AgentTranscript(agent_id, path)
        average = transcript.total_context_tokens // transcript.model_calls if transcript.model_calls else 0
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」调用 {transcript.model_calls} 次 "
                     f"起步上下文 {format_tokens(transcript.first_context_tokens)} 平均 {format_tokens(average)} 最后 {format_tokens(transcript.last_context_tokens)} "
                     f"新输入 {format_tokens(transcript.fresh_input_tokens)}（其中整份重写 {transcript.full_cache_rewrites} 次 {format_tokens(transcript.full_cache_rewrite_tokens)}） "
                     f"读缓存 {format_tokens(transcript.cache_read_tokens)} 输出 {format_tokens(transcript.output_tokens)}")
        all_characters = sum(characters for _, characters in transcript.tool_result_characters.values()) or 1
        ranked = sorted(transcript.tool_result_characters.items(), key=lambda item: -item[1][1])
        lines.append("  工具结果：" + "；".join(f"{category} {count} 次 {characters} 字符（{100 * characters / all_characters:.0f}%）"
                                          for category, (count, characters) in ranked[:6]))
        files = sorted(transcript.read_characters_by_file.items(), key=lambda item: -item[1][1])[:3]
        if files:
            lines.append("  读得最多的文件：" + "；".join(f"{name} {count} 次 {characters} 字符" for name, (count, characters) in files))
        totals["calls"] += transcript.model_calls
        totals["fresh"] += transcript.fresh_input_tokens
        totals["cache"] += transcript.cache_read_tokens
        totals["output"] += transcript.output_tokens
        totals["rewrite"] += transcript.full_cache_rewrite_tokens
    lines.append(f"合计：调用 {totals['calls']} 次 新输入 {format_tokens(totals['fresh'])}（整份重写 {format_tokens(totals['rewrite'])}） "
                 f"读缓存 {format_tokens(totals['cache'])} 输出 {format_tokens(totals['output'])}")
    return lines, totals


def elapsed_mark_label(mark_minutes):
    """一格写成人读的样子：整小时写「N 小时」；watch.conf 把 ask-every-minutes 改成不是整小时的，写「M 分钟」。"""
    if mark_minutes % 60 == 0:
        return f"{int(mark_minutes // 60)} 小时"
    return f"{mark_minutes:g} 分钟"


def own_scratch_directories(transcript):
    """派发提示行首「草稿目录：」那几行点名的目录（认法只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_directories：~/… 这类写法先展开，
    只留工作根底下的），按出现次序；没有那一行返回空表。"""
    return prompt_scratch_line.scratch_directories(transcript.dispatch_prompt or "")


def readable_mtime(path):
    """监控的是正在变化的文件；列出后已删除或不可读时返回 None。"""
    try:
        return os.path.getmtime(path)
    except OSError:
        return None


def progress_file_of(transcript):
    """本提示草稿目录（own_scratch_directories）下的 progress.md，几行点名几个目录的取改得最晚的那一份；没有那一行、或那几个目录下都没有，返回 None，
    不到提示里点名的别处（别的子 agent 的报告、目录）去找。
    弄坏开关 progressanypath 下取法放宽：提示里点名的工作根或会话临时目录根下的路径都算候选（是 progress.md 就是它，是目录取目录下的，是文件取同目录的）。"""
    if BROKEN_DETECTION == "progressanypath":
        candidates = set()
        for mention in SCRATCH_PATH_MENTION.findall(transcript.dispatch_prompt or ""):
            path = work_roots.expand_written_path(mention).rstrip("/.")
            if os.path.basename(path) == PROGRESS_FILE_NAME:
                candidates.add(path)
            elif os.path.isdir(path):
                candidates.add(os.path.join(path, PROGRESS_FILE_NAME))
            else:
                candidates.add(os.path.join(os.path.dirname(path), PROGRESS_FILE_NAME))
    else:
        candidates = {os.path.join(directory, PROGRESS_FILE_NAME) for directory in own_scratch_directories(transcript)}
    existing = [(moment, path) for path in candidates if os.path.isfile(path)
                if (moment := readable_mtime(path)) is not None]
    return max(existing)[1] if existing else None


def progress_note(transcript, now):
    """告警里附的一句：progress.md 的末几行与修改时刻；派发提示里没有行首「草稿目录：」一行的写取不到，那个目录下没有的写没有。
    弄坏开关 progresstailhead 下取头几行、不取末几行；progresstailreversed 下末几行倒着附；progressblankkept 下空行也算一行（末几行里混进空行）。"""
    if BROKEN_DETECTION == "progressnote":
        return ""
    path = progress_file_of(transcript)
    if path is None and BROKEN_DETECTION != "progressanypath" and not own_scratch_directories(transcript):
        return "；派发提示里没有行首「草稿目录：」一行，取不到 progress.md（不到提示点名的别处去找）"
    if path is None:
        return "；派发提示点名的草稿目录下没有 progress.md"
    moment = readable_mtime(path)
    try:
        if moment is None:
            return "；progress.md 在检查期间已消失或不可读"
        modified = datetime.fromtimestamp(moment, timezone.utc)
        with open(path, encoding="utf-8", errors="replace") as handle:
            filled_lines = [line.strip() for line in handle.read().splitlines() if line.strip() or BROKEN_DETECTION == "progressblankkept"]
            lines = filled_lines[:PROGRESS_TAIL_LINES] if BROKEN_DETECTION == "progresstailhead" else filled_lines[-PROGRESS_TAIL_LINES:]
            if BROKEN_DETECTION == "progresstailreversed":
                lines = lines[::-1]
    except OSError:
        return "；progress.md 在检查期间已消失或不可读"
    return (f"；{path} 改于 {format_duration((now - modified).total_seconds())} 前，"
            f"末 {len(lines)} 行：{' ｜ '.join(line[:120] for line in lines) or '（空）'}")


def routine_inquiry_alert(transcript, now, thresholds):
    """「跑满 N 小时要主 agent 问一次」（判法见文件头）：返回 (告警名, 说明, 下一步)，不报返回 None。ask-every-minutes 写 0 就不报。"""
    if BROKEN_DETECTION == "inquiry" or transcript.first_timestamp is None or thresholds.ask_every_minutes <= 0:
        return None
    if BROKEN_DETECTION != "inquiryended" and transcript.ended(session_task_events(transcript.parent_transcript_path)[1]) is not None:
        return None
    last_message = transcript.last_main_agent_message_timestamp()
    counted_from = transcript.first_timestamp
    if BROKEN_DETECTION == "inquiryreset" and last_message is not None:
        counted_from = max(counted_from, last_message)   # 弄坏：退回「从上一次消息起重新计时」
    period_seconds = thresholds.ask_every_minutes * 60
    marks_crossed = int((now - counted_from).total_seconds() // period_seconds)
    if marks_crossed < 1:
        return None
    mark_reached_at = datetime.fromtimestamp(counted_from.timestamp() + marks_crossed * period_seconds, timezone.utc)
    if last_message is not None and last_message >= mark_reached_at:
        return None   # 这一格之后主 agent 发过消息：问过了
    label = elapsed_mark_label(marks_crossed * thresholds.ask_every_minutes)
    message_text = (f"主 agent 上一次给它发消息在 {format_duration((now - last_message).total_seconds())} 以前，早于这一格"
                    if last_message is not None else "派发之后主 agent 一次都没给它发过消息")
    return (f"{ROUTINE_INQUIRY_ALERT_PREFIX}{label}{ROUTINE_INQUIRY_ALERT_SUFFIX}",
            f"从派发起已跑 {format_duration((now - transcript.first_timestamp).total_seconds())}，"
            f"{format_duration((now - mark_reached_at).total_seconds())} 前跑满 {label}；"
            f"{message_text}（只是慢、这一格不问就 --ack '{transcript.agent_id}:{ROUTINE_INQUIRY_ALERT_PREFIX}{label}'，只确认这一格）"
            f"{progress_note(transcript, now)}",
            ROUTINE_INQUIRY_NEXT_STEP)


def stopped_midway_next_step(transcript):
    """「中途停下没交回」的出路（文件头同名一段）：续做，或抽交接摘要重派一个同类 agent。"""
    return (f"主 agent 先发消息让它接着做或交回；不回应的，抽交接摘要、新派一个 {transcript.agent_type} 接着做："
            f"python3 scripts/agents/agent-handover.py --agent {transcript.agent_id} --out <它的草稿目录>/{transcript.agent_id}-handover.md，"
            f"派发提示指到那份摘要与它的草稿目录；看过、判定它就该停在这里的，--ack {transcript.agent_id}:{STOPPED_MIDWAY_ALERT}")


def agent_alerts(transcript, now, thresholds):
    """返回 [(告警名, 说明, 下一步)]。"""
    alerts = []
    if transcript.is_done():
        return alerts
    if transcript.pending_tool is not None:
        started, tool_name, detail = transcript.pending_tool
        running_seconds = (now - started).total_seconds()
        is_wait_loop = tool_name == "Bash" and WAIT_LOOP_PATTERN.search(detail) and "timeout" not in detail
        if is_wait_loop and BROKEN_DETECTION != "loop" and running_seconds >= thresholds.wait_loop_minutes * 60:
            alerts.append(("等待循环", f"{tool_name} 已跑 {format_duration(running_seconds)}，命令是没有超时的等待循环：{detail[:160]}",
                           "主 agent 判断：它等的条件会不会成立（2026-09-17 实测等过一行永远不会写进日志的字）；会成立就接着盯，不会就决定发消息让它改、还是停掉"))
        elif BROKEN_DETECTION != "timeout" and running_seconds >= thresholds.tool_minutes * 60:
            alerts.append(("工具调用过长", f"{tool_name} 已跑 {format_duration(running_seconds)}：{detail[:160]}",
                           "主 agent 判断：这条命令起的进程还在不在动（ps 看它的子孙进程与它写的文件）、预期还要多久；动就接着盯，不动再决定怎么处理"))
    elif (BROKEN_DETECTION != "nowake" and BROKEN_DETECTION != "midwaymerged" and not transcript.background_started
          and transcript.ended_turn_with_nothing_to_wake_it(now)):
        alerts.append((STOPPED_MIDWAY_ALERT, f"结束本轮、没交回已 {format_duration((now - transcript.last_turn_end_timestamp).total_seconds())}，"
                       f"一个后台任务都没起过，没有东西会叫醒它；最后一句：「{' '.join(transcript.last_text.split())[:160]}」",
                       stopped_midway_next_step(transcript)))
    elif BROKEN_DETECTION != "nowake" and transcript.ended_turn_with_nothing_to_wake_it(now):
        orphaned = transcript.orphaned_background_tasks()
        orphan_text = (f"（其中 {', '.join(orphaned)} 没收到完成通知，但它的输出文件已经没有进程开着，通知不会再来）" if orphaned else "")
        alerts.append(("结束本轮却不会醒", f"结束本轮、没交回已 {format_duration((now - transcript.last_turn_end_timestamp).total_seconds())}，"
                       f"起过的 {len(transcript.background_started)} 个后台任务都已结束{orphan_text}，没有东西会叫醒它",
                       "主 agent 看它最后在等什么：发消息让它接着做或交回，或者停掉；多半是它把活自己放了后台（run_in_background 里又加了 &）、忘了交回，"
                       "或者它等的后台任务在一次中断（失败、进程被杀）里没了"))
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and not transcript.waiting_on_its_own_background_tasks()
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或失败；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
    # 上下文越过一条线：每次调用都要把整段上下文从缓存再读一遍，一旦隔久了还要整份重写，越往后每一步越贵，逼近窗口上限还会被迫压缩。
    # 这条只叫主 agent 来看，不是停的理由：主 agent 看任务是不是异常，不是就继续跑。
    # 只对实现员与执行员报「该交接」，别的类型不报；线写 0 不报（子 agent 上下文多大都不交回）
    if (BROKEN_DETECTION != "context" and (thresholds.context_tokens > 0 or BROKEN_DETECTION == "contextzero")
            and transcript.last_context_tokens >= thresholds.context_tokens
            and (transcript.agent_type in HANDOVER_AGENT_TYPES or BROKEN_DETECTION == "handoveranytype")):
        heaviest = sorted(transcript.tool_result_characters.items(), key=lambda item: item[1][1], reverse=True)[:HANDOVER_TOP_OUTPUTS_SHOWN]
        heaviest_text = "、".join(f"{category} {count} 次共 {characters // 1000}k 字" for category, (count, characters) in heaviest) or "没数到工具结果"
        alerts.append((HANDOVER_ALERT, f"{transcript.agent_type} 最后一次调用的上下文 {transcript.last_context_tokens / 10000:.1f} 万（线在 {thresholds.context_tokens / 10000:.0f} 万），"
                       f"已调用 {transcript.model_calls} 次；吃上下文最多的工具输出：{heaviest_text}",
                       "主 agent 发消息让它停在最近一个编得过（或自证全绿）的点、把做完的与没做的写进报告交回，再新派一个同类 agent 接着做"
                       "（研究脚本 agent-handover.py 抽交接摘要）；看过、判定这一件马上就完的，--ack <agent 号>:该交接"))
    if (BROKEN_DETECTION != "progressstale" and transcript.waiting_on_its_own_background_tasks()
            and getattr(thresholds, "progress_stale_minutes", 0) > 0):
        progress_path = progress_file_of(transcript)
        progress_moment = readable_mtime(progress_path) if progress_path is not None else None
        if progress_moment is not None and (now.timestamp() - progress_moment) >= thresholds.progress_stale_minutes * 60:
            alerts.append((PROGRESS_STALE_ALERT, f"结束本轮在等自己的后台任务，{PROGRESS_FILE_NAME} 超过 {thresholds.progress_stale_minutes:g} 分钟没改"
                           f"{progress_note(transcript, now)}",
                           "主 agent 看它等的后台任务还在不在动（本报告列的开着输出文件的进程）：在动就 --ack <agent 号>:进度文件不涨 接着盯；不动再决定发消息还是停掉"))
    routine_inquiry = routine_inquiry_alert(transcript, now, thresholds)
    if routine_inquiry is not None:
        alerts.append(routine_inquiry)
    recent_entries = transcript.bash_commands[-10:]
    if BROKEN_DETECTION != "repeat":
        counts = {}
        for _, command, output, _ in recent_entries:
            if output is None:
                continue
            key = (" ".join(command.split()), output)
            counts[key] = counts.get(key, 0) + 1
        for (command, _), count in counts.items():
            if count >= thresholds.repeat_count:
                alerts.append(("同一命令反复且输出不变", f"最近 10 次命令里这条出现 {count} 次，每次输出一模一样：{command[:160]}",
                               "定期复检输出会变，一模一样多半是在等一个不会变的结果；主 agent 判断要不要发消息让它换个看法，还是接着等"))
    if BROKEN_DETECTION != "banned":
        banned_window_seconds = max(BANNED_COMMAND_WINDOW_SECONDS, 2 * thresholds.interval_seconds)
        for issued_at, command, output, _ in recent_entries:
            if BROKEN_DETECTION != "bannedwindow" and (now - issued_at).total_seconds() > banned_window_seconds:
                continue
            if (output or "").startswith("PreToolUse:Bash hook error") and BROKEN_DETECTION != "bannedrefused":
                continue   # 入口已拒绝、没执行：hook 检出那一路记着，这里不再报一遍
            if BANNED_COMMAND_PATTERN.search(command_without_heredoc_bodies(command)):
                alerts.append(("禁用命令", f"用了按模式匹配的进程命令：{command[:160]}",
                               "pgrep -f / pkill -f 会命中自己所在的 shell（command-safety.md）；主 agent 判断它等的或要杀的是哪个进程，再发消息让它改用写死的 pid"))
                break
    return alerts


def background_task_processes(transcript):
    """子 agent 在等的后台任务：开着它输出文件的最上层进程，连同子孙进程跑了多久、累计用了多少 CPU、最新的子孙进程多久前起的。
    批次脚本一轮轮起一分钟的探针时，按进程跑了多久列的那一节一个都不够格、根本不出现，主 agent 只好自己去翻 ps；
    所以这里按开着输出文件认，不按进程跑了多久认。"""
    if BROKEN_DETECTION == "holders":
        return []
    live = set(transcript.effectively_live_background_tasks())
    wanted = {os.path.realpath(path) for task_id, path in transcript.background_output_paths.items() if task_id in live}
    if not wanted:
        return []
    table = process_table()
    holders = set()
    for pid in table:
        try:
            descriptors = os.listdir(f"/proc/{pid}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                if os.path.realpath(os.readlink(f"/proc/{pid}/fd/{descriptor}")) in wanted:
                    holders.add(pid)
                    break
            except OSError:
                continue
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    def subtree_of(top):
        subtree, frontier = [top], [top]
        while frontier:
            for child in children.get(frontier.pop(), []):
                subtree.append(child)
                frontier.append(child)
        return subtree

    def cpu_ticks_of(process_ids):
        total = 0
        for pid in process_ids:
            try:
                fields = open(f"/proc/{pid}/stat").read().rsplit(")", 1)[1].split()
                total += sum(int(field) for field in fields[11:15])   # 自己的与已回收子进程的用户态、内核态
            except (OSError, ValueError, IndexError):
                continue
        return total

    # 报「此刻占几个核」（隔 CPU_SAMPLE_SECONDS 取两次样），不报累计 CPU：累计量读起来像跑了很久或重跑过，
    tops = sorted(pid for pid in holders if table[pid]["parent"] not in holders)
    subtrees = {top: subtree_of(top) for top in tops}
    first_sample = {top: cpu_ticks_of(subtrees[top]) for top in tops}
    time.sleep(CPU_SAMPLE_SECONDS)
    lines = []
    for top in tops:
        cores_now = (cpu_ticks_of(subtrees[top]) - first_sample[top]) / ticks_per_second / CPU_SAMPLE_SECONDS
        youngest = max(table[pid]["start_ticks"] for pid in subtrees[top])
        lines.append(f"    进程 {top} 已跑 {format_duration(uptime_seconds - table[top]['start_ticks'] / ticks_per_second)}，"
                     f"它和 {len(subtrees[top]) - 1} 个子孙进程此刻占 {cores_now:.1f} 个核，"
                     f"最新的一个 {format_duration(uptime_seconds - youngest / ticks_per_second)} 前起：{table[top]['command'][:120]}")
    return lines


def mark_selftest_ready():
    """只给自证用：环境里有 AGENT_WATCH_SELFTEST_READY_FILE 时，看门狗第一次进两次整查之间的等待前建这个文件
    （这时已取完检出偏移、拿到单例锁、记下要盯的进程、做完第一次整查）。自证按它等看门狗起来，不按固定秒数等：
    负载高时 python 起得慢，固定等 1.5、2 秒的格会赶在看门狗起来之前动手。平时不设这个变量，什么都不做。"""
    path = os.environ.get(SELFTEST_READY_VARIABLE)
    if path and not os.path.exists(path):
        open(path, "a").close()


def processes_to_watch(root_pid, excluded_pids):
    """--processes-only 盯的进程：{进程号: 起始时刻}。这个实例底下整棵子树，不只叶子；
    看门狗自己那一支、COMMAND_MARKERS_NOT_WATCHED、已经退出的（僵尸）不算，它们的子孙照样往下找。"""
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    watched, frontier = {}, [root_pid]
    while frontier:
        for child in children.get(frontier.pop(), []):
            frontier.append(child)
            info = table[child]
            if (child in excluded_pids or info["state"] in EXITED_PROCESS_STATES
                    or any(marker in info["command"] for marker in COMMAND_MARKERS_NOT_WATCHED)):
                continue
            watched[child] = info["start_ticks"]
    return watched


def still_running(watched):
    """盯着的进程里还没退出的：{进程号: 进程表那一行}。进程号被别的进程接走（起始时刻对不上）算退出了，僵尸也算。"""
    table = process_table()
    running = {}
    for pid, start_ticks in watched.items():
        info = table.get(pid)
        if info is None or info["start_ticks"] != start_ticks:
            continue
        if info["state"] in EXITED_PROCESS_STATES and BROKEN_DETECTION != "zombie":
            continue
        running[pid] = info
    return running


def run_processes_only(arguments, process_root_pid, excluded_pids):
    """--processes-only：起的时候在跑的那些都退出了退 0，有告警（交回之后后台还在跑、副本还在、根分区余量）退 3，到定时回报（report_deadline_of）退 4；
    退 3、退 4 都以重起看门狗那一行收尾。report 只看一眼，有告警退 3、没有退 0。进程本身的卡住、过长、无输出、内存、核数、判定行都归 task-server 判，这里只盯它们还在不在。
    弄坏开关 processesalertrestartomitted：告警退 3 不打重起看门狗那一行。"""
    watched = processes_to_watch(process_root_pid, excluded_pids)
    watch_started = time.time()
    # 「知识同步提醒」的基线只在 watch 里定（落盘到 --session-dir 那个会话目录；没给就只记在这一只里）
    repository_baseline = (take_repository_baseline(arguments, watch_started, knowledge_state_path(arguments, []))
                           if arguments.mode == "watch" else None)
    report_deadline, reminder_first = report_deadline_of(watch_started, repository_baseline, arguments)
    while True:
        running = still_running(watched)
        topmost_running_pids = sorted(pid for pid, info in running.items() if info["parent"] not in running)
        lines = [f"只盯进程：进程 {process_root_pid} 底下看门狗起的时候就在跑的 {len(watched)} 个进程，还没退出的 {len(running)} 个"
                 + ("，最上层的：" if topmost_running_pids else "")]
        lines += [f"  进程 {pid}：{command_for_display(running[pid]['command'])[:140]}" for pid in topmost_running_pids]
        # 根分区余量（文件头「根分区余量过低」）：告警的主体是本机，不是哪个进程
        root_lines, root_found = root_free_report(arguments)
        lines.extend(root_lines)
        found_alerts = [(ROOT_FREE_SUBJECT, name, explanation, next_step) for name, explanation, next_step in root_found]
        if arguments.session_dir:
            leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(
                {os.path.normpath(arguments.session_dir)}, datetime.now(timezone.utc), arguments)
            kept_lines, kept_alerts = kept_copy_report({os.path.normpath(arguments.session_dir)}, datetime.now(timezone.utc), arguments)
        else:
            leftover_lines, leftover_alerts, leftover_pending_until = (
                ["交回、被停或失败之后后台还在跑：没给 --session-dir，这一类没查（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找会话目录传进来）"], [], None)
            kept_lines, kept_alerts = ["交回之后副本还在：没给 --session-dir，这一类没查"], []
        lines.extend(leftover_lines)
        lines.extend(kept_lines)
        alerts, informational_lines = split_informational(found_alerts + leftover_alerts + kept_alerts)
        lines.extend(informational_lines)
        alerts, acknowledged = split_acknowledged(alerts, set(arguments.ack or []))
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            emit_knowledge_sync_section(repository_baseline, arguments, on_alert=True)
            if BROKEN_DETECTION != "processesalertrestartomitted":
                print(processes_restart_line())
            return 3
        # 只剩交回之后宽限里的后台进程时不退：宽限一过再查一次，免得它们刚好在退出之后才够格报
        if not running and BROKEN_DETECTION != "processexit" and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            print_report(lines, alerts)
            if watched:
                print(f"看门狗起的时候就在跑的 {len(watched)} 个进程都退出了。")
            else:
                print("看门狗起的时候这个 Claude 实例底下就没有要盯的进程（看门狗自己那一支、claude 本体、IDE、node 与别的看门狗不算）："
                      "长命令是已经跑完了，还是还没起？")
            return 0
        if time.time() >= report_deadline:
            print_report(lines, alerts)
            print(f"{timed_report_head(watch_started, arguments, reminder_first)}，没有告警，盯着的进程还有 {len(running)} 个没退出。")
            emit_knowledge_sync_section(repository_baseline, arguments, on_alert=False)
            print(processes_restart_line())
            return 4
        next_check_in = min(arguments.interval_seconds, report_deadline - time.time())
        if leftover_pending_until is not None:
            next_check_in = min(next_check_in, leftover_pending_until.timestamp() + 1 - time.time())
        mark_selftest_ready()
        time.sleep(max(0.0, next_check_in))


def build_report(agent_ids, session_dir, thresholds, watch_started):
    now = datetime.now(timezone.utc)
    lines, alerts, done_flags, active_ids, unstarted_ids = [], [], [], [], []
    # --discover 找出来一个没结束的都没有时就是没有要盯的，不退回「会话里最近动过的全部子 agent」
    if agent_ids or (getattr(thresholds, "discover", False) and BROKEN_DETECTION != "discoverfallback"):
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        recent = [path for path in paths if (moment := readable_mtime(path)) is not None
                  and time.time() - moment <= thresholds.active_minutes * 60]
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(recent)]
    live = live_sessions()
    any_handed_back = False
    stopped_midway = []   # 「中途停下没交回」那一节的行
    for agent_id, path in targets:
        if path is None:
            waited = time.time() - watch_started
            lines.append(f"子 agent {agent_id}：还没有会话记录（已等 {format_duration(waited)}）")
            done_flags.append(False)
            unstarted_ids.append(agent_id)
            if waited >= thresholds.not_started_minutes * 60:
                alerts.append((agent_id, "没启动", f"派发 {format_duration(waited)} 之后还没有会话记录", "查派发是否失败、agent id 是否抄对"))
            continue
        try:
            transcript = AgentTranscript(agent_id, path)
        except OSError:
            lines.append(f"子 agent {agent_id}：会话记录在检查期间已消失或不可读，下次重查")
            done_flags.append(False)
            active_ids.append(agent_id)
            continue
        stale_reason = None if transcript.is_done() else stale_session_reason(path, transcript.last_timestamp, session_dir, live)
        if stale_reason:
            # 重启前旧进程留下的记录：不当要盯的，不按「无动静」「工具调用过长」报，也不拦着看门狗在全部交回时退出
            done_flags.append(True)
            lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态=已失效：{stale_reason}；不报告警")
            continue
        done_flags.append(transcript.is_done())
        any_handed_back = any_handed_back or transcript.is_done()
        if not transcript.is_done():
            active_ids.append(agent_id)
        since_last = format_duration((now - transcript.last_timestamp).total_seconds()) if transcript.last_timestamp else "?"
        total = format_duration((transcript.last_timestamp - transcript.first_timestamp).total_seconds()) if transcript.first_timestamp else "?"
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态={transcript.state} 已跑 {total} "
                     f"最后动作 {since_last} 前 调用 {transcript.model_calls} 次 上下文 {transcript.last_context_tokens // 1000}k 缓存整份重写 {transcript.full_cache_rewrites} 次")
        if transcript.parent_transcript_missing and not transcript.is_done():
            lines.append(f"  主会话记录 {transcript.parent_transcript_path} 不在：被 TaskStop 停掉的认不出，被停只按会话记录里的打断判")
        if transcript.pending_tool is not None and not transcript.is_done():
            started, tool_name, detail = transcript.pending_tool
            lines.append(f"  正在跑 {tool_name}（{format_duration((now - started).total_seconds())}）：{' '.join(detail.split())[:200]}")
        if transcript.waiting_on_its_own_background_tasks():
            orphaned = transcript.orphaned_background_tasks()
            orphan_note = f"；{', '.join(orphaned)} 的输出文件已没有进程开着、通知不会再来，不算在等" if orphaned else ""
            holder_lines = background_task_processes(transcript)
            lines.append(f"  结束本轮在等自己的后台任务 {', '.join(transcript.effectively_live_background_tasks())}"
                         f"{'，开着它们输出文件的进程：' if holder_lines else '：没找到开着它们输出文件的进程'}{orphan_note}")
            lines.extend(holder_lines)
        for name, explanation, next_step in agent_alerts(transcript, now, thresholds):
            alerts.append((agent_id, name, explanation, next_step))
            if name == STOPPED_MIDWAY_ALERT:
                stopped_midway.append(f"  子 agent {agent_id} {transcript.agent_type}「{transcript.description}」：{explanation}；"
                                      f"重派：python3 scripts/agents/agent-handover.py --agent {agent_id} --out <它的草稿目录>/{agent_id}-handover.md")
    # 「中途停下没交回」另起一节（文件头同名一段）：被看的子 agent 里这一类几个，一个都没有也写 0 个
    if targets and BROKEN_DETECTION != "midwaymerged":
        lines.append(f"中途停下没交回的子 agent：被看的 {len(targets)} 个里 {len(stopped_midway)} 个"
                     f"{'（结束本轮、没交回、一个后台任务都没起过；先发消息续做，不回应的照下面的命令抽交接摘要重派）' if stopped_midway else ''}")
        lines.extend(stopped_midway)
    # 被看的子 agent 所在的会话：交回之后丢下的后台进程、交回之后副本还在查这些会话里全部子 agent
    session_dirs = {os.path.normpath(session_dir)} if session_dir else set()
    session_dirs |= {os.path.dirname(os.path.dirname(path)) for _, path in targets if path}
    # 根分区余量（文件头「根分区余量过低」）：每次整查都读
    root_lines, root_found = root_free_report(thresholds)
    lines.extend(root_lines)
    alerts.extend((ROOT_FREE_SUBJECT, name, explanation, next_step) for name, explanation, next_step in root_found)
    leftover_pending_until = None
    if session_dirs:
        leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(session_dirs, now, thresholds)
        lines.extend(leftover_lines)
        alerts.extend(leftover_alerts)
        kept_lines, kept_alerts = kept_copy_report(session_dirs, now, thresholds)
        lines.extend(kept_lines)
        alerts.extend(kept_alerts)
        if any_handed_back:
            # 有子 agent 交回了才查草稿根（交回时报）；只进报告，不叫醒
            lines.extend(scratch_hog_report(scratch_hog_roots({os.path.basename(directory) for directory in session_dirs}), now,
                                            getattr(thresholds, "scratch_idle_hours", 6)))
    else:
        lines.append("交回、被停或失败之后后台还在跑：被看的子 agent 都还没有会话记录，不知道是哪个会话，这一类没查")
        lines.append("交回之后副本还在：被看的子 agent 都还没有会话记录，不知道是哪个会话，这一类没查")
    return lines, alerts, bool(done_flags) and all(done_flags), (active_ids, unstarted_ids), leftover_pending_until


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


SCOPE_SEGMENT_KEYS = ("种类", "跑什么", "范围", "为什么")   # 与 .claude/hooks/lib_run_scope.py 的 SEGMENT_KEYS 同一组


def scope_declaration_line(entry):
    """范围声明钩子（.claude/hooks/run-scope-guard.sh）放行的一条在「范围声明」一节里的一行：谁、命令、四段（照记录里切开的 segments，没切开的写声明原文）。"""
    scope = entry.get("run_scope") if isinstance(entry.get("run_scope"), dict) else {}
    segments = scope.get("segments") if isinstance(scope.get("segments"), dict) else {}
    four = ("；".join(f"{key}={segments.get(key, '')}" for key in SCOPE_SEGMENT_KEYS) if segments
            else f"声明原文 {scope.get('declaration') or '（没有）'}")
    who = f"{entry.get('agent_type') or '?'}{' ' + entry['agent_id'] if entry.get('agent_id') else ''}"
    where = "" if scope.get("where") in (None, "命令文本") else f"（{scope['where']}）"
    return f"{who}：{(entry.get('command') or '')[:160]}{where}：{four}"


def scope_declaration_section(scope_declarations):
    """定时回报里的「范围声明」一节：放行记录一条一行，只列不叫醒；一条都没有时不出这一节。"""
    if not scope_declarations:
        return []
    return ([f"范围声明（run-scope-guard.sh 放行的 {len(scope_declarations)} 条，只列不叫醒；范围合不合适由主 agent 对着看）："]
            + [f"  {line}" for line in scope_declarations])


def read_detections(detections_path, start_offset, session_ids, watched_agent_ids=(), scope_declarations=None):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话、被看的子 agent 或主 agent 的告警], [只进报告的记录])。
    范围声明钩子的放行记录（会话号在 scope_session_id、带 run_scope）不是检出：给了 scope_declarations 就把属于这些会话的逐条记成一行追加进去
    （scope_declaration_line），不叫醒；弄坏开关 scopeunlisted 下一条都不记。"""
    alerts, notes = [], []
    if not os.path.isfile(detections_path):
        return start_offset, alerts, notes
    position = start_offset
    with open(detections_path, "rb") as handle:
        handle.seek(start_offset)
        for raw_line in handle:
            position += len(raw_line)
            try:
                entry = json.loads(raw_line.decode("utf-8", "replace"))
            except ValueError:
                continue
            if not isinstance(entry, dict):
                continue
            if entry.get("run_scope") is not None and entry.get("scope_session_id") is not None:
                if (scope_declarations is not None and BROKEN_DETECTION != "scopeunlisted" and entry.get("scope_session_id") in session_ids
                        and not (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids)):
                    scope_declarations.append(scope_declaration_line(entry))
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            if (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids
                    and BROKEN_DETECTION != "otheragent"):
                continue   # 同一会话里别的子 agent 的检出归盯它的那个看门狗
            findings = [str(finding) for finding in (entry.get("findings") or [])]
            if findings and all(finding.startswith("没有 timeout 的等待循环") for finding in findings) and BROKEN_DETECTION != "loopnoise":
                # 只检出等待循环的不立刻叫醒：正常等编译、等全量跑也是这个形状。会话记录那一路在它跑满 --wait-loop-minutes 时再报。
                continue
            refused = any(finding.startswith("写被拒") for finding in findings)
            if entry.get("agent_type") == "主 agent" and refused and BROKEN_DETECTION != "ownrefusal":
                # 主 agent 自己的写被拒：拒绝信息已经在它的工具结果里，再叫醒一次只是打断。
                # 按模式找进程由上游钩子在执行前拒绝，不写检出记录，这里读不到它。
                notes.append(f"主 agent 自己的命令被 hook 拒了（不叫醒，拒绝信息在它的工具结果里）：{(entry.get('command') or '')[:120]}")
                continue
            if findings and all(finding.startswith(INFO_DETECTION_PREFIXES) for finding in findings) and BROKEN_DETECTION != "infowakes":
                notes.append(f"hook 检出（只记不叫醒）：{entry.get('agent_type') or '?'} {entry.get('agent_id') or ''}：{'、'.join(findings)}："
                             f"{(entry.get('command') or '')[:120]}")
                continue
            if findings and detection_outcome(entry) == DETECTION_REFUSED and BROKEN_DETECTION != "refusedwakes":
                # 钩子在执行前已经拒掉的命令（不经 task-server 跑项目代码被拒、认不出起什么的写法被拒……）：没有执行，拒绝信息与出路在它的工具结果里，
                # 主 agent 醒了也没有可处置的；只进报告，不叫醒（「记录不处置」那一类）。
                notes.append(f"hook 检出（执行前已被 hook 拒了、没有执行，只记不叫醒）：{entry.get('agent_type') or '?'} {entry.get('agent_id') or ''}："
                             f"{'、'.join(findings)}：{(entry.get('command') or '')[:120]}")
                continue
            # 主体写 agent 号（有的话）：--ack <agent 号>:hook 检出 只压这一个子 agent 的检出；类型写进说明
            subject = (entry.get("agent_id") if BROKEN_DETECTION != "detectiontype" else None) or entry.get("agent_type") or "?"
            alerts.append((subject, "hook 检出",
                           f"{entry.get('agent_type') or '?'}：{'、'.join(entry.get('findings') or [])}：{(entry.get('command') or '')[:160]}",
                           detection_next_step(entry)))
    return position, alerts, notes


# 检出记录的结局（钩子写在 outcome 一栏）；没写这一栏的旧记录按发现的起头认：这几种起头是执行前拒掉的
DETECTION_REFUSED, DETECTION_RECORDED = "被拒", "只记"
REFUSED_FINDING_PREFIXES = ("写被拒", "不经 task-server 跑项目代码被拒", "不经 task-server 起要经服务的外部程序被拒", "认不出起什么的写法被拒", "绕开 task-server 的写法被拒",
                            "要临时 prune-server 的自证写成了裸的 --selftest 被拒")


def detection_outcome(entry):
    """这一条检出的结局：「被拒」（执行前已拒、没执行）或「只记」（照常执行）。弄坏开关 outcomeignored 下退回只认「写被拒」的旧判法。"""
    findings = [str(finding) for finding in (entry.get("findings") or [])]
    if BROKEN_DETECTION == "outcomeignored":
        return DETECTION_REFUSED if any(finding.startswith("写被拒") for finding in findings) else DETECTION_RECORDED
    if entry.get("outcome") in (DETECTION_REFUSED, DETECTION_RECORDED):
        return entry["outcome"]
    return DETECTION_REFUSED if any(finding.startswith(REFUSED_FINDING_PREFIXES) for finding in findings) else DETECTION_RECORDED


def detection_next_step(entry):
    """hook 检出告警的下一步：写被拒、命令执行前被拒、只记照跑三种各说各的。"""
    if any(str(finding).startswith("写被拒") for finding in (entry.get("findings") or [])):
        return "这次写被 hook 拒了、没写进去；主 agent 判断是不是该写的，再决定发消息让它改、自己改、还是派该写的 agent"
    if detection_outcome(entry) == DETECTION_REFUSED:
        return ("这条命令执行前已被 hook 拒了、没有执行（拒绝信息与出路在它的工具结果里）；主 agent 判断它该不该跑，"
                "再决定发消息让它照出路改、交给该跑的一方、还是不管（主 agent 自己的，照拒绝信息最前面那条加好前缀的命令重跑或不跑）")
    return "检出 hook 只记不拦，命令照常在跑；主 agent 判断它会不会出问题，再决定接着盯、发消息让它改、还是处理"


def print_report(lines, alerts):
    now = datetime.now(timezone.utc)
    print(f"[agent-watch {now.astimezone(ZoneInfo('Asia/Tokyo')).strftime('%Y-%m-%d')}]")
    for line in lines:
        print(line)
    for owner, name, explanation, next_step in alerts:
        print(f"⚠️ {name}（{owner}）：{explanation}")
        print(f"   → 怎么办：{next_step}")
    if not alerts:
        print("没有告警。")


def restart_command(active_ids, unstarted_ids, discover):
    """出路句里照抄就能起的那条命令：主 agent 经 watch.sh 起看门狗，它不认 agent-watch.py 自己的 --agents / --processes-only。"""
    if BROKEN_DETECTION == "restarthint":
        return f"--agents {','.join(active_ids + unstarted_ids)}"
    if discover:
        return "bash scripts/agents/watch.sh"
    return f"bash scripts/agents/watch.sh {','.join(active_ids + unstarted_ids)}"


def print_active_list(active_ids, unstarted_ids, discover=False):
    if not active_ids and not unstarted_ids:
        return
    print(f"还没交回、没被停的子 agent（{len(active_ids)} 个，另有还没会话记录的 {len(unstarted_ids)} 个）：{','.join(active_ids + unstarted_ids)}")
    print(f"   → 接着盯就用 run_in_background 再起一个看门狗：{restart_command(active_ids, unstarted_ids, discover)}")


def own_process_chain(table, start_pid):
    chain, pid = set(), start_pid
    while pid in table and pid > 1:
        chain.add(pid)
        pid = table[pid]["parent"]
    return chain


def split_informational(alerts):
    """三类只记不叫醒的条件告警：从告警里摘出来，写成报告行；AGENT_WATCH_BREAK=infoalertwakes 时照旧叫醒（自检用）。"""
    if BROKEN_DETECTION == "infoalertwakes":
        return alerts, []
    remaining, informational = [], []
    for alert in alerts:
        (informational if alert[1] in INFO_ALERT_NAMES else remaining).append(alert)
    return remaining, [f"只记不叫醒：{name}（{subject}）：{explanation}" for subject, name, explanation, _ in informational]


def ack_scope_keys(alert):
    """一条告警能被哪种范围确认认：类:<告警名>。"""
    if BROKEN_DETECTION == "ackscope":
        return set()
    return {f"{ACK_CLASS_PREFIX}{alert[1]}"}


def split_acknowledged(alerts, acknowledgements):
    """主 agent 用 --ack 确认过「只是慢、接着盯」的告警：只写进报告，不叫醒主 agent。
    规程写「只是慢就再起一个看门狗接着盯」，而「工具调用过长」这类条件在那次调用结束之前一直成立，
    新起的看门狗当场又报同一句退出，等于盯不住；确认只在这一次看门狗里有效，下一次起不带就失效。"""
    if BROKEN_DETECTION == "ack" or not acknowledgements:
        return alerts, []
    remaining, acknowledged = [], []
    for alert in alerts:
        subject, name, explanation, _ = alert
        keys = {f"{subject}:{name}"} | ack_scope_keys(alert)
        if name == LEFTOVER_BACKGROUND_ALERT and BROKEN_DETECTION != "leftoverack":
            process_id = re.match(r"进程 (\d+) ", explanation)
            if process_id:
                keys.add(f"进程:{process_id.group(1)}")
        # 「跑满 N 小时要主 agent 问一次」也认短写「<agent 号>:跑满 N 小时」：只确认那一格，下一格告警名不同，照报
        if name.startswith(ROUTINE_INQUIRY_ALERT_PREFIX) and name.endswith(ROUTINE_INQUIRY_ALERT_SUFFIX) and BROKEN_DETECTION != "inquiryack":
            keys.add(f"{subject}:{name[:-len(ROUTINE_INQUIRY_ALERT_SUFFIX)]}")
        (acknowledged if keys & acknowledgements else remaining).append(alert)
    return remaining, acknowledged


def session_state_directory(session_dir):
    return os.path.join(STATE_ROOT, os.path.basename(os.path.normpath(session_dir)))


def acquire_single_instance(state_dir):
    """拿这个会话的看门狗锁（flock，进程退出就放）：拿到返回打开着的锁文件，已有一个在盯返回 (None, 那一个的 pid 与起的时刻)。"""
    import fcntl
    os.makedirs(state_dir, exist_ok=True)
    handle = open(os.path.join(state_dir, "watch.lock"), "a+", encoding="utf-8")
    try:
        if BROKEN_DETECTION != "nosingleinstance":
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.seek(0)
        holder = handle.read().strip()
        handle.close()
        return None, holder
    handle.seek(0)
    handle.truncate()
    handle.write(f"pid {os.getpid()} 起于 {datetime.now(ZoneInfo('Asia/Tokyo')).strftime('%Y-%m-%d')}")
    handle.flush()
    return handle, None


def read_acks(state_dir):
    path = os.path.join(state_dir, "watch-acks.tsv")
    if BROKEN_DETECTION == "acksnotpersisted" or not os.path.isfile(path):
        return set()
    return {line.rstrip("\n").split("\t", 1)[0] for line in open(path, encoding="utf-8") if line.strip()}


def write_acks(state_dir, acks):
    os.makedirs(state_dir, exist_ok=True)
    path = os.path.join(state_dir, "watch-acks.tsv")
    temporary = f"{path}.{os.getpid()}.{threading.get_ident()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        for key in sorted(acks):
            handle.write(f"{key}\t{datetime.now(timezone.utc).isoformat()}\n")
    os.replace(temporary, path)


ACK_UPDATE_PAUSE_SECONDS = 0.0   # 读与写之间停多久：只给 --selftest 拉开两只同时写的窗口用


def update_acks(state_dir, change):
    """watch-acks.tsv 的读改写：change(读到的确认集合) 交回要写下的集合。整段拿着同目录 watch-acks.lock 的 flock（进程退出就放）：
    两只看门狗同时写（在盯的那一只剪掉作废的确认、另起被拒的那一只记下新的 --ack），不加锁时后写的冲掉先写的，新给的 --ack 丢掉
    （records/2026-10-23-subagent拆分提案.md 第一百二十一节）。弄坏开关 acksunlocked 不拿锁。"""
    import fcntl
    os.makedirs(state_dir, exist_ok=True)
    with open(os.path.join(state_dir, "watch-acks.lock"), "a", encoding="utf-8") as lock:
        if BROKEN_DETECTION != "acksunlocked":
            fcntl.flock(lock, fcntl.LOCK_EX)
        acks = change(read_acks(state_dir))
        if ACK_UPDATE_PAUSE_SECONDS:
            time.sleep(ACK_UPDATE_PAUSE_SECONDS)
        write_acks(state_dir, acks)
    return acks


def alert_keys(alert):
    """一条告警能被哪些 --ack 键确认（与 split_acknowledged 认的同一套）。"""
    subject, name, explanation, _ = alert
    keys = {f"{subject}:{name}"}
    if BROKEN_DETECTION != "ackscopepersist":
        keys |= ack_scope_keys(alert)
    process_id = re.match(r"进程 (\d+) ", explanation)
    if process_id and name == LEFTOVER_BACKGROUND_ALERT:
        keys.add(f"进程:{process_id.group(1)}")
    if name.startswith(ROUTINE_INQUIRY_ALERT_PREFIX) and name.endswith(ROUTINE_INQUIRY_ALERT_SUFFIX):
        keys.add(f"{subject}:{name[:-len(ROUTINE_INQUIRY_ALERT_SUFFIX)]}")
    return keys


def pruned_acks(acks, alerts, unfinished_agent_ids):
    """条件类告警的确认在条件消失（这一次整查没有它）时作废；事件类（hook 检出）的确认到那个子 agent 结束才作废。"""
    present = set().union(*(alert_keys(alert) for alert in alerts)) if alerts else set()
    kept = set()
    for key in acks:
        subject, _, name = key.partition(":")
        is_event = any(name.startswith(event) for event in EVENT_ALERT_NAMES)
        if (is_event and subject in unfinished_agent_ids) or (not is_event and key in present) or BROKEN_DETECTION == "acksneverpruned":
            kept.add(key)
    return kept


def discovered_agent_ids(session_dir, state_dir, cache, thresholds):
    """这个会话里还没交回、没被停、没失败的子 agent：会话记录 --active-minutes 以内动过的，加上更早起的看门狗见过、还没结束的
    （记在状态目录的 watch-agents.tsv，等后台任务等得久、会话记录好久没动的也接得上）。cache：路径 → (大小, 修改时刻, 结束了没有)。"""
    if BROKEN_DETECTION == "discovernone":
        return []
    agents_path = os.path.join(state_dir, "watch-agents.tsv")
    remembered = {line.strip() for line in open(agents_path, encoding="utf-8")} if os.path.isfile(agents_path) else set()
    notifications = session_task_events(os.path.normpath(session_dir) + ".jsonl")[1]
    live = live_sessions()
    unfinished = []
    for path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
        agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
        status = os.stat(path)
        key = (status.st_size, status.st_mtime)
        known = cache.get(path)
        if known is not None and known[:2] == key and known[2]:
            continue
        if known is None and agent_id not in remembered and time.time() - status.st_mtime > thresholds.active_minutes * 60:
            continue
        transcript = AgentTranscript(agent_id, path)
        ended = (transcript.is_done() or transcript.ended(notifications) is not None
                 or stale_session_reason(path, transcript.last_timestamp, session_dir, live) is not None)   # 重启前旧进程留下的记录不再当要盯的
        cache[path] = (key[0], key[1], ended)
        if not ended:
            unfinished.append(agent_id)
    os.makedirs(state_dir, exist_ok=True)
    temporary = f"{agents_path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{agent_id}\n" for agent_id in unfinished))
    os.replace(temporary, agents_path)
    return unfinished


def run(arguments):
    if arguments.processes_only and (arguments.agents or arguments.mode == "cost" or arguments.process_root_pid == 0):
        print("✗ --processes-only 只盯进程，不和 --agents、cost、--process-root-pid 0 一起用\n"
              "→ 怎么办：盯子 agent 就去掉 --processes-only（子 agent 在等的后台任务照样在报告里列出开着输出文件的进程）；只盯主 agent 自己放后台的长命令就只给 --processes-only"
              "（可带 --session-dir，同时查交回之后后台还在跑）",
              file=sys.stderr)
        return 2
    if not arguments.agents and not arguments.session_dir and not arguments.processes_only:
        print("✗ 要给 --agents 或 --session-dir\n→ 怎么办：派发返回的 agent id 用逗号连起来传 --agents；只盯主 agent 自己放后台的长命令用 --processes-only",
              file=sys.stderr)
        return 2
    if getattr(arguments, "discover", False) and (not arguments.session_dir or arguments.agents or arguments.processes_only or arguments.mode != "watch"):
        print("✗ --discover 只和 watch、--session-dir 一起用，不给 --agents、--processes-only\n"
              "→ 怎么办：bash scripts/agents/watch.sh 不带 agent 号（它从 CLAUDE_CODE_SESSION_ID 找会话目录）", file=sys.stderr)
        return 2
    agent_ids = [item for item in (arguments.agents or "").split(",") if item]
    discover = bool(getattr(arguments, "discover", False))
    state_dir = session_state_directory(arguments.session_dir) if discover else None
    if discover:
        lock_handle, holder = acquire_single_instance(state_dir)
        if lock_handle is None:
            update_acks(state_dir, lambda acks: acks | set(arguments.ack or []))
            print(f"已有一个看门狗在盯这个会话（{holder}），这一次不另起"
                  f"{'；--ack 已记进 ' + os.path.join(state_dir, 'watch-acks.tsv') + '，在盯的那一个下一次检查时读到' if arguments.ack else ''}。")
            return SINGLE_INSTANCE_EXIT
        update_acks(state_dir, lambda acks: acks | set(arguments.ack or []))
    discovery_cache = {}
    if arguments.processes_only:
        table = process_table()
        process_root_pid = arguments.process_root_pid if arguments.process_root_pid is not None else claude_instance_pid(table, os.getpid())
        if process_root_pid is None:
            print("✗ 往上找不到跑这个看门狗的 Claude 实例，--processes-only 没有进程可盯\n"
                  "→ 怎么办：在 Claude 的 Bash 工具里起它，或者用 --process-root-pid 指定从哪个进程往下看", file=sys.stderr)
            return 2
        return run_processes_only(arguments, process_root_pid, own_process_chain(table, os.getpid()))
    if arguments.mode == "cost":
        lines, _ = cost_report(agent_ids, arguments.session_dir, arguments)
        print("\n".join(lines))
        return 0
    watch_started = time.time()
    detections_path = arguments.detections_file
    detections_offset = os.path.getsize(detections_path) if (arguments.mode == "watch" and os.path.isfile(detections_path)) else 0
    detection_notes = []
    scope_declarations = []   # 范围声明钩子放行的（这一个看门狗起来之后读到的），定时回报里列成「范围声明」一节
    # 「知识同步提醒」的基线只在 watch 里定（落盘到会话目录：--session-dir，没给取被看的子 agent 会话记录所在的那一层）
    repository_baseline = (take_repository_baseline(arguments, watch_started, knowledge_state_path(arguments, agent_ids))
                           if arguments.mode == "watch" else None)
    report_deadline, reminder_first = report_deadline_of(watch_started, repository_baseline, arguments)

    def acknowledgements():
        keys = set(arguments.ack or []) | (read_acks(state_dir) if discover else set())
        if BROKEN_DETECTION == "ackclassfiltered":
            # 弄坏开关（只给 watch.sh --selftest 自检⑬用，那一格经 watch.sh 真起看门狗子进程）：滤掉 类: 开头的键，--ack 类:<告警名> 确认过的告警照样叫醒
            keys = {key for key in keys if not key.startswith(ACK_CLASS_PREFIX)}
        return keys
    while True:
        if discover:
            agent_ids = discovered_agent_ids(arguments.session_dir, state_dir, discovery_cache, arguments)
        lines, alerts, all_done, active_lists, leftover_pending_until = build_report(agent_ids, arguments.session_dir, arguments, watch_started)
        watched_transcripts = [path for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path]
        session_ids = {session_id_of(path) for path in watched_transcripts}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
        detections_offset, detection_alerts, new_notes = read_detections(
            detections_path, detections_offset, session_ids, set(agent_ids), scope_declarations)
        detection_notes += new_notes
        lines += detection_notes
        lines += scope_declaration_section(scope_declarations)
        alerts += detection_alerts
        if discover:
            update_acks(state_dir, lambda acks: pruned_acks(set(arguments.ack or []) | acks, alerts, set(agent_ids)))
        alerts, informational_lines = split_informational(alerts)
        lines += informational_lines
        alerts, acknowledged = split_acknowledged(alerts, acknowledgements())
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            emit_knowledge_sync_section(repository_baseline, arguments, on_alert=True)
            print_active_list(*active_lists, discover)
            return 3
        # 全部交回照旧退出，这一次 build_report 已经把「交回之后后台还在跑」查过了；只剩宽限里的，等宽限过了再查一次才退
        if all_done and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
            print_report(lines, alerts)
            if BROKEN_DETECTION == "repositoryonfinish":
                print_lines(repository_status_section(repository_baseline, arguments)[0])
            print("被看的子 agent 全部交回或被停。")
            return 0
        if time.time() >= report_deadline:
            print_report(lines, alerts)
            print(f"{timed_report_head(watch_started, arguments, reminder_first)}，没有告警。")
            emit_knowledge_sync_section(repository_baseline, arguments, on_alert=False)
            print_active_list(*active_lists, discover)
            return 4
        next_full_check = time.time() + arguments.interval_seconds
        if BROKEN_DETECTION != "deadline":
            next_full_check = min(next_full_check, report_deadline)
        if all_done:
            next_full_check = min(next_full_check, leftover_pending_until.timestamp() + 1)
        mark_selftest_ready()
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts, new_notes = read_detections(
                detections_path, detections_offset, session_ids, set(agent_ids), scope_declarations)
            detection_notes += new_notes
            # 检出等不到下一次整查：读到本会话的新检出就马上整查一次，有没确认的告警就退 3
            if detection_alerts:
                lines, alerts, _, active_lists, _ = build_report(agent_ids, arguments.session_dir, arguments, watch_started)
                alerts, informational_lines = split_informational(alerts)
                lines += informational_lines
                alerts, acknowledged = split_acknowledged(alerts + detection_alerts, acknowledgements())
                if alerts:
                    lines = (lines + detection_notes + scope_declaration_section(scope_declarations)
                             + [f"已确认接着盯（--ack {subject}:{name}）：{explanation}"
                                for subject, name, explanation, _ in acknowledged])
                    print_report(lines, alerts)
                    emit_knowledge_sync_section(repository_baseline, arguments, on_alert=True)
                    print_active_list(*active_lists, discover)
                    return 3


def selftest_kept_copies(work, thresholds, failures):
    """「交回之后副本还在」的自检：另起会话目录，交回正文与它点名的报告文件里写「没删 <全路径>」的几个合成子 agent。返回查了几个合成子 agent。
    弄坏开关：keptcopy（两格红）、keptcopyage（「只报交回超过 6 小时……」那一格红：keptrecent 也报）、keptcopyexists（同一格红：已经删掉的也报）。"""
    session = os.path.join(work, "kept-copy-session")
    copies = os.path.join(work, "kept-copies")
    paths = {name: os.path.join(copies, name) for name in ("old-repo", "old-target", "recent-repo", "report-repo", "running-repo", "deleted-repo")}
    for name, path in paths.items():
        if name != "deleted-repo":
            os.makedirs(os.path.join(path, "crates"), exist_ok=True)
    report_path = os.path.join(work, "kept-copy-report.md")
    with open(report_path, "w", encoding="utf-8") as handle:
        handle.write(f"# 报告\n\n## 没做什么\n\n- 没删 `{paths['report-repo']}`：主 agent 要在里面交 crates 变异那一道带 --item\n")

    def handed_back(agent_id, hours_ago, message):
        minutes = hours_ago * 60
        records = [bash_use(minutes + 5, "t1", "ls", "m1"), bash_result(minutes + 4, "t1"), *handback_records(minutes, "h1", "m2")]
        records[2]["message"]["content"][0]["input"]["message"] = message
        write_transcript(session, agent_id, records, {"agentType": "implementation-writer", "description": agent_id})

    handed_back("keptold", 7, f"结论：做完了。\n没删 {paths['old-repo']}/：主 agent 要复跑\n没删：{paths['old-target']}，留给主 agent\n"
                              f"没删 {paths['deleted-repo']}：已经被别人删了")
    handed_back("keptrecent", 1, f"没删 {paths['recent-repo']}：主 agent 要复跑")
    handed_back("keptreport", 8, f"报告 {report_path}（sha256 0000），交回正文里不写没删的路径")
    handed_back("keptnone", 9, "结论：做完了，草稿都删了")
    write_transcript(session, "keptrunning", [bash_use(500, "t1", f"echo 没删 {paths['running-repo']}", "m1"), bash_result(499, "t1")],
                     {"agentType": "implementation-writer", "description": "还没交回"})
    kept_thresholds = argparse.Namespace(**dict(vars(thresholds), kept_copy_hours=6))
    _lines, found = kept_copy_report({session}, datetime.now(timezone.utc), kept_thresholds)
    got = sorted((agent_id, re.match(r"副本 (\S+) 还在", explanation).group(1)) for agent_id, name, explanation, _ in found if name == KEPT_COPY_ALERT)
    wanted = sorted([("keptold", paths["old-repo"]), ("keptold", paths["old-target"]), ("keptreport", paths["report-repo"])])
    if got != wanted:
        failures.append(f"「{KEPT_COPY_ALERT}」应当只报交回超过 6 小时、交回正文或它点名的报告里写了没删而路径还在的 {wanted}，实际 {got}")
    report_lines, report_alerts, *_ = build_report(["keptold"], session, kept_thresholds, time.time())
    remaining, acknowledged = split_acknowledged(report_alerts, {f"keptold:{KEPT_COPY_ALERT}"})
    if not (any(name == KEPT_COPY_ALERT for _, name, _, _ in report_alerts) and any(line.startswith("交回之后副本还在：查了") for line in report_lines)
            and not any(subject == "keptold" and name == KEPT_COPY_ALERT for subject, name, _, _ in remaining) and acknowledged):
        failures.append(f"看门狗的报告应当带「{KEPT_COPY_ALERT}」告警与一行汇总、--ack keptold:{KEPT_COPY_ALERT} 之后不叫醒，"
                        f"实际告警 {[name for _, name, _, _ in report_alerts]}、确认掉 {len(acknowledged)} 条、汇总 {[line for line in report_lines if '副本' in line]}")
    return 5

# 钩子里「判不了而照常放行」记检出用的 findings 起头，从 .claude/hooks/ 的源码里抽：五种写法各有一条抽法（UNJUDGED_DETECTION_PREFIXES 上方的说明）
UNJUDGED_SOURCE_MARKER = re.compile(r"没判全|没判|判不了|照常放行")


def hook_unjudged_prefixes(hooks_directory):
    """从钩子目录的源码里抽出记检出用的「判不了」起头：{起头: [出处文件名…]}。五种写法：
    ① 名字以 PREFIX 收尾的常量，值里有没判、判不了或照常放行的字样；② findings 那一行里 f"<起头>：…" 的起头（起头里有没判全、没判、判不了字样）；
    ③ python3 /dev/fd/3 "$HOOK_DIR" <脚本名>.sh（入口自己记检出，findings 以「<脚本名> 」起头）；④ not_judged(f"<脚本名>.sh …")；⑤ noted.append(f"判不了 …")。
    钩子拆成入口 .sh 加同层 .py 部分文件时，起头常在部分文件里，所以 .sh 与 .py 都读；弄坏开关 unjudgedpartsunread 下只读 .sh（部分文件里的起头抽不出）。"""
    found = {}

    def add(prefix, name):
        found.setdefault(prefix, [])
        if name not in found[prefix]:
            found[prefix].append(name)
    for name in sorted(os.listdir(hooks_directory)):
        if not name.endswith((".sh", ".py")) or (BROKEN_DETECTION == "unjudgedpartsunread" and name.endswith(".py")):
            continue
        try:
            with open(os.path.join(hooks_directory, name), encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        for line in text.splitlines():
            if line.lstrip().startswith("#"):
                continue
            constant = re.match(r'^[A-Z][A-Z_]*PREFIX\s*=\s*"([^"]+)"', line)
            if constant and UNJUDGED_SOURCE_MARKER.search(constant.group(1)):
                add(constant.group(1), name)
            if "findings" in line:
                for match in re.finditer(r'(?<![\w"])f?"(?!")([^"{}：:\n]*(?:没判全|没判|判不了)[^"{}：:\n]*)[：:]', line):
                    add(f"{name} {match.group(1)[3:]}" if match.group(1).startswith("%s ") else match.group(1), name)   # printf 的 %s 是钩子名（bash 那一层起不来时的检出）
            for pattern in (r'python3 /dev/fd/3 "\$HOOK_DIR" ([\w.-]+\.sh) ', r'not_judged\(f"([\w.-]+\.sh) '):
                match = re.search(pattern, line)
                if match:
                    add(match.group(1) + " ", name)
            guard = re.match(r'^GUARD_SCRIPT_NAME\s*=\s*"([\w.-]+\.sh)"', line)
            if guard:
                add(guard.group(1) + " ", name)
            if re.search(r'noted\.append\(f"判不了 ', line):
                add("判不了 ", name)
    return found


def selftest_unjudged_prefixes(work, failures):
    """钩子判不了而照常放行记下的检出（UNJUDGED_DETECTION_PREFIXES）只记不叫醒：① 从 .claude/hooks/ 的源码里抽出的全部这类起头都在 INFO_DETECTION_PREFIXES 里
    （钩子里加了新起头而没登进来，这一格红）；② 造一个带新起头的假钩子目录，抽法抽得出、对不上表（证明 ① 会红）；③ 表里每个起头造一条检出记录，读下来都只进报告、一条告警都没有；
    同一条记录里夹着真检出的照旧叫醒。返回查了几格。"""
    checks = 0
    hooks_directory = HOOKS_DIRECTORY
    extracted = hook_unjudged_prefixes(hooks_directory)
    checks += 1
    uncovered = {prefix: sources for prefix, sources in extracted.items() if not prefix.startswith(INFO_DETECTION_PREFIXES)}
    if not extracted or uncovered:
        failures.append(f"钩子源码里记检出用的「判不了而照常放行」起头都应当在 INFO_DETECTION_PREFIXES 里（只记不叫醒，真碰到别叫醒主 agent），"
                        f"没登进表的：{uncovered or '一个都没抽到（抽法坏了）'}；加进 UNJUDGED_DETECTION_PREFIXES")
    fake_hooks = os.path.join(work, "unjudged-prefix-hooks")
    os.makedirs(fake_hooks)
    with open(os.path.join(fake_hooks, "fake-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write('NEW_UNJUDGED_PREFIX = "新闸没判、照常放行"\n')
    with open(os.path.join(fake_hooks, "fake-printf-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write('  printf "{\\"findings\\": [\\"%s 没判成：钩子 bash 那一层起不来\\"]}" "$name"\n')   # bash 那一层起不来时的检出：%s 是钩子名
    with open(os.path.join(fake_hooks, "fake-split-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write("# 入口：记检出用的起头常量在同层的拆出文件里\n")
    with open(os.path.join(fake_hooks, "fake-split-guard-decisions.py"), "w", encoding="utf-8") as handle:
        handle.write('SPLIT_UNJUDGED_PREFIX = "拆分闸没判、照常放行"\n')   # 钩子拆成入口加同层部分文件时，起头只在部分文件里
    checks += 1
    fake = hook_unjudged_prefixes(fake_hooks)
    if "新闸没判、照常放行" not in fake or "新闸没判、照常放行".startswith(INFO_DETECTION_PREFIXES) \
            or "fake-printf-guard.sh 没判成" not in fake or "fake-printf-guard.sh 没判成".startswith(INFO_DETECTION_PREFIXES):
        failures.append(f"造一个带新起头的假钩子（常量一份、printf 的 %s 钩子名一份），抽法应当抽出「新闸没判、照常放行」与「fake-printf-guard.sh 没判成」、而表里都没有"
                        f"（这样第一格才会红），实际抽出 {fake}")
    checks += 1
    if fake.get("拆分闸没判、照常放行") != ["fake-split-guard-decisions.py"] or "拆分闸没判、照常放行".startswith(INFO_DETECTION_PREFIXES):
        failures.append(f"造一个拆成入口加同层部分文件的假钩子（起头常量只在部分文件里），抽法应当从部分文件抽出「拆分闸没判、照常放行」、而表里没有"
                        f"（拆分族的部分文件读不到，漏登就不会红），实际抽出 {fake}")
    path = os.path.join(work, "detections-unjudged.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for prefix in UNJUDGED_DETECTION_PREFIXES:
            separator = "" if prefix.endswith(" ") else "："
            handle.write(json.dumps({"session_id": "s1", "agent_type": "主 agent", "agent_id": None, "command": "x",
                                     "findings": [f"{prefix}{separator}读不到共用库（样本）"], "outcome": "只记"}, ensure_ascii=False) + "\n")
        handle.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1", "command": "pgrep -f x",
                                 "findings": [f"{UNJUDGED_DETECTION_PREFIXES[0]}：样本", "命令里有 pgrep -f / pkill -f / killall"], "outcome": "只记"}, ensure_ascii=False) + "\n")
    checks += 1
    _, alerts, notes = read_detections(path, 0, {"s1"}, set())
    quiet_notes = [note for note in notes if "只记不叫醒" in note]
    if len(alerts) != 1 or len(quiet_notes) != len(UNJUDGED_DETECTION_PREFIXES):
        failures.append(f"UNJUDGED_DETECTION_PREFIXES 里 {len(UNJUDGED_DETECTION_PREFIXES)} 个起头各一条检出应当都只记进报告不叫醒，夹着真检出的那一条照旧叫醒（共 1 条告警），"
                        f"实际告警 {len(alerts)} 条、只记 {len(quiet_notes)} 条")
    return checks


def selftest_ack_scopes(failures):
    """--ack 的类范围（split_acknowledged、alert_keys、pruned_acks）：类:<告警名> 确认这一类告警的全部，不分子 agent、不分进程；进程:<pid> 只确认那一个进程；
    记进状态目录的类确认在告警还在时留着、告警没了作废；「进度文件不涨」归只记不叫醒的那一类。返回查了几格。"""
    checks = 0
    alerts = [(ROOT_FREE_SUBJECT, ROOT_FREE_ALERT, "工作根所在的文件系统只剩 40.0 GiB", "清理"),
              ("a1", LEFTOVER_BACKGROUND_ALERT, "进程 333 已跑 3 分钟，sleep 900", "停下它"),
              ("a2", LEFTOVER_BACKGROUND_ALERT, "进程 444 已跑 4 分钟，sleep 800", "停下它")]
    for label, acknowledgements, wanted in (
            (f"类:{ROOT_FREE_ALERT}", {f"类:{ROOT_FREE_ALERT}"}, [ROOT_FREE_ALERT]),
            (f"类:{LEFTOVER_BACKGROUND_ALERT}", {f"类:{LEFTOVER_BACKGROUND_ALERT}"}, [LEFTOVER_BACKGROUND_ALERT, LEFTOVER_BACKGROUND_ALERT]),
            ("进程:333", {"进程:333"}, [LEFTOVER_BACKGROUND_ALERT]),
            ("类里没有的告警名", {"类:不存在的告警"}, [])):
        checks += 1
        remaining, acknowledged = split_acknowledged(list(alerts), acknowledgements)
        got = sorted(alert[1] for alert in acknowledged)
        if got != sorted(wanted) or len(remaining) + len(acknowledged) != len(alerts):
            failures.append(f"--ack {label} 应当只确认告警 {sorted(wanted) or '无'}（别的类、别的进程的照报），实际确认了 {got or '无'}")
    checks += 1
    kept = pruned_acks({f"类:{ROOT_FREE_ALERT}", "类:不存在的告警"}, alerts, set())
    if kept != {f"类:{ROOT_FREE_ALERT}"}:
        failures.append(f"记进状态目录的 类: 确认应当在告警还在时留着、告警没了就作废，实际留下 {sorted(kept)}")
    checks += 1
    remaining, informational = split_informational([("a3", PROGRESS_STALE_ALERT, "结束本轮在等自己的后台任务，progress.md 超过 30 分钟没改", "看进程"), alerts[0]])
    if len(remaining) != 1 or len(informational) != 1 or PROGRESS_STALE_ALERT not in informational[0]:
        failures.append(f"「{PROGRESS_STALE_ALERT}」应当归只记不叫醒的那一类（报告里写成「只记不叫醒：…」，不进告警），实际剩下告警 {len(remaining)} 条、只记 {informational}")
    return checks


SELFTEST_SCRATCH_PREFIX = "agent-watch-selftest-scratch-"
SELFTEST_SCRATCH_OWNER_FILE = ".selftest-pid"
STALE_SELFTEST_SCRATCH_SECONDS = 3600


def remove_stale_selftest_scratch(parent):
    """上一次自证被 SIGKILL 打断时留在 parent 里的 agent-watch-selftest-scratch-* 目录：里面 .selftest-pid 写的进程已不在的删掉；
    没有 .selftest-pid 的最后修改超过 STALE_SELFTEST_SCRATCH_SECONDS 的删掉；进程还在的、没到时候的不动。交回删掉的名字。
    弄坏开关 AGENT_WATCH_BREAK=staleselftestscratch 下一个都不删。"""
    removed = []
    if BROKEN_DETECTION == "staleselftestscratch":
        return removed
    try:
        names = sorted(os.listdir(parent))
    except OSError:
        return removed
    for name in names:
        path = os.path.join(parent, name)
        if not name.startswith(SELFTEST_SCRATCH_PREFIX) or os.path.islink(path) or not os.path.isdir(path):
            continue
        try:
            with open(os.path.join(path, SELFTEST_SCRATCH_OWNER_FILE), encoding="utf-8") as handle:
                owner = int(handle.read().strip())
        except (OSError, ValueError):
            owner = None
        if owner is not None:
            if os.path.exists(f"/proc/{owner}"):
                continue
        else:
            try:
                if time.time() - os.path.getmtime(path) < STALE_SELFTEST_SCRATCH_SECONDS:
                    continue
            except OSError:
                continue
        shutil.rmtree(path, ignore_errors=True)
        removed.append(name)
    return removed


def selftest_ack_concurrency(work, failures):
    """watch-acks.tsv 两只同时写：两个线程各记一条确认，读与写之间都停 ACK_UPDATE_PAUSE_SECONDS（拉开窗口，不加锁时两只都读到空的、后写的冲掉先写的），
    写完两条都要在。flock 按打开的文件判，同一进程里两次 open 的锁照样互斥，与两个看门狗进程同一个判法。AGENT_WATCH_BREAK=acksunlocked 下丢一条：这一格红。返回查了几格。"""
    global ACK_UPDATE_PAUSE_SECONDS
    state_dir = os.path.join(work, "ack-concurrency-state")
    wanted = {"concurrenta:工具调用过长", "进程:4242"}
    ACK_UPDATE_PAUSE_SECONDS = 1.5   # 线程起得慢（负载高）时窗口也要罩得住后起的那一只
    try:
        writers = [threading.Thread(target=update_acks, args=(state_dir, lambda acks, key=key: acks | {key})) for key in sorted(wanted)]
        for writer in writers:
            writer.start()
        for writer in writers:
            writer.join()
    finally:
        ACK_UPDATE_PAUSE_SECONDS = 0.0
    got = read_acks(state_dir)
    if got != wanted:
        failures.append(f"watch-acks.tsv 两只同时写：两条确认 {sorted(wanted)} 都应当在（读改写拿着 watch-acks.lock 的 flock），实际 {sorted(got) or '一条都没有'}")
    return 1


def selftest_stale_scratch_cleanup(work, failures):
    """被杀的自证留下的 agent-watch-selftest-scratch-*：进程已不在的、没有主人记号而超过一小时的删掉；进程还在的（这个自证自己）、没到一小时的、别的名字的不动。返回查了几格。"""
    parent = os.path.join(work, "stale-scratch-parent")
    os.makedirs(parent)
    finished = subprocess.Popen(["true"])
    finished.wait()
    for suffix, owner in (("dead-owner", finished.pid), ("live-owner", os.getpid())):
        os.makedirs(os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix))
        with open(os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix, SELFTEST_SCRATCH_OWNER_FILE), "w", encoding="utf-8") as handle:
            handle.write(f"{owner}\n")
    for suffix, age in (("old-no-owner", 2 * STALE_SELFTEST_SCRATCH_SECONDS), ("fresh-no-owner", 0)):
        made = os.path.join(parent, SELFTEST_SCRATCH_PREFIX + suffix)
        os.makedirs(made)
        os.utime(made, (time.time() - age, time.time() - age))
    os.makedirs(os.path.join(parent, "agent-watch-selftest-other-kept"))
    removed = remove_stale_selftest_scratch(parent)
    wanted = [SELFTEST_SCRATCH_PREFIX + "dead-owner", SELFTEST_SCRATCH_PREFIX + "old-no-owner"]
    if removed != wanted or sorted(os.listdir(parent)) != sorted([SELFTEST_SCRATCH_PREFIX + "live-owner", SELFTEST_SCRATCH_PREFIX + "fresh-no-owner",
                                                                   "agent-watch-selftest-other-kept"]):
        failures.append(f"被杀的自证留下的 {SELFTEST_SCRATCH_PREFIX}*：应当只删主人进程已不在的与没有主人记号而超过一小时的（{wanted}），"
                        f"主人还在的、没到一小时的、别的名字的不动，实际删了 {removed}，剩 {sorted(os.listdir(parent))}")
    return 1


def selftest_session_watch(work, thresholds, failures, watch_runs):
    """这一轮加的几样：「该交接」只报两类、「进度文件不涨」、整点询问附进度、hook 检出主体是 agent 号、--discover 单例与确认落文件。返回查了几格。"""
    session = os.path.join(work, "session-discover")
    # 草稿目录要在工作根下（不看弄坏开关，configured_work_root），派发提示里的路径才认得出（SCRATCH_PATH_MENTION）；它在自检的 TMPDIR 之外，自己删
    scratch_parent = work_roots.configured_work_root()
    os.makedirs(scratch_parent, exist_ok=True)
    remove_stale_selftest_scratch(scratch_parent)   # 上一次被杀的自证留下的：自证开跑先收掉
    scratch = tempfile.mkdtemp(prefix=SELFTEST_SCRATCH_PREFIX, dir=scratch_parent)
    with open(os.path.join(scratch, SELFTEST_SCRATCH_OWNER_FILE), "w", encoding="utf-8") as owner_handle:
        owner_handle.write(f"{os.getpid()}\n")
    try:
        return selftest_session_watch_in(work, session, scratch, thresholds, failures, watch_runs)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def selftest_projects_root(work, failures):
    """没给 --session-dir 时按 id 找会话记录（find_transcript → projects_root）：在一棵临时树里，HOME 指到别处、脚本自己所在的仓也在别处，
    TASK_SERVER_SOURCE_REPOSITORY 指到来源仓，要找到来源仓会话目录里的那一份（HOME 指到的别处有一份同 id 的，不能选它）；
    AGENT_WATCH_PROJECTS_ROOT 设了仍优先（自证造假项目目录靠它）。返回查了几格。"""
    agent_id = "afeedbeef7654321"
    tree = session_directories.build_selftest_tree(os.path.join(work, "projects-root-tree"))
    wanted = os.path.join(tree["session_directory"], "selftest-session", "subagents", f"agent-{agent_id}.jsonl")
    decoy = os.path.join(tree["other_home"], ".claude", "projects", session_directories.mangle(tree["own_repository"]), "selftest-session", "subagents", f"agent-{agent_id}.jsonl")
    for path in (wanted, decoy):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("{}\n")
    found = find_transcript(agent_id, environment=tree["environment"])
    if found != wanted:
        failures.append(f"按 id 找会话记录取来源仓根的会话目录：HOME 指到别处时应当找到 {wanted}，实际 {found}")
    overridden = find_transcript(agent_id, environment=dict(tree["environment"], AGENT_WATCH_PROJECTS_ROOT=os.path.join(tree["other_home"], ".claude", "projects")))
    if overridden != decoy:
        failures.append(f"AGENT_WATCH_PROJECTS_ROOT 设了优先（自证造假项目目录用）：应当找到 {decoy}，实际 {overridden}")
    return 2


def selftest_scratch_mentions(failures):
    """派发提示行首「草稿目录：」那一行点名的草稿目录：工作根（展开的绝对路径与 ~ 写法）下的找得到 progress.md，会话临时目录根下的不认（取不到）；
    提示里另点名的别的子 agent 的目录不取，没有那一行的取不到。建的目录自己删。返回查了几格。"""
    class Prompt:
        def __init__(self, text):
            self.dispatch_prompt = text
    home = os.path.expanduser("~")
    checks, made = 0, []
    try:
        for root in (work_roots.configured_work_root(), work_roots.claude_session_temporary_root()):
            os.makedirs(root, exist_ok=True)
            made.append(tempfile.mkdtemp(prefix="agent-watch-selftest-mention-", dir=root))
            open(os.path.join(made[-1], PROGRESS_FILE_NAME), "w", encoding="utf-8").write("第一格跑完\n")
        new_directory, session_temporary_directory = made
        home_form = "~" + new_directory[len(home):] if new_directory.startswith(home + "/") else new_directory
        for label, written, wanted in (("派发提示点名工作根下的草稿目录（绝对路径）", new_directory, new_directory),
                                       ("派发提示点名工作根下的草稿目录（~ 写法）", home_form, new_directory),
                                       ("派发提示点名会话临时目录根下的草稿目录不认", session_temporary_directory, None)):
            checks += 1
            found = progress_file_of(Prompt(f"跑前登记 x。\n草稿目录：{written}/\n报告：{written}/result.md\n"))
            expected = None if wanted is None else os.path.join(wanted, PROGRESS_FILE_NAME)
            if found != expected:
                failures.append(f"{label}：应当找到 {expected}，实际 {found}")
        # 只认行首「草稿目录：」那一行（lib_prompt_scratch_line.py）：提示里另点名别的子 agent 的目录、它的 progress.md 改得更晚，也不取它；
        # 没有那一行的取不到、告警里写取不到（AGENT_WATCH_BREAK=progressanypath 下取改得最晚的那一份，会话临时目录根下的草稿目录也认得出：那一格与下面这三格共四格红）
        made.append(tempfile.mkdtemp(prefix="agent-watch-selftest-mention-other-", dir=work_roots.configured_work_root()))
        other_directory = made[-1]
        open(os.path.join(other_directory, PROGRESS_FILE_NAME), "w", encoding="utf-8").write("别的子 agent 的一格\n")
        older = time.time() - 600
        os.utime(os.path.join(new_directory, PROGRESS_FILE_NAME), (older, older))
        for label, text, wanted in (
                ("派发提示有本提示的草稿目录一行、另叫它先读别的子 agent 的报告（那边的 progress.md 改得更晚）：应当只取本提示草稿目录下的",
                 f"跑前登记 x。\n草稿目录：{new_directory}\n先读上一个实现员的报告 {other_directory}/report.md，它的探针在 {other_directory}/\n",
                 os.path.join(new_directory, PROGRESS_FILE_NAME)),
                ("派发提示没有行首「草稿目录：」一行、只点名别的子 agent 的报告：应当取不到",
                 f"跑前登记 x。报告写 {new_directory}/result.md，先读 {other_directory}/report.md\n", None)):
            checks += 1
            found = progress_file_of(Prompt(text))
            if found != wanted:
                failures.append(f"{label}，应当 {wanted}，实际 {found}")
        checks += 1
        note = progress_note(Prompt(f"跑前登记 x。先读 {other_directory}/report.md\n"), datetime.now(timezone.utc))
        if "没有行首「草稿目录：」一行，取不到 progress.md" not in note:
            failures.append(f"派发提示没有行首「草稿目录：」一行时，告警里附的一句应当写取不到 progress.md，实际「{note}」")
        # 进度文件行数多于 PROGRESS_TAIL_LINES：附的必须是末几行，按文件里的原次序，空行与只有空白的行不算一行
        # （AGENT_WATCH_BREAK=progresstailhead 下取头几行、progresstailreversed 下倒着附、progressblankkept 下末几行里混进空行，这一格都红）。
        # 上面与 selftest_session_watch_in 造的进度文件都恰好 PROGRESS_TAIL_LINES 行、没有空行，头几行与末几行是同一批，判不出取错了哪头，也走不到空行。
        # 这里的进度行中间夹一行空行与一行只有空白的行、末尾跟两行空行：空行算一行的话，末几行里就是空行、点不全该附的那几行。
        # progress_note 读文件的另两支这里不判：读不到修改时刻交「已消失或不可读」那一支由单元用例
        # test_progress_deleted_before_read_is_reported 判；文件在 readable_mtime 之后、open 之前才消失的 except OSError 一支，自证造不出这个窗口；
        # 读不到修改时刻时交 0.0 的改坏，输出与原来逐字相同，没有测试杀得掉。
        total_lines = PROGRESS_TAIL_LINES + 2
        filled = [f"进度行{index:02d}" for index in range(1, total_lines + 1)]
        gaps_after = {total_lines - 2: "\n", total_lines - 1: "   \n"}   # 末几行之间夹的空行与只有空白的行（键是它前面那一行的行号）
        open(os.path.join(new_directory, PROGRESS_FILE_NAME), "w", encoding="utf-8").write(
            "".join(f"{line}\n{gaps_after.get(index, '')}" for index, line in enumerate(filled, 1)) + "\n\n")
        checks += 1
        note = progress_note(Prompt(f"跑前登记 x。\n草稿目录：{new_directory}/\n"), datetime.now(timezone.utc))
        kept, dropped = filled[-PROGRESS_TAIL_LINES:], filled[:-PROGRESS_TAIL_LINES]
        in_order = f"末 {PROGRESS_TAIL_LINES} 行：{' ｜ '.join(kept)}"
        if in_order not in note or any(line in note for line in dropped):
            failures.append(f"进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行、按原次序、空行不算一行（「{in_order}」）、不含更早的（{dropped}），实际「{note}」")
    finally:
        for directory in made:
            shutil.rmtree(directory, ignore_errors=True)
    return checks


def selftest_session_watch_in(work, session, scratch, thresholds, failures, watch_runs):
    checks = 0
    progress_path = os.path.join(scratch, PROGRESS_FILE_NAME)
    open(progress_path, "w", encoding="utf-8").write("第一格跑完\n第二格跑完\n第三格在跑，预计 40 分钟\n")
    stale_moment = time.time() - 45 * 60
    os.utime(progress_path, (stale_moment, stale_moment))
    prompt = record_at(20, "user", f"跑前登记 x。报告写 {scratch}/result.md\n草稿目录：{scratch}/\n")
    old_prompt = record_at(65, "user", f"跑前登记 x。报告写 {scratch}/result.md\n草稿目录：{scratch}/\n")
    waiting_tail = [bash_use(10, "t1", "cargo run --release", "m1"),
                    record_at(9.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: bq1."}]),
                    record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
    write_transcript(session, "progressstale", [prompt, *waiting_tail], {"agentType": "experiment-runner", "description": "等后台任务、进度 45 分钟没改"})
    write_transcript(session, "attackhuge", [record_at(0.5, "assistant", [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}],
                                                       stop_reason="tool_use", message_id="m1", usage={"input_tokens": 10, "cache_read_input_tokens": 800_000})],
                     {"agentType": "three-way-attack", "description": "攻方上下文 80 万"})
    write_transcript(session, "inquiryprogress", [old_prompt, bash_use(64.9, "t1", "ls", "m1"), bash_result(64.8, "t1"), bash_use(0.5, "t2", "cargo build", "m2")],
                     {"agentType": "experiment-runner", "description": "跑了 65 分钟、草稿目录里有进度文件"})
    open(session + ".jsonl", "a").close()
    now = datetime.now(timezone.utc)
    for agent_id, wanted in (("progressstale", {PROGRESS_STALE_ALERT}), ("attackhuge", set())):
        got = {name for name, _, _ in agent_alerts(AgentTranscript(agent_id, find_transcript(agent_id, session)), now, thresholds)}
        checks += 1
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    inquiry = routine_inquiry_alert(AgentTranscript("inquiryprogress", find_transcript("inquiryprogress", session)), now, thresholds)
    checks += 1
    if inquiry is None or "第三格在跑，预计 40 分钟" not in inquiry[1]:
        failures.append(f"整点询问应当附上 progress.md 的末几行，实际 {inquiry}")
    detections = os.path.join(work, "detections-subject.jsonl")
    with open(detections, "w", encoding="utf-8") as detections_file:
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "Write /x/.claude/kb/a.md", "findings": ["写被拒：越出写范围"]}, ensure_ascii=False) + "\n")
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "pgrep -f cargo", "findings": ["命令里有 pgrep -f / pkill -f / killall"]}, ensure_ascii=False) + "\n")
        detections_file.write(json.dumps({"session_id": "s1", "agent_type": "implementation-writer", "agent_id": "a1",
                                          "command": "bash x.sh", "findings": ["项目代码闸没判全：x.sh 读不进来"]}, ensure_ascii=False) + "\n")
    _, detection_alerts, detection_notes = read_detections(detections, 0, {"s1"}, set())
    checks += 1
    if [alert[0] for alert in detection_alerts] != ["a1"] or not any("只记不叫醒" in note and "写被拒" in note for note in detection_notes) \
            or not any("只记不叫醒" in note and "项目代码闸没判全" in note for note in detection_notes):
        failures.append("hook 检出：写被拒的与项目代码闸没判全的应当只记进报告不叫醒，按模式找进程的应当告警且主体是 agent 号 a1（--ack a1:hook 检出 只压它），"
                        f"实际告警主体 {[alert[0] for alert in detection_alerts]}，记录 {detection_notes}")
    # 钩子执行前已拒掉的检出（写了结局「被拒」的 b1，没写结局而发现以「…被拒」起头的旧记录 b2）只进报告、不叫醒；写了「只记」的 b3 照常告警，下一步说命令照常在跑
    # （AGENT_WATCH_BREAK=outcomeignored 下 b1、b2 被当成只记而叫醒；refusedwakes 下它们照旧叫醒：这一格都红）
    outcome_detections = os.path.join(work, "detections-outcome.jsonl")
    with open(outcome_detections, "w", encoding="utf-8") as detections_file:
        for agent_id, findings, outcome in (("b1", ["不经 task-server 跑项目代码被拒（implementation-writer）：cargo test 带 --workspace / --all"], "被拒"),
                                            ("b2", ["认不出起什么的写法被拒：命令词由变量拼出，执行前展开不了"], None),
                                            ("b3", ["命令里有 pgrep -f / pkill -f / killall"], "只记")):
            entry = {"session_id": "s1", "agent_type": "implementation-writer", "agent_id": agent_id, "command": "x", "findings": findings}
            if outcome:
                entry["outcome"] = outcome
            detections_file.write(json.dumps(entry, ensure_ascii=False) + "\n")
    _, outcome_alerts, outcome_notes = read_detections(outcome_detections, 0, {"s1"}, set())
    next_steps = {alert[0]: alert[3] for alert in outcome_alerts}
    checks += 1
    if not (sorted(next_steps) == ["b3"] and "照常在跑" in next_steps.get("b3", "")
            and all(any(f"implementation-writer {agent_id}：" in note and "执行前已被 hook 拒了、没有执行，只记不叫醒" in note for note in outcome_notes)
                    for agent_id in ("b1", "b2"))):
        failures.append("hook 检出：执行前已被 hook 拒了的（b1 写了结局「被拒」、b2 是没写结局的旧记录）应当只记进报告、不叫醒，"
                        f"只记的（b3）照常告警、下一步说命令照常在跑，实际告警 {sorted(next_steps)}、下一步 {next_steps}、记录 {outcome_notes}")
    # 范围声明钩子的放行记录（scope_session_id、run_scope）：属于这一会话的 c1 与主 agent 那一条列进「范围声明」一节（命令、四段），不叫醒、不进检出记录那几行；
    # 别的会话的 c2 不列（AGENT_WATCH_BREAK=scopeunlisted 下一条都不列：这一格红）
    scope_detections = os.path.join(work, "detections-scope.jsonl")
    with open(scope_detections, "w", encoding="utf-8") as detections_file:
        for session_id, agent_id, agent_type, command in (
                ("s1", "c1", "implementation-writer", "bash .claude/gate.d/formal-verification.sh --item proof /w"),
                ("s1", None, "主 agent", "bash .claude/gate.d/doc-text.sh --check prime-marks"),
                ("s9", "c2", "implementation-writer", "bash scripts/crash/crash-point-replay.sh --item fixed-script")):
            detections_file.write(json.dumps({"hook": "run-scope-guard.sh", "agent_id": agent_id, "agent_type": agent_type, "command": command,
                                              "scope_session_id": session_id, "outcome": "放行",
                                              "run_scope": {"where": "命令文本", "kind_of_command": "items", "what": "样本",
                                                            "declaration": "种类=items; 跑什么=样本; 范围=1 项; 为什么=只改了这一道的判法所以只跑点名的这一项",
                                                            "segments": {"种类": "items", "跑什么": f"样本 {agent_id}", "范围": "1 项",
                                                                         "为什么": "只改了这一道的判法所以只跑点名的这一项"}}}, ensure_ascii=False) + "\n")
    scope_lines = []
    _, scope_alerts, scope_notes = read_detections(scope_detections, 0, {"s1"}, set(), scope_lines)
    section = scope_declaration_section(scope_lines)
    checks += 1
    if not (not scope_alerts and not scope_notes and len(section) == 3 and section[0].startswith("范围声明（run-scope-guard.sh 放行的 2 条")
            and "implementation-writer c1：bash .claude/gate.d/formal-verification.sh --item proof /w：种类=items；跑什么=样本 c1；范围=1 项；为什么=只改了这一道的判法所以只跑点名的这一项" in section[1]
            and section[2].startswith("  主 agent：bash .claude/gate.d/doc-text.sh --check prime-marks：种类=items")
            and not any("c2" in line for line in section)):
        failures.append("范围声明：这一会话放行的两条（c1、主 agent）应当列进「范围声明」一节、每条一行写命令与四段，不叫醒、不进检出记录那几行，别的会话的 c2 不列，"
                        f"实际告警 {scope_alerts}、记录 {scope_notes}、这一节 {section}")
    # --discover：状态目录放自检自己的；一个会话一个看门狗，--ack 落文件下一次读回，条件消失的确认作废
    state_root = os.path.join(work, "watch-state")
    environment = dict(os.environ, AGENT_WATCH_STATE_ROOT=state_root)
    write_transcript(session, "discoveredlong", [prompt, bash_use(19, "t1", "cargo test --release", "m1")],
                     {"agentType": "implementation-writer", "description": "工具调用 20 分钟"})
    write_transcript(session, "discovereddone", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "sweep", "description": "交回了"})
    # 这一段的看门狗都带 --idle-minutes 600：progressstale 最后一条记录写在 9.8 分钟前，默认 10 分钟的线写完 12 秒就过；
    # AGENT_WATCH_BREAK=backgroundwait 下等后台任务的也报「无动静」，负载下看门狗起得晚就当场退 3，这一段要判的锁与 --ack 判不到（判它的是上面 progressstale 那一格）
    quiet_idle = ["--idle-minutes", "600"]

    def discover_watch(extra, timeout=90):
        watch_runs.append(["--discover", *extra])
        return subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "watch", *SELFTEST_SECTION_OFF, "--discover", "--session-dir", session, "--process-root-pid", "0",
                               "--interval-seconds", "1", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle, *extra],
                              capture_output=True, text=True, timeout=timeout, env=environment)
    acknowledged_keys = ["discoveredlong:工具调用过长", "progressstale:进度文件不涨", "inquiryprogress:跑满 1 小时", "progressstale:跑满 1 小时",
                         "attackhuge:该交接", "inquiryprogress:该交接"]
    first = discover_watch([*sum((["--ack", key] for key in acknowledged_keys), []), "--max-minutes", "0.03"])
    checks += 1
    if first.returncode != 4 or "discoveredlong" not in first.stdout or "还没交回、没被停的子 agent" not in first.stdout:
        failures.append(f"--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4，实际 {first.returncode}：{first.stdout[-500:]}")
    if "   → 接着盯就用 run_in_background 再起一个看门狗：bash scripts/agents/watch.sh\n" not in first.stdout:
        failures.append(f"--discover 的出路句应当是不带 agent 号的 bash scripts/agents/watch.sh（一个会话一个看门狗），实际：{first.stdout[-300:]}")
    second = discover_watch(["--max-minutes", "0.03"])
    checks += 1
    if second.returncode != 4 or "已确认接着盯（--ack discoveredlong:工具调用过长）" not in second.stdout:
        failures.append(f"--ack 应当落进状态目录、下一次起不带 --ack 也读回，实际 {second.returncode}：{second.stdout[-400:]}")
    # 在盯的那一个拿到锁、做完第一次整查（按它建的就绪文件等，不按固定秒数）之后再起第二个；第二个查完就停它（自己起的进程号），不等它到点。
    # 负载高时 python 起得慢：固定等 2 秒时它可能还没拿到锁，回报周期 9 秒时第二个起来之前它可能已经到点退了
    holder_ready = os.path.join(work, "ready-discover-holder")
    # 在盯的那一个盯 10 分钟（查完由 finally 停它）：负载下第二个起得慢，盯 1 分钟时它可能先到点退了
    holder = subprocess.Popen([sys.executable, ENTRY_SCRIPT_PATH, "watch", *SELFTEST_SECTION_OFF, "--discover", "--session-dir", session, "--process-root-pid", "0",
                               "--interval-seconds", "1", "--max-minutes", "10", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=dict(environment, **{SELFTEST_READY_VARIABLE: holder_ready}))
    try:
        wait_for_selftest_ready(holder_ready, holder)
        duplicate = discover_watch(["--ack", "attackhuge:hook 检出", "--max-minutes", "0.03"])
        checks += 1
        if duplicate.returncode != SINGLE_INSTANCE_EXIT or "已有一个看门狗在盯" not in duplicate.stdout:
            failures.append(f"同一个会话已有一个看门狗在盯时应当退 {SINGLE_INSTANCE_EXIT} 并写明，实际 {duplicate.returncode}：{duplicate.stdout[-300:]}")
        # 第二个记下的 --ack 要经得起在盯的那一个下一次整查（它每次整查把确认按条件剪一遍再写回，os.replace 换新 inode）：等它写回过一次再读，
        # 不按读的早晚碰运气（AGENT_WATCH_BREAK=discovernone 下它认不出 attackhuge 没结束，这一条确认被剪掉，这一格红）
        acks_path = os.path.join(state_root, os.path.basename(session), "watch-acks.tsv")
        written_by_duplicate = os.stat(acks_path).st_ino if os.path.exists(acks_path) else None
        wait_until(lambda: holder.poll() is not None or (os.path.exists(acks_path) and os.stat(acks_path).st_ino != written_by_duplicate))
        checks += 1
        if "attackhuge:hook 检出" not in read_acks(os.path.join(state_root, os.path.basename(session))):
            failures.append("已有一个在盯时给的 --ack 应当记进状态目录，好让在盯的那一个读到")
    finally:
        holder.terminate()
        holder.communicate(timeout=60)
    # 会话里的子 agent 都交回了：--discover 没有要盯的，接着盯到点（新派的会接上），不退回「会话里最近动过的全部」、不报「全部交回」
    quiet_session = os.path.join(work, "session-all-done")
    write_transcript(quiet_session, "onlydone", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "sweep", "description": "交回了"})
    open(quiet_session + ".jsonl", "a").close()
    watch_runs.append(["--discover", "all-done"])
    quiet = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "watch", *SELFTEST_SECTION_OFF, "--discover", "--session-dir", quiet_session, "--process-root-pid", "0",
                            "--interval-seconds", "1", "--max-minutes", "0.03", "--detections-file", os.path.join(work, "no-detections.jsonl"), *quiet_idle],
                           capture_output=True, text=True, timeout=90, env=environment)
    checks += 1
    if quiet.returncode != 4 or "全部交回" in quiet.stdout:
        failures.append(f"--discover 时会话里的子 agent 都交回了应当接着盯到点退 4、不报全部交回，实际 {quiet.returncode}：{quiet.stdout[-300:]}")
    state_acks = read_acks(os.path.join(state_root, os.path.basename(session)))   # 子进程的 STATE_ROOT 是环境变量给的这一个，自检进程自己的不是
    checks += 1
    if "inquiryprogress:该交接" in state_acks or "discoveredlong:工具调用过长" not in state_acks:
        failures.append(f"条件类告警的确认：条件不在（inquiryprogress 没到线）的应当作废，还在的（discoveredlong 工具过长）应当留着，实际 {sorted(state_acks)}")
    return checks
