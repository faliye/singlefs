#!/usr/bin/env python3
"""辩方腿 V1：把命令写成 hook 输入 JSON 喂给副本里的两道钩子（只判定，不执行命令），打退出码与 stderr 首行。
用法：feed.py <cases.json> [REPO]；cases 每项 [名字, agent_type 或 null, 命令, cwd(可省)]"""
import json, os, subprocess, sys
REPO = sys.argv[2] if len(sys.argv) > 2 else "/tmp/claude-1000/tooling-detached-run-r2-sonnet/repo"
HOOKS = ["bash-command-detector.sh", "heavy-test-guard.sh"]
def run(agent, command, cwd, extra_env):
    payload = {"tool_name": "Bash", "tool_input": {"command": command, "run_in_background": False},
               "session_id": "sonnet-r2-v1", "cwd": cwd, "transcript_path": "/dev/null"}
    if agent:
        payload["agent_type"] = agent
    env = dict(os.environ, AGENT_HOOK_DETECTIONS="/tmp/claude-1000/tooling-detached-run-r2-sonnet/out/detections.jsonl", CLAUDE_PROJECT_DIR=REPO)
    env.pop("SINGLEFS_HEAVY_TESTS", None)
    env.update(extra_env)
    res = []
    for hook in HOOKS:
        p = subprocess.run(["bash", os.path.join(REPO, ".claude/hooks", hook)], input=json.dumps(payload), capture_output=True, text=True, cwd=REPO, env=env, timeout=120)
        first = next((l.strip() for l in p.stderr.splitlines() if l.strip()), "")
        res.append((hook.split("-")[0], p.returncode, first[:150]))
    return res
for case in json.load(open(sys.argv[1])):
    name, agent, cmd = case[0], case[1], case[2]
    cwd = case[3] if len(case) > 3 and case[3] else REPO
    extra = case[4] if len(case) > 4 else {}
    print(f"## {name}\n   agent={agent or '主agent'} cwd={cwd}\n   cmd: {cmd}")
    for hook, rc, first in run(agent, cmd, cwd, extra):
        print(f"   {hook}: rc={rc}  {first}")
