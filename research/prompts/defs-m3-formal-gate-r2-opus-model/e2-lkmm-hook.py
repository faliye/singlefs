#!/usr/bin/env python3
"""E2 旁支：F1 之后 checker-tier-lkmm 的 environment=（lkmm.sh --herd7-version）每次算标记指纹都现跑；同一条命令直接起时闸怎么判。
只喂 JSON、看退出码。用法：python3 e2-lkmm-hook.py <仓根> <草稿目录>"""
import json, os, subprocess, sys
root, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
hook = os.path.join(root, ".claude/hooks/heavy-test-guard.sh")
cases = [
    ("主 agent 直接打 herd7 版本（lkmm 的 environment= 那一条）", None, "bash .claude/scripts/" + "lkmm.sh --herd7-version"),
    ("崩溃验证员不带前缀打 herd7 版本", "crash-verifier", "bash .claude/scripts/" + "lkmm.sh --herd7-version"),
    ("崩溃验证员第 1 步问 lkmm 的标记（F1 之后它在里面现跑上一条）", "crash-verifier", "python3 research/scripts/admission.py stage-marker-check . checker-tier-" + "lkmm.sh"),
    ("对照：主 agent 打 Verus 版本（这一道的 environment= 那一条）", None, "bash .claude/gate.d/checker-tier-" + "formal-verification.sh --tool-versions"),
]
for label, agent, command in cases:
    hook_input = {"tool_name": "Bash", "cwd": root, "session_id": "e2-lkmm-probe", "tool_input": {"command": command, "run_in_background": False}}
    if agent:
        hook_input["agent_type"] = agent
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl"))
    environment.pop("SINGLEFS_HEAVY_TESTS", None)
    completed = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=root)
    print(f"{completed.returncode}\t{agent or '主 agent'}\t{label}")
