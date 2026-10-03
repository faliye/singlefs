#!/usr/bin/env python3
"""喂本仓 heavy-test-guard.sh 一条 JSON，打退出码与 stderr 头一行；被判的命令一条都不起。
用法：hook-probe.py <检出记录文件> <agent_type 或 -> <cwd> <命令>
钩子只认本仓路径上的登记，所以喂的是本仓那一份（只读）；检出记录写到给的草稿文件，不写会话共用的那一份。"""
import json, os, subprocess, sys

detections, agent, cwd, command = sys.argv[1:5]
root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
hook_input = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}, "session_id": "probe-r2-opus"}
if agent != "-":
    hook_input["agent_type"] = agent
environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
environment.pop("SINGLEFS_HEAVY_TESTS", None)
done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input),
                      capture_output=True, text=True, env=environment, timeout=120)
first = (done.stderr.strip().splitlines() or [""])[0]
print(f"exit={done.returncode}\tagent={agent}\tcwd={cwd}\tcommand={command}\tstderr1={first[:240]}")
