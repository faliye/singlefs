"""agent-watch.py 的一块：会话记录与后台任务的读法（交回、被停、结束本轮、新指令）、交回之后后台还在跑、副本还在、草稿根里的编译目录与仓副本、根分区余量，进程表的读法，以及测这几处的自证用例与自证共用的造会话记录的工具。入口 agent-watch.py 导入并再导出这里的名字。"""
import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo
import os as preflight_os, sys as preflight_sys  # noqa: E402
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'hooks'))
import lib_work_root as work_roots  # noqa: E402  工作根与会话临时目录根的唯一定义
import subagent_transcript as transcripts  # noqa: E402  「交回」（送达）的唯一定义，续做闸与 agent-cost.py 取的是同一份
import lib_session_registry as session_registry  # noqa: E402  Claude Code 会话登记（~/.claude/sessions/<pid>.json）的唯一读法与判活
import lib_session_directory as session_directories  # noqa: E402  会话记录目录的唯一取法（来源仓根 TASK_SERVER_SOURCE_REPOSITORY，不经 $HOME；agent-cost.py、agent-handover.py、watch.sh、move-directory-in-use-check.py 同取它）


ENTRY_SCRIPT_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "agent-watch.py")   # 自证起子进程要跑的入口；拆出来的模块里 __file__ 不是入口


def projects_root(environment=None):
    """各项目的会话目录所在的那一层（<家目录>/.claude/projects）：统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY（两台一致）定家目录，没有这个变量（服务外手跑）取本机家目录，
    取法只在 lib_session_directory.py 里（不经 $HOME 拼）；AGENT_WATCH_PROJECTS_ROOT 设了优先，只给 --selftest 造假的项目目录用。取不到抛 session_directories.SessionDirectoryError。"""
    environment = os.environ if environment is None else environment
    return environment.get("AGENT_WATCH_PROJECTS_ROOT") or session_directories.projects_root(environment)

BROKEN_DETECTION = os.environ.get("AGENT_WATCH_BREAK", "")

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
TURN_END_STOP_REASONS = ("end_turn", "stop_sequence")   # 会话记录里结束本轮的两种写法
TEXT_SETTLE_SECONDS = 120            # 最后一条是纯文字、stop_reason 为空而主会话记录不在时：过了这么久没接下文就算结束本轮（AgentTranscript.text_turn_ended）

BACKGROUND_STARTED = re.compile(r"^Command running in background with ID: (\w+)")
# 前台命令跑满超时被 harness 挪进后台，工具结果是另一种写法，之后同样会来完成通知。不认它，在等这种任务的子 agent 会被报「结束本轮却不会醒」
BACKGROUND_MOVED = re.compile(r"^Command did not complete within its \d+s timeout and was moved to the background \(ID: (\w+)\)")
NEW_INSTRUCTION_ORIGINS = ("coordinator", "peer")   # 续做消息与别的会话发来的消息；task-notification 是自己后台任务的完成通知，不算
BACKGROUND_OUTPUT_PATH = re.compile(r"Output is being written to: (\S+?\.output)")
TASK_NOTIFICATION_ID = re.compile(r"<task-id>(\w+)</task-id>")

EXITED_PROCESS_STATES = ("Z", "X")   # /proc/<pid>/stat 的状态：僵尸（退出了、等父进程收尸）与已死，都算退出了
LEFTOVER_BACKGROUND_ALERT = "交回之后后台还在跑"
KEPT_COPY_ALERT = "交回之后副本还在"
# 交回报告里「没删 <全路径>」那一行的写法（上游交回闸 handback-scratch-check.sh 要的就是这一行）：路径取到第一个不是路径字符的字为止（后面紧跟汉字也认）
KEPT_COPY_MENTION = re.compile(r"没删(?:掉)?[：:\s]*[`「“\"']?((?:~/|/)[A-Za-z0-9_./~+@%=,-]+)")
# 交回报告里点名的报告文件（.md，绝对路径或 ~/ 起头）：里面写的「没删 <全路径>」也算
KEPT_COPY_REPORT_MENTION = re.compile(r"((?:~/|/)[A-Za-z0-9_./~+@%=,-]+\.md)")
KEPT_COPY_REPORT_MAX_BYTES = 4 << 20
# 主会话记录里任务通知的这几种状态说明子 agent 已经不在跑了：failed 失败或报错、killed 被停、stopped 上一个 Claude 进程退出时它还没跑完
ENDED_NOTIFICATION_STATUSES = ("failed", "killed", "stopped")
TASK_NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
TASK_NOTIFICATION_STATUS = re.compile(r"<status>([a-z_]+)</status>")
# 重启前旧进程留下的子 agent 记录判「已失效」：Claude Code 给每个进程在这个目录登记一份 <pid>.json（会话号 sessionId、启动时刻 startedAt 毫秒、
# procStart 即 /proc/<pid>/stat 第 22 列）。环境变量只给 --selftest 造假的登记目录用
SESSION_REGISTRY_ENV = "AGENT_WATCH_SESSIONS_DIR"
# 草稿根里超阈值的编译目录与仓副本：编译目录认 CACHEDIR.TAG（签名见 https://bford.info/cachedir/，cargo 的 target 都带），仓副本认 .git 或「Cargo.toml 加 crates/」。
# 环境变量（冒号分隔的根）只给 --selftest 造假的草稿根用
SCRATCH_ROOTS_ENV = "AGENT_WATCH_SCRATCH_ROOTS"
CACHEDIR_TAG_SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"
SCRATCH_SCAN_MAX_DEPTH = 4            # 从草稿根往下找这么深（根 → 任务目录 → 副本 → target）
SCRATCH_SCAN_MAX_DIRECTORIES = 20000  # 一次最多看这么多个目录，到顶就在报告里写明
SCRATCH_MTIME_MAX_DEPTH = 3           # 求「最后修改时刻」往副本里看这么深（target/release/deps 在内）
SCRATCH_MTIME_MAX_ENTRIES = 30000     # 每处副本最多 stat 这么多个条目
ROOT_FREE_ALERT = "根分区余量过低"
ROOT_FREE_SUBJECT = "本机"            # 这一类告警不属于哪个子 agent 或进程，主体写本机；推荐的确认写法是 --ack 类:根分区余量过低（watch.sh 不认「本机:」这个头）
CRASH_AMPLIFICATION_HOME_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOME"   # 崩溃放量家目录，设了用它（crash_amplification_home）
CRASH_AMPLIFICATION_ENTRY = "crash-amplification"   # 没设时工作根下那一层的目录名，与 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory 相同
ROOT_FREE_BYTES_VARIABLE = "AGENT_WATCH_ROOT_FREE_BYTES"   # 环境变量只给 --selftest 起的看门狗子进程用：设了就当根分区的可用字节数，不读 statvfs
ROOT_FREE_GIB_ALERT_DEFAULT = 128     # --root-free-gib-alert 的默认值（依据见那个选项，推的）

# 会话记录按字节扫后台输出文件路径：几百兆的会话记录不逐行解 JSON，先看它提没提到有进程开着的那几个文件
BACKGROUND_OUTPUT_PATH_BYTES = re.compile(rb"Output is being written to: (\S+?\.output)")
PROCESS_STOP_COMMAND = "python3 .claude/singlefs-ai-sop/scripts/proc.py stop"

# 主会话记录里 SendMessage 发给子 agent 的成功结果的两种写法（排队的，与续做的「Resuming agent」），
# 都带 pin，续做的另带 resumedAgentId；发给别的会话的写法不同，不认
MAIN_AGENT_MESSAGE_QUEUED = "Message queued for delivery to "
MAIN_AGENT_MESSAGE_RESUMED = "resumedAgentId"

SELFTEST_READY_TIMEOUT_SECONDS = 120   # 自证按条件等子进程起来的上限；到点不报错，照原判据接着判
# 不核「知识同步提醒」的那几格起的看门狗带上它：几只先后用同一个会话目录时，落盘的上次提醒会把定时回报提前（report_deadline_of），那几格按回报周期等检出与宽限的判法就不稳
SELFTEST_SECTION_OFF = ("--repository-report-lines", "0")
BYTES_PER_GIBIBYTE = 1024 ** 3


