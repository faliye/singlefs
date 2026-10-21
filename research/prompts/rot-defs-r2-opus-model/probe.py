#!/usr/bin/env python3
"""rot-defs-r2 攻方腿的钩子探针：把一条命令当 PreToolUse(Bash) 的输入喂给项目注册在 Bash 上的三道判命令的钩子，打各自的退出码与 stderr 前几行。
不执行命令本身；检出记录与上次红的清单都指到给的草稿目录，不碰仓外共用的那一份。
用法：probe.py <仓根> <草稿目录> <agent_type，主 agent 写 -> <当前目录> <命令文件>"""
import json, os, subprocess, sys
repo, work, agent, cwd, path = sys.argv[1:6]
command = open(path, encoding="utf-8").read().rstrip("\n")
command = command.replace("@REPO@", repo)
payload = {"tool_name": "Bash", "cwd": cwd, "session_id": "probe-rot-defs-r2", "transcript_path": "/nonexistent",
           "tool_input": {"command": command}}
if agent != "-":
    payload["agent_type"] = agent
    payload["agent_id"] = "probe"
env = dict(os.environ, CLAUDE_PROJECT_DIR=repo, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl"),
           SINGLEFS_LAST_RUNS_DIRECTORY=os.path.join(work, "last-runs"))
for hook in ("bash-command-detector.sh", "heavy-test-guard.sh", "run-scope-guard.sh"):
    finished = subprocess.run(["bash", os.path.join(repo, ".claude/hooks", hook)], input=json.dumps(payload),
                              capture_output=True, text=True, env=env, cwd=cwd)
    lines = [line for line in finished.stderr.splitlines() if line.strip()]
    print(f"[{hook}] exit={finished.returncode}")
    for line in lines[:int(os.environ.get("PROBE_LINES", "1"))]:
        print("   ", line[:300])
