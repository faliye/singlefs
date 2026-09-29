#!/usr/bin/env python3
# admission: always 每一次调都按此刻的会话记录算账，上一次的结论不替这一次作保
# run-condition: none 只读 ~/.claude/projects/ 下的会话记录，除了 python3 之外没有环境要求
"""agent 体系的账表：子 agent 与主 agent 的用量、每类的挂钟与等待、派发与闸的拒绝、看门狗的退出。

用法：
    agent-cost.py usage    [--root 会话目录] [--since YYYY-MM-DD]   # 子 agent 按类型的调用、上下文、读缓存、折算；主会话各自的账
    agent-cost.py timing   [--root …] [--since …]                  # 每类的挂钟中位、等待占比、没交回的；三方一轮的关键路径
    agent-cost.py dispatch [--root …] [--since …]                  # 派发按类型、派发闸与钩子的拒绝、重复派发、消息、看门狗退出码
    agent-cost.py --selftest                                       # 造一份假会话目录算一遍，钉住数；AGENT_COST_BREAK=no-dedupe 时必须判红

口径（与 records/2026-09-28-agent体系用量时间与调度分析.md 第一节同）：一次调用是一条带 usage 的 assistant 消息、按消息 id 去重；
上下文 = cache_read + cache_creation + input；折算 = 新输入 ×1 + 读缓存 ×0.1 + 5 分钟缓存写 ×1.25 + 1 小时缓存写 ×2 + 输出 ×5（相对量，不是账单）；
等待 = 相邻两次调用间隔超过 3 分钟。第一版分析用的五份脚本在 research/prompts/agent-cost-2026-09-28/，这一份把它们收进仓、加自证。
"""
import bisect
import collections
import datetime
import glob
import json
import os
import re
import shutil
import statistics
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

DEFAULT_ROOT = os.path.expanduser("~/.claude/projects/-home-fy5090-code-singlefs")
BROKEN = os.environ.get("AGENT_COST_BREAK", "")
UTC = datetime.timezone.utc   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
AGENT_ID = re.compile(r"a[0-9a-f]{16}")
PRIME_MARKS = re.compile("[\u2032\u2033\u2034\u02b9\u02ba]")   # 输出里不留角标字符：门禁 doc-text 的 prime-marks 格扫全仓，抄进记录的原文会被它判红


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


def read_transcript(path):
    """一份会话记录：去重后的调用（按次序）、首末时刻、交回时刻、工具调用计数。"""
    seen, order, first, last, handback = {}, [], None, None, None
    tools = collections.Counter()
    for record in records(path):
        moment = timestamp_of(record)
        if moment:
            first = first or moment
            last = moment
        if record.get("type") != "assistant":
            continue
        message = record.get("message", {})
        usage = message.get("usage")
        if usage:
            key = (message.get("id") or record.get("uuid")) if BROKEN != "no-dedupe" else record.get("uuid") or id(record)
            if key not in seen:
                order.append(key)
            seen[key] = usage
        for block in message.get("content", []) if isinstance(message.get("content"), list) else []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                tools[block.get("name")] += 1
                if block.get("name") == "SubagentHandback":
                    handback = moment
    calls = [seen[key] for key in order]
    totals = collections.Counter()
    for usage in calls:
        creation = usage.get("cache_creation") or {}
        totals["input"] += usage.get("input_tokens", 0)
        totals["cache_read"] += usage.get("cache_read_input_tokens", 0)
        totals["write_5m"] += creation.get("ephemeral_5m_input_tokens", usage.get("cache_creation_input_tokens", 0) if not creation else 0)
        totals["write_1h"] += creation.get("ephemeral_1h_input_tokens", 0)
        totals["output"] += usage.get("output_tokens", 0)
    return {"calls": calls, "totals": totals, "first": first, "last": last, "handback": handback, "tools": tools,
            "wall": (last - first).total_seconds() if first and last else 0}


def subagent_files(root, since):
    for meta in sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.meta.json"))):
        transcript = meta.replace(".meta.json", ".jsonl")
        if os.path.exists(transcript) and recent(transcript, since):
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


def write_json_lines(path, items):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        for item in items:
            handle.write(json.dumps(item, ensure_ascii=False) + "\n")


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
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 read_transcript() 的去重与 command_*() 的口径；AGENT_COST_BREAK=no-dedupe 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：调用按消息 id 去重、读缓存与两档缓存写分开、等待按 3 分钟间隔算、派发只数真正起来的、派发闸的拒绝按钩子归类")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if not argv or argv[0] not in ("usage", "timing", "dispatch"):
        print("  ✗ 用法：agent-cost.py usage|timing|dispatch [--root 会话目录] [--since YYYY-MM-DD] | --selftest\n  → 怎么办：第一个参数写 usage、timing 或 dispatch")
        return 2
    root, since = DEFAULT_ROOT, datetime.datetime.now(UTC) - datetime.timedelta(days=7)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--root" and rest:
            root = rest.pop(0)
        elif flag == "--since" and rest:
            since = datetime.datetime.fromisoformat(rest.pop(0)).replace(tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --root 与 --since YYYY-MM-DD")
            return 2
    if not os.path.isdir(root):
        print(f"  ✗ 会话目录不在：{root}\n  → 怎么办：--root 给 ~/.claude/projects/<项目> 那个目录")
        return 2
    lines = {"usage": lambda: command_usage(root, since)[0], "timing": lambda: command_timing(root, since)[0], "dispatch": lambda: command_dispatch(root, since)[0]}[argv[0]]()
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
