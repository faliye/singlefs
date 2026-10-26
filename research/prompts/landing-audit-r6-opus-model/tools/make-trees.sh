#!/usr/bin/env bash
# 建三棵钩子树：tree-now（仓根此刻的）、tree-v35（任务 35 换上之后、任务 37 之前）、tree-v10（第十批，第十一批之前，取暂存区那一版存进 orig/v10）。
# 用法：bash make-trees.sh <仓根> <模型目录> <草稿目录>
set -euo pipefail
R="$1" M="$2" S="$3"
for tree in tree-now tree-v35 tree-v10; do
  mkdir -p "$S/$tree/.claude/singlefs-ai-sop" "$S/$tree/.claude/scripts" "$S/$tree/scripts"
  cp -a "$R/.claude/hooks" "$R/.claude/agents" "$S/$tree/.claude/"
  cp "$R/.claude/heavy-thread-default" "$S/$tree/.claude/"
  cp -a "$R/.claude/singlefs-ai-sop/scripts" "$S/$tree/.claude/singlefs-ai-sop/"
  cp "$R/.claude/scripts/project_preflight.py" "$S/$tree/.claude/scripts/"
  cp -a "$R/scripts/agents" "$R/scripts/kb" "$R/scripts/hosts" "$S/$tree/scripts/"
done
for f in handback-guard.sh lib_three_way_legs.py runner-dispatch-guard.sh; do cp "$M/orig/v35/$f.orig" "$S/tree-v35/.claude/hooks/$f"; done
for f in handback-guard.sh lib_three_way_legs.py runner-dispatch-guard.sh lib_prompt_scratch_line.py heavy-test-guard.sh; do cp "$M/orig/v10/$f.orig" "$S/tree-v10/.claude/hooks/$f"; done
cp "$M/orig/v10/agent-watch.py.orig" "$S/tree-v10/scripts/agents/agent-watch.py"
for tree in tree-now tree-v35 tree-v10; do
  echo "$tree $(sha256sum "$S/$tree/.claude/hooks/handback-guard.sh" | cut -c1-12) $(sha256sum "$S/$tree/.claude/hooks/lib_three_way_legs.py" | cut -c1-12) $(sha256sum "$S/$tree/.claude/hooks/runner-dispatch-guard.sh" | cut -c1-12) $(sha256sum "$S/$tree/scripts/agents/agent-watch.py" | cut -c1-12)"
done
