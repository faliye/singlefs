#!/usr/bin/env bash
# K4：同一棵树（本仓 c1e9b324 的 --shared 克隆，工作区干净）上，三种基准下用 gate_diff_base gate 的格看到的改动范围，
# 与 archive-past-rounds.py「上一轮」那一批（DIRS 下已跟踪、相对基准没改、不是判决与 abandoned-rounds 的）各多少。
# 用法：k4-scope.sh <冻结副本> <克隆根> <staged-green 提交>
set -uo pipefail
FROZEN="$1"; CLONE="$2"; SG="$3"; cd "$CLONE" || exit 2
source "$FROZEN/scripts/gate/changed-paths.sh"
for label in today_N:HEAD today_S:HEAD~1 geng2:"$SG"; do
  name="${label%%:*}"; base="${label#*:}"
  paths="$(gate_changed_paths "$base" untracked | wc -l)"
  dirty="$(git diff --name-only "$base" -- research/results research/prompts | sort -u)"
  past="$(git ls-files -- research/results research/prompts | grep -vE -- '-main-verification\.md$|abandoned-rounds\.tsv$' | sort -u | comm -23 - <(printf '%s\n' "$dirty" | sed '/^$/d') | wc -l)"
  echo "name=k4_scope label=$name base=$(git rev-parse --short=12 "$base") changed_paths=$paths archive_past_round_files=$past"
done
