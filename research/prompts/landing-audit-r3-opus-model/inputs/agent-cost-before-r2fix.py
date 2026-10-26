#!/usr/bin/env python3
# admission: always 每一次调都按此刻的会话记录与此刻的仓算账，上一次的结论不替这一次作保
# run-condition: command git
"""agent 体系的账表：子 agent 与主 agent 的用量、每类的挂钟与等待、派发与闸的拒绝、看门狗的退出、每个子 agent 的结局。

用法：
    agent-cost.py usage    [--root 会话目录] [--since YYYY-MM-DD]   # 子 agent 按类型的调用、上下文、读缓存、折算；主会话各自的账
    agent-cost.py timing   [--root …] [--since …]                  # 每类的挂钟中位、等待占比、没交回的；三方一轮的关键路径
    agent-cost.py dispatch [--root …] [--since …]                  # 派发按类型、派发闸与钩子的拒绝、重复派发、消息、看门狗退出码
    agent-cost.py outcome  [--root …] [--since …] [--until YYYY-MM-DD] [--repo 仓根] [--summary] [--jobs N]
                                                                   # 每个子 agent 一行的结局账，再汇总；--until 只收那一天及之前开工的（含那一天，不给不设上界）；
                                                                   # --summary 只打汇总；--jobs 读会话记录的进程数（默认 12 与每台默认重活线程数取小）；成功句报 --since、--until 的取值
    agent-cost.py --selftest                                       # 造假会话目录与假仓算一遍，钉住数；弄坏开关打开时必须判红

口径（与 records/2026-09-28-agent体系用量时间与调度分析.md 第一节同）：一次调用是一条带 usage 的 assistant 消息、按消息 id 去重；
上下文 = cache_read + cache_creation + input；折算 = 新输入 ×1 + 读缓存 ×0.1 + 5 分钟缓存写 ×1.25 + 1 小时缓存写 ×2 + 输出 ×5（相对量，不是账单）；
等待 = 相邻两次调用间隔超过 3 分钟。第一版分析用的五份脚本在 research/prompts/agent-cost-2026-09-28/，这一份把它们收进仓、加自证。
四个子命令收哪些子 agent 按起点（首条记录的时刻）筛，「交回」按送达认（调过 SubagentHandback、后面有它对应、不是错误的 tool_result；被交回闸拒的不算）：
两样都只在同目录 subagent_transcript.py 里定义，续做闸 .claude/hooks/continuation-guard.sh 与看门狗 agent-watch.py 取的是同一份。

outcome 的口径（桶名照 records/2026-10-22-子agent结果落地审计.md「按结局分」；起点按子 agent 首条记录的时间，不按文件修改时间；交回的正文取最近一次送达的那一次）：
  写入落地：子 agent 每次成功的写，取新增的行（去首尾空白后不短于 12 个字，Edit 去掉 old_string 里已有的），比对
    「git 历史加进的行（所有分支，--since 往前多取 6 天）∪ git worktree list 列出的每棵工作树里的行（含主工作树，已跟踪与未跟踪未忽略的文件）」。
    认的写：Edit / Write / MultiEdit；Bash 里的 heredoc 写文件（cat / tee 加 >、>>）、python 写文件（heredoc、-c、先 heredoc 写成再执行的脚本，
    按字符串字面量取行）、scripts/kb 下 replace-once.py / replace-batch.py / insert-row.py 的实写（带 --dry-run 的不算）；tool_result 报错的调用不算。
  引用：交回之后，主会话与后来的子 agent 的工具调用里点到它交回里写的路径、它的 agent 号、或它草稿目录的目录名（路径里带变量、通配、花括号、
    循环拼出来的，把那一段当通配与目录名对；同一条命令里赋过的变量先展开）。交回里的路径取到空白、引号、括号、全角标点为止，扩展名后面直接粘着的汉字说明也去掉，
    去掉粘在后面的全角括号与说明文字。两个子 agent 共用同一个目录名时，按目录名点到的只算在下一个共用者开工之前（按完整路径、agent 号点到的不受此限）。
  桶：三方腿类型（three-way 开头）记 verdict-archived；没交回的按最后一句分 cut-by-usage-limit、cut-by-api-error、in-flight（最后一条记录距此刻不到
    30 分钟）、stopped-salvaged（停掉，但后来有人点到）、stopped-abandoned；交回了的写了不少于 10 行的按落地率分 written-landed（不低于 70%）、
    written-partial（30% 到 70%）、written-overwritten（更低，主 agent 点到过）、written-unused（更低，没人点到）；其余按引用分 report-used-by-main、
    report-used-by-later-sub、report-unreferenced。
  返工字样：描述里带 REWORK_WORDS 的一个词（修（「修订」不算：跑前登记的修订是设计员与执行员的正常步骤）、重跑、重派、再派、重做、返工、续做、续派、接续、接手、接着做、（续）、补写、补做、改名、改号、改钉值、第 2 次及以后，与英文的 redispatch、rerun、renumber、retry、fix）。「后续批」这类只说次序的不算。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句冒号前的名字，括号里是那条「✗ 自检：」的开头）：
  AGENT_COST_BREAK=no-dedupe  read_transcript() 不按消息 id 去重，同一个消息 id 的流式重复行各算一次
      子 agent 用量（「✗ 自检：子 agent 用量：应当 1 个、3 次调用（同一个消息 id 的重复行只算一次）」）
  AGENT_COST_BREAK=no-path-normalize  交回里的路径不在全角括号与说明文字处截断，主会话命令里的变量、通配、花括号、目录名一概不展开不对
      路径归一化（「✗ 自检：路径归一化：」）
  AGENT_COST_BREAK=no-bash-writes  Bash 里的 heredoc、python、replace-once / replace-batch / insert-row 的写一概不认，只认 Edit / Write / MultiEdit
      Bash 写入（「✗ 自检：Bash 写入：」）
  AGENT_COST_BREAK=narrow-rework-words  返工字样退回只认「修、重跑、续做、改名、改号」五个词（单字「修」把「修订」也算进去）
      返工词表（「✗ 自检：返工词表：」）
同目录 subagent_transcript.py 的两个开关（判法在那一份，开关表也在它的文件头），打开后这里红的项：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
      交回与起点（「✗ 自检：交回与起点：outcome 应当收…」「✗ 自检：交回与起点：timing 应当收 3 个…」）
  SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime  起点取文件修改时刻，不读首条记录
      交回与起点（「✗ 自检：交回与起点：outcome 应当收…」）
"""
import ast
import bisect
import collections
import concurrent.futures
import datetime
import glob
import json
import multiprocessing
import os
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import warnings
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
import heavy_threads  # noqa: E402  每台默认重活线程数只在这一份里取（登记位 .claude/heavy-thread-default）
import subagent_transcript as transcripts  # noqa: E402  子 agent 会话记录的「交回」与「起点」只在这一份里定义（续做闸、看门狗同取它）

REPOSITORY_ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
# Claude Code 按项目的绝对路径把 / 换成 - 起会话目录名（同 scripts/agents/watch.sh 的 ${root//\//-}）
DEFAULT_ROOT = os.path.join(os.path.expanduser("~/.claude/projects"), REPOSITORY_ROOT.replace("/", "-"))
BROKEN = os.environ.get("AGENT_COST_BREAK", "")
UTC = datetime.timezone.utc   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
AGENT_ID = re.compile(r"a[0-9a-f]{16}")
PRIME_MARKS = re.compile("[" + "".join(chr(code) for code in (0x2032, 0x2033, 0x2034, 0x02B9, 0x02BA)) + "]")   # 输出里不留角标字符：门禁 doc-text 的 prime-marks 格扫全仓，抄进记录的原文会被它判红


def timestamp_of(record):
    try:
        return datetime.datetime.fromisoformat(record.get("timestamp", "").replace("Z", "+00:00"))
    except ValueError:
        return None


def records(path):
    with open(path, errors="replace") as handle:
        for line in handle:
            try:
                yield json.loads(line)
            except ValueError:
                continue


def text_of(content):
    return content if isinstance(content, str) else (json.dumps(content, ensure_ascii=False) if content else "")


def folded(totals):
    return (totals["input"] + totals["cache_read"] * 0.1 + totals["write_5m"] * 1.25 + totals["write_1h"] * 2 + totals["output"] * 5)


def context_of(usage):
    return usage.get("cache_read_input_tokens", 0) + usage.get("cache_creation_input_tokens", 0) + usage.get("input_tokens", 0)


def median(values):
    return statistics.median(values) if values else 0


def recent(path, since):
    return datetime.datetime.fromtimestamp(os.path.getmtime(path), tz=UTC) >= since   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点


def usage_key(message, record):
    """一次调用的去重键：同一个消息 id 的流式重复行只算一次（弄坏开关 no-dedupe 退回按记录号）。"""
    return (message.get("id") or record.get("uuid")) if BROKEN != "no-dedupe" else record.get("uuid") or id(record)


def usage_totals(calls):
    """一串去重后的 usage 加成五个量；read_transcript 与 outcome 的每个子 agent 花费同走这一处。"""
    totals = collections.Counter()
    for usage in calls:
        creation = usage.get("cache_creation") or {}
        totals["input"] += usage.get("input_tokens", 0)
        totals["cache_read"] += usage.get("cache_read_input_tokens", 0)
        totals["write_5m"] += creation.get("ephemeral_5m_input_tokens", usage.get("cache_creation_input_tokens", 0) if not creation else 0)
        totals["write_1h"] += creation.get("ephemeral_1h_input_tokens", 0)
        totals["output"] += usage.get("output_tokens", 0)
    return totals


def read_transcript(path):
    """一份会话记录：去重后的调用（按次序）、首末时刻、交回时刻（最近一次送达的那一次，认法在 subagent_transcript.py）、工具调用计数。"""
    seen, order, first, last = {}, [], None, None
    tools = collections.Counter()
    handbacks = transcripts.HandbackTracker()
    for record in records(path):
        moment = timestamp_of(record)
        if moment:
            first = first or moment
            last = moment
        handbacks.feed(record)
        if record.get("type") != "assistant":
            continue
        message = record.get("message", {})
        usage = message.get("usage")
        if usage:
            key = usage_key(message, record)
            if key not in seen:
                order.append(key)
            seen[key] = usage
        for block in message.get("content", []) if isinstance(message.get("content"), list) else []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                tools[block.get("name")] += 1
    calls = [seen[key] for key in order]
    totals = usage_totals(calls)
    handback = handbacks.delivered["delivered_at"] if handbacks.delivered else None
    return {"calls": calls, "totals": totals, "first": first, "last": last, "handback": handback, "tools": tools,
            "wall": (last - first).total_seconds() if first and last else 0}


def subagent_files(root, since, until=None):
    """起点（首条记录的时刻，subagent_transcript.started_between）落在 [since, until) 里的子 agent：(类型, 会话记录路径)。"""
    for meta in sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.meta.json"))):
        transcript = meta.replace(".meta.json", ".jsonl")
        if os.path.exists(transcript) and transcripts.started_between(transcript, since, until):
            try:
                agent_type = json.load(open(meta)).get("agentType", "?")
            except (OSError, ValueError):
                agent_type = "?"
            yield agent_type, transcript


