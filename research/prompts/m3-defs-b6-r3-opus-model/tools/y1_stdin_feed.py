#!/usr/bin/env python3
"""Y1 的 stdin 入口复核：把 y1_fence_feed.py 写下的几份提示按 PreToolUse:Agent 的 JSON 喂派发闸入口（钩子真正被调的那一条路）。
每份提示用自己的草稿目录（放行时在里面建 .writer-lock）；DISPATCH_GUARD_STATE_ROOT 指到 --work 里，不碰会话共用的登记。
用法：y1_stdin_feed.py --root <名>=<副本根> ... --prompts <y1_fence_feed.py 的 --work> --work <新目录> 名字…"""
import argparse, json, os, subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--root", action="append", required=True)
parser.add_argument("--prompts", required=True)
parser.add_argument("--work", required=True)
parser.add_argument("names", nargs="+")
arguments = parser.parse_args()
for name in arguments.names:
    original = open(os.path.join(arguments.prompts, name + ".txt"), encoding="utf-8").read()
    cells = []
    for label, root in (entry.split("=", 1) for entry in arguments.root):
        scratch = os.path.join(arguments.work, f"{name}-{label}")
        os.makedirs(scratch, exist_ok=True)
        text = original.replace(os.path.join(arguments.prompts, "fake-scratch"), scratch)
        payload = json.dumps({"hook_event_name": "PreToolUse", "tool_name": "Agent", "tool_input": {"subagent_type": "investigator", "prompt": text}, "cwd": root})
        environment = dict(os.environ, DISPATCH_GUARD_STATE_ROOT=os.path.join(arguments.work, "state"), CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1")
        done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh")], input=payload, capture_output=True, text=True, cwd="/", env=environment, timeout=120)
        first = next((line.strip() for line in done.stderr.split("\n") if line.strip().startswith("✗")), "-")
        cells.append(f"{label} 退出={done.returncode} {first[:90]}")
    print(f"{name}\n    " + "\n    ".join(cells))
