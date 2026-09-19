#!/usr/bin/env python3
"""子 agent 与它们起的进程的定时监控：卡死、空转、永久等待当场报出来。

为什么：2026-09-17 一个实验执行员把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`，
等一行永远不会出现的字，主 agent 查进度才发现；同一天另一个会话的层 0 全量测试跑了三个多小时没有任何输出，
没人分得清是慢还是卡。经过：records/2026-09-17-已分配口径三方与两个实验.md 第六节。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py --selftest                         # 造假的会话记录与进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它只看、只报，不停任何子 agent、不杀任何进程：子 agent 与脚本不强制结束，结束不结束由主 agent 看了报告定（用户 2026-09-17）。
退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。
「交回」按交回工具（SubagentHandback）成功返回判：只结束本轮、没交回的子 agent 还在等后台任务，不算结束。
「被停」认两处：子 agent 会话记录里的 `[Request interrupted`（停在工具调用中途）；主会话记录（<会话 id>.jsonl）里 TaskStop 对它的成功结果，
时刻不早于它自己最后一条记录减 30 秒（停在结束本轮、等后台任务的时候，子 agent 的会话记录里一个字都不写；停了之后又被续做的不算）。
主会话记录不在那个位置时，报告里写明「被停只按打断判」，不悄悄退回。
「结束本轮却不会醒」：子 agent 结束本轮、没交回，而它起过的后台任务（工具结果「Command running in background with ID: …」）都已收到完成通知
（会话记录里 origin 为 task-notification 的消息，或忙时排进队的 queued_command 附件），结束本轮之后也没有新的通知进来——没有任何东西会叫醒它，
过 60 秒就报，不等 10 分钟的「无动静」。

hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 只记不拦，把检出写进检出记录；
watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。

会话记录的位置：~/.claude/projects/<项目>/<会话 id>/subagents/agent-<id>.jsonl，同名 .meta.json 里有 agentType 与 description。
进程这一半只看运行这个脚本的那个 Claude 实例的后代进程（主 agent 与它派出的子 agent 起的命令都在里面），不看别的会话。
"""
import argparse
import glob
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

PROJECTS_ROOT = os.path.expanduser("~/.claude/projects")
DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
WAIT_LOOP_PATTERN = re.compile(r"\b(until|while)\b[\s\S]*\bsleep\b")
BANNED_COMMAND_PATTERN = re.compile(r"\bpgrep\s+-f\b|\bpkill\s+-f\b|\bkillall\b")
BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")
# 禁用命令只报这个窗口里发出的：看门狗每次起都从头读会话记录，窗外的旧命令会让每一次重起当场退出、再也盯不住那个 agent。
BANNED_COMMAND_WINDOW_SECONDS = 600
HEREDOC_START_PATTERN = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")


def command_without_heredoc_bodies(command):
    """去掉 heredoc 的正文，只留真会执行的那几行：往报告里写的一句「pgrep -f 会命中自己」不是在用它。"""
    if BROKEN_DETECTION == "heredoc":
        return command
    kept, terminators = [], []
    for line in command.split("\n"):
        if terminators:
            if line.strip() == terminators[0]:
                terminators.pop(0)
            continue
        kept.append(line)
        terminators.extend(match.group(2) for match in HEREDOC_START_PATTERN.finditer(line))
    return "\n".join(kept)
STOP_AFTER_LAST_RECORD_SECONDS = 30  # 停在工具中途时，被停之后还可能落几条收尾记录
NO_WAKE_GRACE_SECONDS = 60           # 结束本轮之后，完成通知可能还在路上
TURN_END_STOP_REASONS = ("end_turn", "stop_sequence")   # 会话记录里结束本轮的两种写法（2026-09-19 实测两种都有）
TEXT_SETTLE_SECONDS = 120            # 最后一条是纯文字、stop_reason 为空：流式写入时后面可能还接工具调用，过了这么久没接就是结束本轮
CPU_SAMPLE_SECONDS = 2               # 写着的文件不动的进程，隔这么久再取一次 CPU 时间，分「在算」与「不动」
BACKGROUND_STARTED = re.compile(r"^Command running in background with ID: (\w+)")
BACKGROUND_OUTPUT_PATH = re.compile(r"Output is being written to: (\S+?\.output)")
TASK_NOTIFICATION_ID = re.compile(r"<task-id>(\w+)</task-id>")