def command_usage(root, since):
    by_type = collections.defaultdict(collections.Counter)
    first_contexts = collections.defaultdict(list)
    for agent_type, transcript in subagent_files(root, since):
        data = read_transcript(transcript)
        if not data["calls"]:
            continue
        totals = data["totals"]
        totals["n"] = 1
        totals["calls"] = len(data["calls"])
        totals["folded"] = folded(totals)
        by_type[agent_type].update(totals)
        first_contexts[agent_type].append(context_of(data["calls"][0]))
    lines = [f"## 子 agent 按类型（{since.date()} 起）", f"{'类型':26}{'个数':>5}{'调用':>7}{'起步上下文中位':>12}{'读缓存(亿)':>10}{'5分钟写(亿)':>11}{'输出(万)':>9}{'折算(亿)':>9}"]
    grand = collections.Counter()
    for agent_type, totals in sorted(by_type.items(), key=lambda item: -item[1]["folded"]):
        lines.append(f"{agent_type:26}{totals['n']:>5}{totals['calls']:>7}{median(first_contexts[agent_type]):>12.0f}{totals['cache_read'] / 1e8:>10.2f}"
                     f"{totals['write_5m'] / 1e8:>11.2f}{totals['output'] / 1e4:>9.0f}{totals['folded'] / 1e8:>9.2f}")
        grand.update(totals)
    lines.append(f"{'合计':26}{grand['n']:>5}{grand['calls']:>7}{'':>12}{grand['cache_read'] / 1e8:>10.2f}{grand['write_5m'] / 1e8:>11.2f}{grand['output'] / 1e4:>9.0f}{grand['folded'] / 1e8:>9.2f}")
    lines.append("")
    lines.append(f"## 主会话（{since.date()} 起）")
    lines.append(f"{'会话':10}{'调用':>7}{'读缓存(亿)':>10}{'1小时写(亿)':>11}{'折算(亿)':>9}{'平均上下文(万)':>12}{'挂钟(时)':>9}{'子agent':>8}")
    main_grand = collections.Counter()
    for transcript in sorted(glob.glob(os.path.join(root, "*.jsonl"))):
        if not recent(transcript, since):
            continue
        data = read_transcript(transcript)
        if not data["calls"]:
            continue
        totals = data["totals"]
        subagents = len(glob.glob(transcript[:-6] + "/subagents/*.jsonl"))
        lines.append(f"{os.path.basename(transcript)[:8]:10}{len(data['calls']):>7}{totals['cache_read'] / 1e8:>10.2f}{totals['write_1h'] / 1e8:>11.2f}"
                     f"{folded(totals) / 1e8:>9.2f}{totals['cache_read'] / max(1, len(data['calls'])) / 1e4:>12.0f}{data['wall'] / 3600:>9.1f}{subagents:>8}")
        totals["calls"] = len(data["calls"])
        totals["folded"] = folded(totals)
        main_grand.update(totals)
    lines.append(f"{'合计':10}{main_grand['calls']:>7}{main_grand['cache_read'] / 1e8:>10.2f}{main_grand['write_1h'] / 1e8:>11.2f}{main_grand['folded'] / 1e8:>9.2f}")
    return lines, by_type, main_grand


def command_timing(root, since):
    by_type = collections.defaultdict(list)
    for agent_type, transcript in subagent_files(root, since):
        data = read_transcript(transcript)
        if not data["calls"]:
            continue
        moments = [timestamp_of(record) for record in records(transcript) if record.get("type") == "assistant" and record.get("message", {}).get("usage")]
        moments = [moment for moment in moments if moment]
        gaps = [(later - earlier).total_seconds() for earlier, later in zip(moments, moments[1:])]
        waiting = sum(gap for gap in gaps if gap > 180)
        by_type[agent_type].append((data["wall"], waiting, data["handback"] is None))
    lines = [f"## 每类的挂钟与等待（{since.date()} 起）", f"{'类型':26}{'个数':>5}{'挂钟中位(分)':>11}{'挂钟p90':>8}{'等待占挂钟':>10}{'没交回':>7}"]
    total_wall = total_wait = 0
    for agent_type, rows in sorted(by_type.items(), key=lambda item: -len(item[1])):
        walls = sorted(row[0] / 60 for row in rows)
        wall_sum = sum(row[0] for row in rows)
        wait_sum = sum(row[1] for row in rows)
        total_wall += wall_sum
        total_wait += wait_sum
        lines.append(f"{agent_type:26}{len(rows):>5}{median(walls):>11.0f}{walls[int(len(walls) * 0.9)] if walls else 0:>8.0f}"
                     f"{(wait_sum / wall_sum if wall_sum else 0):>10.0%}{sum(1 for row in rows if row[2]):>7}")
    lines.append(f"合计挂钟 {total_wall / 3600:.0f} 小时，其中等待 {total_wait / 3600:.0f} 小时（{(total_wait / total_wall if total_wall else 0):.0%}）")
    return lines, by_type


def command_dispatch(root, since):
    dispatches = collections.Counter()
    launched = 0
    rejections = collections.defaultdict(collections.Counter)
    repeats = collections.Counter()
    messages = collections.Counter()
    notifications = collections.Counter()
    batch_sizes = []
    serial = 0
    for transcript in sorted(glob.glob(os.path.join(root, "*.jsonl"))):
        if not recent(transcript, since):
            continue
        session = os.path.basename(transcript)[:8]
        pending, dispatch_moments, handback_moments, id2command = {}, [], [], {}
        for record in records(transcript):
            record_type = record.get("type")
            message = record.get("message", {})
            content = message.get("content")
            moment = timestamp_of(record)
            if record_type == "assistant" and isinstance(content, list):
                for block in content:
                    if not (isinstance(block, dict) and block.get("type") == "tool_use"):
                        continue
                    name = block.get("name")
                    tool_input = block.get("input") or {}
                    if name in ("Agent", "Task"):
                        dispatches[tool_input.get("subagent_type", "?")] += 1
                        repeats[(session, (tool_input.get("description") or "")[:40])] += 1
                        pending[block.get("id")] = moment
                    elif name == "Bash":
                        id2command[block.get("id")] = tool_input.get("command", "")
                    elif name == "SendMessage":
                        to = str(tool_input.get("to", ""))
                        messages["给子 agent" if AGENT_ID.fullmatch(to) else "给别的会话"] += 1
            elif record_type == "user":
                text = text_of(content)
                if isinstance(content, list):
                    for block in content:
                        if not (isinstance(block, dict) and block.get("type") == "tool_result"):
                            continue
                        body = text_of(block.get("content"))
                        if block.get("tool_use_id") in pending:
                            started = pending.pop(block["tool_use_id"])
                            result = record.get("toolUseResult")
                            if isinstance(result, dict) and result.get("agentId"):
                                launched += 1
                                if started:
                                    dispatch_moments.append(started)
                        if "hook error" in body and "✗" in body:
                            hook = re.search(r"hook error: \[bash [^\]]*?/([a-z\-]+\.sh)\]", body)
                            reason = re.search(r"✗ ?(.{0,40})", body)
                            reason_text = PRIME_MARKS.sub("[角标]", re.sub(r"\d+", "N", re.sub(r"a[0-9a-f]{16}", "<id>", reason.group(1) if reason else "")))
                            rejections[hook.group(1) if hook else "?"][reason_text[:40]] += 1
                if "<task-notification>" in text:
                    status = re.search(r"<status>(\w+)</status>", text)
                    task = re.search(r"<task-id>([^<]+)</task-id>", text)
                    tool_use = re.search(r"<tool-use-id>([^<]+)</tool-use-id>", text)
                    command = id2command.get(tool_use.group(1), "") if tool_use else ""
                    exit_code = re.search(r"exit code (\d+)", text)
                    if task and AGENT_ID.fullmatch(task.group(1)):
                        kind = "子 agent"
                        if moment:
                            handback_moments.append(moment)
                    elif "watch.sh" in command or "agent-watch.py" in command:
                        kind = f"看门狗 退 {exit_code.group(1) if exit_code else '-'}"
                    else:
                        kind = "别的后台命令"
                    notifications[(kind, status.group(1) if status else "?")] += 1
        dispatch_moments.sort()
        current = 1
        for earlier, later in zip(dispatch_moments, dispatch_moments[1:]):
            if (later - earlier).total_seconds() <= 90:
                current += 1
            else:
                batch_sizes.append(current)
                current = 1
        if dispatch_moments:
            batch_sizes.append(current)
        handback_moments.sort()
        for started in dispatch_moments:
            index = bisect.bisect_right(handback_moments, started)
            if index > 0 and (started - handback_moments[index - 1]).total_seconds() <= 600:
                serial += 1
    lines = [f"## 派发（{since.date()} 起）：Agent 工具调用 {sum(dispatches.values())} 次，真正起来的 {launched} 个"]
    lines += [f"  {agent_type:28}{count}" for agent_type, count in dispatches.most_common()]
    lines.append("## 钩子拒绝（主会话侧，按钩子与原因）")
    for hook, reasons in sorted(rejections.items(), key=lambda item: -sum(item[1].values())):
        lines.append(f"  {hook}: {sum(reasons.values())}")
        lines += [f"      {count:4d} {reason}" for reason, count in reasons.most_common(5)]
    repeated = [(key, count) for key, count in repeats.items() if count >= 3]
    lines.append(f"## 同一描述在同一会话派第三次以上：{len(repeated)} 种描述、{sum(count for _, count in repeated)} 次派发"
                 f"（占 {sum(count for _, count in repeated) / max(1, sum(dispatches.values())):.0%}）")
    lines.append(f"## 派发批（相邻 90 秒内算一批）：中位 {median(batch_sizes):.0f} 个，只派 1 个的批占 "
                 f"{sum(1 for size in batch_sizes if size == 1) / max(1, len(batch_sizes)):.0%}；交回后 10 分钟内补派 {serial} 次（{serial / max(1, launched):.0%}）")
    lines.append(f"## 主 agent 发的消息：{dict(messages)}")
    lines.append("## 后台任务通知（类别、状态）")
    lines += [f"  {count:5d} {kind} {status}" for (kind, status), count in notifications.most_common(12)]
    return lines, dispatches, rejections, launched


# ---------------------------------------------------------------------------------------------------------------------------------
# outcome：每个子 agent 交回之后落没落地、被谁用上
# ---------------------------------------------------------------------------------------------------------------------------------
LINE_MINIMUM = 12                  # 一行去首尾空白后不短于这么多个字才算一条可比对的行
HISTORY_MARGIN_DAYS = 6            # git 历史往 --since 之前多取几天
LANDED_AT, PARTIAL_AT, WRITTEN_MINIMUM = 0.7, 0.3, 10
IN_FLIGHT_MINUTES = 30
REWORK_WORDS = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.IGNORECASE)
NARROW_REWORK_WORDS_FOR_BREAK = re.compile("修|重跑|续做|改名|改号")


