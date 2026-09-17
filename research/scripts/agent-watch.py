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
退出码：0 全部子 agent 已结束且没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。

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


def parse_timestamp(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00"))


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
        self.state = "思考中"
        self._read()

    def _read(self):
        seen_message_ids = set()
        pending_by_id = {}
        tool_inputs_by_id = {}
        previous_call_time = None
        last_kind = None
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
                            last_kind = "tool_use"
                        elif block.get("type") == "text":
                            last_kind = "end_turn" if message.get("stop_reason") == "end_turn" else "text"
            elif role == "user":
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_result":
                            pending_by_id.pop(block.get("tool_use_id"), None)
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
        self.pending_tool = max(pending_by_id.values(), key=lambda item: item[0]) if pending_by_id else None
        if last_kind == "interrupted":
            self.state = "被停"
        elif last_kind == "end_turn" and not pending_by_id and BROKEN_DETECTION != "finished":
            self.state = "已结束"
        elif self.pending_tool is not None:
            self.state = "执行工具中"
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
        return self.state in ("已结束", "被停")


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
    elif (transcript.last_timestamp is not None and BROKEN_DETECTION != "idle"
          and (now - transcript.last_timestamp).total_seconds() >= thresholds.idle_minutes * 60):
        alerts.append(("无动静", f"最后一条记录在 {format_duration((now - transcript.last_timestamp).total_seconds())} 以前，没有工具在跑",
                       "主 agent 判断：模型调用是不是卡住或撞了限额；看会话记录最后几条，再决定等、发消息续做、还是停掉重派"))
    recent_entries = transcript.bash_commands[-10:]
    recent_commands = [entry[1] for entry in recent_entries]
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
        for command in recent_commands:
            if BANNED_COMMAND_PATTERN.search(command):
                alerts.append(("禁用命令", f"用了按模式匹配的进程命令：{command[:160]}",
                               "pgrep -f / pkill -f 会命中自己所在的 shell（command-safety.md）；主 agent 判断它等的或要杀的是哪个进程，再发消息让它改用写死的 pid"))
                break
    return alerts


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
    lines, alerts = [], []
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
            alerts.append(("进程无输出", f"进程 {pid} 已跑 {format_duration(elapsed)}，写着的文件 {format_duration(newest_age)} 没变：{info['command'][:140]}",
                           "输出不涨不一定是卡住（有的测试只在结束时写）；主 agent 看它的 CPU 在不在涨、预期多久，再决定接着盯、查、还是报给用户"))
        elif elapsed >= thresholds.process_max_minutes * 60:
            alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}：{info['command'][:140]}",
                           "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
    return lines, alerts


def build_report(agent_ids, session_dir, thresholds, watch_started, excluded_pids, process_root_pid):
    now = datetime.now(timezone.utc)
    lines, alerts, done_flags = [], [], []
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
            if waited >= thresholds.not_started_minutes * 60:
                alerts.append((agent_id, "没启动", f"派发 {format_duration(waited)} 之后还没有会话记录", "查派发是否失败、agent id 是否抄对"))
            continue
        transcript = AgentTranscript(agent_id, path)
        done_flags.append(transcript.is_done())
        since_last = format_duration((now - transcript.last_timestamp).total_seconds()) if transcript.last_timestamp else "?"
        total = format_duration((transcript.last_timestamp - transcript.first_timestamp).total_seconds()) if transcript.first_timestamp else "?"
        lines.append(f"子 agent {agent_id} {transcript.agent_type}「{transcript.description}」状态={transcript.state} 已跑 {total} "
                     f"最后动作 {since_last} 前 调用 {transcript.model_calls} 次 上下文 {transcript.last_context_tokens // 1000}k 缓存整份重写 {transcript.full_cache_rewrites} 次")
        if transcript.pending_tool is not None and not transcript.is_done():
            started, tool_name, detail = transcript.pending_tool
            lines.append(f"  正在跑 {tool_name}（{format_duration((now - started).total_seconds())}）：{' '.join(detail.split())[:200]}")
        for name, explanation, next_step in agent_alerts(transcript, now, thresholds):
            alerts.append((agent_id, name, explanation, next_step))
    if process_root_pid is not None:
        process_lines, found = process_alerts(process_root_pid, excluded_pids, thresholds)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        for name, explanation, next_step in found:
            alerts.append(("进程", name, explanation, next_step))
    return lines, alerts, bool(done_flags) and all(done_flags)


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