def parse_timestamp(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00"))


def is_new_instruction(record):
    """会话记录里这一条是不是给子 agent 的新指令：交回之后来一条就撤销交回。
    认：续做消息（user 记录，origin 为 coordinator；别的会话发来的为 peer）；它忙时排进队的续做消息（queued_command 附件，附件自己带
    origin coordinator，没有对应的 user 记录）；不带 origin、不是 harness 自己插的一句 user 文字。
    不认：后台任务的完成通知（origin 为 task-notification，或不带 origin 的 queued_command 附件）、工具结果、harness 插的提示（isMeta：
    system-reminder、「Output token limit hit」「[handback-send-enforce]」这类）、打断、压缩摘要（isCompactSummary）。"""
    attachment = record.get("attachment")
    if isinstance(attachment, dict):
        return attachment.get("type") == "queued_command" and (attachment.get("origin") or {}).get("kind") in NEW_INSTRUCTION_ORIGINS
    origin_kind = (record.get("origin") or {}).get("kind")
    if origin_kind is not None:
        return origin_kind in NEW_INSTRUCTION_ORIGINS
    message = record.get("message")
    if record.get("isMeta") or record.get("isCompactSummary") or not isinstance(message, dict) or message.get("role") != "user":
        return False
    content = message.get("content")
    if isinstance(content, str):
        texts = [content]
    elif isinstance(content, list) and not any(isinstance(block, dict) and block.get("type") == "tool_result" for block in content):
        texts = [block.get("text", "") for block in content if isinstance(block, dict) and block.get("type") == "text"]
    else:
        return False
    return any(text.strip() for text in texts) and not any("[Request interrupted" in text for text in texts)


def parent_session_transcript(transcript_path):
    """…/<会话 id>/subagents/agent-<id>.jsonl → …/<会话 id>.jsonl（派它的那个会话的记录）。"""
    return os.path.dirname(os.path.dirname(transcript_path)) + ".jsonl"


SESSION_TASK_EVENTS_CACHE = {}   # 主会话记录路径 → ((大小, 修改时刻), 读出来的两张表)


def is_task_notification_record(record):
    """真正的任务通知：origin 为 task-notification 的消息、排队的附件、队列操作；主 agent 自己的命令与回复里写着的 <status> 不算。"""
    attachment = record.get("attachment")
    return ((record.get("origin") or {}).get("kind") == "task-notification" or record.get("type") == "queue-operation"
            or (isinstance(attachment, dict) and attachment.get("type") == "queued_command"))


def session_task_events(session_transcript_path):
    """主会话记录一遍读完，返回三张表：{子 agent id: TaskStop 对它成功结果的最晚时刻}、{任务 id: (最近一次任务通知的状态, 时刻)}、
    {子 agent id: 主 agent 用 SendMessage 发给它的最晚一次成功结果的时刻}。
    按文件大小与修改时刻缓存：主会话记录能有上百兆，逐个子 agent 各读一遍太慢。"""
    try:
        status = os.stat(session_transcript_path)
    except OSError:
        return {}, {}, {}
    key = (status.st_size, status.st_mtime_ns)
    cached = SESSION_TASK_EVENTS_CACHE.get(session_transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1]
    stops, notifications, messages_sent = {}, {}, {}
    for line in open(session_transcript_path, encoding="utf-8", errors="replace"):
        mentions_stop = "Successfully stopped task" in line
        mentions_status = "<status>" in line
        mentions_message_sent = MAIN_AGENT_MESSAGE_QUEUED in line or MAIN_AGENT_MESSAGE_RESUMED in line
        if not (mentions_stop or mentions_status or mentions_message_sent):
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if not record.get("timestamp"):
            continue
        timestamp = parse_timestamp(record["timestamp"])
        result = record.get("toolUseResult")
        if mentions_stop and isinstance(result, dict) and "Successfully stopped task" in str(result.get("message", "")):
            agent_id = result.get("task_id")
            stops[agent_id] = timestamp if agent_id not in stops else max(stops[agent_id], timestamp)
        if mentions_status and is_task_notification_record(record):
            # 一条通知可以列好几个任务 id、共用一个状态（上一个 Claude 进程退出时没跑完的那一批）
            for block in TASK_NOTIFICATION_BLOCK.findall(json.dumps(record, ensure_ascii=False)):
                found_status = TASK_NOTIFICATION_STATUS.search(block)
                if found_status:
                    for task_id in TASK_NOTIFICATION_ID.findall(block):
                        notifications[task_id] = (found_status.group(1), timestamp)
        if (mentions_message_sent and isinstance(result, dict) and result.get("success") is True
                and (str(result.get("message", "")).startswith(MAIN_AGENT_MESSAGE_QUEUED) or result.get(MAIN_AGENT_MESSAGE_RESUMED))):
            agent_id = result.get(MAIN_AGENT_MESSAGE_RESUMED) or (result.get("pin") or {}).get("id")
            if agent_id:
                messages_sent[agent_id] = timestamp if agent_id not in messages_sent else max(messages_sent[agent_id], timestamp)
    SESSION_TASK_EVENTS_CACHE[session_transcript_path] = (key, (stops, notifications, messages_sent))
    return stops, notifications, messages_sent


def task_stop_time(agent_id, session_transcript_path):
    """主会话记录里 TaskStop 停掉这个子 agent 的最晚一次成功结果的时刻；没有就 None。"""
    return session_task_events(session_transcript_path)[0].get(agent_id)


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
        self.handed_back_timestamp = None    # 算交回的那一次交回工具成功返回的时刻
        self.handback_message = ""           # 算交回的那一次交回工具的正文（找「没删 <全路径>」用）
        self.interrupted_timestamp = None    # 最后一次被打断（[Request interrupted）的时刻
        self.task_stopped_timestamp = None   # 按主会话记录里的 TaskStop 判成被停时，那次停的时刻
        self.main_agent_message_delivered_timestamp = None   # 它会话记录里主 agent 的消息最晚一次送达的时刻（origin coordinator）
        self.state = "思考中"
        self.dispatch_prompt = ""            # 会话记录里第一条 user 文字：派发提示（找草稿目录里的 progress.md 用）
        self.last_text = ""                  # 它最后一句话（「中途停下没交回」告警与报告那一节用）
        self._read()
        parent_path = parent_session_transcript(transcript_path)
        self.parent_transcript_missing = not os.path.isfile(parent_path)
        self.parent_transcript_path = parent_path
        # 主会话记录里 SendMessage 发给它的最晚一次成功结果：忙着的子 agent 要等下一轮才把消息落进自己的会话记录
        self.main_agent_message_sent_timestamp = session_task_events(parent_path)[2].get(agent_id)
        stopped_at = task_stop_time(agent_id, parent_path)
        if (stopped_at is not None and BROKEN_DETECTION != "taskstop" and self.state != "已交回"
                and (self.last_timestamp is None or (self.last_timestamp - stopped_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            self.state = "被停"
            self.task_stopped_timestamp = stopped_at

    def _read(self):
        seen_message_ids = set()
        pending_by_id = {}
        tool_inputs_by_id = {}
        handbacks = transcripts.HandbackTracker()   # 交回送没送达照 subagent_transcript.py 认（调用之后对应的 tool_result 不是错误）
        handback_message = ""
        is_handed_back = False  # 最近一次交回成功之后没有来过新指令（is_new_instruction）；交回之后它自己再调工具不撤销
        handback_timestamp = None
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
            # 主 agent 的消息送达：空闲时落成 origin 为 coordinator 的 user 记录，忙时落成附件自己带 origin coordinator 的 queued_command
            if (((record.get("origin") or {}).get("kind") == "coordinator"
                 or (isinstance(attachment, dict) and attachment.get("type") == "queued_command"
                     and (attachment.get("origin") or {}).get("kind") == "coordinator"))
                    and BROKEN_DETECTION != "inquirydelivered"):
                self.main_agent_message_delivered_timestamp = timestamp
            # 交回之后来的新指令撤销交回（认法见 is_new_instruction）：还没调工具也已经不算交回——续做的头几秒它在想，看门狗不能当场报「全部交回」退出。
            # 它忙时排进队的续做消息只落成 queued_command 附件、没有 user 记录，所以在这里判，不放进下面按 message 分的那几支。
            # 后台任务的完成通知不算，它叫醒之后又调工具也不算：交回之后调工具不撤销（见 tool_use 那一支的注释）。
            if is_new_instruction(record) and BROKEN_DETECTION != "resume":
                is_handed_back = False
            delivered = handbacks.feed(record)
            if delivered is not None:
                is_handed_back = True
                handback_timestamp = delivered["delivered_at"] or timestamp
                handback_message = delivered["message"]
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
                            # 交回之后它自己再调工具不撤销交回，撤销只认新指令：交回之后被自己后台任务的完成通知叫醒、看一眼输出再结束本轮的，
                            # 按调工具撤销会被判成没交回、报「结束本轮却不会醒」。
                            if block.get("name") != transcripts.HANDBACK_TOOL and BROKEN_DETECTION == "toolrevoke":
                                is_handed_back = False
                            last_kind = "tool_use"
                        elif block.get("type") == "text":
                            last_kind = "end_turn" if message.get("stop_reason") in TURN_END_STOP_REASONS else "text"
                            last_text_timestamp = timestamp
                            self.last_text = str(block.get("text") or "")
                            if last_kind == "end_turn":
                                self.last_turn_end_timestamp = timestamp
            elif role == "user":
                if not self.dispatch_prompt:
                    self.dispatch_prompt = content if isinstance(content, str) else " ".join(
                        block.get("text", "") for block in (content or []) if isinstance(block, dict) and block.get("type") == "text")
                if isinstance(content, list):
                    for block in content:
                        if block.get("type") == "tool_result":
                            pending_by_id.pop(block.get("tool_use_id"), None)
                            started = background_start_in_result(block.get("content"))
                            if started:
                                task_id, output_path = started
                                self.background_started.append(task_id)
                                if output_path:
                                    self.background_output_paths[task_id] = output_path
                            if block.get("tool_use_id") in tool_inputs_by_id:
                                self._count_tool_result(*tool_inputs_by_id[block.get("tool_use_id")], block.get("content"))
                            for entry in self.bash_commands[-20:]:
                                if entry[3] == block.get("tool_use_id"):
                                    result = block.get("content")
                                    entry[2] = result if isinstance(result, str) else json.dumps(result, ensure_ascii=False)
                            last_kind = "tool_result"
                        elif block.get("type") == "text" and "[Request interrupted" in block.get("text", ""):
                            last_kind = "interrupted"
                            self.interrupted_timestamp = timestamp
                elif isinstance(content, str) and "[Request interrupted" in content:
                    last_kind = "interrupted"
                    self.interrupted_timestamp = timestamp
                # 结束本轮之后来的续做消息：它被叫醒了，在想下一步，不再是「结束本轮」——不改的话续做头几秒它还没落下新记录，
                # 看门狗照上一轮的结束时刻报「结束本轮却不会醒」。
                if (record.get("origin") or {}).get("kind") == "coordinator" and BROKEN_DETECTION != "resumeturn":
                    last_kind = "resumed"
        # 一次工具调用只有在「它之后再没有别的记录」时才算还在跑。断网 / 被停时砍掉的那一次调用
        # 只落了发起、永远等不到结果，一直当成在跑会让「工具调用过长」每一轮都假报一次。
        if BROKEN_DETECTION != "stalepending" and self.last_timestamp is not None:
            pending_by_id = {
                key: item for key, item in pending_by_id.items() if item[0] >= self.last_timestamp
            }
        self.pending_tool = max(pending_by_id.values(), key=lambda item: item[0]) if pending_by_id else None
        # 最后一条是纯文字、stop_reason 为空：流式写入时同一条消息后面还会接工具调用，一次 Write 整份生成能隔几分钟才落下来；
        # 结束本轮了的认主会话记录里的任务通知（text_turn_ended）：写一句文字就等后台任务的，
        # 最后两条记录 stop_reason 都是空的，不认通知会判成「思考中」、过一阵报「无动静」。
        if (last_kind == "text" and last_text_timestamp is not None and BROKEN_DETECTION != "settle"
                and self.text_turn_ended(last_text_timestamp)):
            last_kind = "end_turn"
            self.last_turn_end_timestamp = last_text_timestamp
        if last_kind == "interrupted":
            self.state = "被停"
        elif is_handed_back and not pending_by_id and BROKEN_DETECTION != "finished":
            self.state = "已交回"
            self.handed_back_timestamp = handback_timestamp
            self.handback_message = handback_message
        elif self.pending_tool is not None:
            self.state = "执行工具中"
        elif last_kind == "end_turn" and BROKEN_DETECTION == "handback":
            self.state = "已交回"
        elif last_kind == "end_turn":
            self.state = "本轮结束、没交回"
        else:
            self.state = "思考中"

    def text_turn_ended(self, text_timestamp):
        """最后一条是纯文字、stop_reason 为空时，这一轮结束了没有：主会话记录里这个子 agent 的任务通知（completed，或 failed / killed / stopped）
        不早于这条文字才算。只按时长认，文字之后一次 Write 生成几分钟的会被当成结束本轮、报「中途停下没交回」（records/2026-10-23-subagent拆分提案.md
        第三十五节）。主会话记录不在时退回按时长认：过了 TEXT_SETTLE_SECONDS 没接下文算结束。弄坏开关 settlenotification 只按时长认。"""
        parent_path = parent_session_transcript(self.transcript_path)
        if BROKEN_DETECTION == "settlenotification" or not os.path.isfile(parent_path):
            return (datetime.now(timezone.utc) - text_timestamp).total_seconds() >= TEXT_SETTLE_SECONDS
        notified = session_task_events(parent_path)[1].get(self.agent_id)
        return notified is not None and notified[1] >= text_timestamp

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

    def last_main_agent_message_timestamp(self):
        """主 agent 最晚一次给它发消息的时刻：它会话记录里送达的与主会话记录里发出的，取晚的那个；都没有是 None。"""
        sent = None if BROKEN_DETECTION == "inquirysent" else self.main_agent_message_sent_timestamp
        moments = [moment for moment in (self.main_agent_message_delivered_timestamp, sent) if moment is not None]
        return max(moments) if moments else None

    def live_background_tasks(self):
        return [task_id for task_id in self.background_started if task_id not in self.background_finished]

    def orphaned_background_tasks(self):
        """起过、没收到完成通知，而 harness 写它输出的那个文件还在、却没有任何进程开着它：进程已经没了，通知不会再来。
        子 agent 失败中断时，失败之前起的探针任务就停在这个样子，不判的话看门狗一直把它当「还在跑」。
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
        """结束本轮、没交回，手里还有没收到完成通知的后台任务：它在等，会话记录里本来就一个字都不写，醒不醒由那些任务决定，
        不按会话记录的「无动静」判——等吃满 CPU 的探针时，看门狗每次重起都会当场报「无动静」退出，盯不住。"""
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

    def ended(self, notifications):
        """它还在不在跑：交回、被停或失败了返回 (怎么结束的, 那一刻)，还在跑返回 None。notifications 是 session_task_events 的第二张表。
        失败认它最近一次任务通知的状态在 ENDED_NOTIFICATION_STATUSES 里，且那条通知不早于它自己最后一条记录减
        STOP_AFTER_LAST_RECORD_SECONDS（失败之后又被续做、还在跑的不算，与 TaskStop 同一个判法）。"""
        if self.state == "已交回":
            return "已交回", self.handed_back_timestamp or self.last_timestamp
        if self.state == "被停":
            return "被停", self.task_stopped_timestamp or self.interrupted_timestamp or self.last_timestamp
        status, notified_at = notifications.get(self.agent_id, (None, None))
        if (status in ENDED_NOTIFICATION_STATUSES and BROKEN_DETECTION != "leftovernotified"
                and (self.last_timestamp is None or (self.last_timestamp - notified_at).total_seconds() <= STOP_AFTER_LAST_RECORD_SECONDS)):
            return f"任务通知 {status}", notified_at
        return None


def background_start_in_result(content):
    """一条工具结果里起了后台任务的（run_in_background 的「Command running in background with ID: …」，或前台跑满超时被挪进后台的写法）：
    交 (任务 id, 输出文件或 None)；不是交 None。AgentTranscript 与交回闸（own_background_tasks_still_running）共用这一个认法。"""
    text = content if isinstance(content, str) else ""
    started = BACKGROUND_STARTED.match(text) or (BACKGROUND_MOVED.match(text) if BROKEN_DETECTION != "movedbackground" else None)
    if not started:
        return None
    output_path = BACKGROUND_OUTPUT_PATH.search(text)
    return started.group(1), (output_path.group(1) if output_path else None)


def own_background_tasks_still_running(transcript_path):
    """交回闸（.claude/hooks/handback-guard.sh ④）用：这份子 agent 会话记录里它自己起过的后台任务，此刻还有进程开着输出文件的
    （与「交回之后后台还在跑」同一套认法：输出文件还有进程开着就是还在跑，不看完成通知）。
    交 [(任务 id, 输出文件, [(最上层的进程号, 已跑秒数, 命令, 从最底层往上的停止次序)])]，按任务 id 排；会话记录读不了抛 OSError。"""
    outputs = {}
    with open(transcript_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if "Output is being written to" not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            message = record.get("message") if isinstance(record, dict) else None
            content = message.get("content") if isinstance(message, dict) and message.get("role") == "user" else None
            for block in content if isinstance(content, list) else []:
                started = background_start_in_result(block.get("content")) if isinstance(block, dict) and block.get("type") == "tool_result" else None
                if started and started[1]:
                    outputs[started[0]] = started[1]
    if not outputs:
        return []
    holders_by_file = output_files_held_open()
    held = {task_id: holders_by_file[os.path.realpath(path)] for task_id, path in outputs.items() if os.path.realpath(path) in holders_by_file}
    if not held:
        return []
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)

    def bottom_up(pid):
        order = []
        for child in sorted(children.get(pid, [])):
            order += bottom_up(child)
        return order + ([pid] if table[pid]["state"] not in EXITED_PROCESS_STATES else [])

    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    found = []
    for task_id in sorted(held):
        holders = {pid for pid in held[task_id] if pid in table}
        tops = sorted(pid for pid in holders if table[pid]["parent"] not in holders)
        found.append((task_id, outputs[task_id], [(pid, uptime_seconds - table[pid]["start_ticks"] / ticks_per_second, command_for_display(table[pid]["command"]),
                                                   bottom_up(pid)) for pid in tops]))
    return [item for item in found if item[2]]


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


def output_files_held_open():
    """本机进程开着的后台任务输出文件（.output，按真实路径）→ 开着它的进程号集合。"""
    holders_by_file = {}
    if BROKEN_DETECTION == "leftover":
        return holders_by_file
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            descriptors = os.listdir(f"/proc/{entry}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                target = os.readlink(f"/proc/{entry}/fd/{descriptor}")
            except OSError:
                continue
            if target.endswith(".output"):
                holders_by_file.setdefault(os.path.realpath(target), set()).add(int(entry))
    return holders_by_file


TRANSCRIPT_OUTPUT_PATHS_CACHE = {}   # 子 agent 会话记录路径 → ((大小, 修改时刻), 它提到的后台输出文件的真实路径)


def output_paths_mentioned(transcript_path):
    """会话记录里提到过的后台输出文件（真实路径）：只用来挑出值得逐条解析的那几份，归属由 AgentTranscript 按工具结果认。"""
    try:
        status = os.stat(transcript_path)
    except OSError:
        return set()
    key = (status.st_size, status.st_mtime_ns)
    cached = TRANSCRIPT_OUTPUT_PATHS_CACHE.get(transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1]
    with open(transcript_path, "rb") as handle:
        mentioned = {os.path.realpath(match.decode(errors="replace")) for match in BACKGROUND_OUTPUT_PATH_BYTES.findall(handle.read())}
    TRANSCRIPT_OUTPUT_PATHS_CACHE[transcript_path] = (key, mentioned)
    return mentioned


def leftover_background_report(session_dirs, now, thresholds):
    """交回、被停或失败的子 agent 起过的后台任务，它结束过了宽限还有进程开着输出文件：返回 (报告行, 告警, 宽限最早到期的时刻或 None)。
    扫这些会话目录里全部子 agent，不只被看的那几个；只检出、只报，不停进程（检出与处置分开）。"""
    holders_by_file = output_files_held_open()
    grace_seconds = thresholds.leftover_background_grace_minutes * 60
    detail_lines, alerts, pending_until = [], [], None
    scanned, agents_within_grace, table = 0, 0, None
    for session_dir in sorted(session_dirs):
        transcript_paths = sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl")))
        scanned += len(transcript_paths)
        notifications = None
        for transcript_path in transcript_paths:
            if not holders_by_file or not (output_paths_mentioned(transcript_path) & holders_by_file.keys()):
                continue
            agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
            transcript = AgentTranscript(agent_id, transcript_path)
            if notifications is None:
                notifications = session_task_events(parent_session_transcript(transcript_path))[1]
            ended = transcript.ended(notifications)
            if ended is None:
                continue   # 还在跑的归别的告警管
            how_it_ended, ended_at = ended
            held_tasks = sorted(task_id for task_id, path in transcript.background_output_paths.items()
                                if os.path.realpath(path) in holders_by_file)
            if not held_tasks:
                continue
            since_ended = (now - ended_at).total_seconds()
            if since_ended < grace_seconds and BROKEN_DETECTION != "leftovergrace":
                grace_ends_at = datetime.fromtimestamp(ended_at.timestamp() + grace_seconds, timezone.utc)
                pending_until = grace_ends_at if pending_until is None else min(pending_until, grace_ends_at)
                agents_within_grace += 1
                detail_lines.append(f"  子 agent {agent_id} {how_it_ended}，距今 {format_duration(since_ended)}，它起的后台任务 {', '.join(held_tasks)} "
                                    f"的输出文件还有进程开着：还在 {format_duration(grace_seconds)} 宽限里，过了再查")
                continue
            holder_pids = set().union(*(holders_by_file[os.path.realpath(transcript.background_output_paths[task_id])] for task_id in held_tasks))
            table = table if table is not None else process_table()
            topmost = sorted(pid for pid in holder_pids if pid in table and table[pid]["parent"] not in holder_pids)
            detail_lines.append(f"  子 agent {agent_id} {transcript.agent_type}「{transcript.description}」{how_it_ended}，距今 {format_duration(since_ended)}，"
                                f"它起的后台任务 {', '.join(held_tasks)} 的输出文件还有 {len(holder_pids)} 个进程开着，最上层的 {len(topmost)} 个各报一条")
            ticks_per_second = os.sysconf("SC_CLK_TCK")
            uptime_seconds = float(open("/proc/uptime").read().split()[0])
            for pid in topmost:
                running_seconds = uptime_seconds - table[pid]["start_ticks"] / ticks_per_second
                alerts.append((agent_id, LEFTOVER_BACKGROUND_ALERT,
                               f"进程 {pid} 已跑 {format_duration(running_seconds)}，{command_for_display(table[pid]['command'])[:140]}",
                               f"看它在等什么；确认没用就按进程号停：`{PROCESS_STOP_COMMAND} {pid}`；要留着它接着跑就 --ack 进程:{pid}"))
    summary = (f"交回、被停或失败之后后台还在跑：查了 {scanned} 个子 agent 的会话记录，本机有进程开着的后台输出文件 {len(holders_by_file)} 个，"
               f"报 {len(alerts)} 条，宽限里的 {agents_within_grace} 个子 agent")
    return [summary, *detail_lines], alerts, pending_until


KEPT_COPY_TRANSCRIPT_CACHE = {}   # 会话记录路径 → ((大小, 修改时刻), 交回的时刻或 None, 交回正文)


def handback_of(agent_id, transcript_path):
    """(交回的时刻或 None, 交回正文)：会话记录没变就用上一次读的（交回过的会话记录很少再变）。"""
    try:
        status = os.stat(transcript_path)
    except OSError:
        return None, ""
    key = (status.st_size, status.st_mtime_ns)
    cached = KEPT_COPY_TRANSCRIPT_CACHE.get(transcript_path)
    if cached is not None and cached[0] == key:
        return cached[1], cached[2]
    transcript = AgentTranscript(agent_id, transcript_path)
    handed_back = transcript.handed_back_timestamp if transcript.state == "已交回" else None
    KEPT_COPY_TRANSCRIPT_CACHE[transcript_path] = (key, handed_back, transcript.handback_message)
    return handed_back, transcript.handback_message


def kept_copy_roots():
    """「没删 <路径>」里只认落在这几处底下的路径（工作根、/tmp（会话临时目录根在内）与本进程的临时目录）：仓副本与编译目录只建在这里，别处的不报、不叫人删。"""
    roots = [*work_roots.scratch_roots(), "/tmp", tempfile.gettempdir()]
    return tuple(dict.fromkeys(os.path.normpath(root) for root in roots))


def kept_copy_paths(texts):
    """交回正文与它点名的报告文件里「没删 <全路径>」点的路径（展开 ~、去掉末尾的 / 与句读），去重、保序。"""
    found = []
    for text in texts:
        for match in KEPT_COPY_MENTION.finditer(text):
            path = os.path.normpath(os.path.expanduser(match.group(1).rstrip("/.,")))
            if path not in found:
                found.append(path)
    return found


def mentioned_report_texts(message):
    """交回正文点名的报告文件（.md，在 kept_copy_roots 底下、不超过 KEPT_COPY_REPORT_MAX_BYTES）的内容；读不了的跳过。"""
    texts = []
    for match in KEPT_COPY_REPORT_MENTION.finditer(message):
        path = os.path.normpath(os.path.expanduser(match.group(1)))
        if not any(path.startswith(root + os.sep) for root in kept_copy_roots()):
            continue
        try:
            if os.path.getsize(path) <= KEPT_COPY_REPORT_MAX_BYTES:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    texts.append(handle.read())
        except OSError:
            continue
    return texts


def kept_copy_report(session_dirs, now, thresholds):
    """交回超过 --kept-copy-hours、交回正文（或它点名的报告文件）里写了「没删 <全路径>」而那个路径还在的：返回 (报告行, 告警)。
    扫这些会话目录里全部子 agent；只报、不删（检出与处置分开），删由主 agent 核过没人在用之后做。
    弄坏开关：keptcopy 一条都不报；keptcopyage 不看交回了多久；keptcopyexists 不看路径还在不在。"""
    hours = getattr(thresholds, "kept_copy_hours", 0) or 0
    if hours <= 0:
        return ["交回之后副本还在：--kept-copy-hours 写的是 0，这一类没查"], []
    if BROKEN_DETECTION == "keptcopy":
        return ["交回之后副本还在：弄坏开关 AGENT_WATCH_BREAK=keptcopy，一条都不报"], []
    threshold_seconds = hours * 3600
    roots = kept_copy_roots()
    scanned = mentioning = within = gone = 0
    detail_lines, alerts = [], []
    for session_dir in sorted(session_dirs):
        for transcript_path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
            scanned += 1
            agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
            handed_back, message = handback_of(agent_id, transcript_path)
            if handed_back is None or not message:
                continue
            paths = [path for path in kept_copy_paths([message, *mentioned_report_texts(message)])
                     if any(path.startswith(root + os.sep) for root in roots)]
            if not paths:
                continue
            mentioning += 1
            since = (now - handed_back).total_seconds()
            if since < threshold_seconds and BROKEN_DETECTION != "keptcopyage":
                within += 1
                continue
            still_there = [path for path in paths if os.path.lexists(path) or BROKEN_DETECTION == "keptcopyexists"]
            gone += len(paths) - len(still_there)
            for path in still_there:
                alerts.append((agent_id, KEPT_COPY_ALERT,
                               f"副本 {path} 还在：子 agent 交回 {format_duration(since)} 前在交回报告里写了没删它",
                               f"核过没有进程在用（工作目录、命令行、CARGO_TARGET_DIR 指到它）就删：rm -rf -- '{path}'；"
                               f"还要留着就 --ack {agent_id}:{KEPT_COPY_ALERT}"))
            if still_there:
                detail_lines.append(f"  子 agent {agent_id} 交回 {format_duration(since)} 前写了没删的 {len(paths)} 处，还在 {len(still_there)} 处")
    summary = (f"交回之后副本还在：查了 {scanned} 个子 agent 的会话记录，交回报告里写了「没删 <路径>」的 {mentioning} 个"
               f"（交回不到 {hours:g} 小时、先不报的 {within} 个；已经删掉的 {gone} 处），报 {len(alerts)} 条")
    return [summary, *detail_lines], alerts


process_start_ticks = session_registry.process_start_ticks   # /proc/<pid>/stat 第 22 列；读法只在 lib_session_registry.py 里


def live_sessions():
    """{会话号: 此刻活着的 Claude 进程里最早的启动时刻（秒；登记里没写是 None）}。
    活着 = 登记里的 pid 还在、而且 /proc/<pid>/stat 第 22 列与登记的 procStart 一致（pid 被别的进程占了不算）。
    登记目录读不了、或里面一份能解的登记都没有，返回 None：判不了，不把所有会话当成已失效。"""
    directory = os.environ.get(SESSION_REGISTRY_ENV) or session_registry.registry_directory()
    entries = session_registry.session_registry_entries(directory)
    if entries is None:
        return None
    live = {}
    for entry in entries:
        if not entry.is_alive:
            continue
        if entry.session_id not in live:
            live[entry.session_id] = entry.started_seconds
        elif entry.started_seconds is None or live[entry.session_id] is None:
            live[entry.session_id] = None
        else:
            live[entry.session_id] = min(live[entry.session_id], entry.started_seconds)
    return live


def stale_session_reason(transcript_path, last_timestamp, session_dir, live):
    """重启前旧进程留下的子 agent 记录判已失效，返回原因；不是返回 None。两种：
    一、它所在的会话号不是本会话（--session-dir 的目录名，没给取 CLAUDE_CODE_SESSION_ID）、登记里也没有活着的 Claude 进程；
    二、会话号还活着（重启后接着用同一个会话号），而它最后一条记录早于那个会话现在活着的最早一个进程的启动时刻——启动之前就停了的，不是现在这个进程派的。
    判不了（登记读不出、登记里没写启动时刻、没有最后一条记录）的不判失效。
    弄坏开关：stalesession 两种都不判；stalesessionrestart 不判第二种。"""
    if live is None or BROKEN_DETECTION == "stalesession":
        return None
    session_id = os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))
    current = os.path.basename(os.path.normpath(session_dir)) if session_dir else os.environ.get("CLAUDE_CODE_SESSION_ID")
    if session_id not in live:
        if session_id == current:
            return None   # 本会话自己的进程没登记：判不了，不把本会话的子 agent 全当成失效
        return f"属于已不在的会话 {session_id}（Claude 进程登记里没有活着的），是重启前旧进程留下的记录"
    started = live[session_id]
    if (started is not None and last_timestamp is not None and BROKEN_DETECTION != "stalesessionrestart"
            and last_timestamp.timestamp() < started):
        return f"最后一条记录在会话 {session_id} 现在这个 Claude 进程启动之前，是重启前旧进程留下的记录"
    return None


def scratch_hog_roots(session_ids):
    """要查的草稿根：工作根整个；会话临时目录根 /tmp/claude-<uid> 只看本会话的那一份（<会话临时目录根>/<项目>/<会话号>，Claude Code 的 scratchpad 在里面）。
    环境变量 AGENT_WATCH_SCRATCH_ROOTS 给了就只用它（自证用）。弄坏开关 scratchhogsessionroot 下只看工作根。"""
    override = os.environ.get(SCRATCH_ROOTS_ENV)
    if override is not None:
        return [root for root in override.split(os.pathsep) if root]
    roots = [work_roots.work_root()]
    if BROKEN_DETECTION != "scratchhogsessionroot":
        for session_id in sorted(session_ids):
            roots.extend(sorted(glob.glob(os.path.join(work_roots.claude_session_temporary_root(), "*", session_id))))
    return roots


def scratch_hog_kind(path, names):
    """这个目录是什么：带签名对得上的 CACHEDIR.TAG 的是「编译目录」；有 .git（目录或工作树的文件）、或同时有 Cargo.toml 与 crates 的是「仓副本」；都不是 None。
    弄坏开关 scratchhogtag：不认 CACHEDIR.TAG（编译目录一概认不出）。"""
    if "CACHEDIR.TAG" in names and BROKEN_DETECTION != "scratchhogtag":
        try:
            with open(os.path.join(path, "CACHEDIR.TAG"), "rb") as handle:
                if handle.read(len(CACHEDIR_TAG_SIGNATURE)) == CACHEDIR_TAG_SIGNATURE:
                    return "编译目录"
        except OSError:
            pass
    if ".git" in names or ("Cargo.toml" in names and "crates" in names):
        return "仓副本"
    return None


def find_scratch_hogs(roots):
    """从草稿根往下找编译目录与仓副本（认出的不再往里找）：返回 ([(路径, 种类)], 看了几个目录, 是否到顶没看完)。符号链接不跟。"""
    found, visited = [], 0
    stack = [(root, 0) for root in reversed(roots) if os.path.isdir(root)]
    while stack:
        path, depth = stack.pop()
        if visited >= SCRATCH_SCAN_MAX_DIRECTORIES:
            return found, visited, True
        visited += 1
        try:
            entries = list(os.scandir(path))
        except OSError:
            continue
        kind = scratch_hog_kind(path, {entry.name for entry in entries}) if depth > 0 else None
        if kind:
            found.append((path, kind))
            continue
        if depth < SCRATCH_SCAN_MAX_DEPTH:
            stack.extend((entry.path, depth + 1) for entry in sorted(entries, key=lambda item: item.name, reverse=True)
                         if entry.is_dir(follow_symlinks=False))
    return found, visited, False


def newest_modification(path):
    """一处副本里最近一次修改的时刻（秒）：它自己与往里 SCRATCH_MTIME_MAX_DEPTH 层的条目（目录与文件）里 st_mtime 最大的。
    只看这么深、这么多：几十万文件的编译目录不全树走一遍；cargo 重编换掉 target/release/deps 里的文件，那一层目录的修改时刻会动。"""
    newest = os.lstat(path).st_mtime
    stack, seen = [(path, 0)], 0
    while stack and seen < SCRATCH_MTIME_MAX_ENTRIES:
        directory, depth = stack.pop()
        try:
            entries = list(os.scandir(directory))
        except OSError:
            continue
        for entry in entries:
            seen += 1
            try:
                newest = max(newest, entry.stat(follow_symlinks=False).st_mtime)
            except OSError:
                continue
            if depth + 1 < SCRATCH_MTIME_MAX_DEPTH and entry.is_dir(follow_symlinks=False):
                stack.append((entry.path, depth + 1))
    return newest


def scratch_hog_report(roots, now, hours):
    """草稿根里最后修改距今超过 hours 小时的编译目录与仓副本，最久没动的排前面：返回报告行。只报不删（检出与处置分开），删由主 agent 核过没人在用之后做。
    弄坏开关：scratchhog 一条都不报；scratchhogage 不看最后修改的时刻，不到阈值的也报；scratchhogtag 见 scratch_hog_kind。"""
    if hours <= 0:
        return ["草稿根里超阈值的编译目录与仓副本：--scratch-idle-hours 写的是 0，这一类没查"]
    if BROKEN_DETECTION == "scratchhog":
        return ["草稿根里超阈值的编译目录与仓副本：弄坏开关 AGENT_WATCH_BREAK=scratchhog，一条都不报"]
    found, visited, truncated = find_scratch_hogs(roots)
    idle, recent = [], 0
    for path, kind in found:
        try:
            newest = newest_modification(path)
        except OSError:
            continue
        age = now.timestamp() - newest
        if age >= hours * 3600 or BROKEN_DETECTION == "scratchhogage":
            idle.append((age, newest, path, kind))
        else:
            recent += 1
    idle.sort(key=lambda item: -item[0])
    summary = (f"草稿根里超阈值的编译目录与仓副本：查了 {len(roots)} 个根、{visited} 个目录{'（到顶没看完）' if truncated else ''}，找到编译目录 "
               f"{sum(kind == '编译目录' for _, kind in found)} 处、仓副本 {sum(kind == '仓副本' for _, kind in found)} 处；"
               f"最后修改不到 {hours:g} 小时的 {recent} 处不报，报 {len(idle)} 处（按最后修改时刻、最久没动的在前）；只报不删")
    lines = [summary]
    for age, newest, path, kind in idle:
        modified_on = datetime.fromtimestamp(newest, ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")
        lines.append(f"  {kind} {path}：最后修改 {format_duration(age)} 前（{modified_on}）")
    if idle:
        lines.append("   → 怎么办：主 agent 核过没有进程在用（工作目录、命令行、CARGO_TARGET_DIR 指到它）、也不是哪个子 agent 还要复跑的，再逐个 rm -rf -- '<路径>'；"
                     "一批清完提醒用户跑 sudo fstrim -v /")
    return lines


def root_free_path():
    """读可用量的那个路径：工作根；还没建的往上取第一个在的上层（statvfs 要路径在）。"""
    path = work_roots.work_root()
    while not os.path.exists(path) and os.path.dirname(path) != path:
        path = os.path.dirname(path)
    return path


def root_free_bytes(path):
    """path 所在文件系统普通用户还能写的字节数（f_bavail × f_frsize，与 df 的 Avail 同一个数）；环境变量 AGENT_WATCH_ROOT_FREE_BYTES 设了照它（自证用）。"""
    override = os.environ.get(ROOT_FREE_BYTES_VARIABLE)
    if override is not None:
        return int(override)
    status = os.statvfs(path)
    return status.f_bavail * status.f_frsize


def crash_amplification_home(environment=None):
    """崩溃放量家目录（文件头「根分区余量过低」）：SINGLEFS_CRASH_AMPLIFICATION_HOME 设了用它，没设是工作根下的 crash-amplification（共享根里、两台同一份）。
    environment 不给取 os.environ。弄坏开关 amplificationhome：没设时取家目录下那一份。"""
    environment = os.environ if environment is None else environment
    configured = environment.get(CRASH_AMPLIFICATION_HOME_VARIABLE, "")
    if configured:
        return configured
    if BROKEN_DETECTION == "amplificationhome":
        return os.path.join(work_roots.home_directory(), ".local", "share", "singlefs", CRASH_AMPLIFICATION_ENTRY)
    return os.path.join(work_roots.work_root(), CRASH_AMPLIFICATION_ENTRY)


def root_free_report(thresholds, free_bytes_of=root_free_bytes):
    """根分区（工作根所在的文件系统）的可用量：返回 (报告行, [(告警名, 说明, 下一步)])，低于 --root-free-gib-alert 报「根分区余量过低」。
    free_bytes_of 是读可用字节数的函数（收路径），自证传假的。弄坏开关 rootfree：不看余量、一条都不报。"""
    threshold = getattr(thresholds, "root_free_gib_alert", ROOT_FREE_GIB_ALERT_DEFAULT)
    if threshold <= 0:
        return ["根分区余量：--root-free-gib-alert 写的是 0，这一类没查"], []
    if BROKEN_DETECTION == "rootfree":
        return ["根分区余量：弄坏开关 AGENT_WATCH_BREAK=rootfree，不看余量"], []
    path = root_free_path()
    try:
        free = free_bytes_of(path) / BYTES_PER_GIBIBYTE
    except (OSError, ValueError) as error:
        return [f"根分区余量：读不出 {path} 所在文件系统的可用量（{error}），这一类没查"], []
    line = f"根分区余量：工作根 {path} 所在的文件系统可用 {free:.1f} GiB（阈值 {threshold:g} GiB，--root-free-gib-alert）"
    if free >= threshold:
        return [line], []
    amplification_runs = os.path.join(crash_amplification_home(), "runs")
    explanation = f"工作根 {path} 所在的文件系统只剩 {free:.1f} GiB 可用，低于阈值 {threshold:g} GiB（--root-free-gib-alert），再写下去会撞 No space left on device"
    next_step = (f"先查各会话的副本（du -sh {work_roots.work_root()}/*，大的先看），再查 {amplification_runs}/ 下的各运行目录（du -sh {amplification_runs}/*）；以 du 为准，"
                 "只删没有进程在用的（ps 里没有进程的工作目录、命令行或 CARGO_TARGET_DIR 指到它），别人在用的不删；删完再看一次 df，提醒用户跑 sudo fstrim -v /；"
                 f"一时删不了、确认接着跑的用 --ack {ACK_CLASS_PREFIX}{ROOT_FREE_ALERT}")
    return [line], [(ROOT_FREE_ALERT, explanation, next_step)]


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


def find_transcript(agent_id, session_dir=None, environment=None):
    if session_dir:
        candidate = os.path.join(session_dir, "subagents", f"agent-{agent_id}.jsonl")
        return candidate if os.path.isfile(candidate) else None
    matches = glob.glob(os.path.join(glob.escape(projects_root(environment)), "*", "*", "subagents", f"agent-{agent_id}.jsonl"))
    return matches[0] if matches else None


def process_table():
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as handle:
                stat_text = handle.read()
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                command_line = handle.read().replace(b"\0", b" ").decode(errors="replace").strip()
        except OSError:
            continue
        after_name = stat_text[stat_text.rfind(")") + 2:].split()
        table[int(entry)] = {"parent": int(after_name[1]), "start_ticks": int(after_name[19]), "state": after_name[0],
                             "cpu_ticks": int(after_name[11]) + int(after_name[12]),
                             "cpu_ticks_with_reaped": sum(int(field) for field in after_name[11:15]),   # 连同已回收子进程的用户态、内核态
                             "command": command_line}
    return table


def claude_instance_pid(table, start_pid):
    pid = start_pid
    while pid in table and pid > 1:
        if "native-binary/claude" in table[pid]["command"] or os.path.basename(table[pid]["command"].split(" ")[0]) == "claude":
            return pid
        pid = table[pid]["parent"]
    return None


def process_arguments(pid):
    """/proc/<pid>/cmdline 按 NUL 切开的那一串；读不到或是空的（内核线程、僵尸）交 None。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            raw = handle.read()
    except OSError:
        return None
    if not raw:
        return None
    return [part.decode("utf-8", "replace") for part in raw.rstrip(b"\0").split(b"\0")]


BASH_TOOL_WRAPPER_COMMAND = re.compile(r"&& eval '(.*)'(?: < /dev/null)? && pwd -P >\| \S+\s*$", re.S)


def command_for_display(command):
    """Claude 的 Bash 工具把命令包在「source 快照 && … && eval '<命令>' [< /dev/null] && pwd -P >| …」里：报告里只列 eval 里那一段
    （还原 eval 里转义过的单引号、换行并成空格），不然最上层每一行都是同一串包装。"""
    if BROKEN_DETECTION == "display":
        return command
    wrapped = BASH_TOOL_WRAPPER_COMMAND.search(command)
    shown = wrapped.group(1).replace("'\"'\"'", "'") if wrapped else command
    return " ".join(shown.split())

# --ack 的范围写法（文件头「--ack 的写法」）：类:<告警名> 确认这一类告警的全部（不分子 agent、不分进程）。
# 弄坏开关 ackscope 不认它（只能按进程号、按 agent 号逐个确认）；ackscopepersist 下记进状态目录的这种确认在下一次整查被当成条件已消失而作废
ACK_CLASS_PREFIX = "类:"


def write_transcript(directory, agent_id, records, meta=None):
    os.makedirs(os.path.join(directory, "subagents"), exist_ok=True)
    path = os.path.join(directory, "subagents", f"agent-{agent_id}.jsonl")
    with open(path, "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    if meta:
        json.dump(meta, open(path[: -len(".jsonl")] + ".meta.json", "w", encoding="utf-8"), ensure_ascii=False)


def write_session_stops(session_directory, minutes_ago_by_agent):
    """主会话记录里 TaskStop 的调用与成功结果，结果的形态照主会话记录里的原样。"""
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
    """交回工具的调用与成功返回，返回的形态照会话记录里的原样。"""
    return [record_at(minutes_ago, "assistant", [{"type": "tool_use", "id": tool_id, "name": "SubagentHandback", "input": {"message": "报告"}}],
                      stop_reason="tool_use", message_id=message_id, usage={"input_tokens": 10, "cache_read_input_tokens": 1000}),
            record_at(minutes_ago - 0.1, "user", [{"type": "tool_result", "tool_use_id": tool_id,
                                                   "content": [{"type": "text", "text": '{"success":true,"message":"Report delivered to your caller."}'}]}])]


def wait_until(condition, timeout_seconds=SELFTEST_READY_TIMEOUT_SECONDS):
    """自证按条件等（子进程起到要测的样子），不按固定秒数等：条件成立回 True，到点回 False，调用方照原判据接着判。"""
    deadline = time.time() + timeout_seconds
    while not condition():
        if time.time() >= deadline:
            return False
        time.sleep(0.05)
    return True


def wait_for_selftest_ready(ready_path, watcher, timeout_seconds=SELFTEST_READY_TIMEOUT_SECONDS):
    """等自证拉起的看门狗建就绪文件（mark_selftest_ready）：建了回 True；看门狗先退了或到点回 False，调用方照原判据接着判。"""
    return wait_until(lambda: os.path.exists(ready_path) or watcher.poll() is not None, timeout_seconds) and os.path.exists(ready_path)


def selftest_scratch_hogs(work, thresholds, failures, watch_runs):
    """「草稿根里超阈值的编译目录与仓副本」的自检：造一个草稿根，里面一个超阈值的编译目录（带 CACHEDIR.TAG）与一个不超的、
    一个超阈值的仓副本与一个不超的、一个超阈值却没有 CACHEDIR.TAG 签名的目录与一个超阈值的普通目录；交回时只报超阈值的编译目录与仓副本。返回查了几格。
    弄坏开关：scratchhog（报告里一条都没有）、scratchhogage（不超的也报）、scratchhogtag（编译目录认不出）。"""
    checks = 0
    root = os.path.join(work, "scratch-hog-root")
    tag = CACHEDIR_TAG_SIGNATURE + b"\n# sample\n"
    layout = {   # 路径 → (文件 → 内容)，空内容的键以 / 结尾是目录
        "old-build/target": {"CACHEDIR.TAG": tag, "release/deps/lib.rlib": b"x"},
        "fresh-build/target": {"CACHEDIR.TAG": tag, "release/deps/lib.rlib": b"x"},
        "old-copy": {".git/HEAD": b"ref: refs/heads/x\n", "Cargo.toml": b"[workspace]\n"},
        "fresh-copy": {".git/HEAD": b"ref: refs/heads/x\n", "Cargo.toml": b"[workspace]\n"},
        "old-minimal-copy": {"Cargo.toml": b"[workspace]\n", "crates/a/lib.rs": b"//\n"},
        "old-badtag/target": {"CACHEDIR.TAG": b"Signature: 00000000000000000000000000000000\n", "x": b"x"},
        "old-plain": {"notes.txt": b"x"},
    }
    for relative, files in layout.items():
        for name, content in files.items():
            path = os.path.join(root, relative, name)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "wb") as handle:
                handle.write(content)
    hours_idle = {"old-build": 30, "old-copy": 10, "old-minimal-copy": 20, "old-badtag": 10, "old-plain": 10}   # 排序要看得出：build 最久、minimal 其次、copy 最近
    for top, hours in hours_idle.items():
        moment = time.time() - hours * 3600
        for directory, directories, files in os.walk(os.path.join(root, top), topdown=False):
            for name in [*files, *directories]:
                os.utime(os.path.join(directory, name), (moment, moment), follow_symlinks=False)
            os.utime(directory, (moment, moment))
    wanted = {"编译目录 " + os.path.join(root, "old-build/target"), "仓副本 " + os.path.join(root, "old-copy"),
              "仓副本 " + os.path.join(root, "old-minimal-copy")}
    unwanted = [os.path.join(root, name) for name in ("fresh-build", "fresh-copy", "old-badtag", "old-plain")]
    lines = scratch_hog_report([root], datetime.now(timezone.utc), 6)
    detail_lines = [line for line in lines if line.startswith(("  编译目录 ", "  仓副本 "))]
    reported = {" ".join(re.match(r"  (编译目录|仓副本) (\S+)：", line).groups()) for line in detail_lines}
    checks += 1
    if reported != wanted:
        failures.append(f"草稿根里超阈值的编译目录与仓副本：应当只报 {sorted(wanted)}（超过 6 小时没动的编译目录、仓副本），实际 {sorted(reported)}")
    checks += 1
    if not (lines and lines[0].startswith("草稿根里超阈值的编译目录与仓副本：查了 1 个根") and "只报不删" in lines[0]):
        failures.append(f"草稿根那一节的第一行应当是汇总（查了几个根、报几处、只报不删），实际 {lines[:1]}")
    checks += 1
    order = [re.match(r"  (?:编译目录|仓副本) (\S+)：", line).group(1) for line in detail_lines]
    if order != [os.path.join(root, "old-build/target"), os.path.join(root, "old-minimal-copy"), os.path.join(root, "old-copy")]:
        failures.append(f"草稿根那一节应当按最后修改时刻、最久没动的在前（old-build 30 小时、old-minimal-copy 20 小时、old-copy 10 小时），实际次序 {order}")
    checks += 1
    if scratch_hog_report([root], datetime.now(timezone.utc), 0)[0].count("这一类没查") != 1:
        failures.append("--scratch-idle-hours 写 0 时应当写明这一类没查")
    # 端到端：有子 agent 交回时看门狗的报告里带这一节、只含超阈值的；没有子 agent 交回的报告里不带（交回时才报）
    session = os.path.join(work, "scratch-hog-session")
    write_transcript(session, "hoghanded", [bash_use(20, "t1", "ls", "m1"), bash_result(19, "t1"), *handback_records(18, "h1", "m2")],
                     {"agentType": "implementation-writer", "description": "交回了"})
    write_transcript(session, "hogrunning", [bash_use(1, "t1", "cargo build", "m1"), bash_result(0.5, "t1")],
                     {"agentType": "implementation-writer", "description": "还在跑"})
    environment = dict(os.environ, **{SCRATCH_ROOTS_ENV: root})

    def report_for(agent_id):
        watch_runs.append(["report", "--agents", agent_id])
        return subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--agents", agent_id, "--session-dir", session,
                               "--process-root-pid", "0"], capture_output=True, text=True, timeout=120, env=environment)
    handed = report_for("hoghanded")
    checks += 1
    if (handed.returncode != 0 or not all(path.split(" ", 1)[1] in handed.stdout for path in wanted)
            or any(path in handed.stdout for path in unwanted)):
        failures.append(f"有子 agent 交回时看门狗的报告应当列出超阈值的 {sorted(wanted)}、不列 {unwanted}，实际退出码 {handed.returncode}：{handed.stdout[-900:]}")
    running = report_for("hogrunning")
    checks += 1
    if "草稿根里超阈值的编译目录与仓副本" in running.stdout:
        failures.append(f"没有子 agent 交回时看门狗的报告里不该带草稿根那一节，实际：{running.stdout[-600:]}")
    # 会话临时目录根下本会话的那一份照查（Claude Code 的 scratchpad 在里面）：不给 AGENT_WATCH_SCRATCH_ROOTS 时 scratch_hog_roots 交工作根与它；
    # 在真的会话临时目录根下造 <项目目录名>/<会话号>/scratchpad/rerun/target（项目目录名那一层是自己建的临时目录），只扫本会话那一份，不扫真的工作根
    session_temporary_root = work_roots.claude_session_temporary_root()
    os.makedirs(session_temporary_root, exist_ok=True)
    project_directory = tempfile.mkdtemp(prefix="-home-agent-watch-selftest-", dir=session_temporary_root)
    try:
        session_id = f"agent-watch-selftest-{os.getpid()}"
        session_directory = os.path.join(project_directory, session_id)
        scratchpad_build = os.path.join(session_directory, "scratchpad", "rerun", "target")
        os.makedirs(os.path.join(scratchpad_build, "release"))
        with open(os.path.join(scratchpad_build, "CACHEDIR.TAG"), "wb") as handle:
            handle.write(tag)
        moment = time.time() - 30 * 3600
        for directory, directories, files in os.walk(scratchpad_build, topdown=False):
            for name in [*files, *directories]:
                os.utime(os.path.join(directory, name), (moment, moment), follow_symlinks=False)
            os.utime(directory, (moment, moment))
        saved_override = os.environ.pop(SCRATCH_ROOTS_ENV, None)
        try:
            roots = scratch_hog_roots({session_id})
        finally:
            if saved_override is not None:
                os.environ[SCRATCH_ROOTS_ENV] = saved_override
        checks += 1
        if session_directory not in roots:
            failures.append(f"会话 scratchpad 所在的本会话目录在草稿根里：应当有 {session_directory}，实际 {roots}")
        lines = scratch_hog_report([session_directory], datetime.now(timezone.utc), 6)
        checks += 1
        if not any(line.startswith(f"  编译目录 {scratchpad_build}：") for line in lines):
            failures.append(f"会话 scratchpad 里超阈值的编译目录照报：应当报 {scratchpad_build}，实际 {lines}")
    finally:
        shutil.rmtree(project_directory, ignore_errors=True)
    return checks


def selftest_root_free(work, failures, watch_runs):
    """「根分区余量过低」的自检：读可用量的函数换成假的（不读 os.statvfs），阈值 128 GiB 时判一次过线（可用 40 GiB）、一次不过线（可用 500 GiB），阈值 0 不查；
    再起看门狗子进程（--agents 与 --processes-only 两条路的 report，环境变量 AGENT_WATCH_ROOT_FREE_BYTES 给假的可用量）看告警、退出码与 --ack。返回查了几格。
    弄坏开关：rootfree（过线也不报）。"""
    checks = 0
    asked = []

    def fake_free(gibibytes):
        def read(path):
            asked.append(path)
            return int(gibibytes * BYTES_PER_GIBIBYTE)
        return read
    threshold = argparse.Namespace(root_free_gib_alert=128)
    lines, found = root_free_report(threshold, fake_free(40))
    checks += 1
    if ([name for name, _, _ in found] != [ROOT_FREE_ALERT] or "40.0 GiB" not in found[0][1] or "128 GiB" not in found[0][1]
            or not all(text in found[0][2] for text in ("先查各会话的副本", "crash-amplification/runs/", "du -sh", "别人在用的不删",
                                                        f"--ack {ACK_CLASS_PREFIX}{ROOT_FREE_ALERT}"))):
        failures.append(f"根分区可用 40 GiB、阈值 128 GiB 时应当报一条「{ROOT_FREE_ALERT}」，说明写可用与阈值、出路先点名各会话的副本、再点名崩溃放量 runs/ 下的运行目录、"
                        f"以 du 为准、别人在用的不删、--ack 写法，实际 {found}")
    checks += 1
    if asked != [root_free_path()] or not os.path.exists(asked[0]):
        failures.append(f"根分区可用量应当只读一次、读的是工作根（还没建的取第一个在的上层）{root_free_path()}，实际读了 {asked}")
    # 崩溃放量家目录（crash_amplification_home）：没设变量是工作根下的 crash-amplification、不经家目录，设了照它；出路点名的是此刻环境下那一份的 runs/
    checks += 1
    expected_home = os.path.join(work_roots.work_root(), CRASH_AMPLIFICATION_ENTRY)
    configured_home = os.path.join(work, "amplification-home")
    unset_home = crash_amplification_home({})
    set_home = crash_amplification_home({CRASH_AMPLIFICATION_HOME_VARIABLE: configured_home})
    # 出路那一半只在过线那一次报了告警时核：没报归「可用 40 GiB 时应当报」那一格判，这一格只判家目录取哪一份，不跟着那一格一起红
    if (unset_home != expected_home or set_home != configured_home
            or (found and f"{crash_amplification_home()}/runs/" not in found[0][2])):
        failures.append(f"没设 {CRASH_AMPLIFICATION_HOME_VARIABLE} 时崩溃放量家目录应当是工作根下的 crash-amplification（{expected_home}）、设了照它（{configured_home}），"
                        f"报了告警时出路点名那一份的 runs/，实际没设时 {unset_home}、设了时 {set_home}、出路 {found[0][2] if found else '没报（这一半不核）'}")
    lines, found = root_free_report(threshold, fake_free(500))
    checks += 1
    if found or not any(line.startswith("根分区余量：") and "可用 500.0 GiB" in line for line in lines):
        failures.append(f"根分区可用 500 GiB、阈值 128 GiB 时不该报，报告里应当记一行可用多少，实际告警 {found}、报告行 {lines}")
    lines, found = root_free_report(argparse.Namespace(root_free_gib_alert=0), fake_free(1))
    checks += 1
    if found or not lines or "这一类没查" not in lines[0]:
        failures.append(f"--root-free-gib-alert 写 0 时应当不报、写明这一类没查，实际告警 {found}、报告行 {lines}")
    # 端到端：看门狗子进程照环境变量读假的可用量；--agents 那条路过线退 3、--ack 之后退 0 并写「已确认接着盯」、不过线退 0
    session = os.path.join(work, "root-free-session")
    write_transcript(session, "rootfreerunning", [bash_use(1, "t1", "ls", "m1"), bash_result(0.5, "t1")],
                     {"agentType": "implementation-writer", "description": "还在跑"})

    def report_with(gibibytes, *extra):
        watch_runs.append(["report", "--agents", "rootfreerunning", f"root-free-gibibytes={gibibytes}", *extra])
        return subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--agents", "rootfreerunning", "--session-dir", session,
                               "--process-root-pid", "0", "--root-free-gib-alert", "128", *extra], capture_output=True, text=True, timeout=120,
                              env=dict(os.environ, **{ROOT_FREE_BYTES_VARIABLE: str(int(gibibytes * BYTES_PER_GIBIBYTE))}))
    low = report_with(40)
    checks += 1
    if low.returncode != 3 or f"⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）：" not in low.stdout:
        failures.append(f"看门狗 report 在根分区可用 40 GiB 时应当报「⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）」、退出码 3，实际 {low.returncode}：{low.stdout[-600:]}")
    acked = report_with(40, "--ack", f"{ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}")
    checks += 1
    if acked.returncode != 0 or f"已确认接着盯（--ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT}）" not in acked.stdout:
        failures.append(f"根分区余量过低被 --ack {ROOT_FREE_SUBJECT}:{ROOT_FREE_ALERT} 确认之后看门狗 report 应当退 0、写「已确认接着盯」，实际 {acked.returncode}：{acked.stdout[-600:]}")
    high = report_with(500)
    checks += 1
    if high.returncode != 0 or ROOT_FREE_ALERT in high.stdout or "可用 500.0 GiB" not in high.stdout:
        failures.append(f"看门狗 report 在根分区可用 500 GiB 时不该报、应当退 0 并记一行可用多少，实际 {high.returncode}：{high.stdout[-600:]}")
    # --processes-only 那条路：盯一个自己起的 sleep（它底下没有别的进程，进程一半不报），过线同样报、退 3；查完只停自己起的这一个
    sleeper = subprocess.Popen(["sleep", "120"])
    try:
        watch_runs.append(["report", "--processes-only", "root-free-gibibytes=40"])
        processes_only = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--processes-only", "--process-root-pid", str(sleeper.pid),
                                         "--root-free-gib-alert", "128"], capture_output=True, text=True, timeout=120,
                                        env=dict(os.environ, **{ROOT_FREE_BYTES_VARIABLE: str(40 * BYTES_PER_GIBIBYTE)}))
    finally:
        sleeper.terminate()
        sleeper.wait()
    checks += 1
    if processes_only.returncode != 3 or f"⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）：" not in processes_only.stdout:
        failures.append(f"--processes-only 的 report 在根分区可用 40 GiB 时应当同样报「⚠️ {ROOT_FREE_ALERT}（{ROOT_FREE_SUBJECT}）」、退出码 3，"
                        f"实际 {processes_only.returncode}：{processes_only.stdout[-600:]}")
    return checks


