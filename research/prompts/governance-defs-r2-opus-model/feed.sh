#!/usr/bin/env bash
# 用法：bash feed.sh <草稿目录> <agent_type 或 -（主 agent）> <run_in_background: 0|1> <命令文件>
# 把合成的 PreToolUse JSON 依次喂给项目 settings 里挂在 Bash 上的三个钩子，打印各自的退出码与 stderr 首行。
set -uo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
draft="$1"; agent="$2"; background="$3"; command_file="$4"
mkdir -p "$draft"
json="$(python3 - "$agent" "$background" "$command_file" "$repo" <<'PY'
import json, sys
agent, background, path, repo = sys.argv[1:5]
command = open(path, encoding="utf-8").read()
payload = {"tool_name": "Bash", "cwd": repo, "session_id": "defs-r2-opus-model",
           "tool_input": {"command": command, "run_in_background": background == "1"}}
if agent != "-":
    payload["agent_type"] = agent
print(json.dumps(payload, ensure_ascii=False))
PY
)"
for hook in .claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh .claude/hooks/bash-command-detector.sh .claude/hooks/heavy-test-guard.sh; do
  err="$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" AGENT_HOOK_DETECTIONS="$draft/detections.jsonl" bash "$repo/$hook" 2>&1 >/dev/null)"
  rc=$?
  first="$(printf '%s\n' "$err" | head -n 1)"
  printf '%s\trc=%s\t%s\n' "$(basename "$hook")" "$rc" "$first"
done
