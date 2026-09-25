#!/usr/bin/env python3
"""子 agent 与它们起的进程的定时监控：卡死、空转、永久等待当场报出来。

为什么：2026-09-17 一个实验执行员把 `echo "exit=$?"` 打到标准输出、却在日志里 `until grep -q "^exit="`，
等一行永远不会出现的字，主 agent 查进度才发现；同一天另一个会话的层 0 全量测试跑了三个多小时没有任何输出，
没人分得清是慢还是卡。经过：records/2026-09-17-已分配口径三方与两个实验.md 第六节。

用法：
    agent-watch.py report --agents ID[,ID…]          # 看一眼：每个子 agent 的状态、最后一条命令、告警
    agent-watch.py report --session-dir DIR           # 看这个会话里最近活动过的全部子 agent
    agent-watch.py watch  --agents ID[,ID…]          # 看门狗：每隔一段时间查一次，有告警或全部结束就退出
    agent-watch.py watch  --processes-only            # 不看子 agent，只盯这个 Claude 实例底下的进程：主 agent 自己放后台的长命令用它盯
    agent-watch.py cost   --agents ID[,ID…]          # 用量：调用次数、起步与平均上下文、新输入、读缓存、输出、整份重写、工具结果被什么撑大
    agent-watch.py --selftest                         # 造假的会话记录与进程走一遍各种告警；AGENT_WATCH_BREAK=<项> 时必须判红

主 agent 派发之后用 Bash 的 run_in_background 起 watch：它一退出，harness 就会叫醒主 agent。
它只看、只报，不停任何子 agent、不杀任何进程：子 agent 与脚本不强制结束，结束不结束由主 agent 看了报告定（用户 2026-09-17）。
退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）；2 用法错。
带 --processes-only 时 0 是看门狗起的时候就在跑的那些进程都退出了（起的时候一个都没有也退 0，报告里写明是哪一种）。
「交回」按交回工具（SubagentHandback）成功返回判：只结束本轮、没交回的子 agent 还在等后台任务，不算结束。
交回之后只有新指令撤销它（续做消息，认法见 is_new_instruction）；交回之后被自己后台任务的完成通知叫醒、又调了工具再结束本轮的，仍算交回。
「被停」认两处：子 agent 会话记录里的 `[Request interrupted`（停在工具调用中途）；主会话记录（<会话 id>.jsonl）里 TaskStop 对它的成功结果，
时刻不早于它自己最后一条记录减 30 秒（停在结束本轮、等后台任务的时候，子 agent 的会话记录里一个字都不写；停了之后又被续做的不算）。
主会话记录不在那个位置时，报告里写明「被停只按打断判」，不悄悄退回。
「结束本轮却不会醒」：子 agent 结束本轮、没交回，而它起过的后台任务（工具结果「Command running in background with ID: …」，
或前台命令跑满超时被挪进后台的「… was moved to the background (ID: …)」）都已收到完成通知
（会话记录里 origin 为 task-notification 的消息，或忙时排进队的 queued_command 附件），结束本轮之后也没有新的通知进来——没有任何东西会叫醒它，
过 60 秒就报，不等 10 分钟的「无动静」。

hook 的检出也在这里汇给主 agent：`.claude/hooks/bash-command-detector.sh` 除起看门狗的错误写法当场拒绝之外只记不拦，把检出写进检出记录；
watch 每 15 秒读一次，读到被看子 agent 所在会话的检出就退出、叫醒主 agent。

会话记录的位置：~/.claude/projects/<项目>/<会话 id>/subagents/agent-<id>.jsonl，同名 .meta.json 里有 agentType 与 description。
进程这一半只看运行这个脚本的那个 Claude 实例的后代进程（主 agent 与它派出的子 agent 起的命令都在里面），不看别的会话。

`--processes-only`：不给子 agent，只看进程这一半——主 agent 自己放后台的长命令没有会话记录可看，只有进程。
盯的是看门狗起的时候这个实例底下已经在跑的进程（整棵子树，不只叶子：gate.sh 这类长命令一段一段起叶子，叶子换了，命令本身还在跑；
看门狗自己那一支与 COMMAND_MARKERS_NOT_WATCHED 不算），它们都退出了（剩下没人收尸的僵尸也算退出）就退 0。
告警与进程一节同一套：写着的文件 --process-stale-minutes 没动、CPU 也不涨，stdin 若是管道、上游写端连同子孙也既不算也不写，报「进程无输出」，跑过 --process-max-minutes 报「进程过长」，
报了就退 3；--ack 进程:PID 与定时回报照旧。它不读 hook 的检出记录：认不出是哪个会话的。
给了 --session-dir（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找）就同时查「交回之后后台还在跑」；没给就在报告里写明这一类没查。

「交回之后后台还在跑」：扫会话目录里全部子 agent（不只被看的那几个）。已交回、被停、或主会话记录里最近一次任务通知是
failed / killed / stopped（stopped：上一个 Claude 进程退出时它还没跑完）的子 agent，它用 run_in_background 起过的后台任务的输出文件
还有进程开着、而且它结束已过 --leftover-background-grace-minutes（交回那一刻最后一条完成通知可能还在路上），每个最上层的进程报一条，
--ack 进程:PID 确认。2026-09-24 查到两条交回之后丢下的 `until … do sleep` 跑了 5 个多小时，看门狗只盯没交回的，没人报。
被看的子 agent 全部交回时照旧退出，但退出前查一遍这一类；只剩宽限里的，等宽限过了再查一次才退。
只认会话记录里写着输出文件路径的后台任务：命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程，与子 agent 之间没有可认的联系，这里认不出。

「跑满 N 小时要主 agent 问一次」：还没交回、没被停（任务通知也不是 failed / killed / stopped）的子 agent，从派发（它会话记录的第一条）起
每跑满 --ask-every-minutes（默认 60 分钟，即每个整点）报一次，N 是已跨过的最近那一格；主 agent 中途发的消息不重新计时。
那一格之后主 agent 给它发过消息就算问过了、这一格不再报：看门狗每次起都从头读，按「那一格之后有没有消息」判才做得到每个整点只报一次。
主 agent 的消息取两处里晚的那个：子 agent 会话记录里送达的（origin 为 coordinator 的消息，或忙时排进队、附件自己带 origin coordinator 的
queued_command），与主会话记录里 SendMessage 发给它的成功结果（忙着跑一条长工具调用的子 agent，消息要等它下一轮才落进它自己的会话记录）。
--ack <agent 号>:跑满 N 小时 只确认那一格，下一格照报。2026-09-24 几个子 agent 结束本轮去等后台任务、一等两个多小时，
主 agent 发了一条消息它们当场醒来、很快交回；看门狗只在后台任务全结束、通知没到时报「结束本轮却不会醒」，等的还开着就不报。

「重型测试没带前缀」：每次复检扫这个 Claude 实例底下的全部进程（看门狗自己那一支、另一个看门狗连同它的自检整棵不扫），
每个进程的 /proc/<pid>/cmdline（按 NUL 切开）与 /proc/<pid>/cwd 交给 .claude/hooks/lib_heavy_tests.py 的 classify_process 判
（与执行前的闸 heavy-test-guard.sh 同一份判定，按文件路径导入），判为重型、而 /proc/<pid>/environ 里没有 SINGLEFS_HEAVY_TESTS=commit
或 =user-request 的报，叫醒主 agent。`bash -c '…'` 这一层不单判：写在代码串里的前缀不在这个 shell 自己的环境里，它起的命令各自是一个进程、各判各的。
没带前缀、底下却有带前缀的重型进程的不报（`timeout 600 env SINGLEFS_HEAVY_TESTS=commit …` 里的 timeout）；一条链上只报最上面那个，
下一步给整棵子树从最底层往上逐个停的次序。归属照「交回之后后台还在跑」那套：它或它的上层开着哪个子 agent（或主 agent）后台任务的输出文件，
认不出写认不出。读不到 cmdline、cwd、environ 的（进程刚退出、权限不够）跳过不报；lib_heavy_tests.py 导入失败时报告里写明这一类没查。
--ack 进程:PID 确认。射程之外：在同一个 shell 里 source 进来跑、不另起进程的。2026-09-24 实十九把重型测试写进 run-chain.sh，
执行前的闸看不进脚本（records/2026-09-16-subagent拆分提案.md 第四十节第 16 行）。

「书记员攒下的没跟上检出」：kb-scribe-followup.sh 写的「书记官写入之后相关记录没跟上」检出，出自还没交回的 kb-scribe 时不叫醒，按书记员攒着；
它交回、被停或失败时报一条汇总（哪几道门禁各几次、最后一次几点、涉及哪些文件），主 agent 对着它的交回报告核那几道门禁跑没跑。
看门狗起的时候把被看的书记员此前的这一类检出从检出记录开头重读一遍（上一个看门狗退出时还没报的接得上）；
记录里没有 agent 号、或它的会话记录找不到的认不出是哪一个：那个会话里还有书记员在跑就攒着，一个都不在跑了才报。
书记员的别的类别检出照旧立刻叫醒。--ack <书记员 agent 号>:书记员攒下的没跟上检出 确认。2026-09-24 书记员写到一半每写一处就叫醒一次主 agent，
一小时里十来次（同一张表第 6、16 行）。

「进程常驻内存过线」：这个 Claude 实例底下整棵子树（不只跑得久的叶子；看门狗自己那一支、别的看门狗与它的自检整棵不看）任一进程的常驻内存
（/proc/<pid>/statm 第二列乘页大小）超过 --process-rss-gibibytes（默认 8 GiB，依据见那个选项；写 0 不查）就报，叫醒主 agent：
写进程号、命令、常驻内存、已跑多久、归属（照「交回之后后台还在跑」那套认开着输出文件的子 agent，认不出写认不出）与「怎么办」。
整查（--interval-seconds）之外，watch 与 --processes-only 两次整查之间每隔 --detection-poll-seconds 再单查一遍常驻内存，过线就马上整查一次、叫醒主 agent：
涨得慢的等不到下一次整查；涨得快的（E142 那条变异每秒约 5 GB、十几秒吃满整机）单查也来不及，只有内存上限（research/scripts/run-with-memory-cap.sh）挡得住，
这一类告警是它之外的第二道。它只报不杀（同一份文件头「只看、只报」）；--ack 进程:PID 确认。
2026-09-25 两次整机 OOM：E142 的变异跑道里一条变异让测试无界分配，涨到 53.6 GiB 与 56.6 GiB 才被内核杀掉，连带杀了本地模型服务与 Claude Code 会话
（records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
"""
import argparse
import glob
import importlib.util
import json
import os
import re
import signal
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
# 前台命令跑满超时被 harness 挪进后台，工具结果是另一种写法，之后同样会来完成通知。不认它，在等这种任务的子 agent 会被报「结束本轮却不会醒」
# （2026-09-23 门禁审计会话的 ac26abd494cda0968：doc-lint 超时转后台，看门狗报「起过的 0 个后台任务」；本机这个项目的子 agent 会话记录里 79 份有这种写法）。
BACKGROUND_MOVED = re.compile(r"^Command did not complete within its \d+s timeout and was moved to the background \(ID: (\w+)\)")
NEW_INSTRUCTION_ORIGINS = ("coordinator", "peer")   # 续做消息与别的会话发来的消息；task-notification 是自己后台任务的完成通知，不算
BACKGROUND_OUTPUT_PATH = re.compile(r"Output is being written to: (\S+?\.output)")
TASK_NOTIFICATION_ID = re.compile(r"<task-id>(\w+)</task-id>")
# 进程一节不看的命令：claude 本体、IDE、node（常驻的 MCP 一类）、别的看门狗。进程告警与 --processes-only 盯哪些进程读的是这同一份。
COMMAND_MARKERS_NOT_WATCHED = ("native-binary/claude", "vscode-server", "/node ", "agent-watch.py")
EXITED_PROCESS_STATES = ("Z", "X")   # /proc/<pid>/stat 的状态：僵尸（退出了、等父进程收尸）与已死，都算退出了
LEFTOVER_BACKGROUND_ALERT = "交回之后后台还在跑"
# 主会话记录里任务通知的这几种状态说明子 agent 已经不在跑了：failed 撞限额或报错、killed 被停、stopped 上一个 Claude 进程退出时它还没跑完
ENDED_NOTIFICATION_STATUSES = ("failed", "killed", "stopped")
TASK_NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
TASK_NOTIFICATION_STATUS = re.compile(r"<status>([a-z_]+)</status>")
# 会话记录按字节扫后台输出文件路径：几百兆的会话记录不逐行解 JSON，先看它提没提到有进程开着的那几个文件
BACKGROUND_OUTPUT_PATH_BYTES = re.compile(rb"Output is being written to: (\S+?\.output)")
PROCESS_STOP_COMMAND = "python3 .claude/singlefs-ai-sop/scripts/proc.py stop"
# 「跑满 N 小时要主 agent 问一次」：告警名是「跑满 <已跨过的那一格> 要主 agent 问一次」，--ack 写「<agent 号>:跑满 <那一格>」
ROUTINE_INQUIRY_ALERT_PREFIX = "跑满 "
ROUTINE_INQUIRY_ALERT_SUFFIX = "要主 agent 问一次"
ROUTINE_INQUIRY_NEXT_STEP = ("主 agent 给它发一条例行询问，并写明「把回答写进它草稿目录的 progress.md」（在做什么、还差几步、在等哪个进程、预计多久；"
                             "等的已经结束就接着做或交回），发完再起看门狗")