def parse_timestamp(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00"))


def parent_session_transcript(transcript_path):
    """…/<会话 id>/subagents/agent-<id>.jsonl → …/<会话 id>.jsonl（派它的那个会话的记录）。"""
    return os.path.dirname(os.path.dirname(transcript_path)) + ".jsonl"


def task_stop_time(agent_id, session_transcript_path):
    """主会话记录里 TaskStop 停掉这个子 agent 的最晚一次成功结果的时刻；没有就 None。"""
    if not os.path.isfile(session_transcript_path):
        return None
    stopped_at = None
    for line in open(session_transcript_path, encoding="utf-8", errors="replace"):
        if agent_id not in line or "Successfully stopped task" not in line:
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        result = record.get("toolUseResult")
        if not isinstance(result, dict) or result.get("task_id") != agent_id or "Successfully stopped task" not in str(result.get("message", "")):
            continue
        if record.get("timestamp"):
            timestamp = parse_timestamp(record["timestamp"])
            stopped_at = timestamp if stopped_at is None else max(stopped_at, timestamp)
    return stopped_at


def format_duration(seconds):
    seconds = int(max(seconds, 0))
    if seconds >= 3600:
        return f"{seconds // 3600}h{(seconds % 3600) // 60:02d}m"
    if seconds >= 60:
        return f"{seconds // 60}m{seconds % 60:02d}s"
    return f"{seconds}s"


class AgentTranscript:
    """一个子 agent 会话记录的摘要：状态、最后一条命令、调用次数、上下文、缓存整份重写次数。"""

    def __init__(self, agent_id, transcript_path):
        self.agent_id = agent_id
        self.transcript_path = transcript_path
        self.agent_type = "?"
        self.description = ""
        meta_path = transcript_path[: -len(".jsonl")] + ".meta.json"
        if os.path.isfile(meta_path):
            try:
                meta = json.load(open(meta_path, encoding="utf-8"))
                self.agent_type = meta.get("agentType", "?")
                self.description = meta.get("description", "")
            except (OSError, ValueError):
                pass
        self.first_timestamp = None
        self.last_timestamp = None
        self.model_calls = 0
        self.last_context_tokens = 0
        self.full_cache_rewrites = 0
        self.full_cache_rewrite_tokens = 0
        self.first_context_tokens = 0
        self.total_context_tokens = 0
        self.fresh_input_tokens = 0
        self.cache_read_tokens = 0
        self.output_tokens = 0
        self.tool_result_characters = {}  # 类别 → [次数, 字符数]
        self.read_characters_by_file = {}  # 文件名 → [次数, 字符数]
        self.bash_commands = []          # [[时间, 命令, 输出（还没回来是 None）, 工具调用 id]]
        self.pending_tool = None         # (时间, 工具名, 命令或路径) —— 发出了还没收到结果
        self.background_started = []     # 它用 run_in_background 起过的后台任务 id
        self.background_output_paths = {}  # 后台任务 id → harness 写它输出的文件（工具结果里带着）
        self.background_finished = set() # 收到过完成通知的后台任务 id
        self.last_turn_end_timestamp = None
        self.last_notification_timestamp = None
        self.state = "思考中"
        self._read()
        parent_path = parent_session_transcript(transcript_path)
        self.parent_transcript_missing = not os.path.isfile(parent_path)
        self.parent_transcript_path = parent_path
        stopped_at = task_stop_time(agent_id, parent_path)
        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            self.state = "被停"

    def _read(self):
        seen_message_ids = set()
        pending_by_id = {}
        tool_inputs_by_id = {}
        handback_tool_ids = set()
        is_handed_back = False  # 最近一次交回成功之后没有再调别的工具（续做之后又调工具就回到没交回）
        previous_call_time = None
        last_kind = None
        last_text_timestamp = None
        for line in open(self.transcript_path, encoding="utf-8", errors="replace"):
            try:
                record = json.loads(line)
            except ValueError:
                continue
            timestamp_text = record.get("timestamp")
            if not timestamp_text:
                continue
            timestamp = parse_timestamp(timestamp_text)
            self.first_timestamp = self.first_timestamp or timestamp
            self.last_timestamp = timestamp
            attachment = record.get("attachment")
            is_notification = ((record.get("origin") or {}).get("kind") == "task-notification"
                               or (isinstance(attachment, dict) and attachment.get("type") == "queued_command" and "<task-notification>" in json.dumps(attachment)))
            if is_notification:
                self.background_finished.update(TASK_NOTIFICATION_ID.findall(json.dumps(record.get("message") or attachment, ensure_ascii=False)))
                self.last_notification_timestamp = timestamp
            message = record.get("message")
            if not isinstance(message, dict):
                continue
            role = message.get("role")
            content = message.get("content")
            if role == "assistant":
                usage = message.get("usage") or {}
                message_id = message.get("id")
                if usage and message_id not in seen_message_ids:
                    seen_message_ids.add(message_id)
                    self.model_calls += 1
                    created = usage.get("cache_creation_input_tokens", 0) or 0
                    read_from_cache = usage.get("cache_read_input_tokens", 0) or 0
                    uncached = usage.get("input_tokens", 0) or 0
                    self.last_context_tokens = read_from_cache + created + uncached
                    self.first_context_tokens = self.first_context_tokens or self.last_context_tokens
                    self.total_context_tokens += self.last_context_tokens
                    self.fresh_input_tokens += created + uncached
                    self.cache_read_tokens += read_from_cache
                    self.output_tokens += usage.get("output_tokens", 0) or 0
                    if created > 100_000 and previous_call_time is not None and (timestamp - previous_call_time).total_seconds() >= 300:
                        self.full_cache_rewrites += 1
                        self.full_cache_rewrite_tokens += created
                    previous_call_time = timestamp
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_use":
                            tool_input = block.get("input") or {}
                            detail = tool_input.get("command") or tool_input.get("file_path") or tool_input.get("description") or ""
                            pending_by_id[block.get("id")] = (timestamp, block.get("name"), detail)
                            tool_inputs_by_id[block.get("id")] = (block.get("name"), tool_input)
                            if block.get("name") == "Bash":
                                self.bash_commands.append([timestamp, tool_input.get("command", ""), None, block.get("id")])
                            if block.get("name") == "SubagentHandback":
                                handback_tool_ids.add(block.get("id"))
                            else:
                                is_handed_back = False
                            last_kind = "tool_use"
                        elif block.get("type") == "text":
                            last_kind = "end_turn" if message.get("stop_reason") in TURN_END_STOP_REASONS else "text"
                            last_text_timestamp = timestamp
                            if last_kind == "end_turn":
                                self.last_turn_end_timestamp = timestamp
            elif role == "user":
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_result":
                            pending_by_id.pop(block.get("tool_use_id"), None)
                            started = BACKGROUND_STARTED.match(block.get("content")) if isinstance(block.get("content"), str) else None
                            if started:
                                self.background_started.append(started.group(1))
                                output_path = BACKGROUND_OUTPUT_PATH.search(block.get("content"))
                                if output_path:
                                    self.background_output_paths[started.group(1)] = output_path.group(1)
                            if block.get("tool_use_id") in handback_tool_ids:
                                result_text = json.dumps(block.get("content"), ensure_ascii=False).replace("\\", "").replace(" ", "")
                                if '"success":true' in result_text:
                                    is_handed_back = True
                            if block.get("tool_use_id") in tool_inputs_by_id:
                                self._count_tool_result(*tool_inputs_by_id[block.get("tool_use_id")], block.get("content"))
                            for entry in self.bash_commands[-20:]:
                                if entry[3] == block.get("tool_use_id"):
                                    result = block.get("content")
                                    entry[2] = result if isinstance(result, str) else json.dumps(result, ensure_ascii=False)
                            last_kind = "tool_result"
                        elif block.get("type") == "text" and "[Request interrupted" in block.get("text", ""):
                            last_kind = "interrupted"
                elif isinstance(content, str) and "[Request interrupted" in content:
                    last_kind = "interrupted"
                # 交回之后来的续做消息（origin 是 coordinator）：还没调工具也已经不算交回——续做的头几秒它在想，看门狗不能当场报「全部交回」退出。
                # 后台任务的完成通知（origin 是 task-notification）不算：它叫醒之后要是又调工具，上面按调工具撤销。
                if (record.get("origin") or {}).get("kind") == "coordinator" and BROKEN_DETECTION != "resume":
                    is_handed_back = False
                # 结束本轮之后来的续做消息：它被叫醒了，在想下一步，不再是「结束本轮」——不改的话续做头几秒它还没落下新记录，
                # 看门狗照上一轮的结束时刻报「结束本轮却不会醒」（2026-09-19 E155 执行员续派第二段，最后动作 4 秒前就被报了）。
                if (record.get("origin") or {}).get("kind") == "coordinator" and BROKEN_DETECTION != "resumeturn":
                    last_kind = "resumed"
        self.pending_tool = max(pending_by_id.values(), key=lambda item: item[0]) if pending_by_id else None
        # 最后一条是纯文字、stop_reason 为空：流式写入时同一条消息后面还会接工具调用，几秒之内就会落下来；
        # 过了 TEXT_SETTLE_SECONDS 还没接任何记录，就是结束本轮了（2026-09-19 c381-r1 攻方腿 10:57 写一句「等 phase B」就等后台任务，
        # 最后两条记录 stop_reason 都是空的，看门狗判成「思考中」，12 分钟后报「无动静」）。
        if (last_kind == "text" and last_text_timestamp is not None and BROKEN_DETECTION != "settle"
                and (datetime.now(timezone.utc) - last_text_timestamp).total_seconds() >= TEXT_SETTLE_SECONDS):
            last_kind = "end_turn"
            self.last_turn_end_timestamp = last_text_timestamp
        if last_kind == "interrupted":
            self.state = "被停"
        elif is_handed_back and not pending_by_id and BROKEN_DETECTION != "finished":
            self.state = "已交回"
        elif self.pending_tool is not None:
            self.state = "执行工具中"
        elif last_kind == "end_turn" and BROKEN_DETECTION == "handback":
            self.state = "已交回"
        elif last_kind == "end_turn":
            self.state = "本轮结束、没交回"
        else:
            self.state = "思考中"

    def _count_tool_result(self, tool_name, tool_input, content):
        text = content if isinstance(content, str) else json.dumps(content, ensure_ascii=False)
        category = tool_result_category(tool_name, tool_input)
        entry = self.tool_result_characters.setdefault(category, [0, 0])
        entry[0] += 1
        entry[1] += len(text)
        if tool_name == "Read":
            file_entry = self.read_characters_by_file.setdefault(os.path.basename(tool_input.get("file_path", "")), [0, 0])
            file_entry[0] += 1
            file_entry[1] += len(text)

    def is_done(self):
        return self.state in ("已交回", "被停")

    def live_background_tasks(self):
        return [task_id for task_id in self.background_started if task_id not in self.background_finished]

    def orphaned_background_tasks(self):
        """起过、没收到完成通知，而 harness 写它输出的那个文件还在、却没有任何进程开着它：进程已经没了，通知不会再来。
        2026-09-19 c381-r1 攻方腿撞会话限额中断，限额之前起的探针任务就停在这个样子，看门狗一直把它当「还在跑」。
        输出文件路径不知道、或文件不在的，判不了，照旧算还在跑。"""
        if BROKEN_DETECTION == "orphan":
            return []
        candidates = [task_id for task_id in self.live_background_tasks()
                      if os.path.isfile(self.background_output_paths.get(task_id, ""))]
        if not candidates:
            return []
        held = files_held_open_by_any_process()
        return [task_id for task_id in candidates if os.path.realpath(self.background_output_paths[task_id]) not in held]

    def effectively_live_background_tasks(self):
        orphaned = set(self.orphaned_background_tasks())
        return [task_id for task_id in self.live_background_tasks() if task_id not in orphaned]

    def waiting_on_its_own_background_tasks(self):
        """结束本轮、没交回，手里还有没收到完成通知的后台任务：它在等，会话记录里本来就一个字都不写，醒不醒由那些任务决定。
        那些任务的进程动不动归进程一节判（进程无输出、进程过长），不按会话记录的「无动静」判——
        2026-09-19 c381-r1 攻方腿等两个吃满 CPU 的探针，看门狗每次重起都当场报「无动静」退出，盯不住。"""
        if BROKEN_DETECTION == "backgroundwait":
            return False
        return self.state == "本轮结束、没交回" and bool(self.effectively_live_background_tasks())

    def ended_turn_with_nothing_to_wake_it(self, now):
        """结束本轮、没交回、起过的后台任务都完了，结束之后也没有通知进来，而且已经过了宽限。"""
        if self.state != "本轮结束、没交回" or self.effectively_live_background_tasks() or self.last_turn_end_timestamp is None:
            return False
        if self.last_notification_timestamp is not None and self.last_notification_timestamp >= self.last_turn_end_timestamp:
            return False
        return (now - self.last_turn_end_timestamp).total_seconds() >= NO_WAKE_GRACE_SECONDS


def files_held_open_by_any_process():
    """本机全部进程开着的文件（按真实路径）：判一个后台任务的输出文件还有没有进程在写。"""
    held = set()
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            descriptors = os.listdir(f"/proc/{entry}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                held.add(os.path.realpath(os.readlink(f"/proc/{entry}/fd/{descriptor}")))
            except OSError:
                continue
    return held


def tool_result_category(tool_name, tool_input):
    """工具结果按「撑大上下文的是哪一类读法」分类。"""
    if tool_name == "Read":
        file_path = tool_input.get("file_path", "")
        whole = "offset" not in tool_input and "limit" not in tool_input
        if "prereg" in file_path:
            kind = "Read 跑前登记"
        elif file_path.endswith(".rs"):
            kind = "Read 源码"
        elif file_path.endswith(".md"):
            kind = "Read 其他 md"
        else:
            kind = "Read 其他"
        return kind + ("（整份）" if whole else "（分段）")
    if tool_name == "Bash":
        command = tool_input.get("command", "")
        if "mutate.sh" in command:
            return "Bash 变异表"
        if re.search(r"\bcargo\b", command):
            return "Bash cargo"
        if re.search(r"\b(sed -n|cat|head|tail)\b", command):
            return "Bash 看文件（sed/cat/head/tail）"
        if re.search(r"\bgrep\b", command):
            return "Bash grep"
        return "Bash 其他"
    return tool_name


def format_tokens(count):
    if count >= 100_000_000:
        return f"{count / 100_000_000:.2f} 亿"
    if count >= 10_000:
        return f"{count / 10_000:.1f} 万"
    return str(count)


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


def find_transcript(agent_id, session_dir=None):
    if session_dir:
        candidate = os.path.join(session_dir, "subagents", f"agent-{agent_id}.jsonl")
        return candidate if os.path.isfile(candidate) else None
    matches = glob.glob(os.path.join(PROJECTS_ROOT, "*", "*", "subagents", f"agent-{agent_id}.jsonl"))
    return matches[0] if matches else None


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
                           "主 agent 判断：这条命令起的进程还在不在动（本报告的进程一节）、预期还要多久；动就接着盯，不动再决定怎么处理"))
    elif BROKEN_DETECTION != "nowake" and transcript.ended_turn_with_nothing_to_wake_it(now):
        orphaned = transcript.orphaned_background_tasks()
        orphan_text = (f"（其中 {', '.join(orphaned)} 没收到完成通知，但它的输出文件已经没有进程开着，通知不会再来）" if orphaned else "")
        alerts.append(("结束本轮却不会醒", f"结束本轮、没交回已 {format_duration((now - transcript.last_turn_end_timestamp).total_seconds())}，"
                       f"起过的 {len(transcript.background_started)} 个后台任务都已结束{orphan_text}，没有东西会叫醒它",
                       "主 agent 看它最后在等什么：发消息让它接着做或交回，或者停掉；多半是它把活自己放了后台（run_in_background 里又加了 &）、忘了交回，"
                       "或者它等的后台任务在一次中断（会话限额、进程被杀）里没了"))
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and not transcript.waiting_on_its_own_background_tasks()
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或撞了限额；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
    # 上下文越过一条线：每次调用都要把整段上下文从缓存再读一遍，一旦隔久了还要整份重写，越往后每一步越贵，逼近窗口上限还会被迫压缩。
    # 2026-09-19 实测：同一个执行员被续派三次（第一段、补判、第二段），上下文涨到 95 万，平均每次调用 62.6 万、累计读缓存 2.36 亿，
    # 真正的输出只有 2.9 万——没有任何告警，是用户问起才看见的。
    # 这条只叫主 agent 来看，不是停的理由（用户同日：「不应该限制 280K……应该告诉主 agent 看看任务是不是异常，如果不是就继续跑」）。
    if BROKEN_DETECTION != "context" and transcript.last_context_tokens >= thresholds.context_tokens:
        alerts.append(("上下文过大", f"最后一次调用的上下文 {transcript.last_context_tokens / 10000:.1f} 万（线在 {thresholds.context_tokens / 10000:.0f} 万），"
                       f"已调用 {transcript.model_calls} 次",
                       "主 agent 看任务是不是异常：在不在原地打转（同一批文件反复读、同一条命令反复跑、产物不涨）、是不是在做派发之外的事；"
                       "不异常就用 --ack 确认这条告警、让它接着跑，上下文大小本身不是停它的理由；异常再决定发消息让它改、还是停掉新开"))
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
    2026-09-19：报告写「进程动不动见下面进程一节」，而进程一节只列跑满 --process-report-minutes 的叶子进程——
    批次脚本一轮轮起一分钟的探针，一个都不够格，那一节根本不出现，主 agent 只好自己去翻 ps。"""
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
    # 用户 2026-09-19 看到「累计 11 小时 CPU」以为是重跑的，并明说累计没有意义。
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


def process_table():
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            stat_text = open(f"/proc/{entry}/stat").read()
            command_line = open(f"/proc/{entry}/cmdline", "rb").read().replace(b"\0", b" ").decode(errors="replace").strip()
        except OSError:
            continue
        after_name = stat_text[stat_text.rfind(")") + 2:].split()
        table[int(entry)] = {"parent": int(after_name[1]), "start_ticks": int(after_name[19]),
                             "cpu_ticks": int(after_name[11]) + int(after_name[12]), "command": command_line}
    return table


def claude_instance_pid(table, start_pid):
    pid = start_pid
    while pid in table and pid > 1:
        if "native-binary/claude" in table[pid]["command"] or os.path.basename(table[pid]["command"].split(" ")[0]) == "claude":
            return pid
        pid = table[pid]["parent"]
    return None


def written_files(pid):
    files = []
    try:
        descriptors = os.listdir(f"/proc/{pid}/fd")
    except OSError:
        return files
    for descriptor in descriptors:
        try:
            target = os.readlink(f"/proc/{pid}/fd/{descriptor}")
            flags_line = [line for line in open(f"/proc/{pid}/fdinfo/{descriptor}") if line.startswith("flags:")][0]
        except (OSError, IndexError):
            continue
        access_mode = int(flags_line.split()[1], 8) & 0o3
        if target.startswith("/") and access_mode in (1, 2) and os.path.isfile(target):
            files.append(target)
    return sorted(set(files))


def process_alerts(root_pid, excluded_pids, thresholds):
    """这个 Claude 实例底下跑得久的叶子进程：报跑了多久、写的文件多久没动；过期的告警。"""
    table = process_table()
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    descendants, frontier = [], [root_pid]
    while frontier:
        pid = frontier.pop()
        for child in children.get(pid, []):
            descendants.append(child)
            frontier.append(child)
    lines, alerts, quiet = [], [], []
    for pid in sorted(descendants):
        if pid in excluded_pids or children.get(pid):
            continue
        info = table[pid]
        if any(marker in info["command"] for marker in ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")):
            continue
        elapsed = uptime_seconds - info["start_ticks"] / ticks_per_second
        if elapsed < thresholds.process_report_minutes * 60:
            continue
        files = written_files(pid)
        newest_age = min((time.time() - os.path.getmtime(path) for path in files), default=None)
        age_text = "没有写着的文件" if newest_age is None else f"写着的文件最近一次变动在 {format_duration(newest_age)} 以前"
        lines.append(f"  进程 {pid} 已跑 {format_duration(elapsed)}，{age_text}：{info['command'][:140]}")
        if BROKEN_DETECTION != "stale" and newest_age is not None and elapsed >= thresholds.process_stale_minutes * 60 \
                and newest_age >= thresholds.process_stale_minutes * 60:
            quiet.append((pid, elapsed, newest_age, info))
        elif elapsed >= thresholds.process_max_minutes * 60:
            alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}：{info['command'][:140]}",
                           "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
    # 写着的文件不动的进程，过一小会儿再取一次 CPU 时间：CPU 在涨是在算、只是不报进度（有的测试只在结束时写），报一行、不告警；
    # CPU 也不涨才是可能卡住了，告警。2026-09-19 c381-r1 攻方腿两个吃满 CPU 的探针 21 分钟不写一个字，每个看门狗一起来就被它当场叫醒。
    if quiet:
        time.sleep(CPU_SAMPLE_SECONDS)
        later = process_table()
    for pid, elapsed, newest_age, info in quiet:
        if pid not in later or later[pid]["start_ticks"] != info["start_ticks"]:
            continue   # 采样那两秒里跑完了（或 pid 被别的进程接走）
        cpu_seconds = (later[pid]["cpu_ticks"] - info["cpu_ticks"]) / ticks_per_second
        if BROKEN_DETECTION != "cpu" and cpu_seconds > 0:
            lines.append(f"  进程 {pid} 在算：{CPU_SAMPLE_SECONDS} 秒里用了 {cpu_seconds:.1f} 核秒 CPU，只是写着的文件 {format_duration(newest_age)} 没动（不告警；"
                         f"跑到 {thresholds.process_max_minutes:.0f} 分钟报「进程过长」）")
            if elapsed >= thresholds.process_max_minutes * 60:
                alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}、一直在算却不写：{info['command'][:140]}",
                               "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
            continue
        alerts.append(("进程无输出", f"进程 {pid} 已跑 {format_duration(elapsed)}，写着的文件 {format_duration(newest_age)} 没变，"
                                     f"{CPU_SAMPLE_SECONDS} 秒里 CPU 也没涨：{info['command'][:140]}",
                       "既不写也不算，可能卡在等什么（锁、管道、网络、另一个进程）；主 agent 看它在等什么（/proc/<pid>/wchan、它的父进程与输入），"
                       "再决定接着盯、查、还是报给用户"))
    return lines, alerts


def build_report(agent_ids, session_dir, thresholds, watch_started, excluded_pids, process_root_pid):
    now = datetime.now(timezone.utc)
    lines, alerts, done_flags, active_ids, unstarted_ids = [], [], [], [], []
    if agent_ids:
        targets = [(agent_id, find_transcript(agent_id, session_dir)) for agent_id in agent_ids]
    else:
        paths = glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))
        recent = [path for path in paths if time.time() - os.path.getmtime(path) <= thresholds.active_minutes * 60]
        targets = [(os.path.basename(path)[len("agent-"):-len(".jsonl")], path) for path in sorted(recent)]
    for agent_id, path in targets:
        if path is None:
            waited = time.time() - watch_started
            lines.append(f"子 agent {agent_id}：还没有会话记录（已等 {format_duration(waited)}）")
            done_flags.append(False)
            unstarted_ids.append(agent_id)
            if waited >= thresholds.not_started_minutes * 60:
                alerts.append((agent_id, "没启动", f"派发 {format_duration(waited)} 之后还没有会话记录", "查派发是否失败、agent id 是否抄对"))
            continue
        transcript = AgentTranscript(agent_id, path)
        done_flags.append(transcript.is_done())
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
    if process_root_pid is not None:
        process_lines, found = process_alerts(process_root_pid, excluded_pids, thresholds)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        for name, explanation, next_step in found:
            alerts.append(("进程", name, explanation, next_step))
    return lines, alerts, bool(done_flags) and all(done_flags), (active_ids, unstarted_ids)


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


def read_detections(detections_path, start_offset, session_ids, watched_agent_ids=()):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话、被看的子 agent 或主 agent 的告警], [只进报告的记录])。"""
    alerts, notes = [], []
    if not os.path.isfile(detections_path):
        return start_offset, alerts, notes
    with open(detections_path, encoding="utf-8", errors="replace") as handle:
        handle.seek(start_offset)
        for line in handle:
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            if (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids
                    and BROKEN_DETECTION != "otheragent"):
                continue   # 同一会话里别的子 agent 的检出归盯它的那个看门狗（2026-09-19：盯攻方腿的看门狗被上游那个 agent 的检出叫醒）
            findings = [str(finding) for finding in (entry.get("findings") or [])]
            if findings and all(finding.startswith("没有 timeout 的等待循环") for finding in findings) and BROKEN_DETECTION != "loopnoise":
                # 只检出等待循环的不立刻叫醒：正常等编译、等全量跑也是这个形状。会话记录那一路在它跑满 --wait-loop-minutes 时再报。
                continue
            refused = any(finding.startswith("写被拒") for finding in findings)
            if entry.get("agent_type") == "主 agent" and refused and BROKEN_DETECTION != "ownrefusal":
                # 主 agent 自己的写被拒：拒绝信息已经在它的工具结果里，再叫醒一次只是打断（2026-09-19 一晚上被自己被拒的命令叫醒两次）。
                # 按模式找进程由上游钩子在执行前拒绝，不写检出记录，这里读不到它。
                notes.append(f"主 agent 自己的命令被 hook 拒了（不叫醒，拒绝信息在它的工具结果里）：{(entry.get('command') or '')[:120]}")
                continue
            alerts.append((entry.get("agent_type") or "?", "hook 检出",
                           f"{'、'.join(entry.get('findings') or [])}：{(entry.get('command') or '')[:160]}",
                           ("这次写被 hook 拒了、没写进去；主 agent 判断是不是该写的，再决定发消息让它改、自己改、还是派该写的 agent"
                            if any(str(finding).startswith("写被拒") for finding in (entry.get("findings") or []))
                            else "检出 hook 只记不拦，命令照常在跑；主 agent 判断它会不会出问题，再决定接着盯、发消息让它改、还是处理")))
        return handle.tell(), alerts, notes