def selftest_stale_sessions(work, failures, watch_runs):
    """「重启前旧进程留下的子 agent 记录判已失效」的自检：造登记目录（~/.claude/sessions 的形态）与项目目录，三类记录：
    一、已不在的会话（登记的进程已退出）里的、本会话里的 → 前者不报「工具调用过长」「无动静」，后者照报；
    二、会话号还活着、但最后一条记录早于它现在这个进程启动时刻的 → 不报，晚于的照报。返回查了几格。
    弄坏开关：stalesession（第一类红）、stalesessionrestart（第二类红）。"""
    checks = 0
    registry = os.path.join(work, "session-registry")
    projects = os.path.join(work, "stale-projects", "project")
    os.makedirs(registry, exist_ok=True)
    finished = subprocess.Popen(["true"])
    finished.wait()   # 已退出并收了尸的子进程：它的 pid 此刻不在 /proc
    own_start = process_start_ticks(os.getpid())
    for file_name, session_id, pid, proc_start, started_minutes_ago in (
            ("dead.json", "dead-session", finished.pid, "1", 600),
            ("live.json", "live-session", os.getpid(), own_start, 600),
            ("restarted.json", "restarted-session", os.getpid(), own_start, 10),
            ("reused-pid.json", "reused-pid-session", os.getpid(), "1", 600)):   # pid 活着但被别的进程占了（procStart 对不上）：也算不在
        with open(os.path.join(registry, file_name), "w", encoding="utf-8") as handle:
            json.dump({"pid": pid, "sessionId": session_id, "procStart": proc_start,
                       "startedAt": int((time.time() - started_minutes_ago * 60) * 1000)}, handle)
    long_tool = [record_at(40, "user", "派发"), bash_use(40, "t1", "cargo test --release", "m1")]
    for session_id, agent_id, records in (
            ("dead-session", "deadlong", long_tool),                                                       # 工具调用过长的形状
            ("dead-session", "deadidle", [bash_use(30, "t1", "ls", "m1"), bash_result(29, "t1")]),         # 无动静的形状
            ("reused-pid-session", "reusedpid", long_tool),
            ("live-session", "livelong", long_tool),                                                       # 本会话的：照报
            ("restarted-session", "beforestart", long_tool),                                               # 进程 10 分钟前启动，这一条最后一条记录 40 分钟前：旧进程的
            ("restarted-session", "afterstart", [record_at(12, "user", "派发"), bash_use(9, "t1", "cargo test --release", "m1")])):   # 启动之后派的：照报
        write_transcript(os.path.join(projects, session_id), agent_id, records, {"agentType": "implementation-writer", "description": agent_id})
    environment = dict(os.environ, **{SESSION_REGISTRY_ENV: registry, "AGENT_WATCH_PROJECTS_ROOT": os.path.join(work, "stale-projects"),
                                      "CLAUDE_CODE_SESSION_ID": "live-session", SCRATCH_ROOTS_ENV: os.path.join(work, "no-scratch")})

    def report_for(agent_ids):
        watch_runs.append(["report", "--agents", agent_ids, "stale"])
        return subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--agents", agent_ids, "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=120, env=environment)
    everything = report_for("deadlong,deadidle,reusedpid,livelong,beforestart,afterstart")
    alerted = {match.group(1) for match in re.finditer(r"^⚠️ [^（\n]+（(\w+)）：", everything.stdout, re.M)}
    checks += 1
    if alerted != {"livelong", "afterstart"}:
        failures.append("重启前旧进程留下的记录（已不在的会话 deadlong / deadidle / reusedpid、会话号还活着但最后一条记录早于进程启动的 beforestart）不该报告警，"
                        f"本会话的 livelong 与进程启动之后的 afterstart 照报；实际告警的是 {sorted(alerted)}：{everything.stdout[-900:]}")
    checks += 1
    for agent_id in ("deadlong", "deadidle", "reusedpid", "beforestart"):
        if not re.search(rf"子 agent {agent_id} .*状态=已失效", everything.stdout):
            failures.append(f"子 agent {agent_id} 应当在报告里标成「状态=已失效」，实际：{everything.stdout[-900:]}")
            break
    only_stale = report_for("deadlong,deadidle,beforestart")
    checks += 1
    if only_stale.returncode != 0 or "⚠️" in only_stale.stdout:
        failures.append(f"被看的只有重启前旧进程留下的记录时应当没有告警、退出码 0，实际 {only_stale.returncode}：{only_stale.stdout[-600:]}")
    unreadable = dict(environment, **{SESSION_REGISTRY_ENV: os.path.join(work, "no-such-registry")})
    watch_runs.append(["report", "--agents", "deadlong", "unreadable-registry"])
    unjudged = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "report", "--agents", "deadlong", "--process-root-pid", "0"],
                              capture_output=True, text=True, timeout=120, env=unreadable)
    checks += 1
    if "⚠️ 工具调用过长（deadlong）" not in unjudged.stdout:
        failures.append(f"登记目录读不了时判不了会话在不在，不该把子 agent 当成已失效而吞掉告警，实际：{unjudged.stdout[-600:]}")
    return checks