# 主会话记录里 SendMessage 发给子 agent 的成功结果的两种写法（2026-09-24 查本机这个项目的主会话记录：157 条排队、另有续做的「Resuming agent」），
# 都带 pin，续做的另带 resumedAgentId；发给别的会话的写法不同，不认
MAIN_AGENT_MESSAGE_QUEUED = "Message queued for delivery to "
MAIN_AGENT_MESSAGE_RESUMED = "resumedAgentId"
# 「重型测试没带前缀」：判定与执行前的闸 heavy-test-guard.sh 同一份，从 research/scripts/ 往上两级是仓根
HEAVY_TESTS_LIBRARY_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                        ".claude", "hooks", "lib_heavy_tests.py")
HEAVY_TEST_VARIABLE = "SINGLEFS_HEAVY_TESTS"
HEAVY_TEST_OCCASIONS = ("commit", "user-request")
HEAVY_TEST_ALERT = "重型测试没带前缀"
HEAVY_TEST_STOP_ORDER_SHOWN = 24   # 下一步里列出的停止次序最多这么多个进程号，再多写「共 N 个」
# 「进程常驻内存过线」
MEMORY_ALERT = "进程常驻内存过线"
PAGE_SIZE_BYTES = os.sysconf("SC_PAGE_SIZE")
BYTES_PER_GIBIBYTE = 1024 ** 3
# 「书记员攒下的没跟上检出」：kb-scribe-followup.sh 写的检出（findings 每条都以这句开头）
SCRIBE_AGENT_TYPE = "kb-scribe"
SCRIBE_FOLLOWUP_FINDING_PREFIX = "书记官写入之后相关记录没跟上"
SCRIBE_FOLLOWUP_ALERT = "书记员攒下的没跟上检出"
SCRIBE_FOLLOWUP_FILES_SHOWN = 8
SCRIBE_FOLLOWUP_NEXT_STEP = ("主 agent 对着它的交回报告核这几道门禁它最后跑没跑、判没判绿：报告里写了绿的，自己再跑一次那几道核实（都是轻阶段，谁都能跑）；"
                             "报告里没写或写了红的，派 kb-scribe 补，或者自己补")


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
        self.interrupted_timestamp = None    # 最后一次被打断（[Request interrupted）的时刻
        self.task_stopped_timestamp = None   # 按主会话记录里的 TaskStop 判成被停时，那次停的时刻
        self.main_agent_message_delivered_timestamp = None   # 它会话记录里主 agent 的消息最晚一次送达的时刻（origin coordinator）
        self.state = "思考中"
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
        handback_tool_ids = set()
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
                            # 按调工具撤销会被判成没交回、报「结束本轮却不会醒」（2026-09-23 门禁审计会话的 ac26abd494cda0968：交回、结束本轮、
                            # 超时转后台的 doc-lint 的完成通知进来、grep 一次输出、结束本轮）。
                            if block.get("name") == "SubagentHandback":
                                handback_tool_ids.add(block.get("id"))
                            elif BROKEN_DETECTION == "toolrevoke":
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
                            plain_result_text = block.get("content") if isinstance(block.get("content"), str) else ""
                            started = BACKGROUND_STARTED.match(plain_result_text) or (
                                BACKGROUND_MOVED.match(plain_result_text) if BROKEN_DETECTION != "movedbackground" else None)
                            if started:
                                self.background_started.append(started.group(1))
                                output_path = BACKGROUND_OUTPUT_PATH.search(block.get("content"))
                                if output_path:
                                    self.background_output_paths[started.group(1)] = output_path.group(1)
                            if block.get("tool_use_id") in handback_tool_ids:
                                result_text = json.dumps(block.get("content"), ensure_ascii=False).replace("\\", "").replace(" ", "")
                                if '"success":true' in result_text:
                                    is_handed_back = True
                                    handback_timestamp = timestamp
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
                # 看门狗照上一轮的结束时刻报「结束本轮却不会醒」（2026-09-19 E155 执行员续派第二段，最后动作 4 秒前就被报了）。
                if (record.get("origin") or {}).get("kind") == "coordinator" and BROKEN_DETECTION != "resumeturn":
                    last_kind = "resumed"
        # 一次工具调用只有在「它之后再没有别的记录」时才算还在跑。断网 / 被停时砍掉的那一次调用
        # 只落了发起、永远等不到结果，旧写法会一直把它当成在跑，于是「工具调用过长」每一轮都假报一次
        # （2026-09-22 实测：三条腿被断网停掉、续做之后又调了 5 次，看门狗照旧报那条已死的 Bash 跑了 10 分 55 秒）。
        if BROKEN_DETECTION != "stalepending" and self.last_timestamp is not None:
            pending_by_id = {
                key: item for key, item in pending_by_id.items() if item[0] >= self.last_timestamp
            }
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
            self.handed_back_timestamp = handback_timestamp
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

    def last_main_agent_message_timestamp(self):
        """主 agent 最晚一次给它发消息的时刻：它会话记录里送达的与主会话记录里发出的，取晚的那个；都没有是 None。"""
        sent = None if BROKEN_DETECTION == "inquirysent" else self.main_agent_message_sent_timestamp
        moments = [moment for moment in (self.main_agent_message_delivered_timestamp, sent) if moment is not None]
        return max(moments) if moments else None

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


def elapsed_mark_label(mark_minutes):
    """一格写成人读的样子：整小时写「N 小时」；watch.conf 把 ask-every-minutes 改成不是整小时的，写「M 分钟」。"""
    if mark_minutes % 60 == 0:
        return f"{int(mark_minutes // 60)} 小时"
    return f"{mark_minutes:g} 分钟"


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
    tokyo = ZoneInfo("Asia/Tokyo")
    message_text = (f"主 agent 上一次给它发消息在 {format_duration((now - last_message).total_seconds())} 以前，早于这一格"
                    if last_message is not None else "派发之后主 agent 一次都没给它发过消息")
    return (f"{ROUTINE_INQUIRY_ALERT_PREFIX}{label}{ROUTINE_INQUIRY_ALERT_SUFFIX}",
            f"从派发（{transcript.first_timestamp.astimezone(tokyo).strftime('%H:%M')} JST）起已跑 "
            f"{format_duration((now - transcript.first_timestamp).total_seconds())}，{mark_reached_at.astimezone(tokyo).strftime('%H:%M')} JST 跑满 {label}；"
            f"{message_text}（只是慢、这一格不问就 --ack '{transcript.agent_id}:{ROUTINE_INQUIRY_ALERT_PREFIX}{label}'，只确认这一格）",
            ROUTINE_INQUIRY_NEXT_STEP)


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


def stdin_pipe_writers(reader_pid, table):
    """这个进程的 stdin 是管道（/proc/<pid>/fd/0 指向 pipe:[inode]）时，本机别的活进程里把同一个管道开在写端的那些进程号。
    它自己与它的祖先不算：管道的上游是兄弟一类，不是祖先；祖先的子孙里就有这个进程自己那一支和不相干的兄弟，拿它的整棵子树判「在动」会把告警整个关掉。
    stdin 不是管道、读不到，或写端进程的 /proc 读不了（别的用户的进程），都当作看不到写端。"""
    try:
        stdin_target = os.readlink(f"/proc/{reader_pid}/fd/0")
    except OSError:
        return []
    if not stdin_target.startswith("pipe:["):
        return []
    ancestors, pid = set(), reader_pid
    while pid in table and pid > 1 and BROKEN_DETECTION != "pipeancestor":
        pid = table[pid]["parent"]
        ancestors.add(pid)
    writers = []
    for other in table:
        if other == reader_pid or other in ancestors or table[other]["state"] in EXITED_PROCESS_STATES:
            continue
        try:
            descriptors = os.listdir(f"/proc/{other}/fd")
        except OSError:
            continue
        for descriptor in descriptors:
            try:
                if os.readlink(f"/proc/{other}/fd/{descriptor}") != stdin_target:
                    continue
                flags_line = [line for line in open(f"/proc/{other}/fdinfo/{descriptor}") if line.startswith("flags:")][0]
            except (OSError, IndexError):
                continue
            if (int(flags_line.split()[1], 8) & 0o3) in (1, 2):   # 开在写端（只写或读写）
                writers.append(other)
                break
    return writers


def pipe_upstream_activity(reader_pid, earlier, later, thresholds):
    """写着的文件不动、CPU 也不涨的进程，stdin 是管道而上游在动：返回一句说明；看不到写端、或写端连同子孙既不算也不写，返回 None。
    窗口与「进程无输出」同一套：CPU 比 process_alerts 里 earlier、later 那两次采样（隔 CPU_SAMPLE_SECONDS），已回收的子进程与采样之间新起的子孙也算；
    写着的文件看 --process-stale-minutes 以内变没变过。"""
    if BROKEN_DETECTION == "pipeupstreamalways":
        return "（AGENT_WATCH_BREAK=pipeupstreamalways：这条判据恒真）"
    if BROKEN_DETECTION == "pipeupstream":
        return None
    writers = [pid for pid in stdin_pipe_writers(reader_pid, earlier)
               if pid in later and later[pid]["start_ticks"] == earlier[pid]["start_ticks"]]
    if not writers:
        return None
    children = {}
    for pid, info in later.items():
        children.setdefault(info["parent"], []).append(pid)
    subtree, frontier = set(), list(writers)
    while frontier:
        pid = frontier.pop()
        if pid in subtree or pid == reader_pid or later[pid]["state"] in EXITED_PROCESS_STATES:
            continue
        subtree.add(pid)
        frontier.extend(children.get(pid, []))
    cpu_ticks = 0
    for pid in subtree:
        before = earlier.get(pid)
        started_before_window = before is not None and before["start_ticks"] == later[pid]["start_ticks"]
        cpu_ticks += later[pid]["cpu_ticks_with_reaped"] - (before["cpu_ticks_with_reaped"] if started_before_window else 0)
    cpu_seconds = cpu_ticks / os.sysconf("SC_CLK_TCK")
    file_ages = []
    for pid in subtree:
        for path in written_files(pid):
            try:
                file_ages.append(time.time() - os.path.getmtime(path))
            except OSError:
                continue
    newest_age = min(file_ages, default=None)
    writers_text = f"写端进程 {', '.join(str(pid) for pid in sorted(writers))}（连同子孙 {len(subtree)} 个进程）"
    if cpu_seconds > 0:
        return f"stdin 是管道，{writers_text}{CPU_SAMPLE_SECONDS} 秒里用了 {cpu_seconds:.1f} 核秒 CPU"
    if newest_age is not None and newest_age < thresholds.process_stale_minutes * 60:
        return f"stdin 是管道，{writers_text}写着的文件最近一次变动在 {format_duration(newest_age)} 以前"
    return None


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
        if any(marker in info["command"] for marker in COMMAND_MARKERS_NOT_WATCHED):
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
        # 管道尾（`cargo test … | tail -30`、`… | ugrep`）在等上游、不写也不算：上游连同子孙在算或在写就不是卡住，报一行、不告警。
        # 2026-09-24 实测这两种形态各被报过「进程无输出」，上游的测试二进制一直占满一个核。
        upstream = pipe_upstream_activity(pid, table, later, thresholds)
        if upstream is not None:
            lines.append(f"  进程 {pid} 在等上游：{upstream}，它自己写着的文件 {format_duration(newest_age)} 没动（不告警；"
                         f"跑到 {thresholds.process_max_minutes:.0f} 分钟报「进程过长」）")
            if elapsed >= thresholds.process_max_minutes * 60:
                alerts.append(("进程过长", f"进程 {pid} 已跑 {format_duration(elapsed)}、一直在等还在动的上游：{info['command'][:140]}",
                               "超过预期时长；主 agent 确认它是谁起的、还要多久，再决定接着盯还是处理"))
            continue
        alerts.append(("进程无输出", f"进程 {pid} 已跑 {format_duration(elapsed)}，写着的文件 {format_duration(newest_age)} 没变，"
                                     f"{CPU_SAMPLE_SECONDS} 秒里 CPU 也没涨：{info['command'][:140]}",
                       "既不写也不算，可能卡在等什么（锁、管道、网络、另一个进程）；主 agent 看它在等什么（/proc/<pid>/wchan、它的父进程与输入），"
                       "再决定接着盯、查、还是报给用户"))
    return lines, alerts


HEAVY_TESTS_LIBRARY = None   # (模块或 None, 导入失败的原因或 None)：第一次用时按文件路径导入，之后复用


def heavy_tests_library():
    """.claude/hooks/lib_heavy_tests.py（它自己再导入同目录的 lib_shell_words.py）：返回 (模块, None)，导入失败返回 (None, 原因)。"""
    global HEAVY_TESTS_LIBRARY
    if HEAVY_TESTS_LIBRARY is None:
        try:
            spec = importlib.util.spec_from_file_location("lib_heavy_tests", HEAVY_TESTS_LIBRARY_PATH)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            HEAVY_TESTS_LIBRARY = (module, None)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败都算：报告里写明这一类没查，不让看门狗整个退出
            HEAVY_TESTS_LIBRARY = (None, f"导入 {HEAVY_TESTS_LIBRARY_PATH} 失败（{type(error).__name__}：{error}）")
    return HEAVY_TESTS_LIBRARY


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


def heavy_test_occasion(pid):
    """这个进程环境里 SINGLEFS_HEAVY_TESTS 的值，没有是 None；读不到 /proc/<pid>/environ 抛 OSError。"""
    with open(f"/proc/{pid}/environ", "rb") as handle:
        for entry in handle.read().split(b"\0"):
            if entry.startswith(HEAVY_TEST_VARIABLE.encode() + b"="):
                return entry.split(b"=", 1)[1].decode("utf-8", "replace")
    return None


def is_watchdog_process(arguments):
    """python 起的 agent-watch.py（别的看门狗与它的自检）：它那一支整棵不扫，自检起的假重型进程不是真在跑的测试。"""
    if not os.path.basename(arguments[0]).startswith("python"):
        return False
    script = next((argument for argument in arguments[1:] if not argument.startswith("-")), "")
    return os.path.basename(script) == "agent-watch.py"


def is_shell_code_string_process(library, arguments):
    """`bash -c '…'` 这一层：前缀写在代码串里、不在这个 shell 自己的环境里，它起的命令各自是一个进程，各判各的。"""
    shell_words = library.shell_words
    return (os.path.basename(arguments[0]) in shell_words.SHELL_NAMES
            and shell_words.shell_invocation(arguments[1:]).code_string is not None)


def agent_meta(transcript_path):
    """子 agent 会话记录旁边 .meta.json 里的 (agentType, description)；读不到是 ("?", "")。"""
    try:
        meta = json.load(open(transcript_path[: -len(".jsonl")] + ".meta.json", encoding="utf-8"))
        return meta.get("agentType", "?"), meta.get("description", "")
    except (OSError, ValueError):
        return "?", ""


