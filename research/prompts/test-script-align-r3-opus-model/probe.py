#!/usr/bin/env python3
import json, os, subprocess, sys
detections, agent, cwd, command = sys.argv[1:5]
root = "/home/fy5090/code/singlefs"
hi = {"tool_name":"Bash","cwd":cwd,"tool_input":{"command":command},"session_id":"probe-r3-opus"}
if agent != "-": hi["agent_type"]=agent
env = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
env.pop("SINGLEFS_HEAVY_TESTS", None)
d = subprocess.run(["bash", os.path.join(root,".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hi),
                   capture_output=True, text=True, env=env, timeout=120)
first=(d.stderr.strip().splitlines() or [""])[0]
print(f"exit={d.returncode}\tagent={agent}\tcommand={command}\tstderr1={first[:200]}")