def rework_words():
    return NARROW_REWORK_WORDS_FOR_BREAK if BROKEN == "narrow-rework-words" else REWORK_WORDS
USAGE_LIMIT_TAIL = re.compile(r"limit · resets|weekly limit|usage limit")
API_ERROR_TAIL = re.compile(r"API Error|Not logged in|Overloaded")
REPOSITORY_MARKS = ("crates", ".claude", "scripts", "research", "records", "briefs", "litmus")
NOT_REPOSITORY = ("projects", "plans", "todos", "shell-snapshots")   # ~/.claude/ 底下的，不是仓里的 .claude
EVENT_TOOLS = ("Bash", "Read", "Edit", "Write", "MultiEdit", "Agent", "Task", "SendMessage", "Grep", "Glob")
EVENT_TEXT_LIMIT = 30000
PATH_CHARS = r"[A-Za-z0-9_.\-/~@+=%$一-鿿]"   # ASCII 加汉字（目录名里有「甲」「乙」这类）；全角标点、括号、空白不在里面，路径到那里为止
WORK_PATH = re.compile(r"(?:~|/home/[^/\s]+)/\.local/share/singlefs/work/" + PATH_CHARS + r"+|/tmp/claude-\d+/" + PATH_CHARS + r"+|research/prompts/" + PATH_CHARS + r"+")
# 改前的路径写法：只在标点处截断，全角括号与说明文字会粘进路径（弄坏开关 no-path-normalize 退回它）
OLD_WORK_PATH = re.compile(r"(?:~|/home/[^/\s]+)/\.local/share/singlefs/work/[^\s'\"`)>\]，。；、：:,]+|/tmp/claude-\d+/[^\s'\"`)>\]，。；、：:,]+|research/prompts/[^\s'\"`)>\]，。；、：:,]+")
PATH_TOKEN = re.compile(r"[^\s'\"`()<>|;&]*/[^\s'\"`()<>|;&]*")
VARIABLE_ASSIGNMENT = re.compile(r"(?:^|[\s;&(])([A-Za-z_][A-Za-z0-9_]*)=(\"[^\"\n]*\"|'[^'\n]*'|[^\s;&|()]+)")
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_]\w*)\}|([A-Za-z_]\w*))")
WILDCARD_CHARS = re.compile(r"[$*?{]")
NAME_MINIMUM = 6                   # 目录名用作键，至少这么长、带连字符（草稿目录都是这样起名的）；再短的什么都能碰上
FILE_LIKE = re.compile(r"\.[A-Za-z0-9]{1,6}$")
GENERIC_NAMES = {"work", "draft", "tmp", "report", "results", "prompts", "patch", "scratch"}
BUCKET_GLOSS = [
    ("written-landed", "写了、不低于 70% 的行进了提交或工作树"),
    ("written-partial", "写了、30% 到 70%"),
    ("verdict-archived", "三方腿"),
    ("report-used-by-main", "只交报告、交回后主 agent 点到"),
    ("report-used-by-later-sub", "只交报告、只有后来的子 agent 点到"),
    ("report-unreferenced", "只交报告、没人点到"),
    ("written-overwritten", "写了、低于 30%、主 agent 点到过"),
    ("written-unused", "写了、低于 30%、没人点到"),
    ("cut-by-usage-limit", "没交回、用量撞顶"),
    ("cut-by-api-error", "没交回、API 错"),
    ("stopped-salvaged", "没交回、被停掉、后来有人点到"),
    ("stopped-abandoned", "没交回、被停掉、没人点到"),
    ("in-flight", "没交回、还在跑"),
]
# 调度记录「按结局分」那张表的五行各含哪几桶；不在任何一行里的归「其余」
RECORD_TABLE_ROWS = [
    ("写了东西、不低于 70% 的行进了提交或工作树", ("written-landed",)),
    ("写了东西、30% 到 70%", ("written-partial",)),
    ("三方腿", ("verdict-archived",)),
    ("只交报告、交回后主 agent 有动作点到", ("report-used-by-main",)),
    ("看起来没效果的几桶", ("report-unreferenced", "written-overwritten", "written-unused", "report-used-by-later-sub", "stopped-abandoned", "cut-by-usage-limit")),
]


def repository_relative(path, allow_relative=False):
    """路径里最靠外的仓内顶层目录起的相对路径；认不出是仓内文件的回 None。"""
    if not path or not isinstance(path, str) or "\n" in path:
        return None
    parts = path.split("/")
    for index, segment in enumerate(parts):
        if segment in REPOSITORY_MARKS and (index > 0 or allow_relative):
            if segment == ".claude" and index + 1 < len(parts) and parts[index + 1] in NOT_REPOSITORY:
                continue
            return "/".join(parts[index:])
    return None


def significant_lines(text):
    return [line for line in (raw.strip() for raw in (text or "").splitlines()) if len(line) >= LINE_MINIMUM]


def moment_of(raw):
    try:
        return datetime.datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError:
        return None


def shell_words(text):
    """shlex 切词；引号没闭合时读到哪算哪。"""
    lexer = shlex.shlex(text[:60000], posix=True, punctuation_chars=True)
    lexer.whitespace_split = True
    words = []
    try:
        for word in lexer:
            words.append(word)
    except ValueError:
        pass
    return words


HEREDOC = re.compile(r"(?<!<)<<(?!<)-?[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")
OPERATOR_WORD = re.compile(r"^[;&|()<>]+$")
WRITE_HINT = re.compile(r"write_text\(|write_bytes\(|\.write\(|writelines\(|open\([^)]*[\"'][wax]b?\+?[\"']")
SCRIPT_CALL = re.compile(r"(?:^|[\s/'\"])(replace-once|replace-batch|insert-row)\.py(?=[\s'\";|&]|$)")
INSERT_ROW_VALUE_FLAGS = {"--after", "--before", "--after-literal", "--before-literal", "--line", "--absent"}


def heredoc_bodies(command):
    """命令里的每个 heredoc：(同一行里 << 之前的字, << 之后同一行的字, 正文)。"""
    found, position = [], 0
    while True:
        match = HEREDOC.search(command, position)
        if not match:
            return found
        line_start = command.rfind("\n", 0, match.start()) + 1
        line_end = command.find("\n", match.end())
        if line_end < 0:
            return found
        tag = match.group(2)
        body, consumed = [], 0
        for line in command[line_end + 1:].split("\n"):
            consumed += len(line) + 1
            if line.strip() == tag:
                break
            body.append(line)
        found.append((command[line_start:match.start()], command[match.end():line_end], "\n".join(body)))
        position = line_end + 1 + consumed


def heredoc_target(before, after):
    """cat > 文件 <<EOF、tee [-a] 文件 <<EOF、cat <<EOF > 文件 三种写法的目标文件；认不出回 None。"""
    for text, pattern in ((before, r"(?:>>?|\btee\b(?:\s+-\w+)*)\s*(\S+)\s*$"), (after, r"^\s*>>?\s*(\S+)")):
        match = re.search(pattern, text)
        if match:
            return match.group(1).strip("'\"")
    return None


def python_literal_lines(code):
    """python 代码里写文件的那一类：字符串字面量里的行（replace 的旧串里已有的行除外）；没有写文件的动作、或没有指向仓内文件的路径，回空。"""
    if not WRITE_HINT.search(code):
        return []
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")   # 命令里嵌的 python 常带 shell 转义，ast 的 SyntaxWarning 不是要报的东西
            tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    constants, old_lines, targets = [], set(), 0
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            constants.append(node.value)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "replace" and node.args:
            if isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
                old_lines.update(significant_lines(node.args[0].value))
    lines = []
    for value in constants:
        if len(value) < 400 and not re.search(r"\s", value) and repository_relative(value, allow_relative=True):
            targets += 1
            continue
        lines += [line for line in significant_lines(value) if line not in old_lines]
    return lines if targets else []


def python_code_of(command, state):
    """命令里喂给 python 的代码：heredoc 正文、-c 的参数、先 heredoc 写成再执行的脚本。"""
    codes = []
    for before, _, body in heredoc_bodies(command):
        if re.search(r"\bpython3?\b", before):
            codes.append(body)
    for match in re.finditer(r"\bpython3?\s+(?:-[A-Za-z]\s+)*-c\s", command):
        words = shell_words(command[match.end() - 3:])
        if len(words) >= 2 and words[0] == "-c":
            codes.append(words[1])
    for match in re.finditer(r"\bpython3?\s+(?:-[A-Za-z]\s+)*(\S+\.py)\b", command):
        if match.group(1) in state["files"]:
            codes.append(state["files"][match.group(1)])
    return codes


def script_writes(command, state):
    """replace-once.py / replace-batch.py / insert-row.py 的实写：[(目标文件, 新增行)]。"""
    writes = []
    for match in SCRIPT_CALL.finditer(command):
        name, flags, arguments, values = match.group(1), set(), [], {}
        words, position = shell_words(command[match.end():]), 0
        while position < len(words):
            word = words[position]
            position += 1
            if OPERATOR_WORD.match(word) or re.fullmatch(r"python3?", word):
                break   # 下一条命令开始了
            if name == "insert-row" and word in INSERT_ROW_VALUE_FLAGS:
                values.setdefault(word, words[position] if position < len(words) else "")
                position += 1
            elif word.startswith("--"):
                flags.add(word)
            else:
                arguments.append(word)
        if name == "replace-once" and len(arguments) >= 3:
            old_lines = set(significant_lines(arguments[1]))
            writes.append((arguments[0], [line for line in significant_lines(arguments[2]) if line not in old_lines]))
        elif name == "insert-row" and "--dry-run" not in flags and arguments and "--line" in values:
            writes.append((arguments[0], significant_lines(values["--line"])))
        elif name == "replace-batch" and "--dry-run" not in flags and arguments:
            spec_text = state["files"].get(arguments[0])
            if spec_text is None:
                try:
                    with open(os.path.expanduser(arguments[0]), errors="replace") as handle:
                        spec_text = handle.read(5_000_000)
                except OSError:
                    spec_text = ""
            try:
                spec = json.loads(spec_text)
            except ValueError:
                continue
            for item in spec if isinstance(spec, list) else []:
                if isinstance(item, dict):
                    old_lines = set(significant_lines(item.get("old")))
                    writes.append((item.get("file"), [line for line in significant_lines(item.get("new")) if line not in old_lines]))
    return writes


