#!/usr/bin/env python3
"""defs-m2-closeout-r2 云端攻方腿探针：往 hook 喂 PreToolUse 的 JSON，只看退出码与 stderr 第一行；被判的命令一条都不执行。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r2-opus-model/probe.py <cases.json>
  hook 名：heavy / detector / dispatch（派发闸，agent 字段当 subagent_type）/ ask（今天的文件）/ ask-head（HEAD 73ba4a4 那一版，git show 取到临时目录，只读）。
  case 的 cwd 里写 @COPY@ 的，换成环境变量 PROBE_COPY_ROOT（rerun.sh 建的最小仓副本，只含 Cargo 清单与 crates 源码）。
检出记录写进 AGENT_HOOK_DETECTIONS 指的临时文件（不进会话共用的那份，看门狗读不到）。"""
import json, os, subprocess, sys, tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
HEAD_COMMIT = "73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321"

def head_copy_of_ask_hook():
    directory = tempfile.mkdtemp(prefix="probe-ask-head-")
    path = os.path.join(directory, "ask-user-claim-guard.sh")
    content = subprocess.run(["git", "-C", ROOT, "show", f"{HEAD_COMMIT}:.claude/hooks/ask-user-claim-guard.sh"],
                             capture_output=True, check=True).stdout
    with open(path, "wb") as handle:
        handle.write(content)
    return path

def main():
    cases = json.load(open(sys.argv[1], encoding="utf-8"))
    hooks = {
        "heavy": ["bash", os.path.join(ROOT, ".claude/hooks/heavy-test-guard.sh")],
        "detector": ["bash", os.path.join(ROOT, ".claude/hooks/bash-command-detector.sh")],
        "ask": ["bash", os.path.join(ROOT, ".claude/hooks/ask-user-claim-guard.sh")],
        "dispatch": ["bash", os.path.join(ROOT, ".claude/hooks/runner-dispatch-guard.sh")],
    }
    if any(case["hook"] == "ask-head" for case in cases):
        hooks["ask-head"] = ["bash", head_copy_of_ask_hook()]
    copy_root = os.environ.get("PROBE_COPY_ROOT", "")
    detections = tempfile.NamedTemporaryFile(prefix="probe-detections-", suffix=".jsonl", delete=False).name
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, CLAUDE_PROJECT_DIR=ROOT)
    for case in cases:
        cwd = case.get("cwd", ROOT).replace("@COPY@", copy_root)
        command = case["command"].replace("@COPY@", copy_root)
        if case["hook"] == "dispatch":
            payload = {"tool_name": "Agent", "session_id": "probe-defs-m2-closeout-r2-opus",
                       "tool_input": {"subagent_type": case.get("agent") or "general-purpose", "description": "probe", "prompt": command}}
        elif case["hook"].startswith("ask"):
            payload = {"tool_name": "AskUserQuestion", "session_id": "probe-defs-m2-closeout-r2-opus",
                       "tool_input": {"questions": [{"question": command, "header": "probe",
                                                      "options": [{"label": "a", "description": "a"}, {"label": "b", "description": "b"}]}]}}
        else:
            payload = {"tool_name": "Bash", "session_id": "probe-defs-m2-closeout-r2-opus", "cwd": cwd,
                       "tool_input": {"command": command, "run_in_background": bool(case.get("rib", False))}}
        if case.get("agent") and case["hook"] != "dispatch":
            payload["agent_type"] = case["agent"]
        result = subprocess.run(hooks[case["hook"]], input=json.dumps(payload), capture_output=True, text=True,
                                env=environment, cwd=ROOT, timeout=120)
        first = next((line for line in result.stderr.splitlines() if line.strip()), "")
        print("\t".join([case["id"], case["hook"], case.get("agent") or "主 agent", "rib" if case.get("rib") else "fg",
                         f"exit={result.returncode}", first[:160]]))
    with open(detections, encoding="utf-8") as handle:
        lines = [line for line in handle if line.strip()]
    for line in lines:
        record = json.loads(line)
        print("# detection: " + json.dumps({key: record.get(key) for key in ("command", "detections", "kind", "reasons") if key in record},
                                           ensure_ascii=False)[:300])
    print(f"# detections: {len(lines)} line(s) in {detections}")

if __name__ == "__main__":
    main()