def print_report(lines, alerts):
    now = datetime.now(timezone.utc)
    print(f"[agent-watch {now.strftime('%H:%M:%S')} UTC / {now.astimezone(ZoneInfo('Asia/Tokyo')).strftime('%H:%M:%S')} JST]")
    for line in lines:
        print(line)
    for owner, name, explanation, next_step in alerts:
        print(f"⚠️ {name}（{owner}）：{explanation}")
        print(f"   → 怎么办：{next_step}")
    if not alerts:
        print("没有告警。")


def print_active_list(active_ids, unstarted_ids):
    if not active_ids and not unstarted_ids:
        return
    print(f"还没交回、没被停的子 agent（{len(active_ids)} 个，另有还没会话记录的 {len(unstarted_ids)} 个）：{','.join(active_ids + unstarted_ids)}")
    print(f"   → 接着盯就再起一个看门狗 --agents {','.join(active_ids + unstarted_ids)}")


def own_process_chain(table, start_pid):
    chain, pid = set(), start_pid
    while pid in table and pid > 1:
        chain.add(pid)
        pid = table[pid]["parent"]
    return chain


def split_acknowledged(alerts, acknowledgements):
    """主 agent 用 --ack 确认过「只是慢、接着盯」的告警：只写进报告，不叫醒主 agent。
    2026-09-19：规程写「只是慢就再起一个看门狗接着盯」，而「工具调用过长」这类条件在那次调用结束之前一直成立，
    新起的看门狗当场又报同一句退出，等于盯不住；确认只在这一次看门狗里有效，下一次起不带就失效。"""
    if BROKEN_DETECTION == "ack" or not acknowledgements:
        return alerts, []
    remaining, acknowledged = [], []
    for alert in alerts:
        subject, name, explanation, _ = alert
        keys = {f"{subject}:{name}"}
        if subject == "进程":
            process_id = re.match(r"进程 (\d+) ", explanation)
            if process_id:
                keys.add(f"进程:{process_id.group(1)}")
        (acknowledged if keys & acknowledgements else remaining).append(alert)
    return remaining, acknowledged


