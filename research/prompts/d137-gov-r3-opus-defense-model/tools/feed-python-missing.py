#!/usr/bin/env python3
"""H13 现喂：feed-python-missing.py <钩子树> <草稿目录>
PATH 只放 cat、date、mkdir、dirname 的链接（没有 python3），喂两份钩子一条 ls，打退出码与检出记录多了几行。进程组同 feed.py。"""
import json, os, signal, subprocess, sys

tree, scratch = sys.argv[1:3]
bin_dir = os.path.join(scratch, "no-python-bin")
os.makedirs(bin_dir, exist_ok=True)
for tool in ("cat", "date", "mkdir", "dirname"):
    link = os.path.join(bin_dir, tool)
    if not os.path.lexists(link):
        os.symlink(f"/usr/bin/{tool}", link)
detections = os.path.join(scratch, "python-missing-detections.jsonl")
payload = json.dumps({"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": "s-d137-gov-r3-defense",
                      "cwd": scratch, "agent_type": "implementation-writer", "tool_input": {"command": "ls", "run_in_background": False}})
for hook in ("heavy-test-guard.sh", "bash-command-detector.sh"):
    before = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    env = {"PATH": bin_dir, "HOME": os.environ["HOME"], "AGENT_HOOK_DETECTIONS": detections, "SINGLEFS_WORK_ROOT": os.path.join(scratch, "wr")}
    process = subprocess.Popen(["/bin/bash", os.path.join(tree, ".claude", "hooks", hook)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, env=env, cwd=scratch, start_new_session=True)
    try:
        out, err = process.communicate(payload, timeout=60)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL); process.communicate(); print(f"name=python-missing hook={hook} timeout"); continue
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    print(f"name=python-missing hook={hook} rc={process.returncode} detections_added={after - before}")
