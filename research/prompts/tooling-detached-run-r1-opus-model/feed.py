#!/usr/bin/env python3
"""Feed hook-input JSON to the copied hooks; print exit code and first stderr line per hook."""
import json, os, subprocess, sys
REPO = "/tmp/claude-1000/tooling-detached-run-r1-opus/repo"
HOOKS = ["bash-command-detector.sh", "heavy-test-guard.sh"]
def run(case_name, agent_type, command, background=False):
    payload = {"tool_name": "Bash", "tool_input": {"command": command, "run_in_background": background},
               "session_id": "opus-r1-probe", "cwd": REPO, "transcript_path": "/dev/null"}
    if agent_type:
        payload["agent_type"] = agent_type
    env = dict(os.environ, AGENT_HOOK_DETECTIONS="/tmp/claude-1000/tooling-detached-run-r1-opus/t3/detections.jsonl",
               CLAUDE_PROJECT_DIR=REPO)
    env.pop("SINGLEFS_HEAVY_TESTS", None)
    out = []
    for hook in HOOKS:
        p = subprocess.run(["bash", os.path.join(REPO, ".claude/hooks", hook)], input=json.dumps(payload),
                           capture_output=True, text=True, cwd=REPO, env=env, timeout=120)
        first = next((l.strip() for l in p.stderr.splitlines() if l.strip()), "")
        out.append((hook.split("-")[0], p.returncode, first[:160]))
    return out
if __name__ == "__main__":
    cases = json.load(open(sys.argv[1]))
    for case in cases:
        name, agent, cmd = case[0], case[1], case[2]
        bg = case[3] if len(case) > 3 else False
        res = run(name, agent, cmd, bg)
        print(f"## {name}\n   agent={agent or '主agent'} bg={bg}\n   cmd: {cmd}")
        for hook, rc, first in res:
            print(f"   {hook}: rc={rc}  {first}")