def run(arguments):
    if not arguments.agents and not arguments.session_dir:
        print("✗ 要给 --agents 或 --session-dir\n→ 怎么办：派发返回的 agent id 用逗号连起来传 --agents", file=sys.stderr)
        return 2
    agent_ids = [item for item in (arguments.agents or "").split(",") if item]
    table = process_table()
    if arguments.process_root_pid == 0:
        process_root_pid = None
    elif arguments.process_root_pid is not None:
        process_root_pid = arguments.process_root_pid
    else:
        process_root_pid = claude_instance_pid(table, os.getpid())
    excluded_pids = own_process_chain(table, os.getpid())
    if arguments.mode == "cost":
        lines, _ = cost_report(agent_ids, arguments.session_dir, arguments)
        print("\n".join(lines))
        return 0
    watch_started = time.time()
    detections_path = arguments.detections_file
    detections_offset = os.path.getsize(detections_path) if (arguments.mode == "watch" and os.path.isfile(detections_path)) else 0
    report_deadline = watch_started + arguments.max_minutes * 60
    detection_notes = []
    while True:
        lines, alerts, all_done, active_lists = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
        session_ids = {session_id_of(path) for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
        detections_offset, detection_alerts, new_notes = read_detections(detections_path, detections_offset, session_ids, set(agent_ids))
        detection_notes += new_notes
        lines += detection_notes
        alerts += detection_alerts
        alerts, acknowledged = split_acknowledged(alerts, set(arguments.ack or []))
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            print_active_list(*active_lists)
            return 3
        if all_done:
            print_report(lines, alerts)
            print("被看的子 agent 全部交回或被停。")
            return 0
        if time.time() >= report_deadline:
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警。")
            print_active_list(*active_lists)
            return 4
        next_full_check = time.time() + arguments.interval_seconds
        if BROKEN_DETECTION != "deadline":
            next_full_check = min(next_full_check, report_deadline)
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts, new_notes = read_detections(detections_path, detections_offset, session_ids, set(agent_ids))
            detection_notes += new_notes
            if detection_alerts:
                lines, alerts, _, active_lists = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
                print_report(lines + detection_notes, alerts + detection_alerts)
                print_active_list(*active_lists)
                return 3


def write_transcript(directory, agent_id, records, meta=None):
    os.makedirs(os.path.join(directory, "subagents"), exist_ok=True)
    path = os.path.join(directory, "subagents", f"agent-{agent_id}.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    if meta:
        json.dump(meta, open(path[: -len(".jsonl")] + ".meta.json", "w", encoding="utf-8"), ensure_ascii=False)


def write_session_stops(session_directory, minutes_ago_by_agent):
    """主会话记录里 TaskStop 的调用与成功结果，结果的形态照 2026-09-18 主会话记录里的原样。"""
    with open(session_directory + ".jsonl", "w", encoding="utf-8") as handle:
        for index, (agent_id, minutes_ago) in enumerate(minutes_ago_by_agent.items()):
            use = record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": f"s{index}", "name": "TaskStop", "input": {"task_id": agent_id}}],
                            stop_reason="tool_use", message_id=f"ms{index}")
            message = f"Successfully stopped task: {agent_id} (样本)"
            result = record_at(minutes_ago - 0.01, "user", [{"tool_use_id": f"s{index}", "type": "tool_result",
                                                             "content": json.dumps({"message": message, "task_id": agent_id}, ensure_ascii=False)}])
            result["toolUseResult"] = {"message": message, "task_id": agent_id, "task_type": "local_agent", "command": "样本"}
            handle.write(json.dumps(use, ensure_ascii=False) + "\n")
            handle.write(json.dumps(result, ensure_ascii=False) + "\n")


def iso_minutes_ago(minutes):
    return (datetime.now(timezone.utc).timestamp() - minutes * 60)


def record_at(minutes_ago, role, content, stop_reason=None, message_id=None, usage=None, origin_kind=None):
    stamp = datetime.fromtimestamp(iso_minutes_ago(minutes_ago), timezone.utc).isoformat().replace("+00:00", "Z")
    message = {"role": role, "content": content}
    if stop_reason:
        message["stop_reason"] = stop_reason
    if message_id:
        message["id"] = message_id
    if usage:
        message["usage"] = usage
    record = {"timestamp": stamp, "type": role, "message": message}
    if origin_kind:
        record["origin"] = {"kind": origin_kind}
    return record


def bash_use(minutes_ago, tool_id, command, message_id):
    return record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "Bash", "input": {"command": command}}],
                     stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000})


