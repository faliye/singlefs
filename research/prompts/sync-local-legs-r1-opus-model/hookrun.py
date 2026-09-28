#!/usr/bin/env python3
"""Feed a PreToolUse Bash JSON to the three registered Bash hooks (read-only repo), detections go to a scratch file."""
import json, os, subprocess, sys
REPO = "<仓根>"
DRAFT = os.environ.get("ATTACK_DRAFT", "/tmp/claude-1000/sync-local-legs-r1-attack")
DET = os.path.join(DRAFT, "detections.jsonl")
HOOKS = [
    ("pattern", os.path.join(REPO, ".claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh")),
    ("detector", os.path.join(REPO, ".claude/hooks/bash-command-detector.sh")),
    ("heavy", os.path.join(REPO, ".claude/hooks/heavy-test-guard.sh")),
]
def run(name, agent_type, bg, command):
    payload = {"session_id": "fake-session-attack", "transcript_path": os.path.join(DRAFT, "fake.jsonl"),
               "cwd": REPO, "hook_event_name": "PreToolUse", "tool_name": "Bash",
               "tool_input": {"command": command, "description": "x", **({"run_in_background": True} if bg else {})}}
    if agent_type:
        payload["agent_type"] = agent_type; payload["agent_id"] = "a0000fake0000"
    env = dict(os.environ, AGENT_HOOK_DETECTIONS=DET, CLAUDE_PROJECT_DIR=REPO)
    before = sum(1 for _ in open(DET)) if os.path.exists(DET) else 0
    out = []
    for hook_name, path in HOOKS:
        p = subprocess.run(["bash", path], input=json.dumps(payload), capture_output=True, text=True, env=env, cwd=REPO)
        first = (p.stderr.strip().splitlines() or [""])[0][:140]
        out.append(f"{hook_name}={p.returncode}" + (f"[{first}]" if p.returncode else ""))
    after = sum(1 for _ in open(DET)) if os.path.exists(DET) else 0
    print(f"{name}\t{agent_type or 'main'}\tbg={bg}\t{' '.join(out)}\tdetections+={after-before}")
cases = json.load(open(sys.argv[1]))
for c in cases:
    run(*c)
