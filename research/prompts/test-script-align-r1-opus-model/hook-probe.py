#!/usr/bin/env python3
"""喂 heavy-test-guard.sh 一条 JSON，打退出码与 stderr 头一行。
用法：hook-probe.py <仓拷贝根> <检出记录文件> <agent_type 或 -> <cwd> <命令>
只喂仓拷贝里的那一份钩子（它的登记表、HOOK_REPOSITORY 都是拷贝里的），检出记录写到给的文件，不写会话共用的那一份。"""
import json, os, subprocess, sys

root, detections, agent, cwd, command = sys.argv[1:6]
hook_input = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}, "session_id": "probe"}
if agent != "-":
    hook_input["agent_type"] = agent
environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
environment.pop("SINGLEFS_HEAVY_TESTS", None)
done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input),
                      capture_output=True, text=True, env=environment, timeout=120)
first = (done.stderr.strip().splitlines() or [""])[0]
print(f"exit={done.returncode}\tagent={agent}\tcwd={cwd}\tcommand={command}\tstderr1={first[:300]}")