def owners_by_background_output(pids, table, session_dirs, stop_at_pid):
    """{进程号: 归属说明}：它自己或它的上层（到 stop_at_pid 为止）开着某个后台任务的输出文件（.output），
    会话记录里写着这个输出文件的那个子 agent；主会话记录里写着的算主 agent。与「交回之后后台还在跑」同一套认法。"""
    held_by = {}
    for path, holder_pids in output_files_held_open().items():
        for holder in holder_pids:
            held_by.setdefault(holder, set()).add(path)
    files_by_pid = {}
    for pid in pids:
        files, walker = set(), pid
        while walker in table and walker > 1 and walker != stop_at_pid:
            files |= held_by.get(walker, set())
            walker = table[walker]["parent"]
        files_by_pid[pid] = files
    mentioned_by = []   # [(说明, 它提到的输出文件)]
    if any(files_by_pid.values()):
        for session_dir in sorted(session_dirs):
            for transcript_path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
                agent_type, description = agent_meta(transcript_path)
                agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
                mentioned_by.append((f"子 agent {agent_id} {agent_type}「{description}」", output_paths_mentioned(transcript_path)))
            mentioned_by.append((f"主 agent（会话 {os.path.basename(os.path.normpath(session_dir))}）",
                                 output_paths_mentioned(os.path.normpath(session_dir) + ".jsonl")))
    owners = {}
    for pid, files in files_by_pid.items():
        found = [f"{who}（开着它后台任务的输出文件 {os.path.basename(path)}）"
                 for who, mentioned in mentioned_by for path in sorted(files & mentioned)]
        if found:
            owners[pid] = "；".join(found)
        elif not session_dirs:
            owners[pid] = "认不出（没给会话目录，不知道去哪个会话的记录里找）"
        else:
            owners[pid] = "认不出（它和它的上层都没开着哪个子 agent 或主 agent 后台任务的输出文件：前台跑的、命令里自己放后台的认不出）"
    return owners


def heavy_test_report(root_pid, excluded_pids, session_dirs):
    """这个 Claude 实例底下在跑的重型测试没带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request（判法见文件头）：返回 (报告行, [(告警名, 说明, 下一步)])。"""
    if BROKEN_DETECTION == "heavyscan":
        return [], []
    library, problem = heavy_tests_library()
    if library is None:
        return [f"{HEAVY_TEST_ALERT}：这一类没查——{problem}"], []
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    flagged, judgement_errors = {}, []   # flagged：进程号 → (认出的重型用法, SINGLEFS_HEAVY_TESTS 的值)
    examined, shell_layers, frontier = 0, 0, [root_pid]
    while frontier:
        for child in sorted(children.get(frontier.pop(), [])):
            arguments = process_arguments(child)
            if arguments and is_watchdog_process(arguments) and BROKEN_DETECTION != "heavywatchdog":
                continue   # 别的看门狗与它的自检：整棵不扫
            frontier.append(child)
            if child in excluded_pids or table[child]["state"] in EXITED_PROCESS_STATES or not arguments:
                continue
            examined += 1
            if is_shell_code_string_process(library, arguments) and BROKEN_DETECTION != "heavywrapper":
                shell_layers += 1
                continue
            try:
                directory = os.readlink(f"/proc/{child}/cwd")
                tests = library.classify_process(arguments, directory)
                if tests:
                    flagged[child] = (tests, heavy_test_occasion(child))
            except OSError:
                continue   # 刚退出、权限不够：跳过不报
            except Exception as error:   # 共用判定自己的 bug：这一个进程没判成，写进报告，别的照判
                judgement_errors.append(f"  进程 {child} 没判成（{type(error).__name__}：{error}）：{command_for_display(table[child]['command'])[:100]}")

    def descendants_of(pid):
        found, stack = set(), [pid]
        while stack:
            for child in children.get(stack.pop(), []):
                if child not in found:
                    found.add(child)
                    stack.append(child)
        return found

    def ancestors_of(pid):
        found, walker = set(), table[pid]["parent"]
        while walker in table and walker > 1 and walker != root_pid:
            found.add(walker)
            walker = table[walker]["parent"]
        return found

    def bottom_up(pid):
        """整棵子树从最底层往上的次序（先子孙、后自己），只列还活着的。"""
        order = []
        for child in sorted(children.get(pid, [])):
            order += bottom_up(child)
        if table[pid]["state"] not in EXITED_PROCESS_STATES:
            order.append(pid)
        return order

    prefixed = {pid for pid, (_, occasion) in flagged.items() if occasion in HEAVY_TEST_OCCASIONS and BROKEN_DETECTION != "heavyprefix"}
    unprefixed = {pid for pid in flagged if pid not in prefixed
                  and (BROKEN_DETECTION == "heavyexempt" or not (descendants_of(pid) & prefixed))}
    reported = sorted(pid for pid in unprefixed if BROKEN_DETECTION == "heavytopmost" or not (ancestors_of(pid) & unprefixed))
    owners = owners_by_background_output(reported, table, session_dirs, root_pid) if reported else {}
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    alerts = []
    for pid in reported:
        tests, occasion = flagged[pid]
        below = sorted(descendants_of(pid) & unprefixed)
        categories = list(dict.fromkeys([test.category for test in tests] + [test.category for other in below for test in flagged[other][0]]))
        carried = "没带" if occasion is None else f"带的是 {HEAVY_TEST_VARIABLE}={occasion}"
        below_text = f"，底下另有 {len(below)} 个同样没带前缀的重型进程（{', '.join(map(str, below[:6]))}）" if below else ""
        upper = []
        walker = table[pid]["parent"]
        while walker in table and walker > 1 and walker != root_pid and len(upper) < 3:
            upper.append(f"{walker} {command_for_display(table[walker]['command'])[:60]}")
            walker = table[walker]["parent"]
        stop_order = bottom_up(pid)
        shown = " ".join(map(str, stop_order[:HEAVY_TEST_STOP_ORDER_SHOWN]))
        if len(stop_order) > HEAVY_TEST_STOP_ORDER_SHOWN:
            shown += f" …（共 {len(stop_order)} 个，最后停 {pid}）"
        alerts.append((HEAVY_TEST_ALERT,
                       f"进程 {pid} 已跑 {format_duration(uptime_seconds - table[pid]['start_ticks'] / ticks_per_second)}，"
                       f"{'、'.join(categories)}（{'；'.join(test.detail for test in tests)}），{carried} {HEAVY_TEST_VARIABLE}=commit 或 =user-request"
                       f"{below_text}：{command_for_display(table[pid]['command'])[:140]}；归属：{owners.get(pid, '认不出')}；"
                       f"上层：{' ← '.join(upper) if upper else '直接挂在 Claude 实例底下'}",
                       f"主 agent 判断：是该跑的（提交流程里、用户要求的）只是前缀漏了，还是子 agent 越界、或绕过了执行前的闸（写进脚本、python、make 起的）；"
                       f"要停就从最底层往上逐个停整棵子进程，按这个次序逐个跑 `{PROCESS_STOP_COMMAND} <进程号>`：{shown}；"
                       f"或者给归属的那个子 agent 发消息让它自己停；判定接着跑就 --ack 进程:{pid}"))
    summary = (f"{HEAVY_TEST_ALERT}：查了这个 Claude 实例底下 {examined} 个进程（其中 bash -c 那一层 {shell_layers} 个不单判），"
               f"认出重型 {len(flagged)} 个、带前缀的 {len(prefixed)} 个，报 {len(alerts)} 条")
    return [summary, *judgement_errors], alerts


def resident_memory_bytes(pid):
    """/proc/<pid>/statm 第二列（常驻页数）乘页大小；读不到（刚退出、权限不够）交 None。"""
    try:
        with open(f"/proc/{pid}/statm") as handle:
            return int(handle.read().split()[1]) * PAGE_SIZE_BYTES
    except (OSError, ValueError, IndexError):
        return None


def memory_report(root_pid, excluded_pids, session_dirs, thresholds, attribute_owners=True):
    """这个 Claude 实例底下常驻内存超过 --process-rss-gibibytes 的进程（判法见文件头）：返回 (报告行, [(告警名, 说明, 下一步)])。
    attribute_owners 为 False 时不去认归属（两次整查之间的单查只要知道有没有过线，归属留给随后那次整查）。"""
    threshold_gibibytes = thresholds.process_rss_gibibytes
    if threshold_gibibytes <= 0:
        return [f"{MEMORY_ALERT}：--process-rss-gibibytes 是 0，这一类不查"], []
    if BROKEN_DETECTION == "memoryscan":
        return [], []
    threshold_bytes = threshold_gibibytes * (1024 ** 2 if BROKEN_DETECTION == "memoryunit" else BYTES_PER_GIBIBYTE)
    table = process_table()
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    examined, largest, over = 0, None, []   # over：[(进程号, 常驻字节)]
    frontier = [root_pid]
    while frontier:
        for child in sorted(children.get(frontier.pop(), [])):
            arguments = process_arguments(child)
            if arguments and is_watchdog_process(arguments):
                continue   # 别的看门狗与它的自检：整棵不看
            frontier.append(child)
            if child in excluded_pids or table[child]["state"] in EXITED_PROCESS_STATES:
                continue
            resident = resident_memory_bytes(child)
            if resident is None:
                continue
            examined += 1
            if largest is None or resident > largest[1]:
                largest = (child, resident)
            if resident > threshold_bytes:
                over.append((child, resident))
    owners = owners_by_background_output([pid for pid, _ in over], table, session_dirs, root_pid) if (over and attribute_owners) else {}
    ticks_per_second = os.sysconf("SC_CLK_TCK")
    uptime_seconds = float(open("/proc/uptime").read().split()[0])
    alerts = []
    for pid, resident in over:
        alerts.append((MEMORY_ALERT,
                       f"进程 {pid} 常驻内存 {resident / BYTES_PER_GIBIBYTE:.2f} GiB（阈值 {threshold_gibibytes:g} GiB），"
                       f"已跑 {format_duration(uptime_seconds - table[pid]['start_ticks'] / ticks_per_second)}：{command_for_display(table[pid]['command'])[:140]}；"
                       f"归属：{owners.get(pid, '认不出') if attribute_owners else '（单查不认归属）'}",
                       f"先看它还涨不涨（隔几秒再读一次 `grep VmRSS /proc/{pid}/status`）：还在涨多半是无界分配，按进程号停它（`{PROCESS_STOP_COMMAND} {pid}`）"
                       f"或给归属的那个子 agent 发消息让它停；查清为什么涨之前别照原样重派同一件活，重派时让那一步经 research/scripts/run-with-memory-cap.sh 带上限跑；"
                       f"确实要用这么多就 --ack 进程:{pid}"))
    largest_text = (f"常驻内存最大的是进程 {largest[0]} {largest[1] / BYTES_PER_GIBIBYTE:.2f} GiB" if largest else "一个都没读到")
    summary = (f"{MEMORY_ALERT}：查了这个 Claude 实例底下 {examined} 个进程，{largest_text}（阈值 {threshold_gibibytes:g} GiB），报 {len(alerts)} 条")
    return [summary], alerts


def memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
    """两次整查之间单查一遍常驻内存：有没被 --ack 确认过的过线进程就返回 True，调用方马上整查一次、叫醒主 agent。"""
    if process_root_pid is None or BROKEN_DETECTION == "memorypoll":
        return False
    _, found = memory_report(process_root_pid, excluded_pids, set(), arguments, attribute_owners=False)
    remaining, _ = split_acknowledged([("进程", name, explanation, next_step) for name, explanation, next_step in found],
                                      set(arguments.ack or []))
    return bool(remaining)


BASH_TOOL_WRAPPER_COMMAND = re.compile(r"&& eval '(.*)'(?: < /dev/null)? && pwd -P >\| \S+\s*$", re.S)


def command_for_display(command):
    """Claude 的 Bash 工具把命令包在「source 快照 && … && eval '<命令>' [< /dev/null] && pwd -P >| …」里：报告里只列 eval 里那一段
    （还原 eval 里转义过的单引号、换行并成空格），不然最上层每一行都是同一串包装。"""
    if BROKEN_DETECTION == "display":
        return command
    wrapped = BASH_TOOL_WRAPPER_COMMAND.search(command)
    shown = wrapped.group(1).replace("'\"'\"'", "'") if wrapped else command
    return " ".join(shown.split())


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
    """--processes-only：起的时候在跑的那些都退出了退 0，进程一节报了告警退 3，到定时回报退 4；report 只看一眼，有告警退 3、没有退 0。"""
    watched = processes_to_watch(process_root_pid, excluded_pids)
    report_deadline = time.time() + arguments.max_minutes * 60
    while True:
        running = still_running(watched)
        topmost_running_pids = sorted(pid for pid, info in running.items() if info["parent"] not in running)
        lines = [f"只盯进程：进程 {process_root_pid} 底下看门狗起的时候就在跑的 {len(watched)} 个进程，还没退出的 {len(running)} 个"
                 + ("，最上层的：" if topmost_running_pids else "")]
        lines += [f"  进程 {pid}：{command_for_display(running[pid]['command'])[:140]}" for pid in topmost_running_pids]
        process_lines, found = process_alerts(process_root_pid, excluded_pids, arguments)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        heavy_lines, heavy_found = heavy_test_report(process_root_pid, excluded_pids,
                                                     {os.path.normpath(arguments.session_dir)} if arguments.session_dir else set())
        lines.extend(heavy_lines)
        memory_lines, memory_found = memory_report(process_root_pid, excluded_pids,
                                                   {os.path.normpath(arguments.session_dir)} if arguments.session_dir else set(), arguments)
        lines.extend(memory_lines)
        found = found + heavy_found + memory_found
        if arguments.session_dir:
            leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(
                {os.path.normpath(arguments.session_dir)}, datetime.now(timezone.utc), arguments)
        else:
            leftover_lines, leftover_alerts, leftover_pending_until = (
                ["交回、被停或失败之后后台还在跑：没给 --session-dir，这一类没查（watch.sh --processes 从 CLAUDE_CODE_SESSION_ID 找会话目录传进来）"], [], None)
        lines.extend(leftover_lines)
        alerts, acknowledged = split_acknowledged([("进程", name, explanation, next_step) for name, explanation, next_step in found]
                                                  + leftover_alerts, set(arguments.ack or []))
        for subject, name, explanation, _ in acknowledged:
            lines.append(f"已确认接着盯（--ack {subject}:{name}）：{explanation}")
        if arguments.mode == "report" or alerts:
            print_report(lines, alerts)
            return 3 if alerts else 0
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
            print(f"定时回报：已经看了 {arguments.max_minutes:g} 分钟，没有告警，盯着的进程还有 {len(running)} 个没退出。")
            print("   → 接着盯就再起一个看门狗 --processes-only")
            return 4
        next_check_in = min(arguments.interval_seconds, report_deadline - time.time())
        if leftover_pending_until is not None:
            next_check_in = min(next_check_in, leftover_pending_until.timestamp() + 1 - time.time())
        # 两次整查之间每隔 --detection-poll-seconds 单查一遍常驻内存，过线就提前整查（整查那一次报告警、退 3）
        next_full_check = time.time() + max(0.0, next_check_in)
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(0.0, next_full_check - time.time())))
            if memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
                break


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
    # 被看的子 agent 所在的会话：交回之后丢下的后台进程查这些会话里全部子 agent，重型测试的归属也在这些会话的记录里找
    session_dirs = {os.path.normpath(session_dir)} if session_dir else set()
    session_dirs |= {os.path.dirname(os.path.dirname(path)) for _, path in targets if path}
    if process_root_pid is not None:
        process_lines, found = process_alerts(process_root_pid, excluded_pids, thresholds)
        if process_lines:
            lines.append("这个 Claude 实例底下跑得久的进程：")
            lines.extend(process_lines)
        heavy_lines, heavy_found = heavy_test_report(process_root_pid, excluded_pids, session_dirs)
        lines.extend(heavy_lines)
        memory_lines, memory_found = memory_report(process_root_pid, excluded_pids, session_dirs, thresholds)
        lines.extend(memory_lines)
        for name, explanation, next_step in found + heavy_found + memory_found:
            alerts.append(("进程", name, explanation, next_step))
    leftover_pending_until = None
    if session_dirs:
        leftover_lines, leftover_alerts, leftover_pending_until = leftover_background_report(session_dirs, now, thresholds)
        lines.extend(leftover_lines)
        alerts.extend(leftover_alerts)
    else:
        lines.append("交回、被停或失败之后后台还在跑：被看的子 agent 都还没有会话记录，不知道是哪个会话，这一类没查")
    return lines, alerts, bool(done_flags) and all(done_flags), (active_ids, unstarted_ids), leftover_pending_until