def selftest_leftover_background(work, thresholds, failures, watch_runs, probed_processes):
    """「交回之后后台还在跑」的自检：另起会话目录，免得这些开着输出文件的进程搅到别的看门狗自检。返回查了几个合成子 agent。
    每个要看的子 agent 起过一个后台任务，自检起一个 sleep 把它的输出文件开成标准输出（harness 起的后台命令就是这个形状）。"""
    leftover_session = os.path.join(work, "leftover-session")
    holders = {}

    def write_leftover_agent(session_directory, agent_id, started_minutes_ago, tail_records, description, keep_output_open=True):
        output_path = os.path.join(work, f"{agent_id}.output")
        with open(output_path, "a") as output_handle:
            if keep_output_open:
                holders[agent_id] = subprocess.Popen(["sleep", "300"], stdout=output_handle)
                probed_processes.append(holders[agent_id])
        started_record = record_at(started_minutes_ago - 0.05, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                   "content": f"Command running in background with ID: bg{agent_id}. Output is being written to: {output_path}. "
                                              "You will be notified when it completes."}])
        write_transcript(session_directory, agent_id, [bash_use(started_minutes_ago, "t1", "until [ -f /nonexistent ]; do sleep 5; done", "m1"),
                                                       started_record, *tail_records],
                         {"agentType": "three-way-verifier", "description": description})

    def stop_holder(agent_id):
        holders[agent_id].kill()
        holders[agent_id].wait()

    def alerting_pids(found):
        """{子 agent id: 告警里报的进程号}；说明不是「进程 <pid> 已跑 …」开头的，原样放说明，比对时照样对不上。"""
        pids = {}
        for owner, name, explanation, _ in found:
            if name == LEFTOVER_BACKGROUND_ALERT:
                reported_pid = re.match(r"进程 (\d+) 已跑 ", explanation)
                pids[owner] = int(reported_pid.group(1)) if reported_pid else explanation
        return pids

    try:
        write_leftover_agent(leftover_session, "leftoverhandedback", 12, [*handback_records(10, "h1", "m2"),
                             record_at(9.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "交回 10 分钟、它起的等待循环还在跑")
        write_leftover_agent(leftover_session, "leftovergrace", 1, [*handback_records(0.6, "h1", "m2"),
                             record_at(0.4, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "交回 30 秒、还在宽限里")
        write_leftover_agent(leftover_session, "leftoverstopped", 12, [
                             record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                             "结束本轮等后台任务时被 TaskStop 停掉")
        write_leftover_agent(leftover_session, "leftoverfailed", 12, [
                             record_at(9.7, "assistant", [{"type": "text", "text": "接着跑"}], message_id="m2")],
                             "失败（最近一次任务通知是 failed）")
        write_leftover_agent(leftover_session, "leftoverrunning", 12, [
                             record_at(9.8, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
                             "没交回、结束本轮在等自己的后台任务")
        write_leftover_agent(leftover_session, "leftoverresumed", 30, [
                             record_at(20, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2"),
                             record_at(3, "user", "续做：接着跑", origin_kind="coordinator"), bash_use(2, "t2", "cargo build", "m3")],
                             "上一个 Claude 进程退出时没跑完（stopped）、之后又被续做、正在跑")
        write_session_stops(leftover_session, {"leftoverstopped": 9})
        with open(leftover_session + ".jsonl", "a", encoding="utf-8") as handle:
            failed_notification = record_at(9.5, "user", "<task-notification>\n<task-id>leftoverfailed</task-id>\n<status>failed</status>\n"
                                                         "<summary>Agent failed: API error</summary>\n</task-notification>", origin_kind="task-notification")
            stopped_notification = {"type": "queue-operation", "operation": "enqueue", "timestamp": record_at(15, "user", "")["timestamp"],
                                    "content": "<task-notification>\n<task-id>leftoverresumed</task-id>\n<task-id>aother</task-id>\n<status>stopped</status>\n"
                                               "<summary>2 background agents didn't finish before the previous session ended</summary>\n</task-notification>"}
            handle.write(json.dumps(failed_notification, ensure_ascii=False) + "\n")
            handle.write(json.dumps(stopped_notification, ensure_ascii=False) + "\n")
        wanted_alerting = {agent_id: holders[agent_id].pid for agent_id in ("leftoverhandedback", "leftoverstopped", "leftoverfailed")}
        lines, found, pending_until = leftover_background_report({leftover_session}, datetime.now(timezone.utc), thresholds)
        if alerting_pids(found) != wanted_alerting:
            failures.append(f"「{LEFTOVER_BACKGROUND_ALERT}」应当只报交回、被 TaskStop 停掉、任务通知 failed 的三个、各报开着输出文件的那个进程 {wanted_alerting}"
                            f"（交回不到宽限、没交回在等、stopped 之后又被续做的不报），实际 {alerting_pids(found) or '无'}")
        if pending_until is None or not any("宽限里" in line and "leftovergrace" in line for line in lines):
            failures.append(f"交回 30 秒、宽限 2 分钟的 leftovergrace 应当记成宽限里、给出宽限到期的时刻，实际到期时刻 {pending_until}，报告：{lines}")
        stop_holder("leftovergrace")

        def run_leftover_watch(agent_ids, session_directory, extra_arguments, timeout=60):
            """看门狗到 timeout 秒还没退：不抛出去，退出码记成「超时（N 秒没退出）」交回，那一格照原判据判红、打 ✗ 行，别的格接着跑。"""
            watch_runs.append(["leftover", agent_ids, *extra_arguments])
            try:
                return subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "watch", *SELFTEST_SECTION_OFF, "--agents", agent_ids, "--session-dir", session_directory,
                                       "--process-root-pid", "0", "--interval-seconds", "1", "--max-minutes", "1", *extra_arguments],
                                      capture_output=True, text=True, timeout=timeout)
            except subprocess.TimeoutExpired as expired:
                partial = expired.stdout or ""
                if isinstance(partial, bytes):
                    partial = partial.decode("utf-8", errors="replace")
                return subprocess.CompletedProcess(expired.cmd, f"超时（{timeout} 秒没退出）", partial, "")

        handed_back_pid = wanted_alerting["leftoverhandedback"]
        wanted_alert_line = f"⚠️ {LEFTOVER_BACKGROUND_ALERT}（leftoverhandedback）：进程 {handed_back_pid} 已跑 "
        all_done_run = run_leftover_watch("leftoverhandedback", leftover_session, [])
        if (all_done_run.returncode != 3 or wanted_alert_line not in all_done_run.stdout
                or f"{PROCESS_STOP_COMMAND} {handed_back_pid}" not in all_done_run.stdout):
            failures.append(f"被看的子 agent 全部交回、它丢下的后台进程还在时，看门狗退出前应当报「{wanted_alert_line}…」、下一步给按进程号停的命令、退出码 3，"
                            f"实际退出码 {all_done_run.returncode}，输出：{all_done_run.stdout[-400:]}")
        acknowledged_run = run_leftover_watch("leftoverhandedback", leftover_session,
                                              [argument for pid in wanted_alerting.values() for argument in ("--ack", f"进程:{pid}")])
        if acknowledged_run.returncode != 0 or "已确认接着盯" not in acknowledged_run.stdout or "全部交回或被停" not in acknowledged_run.stdout:
            failures.append(f"「{LEFTOVER_BACKGROUND_ALERT}」的进程都用 --ack 进程:<pid> 确认过时不该叫醒：应当退出码 0、写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-400:]}")
        watch_runs.append(["--processes-only", "--session-dir", "leftover"])
        processes_run = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                                        "--session-dir", leftover_session, "--interval-seconds", "1", "--max-minutes", "0.5", *SELFTEST_SECTION_OFF],
                                       capture_output=True, text=True, timeout=120)
        if processes_run.returncode != 3 or wanted_alert_line not in processes_run.stdout:
            failures.append(f"--processes-only 带 --session-dir 时也应当报「{wanted_alert_line}…」并退出码 3，"
                            f"实际退出码 {processes_run.returncode}，输出：{processes_run.stdout[-400:]}")
        # 被看的子 agent 刚交回、它的后台进程还在宽限里：看门狗不许当场退 0 放过，要等宽限过了再查一次、报出来
        wait_session = os.path.join(work, "leftover-wait-session")
        write_leftover_agent(wait_session, "leftoverjustnow", 0.3, [*handback_records(0.15, "h1", "m2"),
                             record_at(0.04, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                             "刚交回、后台进程还开着输出文件")
        wait_run = run_leftover_watch("leftoverjustnow", wait_session, ["--leftover-background-grace-minutes", "0.15", "--interval-seconds", "30"])
        wanted_wait_line = f"⚠️ {LEFTOVER_BACKGROUND_ALERT}（leftoverjustnow）：进程 {holders['leftoverjustnow'].pid} 已跑 "
        if wait_run.returncode != 3 or wanted_wait_line not in wait_run.stdout:
            failures.append(f"被看的子 agent 刚交回、后台进程还在宽限里时看门狗应当等宽限过了再查、报「{wanted_wait_line}…」退出码 3，"
                            f"实际退出码 {wait_run.returncode}，输出：{wait_run.stdout[-400:]}")
        stop_holder("leftoverhandedback")
        _, found_after_exit, _ = leftover_background_report({leftover_session}, datetime.now(timezone.utc), thresholds)
        if "leftoverhandedback" in alerting_pids(found_after_exit):
            failures.append(f"leftoverhandedback 丢下的进程 {handed_back_pid} 已经没了，不该再报「{LEFTOVER_BACKGROUND_ALERT}」，实际 {alerting_pids(found_after_exit)}")
    finally:
        for process in holders.values():
            process.kill()
            process.wait()
    return len(holders)