def bash_written_lines(command, state):
    """一条成功的 Bash 命令写进仓内文件的新增行：(确定写进去的行, 只作加分的行)。heredoc 写文件和三个定点替换脚本的新串是确定写进去的；
    python 代码里的字符串字面量只是写了什么的上限（模板、命令、说明文字也在里面），只在仓里找得到时才算，找不到不扣分。
    state["files"] 记这个子 agent 用 heredoc 写出过的文件（后面执行脚本、读规格要用）。"""
    if BROKEN == "no-bash-writes":
        return [], []
    lines, credit = [], []
    for before, after, body in heredoc_bodies(command):
        target = heredoc_target(before, after)
        if target:
            state["files"][target] = body
            if repository_relative(target, allow_relative=True):
                lines += significant_lines(body)
    for code in python_code_of(command, state):
        credit += python_literal_lines(code)
    for target, new_lines in script_writes(command, state):
        if repository_relative(target, allow_relative=True):
            lines += new_lines
    return lines, credit


def written_lines(uses, errors):
    """一个子 agent 全部成功的写的新增行：(确定写进去的行, 只作加分的行)；同一行写几次算几次，与 usage 的逐次口径一致。"""
    state, lines, credit = {"files": {}}, [], []
    for tool_id, (name, tool_input) in uses.items():
        if tool_id in errors:
            continue
        if name == "Edit":
            if repository_relative(tool_input.get("file_path")):
                old_lines = set(line.strip() for line in (tool_input.get("old_string") or "").splitlines())
                lines += [line for line in significant_lines(tool_input.get("new_string")) if line not in old_lines]
        elif name == "MultiEdit":
            if repository_relative(tool_input.get("file_path")):
                for edit in tool_input.get("edits") or []:
                    old_lines = set(line.strip() for line in (edit.get("old_string") or "").splitlines())
                    lines += [line for line in significant_lines(edit.get("new_string")) if line not in old_lines]
        elif name == "Write":
            if repository_relative(tool_input.get("file_path")):
                lines += significant_lines(tool_input.get("content"))
        elif name == "Bash":
            strict, extra = bash_written_lines(tool_input.get("command") or "", state)
            lines += strict
            credit += extra
    return lines, credit


def event_text(name, tool_input):
    if name == "Bash":
        text = tool_input.get("command") or ""
    elif name in ("Read", "Grep", "Glob"):
        text = "\n".join(str(tool_input.get(key) or "") for key in ("file_path", "path", "pattern"))
    elif name in ("Edit", "Write"):
        text = (tool_input.get("file_path") or "") + "\n" + (tool_input.get("new_string") or tool_input.get("content") or "")
    elif name == "MultiEdit":
        text = (tool_input.get("file_path") or "") + "\n" + "\n".join(edit.get("new_string") or "" for edit in tool_input.get("edits") or [])
    elif name in ("Agent", "Task"):
        text = tool_input.get("prompt") or ""
    else:
        text = json.dumps(tool_input, ensure_ascii=False)
    return text[:EVENT_TEXT_LIMIT]


def raw_timestamp(line):
    index = line.rfind('"timestamp":"')
    return line[index + 13:index + 13 + 40].split('"', 1)[0] if index >= 0 else None


def scan_subagent(path):
    """一个子 agent 的会话记录：开始与结束、交回正文、最后一句、花费、写入的新增行、工具调用的文本（供引用比对）。
    只解析 assistant 行、报错的 user 行、开头的 user 行与交回快筛留下的行。起点（首条记录的时刻）与交回（最近一次送达的那一次的正文）照 subagent_transcript.py。"""
    calls, uses, errors, events = {}, {}, set(), []
    end_raw = prompt = None
    last_text = ""
    handbacks = transcripts.HandbackTracker()
    with open(path, errors="replace") as handle:
        for number, line in enumerate(handle):
            raw = raw_timestamp(line)
            if raw:
                end_raw = raw
            is_assistant = '"type":"assistant"' in line
            is_handback_line = handbacks.wants(line)
            if not is_assistant and not is_handback_line and not (number == 0 or '"is_error":true' in line or (prompt is None and '"type":"user"' in line)):
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if is_handback_line:
                handbacks.feed(record)
            message = record.get("message") if isinstance(record.get("message"), dict) else {}
            content = message.get("content")
            if record.get("type") == "user":
                if prompt is None:
                    prompt = content if isinstance(content, str) else " ".join(block.get("text", "") for block in content or [] if isinstance(block, dict))
                for block in content if isinstance(content, list) else []:
                    if isinstance(block, dict) and block.get("type") == "tool_result" and block.get("is_error"):
                        errors.add(block.get("tool_use_id"))
                continue
            if not is_assistant:
                continue
            if message.get("usage"):
                calls[usage_key(message, record)] = message["usage"]
            stamp = None
            for block in content if isinstance(content, list) else []:
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "text":
                    last_text = block.get("text") or last_text
                elif block.get("type") == "tool_use":
                    name, tool_input = block.get("name"), block.get("input") or {}
                    if name == transcripts.HANDBACK_TOOL:   # 送没送达、正文是哪一次的，由上面的 handbacks 认
                        continue
                    if block.get("id") not in uses and name in ("Edit", "Write", "MultiEdit", "Bash"):
                        uses[block.get("id")] = (name, tool_input)
                    if name in EVENT_TOOLS:
                        if stamp is None:
                            moment = moment_of(raw or "")
                            stamp = moment.timestamp() if moment else 0.0
                        tokens = event_tokens(event_text(name, tool_input))
                        if tokens:
                            events.append((stamp, False, tokens))
    written = written_lines(uses, errors)
    draft = re.search(r"草稿目录[：:]\s*(" + PATH_CHARS + "+)", prompt or "")
    handback = handbacks.delivered["message"] if handbacks.delivered else None
    return {"start": transcripts.first_record_moment(path), "end": moment_of(end_raw or ""), "handback": handback, "last_text": last_text[-3000:],
            "totals": usage_totals(list(calls.values())), "lines": written[0], "credit_lines": written[1], "events": events,
            "draft": normalize_path(draft.group(1)) if draft else None}


def scan_main_session(path):
    """主会话的工具调用文本（供引用比对）。"""
    events = []
    with open(path, errors="replace") as handle:
        for line in handle:
            if '"type":"assistant"' not in line or '"tool_use"' not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            message = record.get("message") if isinstance(record.get("message"), dict) else {}
            moment = moment_of(record.get("timestamp") or "")
            for block in message.get("content") if isinstance(message.get("content"), list) else []:
                if isinstance(block, dict) and block.get("type") == "tool_use" and block.get("name") in EVENT_TOOLS and moment:
                    tokens = event_tokens(event_text(block["name"], block.get("input") or {}))
                    if tokens:
                        events.append((moment.timestamp(), True, tokens))
    return events


def normalize_path(path):
    path = re.sub(r"^/home/[^/]+/", "~/", path).replace("$HOME/", "~/").replace("${HOME}/", "~/")
    path = re.sub(r"(\.[A-Za-z0-9]{1,6})[一-鿿].*$", r"\1", path)   # 扩展名后面直接粘着的说明文字
    return path.rstrip(".,=/")


def handback_paths(text):
    """交回正文里点名的产物路径：取到空白、引号、括号、全角标点为止，扩展名后面直接粘着的汉字说明也去掉；弄坏开关退回只在标点处截断。"""
    pattern = OLD_WORK_PATH if BROKEN == "no-path-normalize" else WORK_PATH
    return sorted(set(normalize_path(match.group(0)) for match in pattern.finditer(text or "")))


def path_directory_name(path):
    """路径所在的那个具名目录：工作根（或 /tmp/claude-N/）底下直接的一层，且路径恰好是这一层目录本身或它里面的一个文件；更深的返回 None
    （同一个父目录底下分给几个子 agent 各一个子目录时，父目录名是共用的，认不出是谁的）。"""
    parts = path.split("/")
    if path.startswith("~/.local/share/singlefs/work/") and 6 <= len(parts) <= 7:
        return parts[5]
    if path.startswith("/tmp/claude-") and 4 <= len(parts) <= 5:
        return parts[3]
    return None


def expand_variables(text):
    """同一条命令里赋过值的变量先展开（$NAME、${NAME}，$HOME 记成 ~）；没赋过的原样留着，当通配对。"""
    values = {"HOME": "~"}
    for match in VARIABLE_ASSIGNMENT.finditer(text):
        values[match.group(1)] = match.group(2).strip("\"'")
    for _ in range(3):
        expanded = VARIABLE_REFERENCE.sub(lambda match: values.get(match.group(1) or match.group(2), match.group(0)), text)
        if expanded == text:
            break
        text = expanded
    return text


def wildcard_regex(component):
    """路径的一段里带变量、星号、问号、花括号的，转成正则；字面字符不足 4 个的（什么都能对上）回 None。"""
    pieces, literal, index = [], 0, 0
    while index < len(component):
        char = component[index]
        if char == "$":
            match = re.match(r"\$(?:\{[^}]*\}|[A-Za-z_]\w*)", component[index:])
            if match:
                pieces.append(".+")
                index += match.end()
                continue
        if char == "*":
            pieces.append(".*")
        elif char == "?":
            pieces.append(".")
        elif char == "{":
            end = component.find("}", index)
            if end < 0:
                return None
            body = component[index + 1:end]
            if "," in body:
                alternatives = body.split(",")
                pieces.append("(?:" + "|".join(re.escape(alternative) for alternative in alternatives) + ")")
                literal += min(len(alternative) for alternative in alternatives)
            else:
                pieces.append(".+")
            index = end + 1
            continue
        else:
            pieces.append(re.escape(char))
            literal += 1
        index += 1
    return re.compile("".join(pieces)) if literal >= 4 else None


def event_tokens(text):
    """一条工具调用文本里和引用有关的记号，各带一个字符的标签：i 是 agent 号，p 是工作目录一类的路径（连同各级上层目录），
    n 是目录名一类的路径段（带连字符、不像文件名），w 是带变量、星号、问号、花括号的这样一段。
    弄坏开关 no-path-normalize 时只剩 agent 号与旧写法认出的路径，变量不展开、路径段不取。"""
    normalize = BROKEN != "no-path-normalize"
    tokens = set()
    if normalize and "$" in text:
        text = expand_variables(text)
    tokens.update("i:" + match.group(0) for match in AGENT_ID.finditer(text))
    for match in (WORK_PATH if normalize else OLD_WORK_PATH).finditer(text):
        parts = normalize_path(match.group(0)).split("/")
        tokens.update("p:" + "/".join(parts[:length]) for length in range(len(parts), 2, -1))
    if normalize:
        for token in PATH_TOKEN.findall(text):
            for component in token.split("/"):
                component = component.rstrip(".,:;")
                if len(component) < NAME_MINIMUM or "-" not in component or FILE_LIKE.search(component):
                    continue
                tokens.add(("w:" if WILDCARD_CHARS.search(component) else "n:") + component)
    return tokens