def session_id_of(transcript_path):
    return os.path.basename(os.path.dirname(os.path.dirname(transcript_path)))


def is_scribe_followup(entry):
    """kb-scribe-followup.sh 写的「书记官写入之后相关记录没跟上」：出自 kb-scribe，findings 每条都是这一类。"""
    findings = [str(finding) for finding in (entry.get("findings") or [])]
    if BROKEN_DETECTION == "scribeallfindings":
        return entry.get("agent_type") == SCRIBE_AGENT_TYPE
    return (entry.get("agent_type") == SCRIBE_AGENT_TYPE and bool(findings)
            and all(finding.startswith(SCRIBE_FOLLOWUP_FINDING_PREFIX) for finding in findings))


def read_detections(detections_path, start_offset, session_ids, watched_agent_ids=(), alert_from_offset=None):
    """从 start_offset 往后读检出记录，返回 (新的偏移, [属于这些会话、被看的子 agent 或主 agent 的告警], [只进报告的记录], [书记员写到一半的检出])。
    书记员写到一半的检出（is_scribe_followup）不进告警，交给调用方按书记员攒着（scribe_followup_report）。
    给了 alert_from_offset 时，它之前那一段只捡书记员写到一半、认得出是哪个书记员的检出：看门狗起的时候重读一遍，接上一个看门狗退出时还没报的；
    别的类别在那一段里上一个看门狗已经报过，不再报。"""
    alerts, notes, scribe_entries = [], [], []
    if not os.path.isfile(detections_path):
        return start_offset, alerts, notes, scribe_entries
    position = start_offset
    with open(detections_path, "rb") as handle:
        handle.seek(start_offset)
        for raw_line in handle:
            line_start = position
            position += len(raw_line)
            in_history = alert_from_offset is not None and line_start < alert_from_offset
            try:
                entry = json.loads(raw_line.decode("utf-8", "replace"))
            except ValueError:
                continue
            if not isinstance(entry, dict):
                continue
            if BROKEN_DETECTION == "detections" or entry.get("session_id") not in session_ids:
                continue
            if (watched_agent_ids and entry.get("agent_id") and entry.get("agent_id") not in watched_agent_ids
                    and BROKEN_DETECTION != "otheragent"):
                continue   # 同一会话里别的子 agent 的检出归盯它的那个看门狗（2026-09-19：盯攻方腿的看门狗被上游那个 agent 的检出叫醒）
            if is_scribe_followup(entry) and BROKEN_DETECTION != "scribedefer":
                if not in_history or entry.get("agent_id"):
                    scribe_entries.append(entry)
                continue
            if in_history:
                continue
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
    return position, alerts, notes, scribe_entries


def add_scribe_followups(pool, entries):
    """书记员写到一半的检出按 (会话 id, 书记员 agent 号或 None) 攒进 pool。"""
    for entry in entries:
        pool.setdefault((entry.get("session_id"), entry.get("agent_id") or None), []).append(entry)


def session_directory_for(session_id, known_session_dirs, entries):
    """检出记录里的会话 id → 会话目录：先找看门狗已知的，再用记录里的 transcript_path（<会话目录>.jsonl），最后在 PROJECTS_ROOT 下找。"""
    for directory in sorted(known_session_dirs):
        if os.path.basename(os.path.normpath(directory)) == session_id:
            return os.path.normpath(directory)
    for entry in entries:
        transcript_path = entry.get("transcript_path") or ""
        if transcript_path.endswith(".jsonl") and os.path.isdir(transcript_path[: -len(".jsonl")]):
            return transcript_path[: -len(".jsonl")]
    matches = sorted(glob.glob(os.path.join(PROJECTS_ROOT, "*", session_id))) if session_id else []
    return matches[0] if matches else None


def running_scribes(session_dir):
    """这个会话里还没交回、没被停、没失败的书记员的 agent 号。"""
    if not session_dir:
        return []
    running = []
    notifications = session_task_events(os.path.normpath(session_dir) + ".jsonl")[1]
    for transcript_path in sorted(glob.glob(os.path.join(session_dir, "subagents", "agent-*.jsonl"))):
        if agent_meta(transcript_path)[0] != SCRIBE_AGENT_TYPE:
            continue
        agent_id = os.path.basename(transcript_path)[len("agent-"):-len(".jsonl")]
        if AgentTranscript(agent_id, transcript_path).ended(notifications) is None:
            running.append(agent_id)
    return running


def scribe_followup_summary(entries):
    """一个书记员名下攒下的检出：几次、按门禁分各几次与最后一次几点（JST）、涉及哪些文件。"""
    tokyo = ZoneInfo("Asia/Tokyo")
    by_stage, files = {}, []
    for entry in entries:
        try:
            moment = datetime.fromisoformat(str(entry.get("time"))).astimezone(tokyo).strftime("%H:%M")
        except ValueError:
            moment = "?"
        for finding in entry.get("findings") or []:
            stage = str(finding).split("：", 1)[1].strip() if "：" in str(finding) else str(finding)
            count, _ = by_stage.get(stage, (0, None))
            by_stage[stage] = (count + 1, moment)
        for path in (entry.get("command") or "").split()[1:]:   # 写法是「工具名 路径 路径 …」
            if path not in files:
                files.append(path)
    stage_text = "、".join(f"{stage}（{count} 次，最后一次 {moment} JST）" for stage, (count, moment) in by_stage.items())
    files_text = "、".join(files[:SCRIBE_FOLLOWUP_FILES_SHOWN]) + (f" 等 {len(files)} 份" if len(files) > SCRIBE_FOLLOWUP_FILES_SHOWN else "")
    return (f"写的过程中 hook 记了 {len(entries)} 次「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」，按门禁分：{stage_text or '记录里没写'}；"
            f"涉及的文件：{files_text or '记录里没写'}")


def scribe_followup_report(pool, known_session_dirs):
    """攒下的书记员写到一半的检出：返回 (报告行, [(主体, 告警名, 说明, 下一步)])。判法见文件头「书记员攒下的没跟上检出」。
    认得出是哪个书记员的：它交回、被停或失败了报一条汇总，还在跑只记一行；认不出的：那个会话里还有书记员在跑就只记一行，一个都不在跑了才报。"""
    lines, alerts, unattributed = [], [], {}
    tokyo = ZoneInfo("Asia/Tokyo")
    for (session_id, agent_id), entries in sorted(pool.items(), key=lambda item: (str(item[0][0]), str(item[0][1]))):
        session_dir = session_directory_for(session_id, known_session_dirs, entries)
        transcript_path = os.path.join(session_dir, "subagents", f"agent-{agent_id}.jsonl") if session_dir and agent_id else None
        if transcript_path is None or not os.path.isfile(transcript_path):
            unattributed.setdefault(session_id, []).extend(entries)
            continue
        transcript = AgentTranscript(agent_id, transcript_path)
        ended = transcript.ended(session_task_events(transcript.parent_transcript_path)[1])
        if ended is None or BROKEN_DETECTION == "scribesummary":
            lines.append(f"书记员 {agent_id}「{transcript.description}」还在写，先攒着不叫醒：{scribe_followup_summary(entries)}；它交回、被停或失败时一起报")
            continue
        how_it_ended, ended_at = ended
        alerts.append((agent_id, SCRIBE_FOLLOWUP_ALERT,
                       f"书记员「{transcript.description}」{how_it_ended}（{ended_at.astimezone(tokyo).strftime('%H:%M')} JST）；"
                       f"{scribe_followup_summary(entries)}（看过了、不用再报就 --ack {agent_id}:{SCRIBE_FOLLOWUP_ALERT}）",
                       SCRIBE_FOLLOWUP_NEXT_STEP))
    for session_id, entries in sorted(unattributed.items(), key=lambda item: str(item[0])):
        still_running = running_scribes(session_directory_for(session_id, known_session_dirs, entries))
        if still_running and BROKEN_DETECTION != "scribeanyrunning":
            lines.append(f"认不出是哪个书记员的「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」检出 {len(entries)} 条，会话里还有书记员在跑（{', '.join(still_running)}），"
                         f"先攒着，最后一个交回、被停或失败时一起报：{scribe_followup_summary(entries)}")
            continue
        alerts.append((f"{SCRIBE_AGENT_TYPE}（认不出是哪一个）", SCRIBE_FOLLOWUP_ALERT,
                       f"会话 {session_id} 里的书记员都已交回、被停或失败；{scribe_followup_summary(entries)}",
                       SCRIBE_FOLLOWUP_NEXT_STEP + "；记录里没有 agent 号，按涉及的文件与时刻对是哪一个书记员的报告"))
    return lines, alerts


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
        if subject == "进程" or (name == LEFTOVER_BACKGROUND_ALERT and BROKEN_DETECTION != "leftoverack"):
            process_id = re.match(r"进程 (\d+) ", explanation)
            if process_id:
                keys.add(f"进程:{process_id.group(1)}")
        # 「跑满 N 小时要主 agent 问一次」也认短写「<agent 号>:跑满 N 小时」：只确认那一格，下一格告警名不同，照报
        if name.startswith(ROUTINE_INQUIRY_ALERT_PREFIX) and name.endswith(ROUTINE_INQUIRY_ALERT_SUFFIX) and BROKEN_DETECTION != "inquiryack":
            keys.add(f"{subject}:{name[:-len(ROUTINE_INQUIRY_ALERT_SUFFIX)]}")
        (acknowledged if keys & acknowledgements else remaining).append(alert)
    return remaining, acknowledged


def run(arguments):
    if arguments.processes_only and (arguments.agents or arguments.mode == "cost" or arguments.process_root_pid == 0):
        print("✗ --processes-only 只盯进程，不和 --agents、cost、--process-root-pid 0 一起用\n"
              "→ 怎么办：盯子 agent 就去掉 --processes-only（子 agent 起的进程照样在进程一节里）；只盯主 agent 自己放后台的长命令就只给 --processes-only"
              "（可带 --session-dir，同时查交回之后后台还在跑）",
              file=sys.stderr)
        return 2
    if not arguments.agents and not arguments.session_dir and not arguments.processes_only:
        print("✗ 要给 --agents 或 --session-dir\n→ 怎么办：派发返回的 agent id 用逗号连起来传 --agents；只盯主 agent 自己放后台的长命令用 --processes-only",
              file=sys.stderr)
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
    if arguments.processes_only:
        if process_root_pid is None:
            print("✗ 往上找不到跑这个看门狗的 Claude 实例，--processes-only 没有进程可盯\n"
                  "→ 怎么办：在 Claude 的 Bash 工具里起它，或者用 --process-root-pid 指定从哪个进程往下看", file=sys.stderr)
            return 2
        return run_processes_only(arguments, process_root_pid, excluded_pids)
    if arguments.mode == "cost":
        lines, _ = cost_report(agent_ids, arguments.session_dir, arguments)
        print("\n".join(lines))
        return 0
    watch_started = time.time()
    detections_path = arguments.detections_file
    detections_offset = os.path.getsize(detections_path) if (arguments.mode == "watch" and os.path.isfile(detections_path)) else 0
    report_deadline = watch_started + arguments.max_minutes * 60
    detection_notes = []
    scribe_followups = {}   # (会话 id, 书记员 agent 号或 None) → 攒着的「书记官写入之后相关记录没跟上」检出
    # 第一次读检出记录时，被看的书记员此前的这一类检出从文件开头重读（上一个看门狗退出时还没报的接得上）；只给 --session-dir 时不重读，免得把早已交回的书记员翻出来再报
    history_start = 0 if (agent_ids and BROKEN_DETECTION != "scriberescan") else detections_offset
    while True:
        lines, alerts, all_done, active_lists, leftover_pending_until = build_report(
            agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
        watched_transcripts = [path for path in (find_transcript(agent_id, arguments.session_dir) for agent_id in agent_ids) if path]
        session_ids = {session_id_of(path) for path in watched_transcripts}
        known_session_dirs = {os.path.dirname(os.path.dirname(path)) for path in watched_transcripts}
        if arguments.session_dir:
            session_ids.add(os.path.basename(os.path.normpath(arguments.session_dir)))
            known_session_dirs.add(os.path.normpath(arguments.session_dir))
        if history_start is not None:
            detections_offset, detection_alerts, new_notes, new_scribe_entries = read_detections(
                detections_path, history_start, session_ids, set(agent_ids), alert_from_offset=detections_offset)
            history_start = None
        else:
            detections_offset, detection_alerts, new_notes, new_scribe_entries = read_detections(
                detections_path, detections_offset, session_ids, set(agent_ids))
        add_scribe_followups(scribe_followups, new_scribe_entries)
        detection_notes += new_notes
        scribe_lines, scribe_alerts = scribe_followup_report(scribe_followups, known_session_dirs)
        lines += detection_notes + scribe_lines
        alerts += detection_alerts + scribe_alerts
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
        # 全部交回照旧退出，这一次 build_report 已经把「交回之后后台还在跑」查过了；只剩宽限里的，等宽限过了再查一次才退
        if all_done and (leftover_pending_until is None or BROKEN_DETECTION == "leftoverexit"):
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
        if all_done:
            next_full_check = min(next_full_check, leftover_pending_until.timestamp() + 1)
        while time.time() < next_full_check:
            time.sleep(min(arguments.detection_poll_seconds, max(next_full_check - time.time(), 0)))
            detections_offset, detection_alerts, new_notes, new_scribe_entries = read_detections(
                detections_path, detections_offset, session_ids, set(agent_ids))
            add_scribe_followups(scribe_followups, new_scribe_entries)   # 书记员写到一半的不叫醒：攒着，下一次整查时看它交没交回
            detection_notes += new_notes
            # 常驻内存过线等不到下一次整查：单查到了就同检出一样马上整查一次（整查的报告里带着内存一节），有没确认的告警就退 3
            if detection_alerts or memory_alarm_between_checks(process_root_pid, excluded_pids, arguments):
                lines, alerts, _, active_lists, _ = build_report(agent_ids, arguments.session_dir, arguments, watch_started, excluded_pids, process_root_pid)
                # 这一次退出之后攒着的就没人接了（书记员已交回的不在下一个看门狗的名单里）：交回了的这里一起报，还在写的写进报告
                scribe_lines, scribe_alerts = scribe_followup_report(scribe_followups, known_session_dirs)
                alerts, acknowledged = split_acknowledged(alerts + detection_alerts + scribe_alerts, set(arguments.ack or []))
                if alerts:
                    lines = lines + detection_notes + scribe_lines + [f"已确认接着盯（--ack {subject}:{name}）：{explanation}"
                                                                      for subject, name, explanation, _ in acknowledged]
                    print_report(lines, alerts)
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
                             "撞限额失败（最近一次任务通知是 failed）")
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
            watch_runs.append(["leftover", agent_ids, *extra_arguments])
            return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--agents", agent_ids, "--session-dir", session_directory,
                                   "--process-root-pid", "0", "--interval-seconds", "1", "--max-minutes", "1", *extra_arguments],
                                  capture_output=True, text=True, timeout=timeout)

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
        processes_run = subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                                        "--session-dir", leftover_session, "--interval-seconds", "1", "--max-minutes", "0.5"],
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


