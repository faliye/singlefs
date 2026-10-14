#!/usr/bin/env bash
# F-Y2 试改的对照：嵌套 env -C 落到 harness 包或别的非 checker 目录的，试改之后仍应放行。
# 用法：y2_allow_feed.sh <被判仓根>
set -u
repo=$(cd "$1" && pwd)
cwd="$repo/crates/singlefs-harness"
W="bash ../../scripts/hosts/run-with-memory-cap.sh 2G"
feed() {
  local json rc
  json=$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","cwd":sys.argv[1],"agent_type":"implementation-writer","tool_input":{"command":sys.argv[2]}}))' "$cwd" "$2")
  printf '%s' "$json" | PYTHONDONTWRITEBYTECODE=1 CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" >/dev/null 2>&1; rc=$?
  printf '%s\trc=%s\n' "$1" "$rc"
}
feed A1-nested-back-to-harness "$W env -C .. env -C singlefs-harness cargo test --lib"
feed A2-nested-abs-second "$W env -C .. env -C $cwd cargo test --lib"
feed A3-single-env-two-C-last-wins "$W env -C ../singlefs-checker-tier -C . cargo test --lib"
