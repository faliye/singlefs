"""K3 ㉛：subagent_transcript.py 的守卫只管直接执行；import 它的续做闸平常跑（hook 模式）时，环境里漏进 SUBAGENT_TRANSCRIPT_BREAK 会怎样。
造一份子 agent 会话记录：交回被交回闸拒（tool_result is_error），之后没再交；主 agent 给它发 SendMessage。不设开关、设开关各喂一次。
用法：python3 -I import_break.py <草稿目录> <改后树（下面有 .claude/hooks 与 scripts/agents）>"""
import json, os, shutil, subprocess, sys
S, tree = sys.argv[1:3]
session = os.path.join(S, "import-break-session")
shutil.rmtree(session, ignore_errors=True)
os.makedirs(os.path.join(session, "subagents"))
open(session + ".jsonl", "w").close()
agent = "abreakprobe0000001"
records = [
    {"type": "user", "timestamp": "2026-10-23T00:00:00+00:00", "message": {"role": "user", "content": "# 派发"}},
    {"type": "assistant", "timestamp": "2026-10-23T00:01:00+00:00", "message": {"role": "assistant", "content": [{"type": "tool_use", "id": "h1", "name": "SubagentHandback", "input": {"message": "交回"}}]}},
    {"type": "user", "timestamp": "2026-10-23T00:01:01+00:00", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "h1", "is_error": True, "content": "PreToolUse:SubagentHandback hook error: ✗ 被拒"}]}},
]
with open(os.path.join(session, "subagents", f"agent-{agent}.jsonl"), "w", encoding="utf-8") as handle:
    for record in records:
        handle.write(json.dumps(record, ensure_ascii=False) + "\n")
hook_input = {"hook_event_name": "PreToolUse", "tool_name": "SendMessage", "tool_input": {"to": agent, "message": "接着做"}, "transcript_path": session + ".jsonl"}
for label, extra in (("不设开关", {}), ("环境里有 SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered", {"SUBAGENT_TRANSCRIPT_BREAK": "refused-counts-delivered"})):
    environment = {name: value for name, value in os.environ.items() if not name.endswith("_BREAK")}
    environment.update(extra)
    done = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "continuation-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=60)
    first = next((line for line in done.stderr.splitlines() if line.startswith("✗")), "")
    print(f"续做闸（hook 模式）{label}：退 {done.returncode}{('；' + first[:80]) if first else ''}")
shutil.rmtree(session, ignore_errors=True)
os.remove(session + ".jsonl")
