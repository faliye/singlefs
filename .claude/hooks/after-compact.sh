#!/usr/bin/env bash
# SessionStart hook（matcher compact）：上下文压缩之后，把接续工作要先对齐的几件事补回上下文。
#
# 为什么：长会话里任务转向时要尽快压缩上下文（用户 2026-09-17）；模型与 hook 都触发不了压缩，只能由用户 `/compact <保留什么>`。
# 压缩摘要会丢细节，接续最容易错的是三件：还有没有子 agent 在跑、工作区里哪些是没提交的、这件活的状态记在哪份记录里。
# SessionStart 在 compact 之后跑，标准输出会进新上下文，所以在这里补。只报、不改任何东西。
#
#   after-compact.sh             # 从 stdin 读 hook 的 JSON，往标准输出写一段对齐提示，退出码恒 0
#   after-compact.sh --selftest  # 喂一段假 JSON，核输出里有分支、未提交数、最近的记录、看门狗命令；AFTER_COMPACT_DISABLE=1 时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, subprocess, sys

def reminder(hook_input, project_root):
    if os.environ.get("AFTER_COMPACT_DISABLE") == "1":
        return ""
    def git(*arguments):
        completed = subprocess.run(["git", "-C", project_root, *arguments], capture_output=True, text=True)
        return completed.stdout.strip() if completed.returncode == 0 else ""
    branch = git("branch", "--show-current") or "（读不出）"
    status_lines = [line for line in git("status", "--porcelain").split("\n") if line.strip()]
    records_dir = os.path.join(project_root, "records")
    records = []
    if os.path.isdir(records_dir):
        records = sorted((name for name in os.listdir(records_dir) if name.endswith(".md")),
                         key=lambda name: os.path.getmtime(os.path.join(records_dir, name)), reverse=True)[:3]
    transcript = hook_input.get("transcript_path") or ""
    session_id = hook_input.get("session_id") or ""
    session_dir = os.path.join(os.path.dirname(transcript), session_id) if transcript and session_id else "<会话目录>"
    lines = [
        "【上下文刚压缩过，接着干活之前先对齐这几件】",
        f"- 分支 {branch}；工作区未提交 {len(status_lines)} 个路径（`git status --short` 看清哪些是这一轮的，别的会话的不碰）。",
        "- 最近动过的记录：" + ("、".join(f"records/{name}" for name in records) if records else "（没有）") + "；接续的状态以记录里的状态表与岔路单为准，不凭压缩摘要里的印象。",
        f"- 派出过子 agent 的，先看还有没有在跑：`python3 research/scripts/agent-watch.py report --session-dir {session_dir}`；在跑的照 CLAUDE.md「派出去之后要盯住」重起看门狗。",
    ]
    return "\n".join(lines)

def selftest(hook_dir):
    script = os.path.join(hook_dir, "after-compact.sh")
    project_root = os.path.dirname(os.path.dirname(hook_dir))
    hook_input = {"hook_event_name": "SessionStart", "source": "compact", "session_id": "abc",
                  "transcript_path": "/home/x/.claude/projects/p/abc.jsonl"}
    completed = subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True,
                               env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
    wanted = ["上下文刚压缩过", "分支 ", "工作区未提交", "最近动过的记录", "agent-watch.py report --session-dir /home/x/.claude/projects/p/abc"]
    missing = [piece for piece in wanted if piece not in completed.stdout]
    failures = []
    if completed.returncode != 0:
        failures.append(f"退出码应当是 0，实际 {completed.returncode}")
    if missing:
        failures.append(f"输出里缺：{'、'.join(missing)}")
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 reminder() 与入口；AFTER_COMPACT_DISABLE 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：压缩后的对齐提示里有分支、未提交数、最近的记录与看门狗命令，退出码 0（查了 {len(wanted) + 1} 项）")
    return 0

hook_dir = sys.argv[1]
if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest(hook_dir))
try:
    hook_input = json.load(sys.stdin)
except ValueError:
    hook_input = {}
project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
text = reminder(hook_input, project_root)
if text:
    print(text)
sys.exit(0)
PY
