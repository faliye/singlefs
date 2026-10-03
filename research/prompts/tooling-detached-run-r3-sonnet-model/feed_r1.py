#!/usr/bin/env python3
import json, os, subprocess
W = "/tmp/claude-1000/tooling-detached-run-r3-sonnet/work-r1"
CASES = [
 ("clerk 真名 start", "clerk", "bash research/scripts/detached-run.sh start --label m -- sleep 1"),
 ("clerk source", "clerk", "source research/scripts/detached-run.sh start --label m -- sleep 1"),
 ("clerk .", "clerk", ". research/scripts/detached-run.sh start --label m -- sleep 1"),
 ("clerk cp 改名再起", "clerk", "cp research/scripts/detached-run.sh /tmp/dr2.sh && bash /tmp/dr2.sh start --label m -- sleep 1"),
 ("主agent 无前缀 改名拷贝 全量", None, "bash /tmp/dr2.sh start --label p -- cargo test --all"),
 ("clerk 改名拷贝 全量", "clerk", "bash /tmp/dr2.sh start --label p -- cargo test --all"),
]
for name, agent, cmd in CASES:
    payload = {"tool_name": "Bash", "tool_input": {"command": cmd, "run_in_background": False}, "session_id": "u1-r1", "cwd": W}
    if agent: payload["agent_type"] = agent
    env = {k: v for k, v in os.environ.items() if k != "SINGLEFS_HEAVY_TESTS"}
    env.update(AGENT_HOOK_DETECTIONS=os.path.join(W, "detections.jsonl"), CLAUDE_PROJECT_DIR=W)
    got = [subprocess.run(["bash", os.path.join(W, ".claude/hooks", h)], input=json.dumps(payload), capture_output=True, text=True, cwd=W, env=env, timeout=120).returncode
           for h in ("bash-command-detector.sh", "heavy-test-guard.sh")]
    print(f"{name}\tdetector={got[0]}\theavy={got[1]}")
