#!/usr/bin/env python3
"""defs-m2-closeout-r3 云端攻方腿探针：往 hook 喂 PreToolUse 的 JSON，只看退出码与 stderr；被判的命令一条都不执行。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r3-opus-model/probe.py <cases.json> [--full-stderr]
  hook 名：heavy（heavy-test-guard.sh）/ detector（bash-command-detector.sh）/ ask（ask-user-claim-guard.sh 今天的文件）。
  case 的 cwd 里写 @COPY@ 的，换成环境变量 PROBE_COPY_ROOT。
检出记录写进 AGENT_HOOK_DETECTIONS 指的临时文件（不进会话共用的那份）。"""
import json, os, subprocess, sys, tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))

def main():
    cases = json.load(open(sys.argv[1], encoding="utf-8"))
    full = "--full-stderr" in sys.argv[2:]
    hooks = {
        "heavy": ["bash", os.path.join(ROOT, ".claude/hooks/heavy-test-guard.sh")],
        "detector": ["bash", os.path.join(ROOT, ".claude/hooks/bash-command-detector.sh")],
        "ask": ["bash", os.path.join(ROOT, ".claude/hooks/ask-user-claim-guard.sh")],
    }
    copy_root = os.environ.get("PROBE_COPY_ROOT", "")
    detections = tempfile.NamedTemporaryFile(prefix="probe-detections-", suffix=".jsonl", delete=False).name
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, CLAUDE_PROJECT_DIR=ROOT)
    for case in cases:
        cwd = case.get("cwd", ROOT).replace("@COPY@", copy_root)
        command = case["command"].replace("@COPY@", copy_root)
        if case["hook"] == "ask":
            payload = {"tool_name": "AskUserQuestion", "session_id": "probe-defs-m2-closeout-r3-opus",
                       "tool_input": {"questions": [{"question": command, "header": "probe",
                                                      "options": [{"label": "a", "description": "a"}, {"label": "b", "description": "b"}]}]}}
        else:
            payload = {"tool_name": "Bash", "session_id": "probe-defs-m2-closeout-r3-opus", "cwd": cwd,
                       "tool_input": {"command": command, "run_in_background": bool(case.get("rib", False))}}
        if case.get("agent"):
            payload["agent_type"] = case["agent"]
        result = subprocess.run(hooks[case["hook"]], input=json.dumps(payload), capture_output=True, text=True,
                                env=environment, cwd=ROOT, timeout=120)
        lines = [line for line in result.stderr.splitlines() if line.strip()]
        shown = lines if full else lines[:1]
        print("\t".join([case["id"], case["hook"], case.get("agent") or "主 agent", "rib" if case.get("rib") else "fg",
                         f"exit={result.returncode}", (shown[0][:200] if shown else "")]))
        for extra in shown[1:]:
            print("\t\t" + extra)
    with open(detections, encoding="utf-8") as handle:
        count = sum(1 for line in handle if line.strip())
    os.unlink(detections)
    print(f"# detections: {count} line(s)")

if __name__ == "__main__":
    main()