def selftest_pipe_upstream(work, thresholds, failures, probed_processes):
    """管道尾在等上游：上游在算（只涨 CPU、不写管道）的不报「进程无输出」、报一行「在等上游」；上游睡着的、写端只在自己祖先手里的照报。
    管道尾的输出文件一小时没动，形状照 2026-09-24 被误报的 `cargo test … 2>&1 | tail -30`。返回查了几个管道。"""
    pipelines = {}
    for label, upstream_command in (("busy", [sys.executable, "-c", "import time\ndeadline = time.time() + 60\nwhile time.time() < deadline:\n    pass\n"]),
                                    ("sleeping", ["sleep", "60"])):
        tail_output = os.path.join(work, f"pipe-{label}-tail.log")
        with open(tail_output, "a") as tail_handle:
            os.utime(tail_output, (time.time() - 3600, time.time() - 3600))
            # stderr 不继承自检进程的：继承来的若是一份正被写着的日志，会被算成上游或管道尾「写着的文件在涨」
            upstream = subprocess.Popen(upstream_command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
            reader = subprocess.Popen(["tail", "-1"], stdin=upstream.stdout, stdout=tail_handle, stderr=subprocess.DEVNULL)
        upstream.stdout.close()   # 自检进程不留管道的读端：管道两端只在上游与管道尾手里，同 shell 起的管道
        pipelines[label] = (upstream, reader)
        probed_processes += [upstream, reader]
    # 写端是自己的祖先（自检进程拿着它 stdin 的写端，而自检进程的子孙里就有上面那个在算的上游）：祖先不算上游，照报
    ancestor_fed_output = os.path.join(work, "pipe-ancestor-fed-tail.log")
    with open(ancestor_fed_output, "a") as tail_handle:
        os.utime(ancestor_fed_output, (time.time() - 3600, time.time() - 3600))
        ancestor_fed = subprocess.Popen(["tail", "-1"], stdin=subprocess.PIPE, stdout=tail_handle, stderr=subprocess.DEVNULL)
    probed_processes.append(ancestor_fed)
    try:
        time.sleep(4)
        process_lines, found = process_alerts(os.getpid(), set(), thresholds)
        busy_upstream, busy_reader = pipelines["busy"]
        sleeping_upstream, sleeping_reader = pipelines["sleeping"]
        if any(f"进程 {busy_reader.pid} " in explanation for _, explanation, _ in found):
            failures.append(f"stdin 是管道、上游 {busy_upstream.pid} 一直在算的管道尾 {busy_reader.pid} 被当成告警（它在等上游）："
                            f"{[explanation for _, explanation, _ in found]}")
        if not any(f"进程 {busy_reader.pid} 在等上游" in line and f"写端进程 {busy_upstream.pid}" in line for line in process_lines):
            failures.append(f"上游 {busy_upstream.pid} 在算的管道尾 {busy_reader.pid} 在报告里没有「在等上游」那一行并点名写端，实际 {process_lines}")
        if not any(name == "进程无输出" and f"进程 {sleeping_reader.pid} " in explanation for name, explanation, _ in found):
            failures.append(f"stdin 是管道、上游 {sleeping_upstream.pid} 睡着（不算也不写）的管道尾 {sleeping_reader.pid} 没报「进程无输出」："
                            f"{[explanation for _, explanation, _ in found] or '一条告警都没有'}")
        if not any(name == "进程无输出" and f"进程 {ancestor_fed.pid} " in explanation for name, explanation, _ in found):
            failures.append(f"stdin 管道的写端只在自己的祖先（自检进程 {os.getpid()}）手里的 {ancestor_fed.pid} 没报「进程无输出」："
                            f"祖先的子孙里有不相干的在算的进程，不能拿来当上游")
    finally:
        for process in [process for pair in pipelines.values() for process in pair] + [ancestor_fed]:
            process.kill()
            process.wait()
        ancestor_fed.stdin.close()
    return len(pipelines) + 1


def selftest_heavy_tests(work, failures, probed_processes, watch_runs):
    """「重型测试没带前缀」的自检：拿自检进程当 Claude 实例，起几个假的重型进程（名字照 cargo 编出来的层 0 测试二进制，里面只是睡）。
    没带前缀的、带别的值的报，带 commit / user-request 的不报；bash -c 里写着带前缀的重型命令、那条命令还没起的不报；
    timeout 包着带前缀的不报，timeout 包着没带前缀的只报 timeout 那一个、下一步给整棵子树从最底层往上的停止次序；
    后台起、开着一个子 agent 后台任务输出文件的，归属报出那个子 agent。返回起了几个假进程。"""
    deps = os.path.join(work, "heavy", "target", "release", "deps")
    os.makedirs(deps, exist_ok=True)
    fake_binary = os.path.join(deps, "first_transaction_step_seven_layer0-0123456789abcdef")   # python 顶着这个 argv[0] 睡，不用真的编
    fake_script = os.path.join(deps, "second_stream_layer0-fedcba9876543210")                    # sh 脚本：timeout、bash -c 要真的 exec 它
    with open(fake_script, "w") as handle:
        handle.write("#!/bin/sh\nsleep 60\nexit 0\n")
    os.chmod(fake_script, 0o755)
    # 另一个看门狗的自检（python 起的 agent-watch.py）底下起的假重型进程：那一支整棵不扫
    fake_watchdog = os.path.join(work, "other-watchdog", "agent-watch.py")
    os.makedirs(os.path.dirname(fake_watchdog), exist_ok=True)
    with open(fake_watchdog, "w") as handle:
        handle.write("import subprocess, sys, time\n"
                     "subprocess.Popen([sys.argv[1], '-c', 'import time\\ntime.sleep(60)\\n'], executable=sys.executable)\n"
                     "time.sleep(60)\n")
    heavy_session = os.path.join(work, "heavy-session")
    owned_output = os.path.join(work, "heavyowner.output")
    write_transcript(heavy_session, "heavyowner", [
        bash_use(1, "t1", f"{fake_binary} --test-threads 4", "m1"),
        record_at(0.95, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                  "content": f"Command running in background with ID: bh1. Output is being written to: {owned_output}. "
                                             "You will be notified when it completes."}]),
        record_at(0.9, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
        {"agentType": "implementation-writer", "description": "后台起了一个没带前缀的层 0 测试二进制"})
    clean_environment = {key: value for key, value in os.environ.items() if key != HEAVY_TEST_VARIABLE}
    started = {}

    def start(label, argv, occasion=None, stdout=subprocess.DEVNULL, executable=None):
        environment = dict(clean_environment, **({HEAVY_TEST_VARIABLE: occasion} if occasion else {}))
        started[label] = subprocess.Popen(argv, executable=executable, env=environment, cwd=work, stdout=stdout, stderr=subprocess.DEVNULL)
        probed_processes.append(started[label])

    sleeping = ["-c", "import time\ntime.sleep(60)\n"]
    try:
        with open(owned_output, "a") as output_handle:
            start("没带前缀", [fake_binary, *sleeping], stdout=output_handle, executable=sys.executable)
        start("带 commit", [fake_binary, *sleeping], "commit", executable=sys.executable)
        start("带 user-request", [fake_binary, *sleeping], "user-request", executable=sys.executable)
        start("带别的值", [fake_binary, *sleeping], "yes", executable=sys.executable)
        start("bash -c 里带前缀、还没起", ["bash", "-c", f"sleep 60; {HEAVY_TEST_VARIABLE}=commit {fake_script}"])
        start("timeout 包着带前缀的", ["timeout", "60", "env", f"{HEAVY_TEST_VARIABLE}=commit", fake_script])
        start("timeout 包着没带前缀的", ["timeout", "60", fake_script])
        start("另一个看门狗的自检底下没带前缀的", [sys.executable, fake_watchdog, fake_binary])
        time.sleep(1.5)   # 等 timeout 起子进程、sh 起 sleep、假看门狗起它的假重型进程
        table = process_table()

        def children_of(pid):
            return sorted(child for child, info in table.items() if info["parent"] == pid)

        timeout_pid = started["timeout 包着没带前缀的"].pid
        script_pid = (children_of(timeout_pid) or [None])[0]
        sleep_pid = (children_of(script_pid) or [None])[0] if script_pid else None
        lines, found = heavy_test_report(os.getpid(), set(), {heavy_session})
        alerted = {}
        for name, explanation, next_step in found:
            reported = re.match(r"进程 (\d+) ", explanation)
            if name == HEAVY_TEST_ALERT and reported:
                alerted[int(reported.group(1))] = (explanation, next_step)
        wanted = {started["没带前缀"].pid: "没带前缀", started["带别的值"].pid: "带别的值", timeout_pid: "timeout 包着没带前缀的"}
        if set(alerted) != set(wanted):
            failures.append(f"「{HEAVY_TEST_ALERT}」应当只报 {sorted(wanted.values())} 这 {len(wanted)} 个（进程号 {sorted(wanted)}），"
                            f"带 commit / user-request 的、bash -c 里带前缀还没起的、timeout 包着带前缀的、timeout 底下那个 sh、"
                            f"另一个看门狗的自检底下的都不报；"
                            f"实际报了 {sorted(alerted)}（{[started_label for started_label, process in started.items() if process.pid in alerted]}），报告：{lines}")
        unprefixed_explanation = alerted.get(started["没带前缀"].pid, ("", ""))[0]
        if not all(part in unprefixed_explanation for part in ("层 0", f"没带 {HEAVY_TEST_VARIABLE}=commit", "子 agent heavyowner", "heavyowner.output")):
            failures.append(f"没带前缀的层 0 进程的告警应当写类别「层 0」、「没带 {HEAVY_TEST_VARIABLE}=commit」、归属「子 agent heavyowner」与它开着的输出文件，"
                            f"实际：{unprefixed_explanation or '没报'}")
        other_value_explanation = alerted.get(started["带别的值"].pid, ("", ""))[0]
        if f"带的是 {HEAVY_TEST_VARIABLE}=yes" not in other_value_explanation or "认不出" not in other_value_explanation:
            failures.append(f"带 {HEAVY_TEST_VARIABLE}=yes 的告警应当写「带的是 {HEAVY_TEST_VARIABLE}=yes」、归属认不出，实际：{other_value_explanation or '没报'}")
        timeout_explanation, timeout_next_step = alerted.get(timeout_pid, ("", ""))
        wanted_order = f"{sleep_pid} {script_pid} {timeout_pid}"
        if ("底下另有 1 个同样没带前缀的重型进程" not in timeout_explanation or wanted_order not in timeout_next_step
                or PROCESS_STOP_COMMAND not in timeout_next_step or f"--ack 进程:{timeout_pid}" not in timeout_next_step):
            failures.append(f"timeout 包着没带前缀的只报 timeout 那一个：说明里写「底下另有 1 个」，下一步给从最底层往上的停止次序「{wanted_order}」、"
                            f"停的命令与 --ack 进程:{timeout_pid}；实际说明：{timeout_explanation or '没报'}，下一步：{timeout_next_step}")
        if not any(line.startswith(f"{HEAVY_TEST_ALERT}：查了") and f"报 {len(wanted)} 条" in line for line in lines):
            failures.append(f"报告里应当有一行「{HEAVY_TEST_ALERT}：查了 … 报 {len(wanted)} 条」，实际 {lines}")
        # 走看门狗：--processes-only 与 --agents 两条路都报、退 3；三个都 --ack 进程:<pid> 之后不叫醒，到点退 4
        wanted_alert_line = f"⚠️ {HEAVY_TEST_ALERT}（进程）：进程 {started['没带前缀'].pid} "
        base = [sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()), "--session-dir", heavy_session,
                "--interval-seconds", "1", "--max-minutes", "0.1"]
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "heavyowner"])):
            watch_runs.append(["heavy", label])
            heavy_run = subprocess.run(base + extra, capture_output=True, text=True, timeout=120)
            if heavy_run.returncode != 3 or wanted_alert_line not in heavy_run.stdout or f"   → 怎么办：主 agent 判断：是该跑的" not in heavy_run.stdout:
                failures.append(f"看门狗 {label} 盯着没带前缀的层 0 进程时应当报「{wanted_alert_line}…」连同出路、退出码 3，"
                                f"实际退出码 {heavy_run.returncode}，输出：{heavy_run.stdout[-500:]}")
        watch_runs.append(["heavy", "--processes-only", "--ack"])
        acknowledged_run = subprocess.run(base + ["--processes-only"] + [argument for pid in wanted for argument in ("--ack", f"进程:{pid}")],
                                          capture_output=True, text=True, timeout=120)
        if acknowledged_run.returncode != 4 or "已确认接着盯（--ack 进程:" not in acknowledged_run.stdout:
            failures.append(f"「{HEAVY_TEST_ALERT}」的三个进程都 --ack 进程:<pid> 之后看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-500:]}")
    finally:
        # timeout 与 bash -c 起的子孙（sh、sleep）不随父进程一起死：停之前先记下这几棵子树的进程号，子孙先停
        table = process_table()
        subtree_pids = []
        for process in started.values():
            stack = [process.pid]
            while stack:
                pid = stack.pop()
                if pid in table:
                    subtree_pids.append(pid)
                    stack.extend(child for child, info in table.items() if info["parent"] == pid)
        for pid in reversed(subtree_pids):
            try:
                os.kill(pid, signal.SIGKILL)
            except OSError:
                pass
        for process in started.values():
            process.wait()
    return len(started)


