#!/usr/bin/env bash
# 用法：bash feed-dispatch.sh <草稿目录>
# dispatch/<subagent_type>--<名字>.txt 的正文当派发提示，合成 PreToolUse[Agent] JSON 喂 runner-dispatch-guard.sh，打印退出码与 stderr 首行。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; draft="$1"
for prompt_file in "$here"/dispatch/*.txt; do
  name="$(basename "$prompt_file" .txt)"; subagent="${name%%--*}"
  json="$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Agent","cwd":sys.argv[3],"session_id":"defs-r2-opus-model","tool_input":{"subagent_type":sys.argv[1],"description":"x","prompt":open(sys.argv[2],encoding="utf-8").read()}},ensure_ascii=False))' "$subagent" "$prompt_file" "$repo")"
  err="$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" AGENT_HOOK_DETECTIONS="$draft/detections.jsonl" bash "$repo/.claude/hooks/runner-dispatch-guard.sh" 2>&1 >/dev/null)"; rc=$?
  printf '%s\trc=%s\t%s\n' "$name" "$rc" "$(printf '%s\n' "$err" | head -n 1)"
done
