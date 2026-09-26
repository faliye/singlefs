#!/usr/bin/env bash
# 用法：probe.sh <hook 名> <agent_type 或 -（主 agent）> <cwd> <命令>
# 把一条合成的 PreToolUse[Bash] JSON 喂给 .claude/hooks/<hook 名>，打印退出码与 stderr 首行；检出记录写进 $SCRATCH/detections.jsonl
set -uo pipefail
REPO=/home/user/singlefs
SCRATCH=${SCRATCH:-/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/leg-opus}
hook=$1 agent=$2 cwd=$3 command=$4 background=${5:-false}
json=$(python3 - "$agent" "$cwd" "$command" "$background" <<'PY'
import json, sys
agent, cwd, command, background = sys.argv[1:5]
payload = {"session_id": "probe-opus", "hook_event_name": "PreToolUse", "tool_name": "Bash", "cwd": cwd,
           "tool_input": {"command": command, "run_in_background": background == "true"}}
if agent != "-":
    payload["agent_type"] = agent
print(json.dumps(payload))
PY
)
err=$(printf '%s' "$json" | AGENT_HOOK_DETECTIONS="$SCRATCH/detections.jsonl" bash "$REPO/.claude/hooks/$hook" 2>&1 >/dev/null)
code=$?
printf 'exit=%s agent=%s hook=%s\n  cmd: %s\n' "$code" "$agent" "$hook" "$command"
[ -n "$err" ] && printf '%s\n' "$err" | head -n "${LINES_SHOWN:-3}" | sed 's/^/  | /'
exit 0