def index_references(events, wanted_names, wanted_paths, wanted_ids):
    """所有工具调用的记号里，点到哪些目录名、路径、agent 号：{键: (主会话的时刻们, 子 agent 的时刻们)}，时刻升序。"""
    hits = collections.defaultdict(lambda: ([], []))
    wildcard_names = {}
    for stamp, is_main, tokens in events:
        keys = set()
        for token in tokens:
            tag, body = token[0], token[2:]
            if tag == "i" and body in wanted_ids:
                keys.add(("id", body))
            elif tag == "p" and body in wanted_paths:
                keys.add(("path", body))
            elif tag == "n" and body in wanted_names:
                keys.add(("name", body))
            elif tag == "w":
                if body not in wildcard_names:
                    pattern = wildcard_regex(body)
                    wildcard_names[body] = [name for name in wanted_names if pattern.fullmatch(name)] if pattern else []
                keys.update(("name", name) for name in wildcard_names[body])
        for key in keys:
            hits[key][0 if is_main else 1].append(stamp)
    for main_stamps, sub_stamps in hits.values():
        main_stamps.sort()
        sub_stamps.sort()
    return hits


def referenced_between(stamps, begin, limit):
    index = bisect.bisect_right(stamps, begin)
    return index < len(stamps) and stamps[index] < limit


def git_output(arguments, environment):
    result = subprocess.run(["git", *arguments], capture_output=True, env=environment)
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(arguments[:3])} 退出 {result.returncode}：{result.stderr.decode('utf-8', 'replace').strip()[:300]}")
    return result.stdout


SHARED = {}   # 要比对的行（建在 fork 工作进程之前，子进程继承同一份，哈希种子也一致）


def line_set(lines):
    """要比对的行：按 utf-8 字节查（快路径），和按字符串哈希查（行首尾是非 ASCII 空白时的慢路径）。"""
    by_bytes = {line.encode("utf-8"): hash(line) for line in lines}
    return by_bytes, set(by_bytes.values())


def matching_hash(stripped):
    """一行（字节、已去首尾 ASCII 空白）若是要比对的行之一，回它的哈希，否则 None。"""
    if len(stripped) < LINE_MINIMUM:
        return None
    by_bytes, hashes = SHARED["lines"]
    found = by_bytes.get(stripped)
    if found is not None:
        return found
    if stripped[0] >= 0x80 or stripped[-1] >= 0x80:
        line = stripped.decode("utf-8", "replace").strip()
        if len(line) >= LINE_MINIMUM and hash(line) in hashes:
            return hash(line)
    return None


def file_matches(tree, names):
    """一棵工作树里这几个文件的行里，要比对的有哪些（二进制和超大的跳过）。"""
    found = set()
    for name in names:
        path = os.path.join(tree, name.decode("utf-8", "replace"))
        try:
            if os.path.getsize(path) > 200_000_000:
                continue
            with open(path, "rb") as handle:
                if b"\0" in handle.read(4096):
                    continue
                handle.seek(0)
                for raw in handle:
                    match = matching_hash(raw.strip())
                    if match is not None:
                        found.add(match)
        except OSError:
            continue
    return found


def history_and_worktree_lines(repository, since, lines, workers):
    """lines 里哪些在 git 历史加进的行或某棵工作树的文件里出现过：返回（找到的行的哈希集合, 工作树个数）。git 出错直接抛。"""
    environment = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0")
    SHARED["lines"] = line_set(lines)
    found = set()
    begin = (since - datetime.timedelta(days=HISTORY_MARGIN_DAYS)).date().isoformat()
    listing = git_output(["-C", repository, "worktree", "list", "--porcelain"], environment).decode("utf-8", "replace")
    trees = [line[len("worktree "):] for line in listing.splitlines() if line.startswith("worktree ")]
    chunks = []
    for tree in trees:
        if os.path.isdir(tree):
            names = [name for name in git_output(["-C", tree, "ls-files", "-co", "--exclude-standard", "-z"], environment).split(b"\0") if name]
            chunks += [(tree, names[start:start + 300]) for start in range(0, len(names), 300)]
    pool = concurrent.futures.ProcessPoolExecutor(max_workers=workers, mp_context=multiprocessing.get_context("fork")) if workers > 1 else None
    try:
        futures = [pool.submit(file_matches, tree, names) for tree, names in chunks] if pool else []
        with tempfile.TemporaryFile() as log_errors:
            log = subprocess.Popen(["git", "-C", repository, "log", "-p", "-U0", "--format=", "--no-color", "--no-ext-diff", "--no-textconv",
                                    f"--since={begin}", "--branches", "HEAD"], stdout=subprocess.PIPE, stderr=log_errors, env=environment)
            for raw in log.stdout:
                if raw[:1] == b"+" and raw[:3] != b"+++":
                    match = matching_hash(raw[1:].strip())
                    if match is not None:
                        found.add(match)
            if log.wait() != 0:
                log_errors.seek(0)
                raise RuntimeError(f"git log 退出 {log.returncode}：{log_errors.read().decode('utf-8', 'replace').strip()[:300]}")
        for future in futures:
            found |= future.result()
        if not pool:
            for tree, names in chunks:
                found |= file_matches(tree, names)
    finally:
        if pool:
            pool.shutdown()
    return found, len(trees)


def cut_reason(last_text):
    tail = last_text[-600:]
    if USAGE_LIMIT_TAIL.search(tail):
        return "用量撞顶"
    if API_ERROR_TAIL.search(tail):
        return "API 错"
    return "停掉"


def outcome_bucket(agent, used_main, used_any, now):
    if agent["handed_back"]:
        if agent["type"].startswith("three-way"):
            return "verdict-archived"
        if agent["written"] >= WRITTEN_MINIMUM:
            share = agent["landed"] / agent["written"]
            if share >= LANDED_AT:
                return "written-landed"
            if share >= PARTIAL_AT:
                return "written-partial"
            return "written-overwritten" if used_main else "written-unused"
        if used_main:
            return "report-used-by-main"
        return "report-used-by-later-sub" if used_any else "report-unreferenced"
    if agent["end"] and (now - agent["end"]).total_seconds() < IN_FLIGHT_MINUTES * 60:
        return "in-flight"
    if agent["cut"] == "用量撞顶":
        return "cut-by-usage-limit"
    if agent["cut"] == "API 错":
        return "cut-by-api-error"
    return "stopped-salvaged" if used_any else "stopped-abandoned"


def scan_job(kind, path):
    return scan_subagent(path) if kind == "sub" else scan_main_session(path)


def collect_outcome(root, since, repository, now, workers=1, until=None):
    """起点（首条记录的时刻）落在 [since, until) 里的每个子 agent 一行（until 为 None 不设上界）：
    [{agent, type, description, start, handed_back, cut, bucket, cost, rework, ...}]，与 git 历史、各工作树比对过。
    会话记录按文件分给 workers 个工作进程读（workers 为 1 时在本进程里顺序读）。"""
    subagents = []
    for meta in sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.meta.json"))):
        transcript = meta.replace(".meta.json", ".jsonl")
        if os.path.exists(transcript) and transcripts.started_between(transcript, since, until):
            subagents.append((meta, transcript))
    sessions = [path for path in sorted(glob.glob(os.path.join(root, "*.jsonl"))) if recent(path, since)]
    jobs = [("sub", transcript) for _, transcript in subagents] + [("main", path) for path in sessions]
    if workers > 1 and len(jobs) > 1:
        with concurrent.futures.ProcessPoolExecutor(max_workers=workers, mp_context=multiprocessing.get_context("fork")) as pool:
            scans = [future.result() for future in [pool.submit(scan_job, *job) for job in jobs]]
    else:
        scans = [scan_job(*job) for job in jobs]
    agents, events = [], []
    for (meta, transcript), scan in zip(subagents, scans):
        events += scan["events"]
        try:
            with open(meta) as handle:
                described = json.load(handle)
        except (OSError, ValueError):
            described = {}
        handback_text = scan["handback"]
        paths = handback_paths(handback_text if handback_text else scan["last_text"])
        names = {name for name in [scan["draft"].split("/")[-1] if scan["draft"] else None] + [path_directory_name(path) for path in paths]
                 if name and len(name) >= NAME_MINIMUM and "-" in name and name not in GENERIC_NAMES}
        if scan["draft"]:
            paths = sorted(set(paths) | {scan["draft"]})
        description = re.sub(r"\s+", " ", described.get("description") or "")
        agents.append({"agent": os.path.basename(transcript)[6:-6], "type": described.get("agentType") or "?", "description": description,
                       "start": scan["start"], "end": scan["end"], "handed_back": handback_text is not None,
                       "cut": "-" if handback_text is not None else cut_reason(scan["last_text"]),
                       "cost": folded(scan["totals"]), "rework": bool(rework_words().search(description)),
                       "lines": scan["lines"], "credit_lines": scan["credit_lines"], "written": len(scan["lines"]), "landed": 0, "paths": paths, "names": names})
    for scan in scans[len(subagents):]:
        events += scan
    if any(agent["lines"] or agent["credit_lines"] for agent in agents):
        found, _ = history_and_worktree_lines(repository, since, {line for agent in agents for line in agent["lines"] + agent["credit_lines"]}, workers)
        for agent in agents:
            credited = sum(1 for line in agent["credit_lines"] if hash(line) in found)
            agent["landed"] = sum(1 for line in agent["lines"] if hash(line) in found) + credited
            agent["written"] = len(agent["lines"]) + credited
    directory_owners = collections.Counter(directory for agent in agents for directory in {path.rsplit("/", 1)[0] for path in agent["paths"] if path.count("/") >= 6})
    for agent in agents:   # 只有这一个子 agent 的产物在里面的目录，点到目录本身也算点到它
        agent["paths"] = sorted(set(agent["paths"]) | {path.rsplit("/", 1)[0] for path in agent["paths"] if path.count("/") >= 6 and directory_owners[path.rsplit("/", 1)[0]] == 1})
    wanted_names = {name for agent in agents for name in agent["names"]}
    wanted_paths = {path for agent in agents for path in agent["paths"]}
    wanted_ids = {agent["agent"] for agent in agents}
    hits = index_references(events, wanted_names, wanted_paths, wanted_ids)
    keys_of = lambda agent: [("id", agent["agent"])] + [("name", name) for name in agent["names"]] + [("path", path) for path in agent["paths"]]
    starts = collections.defaultdict(list)
    for agent in agents:
        for key in keys_of(agent):
            starts[key].append(agent["start"].timestamp())
    for agent in agents:
        used_main = used_sub = False
        begin = agent["start"].timestamp()
        end = agent["end"].timestamp() if agent["end"] else begin
        for key in keys_of(agent):
            later = [stamp for stamp in starts[key] if stamp > begin] if key[0] == "name" else []
            limit = min(later) if later else float("inf")   # 目录名是共用的（重派沿用同一个草稿目录）：下一个共用者开工之后，按名字点到的算不清是谁的
            main_stamps, sub_stamps = hits[key] if key in hits else ([], [])
            used_main = used_main or referenced_between(main_stamps, end, limit)
            used_sub = used_sub or referenced_between(sub_stamps, end, limit)
        agent["bucket"] = outcome_bucket(agent, used_main, used_main or used_sub, now)
        if agent["bucket"] == "in-flight":
            agent["cut"] = "还在跑"
    agents.sort(key=lambda agent: (agent["start"], agent["agent"]))
    return agents