def selftest_memory(work, failures, probed_processes, watch_runs):
    """「进程常驻内存过线」的自检：拿自检进程当 Claude 实例，阈值压到 0.125 GiB。
    一个写满 256 MiB 的进程报（归属认出开着输出文件的子 agent memowner）、一个只睡的小进程不报，阈值 0 不查；
    --processes-only 与 --agents 两条路整查都报、退 3，--ack 进程:<pid> 之后不叫醒、到点退 4；
    两次整查之间的单查：复检间隔 240 秒、看门狗起了之后才冒出来的过线进程，几秒之内就叫醒（不等到回报时刻那次整查）。返回起了几个进程。"""
    memory_session = os.path.join(work, "memory-session")
    owned_output = os.path.join(work, "memowner.output")
    write_transcript(memory_session, "memowner", [
        bash_use(1, "t1", "cargo test --release", "m1"),
        record_at(0.95, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                  "content": f"Command running in background with ID: bm1. Output is being written to: {owned_output}. "
                                             "You will be notified when it completes."}]),
        record_at(0.9, "assistant", [{"type": "text", "text": "等后台任务"}], stop_reason="end_turn", message_id="m2")],
        {"agentType": "experiment-runner", "description": "后台起了一个吃内存的测试"})
    thresholds = argparse.Namespace(process_rss_gibibytes=0.125)
    hog_code = "import time\nheld = b'\\x01' * (256 * 1024 * 1024)\nprint('ready', flush=True)\ntime.sleep(120)\n"
    started = []

    def start(argv, stdout=subprocess.DEVNULL):
        process = subprocess.Popen(argv, stdout=stdout, stderr=subprocess.DEVNULL)
        started.append(process)
        probed_processes.append(process)
        return process

    def wait_until_resident(process, minimum_bytes, timeout_seconds=20):
        deadline = time.time() + timeout_seconds
        while time.time() < deadline:
            resident = resident_memory_bytes(process.pid)
            if resident is not None and resident >= minimum_bytes:
                return True
            time.sleep(0.1)
        return False

    def alerted_pids(found):
        return {int(match.group(1)) for name, explanation, _ in found if name == MEMORY_ALERT
                for match in [re.match(r"进程 (\d+) ", explanation)] if match}

    try:
        with open(owned_output, "a") as output_handle:
            holder = start(["sleep", "120"], stdout=output_handle)   # 一直开着 memowner 那个后台任务的输出文件：它在等的任务还活着
            hog = start([sys.executable, "-c", hog_code], stdout=output_handle)
        small = start([sys.executable, "-c", "import time\ntime.sleep(120)\n"])
        if not wait_until_resident(hog, 200 * 1024 * 1024):
            failures.append(f"写满 256 MiB 的进程 {hog.pid} 20 秒里常驻内存没到 200 MiB，这一段自检没法判")
            return len(started)
        lines, found = memory_report(os.getpid(), set(), {memory_session}, thresholds)
        alerted = alerted_pids(found)
        if alerted != {hog.pid}:
            failures.append(f"「{MEMORY_ALERT}」应当只报写满 256 MiB 的进程 {hog.pid}（阈值 0.125 GiB），只睡的 {small.pid} 与 {holder.pid} 不报；"
                            f"实际报了 {sorted(alerted)}，报告：{lines}")
        hog_explanation, hog_next_step = next(((explanation, next_step) for name, explanation, next_step in found
                                               if explanation.startswith(f"进程 {hog.pid} ")), ("", ""))
        if not all(part in hog_explanation for part in ("常驻内存", "阈值 0.125 GiB", "已跑", "子 agent memowner", "memowner.output")):
            failures.append(f"过线进程的告警应当写常驻内存、阈值、已跑多久与归属「子 agent memowner」（开着 memowner.output），实际：{hog_explanation or '没报'}")
        if not all(part in hog_next_step for part in (f"grep VmRSS /proc/{hog.pid}/status", f"{PROCESS_STOP_COMMAND} {hog.pid}",
                                                     "别照原样重派同一件活", "run-with-memory-cap.sh", f"--ack 进程:{hog.pid}")):
            failures.append(f"过线进程的「怎么办」应当写怎么看还涨不涨、按进程号停它的命令、查清之前别照原样重派、带上限重派与 --ack 进程:{hog.pid}，实际：{hog_next_step}")
        if not any(line.startswith(f"{MEMORY_ALERT}：查了") and "报 1 条" in line and f"进程 {hog.pid} " in line for line in lines):
            failures.append(f"报告里应当有一行「{MEMORY_ALERT}：查了 … 常驻内存最大的是进程 {hog.pid} … 报 1 条」，实际 {lines}")
        disabled_lines, disabled_found = memory_report(os.getpid(), set(), {memory_session}, argparse.Namespace(process_rss_gibibytes=0))
        if disabled_found or not any("这一类不查" in line for line in disabled_lines):
            failures.append(f"--process-rss-gibibytes 0 时应当不报、报告里写「这一类不查」，实际告警 {disabled_found}、报告 {disabled_lines}")
        # 整查：--processes-only 与 --agents 两条路都报、退 3；--ack 进程:<pid> 之后不叫醒，到点退 4
        wanted_alert_line = f"⚠️ {MEMORY_ALERT}（进程）：进程 {hog.pid} "
        base = [sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()), "--session-dir", memory_session,
                "--process-rss-gibibytes", "0.125", "--interval-seconds", "1", "--max-minutes", "0.1"]
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "memowner"])):
            watch_runs.append(["memory", label])
            memory_run = subprocess.run(base + extra, capture_output=True, text=True, timeout=120)
            if memory_run.returncode != 3 or wanted_alert_line not in memory_run.stdout or "   → 怎么办：先看它还涨不涨" not in memory_run.stdout:
                failures.append(f"看门狗 {label} 盯着常驻内存过线的进程时应当报「{wanted_alert_line}…」连同出路、退出码 3，"
                                f"实际退出码 {memory_run.returncode}，输出：{memory_run.stdout[-500:]}")
        watch_runs.append(["memory", "--processes-only", "--ack"])
        acknowledged_run = subprocess.run(base + ["--processes-only", "--ack", f"进程:{hog.pid}"], capture_output=True, text=True, timeout=120)
        if acknowledged_run.returncode != 4 or f"已确认接着盯（--ack 进程:{MEMORY_ALERT}）：进程 {hog.pid} " not in acknowledged_run.stdout:
            failures.append(f"过线进程 --ack 进程:{hog.pid} 之后看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                            f"实际退出码 {acknowledged_run.returncode}，输出：{acknowledged_run.stdout[-500:]}")
        hog.kill()
        hog.wait()
        # 两次整查之间的单查：复检间隔 240 秒、回报时刻 30 秒，过线进程在看门狗起了 6 秒之后才冒出来（第一次整查里等后台任务那一节要取两秒 CPU 样，
        # 冒得早了会被第一次整查直接看见，判不出单查），应当几秒之内叫醒（单查坏了就要等到 30 秒那次整查）
        for label, extra in (("--processes-only", ["--processes-only"]), ("--agents", ["--agents", "memowner"])):
            watch_runs.append(["memory", label, "单查"])
            watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--process-root-pid", str(os.getpid()),
                                        "--session-dir", memory_session, "--process-rss-gibibytes", "0.125", "--interval-seconds", "240",
                                        "--detection-poll-seconds", "0.5", "--max-minutes", "0.5", *extra],
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            watch_started = time.time()
            time.sleep(6)
            late_hog = start([sys.executable, "-c", hog_code])
            output, _ = watcher.communicate(timeout=120)
            elapsed = time.time() - watch_started
            if watcher.returncode != 3 or f"⚠️ {MEMORY_ALERT}（进程）：进程 {late_hog.pid} " not in output or elapsed >= 20:
                failures.append(f"看门狗 {label} 起了之后才冒出来的过线进程 {late_hog.pid} 应当由两次整查之间的单查在几秒之内叫醒（退出码 3），"
                                f"实际退出码 {watcher.returncode}、{elapsed:.1f} 秒才退，输出：{output[-400:]}")
            late_hog.kill()
            late_hog.wait()
    finally:
        for process in started:
            process.kill()
            process.wait()
    return len(started)