def bash_result(minutes_ago, tool_id):
    return record_at(minutes_ago, "user", [{"type": "tool_result", "tool_use_id": tool_id, "content": "ok"}])


def handback_records(minutes_ago, tool_id, message_id):
    """交回工具的调用与成功返回，返回的形态照 2026-09-18 会话记录里的原样。"""
    return [record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "SubagentHandback", "input": {"message": "报告"}}],
                      stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000}),
            record_at(minutes_ago - 0.1, "user", [{"type": "tool_result", "tool_use_id": tool_id,
                                                   "content": [{"type": "text", "text": '{"success":true,"message":"Report delivered to your caller."}'}]}])]


def selftest():
    work = tempfile.mkdtemp(prefix="agent-watch-selftest-")
    thresholds = argparse.Namespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3, interval_seconds=240,
                                    process_report_minutes=0.02, process_stale_minutes=0.05, process_max_minutes=600,
                                    not_started_minutes=5, active_minutes=600, context_tokens=600_000)
    session = os.path.join(work, "session")
    failures = []
    write_transcript(session, "finished", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18.5, "h1", "m2"),
                                           record_at(18, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "experiment-runner", "description": "跑完的"})
    write_transcript(session, "waiting", [bash_use(1, "t1", "cargo test --release", "m1"),
                                          record_at(0.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: b1."}]),
                                          record_at(0.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "experiment-runner", "description": "结束本轮在等后台任务、没交回"})
    write_transcript(session, "waitinglong", [bash_use(20, "t1", "cargo test --release", "m1"),
                                              record_at(19.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: b7."}]),
                                              record_at(19.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮等自己的后台任务已 20 分钟、没交回（不是无动静）"})
    write_transcript(session, "continued", [*handback_records(3, "h1", "m1"),
                                            record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                            record_at(2, "user", "续做：再补一格"), bash_use(1, "t2", "ls", "m3"), bash_result(0.9, "t2")],
                     {"agentType": "experiment-runner", "description": "交回之后被续做"})
    write_transcript(session, "resumedthinking", [*handback_records(3, "h1", "m1"),
                                                  record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                                  record_at(0.2, "user", "续做：按判决改", origin_kind="coordinator")],
                     {"agentType": "implementation-writer", "description": "交回之后刚被续做、还没调工具"})
    write_transcript(session, "resumedafterturnend", [bash_use(10, "t1", "ls", "m1"), bash_result(9.9, "t1"),
                                                      record_at(9, "assistant", [{"type": "text", "text": "第一段做完了"}], stop_reason="end_turn", message_id="m2"),
                                                      record_at(0.1, "user", "续派第二段", origin_kind="coordinator")],
                     {"agentType": "experiment-runner", "description": "结束本轮没交回、刚被续派、还没落下新记录"})
    write_transcript(session, "notifiedafterhandback", [*handback_records(3, "h1", "m1"),
                                                        record_at(2.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m2"),
                                                        record_at(0.2, "user", "[SYSTEM NOTIFICATION - NOT USER INPUT] 后台任务跑完了", origin_kind="task-notification")],
                     {"agentType": "implementation-writer", "description": "交回之后收到后台任务的完成通知"})
    write_transcript(session, "waitloop", [bash_use(5, "t1", 'until grep -q "^exit=" log; do sleep 10; done', "m1")])
    write_transcript(session, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")])
    write_transcript(session, "healthy", [bash_use(0.5, "t1", "cargo build", "m1")])
    write_transcript(session, "hugecontext", [record_at(0.5, "assistant", [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "cargo build"}}],
                                                        stop_reason="tool_use", message_id="m1", usage={"input_tokens": 10, "cache_read_input_tokens": 700_000})],
                     {"agentType": "experiment-runner", "description": "被续派几次、上下文涨到 70 万"})
    write_transcript(session, "boundedloop", [bash_use(5, "t1", 'timeout 600 bash -c "until grep -q x log; do sleep 10; done"', "m1")])
    repeat_records = []
    for index in range(3):
        repeat_records += [bash_use(3 - index * 0.5, f"r{index}", "tail -5 /tmp/out.log", f"mr{index}"), bash_result(2.9 - index * 0.5, f"r{index}")]
    write_transcript(session, "repeat", repeat_records)
    changing_records = []
    for index in range(3):
        changing_records += [bash_use(3 - index * 0.5, f"g{index}", "tail -5 /tmp/growing.log", f"mg{index}"),
                             record_at(2.9 - index * 0.5, "user", [{"type": "tool_result", "tool_use_id": f"g{index}", "content": f"第 {index} 行"}])]
    write_transcript(session, "repeatchanging", changing_records)
    write_transcript(session, "banned", [bash_use(1, "t1", "pgrep -f e154-binary", "m1"), bash_result(0.9, "t1")])
    write_transcript(session, "bannedrefused", [bash_use(1, "t1", "pgrep -f e154-binary", "m1"),
                                                record_at(0.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "is_error": True,
                                                                         "content": "PreToolUse:Bash hook error: [bash hook]: ✗ 命令里有按模式找进程的写法"}])])
    write_transcript(session, "bannedheredoc", [bash_use(1, "t1", "cat >> report.md <<'EOF'\n等进程别用 pgrep -f，它会命中自己\nEOF\nls", "m1"),
                                                bash_result(0.9, "t1")])
    write_transcript(session, "bannedafterheredoc", [bash_use(1, "t1", "cat > note.md <<EOF\n说明\nEOF\npgrep -f e154-binary", "m1"),
                                                     bash_result(0.9, "t1")])
    write_transcript(session, "bannedold", [bash_use(60, "t1", "pgrep -f e154-binary", "m1"), bash_result(59.9, "t1"),
                                            bash_use(1, "t2", "ls", "m2"), bash_result(0.9, "t2")])
    write_transcript(session, "idle", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1")])
    write_transcript(session, "interrupted", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1"),
                                              record_at(39, "user", [{"type": "text", "text": "[Request interrupted by user]"}])])
    waiting_then_idle = [bash_use(25, "t1", "bash gate.sh --staged", "m1"), bash_result(24.9, "t1"),
                         record_at(24.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")]
    write_transcript(session, "stoppedidle", waiting_then_idle,
                     {"agentType": "gate-triage", "description": "结束本轮等后台任务时被 TaskStop 停掉"})
    write_transcript(session, "stoppedcontinued", [*waiting_then_idle, record_at(2, "user", "续做：再跑一遍"), bash_use(0.5, "t2", "cargo build", "m3")],
                     {"agentType": "gate-triage", "description": "被停之后又被续做"})
    write_transcript(session, "stoppedtwice", [*waiting_then_idle, record_at(18, "user", "续做：再跑一遍"), bash_use(17, "t2", "cargo build", "m3"),
                                               bash_result(16, "t2"), record_at(15.9, "assistant", [{"type": "text", "text": "又在等"}], stop_reason="end_turn", message_id="m4")],
                     {"agentType": "gate-triage", "description": "停了、续做、又停"})
    write_session_stops(session, {"stoppedidle": 20, "stoppedcontinued": 20, "stoppedtwice": 20})
    with open(session + ".jsonl", "a", encoding="utf-8") as handle:   # stoppedtwice 第二次被停：晚于续做之后的最后一条记录
        second = record_at(15, "user", [{"tool_use_id": "s9", "type": "tool_result", "content": "stopped"}])
        second["toolUseResult"] = {"message": "Successfully stopped task: stoppedtwice (样本)", "task_id": "stoppedtwice", "task_type": "local_agent"}
        handle.write(json.dumps(second, ensure_ascii=False) + "\n")
    background_started = record_at(4.9, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                  "content": "Command running in background with ID: bx1. Output is being written to: /tmp/x.output."}])
    finished_while_busy = {"timestamp": record_at(4.85, "user", "")["timestamp"], "type": "attachment",
                           "attachment": {"type": "queued_command", "prompt": "<task-notification>\n<task-id>bx1</task-id>\n<status>completed</status>\n</task-notification>"}}
    write_transcript(session, "nowake", [bash_use(5, "t1", "nohup bash gate.sh > gate.log 2>&1 &", "m1"), background_started, finished_while_busy,
                                         record_at(4.8, "assistant", [{"type": "text", "text": "等后台通知"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "gate-triage", "description": "起的后台任务当场完了、结束本轮去等"})
    woken = record_at(2, "user", "<task-notification>\n<task-id>bx1</task-id>\n<status>completed</status>\n</task-notification>")
    woken["origin"] = {"kind": "task-notification"}
    write_transcript(session, "wokenup", [bash_use(5, "t1", "bash gate.sh > gate.log 2>&1", "m1"), background_started,
                                          record_at(4.8, "assistant", [{"type": "text", "text": "等后台通知"}], stop_reason="end_turn", message_id="m2"), woken],
                     {"agentType": "gate-triage", "description": "等的后台任务完了、通知刚到、正要醒"})
    background_plain = record_at(14.9, "user", [{"type": "tool_result", "tool_use_id": "t1", "content": "Command running in background with ID: bs1."}])
    write_transcript(session, "textsettled", [bash_use(15, "t1", "cargo test --release", "m1"), background_plain,
                                              record_at(12, "assistant", [{"type": "text", "text": "等 phase B 跑完"}], message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮那条文字记录 stop_reason 为空、之后 12 分钟没接任何记录"})
    write_transcript(session, "stopsequence", [bash_use(15, "t1", "cargo test --release", "m1"), background_plain,
                                               record_at(12, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="stop_sequence", message_id="m2")],
                     {"agentType": "three-way-attack", "description": "结束本轮写成 stop_sequence"})
    write_transcript(session, "textstreaming", [bash_use(15, "t1", "ls", "m1"), bash_result(14.9, "t1"),
                                                record_at(0.2, "assistant", [{"type": "text", "text": "接下来跑"}], message_id="m2")],
                     {"agentType": "three-way-attack", "description": "纯文字记录刚落、后面的工具调用还在路上"})
    orphan_output = os.path.join(work, "orphan-task.output")
    open(orphan_output, "w").close()
    held_output = os.path.join(work, "held-task.output")
    open(held_output, "w").close()
    holder = subprocess.Popen(["bash", "-c", f"exec 3>>'{held_output}'; exec sleep 60"])
    for sample_id, output_path in (("orphantask", orphan_output), ("heldtask", held_output)):
        started_record = record_at(9.9, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                  "content": f"Command running in background with ID: bo1. Output is being written to: {output_path}."}])
        write_transcript(session, sample_id, [bash_use(10, "t1", "cargo test --release", "m1"), started_record,
                                              record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                         {"agentType": "three-way-attack", "description": "等的后台任务：输出文件没人开着 / 还有进程开着"})
    os.makedirs(os.path.join(work, "orphan"), exist_ok=True)
    write_transcript(os.path.join(work, "orphan"), "noparent", waiting_then_idle, {"agentType": "gate-triage", "description": "主会话记录不在"})
    if not AgentTranscript("noparent", find_transcript("noparent", os.path.join(work, "orphan"))).parent_transcript_missing:
        failures.append("主会话记录不在时应当标出来（parent_transcript_missing），实际没标")
    write_transcript(session, "costed", [
        record_at(30, "assistant", [{"type": "tool_use", "id": "c1", "name": "Read", "input": {"file_path": "/x/e999-preregistration.md"}}],
                  stop_reason="tool_use", message_id="k1",
                  usage={"input_tokens": 3, "cache_creation_input_tokens": 100_000, "cache_read_input_tokens": 0, "output_tokens": 50}),
        record_at(29.9, "user", [{"type": "tool_result", "tool_use_id": "c1", "content": "登" * 400}]),
        record_at(20, "assistant", [{"type": "tool_use", "id": "c2", "name": "Bash", "input": {"command": "cargo test --release"}}],
                  stop_reason="tool_use", message_id="k2",
                  usage={"input_tokens": 2, "cache_creation_input_tokens": 150_000, "cache_read_input_tokens": 0, "output_tokens": 70}),
        record_at(19.9, "user", [{"type": "tool_result", "tool_use_id": "c2", "content": "ok" * 50}]),
        record_at(19.8, "assistant", [{"type": "text", "text": "交回"}], stop_reason="end_turn", message_id="k3",
                  usage={"input_tokens": 1, "cache_creation_input_tokens": 200, "cache_read_input_tokens": 150_002, "output_tokens": 30}),
    ], {"agentType": "experiment-runner", "description": "用量样本"})
    costed = AgentTranscript("costed", find_transcript("costed", session))
    wanted_cost = {"model_calls": 3, "first_context_tokens": 100_003, "fresh_input_tokens": 250_206, "cache_read_tokens": 150_002,
                   "output_tokens": 150, "full_cache_rewrites": 1, "full_cache_rewrite_tokens": 150_000,
                   "total_context_tokens": 100_003 + 150_002 + 150_203}
    for field, wanted in wanted_cost.items():
        got = getattr(costed, field)
        if BROKEN_DETECTION == "cost" and field == "full_cache_rewrite_tokens":
            got = 0
        if got != wanted:
            failures.append(f"用量样本的 {field} 应当是 {wanted}，实际 {got}")
    if costed.tool_result_characters.get("Read 跑前登记（整份）") != [1, 400] or costed.tool_result_characters.get("Bash cargo") != [1, 100]:
        failures.append(f"工具结果分类不对：{costed.tool_result_characters}")
    expectations = {
        "finished": set(), "healthy": set(), "boundedloop": set(), "interrupted": set(), "repeatchanging": set(), "waiting": set(), "continued": set(),
        "stoppedidle": set(), "stoppedcontinued": set(), "stoppedtwice": set(), "wokenup": set(), "resumedafterturnend": set(), "nowake": {"结束本轮却不会醒"}, "waitinglong": set(), "orphantask": {"结束本轮却不会醒"}, "heldtask": set(), "textsettled": set(), "stopsequence": set(), "textstreaming": set(),
        "hugecontext": {"上下文过大"}, "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "bannedrefused": set(), "bannedold": set(), "bannedheredoc": set(), "bannedafterheredoc": {"禁用命令"}, "idle": {"无动静"},
    }
    now = datetime.now(timezone.utc)
    for agent_id, wanted in expectations.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        got = {name for name, _, _ in agent_alerts(transcript, now, thresholds)}
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    held_lines = background_task_processes(AgentTranscript("heldtask", find_transcript("heldtask", session)))
    if not any(f"进程 {holder.pid} 已跑" in line for line in held_lines):
        failures.append(f"结束本轮等自己的后台任务时应当列出开着那个输出文件的进程 {holder.pid}，实际 {held_lines or '一个都没列'}")
    holder.kill()
    holder.wait()
    wanted_done = {"finished": True, "interrupted": True, "stoppedidle": True, "waiting": False, "continued": False, "healthy": False,
                   "stoppedcontinued": False, "stoppedtwice": True, "nowake": False, "wokenup": False, "resumedthinking": False, "notifiedafterhandback": True,
                   "waitinglong": False}
    for agent_id, wanted_state in (("textsettled", "本轮结束、没交回"), ("stopsequence", "本轮结束、没交回"), ("textstreaming", "思考中"),
                                   ("resumedafterturnend", "思考中")):
        got_state = AgentTranscript(agent_id, find_transcript(agent_id, session)).state
        if got_state != wanted_state:
            failures.append(f"子 agent {agent_id} 的状态应当是「{wanted_state}」，实际「{got_state}」")
    for agent_id, wanted in wanted_done.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        if transcript.is_done() != wanted:
            failures.append(f"子 agent {agent_id} 应当{'算' if wanted else '不算'}交回或被停，实际状态「{transcript.state}」")
    stale_file = os.path.join(work, "stale-output.log")
    open(stale_file, "w").close()
    old = time.time() - 3600
    os.utime(stale_file, (old, old))
    sleeper = subprocess.Popen(["bash", "-c", f"exec 3>>'{stale_file}'; touch -d '1 hour ago' '{stale_file}'; exec sleep 30"])
    busy_file = os.path.join(work, "busy-output.log")
    open(busy_file, "w").close()
    # 在算、只在结束时写的进程：写着的文件一小时没动，CPU 一直在涨（2026-09-19 c381-r1 攻方腿探针的形状）
    busy = subprocess.Popen([sys.executable, "-c", "import os, time\n"
                             f"handle = open({busy_file!r}, 'a')\nos.utime({busy_file!r}, (time.time() - 3600, time.time() - 3600))\n"
                             "deadline = time.time() + 30\nwhile time.time() < deadline:\n    pass\n"])
    try:
        time.sleep(4)
        process_lines, found = process_alerts(os.getpid(), set(), thresholds)
        if not any(name == "进程无输出" and str(sleeper.pid) in explanation for name, explanation, _ in found):
            failures.append(f"写着的文件一小时没动、CPU 也不涨的进程 {sleeper.pid} 没报「进程无输出」")
        if any(str(busy.pid) in explanation for _, explanation, _ in found):
            failures.append(f"写着的文件一小时没动、CPU 一直在涨的进程 {busy.pid} 被当成告警（它在算，只是不报进度）")
        if not any("在算" in line for line in process_lines):
            failures.append(f"CPU 在涨的进程 {busy.pid} 在报告里没有「在算」那一行")
    finally:
        for process in (sleeper, busy):
            process.kill()
            process.wait()
    watch_runs = []

    def run_watch(watch_arguments, timeout=None):
        watch_runs.append(watch_arguments)
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", *watch_arguments, "--session-dir", session, "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=timeout)

    watch_exit = run_watch(["--agents", "finished,interrupted,stoppedidle", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 0:
        failures.append(f"被看的子 agent 都交回或被停时看门狗应当退出码 0，实际 {watch_exit}")
    watch_exit = run_watch(["--agents", "waitloop", "--interval-seconds", "1", "--max-minutes", "1"]).returncode
    if watch_exit != 3:
        failures.append(f"有告警时看门狗应当退出码 3，实际 {watch_exit}")
    watch_exit = run_watch(["--agents", "healthy", "--interval-seconds", "1", "--max-minutes", "0.03"]).returncode
    if watch_exit != 4:
        failures.append(f"到定时回报的时刻看门狗应当退出码 4，实际 {watch_exit}")
    acknowledged_run = run_watch(["--agents", "longtool", "--ack", "longtool:工具调用过长", "--interval-seconds", "1", "--max-minutes", "0.03"])
    if acknowledged_run.returncode != 4 or "已确认接着盯" not in acknowledged_run.stdout:
        failures.append(f"「工具调用过长」被 --ack 确认之后看门狗应当到点退出码 4 并写「已确认接着盯」，实际退出码 {acknowledged_run.returncode}")
    watched = run_watch(["--agents", "waiting,finished", "--interval-seconds", "1", "--max-minutes", "0.03"])
    wanted_active_line = "还没交回、没被停的子 agent（1 个，另有还没会话记录的 0 个）：waiting"
    if watched.returncode != 4 or wanted_active_line not in watched.stdout:
        failures.append(f"一个结束本轮没交回、一个已交回时看门狗应当退出码 4 并列出「{wanted_active_line}」，"
                        f"实际退出码 {watched.returncode}，输出：{watched.stdout[-300:]}")
    try:
        started = time.time()
        watched = run_watch(["--agents", "waiting", "--interval-seconds", "240", "--max-minutes", "0.05"], timeout=30)
        if watched.returncode != 4:
            failures.append(f"复检间隔 240 秒、回报周期 3 秒时看门狗应当到点退出码 4，实际 {watched.returncode}")
    except subprocess.TimeoutExpired:
        failures.append(f"复检间隔比回报周期长时看门狗没有按回报周期退出（{time.time() - started:.0f} 秒还没退出）：回报会被拖到下一次复检")
    detections_file = os.path.join(work, "detections.jsonl")
    open(detections_file, "w").close()
    self_background = "run_in_background 里又自己放后台"
    for session_label, finding_text, agent_type, agent_id, wanted_exit, wanted_note in (
            ("session", self_background, "experiment-runner", "healthy", 3, None),
            ("session", "写被拒", "experiment-runner", "healthy", 3, None),
            ("另一个会话", self_background, "experiment-runner", "healthy", 4, None),
            ("session", "没有 timeout 的等待循环", "experiment-runner", "healthy", 4, None),
            ("session", self_background, "experiment-runner", "别的子agent", 4, None),
            ("session", "写被拒", "主 agent", None, 4, "主 agent 自己的命令被 hook 拒了")):
        watch_runs.append(["--detections-file", session_label, finding_text, agent_type, agent_id])
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--agents", "healthy", "--session-dir", session,
                                    "--interval-seconds", "2", "--detection-poll-seconds", "0.5", "--max-minutes", "0.12",
                                    "--process-root-pid", "0", "--detections-file", detections_file],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        time.sleep(1.5)
        with open(detections_file, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({"session_id": session_label, "agent_type": agent_type, "agent_id": agent_id,
                                     "command": "pgrep -f x", "findings": [finding_text]}, ensure_ascii=False) + "\n")
        output, _ = watcher.communicate(timeout=60)
        if watcher.returncode != wanted_exit or (wanted_exit == 3 and "hook 检出" not in output) \
                or (wanted_note is not None and wanted_note not in output):
            failures.append(f"检出记录属于「{session_label}」「{agent_type} {agent_id}」、检出「{finding_text}」时看门狗应当退出码 {wanted_exit}"
                            f"{f'并在报告里记一行「{wanted_note}」' if wanted_note else ''}，实际 {watcher.returncode}")
    subprocess.run(["rm", "-rf", work])
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 agent_alerts() / process_alerts() / run() 的判法；AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算），"
          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算、续做之后又停的算，主会话记录不在的标出来），"
          f"结束本轮而起过的后台任务都已完成、没有通知在路上的报「结束本轮却不会醒」（通知刚到正要醒的不报），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对；结束本轮认 end_turn 与 stop_sequence、纯文字记录两分钟没接下文也认，结束本轮之后收到续做消息就算在想，用 --ack 确认过的告警只进报告不叫醒，主 agent 自己被拒的写只进报告不叫醒、入口已拒的命令不再按禁用命令报、别的子 agent 的检出不叫醒这一个看门狗，等自己的后台任务时列出开着输出文件的进程，结束本轮等自己的后台任务的不报无动静（输出文件已没有进程开着、又没收到通知的后台任务不算在等，报「结束本轮却不会醒」），写着的文件不动而 CPU 在涨的进程只报「在算」不告警（查了 {len(expectations) + 1} 个子 agent、2 个进程、{len(watch_runs)} 次看门狗）")
    return 0


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "--selftest":
        return selftest()
    parser = argparse.ArgumentParser(description="子 agent 与它们起的进程的定时监控")
    parser.add_argument("mode", choices=["report", "watch", "cost"])
    parser.add_argument("--agents", help="逗号分隔的子 agent id（派发结果里的 agentId）")
    parser.add_argument("--session-dir", help="~/.claude/projects/<项目>/<会话 id>；给了 --agents 时可省")
    parser.add_argument("--interval-seconds", type=float, default=240, help="看门狗两次检查的间隔，默认 240 秒")
    parser.add_argument("--max-minutes", type=float, default=60, help="定时回报：没有告警也每隔这么久退出一次叫醒主 agent，默认 60 分钟")
    parser.add_argument("--tool-minutes", type=float, default=8, help="一次工具调用超过这么久告警，默认 8 分钟")
    parser.add_argument("--wait-loop-minutes", type=float, default=3, help="没有超时的等待循环跑过这么久告警，默认 3 分钟")
    parser.add_argument("--idle-minutes", type=float, default=10, help="没有工具在跑、记录这么久没动告警，默认 10 分钟")
    parser.add_argument("--repeat-count", type=int, default=3, help="最近 10 次命令里同一条出现这么多次告警，默认 3")
    parser.add_argument("--context-tokens", type=int, default=600_000, help="子 agent 最后一次调用的上下文到这么多 token 告警，默认 60 万")
    parser.add_argument("--process-report-minutes", type=float, default=5, help="进程跑过这么久才列出来，默认 5 分钟")
    parser.add_argument("--process-stale-minutes", type=float, default=20, help="进程写着的文件这么久没动告警，默认 20 分钟")
    parser.add_argument("--process-max-minutes", type=float, default=120, help="进程跑过这么久告警，默认 120 分钟")
    parser.add_argument("--not-started-minutes", type=float, default=5, help="派发之后这么久还没有会话记录告警，默认 5 分钟")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认 /tmp/claude-1000/agent-hook-detections.jsonl")
    parser.add_argument("--detection-poll-seconds", type=float, default=15, help="watch 两次读检出记录的间隔，默认 15 秒")
    parser.add_argument("--ack", action="append", default=[],
                        help="主 agent 看过、判定只是慢接着盯的告警：子agent id:告警名，进程告警写 进程:PID；可给多次，只在这一次看门狗里有效")
    parser.add_argument("--process-root-pid", type=int, help="进程一半从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例，0 表示不看进程")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
