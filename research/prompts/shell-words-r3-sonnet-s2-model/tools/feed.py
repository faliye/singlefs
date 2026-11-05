#!/usr/bin/env python3
"""把一条命令按 hook 的输入格式（JSON）喂给草稿目录里的那两份 PreToolUse[Bash] 钩子，返回各自的退出码与 stderr。"""
import json, os, subprocess

HOOKS = ["heavy-test-guard.sh", "bash-command-detector.sh"]


def feed(repo, work, command, role="main", background=False, cwd=None, agent_types=None, session="s-r3"):
    agent_types = agent_types or {"sub": "three-way-forward", "editor": "implementation-writer"}
    payload = {"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": session,
               "cwd": cwd or repo, "tool_input": {"command": command, "run_in_background": bool(background)}}
    if role != "main":
        payload["agent_type"] = agent_types[role]
    env = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "CLAUDE_PROJECT_DIR", "CAPPED"))}
    env["SINGLEFS_WORK_ROOT"] = os.path.join(work, "wr")
    env["AGENT_HOOK_DETECTIONS"] = os.path.join(work, "detections.jsonl")
    out = {}
    for hook in HOOKS:
        r = subprocess.run(["bash", os.path.join(repo, ".claude", "hooks", hook)], input=json.dumps(payload), text=True,
                           capture_output=True, env=env, cwd=repo, timeout=150)
        out[hook] = {"rc": r.returncode, "stderr": r.stderr[:2500]}
    return out


def verdict(fed):
    return "拒" if any(v["rc"] == 2 for v in fed.values()) else ("放(异常)" if any(v["rc"] not in (0, 2) for v in fed.values()) else "放")
