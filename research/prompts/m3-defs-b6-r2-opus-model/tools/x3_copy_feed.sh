#!/usr/bin/env bash
# X3 仓副本那一支：在草稿目录（工作根底下）造仓副本 wc-body（改登记了的 write-guard.sh 本身），子 agent 与主 agent（带 / 不带前缀）各喂一遍 heavy-test-guard.sh 入口，打印退出码与 stderr 全文。
# 用法：x3_copy_feed.sh <被判仓根> <草稿根>
set -u
repo=$(cd "$1" && pwd); scratch=$(cd "$2" && pwd)
c="$scratch/wc-body"; rm -rf "$c"; mkdir -p "$c/.claude"; cp -a "$repo/.claude/hooks" "$c/.claude/hooks"; printf '\n# 副本里改的\n' >> "$c/.claude/hooks/write-guard.sh"
feed() {   # $1 标签 $2 agent_type（空 = 主 agent） $3 命令
  local json err rc
  json=$(python3 -c 'import json,sys; d={"tool_name":"Bash","cwd":sys.argv[2],"tool_input":{"command":sys.argv[3]}}
if sys.argv[1]: d["agent_type"]=sys.argv[1]
print(json.dumps(d))' "$2" "$scratch" "$3")
  err=$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" 2>&1 >/dev/null); rc=$?
  printf '== %s rc=%s\n%s\n' "$1" "$rc" "$(printf '%s' "$err" | sed "s#$scratch#<SCRATCH>#g; s#$repo#<REPO>#g")"
}
feed K1-subagent tooling-writer "bash $c/.claude/hooks/write-guard.sh --selftest"
feed K2-main-no-prefix "" "bash $c/.claude/hooks/write-guard.sh --selftest"
feed K3-main-user-request "" "SINGLEFS_HEAVY_TESTS=user-request bash $c/.claude/hooks/write-guard.sh --selftest"
feed K4-main-commit "" "SINGLEFS_HEAVY_TESTS=commit bash $c/.claude/hooks/write-guard.sh --selftest"
feed K5-subagent-user-request tooling-writer "SINGLEFS_HEAVY_TESTS=user-request bash $c/.claude/hooks/write-guard.sh --selftest"