def outcome_summary(agents):
    """汇总：按桶、按类型 × 桶、返工字样、没交回的按截断原因。"""
    total_cost = sum(agent["cost"] for agent in agents) or 1
    by_bucket = collections.defaultdict(lambda: [0, 0.0])
    by_type_bucket = collections.defaultdict(lambda: [0, 0.0])
    for agent in agents:
        for table, key in ((by_bucket, agent["bucket"]), (by_type_bucket, (agent["type"], agent["bucket"]))):
            table[key][0] += 1
            table[key][1] += agent["cost"]
    rework = [agent for agent in agents if agent["rework"]]
    cuts = collections.Counter(agent["cut"] for agent in agents if not agent["handed_back"])
    return {"total_cost": total_cost, "by_bucket": by_bucket, "by_type_bucket": by_type_bucket, "rework": (len(rework), sum(agent["cost"] for agent in rework)), "cuts": cuts}


def window_text(since, until_day=None):
    """账的窗口：「<since> 起」，给了 --until 再接「、到 <until> 为止」（until_day 是 --until 写的那一天，含当天）。"""
    return f"{since.date()} 起" + (f"、到 {until_day} 为止" if until_day else "")


def outcome_lines(agents, since, show_rows, until_day=None):
    lines = []
    if show_rows:
        lines.append(f"## 每个子 agent 一行（{window_text(since, until_day)}，{len(agents)} 个；折算花费单位百万）")
        lines.append("agent\t类型\t开始日期\t交回\t截断原因\t结局桶\t折算花费\t返工字样\t描述")
        for agent in agents:
            lines.append("\t".join([agent["agent"], agent["type"], agent["start"].date().isoformat(), "交回" if agent["handed_back"] else "没交回", agent["cut"], agent["bucket"],
                                    f"{agent['cost'] / 1e6:.2f}", "返工字样" if agent["rework"] else "-", agent["description"][:80]]))
        lines.append("")
    summary = outcome_summary(agents)
    total_cost = summary["total_cost"]
    lines.append(f"## 按结局桶（{window_text(since, until_day)}，{len(agents)} 个，折算合计 {total_cost / 1e6:.2f} 百万）")
    lines.append(f"{'桶':26}{'个数':>6}{'花费占比':>9}  说明")
    for bucket, gloss in BUCKET_GLOSS:
        count, cost = summary["by_bucket"].get(bucket, [0, 0.0])
        lines.append(f"{bucket:26}{count:>6}{cost / total_cost:>9.1%}  {gloss}")
    lines.append("")
    lines.append("## 并成调度记录「按结局分」那张表的五行")
    lines.append(f"{'行':44}{'个数':>6}{'花费占比':>9}")
    covered = set()
    for label, buckets in RECORD_TABLE_ROWS:
        covered.update(buckets)
        count = sum(summary["by_bucket"].get(bucket, [0, 0.0])[0] for bucket in buckets)
        cost = sum(summary["by_bucket"].get(bucket, [0, 0.0])[1] for bucket in buckets)
        lines.append(f"{label:40}{count:>6}{cost / total_cost:>9.1%}")
    rest = [bucket for bucket, _ in BUCKET_GLOSS if bucket not in covered]
    count = sum(summary["by_bucket"].get(bucket, [0, 0.0])[0] for bucket in rest)
    cost = sum(summary["by_bucket"].get(bucket, [0, 0.0])[1] for bucket in rest)
    lines.append(f"{'其余（' + '、'.join(rest) + '）':40}{count:>6}{cost / total_cost:>9.1%}")
    lines.append("")
    lines.append("## 按类型 × 结局桶（个数，花费占全部的比例）")
    for (agent_type, bucket), (count, cost) in sorted(summary["by_type_bucket"].items(), key=lambda item: (item[0][0], -item[1][0])):
        lines.append(f"  {agent_type:26}{bucket:26}{count:>6}{cost / total_cost:>9.1%}")
    lines.append("")
    rework_count, rework_cost = summary["rework"]
    lines.append(f"## 描述带返工字样（REWORK_WORDS，词表在文件头）的派发：{rework_count} 个，占个数 {rework_count / max(1, len(agents)):.1%}，占花费 {rework_cost / total_cost:.1%}")
    cuts = summary["cuts"]
    lines.append(f"## 没交回的 {sum(cuts.values())} 个，按截断原因：" + "、".join(f"{reason} {cuts[reason]}" for reason in ("用量撞顶", "API 错", "停掉", "还在跑") if cuts.get(reason)))
    return lines


def write_json_lines(path, items):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        for item in items:
            handle.write(json.dumps(item, ensure_ascii=False, separators=(",", ":")) + "\n")   # 与 Claude Code 写的会话记录同一种紧凑写法：outcome 按子串先筛行


