#!/usr/bin/env bash
# PreToolUse hook（SendMessage）：给一个已经交回过、或被中断过（撞限额、被停、报错）的子 agent 发消息之前拦一道。
#
# 为什么：交回之后不续做，新活一律新派。用户 2026-09-24 问「启动subagent的时候 为什么会复用 id呢 很奇怪 这个能禁止复用吗」，
# 选的是「交回之后不许续做」；还没交回的（在干活、或结束本轮去等自己后台任务的）照旧放行，整点询问与纠正要送得到。
# 中断过的不续：2026-09-19 实测，E155 第二次跑的执行员撞限额之后被续做，上下文涨到 95 万，380 次调用平均每次 62.6 万、
# 累计读缓存 2.36 亿、整份重写两次共 123.5 万，真正的输出 2.9 万。用户定：「以后这样被中断的 subagent 都应该新开，
# 但是新开的时候要保证上下文和成果物的接续；如果接续会导致主 agent 的上下文膨胀，那么干脆就重开」。
#
# 判法：收件人在本会话的 subagents 目录里有会话记录（`<会话目录>/subagents/agent-<id>.jsonl`，按 agent id 找）才判；
# 找不到会话记录的收件人（别的会话、团队成员、名字而不是 id）放行。有会话记录的，下面两条任一成立就拒绝：
#   ① 交回过：会话记录逐行按 JSON 解析，至少有一条 assistant 消息的 content 块 type 为 tool_use、name 为 SubagentHandback，
#      并且后面有 tool_use_id 与它相同的 tool_result，不是错误（is_error 不为真，内容也不是 {"success":false,…} 那种没送出）。
#      不按字符串搜：每份会话记录开头的工具清单里都写着这个工具名，搜字符串会把没交回的也判成交回。
#   ② 中断过：本会话记录里它最近一次任务通知是 failed 或 killed。中断过的不留「只差收尾」的口子（用户 2026-09-19：
#      「按照你现在的方法实际上也是在让上下文白白膨胀，没有意义」）。
# 两条的出路都是新派一个同类 agent：派发提示指到它的报告与产物；要它会话里的进度，用 research/scripts/agent-handover.py
# 从会话记录里抽出来给新 agent 读。
#
# 上下文多大不拦（用户 2026-09-19：「不应该限制 280K，这个不可理。检测的看门狗不应该有终止的权利，subagent 也不应该有超过了
# token 终止的逻辑，应该告诉主 agent 看看任务是不是异常，如果不是就继续跑」）：看门狗报「上下文过大」，主 agent 看任务有没有异常，
# 没有就接着跑、接着续。这里原先还按上下文 40 万拒绝续做，那天主 agent 据此停掉了一个 28 万、任务并无异常的 agent。
#
#   continuation-guard.sh             # 从 stdin 读 hook 的 JSON
#   continuation-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；CONTINUATION_GUARD_DISABLE_CHECK=1 时自检必须判红，
#                                     # CONTINUATION_GUARD_BREAK=anyrecord / handback-grep / handback-anyresult 各自也必须让它红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 runner-dispatch-guard.sh 同一个写法）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile

def subagent_transcript_path(hook_input, recipient):
    transcript_path = hook_input.get("transcript_path") or ""
    if not transcript_path.endswith(".jsonl") or not re.fullmatch(r"[0-9a-z]{8,}", recipient or ""):
        return None
    candidate = os.path.join(transcript_path[:-len(".jsonl")], "subagents", f"agent-{recipient}.jsonl")
    return candidate if os.path.isfile(candidate) else None

def context_and_continuations(path):
    """最后一次调用的上下文（读缓存 + 写缓存 + 没进缓存的输入），与续做消息的条数。"""
    last_context, continuations = 0, 0
    for line in open(path, encoding="utf-8", errors="replace"):
        try:
            record = json.loads(line)
        except ValueError:
            continue
        message = record.get("message")
        if not isinstance(message, dict):
            continue
        if message.get("role") == "assistant":
            usage = message.get("usage") or {}
            if usage:
                last_context = ((usage.get("cache_read_input_tokens") or 0) + (usage.get("cache_creation_input_tokens") or 0)
                                + (usage.get("input_tokens") or 0))
        elif message.get("role") == "user" and (record.get("origin") or {}).get("kind") == "coordinator":
            continuations += 1
    return last_context, continuations

