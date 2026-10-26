#!/usr/bin/env bash
# K2 例外说法与 merge-and-verify.sh 打出的重跑命令：从脚本里抽出 checker_quick_rerun_command 原文（不改一个字），
# 按主工作区的取值（root、THREADS 31、MEMORY 8G、两格）打出那一条，再喂主 agent 身份的 heavy-test-guard.sh 与 run-scope-guard.sh 入口看退出码。
# 用法：bash rerun_command_feed.sh <仓根（只读）> <钩子目录>
set -uo pipefail
R="$1" HOOKS="$2"
body="$(awk '/^checker_quick_rerun_command\(\) \{/,/^}/' "$R/scripts/gate/merge-and-verify.sh")"
[[ -n "$body" ]] || { echo "抽不到 checker_quick_rerun_command"; exit 1; }
eval "$body"
root="$R" THREADS=31 MEMORY=8G CHECKER_QUICK_STAGE="$R/.claude/gate.d/checker-tier-crash-point-replay.sh"
for heavy in "" user-request; do
  command="$(SINGLEFS_HEAVY_TESTS="$heavy" checker_quick_rerun_command crash-case-replay,feature-gated-tests "$R")"
  echo "打出的命令（起 merge-and-verify 时 SINGLEFS_HEAVY_TESTS=${heavy:-没设}）：$command"
  for hook in heavy-test-guard.sh run-scope-guard.sh; do
    input="$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2],"hook_event_name":"PreToolUse"}))' "$command" "$R")"
    out="$(CLAUDE_PROJECT_DIR="$R" bash "$HOOKS/$hook" <<< "$input" 2>&1)"; rc=$?
    echo "  主 agent 身份喂 $hook：退 $rc ${out:0:300}"
  done
done