def selftest_scribe_followups(work, failures, watch_runs):
    """「书记员攒下的没跟上检出」的自检：另起会话目录与检出记录。书记员还在写时这一类检出不叫醒、到点退 4；
    看门狗盯着的时候它交回，报一条汇总退 3；看门狗起之前攒下、它已交回的，起的时候重读出来报；书记员别的类别的检出照旧立刻叫醒；
    认不出是哪个书记员的，会话里还有书记员在跑就攒着，一个都不在跑了才报。返回查了几个书记员。"""
    session_id = "scribe-session"
    scribe_session = os.path.join(work, session_id)
    detections = os.path.join(work, "scribe-detections.jsonl")
    writing_records = [bash_use(0.2, "t1", "python3 research/scripts/replace-once.py .claude/kb/decisions/23-样本.md 旧 新", "m1")]

    def scribe_meta(description):
        return {"agentType": SCRIBE_AGENT_TYPE, "description": description}

    write_transcript(scribe_session, "scriberunning", writing_records, scribe_meta("一直在写"))
    write_transcript(scribe_session, "scribelater", writing_records, scribe_meta("看门狗盯着的时候交回"))
    handed_back_records = [bash_use(3, "t1", "ls", "m1"), bash_result(2.9, "t1"), *handback_records(1, "h1", "m2"),
                           record_at(0.9, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")]
    write_transcript(scribe_session, "scribedone", handed_back_records, scribe_meta("看门狗起之前就交回了"))

    def detection(agent_id, stage, paths, finding=None):
        return {"time": datetime.now(timezone.utc).isoformat(), "session_id": session_id, "agent_id": agent_id, "agent_type": SCRIBE_AGENT_TYPE,
                "transcript_path": scribe_session + ".jsonl", "command": f"Bash {' '.join(paths)}",
                "findings": [finding or f"{SCRIBE_FOLLOWUP_FINDING_PREFIX}：{stage}"]}

    def append_detection(entry):
        with open(detections, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")

    def run_scribe_watch(agents, max_minutes, later_steps=()):
        """起看门狗盯这几个书记员；later_steps 是 [(起之后等几秒, 要做的事, 说明)]，按次序做。返回 (退出码, 标准输出)。"""
        watch_runs.append(["scribe", agents, *[label for _, _, label in later_steps]])
        watcher = subprocess.Popen([sys.executable, os.path.abspath(__file__), "watch", "--agents", agents, "--session-dir", scribe_session,
                                    "--process-root-pid", "0", "--interval-seconds", "1", "--detection-poll-seconds", "0.3",
                                    "--max-minutes", str(max_minutes), "--detections-file", detections],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        for delay_seconds, action, _ in later_steps:
            time.sleep(delay_seconds)
            action()
        output, _ = watcher.communicate(timeout=120)
        return watcher.returncode, output

    open(detections, "w").close()
    # 看门狗起之前攒下的：scribedone 名下两道门禁各一条
    append_detection(detection("scribedone", "30-decision-history.sh", [".claude/kb/decisions/23-样本.md", ".claude/kb/decisions-history/2026-09.md"]))
    append_detection(detection("scribedone", "75-decision-experiment-links.sh", [".claude/kb/experiments/156-样本.md"]))
    summary_head = f"⚠️ {SCRIBE_FOLLOWUP_ALERT}（scribedone）："
    exit_code, output = run_scribe_watch("scribedone", 0.2)
    if (exit_code != 3 or summary_head not in output or "30-decision-history.sh（1 次" not in output
            or "75-decision-experiment-links.sh（1 次" not in output or ".claude/kb/decisions/23-样本.md" not in output
            or f"   → 怎么办：{SCRIBE_FOLLOWUP_NEXT_STEP}" not in output):
        failures.append(f"看门狗起之前攒下、书记员已交回的「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」应当在起的时候重读出来报一条汇总「{summary_head}…」"
                        f"（两道门禁各 1 次、涉及的文件、出路）并退出码 3，实际退出码 {exit_code}，输出：{output[-600:]}")
    # 书记员还在写：这一类检出不叫醒，攒着写进报告，到点退 4
    exit_code, output = run_scribe_watch("scriberunning", 0.1, [(1.0, lambda: append_detection(
        detection("scriberunning", "75-decision-experiment-links.sh", [".claude/kb/decisions/25-样本.md"])), "写到一半记一条")])
    if exit_code != 4 or "hook 检出" in output or "书记员 scriberunning「一直在写」还在写，先攒着不叫醒" not in output:
        failures.append(f"书记员还在写时「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」不该叫醒：应当到点退出码 4、报告里记一行「还在写，先攒着不叫醒」、没有「hook 检出」，"
                        f"实际退出码 {exit_code}，输出：{output[-600:]}")
    # 看门狗盯着的时候：写到一半记一条（不叫醒），接着它交回（叫醒一次，报汇总）
    def hand_back_later():
        write_transcript(scribe_session, "scribelater", [*writing_records, *handback_records(0.01, "h1", "m2")], scribe_meta("看门狗盯着的时候交回"))

    exit_code, output = run_scribe_watch("scribelater", 0.3, [
        (1.0, lambda: append_detection(detection("scribelater", "30-decision-history.sh", [".claude/kb/decisions/08-样本.md"])), "写到一半记一条"),
        (1.5, hand_back_later, "交回")])
    if exit_code != 3 or "hook 检出" in output or f"⚠️ {SCRIBE_FOLLOWUP_ALERT}（scribelater）：" not in output or "30-decision-history.sh（1 次" not in output:
        failures.append(f"书记员写到一半记的「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」不该当场叫醒、它交回时报一条汇总并退出码 3，"
                        f"实际退出码 {exit_code}，输出：{output[-600:]}")
    # 书记员别的类别的检出照旧立刻叫醒
    exit_code, output = run_scribe_watch("scriberunning", 0.2, [(1.0, lambda: append_detection(
        detection("scriberunning", "", [".claude/kb/decisions/25-样本.md"], finding="写被拒：越出写范围")), "写被拒")])
    if exit_code != 3 or "hook 检出" not in output or "写被拒：越出写范围" not in output:
        failures.append(f"书记员「写被拒」这类检出应当照旧立刻叫醒（退出码 3、报「hook 检出」），实际退出码 {exit_code}，输出：{output[-600:]}")
    # 认不出是哪个书记员的：会话里还有书记员在跑（scriberunning）就攒着；另一个会话里书记员都交回了就报
    unattributed = detection(None, "30-decision-history.sh", [".claude/kb/decisions/23-样本.md"])
    lines, found = scribe_followup_report({(session_id, None): [unattributed]}, {scribe_session})
    if found or not any("认不出是哪个书记员" in line and "scriberunning" in line for line in lines):
        failures.append(f"认不出是哪个书记员的检出，会话里还有书记员 scriberunning 在跑时应当攒着、报告里记一行，实际告警 {found}，报告 {lines}")
    ended_session = os.path.join(work, "scribe-ended-session")
    write_transcript(ended_session, "scribedone", handed_back_records, scribe_meta("已交回"))
    lines, found = scribe_followup_report({("scribe-ended-session", None): [dict(unattributed, session_id="scribe-ended-session")]}, {ended_session})
    if [(owner, name) for owner, name, _, _ in found] != [(f"{SCRIBE_AGENT_TYPE}（认不出是哪一个）", SCRIBE_FOLLOWUP_ALERT)]:
        failures.append(f"认不出是哪个书记员的检出，会话里书记员都交回了应当报一条「{SCRIBE_FOLLOWUP_ALERT}」，实际告警 {found}，报告 {lines}")
    return 3


def selftest():
    work = tempfile.mkdtemp(prefix="agent-watch-selftest-")
    thresholds = argparse.Namespace(tool_minutes=8, wait_loop_minutes=3, idle_minutes=10, repeat_count=3, interval_seconds=240,
                                    process_report_minutes=0.02, process_stale_minutes=0.05, process_max_minutes=600,
                                    not_started_minutes=5, active_minutes=600, context_tokens=600_000, leftover_background_grace_minutes=2,
                                    ask_every_minutes=60, process_rss_gibibytes=8)
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
    # 交回之后被自己后台任务的完成通知叫醒、看一眼输出再结束本轮：仍算交回，不许判成没交回、报「结束本轮却不会醒」
    # （2026-09-23 门禁审计会话 ac26abd494cda0968 实测误报，次序照它的会话记录：前台 doc-lint 超时转后台、交回、结束本轮、完成通知、grep 一次、结束本轮）。
    moved_task_started = record_at(4, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                "content": "Command did not complete within its 120s timeout and was moved to the background (ID: bl1). "
                                                           f"Output is being written to: {os.path.join(work, 'bl1.output')}. You will be notified when it completes."}])
    moved_task_finished = record_at(2.7, "user", "[SYSTEM NOTIFICATION - NOT USER INPUT]\n<task-notification>\n<task-id>bl1</task-id>\n<status>completed</status>\n</task-notification>",
                                    origin_kind="task-notification")
    write_transcript(session, "toolafterhandback", [bash_use(6, "t1", 'bash .claude/singlefs-ai-sop/scripts/doc-lint.sh "$PWD"', "m1"), moved_task_started,
                                                    *handback_records(3, "h1", "m2"),
                                                    record_at(2.8, "assistant", [{"type": "text", "text": "报告已交回主 agent。"}], stop_reason="end_turn", message_id="m3"),
                                                    moved_task_finished, bash_use(2.6, "t2", "grep -n '✗' bl1.output", "m4"), bash_result(2.5, "t2"),
                                                    record_at(2.4, "assistant", [{"type": "text", "text": "后台那次 doc-lint 跑完了，和交回里写的一致"}],
                                                              stop_reason="end_turn", message_id="m5")],
                     {"agentType": "general-purpose", "description": "交回之后被自己后台任务的完成通知叫醒、调了工具再结束本轮"})
    # 前台命令跑满超时被挪进后台之后结束本轮去等它：算在等自己的后台任务，不许报「结束本轮却不会醒」（同一份会话记录里的 doc-lint 就是这样转进后台的）。
    write_transcript(session, "movedbackground", [bash_use(4, "t1", "bash .claude/scripts/gate.sh --staged", "m1"),
                                                  record_at(2, "user", [{"type": "tool_result", "tool_use_id": "t1",
                                                                         "content": "Command did not complete within its 120s timeout and was moved to the background (ID: bm1). "
                                                                                    f"Output is being written to: {os.path.join(work, 'bm1.output')}. You will be notified when it completes."}]),
                                                  record_at(1.8, "assistant", [{"type": "text", "text": "等超时转进后台的那条跑完"}], stop_reason="end_turn", message_id="m2")],
                     {"agentType": "gate-triage", "description": "前台命令超时转进后台、结束本轮等它"})
    # 交回之后、它还在忙的时候来的续做消息只落成 queued_command 附件（附件自己带 origin coordinator），同样撤销交回；
    # 续做之后干完活、没再交回就结束本轮的，要报「结束本轮却不会醒」。
    queued_resume = {"timestamp": record_at(3.8, "user", "")["timestamp"], "type": "attachment",
                     "attachment": {"type": "queued_command", "prompt": "再补一格", "origin": {"kind": "coordinator"}, "isMeta": True}}
    write_transcript(session, "queuedresume", [*handback_records(4, "h1", "m1"), queued_resume, bash_use(3.7, "t2", "ls", "m2"), bash_result(3.6, "t2"),
                                               record_at(3.5, "assistant", [{"type": "text", "text": "补完了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "experiment-runner", "description": "交回之后忙时收到续做消息、干完没再交回就结束本轮"})
    write_transcript(session, "waitloop", [bash_use(5, "t1", 'until grep -q "^exit=" log; do sleep 10; done', "m1")])
    write_transcript(session, "longtool", [bash_use(20, "t1", "cargo test --release", "m1")])
    # 断网 / 被停砍掉的那一次调用：只有发起、永远没有结果，而它之后 agent 又接着动了。
    # 那条调用已经死了，不许再按「工具调用过长」报（2026-09-22 三条腿续做后实测假报）。
    write_transcript(session, "stalepending", [bash_use(20, "t1", "cargo test --release", "m1"),
                                              bash_use(0.5, "t2", "cargo build", "m2")])
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
    # 50 分钟前：在禁用命令的 10 分钟窗口之外，又不到「跑满 1 小时要主 agent 问一次」的第一格
    write_transcript(session, "bannedold", [bash_use(50, "t1", "pgrep -f e154-binary", "m1"), bash_result(49.9, "t1"),
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
    # 「跑满 N 小时要主 agent 问一次」：派发 61 分钟、没收到过消息的报 1 小时；派发 125 分钟的报 2 小时（--ack 了 1 小时那一格照报，见下面的看门狗）；
    # 交回的、任务通知 failed 的不报；中途（第一格之前）收到消息不重新计时、照报；第一格之后收到过消息的（送达的两种写法，
    # 与还在主会话记录里、没送达的 SendMessage）不报。每个都在最后留一条 30 秒前起的工具调用，免得别的告警混进来。
    for inquiry_agent_id, minutes_since_dispatch, inquiry_middle_records, inquiry_description in (
            ("inquiry61", 61, [], "派发 61 分钟、没收到过消息"),
            ("inquiry125", 125, [], "派发 125 分钟、没收到过消息"),
            ("inquirymessagebefore", 90, [record_at(40, "user", "主 agent：线程上限 4", origin_kind="coordinator")], "派发 90 分钟、第一格之前收到过消息"),
            ("inquirydeliveredafter", 90, [record_at(20, "user", "主 agent 例行询问", origin_kind="coordinator")], "派发 90 分钟、第一格之后收到过消息"),
            ("inquiryqueuedafter", 90, [{"timestamp": record_at(20, "user", "")["timestamp"], "type": "attachment",
                                         "attachment": {"type": "queued_command", "prompt": "主 agent 例行询问", "origin": {"kind": "coordinator"}, "isMeta": True}}],
             "派发 90 分钟、第一格之后忙时收到过消息（排进队）")):
        write_transcript(session, inquiry_agent_id, [bash_use(minutes_since_dispatch, "t1", "ls", "m1"), bash_result(minutes_since_dispatch - 0.1, "t1"),
                                                     *inquiry_middle_records, bash_use(0.5, "t2", "cargo build", "m2")],
                         {"agentType": "implementation-writer", "description": inquiry_description})
    write_transcript(session, "inquiryhandedback", [bash_use(90, "t1", "ls", "m1"), bash_result(89.9, "t1"), *handback_records(10, "h1", "m2"),
                                                    record_at(9.8, "assistant", [{"type": "text", "text": "交回了"}], stop_reason="end_turn", message_id="m3")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、10 分钟前交回"})
    # 忙着跑一条长工具调用：主 agent 20 分钟前发的消息还在主会话记录里、没落进它自己的会话记录
    write_transcript(session, "inquirysentafter", [bash_use(90, "t1", "cargo test --release", "m1")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、一直在跑一条工具调用、主 agent 第一格之后发过消息"})
    write_transcript(session, "inquiryfailed", [bash_use(90, "t1", "ls", "m1"), bash_result(89.9, "t1"),
                                                record_at(80, "assistant", [{"type": "text", "text": "接着跑"}], message_id="m2")],
                     {"agentType": "implementation-writer", "description": "派发 90 分钟、撞限额失败（任务通知 failed）"})
    with open(session + ".jsonl", "a", encoding="utf-8") as handle:   # 主会话记录里 SendMessage 结果的形态照 2026-09-24 主会话记录里的原样
        for sent_minutes_ago, sent_result in (
                (20, {"success": True, "message": "Message queued for delivery to inquirysentafter at its next tool round.",
                      "pin": {"id": "inquirysentafter", "name": "inquirysentafter", "ref": "e00001"}}),
                (5, {"success": True, "message": "Resuming agent inquiry", "resumedAgentId": "inquiryresumedsample",
                     "pin": {"id": "inquiryresumedsample", "name": "inquiryresumedsample", "ref": "e00002"}}),
                (5, {"success": False, "message": "Message queued for delivery to inquirynotsent at its next tool round.",
                     "pin": {"id": "inquirynotsent", "name": "inquirynotsent", "ref": "e00003"}})):
            sent_record = record_at(sent_minutes_ago, "user", [{"tool_use_id": "sm1", "type": "tool_result",
                                                                "content": [{"type": "text", "text": json.dumps(sent_result, ensure_ascii=False)}]}])
            sent_record["toolUseResult"] = sent_result
            handle.write(json.dumps(sent_record, ensure_ascii=False) + "\n")
        handle.write(json.dumps(record_at(79.5, "user", "<task-notification>\n<task-id>inquiryfailed</task-id>\n<status>failed</status>\n"
                                                        "<summary>Agent failed: API error</summary>\n</task-notification>", origin_kind="task-notification"),
                                ensure_ascii=False) + "\n")
    messages_sent_to = set(session_task_events(session + ".jsonl")[2])
    if messages_sent_to != {"inquirysentafter", "inquiryresumedsample"}:
        failures.append(f"主会话记录里 SendMessage 发给子 agent 的成功结果应当认出排队与续做两种写法、success 不是 true 的不算："
                        f"应当是 inquirysentafter、inquiryresumedsample，实际 {sorted(messages_sent_to) or '无'}")
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
        "toolafterhandback": set(), "movedbackground": set(), "queuedresume": {"结束本轮却不会醒"},
        "hugecontext": {"上下文过大"}, "waitloop": {"等待循环"}, "longtool": {"工具调用过长"}, "stalepending": set(), "repeat": {"同一命令反复且输出不变"}, "banned": {"禁用命令"}, "bannedrefused": set(), "bannedold": set(), "bannedheredoc": set(), "bannedafterheredoc": {"禁用命令"}, "idle": {"无动静"},
        "inquiry61": {"跑满 1 小时要主 agent 问一次"}, "inquiry125": {"跑满 2 小时要主 agent 问一次"}, "inquiryhandedback": set(),
        "inquirymessagebefore": {"跑满 1 小时要主 agent 问一次"}, "inquirydeliveredafter": set(), "inquiryqueuedafter": set(),
        "inquirysentafter": {"工具调用过长"}, "inquiryfailed": {"结束本轮却不会醒"},
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
                   "waitinglong": False, "toolafterhandback": True, "movedbackground": False, "queuedresume": False, "inquiryhandedback": True}
    for agent_id, wanted_state in (("textsettled", "本轮结束、没交回"), ("stopsequence", "本轮结束、没交回"), ("textstreaming", "思考中"),
                                   ("resumedafterturnend", "思考中")):
        got_state = AgentTranscript(agent_id, find_transcript(agent_id, session)).state
        if got_state != wanted_state:
            failures.append(f"子 agent {agent_id} 的状态应当是「{wanted_state}」，实际「{got_state}」")
    for agent_id, wanted in wanted_done.items():
        transcript = AgentTranscript(agent_id, find_transcript(agent_id, session))
        if transcript.is_done() != wanted:
            failures.append(f"子 agent {agent_id} 应当{'算' if wanted else '不算'}交回或被停，实际状态「{transcript.state}」")
    moved_background_transcript = AgentTranscript("movedbackground", find_transcript("movedbackground", session))
    if moved_background_transcript.background_started != ["bm1"] or not moved_background_transcript.waiting_on_its_own_background_tasks():
        failures.append(f"前台命令超时被挪进后台（ID bm1）之后结束本轮的，应当算在等自己的后台任务，实际数到起过的后台任务 "
                        f"{moved_background_transcript.background_started}、在不在等：{moved_background_transcript.waiting_on_its_own_background_tasks()}")
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
    probed_processes = [sleeper, busy]
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
    pipelines_checked = selftest_pipe_upstream(work, thresholds, failures, probed_processes)
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
    # 派发 125 分钟、--ack 了 1 小时那一格：照报 2 小时，出路是例行询问；--ack 了 2 小时那一格才不叫醒
    first_mark_acknowledged_run = run_watch(["--agents", "inquiry125", "--ack", "inquiry125:跑满 1 小时", "--interval-seconds", "1", "--max-minutes", "1"])
    wanted_inquiry_lines = ("⚠️ 跑满 2 小时要主 agent 问一次（inquiry125）：", f"   → 怎么办：{ROUTINE_INQUIRY_NEXT_STEP}\n")
    if first_mark_acknowledged_run.returncode != 3 or not all(line in first_mark_acknowledged_run.stdout for line in wanted_inquiry_lines):
        failures.append(f"派发 125 分钟、--ack 了「跑满 1 小时」那一格时看门狗应当照报 2 小时那一格并退出码 3，要有这两行 {wanted_inquiry_lines}，"
                        f"实际退出码 {first_mark_acknowledged_run.returncode}，输出：{first_mark_acknowledged_run.stdout[-400:]}")
    second_mark_acknowledged_run = run_watch(["--agents", "inquiry125", "--ack", "inquiry125:跑满 2 小时", "--interval-seconds", "1", "--max-minutes", "0.03"])
    if second_mark_acknowledged_run.returncode != 4 or "已确认接着盯（--ack inquiry125:跑满 2 小时要主 agent 问一次）" not in second_mark_acknowledged_run.stdout:
        failures.append(f"派发 125 分钟、--ack 了「跑满 2 小时」那一格时看门狗不该叫醒：应当到点退出码 4 并写「已确认接着盯」，"
                        f"实际退出码 {second_mark_acknowledged_run.returncode}，输出：{second_mark_acknowledged_run.stdout[-400:]}")
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
    # --processes-only：拿自检进程自己当那个 Claude 实例。一个一直在写文件的、一个写着的文件一小时没动而且不占 CPU 的，
    # 只许后一个报「进程无输出」；一个两秒就退出、退出之后没人收尸（僵尸）的，盯到它退出就退 0；一个都没有也退 0 并写明。
    def run_process_watch(watch_arguments, timeout=120):
        watch_runs.append(["--processes-only", *watch_arguments])
        return subprocess.run([sys.executable, os.path.abspath(__file__), "watch", "--processes-only", "--process-root-pid", str(os.getpid()),
                               "--interval-seconds", "1", *watch_arguments], capture_output=True, text=True, timeout=timeout)

    writing_file = os.path.join(work, "writing-output.log")
    writer = subprocess.Popen([sys.executable, "-c", "import time\n"
                               f"handle = open({writing_file!r}, 'a')\n"
                               "deadline = time.time() + 60\nwhile time.time() < deadline:\n"
                               "    handle.write('一行\\n')\n    handle.flush()\n    time.sleep(0.3)\n"])
    silent_file = os.path.join(work, "silent-output.log")
    open(silent_file, "w").close()
    silent = subprocess.Popen(["bash", "-c", f"exec 3>>'{silent_file}'; touch -d '1 hour ago' '{silent_file}'; exec sleep 60"])
    probed_processes += [writer, silent]
    try:
        writer_and_silent_run = run_process_watch(["--max-minutes", "0.5", "--process-report-minutes", "0.02", "--process-stale-minutes", "0.05",
                                     "--process-max-minutes", "600"])
        alert_lines = [line for line in writer_and_silent_run.stdout.splitlines() if line.startswith("⚠️")]
        if writer_and_silent_run.returncode != 3 or not any("进程无输出" in line and f"进程 {silent.pid} " in line for line in alert_lines):
            failures.append(f"--processes-only 时写着的文件一小时没动、CPU 也不涨的进程 {silent.pid} 应当报「进程无输出」并退出码 3，"
                            f"实际退出码 {writer_and_silent_run.returncode}，告警：{alert_lines or '无'}")
        if any(f"进程 {writer.pid} " in line for line in alert_lines):
            failures.append(f"--processes-only 时一直在写文件的进程 {writer.pid} 被当成告警：{alert_lines}")
    finally:
        for process in (writer, silent):
            process.kill()
            process.wait()
    short = subprocess.Popen(["sleep", "2"])   # 自检在 subprocess.run 里等看门狗，这两秒退出的进程没人收尸，就是个僵尸
    probed_processes.append(short)
    try:
        finished = run_process_watch(["--max-minutes", "0.25"])
        if finished.returncode != 0 or "都退出了" not in finished.stdout:
            failures.append(f"--processes-only 盯着的进程退出了（僵尸没人收尸也算）应当退出码 0 并写「都退出了」，"
                            f"实际退出码 {finished.returncode}，输出：{finished.stdout[-300:]}")
    finally:
        short.kill()
        short.wait()
    nothing = run_process_watch(["--max-minutes", "0.25"])
    if nothing.returncode != 0 or "就没有要盯的进程" not in nothing.stdout:
        failures.append(f"--processes-only 起的时候一个要盯的进程都没有，应当退出码 0 并写明，实际退出码 {nothing.returncode}，输出：{nothing.stdout[-300:]}")
    if "这一类没查" not in nothing.stdout:
        failures.append(f"--processes-only 没给 --session-dir 时报告里应当写明「交回之后后台还在跑」这一类没查，实际输出：{nothing.stdout[-300:]}")
    leftover_cases_checked = selftest_leftover_background(work, thresholds, failures, watch_runs, probed_processes)
    heavy_processes_started = selftest_heavy_tests(work, failures, probed_processes, watch_runs)
    memory_processes_started = selftest_memory(work, failures, probed_processes, watch_runs)
    scribes_checked = selftest_scribe_followups(work, failures, watch_runs)
    for mixed in (["watch", "--processes-only", "--agents", "healthy"], ["cost", "--processes-only"]):
        mixed_run = subprocess.run([sys.executable, os.path.abspath(__file__), *mixed], capture_output=True, text=True, timeout=30)
        if mixed_run.returncode != 2:
            failures.append(f"{' '.join(mixed)} 应当退出码 2（--processes-only 不和子 agent、用量混用），实际 {mixed_run.returncode}")
    for wrapped_command, wanted_display in (
            ("/bin/bash -c source s.sh 2>/dev/null || true && eval 'cargo test\n  --release' < /dev/null && pwd -P >| /tmp/claude-1-cwd", "cargo test --release"),
            ("/bin/bash -c source s.sh 2>/dev/null || true && eval 'grep '\"'\"'x'\"'\"' f' && pwd -P >| /tmp/claude-2-cwd", "grep 'x' f"),
            ("sleep 60", "sleep 60")):
        if command_for_display(wrapped_command) != wanted_display:
            failures.append(f"Bash 工具包着的命令在报告里应当显示成「{wanted_display}」，实际「{command_for_display(wrapped_command)}」")
    subprocess.run(["rm", "-rf", work])
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 agent_alerts() / process_alerts() / heavy_test_report() / memory_report() / read_detections() / scribe_followup_report() / run() 的判法；"
              "AGENT_WATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ agent-watch 自检通过：等待循环、工具过长、同一命令反复且输出不变、禁用命令、无动静、进程无输出六种告警都报得出，"
          f"交回、被停、带超时的循环、输出在变的复检与健康的子 agent 不误报，交回按交回工具成功判（只结束本轮、交回后又被续做的都不算，忙时排进队的续做消息也算续做；"
          f"交回之后被自己后台任务的完成通知叫醒、又调了工具再结束本轮的仍算交回），前台命令超时转进后台的也算它起过的后台任务，"
          f"被停认会话记录里的打断与主会话记录里的 TaskStop（停在等后台任务时的算、停了又被续做的不算、续做之后又停的算，主会话记录不在的标出来），"
          f"结束本轮而起过的后台任务都已完成、没有通知在路上的报「结束本轮却不会醒」（通知刚到正要醒的不报），看门狗三种退出码对、定时回报不被复检间隔拖后并列出还没交回的子 agent，本会话的 hook 检出会叫醒主 agent、别的会话的与只检出等待循环的不立刻叫醒，用量与工具结果分类对；结束本轮认 end_turn 与 stop_sequence、纯文字记录两分钟没接下文也认，结束本轮之后收到续做消息就算在想，用 --ack 确认过的告警只进报告不叫醒，主 agent 自己被拒的写只进报告不叫醒、入口已拒的命令不再按禁用命令报、别的子 agent 的检出不叫醒这一个看门狗，等自己的后台任务时列出开着输出文件的进程，结束本轮等自己的后台任务的不报无动静（输出文件已没有进程开着、又没收到通知的后台任务不算在等，报「结束本轮却不会醒」），写着的文件不动而 CPU 在涨的进程只报「在算」不告警，stdin 是管道的管道尾上游在算只报「在等上游」、上游睡着或写端只在自己祖先手里照报「进程无输出」（{pipelines_checked} 个管道）；--processes-only 只盯进程：一直在写文件的不报、写着的文件不动且 CPU 不涨的报「进程无输出」，"
          f"盯着的进程都退出了（退出之后没人收尸的僵尸也算）退 0、起的时候一个都没有也退 0 并写明，和 --agents、cost 混用退 2，Bash 工具包着的命令在报告里只列 eval 里那一段；"
          f"还没交回的子 agent 从派发起每跨过一个整点报「跑满 N 小时要主 agent 问一次」（报已跨过的最近那一格，--ack <agent 号>:跑满 N 小时 只确认那一格；"
          f"第一格之前收到的消息不重新计时，那一格之后主 agent 发过消息的不报——送达的 user 记录与排队附件、主会话记录里还没送达的 SendMessage 三处都认；"
          f"交回的、任务通知 failed 的不报）；"
          f"交回、被 TaskStop 停掉、最近一次任务通知是 failed 的子 agent 过了宽限还有进程开着它后台任务的输出文件报「交回之后后台还在跑」（交回不到宽限、没交回还在等、"
          f"stopped 之后又被续做的不报，进程没了不报，--ack 进程:<pid> 不叫醒），被看的全部交回时退出前也报、只剩宽限里的等宽限过了再查，--processes-only 带 --session-dir 也报、不带就写明没查；"
          f"名字含 layer0 的测试二进制在跑而环境里没有 {HEAVY_TEST_VARIABLE}=commit / =user-request 的报「{HEAVY_TEST_ALERT}」（带别的值照报，带这两个值的、"
          f"bash -c 里写着带前缀而还没起的、timeout 包着带前缀的不报，另一个看门狗的自检那一支整棵不扫，一条链只报最上面那个、下一步给从最底层往上的停止次序，归属认出开着输出文件的子 agent），"
          f"--processes-only 与 --agents 两条路都报、--ack 进程:<pid> 不叫醒；常驻内存过阈值的进程报「{MEMORY_ALERT}」（写常驻内存、阈值、归属与出路，"
          f"只睡的小进程不报，阈值 0 不查），两条路整查都报、--ack 进程:<pid> 不叫醒，看门狗起了之后才过线的由两次整查之间的单查几秒内叫醒（{memory_processes_started} 个进程）；"
          f"书记员还在写时「{SCRIBE_FOLLOWUP_FINDING_PREFIX}」不叫醒、交回时报一条汇总，"
          f"看门狗起之前攒下的起的时候重读出来，书记员别的类别照旧立刻叫醒，认不出是哪个书记员的等会话里书记员都不在跑了再报"
          f"（查了 {len(expectations) + 1 + leftover_cases_checked + 1 + scribes_checked} 个子 agent、{len(probed_processes)} 个进程"
          f"（其中假的重型进程 {heavy_processes_started} 个）、{len(watch_runs)} 次看门狗）")
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
    parser.add_argument("--leftover-background-grace-minutes", type=float, default=2,
                        help="子 agent 交回、被停或失败之后过了这么久，它起的后台任务还有进程开着输出文件就报「交回之后后台还在跑」，默认 2 分钟")
    parser.add_argument("--ask-every-minutes", type=float, default=60,
                        help="还没交回的子 agent 从派发起每跑满这么久（整数倍）报一次「跑满 N 小时要主 agent 问一次」，"
                             "那一格之后主 agent 给它发过消息就不报；默认 60 分钟，写 0 不报")
    parser.add_argument("--active-minutes", type=float, default=180, help="只给 --session-dir 时，只看这么久以内动过的子 agent")
    parser.add_argument("--detections-file", default=DEFAULT_DETECTIONS, help="hook 的检出记录，默认 /tmp/claude-1000/agent-hook-detections.jsonl")
    parser.add_argument("--detection-poll-seconds", type=float, default=15,
                        help="watch 两次读检出记录的间隔；watch 与 --processes-only 两次整查之间也按它单查常驻内存，默认 15 秒")
    parser.add_argument("--process-rss-gibibytes", type=float, default=8,
                        help="这个 Claude 实例底下任一进程常驻内存超过这么多 GiB 就报「进程常驻内存过线」，写 0 不查；默认 8 GiB。"
                             "依据：本机 60 GiB 内存，本地模型服务连同会话常驻约 12 GiB（2026-09-25 free -g 的 used 列），同时可能有 3 件重活，"
                             "每件分 (60 − 12) ÷ 3 = 16 GiB，阈值取一件的一半；换机器要重算。它管涨得慢的：E142 那条无界分配的变异每秒约 5 GB、十几秒吃满整机，"
                             "快过任何复检间隔，那一类只有内存上限（research/scripts/run-with-memory-cap.sh）挡得住")
    parser.add_argument("--ack", action="append", default=[],
                        help="主 agent 看过、判定只是慢接着盯的告警：子agent id:告警名，进程告警写 进程:PID，"
                             "「跑满 N 小时要主 agent 问一次」可写 子agent id:跑满 N 小时（只确认那一格）；可给多次，只在这一次看门狗里有效")
    parser.add_argument("--process-root-pid", type=int, help="进程一半从哪个 pid 往下看；默认是跑这个脚本的 Claude 实例，0 表示不看进程")
    parser.add_argument("--processes-only", action="store_true",
                        help="不看子 agent，只盯这个 Claude 实例底下起的时候就在跑的进程，它们都退出了就退 0；主 agent 自己放后台的长命令用它盯；"
                             "带 --session-dir 时同时查那个会话里交回之后后台还在跑")
    arguments = parser.parse_args()
    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
