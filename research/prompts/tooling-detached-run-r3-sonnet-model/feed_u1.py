#!/usr/bin/env python3
"""U1 钩子判定：把命令写成钩子输入喂给草稿副本里的两道钩子（不执行命令）。"""
import json, os, subprocess, sys
W = "/tmp/claude-1000/tooling-detached-run-r3-sonnet/work"
CASES = [
 ("G3 判决点名样本 -- 之后 --force 全量", None, "bash research/scripts/detached-run.sh start --label p -- --force cargo test --all"),
 ("G3 带前缀 user-request 放行对照", None, "SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/detached-run.sh start --label p -- --force cargo test --all"),
 ("G3 --label 值位置 --force", None, "bash research/scripts/detached-run.sh start --label --force p -- cargo test --all"),
 ("G3 --force 在最前", None, "bash research/scripts/detached-run.sh --force start --label p -- cargo test --all"),
 ("G3 --force=1 不是整词", None, "bash research/scripts/detached-run.sh start --label p -- --force=1 cargo test --all"),
 ("G3 -- -- --force", None, "bash research/scripts/detached-run.sh start --label p -- -- --force cargo test --all"),
 ("G3 引号里的 --force", None, "bash research/scripts/detached-run.sh start --label p -- '--force' cargo test --all"),
 ("对照 无前缀 全量 cargo（无 detached）", None, "cargo test --all"),
 ("对照 主 agent 真名 start sleep", None, "bash research/scripts/detached-run.sh start --label m -- sleep 1"),
]
extra = dict(a.split("=", 1) for a in sys.argv[1:])
for name, agent, cmd in CASES:
    payload = {"tool_name": "Bash", "tool_input": {"command": cmd, "run_in_background": False}, "session_id": "u1-probe", "cwd": W}
    if agent: payload["agent_type"] = agent
    env = {k: v for k, v in os.environ.items() if k != "SINGLEFS_HEAVY_TESTS"}
    env.update(extra, AGENT_HOOK_DETECTIONS=os.path.join(W, "detections.jsonl"), CLAUDE_PROJECT_DIR=W)
    got = [subprocess.run(["bash", os.path.join(W, ".claude/hooks", h)], input=json.dumps(payload), capture_output=True, text=True, cwd=W, env=env, timeout=120).returncode
           for h in ("bash-command-detector.sh", "heavy-test-guard.sh")]
    print(f"{name}\tdetector={got[0]}\theavy={got[1]}")