TASK_STATUS = re.compile(r"<task-id>([^<]+)</task-id>(?:(?!<task-id>).)*?<status>([a-z_]+)</status>")

def latest_task_status(session_transcript, recipient):
    """本会话记录里这个 agent 最近一次任务通知的状态（completed / failed / killed …），没有通知就是 None。"""
    if not session_transcript or not os.path.isfile(session_transcript):
        return None
    status = None
    for line in open(session_transcript, encoding="utf-8", errors="replace"):
        if recipient not in line or "<status>" not in line:
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        # 只认真正的任务通知：origin 是 task-notification 的消息、排队的附件、队列操作。主 agent 自己的命令与回复里也可能写着
        # 「<task-id>…</task-id>…<status>failed</status>」（2026-09-19 实测：一条 grep 命令的文本让被停的 agent 读成了 failed）。
        is_notification = ((record.get("origin") or {}).get("kind") == "task-notification"
                           or record.get("type") == "queue-operation"
                           or (record.get("attachment") or {}).get("type") == "queued_command")
        if not is_notification and os.environ.get("CONTINUATION_GUARD_BREAK") != "anyrecord":
            continue
        for task_id, found in TASK_STATUS.findall(line):
            if task_id == recipient:
                status = found
    return status

def handback_result_is_error(block):
    """交回调用的 tool_result 算不算没送出：is_error 为真，或内容是 {"success": false, …}（例如「Nothing was sent」）。"""
    if os.environ.get("CONTINUATION_GUARD_BREAK") == "handback-anyresult":
        return False
    if block.get("is_error") is True:
        return True
    content = block.get("content")
    if isinstance(content, list):
        content = "".join(part.get("text") or "" for part in content if isinstance(part, dict))
    try:
        parsed = json.loads(content) if isinstance(content, str) else None
    except ValueError:
        parsed = None
    return isinstance(parsed, dict) and parsed.get("success") is False

def handback_delivered(path):
    """子 agent 的会话记录里有没有一次送达的交回：assistant 调过 SubagentHandback，且后面有它对应、不是错误的 tool_result。"""
    if os.environ.get("CONTINUATION_GUARD_BREAK") == "handback-grep":
        return any("SubagentHandback" in line for line in open(path, encoding="utf-8", errors="replace"))
    handback_call_ids = set()
    for line in open(path, encoding="utf-8", errors="replace"):
        # 只解析可能相关的行：带工具名的（调用），或带着已知调用号的（它的 tool_result）
        if "SubagentHandback" not in line and not any(call_id in line for call_id in handback_call_ids):
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        message = record.get("message")
        if not isinstance(message, dict) or not isinstance(message.get("content"), list):
            continue
        for block in message["content"]:
            if not isinstance(block, dict):
                continue
            if (message.get("role") == "assistant" and block.get("type") == "tool_use"
                    and block.get("name") == "SubagentHandback" and block.get("id")):
                handback_call_ids.add(block["id"])
            elif (message.get("role") == "user" and block.get("type") == "tool_result"
                    and block.get("tool_use_id") in handback_call_ids and not handback_result_is_error(block)):
                return True
    return False

