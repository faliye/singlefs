#!/usr/bin/env bash
# PreToolUse hook（SendMessage）：给一个被中断过（撞限额、被停、报错）的子 agent 续做之前拦一道。
#
# 为什么：2026-09-19 实测，E155 第二次跑的执行员撞限额之后被续做，上下文涨到 95 万，380 次调用平均每次 62.6 万、
# 累计读缓存 2.36 亿、整份重写两次共 123.5 万，真正的输出 2.9 万。用户定：「以后这样被中断的 subagent 都应该新开，
# 但是新开的时候要保证上下文和成果物的接续；如果接续会导致主 agent 的上下文膨胀，那么干脆就重开」。
#
# 判法：收件人在本会话的 subagents 目录里有会话记录（按 agent id 找），且本会话记录里它最近一次任务通知是 failed 或 killed，就拒绝；
# 找不到会话记录的收件人（别的会话、团队成员、名字而不是 id）放行。中断过的不留「只差收尾」的口子（用户 2026-09-19：
# 「按照你现在的方法实际上也是在让上下文白白膨胀，没有意义」）：停掉它，用 research/scripts/agent-handover.py 从会话记录里
# 抽出它的进度与成果，交给一个新的同类 agent。
#
# 上下文多大不拦（用户 2026-09-19：「不应该限制 280K，这个不可理。检测的看门狗不应该有终止的权利，subagent 也不应该有超过了
# token 终止的逻辑，应该告诉主 agent 看看任务是不是异常，如果不是就继续跑」）：看门狗报「上下文过大」，主 agent 看任务有没有异常，
# 没有就接着跑、接着续。这里原先还按上下文 40 万拒绝续做，那天主 agent 据此停掉了一个 28 万、任务并无异常的 agent。
#
#   continuation-guard.sh             # 从 stdin 读 hook 的 JSON
#   continuation-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；CONTINUATION_GUARD_DISABLE_CHECK=1 时自检必须判红
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
        return 0, None
    reason = (f"✗ 子 agent {recipient} 最近一次任务通知是 {status}（撞限额、被停或报错），中断过的不续，新开一个接着做"
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
        def write_agent(agent_id, context_tokens, continuations):
            records = [{"message": {"role": "assistant", "usage": {"input_tokens": 10, "cache_read_input_tokens": context_tokens - 10}}}]
            records += [{"origin": {"kind": "coordinator"}, "message": {"role": "user", "content": "续派"}}] * continuations
            with open(os.path.join(subagents, f"agent-{agent_id}.jsonl"), "w") as handle:
                handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")
        write_agent("aaaa1111bbbb2222", 120_000, 1)
        write_agent("cccc3333dddd4444", 950_000, 3)
        write_agent("gggg7777hhhh8888", 80_000, 0)
        write_agent("iiii9999jjjj0000", 80_000, 1)
        write_agent("kkkk1212llll3434", 80_000, 0)
        def notification(agent_id, status):
            return json.dumps({"origin": {"kind": "task-notification"}, "message": {"role": "user", "content":
                               f"<task-notification>\n<task-id>{agent_id}</task-id>\n<output-file>x</output-file>\n<status>{status}</status>\n</task-notification>"}})
        with open(session_transcript, "w") as handle:
            fake_command = json.dumps({"message": {"role": "assistant", "content": [{"type": "tool_use", "name": "Bash", "input": {"command":
                                       "grep '<task-id>kkkk1212llll3434</task-id>.*<status>completed</status>' session.jsonl"}}]}})
            handle.write("\n".join([notification("gggg7777hhhh8888", "failed"),
                                     notification("iiii9999jjjj0000", "failed"), notification("iiii9999jjjj0000", "completed"),
                                     notification("kkkk1212llll3434", "killed"), fake_command]) + "\n")
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
        ]
        script = os.path.join(hook_dir, "continuation-guard.sh")
        for label, recipient, want in (("stdin:被停过的续做", "kkkk1212llll3434", 2), ("stdin:上下文大的续做", "cccc3333dddd4444", 0)):
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
        print("    → 看 decide() 的判法；CONTINUATION_GUARD_DISABLE_CHECK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：上下文大小不论、收件人没有会话记录、中断之后又完成过的放行；最近一次通知是 failed 或 killed 的一律拒绝（查了 {len(cases)} 种）")
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