def read_detections(detections_path, start_offset, session_ids):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话的告警])。"""
    alerts = []
    if not os.path.isfile(detections_path):
        return start_offset, alerts
    with open(detections_path, encoding="utf-8", errors="replace") as handle:
        handle.seek(start_offset)
        for line in handle:
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            alerts.append((entry.get("agent_type") or "?", "hook 检出",
                           f"{'、'.join(entry.get('findings') or [])}：{(entry.get('command') or '')[:160]}",
                           "hook 只记不拦，命令照常在跑；主 agent 判断它会不会出问题，再决定接着盯、发消息让它改、还是处理"))
        return handle.tell(), alerts


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


def own_process_chain(table, start_pid):
    chain, pid = set(), start_pid
    while pid in table and pid > 1:
        chain.add(pid)
        pid = table[pid]["parent"]
    return chain


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
    while True:
        lines, alerts, all_done = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
        session_ids = {session_id_of(path) for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
        detections_offset, detection_alerts = read_detections(detections_path, detections_offset, session_ids)
        alerts += detection_alerts
        if arguments.mode == "report":
            print_report(lines, alerts)
            return 3 if alerts else 0
        if alerts:
            print_report(lines, alerts)
            return 3
        if all_done:
            print_report(lines, alerts)
            print("被看的子 agent 全部结束。")
            return 0
        if time.time() - watch_started >= arguments.max_minutes * 60:
            print_report(lines, alerts)
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警、子 agent 还在跑。主 agent 看一眼上面的状态，接着盯就再起一个看门狗。")
            return 4
        next_full_check = time.time() + arguments.interval_seconds
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts = read_detections(detections_path, detections_offset, session_ids)
            if detection_alerts:
                lines, alerts, _ = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
                print_report(lines, alerts + detection_alerts)
                return 3


def write_transcript(directory, agent_id, records, meta=None):
    os.makedirs(os.path.join(directory, "subagents"), exist_ok=True)
    path = os.path.join(directory, "subagents", f"agent-{agent_id}.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    if meta:
        json.dump(meta, open(path[: -len(".jsonl")] + ".meta.json", "w", encoding="utf-8"), ensure_ascii=False)


def iso_minutes_ago(minutes):
    return (datetime.now(timezone.utc).timestamp() - minutes * 60)


def record_at(minutes_ago, role, content, stop_reason=None, message_id=None, usage=None):
    stamp = datetime.fromtimestamp(iso_minutes_ago(minutes_ago), timezone.utc).isoformat().replace("+00:00", "Z")
    message = {"role": role, "content": content}
    if stop_reason:
        message["stop_reason"] = stop_reason
    if message_id:
        message["id"] = message_id
    if usage:
        message["usage"] = usage
    return {"timestamp": stamp, "type": role, "message": message}


def bash_use(minutes_ago, tool_id, command, message_id):
    return record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "Bash", "input": {"command": command}}],
                     stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000})


def bash_result(minutes_ago, tool_id):
    return record_at(minutes_ago, "user", [{"type": "tool_result", "tool_use_id": tool_id, "content": "ok"}])


def selftest():
    work = tempfile.mkdtemp(prefix="agent-watch-selftest-")
    thresholds = argparse.Namespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3,
                                    process_report_minutes=0.02, process_stale_minutes=0.05, process_max_minutes=600,
                                    not_started_minutes=5, active_minutes=600)
    session = os.path.join(work, "session")
    failures = []
    write_transcript(session, "finished", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"),
                                           record_at(18, "assistant", [{"type": "text", "text": "交回"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "experiment-runner", "description": "跑完的"})
    write_transcript(session, "waitloop", [bash_use(5, "t1", 'until grep -q "^exit=" log; do sleep 10; done', "m1")])
    write_transcript(session, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")])
    write_transcript(session, "healthy", [bash_use(0.5, "t1", "cargo build", "m1")])
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
    write_transcript(session, "idle", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1")])
    write_transcript(session, "interrupted", [bash_use(40, "t1", "ls", "m1"), bash_result(39, "t1"),
                                              record_at(39, "user", [{"type": "text", "text": "[Request interrupted by user]"}])])
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
        "finished": set(), "healthy": set(), "boundedloop": set(), "interrupted": set(), "repeatchanging": set(),
        "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "idle": {"无动静"},
    }
    now = datetime.now(timezone.utc)
    for agent_id, wanted in expectations.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        got = {name for name, _, _ in agent_alerts(transcript, now, thresholds)}
        if got != wanted:
            failures.append(f"子 agent {agent_id}：应当告警 {sorted(wanted) or '无'}，实际 {sorted(got) or '无'}")
    if AgentTranscript("finished", find_transcript("finished", session)).state != "已结束":
        failures.append("跑完的子 agent 没被认成「已结束」")
    stale_file = os.path.join(work, "stale-output.log")
    open(stale_file, "w").close()
    old = time.time() - 3600
    os.utime(stale_file, (old, old))
    sleeper = subprocess.Popen(["bash", "-c", f"exec 3>>'{stale_file}'; touch -d '1 hour ago' '{stale_file}'; exec sleep 30"])
    try:
        time.sleep(4)
        _, found = process_alerts(os.getpid(), set(), thresholds)
        if not any(name == "进程无输出" and str(sleeper.pid) in explanation for name, explanation, _ in found):
            failures.append(f"写着的文件一小时没动的进程 {sleeper.pid} 没报「进程无输出」")
    finally:
        sleeper.kill()
        sleeper.wait()
    watch_exit = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--agents", "finished,interrupted",
                                 "--session-dir", session, "--interval-seconds", "1", "--max-minutes", "1", "--process-root-pid", "0"],
                                capture_output=True, text=True).returncode
    if watch_exit != 0:
        failures.append(f"被看的子 agent 都已结束时看门狗应当退出码 0，实际 {watch_exit}")
    watch_exit = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--agents", "waitloop",
                                 "--session-dir", session, "--interval-seconds", "1", "--max-minutes", "1", "--process-root-pid", "0"],
                                capture_output=True, text=True).returncode
    if watch_exit != 3:
        failures.append(f"有告警时看门狗应当退出码 3，实际 {watch_exit}")
    watch_exit = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--agents", "healthy",
                                 "--session-dir", session, "--interval-seconds", "1", "--max-minutes", "0.03", "--process-root-pid", "0"],
                                capture_output=True, text=True).returncode
    if watch_exit != 4:
        failures.append(f"到寿命上限时看门狗应当退出码 4，实际 {watch_exit}")
    detections_file = os.path.join(work, "detections.jsonl")
    open(detections_file, "w").close()
    for session_label, wanted_exit in (("session", 3), ("另一个会话", 4)):
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--agents", "healthy", "--session-dir", session,
                                    "--interval-seconds", "2", "--detection-poll-seconds", "0.5", "--max-minutes", "0.12",
                                    "--process-root-pid", "0", "--detections-file", detections_file],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        time.sleep(1.5)
        with open(detections_file, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({"session_id": session_label, "agent_type": "experiment-runner",
                                     "command": "pgrep -f x", "findings": ["按模式匹配的进程命令"]}, ensure_ascii=False) + "\n")
        output, _ = watcher.communicate(timeout=60)
        if watcher.returncode != wanted_exit or (wanted_exit == 3 and "hook 检出" not in output):
            failures.append(f"检出记录属于「{session_label}」时看门狗应当退出码 {wanted_exit}，实际 {watcher.returncode}")
    subprocess.run(["rm", "-rf", work])
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 agent_alerts() / process_alerts() / run() 的判法；AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
          f"跑完、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，看门狗三种退出码对，本会话的 hook 检出会叫醒主 agent、别的会话的不会，用量与工具结果分类对（查了 {len(expectations) + 1} 个子 agent、1 个进程、5 次看门狗）")
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
    parser.add_argument("--process-report-minutes", type=float, default=5, help="进程跑过这么久才列出来，默认 5 分钟")
    parser.add_argument("--process-stale-minutes", type=float, default=20, help="进程写着的文件这么久没动告警，默认 20 分钟")
    parser.add_argument("--process-max-minutes", type=float, default=120, help="进程跑过这么久告警，默认 120 分钟")
    parser.add_argument("--not-started-minutes", type=float, default=5, help="派发之后这么久还没有会话记录告警，默认 5 分钟")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认 /tmp/claude-1000/agent-hook-detections.jsonl")
    parser.add_argument("--detection-poll-seconds", type=float, default=15, help="watch 两次读检出记录的间隔，默认 15 秒")
    parser.add_argument("--process-root-pid", type=int, help="进程一半从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例，0 表示不看进程")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