def decide(hook_input):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    if os.environ.get("CONTINUATION_GUARD_DISABLE_CHECK") == "1":
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    recipient = tool_input.get("to") or ""
    path = subagent_transcript_path(hook_input, recipient)
    if path is None:
        return 0, None
    last_context, continuations = context_and_continuations(path)
    status = latest_task_status(hook_input.get("transcript_path"), recipient)
    if status not in ("failed", "killed"):
        if not handback_delivered(path):
            return 0, None
        return 2, (f"✗ 子 agent {recipient} 已经交回过（它的会话记录里有一次送达的 SubagentHandback），交回之后不续做，新活一律新派。\n"
                   "→ 怎么办：不给它发消息。派一个新的同类 agent，派发提示指到它交回的报告与留下的产物；要它会话里的进度，跑 "
                   f"`python3 research/scripts/agent-handover.py --agent {recipient} --out <交接摘要.md>` 抽出来给新 agent 读"
                   "（交接摘要只给新的读，主 agent 不读它）。")
    reason =(f"✗ 子 agent {recipient} 最近一次任务通知是 {status}（撞限额、被停或报错），中断过的不续，新开一个接着做"
              f"（它最后一次调用的上下文 {last_context / 10000:.1f} 万，已被续做 {continuations} 次）。\n")
    return 2, (reason +
               "→ 怎么办：不续这一个。跑 `python3 research/scripts/agent-handover.py --agent "
               f"{recipient} --out <交接摘要.md>` 从它的会话记录里抽出派发提示、续做消息、写过的文件、跑过的命令与它最后说的话；"
               "派一个新的同类 agent，派发提示指到交接摘要与它留下的产物，第一步先核现场（编不编得过、产物是不是这一轮的），再接着做。"
               "交接摘要只给新的读，主 agent 不读它；接续非得经过主 agent 的上下文，就干脆从头重派。")

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    try:
        session_transcript = os.path.join(work, "session.jsonl")
        open(session_transcript, "w").close()
        subagents = os.path.join(work, "session", "subagents")
        os.makedirs(subagents)
        def write_agent(agent_id, context_tokens, continuations, extra_records=()):
            records = [{"message": {"role": "assistant", "usage": {"input_tokens": 10, "cache_read_input_tokens": context_tokens - 10}}}]
            records += [{"origin": {"kind": "coordinator"}, "message": {"role": "user", "content": "续派"}}] * continuations
            records += list(extra_records)
            with open(os.path.join(subagents, f"agent-{agent_id}.jsonl"), "w") as handle:
                handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")
        # 每份真会话记录开头都有：工具清单（带着 SubagentHandback 这个名字）与派发提示（常常也提到它）
        opening_records = [
            {"type": "attachment", "attachment": {"type": "prompt_snapshot", "tools": [
                {"name": "SubagentHandback", "description": "Deliver your final report to the agent that spawned you"}]}},
            {"type": "user", "message": {"role": "user", "content": "干完用 SubagentHandback 交回"}}]
        def tool_call(call_id, tool_name, tool_input):
            return {"type": "assistant", "message": {"role": "assistant", "content": [
                {"type": "tool_use", "id": call_id, "name": tool_name, "input": tool_input}]}}
        def tool_result(call_id, content, is_error=None):
            block = {"type": "tool_result", "tool_use_id": call_id, "content": content}
            if is_error is not None:
                block["is_error"] = is_error
            return {"type": "user", "message": {"role": "user", "content": [block]}}
        def result_text(payload):
            return [{"type": "text", "text": json.dumps(payload)}]
        write_agent("aaaa1111bbbb2222", 120_000, 1)
        write_agent("cccc3333dddd4444", 950_000, 3)
        write_agent("gggg7777hhhh8888", 80_000, 0)
        write_agent("iiii9999jjjj0000", 80_000, 1)
        write_agent("kkkk1212llll3434", 80_000, 0)
        write_agent("mmmm1313nnnn1414", 80_000, 0, opening_records + [
            tool_call("toolu_handback_delivered", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_delivered", result_text({"success": True, "message": "Report delivered to your caller."}))])
        write_agent("oooo1515pppp1616", 80_000, 0, opening_records + [
            tool_call("toolu_handback_errored", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_errored", "<tool_use_error>InputValidationError</tool_use_error>", is_error=True)])
        write_agent("qqqq1717rrrr1818", 80_000, 0, opening_records + [
            tool_call("toolu_handback_not_sent", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_not_sent", result_text({"success": False, "message": "Nothing was sent"}))])
        write_agent("ssss1919tttt2020", 80_000, 0, opening_records + [
            tool_call("toolu_read_definition", "Read", {"file_path": "agents/x.md"}),
            tool_result("toolu_read_definition", "交回：最后一步调 SubagentHandback，把报告交给主 agent")])
        write_agent("uuuu2121vvvv2222", 80_000, 0, opening_records + [
            tool_call("toolu_background_build", "Bash", {"command": "cargo build", "run_in_background": True}),
            tool_result("toolu_background_build", "Command running in background with ID: b1")])
        write_agent("wwww2323xxxx2424", 80_000, 0, opening_records + [
            tool_call("toolu_handback_pending", "SubagentHandback", {"message": "报告"})])
        def notification(agent_id, status):
            return json.dumps({"origin": {"kind": "task-notification"}, "message": {"role": "user", "content":
                               f"<task-notification>\n<task-id>{agent_id}</task-id>\n<output-file>x</output-file>\n<status>{status}</status>\n</task-notification>"}})
        with open(session_transcript, "w") as handle:
            fake_command = json.dumps({"message": {"role": "assistant", "content": [{"type": "tool_use", "name": "Bash", "input": {"command":
                                       "grep '<task-id>kkkk1212llll3434</task-id>.*<status>completed</status>' session.jsonl"}}]}})
            handle.write("\n".join([notification("gggg7777hhhh8888", "failed"),
                                     notification("iiii9999jjjj0000", "failed"), notification("iiii9999jjjj0000", "completed"),
                                     notification("kkkk1212llll3434", "killed"), notification("mmmm1313nnnn1414", "completed"),
                                     notification("uuuu2121vvvv2222", "completed"), fake_command]) + "\n")
        def case(label, recipient, message, want):
            hook_input = {"tool_name": "SendMessage", "transcript_path": session_transcript, "tool_input": {"to": recipient, "message": message}}
            return (label, want, decide(hook_input)[0])
        cases = [
            case("上下文小的续做", "aaaa1111bbbb2222", "接着做第二段", 0),
            case("上下文大的续做放行：大小不是理由", "cccc3333dddd4444", "接着做第二段", 0),
            case("收件人没有会话记录", "eeee5555ffff6666", "接着做", 0),
            case("收件人是名字不是 id", "reviewer", "接着做", 0),
            case("撞过限额（failed）的上下文小也不续", "gggg7777hhhh8888", "限额重置了，接着做", 2),
            case("failed 之后又 completed 的放行", "iiii9999jjjj0000", "接着做", 0),
            case("被停（killed）的不续", "kkkk1212llll3434", "接着做", 2),
            case("已交回（SubagentHandback 送达）的不续", "mmmm1313nnnn1414", "再补一件", 2),
            case("调过 SubagentHandback 但 tool_result 是错误的放行", "oooo1515pppp1616", "接着做", 0),
            case("调过 SubagentHandback 但 tool_result 是 success:false（没送出）的放行", "qqqq1717rrrr1818", "接着做", 0),
            case("只有工具清单与正文里带着 SubagentHandback 这个名字的放行", "ssss1919tttt2020", "整点询问", 0),
            case("结束本轮在等后台任务、还没交回的放行", "uuuu2121vvvv2222", "整点询问", 0),
            case("调了 SubagentHandback、还没有 tool_result 的放行", "wwww2323xxxx2424", "接着做", 0),
        ]
        script = os.path.join(hook_dir, "continuation-guard.sh")
        for label, recipient, want in (("stdin:被停过的续做", "kkkk1212llll3434", 2), ("stdin:上下文大的续做", "cccc3333dddd4444", 0),
                                       ("stdin:已交回的续做", "mmmm1313nnnn1414", 2), ("stdin:在等后台任务的询问", "uuuu2121vvvv2222", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "SendMessage", "transcript_path": session_transcript,
                                                         "tool_input": {"to": recipient, "message": "接着做"}}),
                                       capture_output=True, text=True)
            cases.append((label, want, completed.returncode))
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() / handback_delivered() 的判法；CONTINUATION_GUARD_DISABLE_CHECK 或 CONTINUATION_GUARD_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：上下文大小不论、收件人没有会话记录、中断之后又完成过、还没交回（在等后台任务、交回报错或没送出、只在工具清单里带着名字）的放行；"
          "交回送达过的、最近一次通知是 failed 或 killed 的一律拒绝")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError:
        return 0
    code, message = decide(hook_input)
    if code:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