def build_outcome_fixture(work):
    """造一个假会话目录与一个带历史、另挂了一棵工作树的假仓；每个结局桶至少一个答案已知的子 agent。返回 (会话目录, 仓, 此刻, {agent 号: 预期的桶})。"""
    root = os.path.join(work, "projects")
    session = os.path.join(root, "s2")
    os.makedirs(os.path.join(session, "subagents"))
    repository, linked_tree = os.path.join(work, "repository"), os.path.join(work, "linked-tree")
    home = "/home/tester/.local/share/singlefs/work"
    base = datetime.datetime(2026, 10, 5, 0, 0, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
    stamp = lambda minutes: (base + datetime.timedelta(minutes=minutes)).isoformat()
    counter = [0]

    def numbered(label, count):
        return [f"let {label}_{number} = compute_{label}({number});" for number in range(count)]

    def assistant(minutes, blocks, usage=None):
        counter[0] += 1
        message = {"id": f"msg-{counter[0]}", "role": "assistant", "content": blocks}
        if usage:
            message["usage"] = usage
        return {"type": "assistant", "uuid": f"uuid-{counter[0]}", "timestamp": stamp(minutes), "message": message}

    def use(tool_id, name, tool_input):
        return {"type": "tool_use", "id": tool_id, "name": name, "input": tool_input}

    def agent_records(start, steps, handback=None, last_text="做到一半", prompt="派发提示", failed=()):
        """steps: [(分钟, [块…])]；failed 里的 tool_use id 记一条报错的 tool_result。用量放在末条，每个子 agent 折算 1000 + 5 × 100 = 1500。"""
        usage = {"input_tokens": 1000, "output_tokens": 100, "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0}
        out = [{"type": "user", "timestamp": stamp(start), "message": {"role": "user", "content": prompt}}]
        for minutes, blocks in steps:
            out.append(assistant(minutes, blocks))
        for tool_id in failed:
            out.append({"type": "user", "timestamp": stamp(start + 1), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": tool_id, "is_error": True, "content": "failed"}]}})
        end = (steps[-1][0] if steps else start) + 1
        if handback is not None:   # 交回要送达才算（subagent_transcript.py）：调用之后跟一条成功的 tool_result
            call_id = f"handback-{counter[0]}"
            out.append(assistant(end, [use(call_id, "SubagentHandback", {"message": handback})], usage))
            out.append({"type": "user", "timestamp": stamp(end), "message": {"role": "user", "content": [
                {"type": "tool_result", "tool_use_id": call_id, "content": [{"type": "text", "text": "{\"success\":true,\"message\":\"Report delivered to your caller.\"}"}]}]}})
        else:
            out.append(assistant(end, [{"type": "text", "text": last_text}], usage))
        return out

    identifiers = {}

    def add_agent(number, agent_type, description, records_list):
        identifier = f"a{number:016x}"
        write_json_lines(os.path.join(session, "subagents", f"agent-{identifier}.jsonl"), records_list)
        with open(os.path.join(session, "subagents", f"agent-{identifier}.meta.json"), "w", encoding="utf-8") as handle:
            json.dump({"agentType": agent_type, "description": description}, handle)
        identifiers[number] = identifier

    spec_lines, tool_lines, dry_lines = numbered("batch", 3), numbered("once", 4), numbered("dryrun", 8)
    insert_lines = [f"insert the row number {number} here now" for number in range(3)]
    git_environment = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.invalid",
                           GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.invalid")

    def git(*arguments):
        subprocess.run(["git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false", *arguments], cwd=repository, env=git_environment, check=True, capture_output=True)

    def put(directory, name, lines):
        os.makedirs(os.path.join(directory, "crates"), exist_ok=True)
        with open(os.path.join(directory, "crates", name), "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")

    # 假仓：历史里加过又删掉的行、留在提交里的行、只在主工作树没提交的文件、只在挂出来的另一棵工作树里没提交的文件
    os.makedirs(repository)
    git("init", "-q")
    put(repository, "base.rs", ["fn base() {}"])
    git("add", "-A")
    git("commit", "-q", "-m", "base")
    put(repository, "history_only.rs", numbered("edit", 10))
    git("add", "-A")
    git("commit", "-q", "-m", "history only")
    git("rm", "-q", "crates/history_only.rs")
    git("commit", "-q", "-m", "remove again")
    put(repository, "tools.rs", tool_lines + insert_lines + spec_lines)
    slow_lines = [chr(0x3000) + f"全角空格开头的比对行 x{number} 用来走慢路径" for number in range(3)]   # 行首是非 ASCII 空白：字节快路径对不上，要走解码去空白的慢路径
    put(repository, "partial.rs", numbered("partial", 2) + slow_lines)
    git("add", "-A")
    git("commit", "-q", "-m", "tools and partial")
    put(repository, "python_written.rs", numbered("python", 11))
    git("worktree", "add", "-q", "-b", "linked", linked_tree)
    put(linked_tree, "heredoc_written.rs", numbered("heredoc", 11))

    def report_handback(directory, tail="（298 行），sha256 略"):
        return f"报告：{home}/{directory}/report.md{tail}"

    def edit_to(name, lines):
        return {"file_path": f"{repository}/crates/{name}", "old_string": "", "new_string": "\n".join(lines)}

    # 写入落地：Edit 10 行在历史里（另有一次报错的 Write 不算）
    add_agent(1, "implementation-writer", "改共享代码", agent_records(0, [
        (1, [use("edit-1", "Edit", edit_to("history_only.rs", numbered("edit", 10)))]),
        (2, [use("write-1", "Write", {"file_path": f"{repository}/crates/nowhere.rs", "content": "\n".join(numbered("nowhere", 10))})])],
        handback="已改完", failed=("write-1",)))
    # Bash heredoc 写进挂出来的另一棵工作树
    heredoc_command = f"cat > {linked_tree}/crates/heredoc_written.rs <<'EOF'\n" + "\n".join(numbered("heredoc", 11)) + "\nEOF"
    add_agent(2, "implementation-writer", "Bash heredoc 写", agent_records(0, [(1, [use("bash-2", "Bash", {"command": heredoc_command})])], handback="已写"))
    # python 写文件：字符串字面量里的 11 行，在主工作树里
    python_command = ("python3 - <<'PY'\nfrom pathlib import Path\nPath(\"" + repository + "/crates/python_written.rs\").write_text(\"\"\"" + "\n".join(numbered("python", 11)) + "\n\"\"\")\nPY")
    add_agent(3, "implementation-writer", "python 写", agent_records(0, [(1, [use("bash-3", "Bash", {"command": python_command})])], handback="已写"))
    # 三个定点替换脚本：replace-once 4 行 + insert-row 3 行 + replace-batch 3 行都在历史里；--dry-run 的规格里另有 8 行没落地，算进去就掉到 partial
    spec = json.dumps([{"file": f"{repository}/crates/tools.rs", "old": "x", "new": "\n".join(spec_lines)}])
    dry_spec = json.dumps([{"file": f"{repository}/crates/tools.rs", "old": "x", "new": "\n".join(dry_lines)}])
    insert_commands = "".join(f"python3 scripts/kb/insert-row.py {repository}/crates/tools.rs --after-literal anchor --line '{line}'\n" for line in insert_lines)
    add_agent(4, "tooling-writer", "三个定点替换脚本", agent_records(0, [
        (1, [use("bash-4a", "Bash", {"command": f"python3 scripts/kb/replace-once.py {repository}/crates/tools.rs 'old text here' '" + "\n".join(tool_lines) + "'"})]),
        (2, [use("bash-4b", "Bash", {"command": insert_commands})]),
        (3, [use("bash-4c", "Bash", {"command": f"cat > /scratch/spec.json <<'EOF'\n{spec}\nEOF\npython3 scripts/kb/replace-batch.py /scratch/spec.json"})]),
        (4, [use("bash-4d", "Bash", {"command": f"cat > /scratch/dry.json <<'EOF'\n{dry_spec}\nEOF\npython3 scripts/kb/replace-batch.py --dry-run /scratch/dry.json"})])], handback="已改"))
    add_agent(5, "implementation-writer", "半截落地", agent_records(0, [(1, [use("edit-5", "Edit", edit_to("partial.rs", numbered("partial", 2) + slow_lines + numbered("gone", 5)))])], handback="已改"))
    add_agent(6, "implementation-writer", "写了但没落地且被点到", agent_records(0, [(1, [use("edit-6", "Edit", edit_to("lost.rs", numbered("lost", 10)))])],
                                                                          handback=report_handback("overwritten-draft"), prompt=f"草稿目录：{home}/overwritten-draft\n"))
    add_agent(7, "implementation-writer", "写了但没落地且没人点到", agent_records(0, [(1, [use("edit-7", "Edit", edit_to("lost2.rs", numbered("lost2", 10)))])], handback="已改"))
    add_agent(8, "three-way-forward", "三方腿", agent_records(0, [(1, [])], handback="腿报告"))
    # 引用：交回里的路径后面粘着全角括号与说明（主会话直接读它）；主会话用循环、花括号拼路径
    add_agent(9, "investigator", "查一件事", agent_records(0, [(1, [])], handback=f"报告：{home}/report-read-by-main.md（298 行），sha256 略"))   # 平铺的文件：没有目录键可退，路径粘着全角括号就对不上
    add_agent(10, "investigator", "循环拼路径", agent_records(0, [(1, [])], handback=report_handback("looped-draft-3", "。"), prompt=f"草稿目录：{home}/looped-draft-3\n"))
    add_agent(11, "clerk", "花括号拼路径", agent_records(0, [(1, [])], handback=report_handback("braced-draft-b", "，其余略"), prompt=f"草稿目录：{home}/braced-draft-b\n"))
    add_agent(12, "investigator", "只被子 agent 用", agent_records(0, [(1, [])], handback=report_handback("read-by-sub")))
    add_agent(13, "investigator", "重跑一次没人看", agent_records(0, [(1, [])], handback=report_handback("nobody-reads")))
    add_agent(14, "experiment-runner", "撞顶", agent_records(0, [(1, [])], last_text="You've hit your limit · resets soon"))
    add_agent(15, "experiment-runner", "接口错", agent_records(0, [(1, [])], last_text="API Error: 529 Overloaded"))
    add_agent(16, "experiment-runner", "停掉被续做", agent_records(0, [(1, [])]))
    add_agent(17, "experiment-runner", "停掉没人管", agent_records(100, [(101, [use("bash-17", "Bash", {"command": f"cat {home}/read-by-sub/report.md"})])]))
    add_agent(18, "experiment-runner", "还在跑", agent_records(300, [(301, [])]))
    main_events = [
        assistant(50, [use("main-1", "Bash", {"command": "cat ~/.local/share/singlefs/work/report-read-by-main.md"})]),
        assistant(51, [use("main-2", "Bash", {"command": 'for i in 1 2 3; do cat "$W/looped-draft-$i/report.md"; done'})]),
        assistant(52, [use("main-3", "Bash", {"command": "ls ~/.local/share/singlefs/work/braced-draft-{b,c}/"})]),
        assistant(53, [use("main-4", "Bash", {"command": f"ls {home}/overwritten-draft"})]),
        assistant(54, [use("main-5", "SendMessage", {"to": identifiers[16], "message": "接着做"})]),
    ]
    write_json_lines(session + ".jsonl", main_events)
    expected = {1: "written-landed", 2: "written-landed", 3: "written-landed", 4: "written-landed", 5: "written-partial", 6: "written-overwritten", 7: "written-unused",
                8: "verdict-archived", 9: "report-used-by-main", 10: "report-used-by-main", 11: "report-used-by-main", 12: "report-used-by-later-sub",
                13: "report-unreferenced", 14: "cut-by-usage-limit", 15: "cut-by-api-error", 16: "stopped-salvaged", 17: "stopped-abandoned", 18: "in-flight"}
    return root, repository, base + datetime.timedelta(minutes=303), {identifiers[number]: bucket for number, bucket in expected.items()}


def selftest_outcome():
    """outcome 的自证：18 个答案已知的子 agent（每个桶至少一个）、一个带历史、另挂了一棵工作树的假仓。返回 (项数, 失败句, 子 agent 个数)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-outcome-selftest-")
    failures, checked, expected = [], 0, {}
    try:
        root, repository, now, expected = build_outcome_fixture(work)
        since = datetime.datetime(2026, 10, 1, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        agents = collect_outcome(root, since, repository, now, workers=2)   # 走工作进程那一支
        actual = {agent["agent"]: agent["bucket"] for agent in agents}
        identifiers = sorted(expected)
        normalized = [identifiers[number - 1] for number in (6, 9, 10, 11, 12)]
        bash_written = [identifiers[number - 1] for number in (2, 3, 4)]
        checked += 1
        if len(agents) != len(expected):
            failures.append(f"子 agent 个数：应当 {len(expected)}，实际 {len(agents)}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in normalized if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"路径归一化：交回里粘着全角括号与说明文字的路径、主会话用循环与花括号拼的路径、只点到草稿目录名的，各自都要点得到（预期桶是 written-overwritten、report-used-by-main、report-used-by-later-sub），实际（预期, 实际）{wrong}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in bash_written if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"Bash 写入：heredoc、python、replace-once / insert-row / replace-batch 的实写都要算进落地（--dry-run 的不算），应得 written-landed，实际（预期, 实际）{wrong}")
        checked += 1
        wrong = {name: (bucket, actual.get(name)) for name, bucket in expected.items() if name not in normalized + bash_written and actual.get(name) != bucket}
        if wrong:
            failures.append(f"结局桶：其余每个桶的答案已知（报错的写不算、半截落地、被点到与没被点到的分开），实际和预期不符（预期, 实际）{wrong}")
        summary = outcome_summary(agents)
        checked += 1
        if summary["rework"] != (2, 3000) or summary["cuts"] != collections.Counter({"用量撞顶": 1, "API 错": 1, "停掉": 2, "还在跑": 1}):
            failures.append(f"汇总：返工字样应当 2 个 3000（「重跑一次没人看」「停掉被续做」）、没交回的应当用量撞顶 1、API 错 1、停掉 2、还在跑 1，实际 {summary['rework']}、{dict(summary['cuts'])}")
        checked += 1
        counted_as_rework = ["七·6 代码轮第二轮攻方腿（接续）", "实验执行：E181 第一段（接续）", "m3-batch28-code-r1 正推腿（续）", "7-2b 接手：M3 在核上证",
                             "准入条款簇三方攻方腿（重派）", "Runner E187 r2 (redispatch)", "Fix E193 guard flush bug and rerun", "E156 第 7 次收尾", "批 12 变异三行修红"]
        not_counted_as_rework = ["七·6 后续批第二批实现", "E171 合谋测试第一段", "第 1 次执行员", "Batch 10 tooling", "乙簇三方正推腿", "E177 跑前登记修订"]
        rework_misjudged = [description for description in counted_as_rework if not rework_words().search(description)] + \
                           [description for description in not_counted_as_rework if rework_words().search(description)]
        if rework_misjudged:
            failures.append(f"返工词表：接续、接手、重派、redispatch、fix、第 N 次这些续派要算返工，「后续批」「第 1 次」这类只说次序的、「修订」不算，判错的 {rework_misjudged}")
        checked += 1
        if abs(summary["total_cost"] - 1500 * len(expected)) > 1e-6 or summary["by_bucket"]["verdict-archived"] != [1, 1500]:
            failures.append(f"花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 {1500 * len(expected)}、三方腿 1 个应当 1500，实际 {summary['total_cost']}、{summary['by_bucket'].get('verdict-archived')}")
        checked += 1
        printed = "\n".join(outcome_lines(agents, since, True))
        if f"{len(agents)} 个" not in printed or "report-used-by-main" not in printed or "返工字样" not in printed:
            failures.append("输出：每个子 agent 一行与汇总都要打出来，汇总标题要带个数")
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        failures.append(f"造假仓或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)   # 假仓里另挂的工作树在同一个临时目录里，一起删
    return checked, failures, len(expected)


def selftest_handback_and_start():
    """交回与起点：subagent_transcript.py 的两份共用样本（先被拒、后来送达 / 被拒之后再没送达）与一份 --until 那一天之后开工的。
    outcome 要判前一份交回、后一份没交回、第三份不收；timing 的「没交回」只数后一份。返回 (项数, 失败句)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-handback-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "projects")
        subagents = os.path.join(root, "s3", "subagents")
        os.makedirs(subagents)
        base = transcripts.SAMPLE_BASE
        start_day = base.replace(hour=0, minute=0)
        samples = {"a00000000000000a1": (transcripts.sample_refused_then_delivered(base), True),
                   "a00000000000000a2": (transcripts.sample_refused_only(base), False),
                   "a00000000000000a3": (transcripts.sample_refused_then_delivered(base + datetime.timedelta(days=2)), None)}   # --until 那一天之后开工：不收
        for identifier, (records_list, _) in samples.items():
            write_json_lines(os.path.join(subagents, f"agent-{identifier}.jsonl"), records_list)
            with open(os.path.join(subagents, f"agent-{identifier}.meta.json"), "w", encoding="utf-8") as handle:
                json.dump({"agentType": "clerk", "description": "交回样本"}, handle)
        since, until = start_day - datetime.timedelta(days=4), start_day + datetime.timedelta(days=1)   # --until 写样本开工的那一天（含那一天）
        agents = collect_outcome(root, since, work, base + datetime.timedelta(days=10), workers=1, until=until)
        got = {agent["agent"]: agent["handed_back"] for agent in agents}
        want = {identifier: delivered for identifier, (_, delivered) in samples.items() if delivered is not None}
        checked += 1
        if got != want:
            failures.append(f"交回与起点：outcome 应当收开工在 --until 那一天及之前的两个、先被拒后送达的算交回、被拒之后再没送达的不算（{want}），实际 {got}")
        _, timing = command_timing(root, since)
        rows = timing.get("clerk", [])
        checked += 1
        if len(rows) != 3 or sum(1 for row in rows if row[2]) != 1:
            failures.append(f"交回与起点：timing 应当收 3 个、「没交回」只数被拒之后再没送达的 1 个，实际 {len(rows)} 个、没交回 {sum(1 for row in rows if row[2])} 个")
    except (OSError, RuntimeError) as error:
        failures.append(f"交回与起点：造样本或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return checked, failures


def selftest():
    work = tempfile.mkdtemp(prefix="agent-cost-selftest-")
    failures, checked = [], 0
    try:
        session = os.path.join(work, "s1")
        os.makedirs(os.path.join(session, "subagents"))
        base = datetime.datetime(2026, 9, 28, 1, 0, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        stamp = lambda minutes: (base + datetime.timedelta(minutes=minutes)).isoformat()
        def assistant(uuid, message_id, minutes, content, usage):
            return {"type": "assistant", "uuid": uuid, "timestamp": stamp(minutes), "message": {"id": message_id, "role": "assistant", "content": content, "usage": usage}}
        usage_a = {"input_tokens": 5, "cache_read_input_tokens": 100_000, "cache_creation_input_tokens": 20_000,
                   "cache_creation": {"ephemeral_5m_input_tokens": 20_000, "ephemeral_1h_input_tokens": 0}, "output_tokens": 300}
        sub = [
            {"type": "user", "timestamp": stamp(0), "message": {"role": "user", "content": "派发提示"}},
            assistant("u1", "m1", 1, [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}], usage_a),
            assistant("u1b", "m1", 1, [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}], usage_a),   # 流式重复行，同一个消息 id
            assistant("u2", "m2", 6, [{"type": "tool_use", "id": "t2", "name": "Bash", "input": {"command": "cargo test"}}], usage_a),   # 隔 5 分钟：算等待
            assistant("u3", "m3", 7, [{"type": "tool_use", "id": "t3", "name": "SubagentHandback", "input": {"message": "交回"}}], usage_a),
            {"type": "user", "timestamp": stamp(7), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t3", "content": "{\"success\":true}"}]}},
        ]
        write_json_lines(os.path.join(session, "subagents", "agent-a0000000000000001.jsonl"), sub)
        with open(os.path.join(session, "subagents", "agent-a0000000000000001.meta.json"), "w", encoding="utf-8") as handle:
            json.dump({"agentType": "sweep", "description": "样本"}, handle)
        main = [
            assistant("m-u1", "mm1", 0, [{"type": "tool_use", "id": "d1", "name": "Agent", "input": {"subagent_type": "sweep", "description": "回扫样本", "prompt": "x"}}],
                      {"input_tokens": 10, "cache_read_input_tokens": 500_000, "cache_creation_input_tokens": 1000, "cache_creation": {"ephemeral_5m_input_tokens": 0, "ephemeral_1h_input_tokens": 1000}, "output_tokens": 50}),
            {"type": "user", "timestamp": stamp(0.1), "toolUseResult": {"agentId": "a0000000000000001"},
             "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "d1", "content": "Async agent launched"}]}},
            assistant("m-u2", "mm2", 0.2, [{"type": "tool_use", "id": "d2", "name": "Agent", "input": {"subagent_type": "general-purpose", "description": "查一下", "prompt": "y"}}],
                      {"input_tokens": 10, "cache_read_input_tokens": 500_000, "cache_creation_input_tokens": 0, "output_tokens": 50}),
            {"type": "user", "timestamp": stamp(0.3),
             "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "d2", "content": "PreToolUse:Agent hook error: [bash \"$CLAUDE_PROJECT_DIR\"/.claude/hooks/runner-dispatch-guard.sh]: ✗ 派 general-purpose：内置通用类型"}]}},
            {"type": "user", "timestamp": stamp(8), "message": {"role": "user", "content": "<task-notification><task-id>a0000000000000001</task-id><status>completed</status></task-notification>"}},
        ]
        write_json_lines(session + ".jsonl", main)
        since = datetime.datetime(2026, 1, 1, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        _, by_type, main_grand = command_usage(work, since)
        checked += 1
        sweep = by_type.get("sweep", {})
        if sweep.get("n") != 1 or sweep.get("calls") != 3 or sweep.get("cache_read") != 300_000 or sweep.get("write_5m") != 60_000 or sweep.get("output") != 900:
            failures.append(f"子 agent 用量：应当 1 个、3 次调用（同一个消息 id 的重复行只算一次）、读缓存 300000、5 分钟写 60000、输出 900，实际 {dict(sweep)}")
        checked += 1
        if main_grand.get("calls") != 2 or main_grand.get("write_1h") != 1000:
            failures.append(f"主会话用量：应当 2 次调用、1 小时写 1000，实际 {dict(main_grand)}")
        _, timing = command_timing(work, since)
        checked += 1
        rows = timing.get("sweep", [])
        if len(rows) != 1 or abs(rows[0][0] - 7 * 60) > 1 or abs(rows[0][1] - 5 * 60) > 1 or rows[0][2]:
            failures.append(f"时间：应当挂钟 7 分钟、等待 5 分钟、已交回，实际 {rows}")
        _, dispatches, rejections, launched = command_dispatch(work, since)
        checked += 1
        if dict(dispatches) != {"sweep": 1, "general-purpose": 1} or launched != 1 or sum(rejections.get("runner-dispatch-guard.sh", {}).values()) != 1:
            failures.append(f"派发：应当 sweep 1、general-purpose 1、真正起来 1、派发闸拒 1，实际 {dict(dispatches)}、{launched}、{dict(rejections)}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    outcome_checked, outcome_failures, outcome_agents = selftest_outcome()
    checked += outcome_checked
    failures += outcome_failures
    handback_checked, handback_failures = selftest_handback_and_start()
    checked += handback_checked
    failures += handback_failures
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 read_transcript() 的去重与 command_*() 的口径；outcome 看 scan_subagent()、handback_paths()、index_references()、bash_written_lines()；"
              "交回与起点看 subagent_transcript.py；AGENT_COST_BREAK 设着 no-dedupe、no-path-normalize、no-bash-writes 或 narrow-rework-words，"
              "或 SUBAGENT_TRANSCRIPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项，其中 outcome 的 {outcome_agents} 个子 agent）：调用按消息 id 去重、读缓存与两档缓存写分开、等待按 3 分钟间隔算、派发只数真正起来的、派发闸的拒绝按钩子归类、"
          "每个结局桶都对得上、路径归一化、Bash 里的写入认得出、「修订」不算返工；交回按送达认（先被拒后送达的算、被拒之后再没送达的不算）、起点按首条记录、--until 含那一天")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if not argv or argv[0] not in ("usage", "timing", "dispatch", "outcome"):
        print("  ✗ 用法：agent-cost.py usage|timing|dispatch|outcome [--root 会话目录] [--since YYYY-MM-DD] | --selftest\n  → 怎么办：第一个参数写 usage、timing、dispatch 或 outcome")
        return 2
    root, since = DEFAULT_ROOT, datetime.datetime.now(UTC) - datetime.timedelta(days=7)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
    try:
        default_workers = min(12, heavy_threads.default_heavy_threads())
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了读会话记录的默认进程数：{error}\n  → 怎么办：{error.howto}；或写 --jobs N")
        return 2
    repository, summary_only, workers = REPOSITORY_ROOT, False, default_workers
    since_given, until_day, until = None, None, None
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--root" and rest:
            root = rest.pop(0)
        elif flag == "--since" and rest:
            since_given = rest.pop(0)
            since = datetime.datetime.fromisoformat(since_given).replace(tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        elif flag == "--until" and rest and argv[0] == "outcome" and re.fullmatch(r"\d{4}-\d{2}-\d{2}", rest[0]):
            until_day = rest.pop(0)
            until = datetime.datetime.fromisoformat(until_day).replace(tzinfo=UTC) + datetime.timedelta(days=1)   # clock-times:allow 含 --until 那一天：上界是第二天零点（UTC，与 --since 同一种切法）
        elif flag == "--repo" and rest and argv[0] == "outcome":
            repository = rest.pop(0)
        elif flag == "--summary" and argv[0] == "outcome":
            summary_only = True
        elif flag == "--jobs" and rest and argv[0] == "outcome" and rest[0].isdigit() and int(rest[0]) >= 1:
            workers = int(rest.pop(0))
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --root 与 --since YYYY-MM-DD；outcome 另认 --until YYYY-MM-DD（含那一天）、--repo 仓根、--summary、--jobs 读会话记录的进程数")
            return 2
    if until is not None and until <= since:
        print(f"  ✗ --until {until_day} 早于 --since {since.date()}，窗口是空的\n  → 怎么办：--until 写不早于 --since 的那一天")
        return 2
    if not os.path.isdir(root):
        print(f"  ✗ 会话目录不在：{root}\n  → 怎么办：--root 给 ~/.claude/projects/<项目> 那个目录")
        return 2
    if argv[0] == "outcome":
        if not os.path.isdir(repository):
            print(f"  ✗ 仓不在：{repository}\n  → 怎么办：--repo 给 git 仓的根目录")
            return 2
        try:
            agents = collect_outcome(root, since, repository, datetime.datetime.now(UTC), workers, until)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        except (RuntimeError, OSError) as error:
            print(f"  ✗ 比对仓里的行时出错：{error}\n  → 怎么办：确认 --repo 是 git 仓、git 在 PATH 上；错误原文在上面一句")
            return 1
        window = (f"--since {since.date()}" + ("" if since_given else "（没给，取 7 天前）")
                  + (f"、--until {until_day}（含那一天）" if until_day else "、--until 没给（不设上界）"))
        if not agents:
            print(f"  ✗ {window} 这个窗口里没有开工的子 agent，什么都没判\n  → 怎么办：--since 往前挪、--until 往后挪，或确认 --root 指的是 ~/.claude/projects/<项目> 那个目录")
            return 77
        print("\n".join(outcome_lines(agents, since, not summary_only, until_day)))
        print(f"  ✓ 查了 {len(agents)} 个子 agent（{window}，按首条记录的时刻筛），逐个归了结局桶")
        return 0
    lines = {"usage": lambda: command_usage(root, since)[0], "timing": lambda: command_timing(root, since)[0], "dispatch": lambda: command_dispatch(root, since)[0]}[argv[0]]()
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
