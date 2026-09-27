#!/usr/bin/env python3
"""defs-m2-closeout-r1 攻方腿探针：往四份 hook 喂 PreToolUse 的 JSON，只看退出码与 stderr 第一行；被判的命令一条都不执行。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r1-opus-model/probe.py research/prompts/defs-m2-closeout-r1-opus-model/cases.json
检出记录写进 AGENT_HOOK_DETECTIONS 指的临时文件（不进会话共用的那份，看门狗读不到）。"""
import json, os, subprocess, sys, tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
HOOKS = {
    "heavy": ["bash", os.path.join(ROOT, ".claude/hooks/heavy-test-guard.sh")],
    "detector": ["bash", os.path.join(ROOT, ".claude/hooks/bash-command-detector.sh")],
    "pattern": ["bash", os.path.join(ROOT, ".claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh")],
    "ask": ["bash", os.path.join(ROOT, ".claude/hooks/ask-user-claim-guard.sh")],
}

def main():
    cases = json.load(open(sys.argv[1], encoding="utf-8"))
    detections = tempfile.NamedTemporaryFile(prefix="probe-detections-", suffix=".jsonl", delete=False).name
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, CLAUDE_PROJECT_DIR=ROOT)
    for case in cases:
        if case["hook"] == "ask":
            payload = {"tool_name": "AskUserQuestion", "session_id": "probe-defs-m2-closeout-r1-opus",
                       "tool_input": {"questions": [{"question": case["command"], "header": "probe",
                                                      "options": [{"label": "a", "description": "a"}, {"label": "b", "description": "b"}]}]}}
        else:
            payload = {"tool_name": "Bash", "session_id": "probe-defs-m2-closeout-r1-opus", "cwd": case.get("cwd", ROOT),
                       "tool_input": {"command": case["command"], "run_in_background": bool(case.get("rib", False))}}
        if case.get("agent"):
            payload["agent_type"] = case["agent"]
        result = subprocess.run(HOOKS[case["hook"]], input=json.dumps(payload), capture_output=True, text=True,
                                env=environment, cwd=ROOT, timeout=120)
        first = next((line for line in result.stderr.splitlines() if line.strip()), "")
        print("\t".join([case["id"], case["hook"], case.get("agent") or "主 agent", "rib" if case.get("rib") else "fg",
                         f"exit={result.returncode}", first[:150]]))
    print(f"# detections file: {detections}")

if __name__ == "__main__":
    main()
