#!/usr/bin/env bash
# 建辩方腿样本要载入的冻结树：只拷搬迁换算要用的几份，逐个核 sha256 在开工快照里（lib_long_run.py 与 changed-paths.sh 之类快照里有的才核）。
# 用法：bash build_frozen.sh <仓根> <新的冻结树目录>
set -euo pipefail
repo="$1" out="$2"
[[ ! -e "$out" ]] || { echo "  ✗ $out 已存在" >&2; exit 2; }
for path in .claude/gate.d/lib-lag-baseline.py .claude/hooks/lib_long_run.py .claude/scripts/project_preflight.py scripts/gate/changed-paths.sh .claude/gate.d/path-moves.tsv \
  scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py; do
  mkdir -p "$out/$(dirname "$path")"
  cp -p "$repo/$path" "$out/$path"
done
snapshot="$repo/research/prompts/apparatus-rename-r3-snapshot/judged-sha256.txt"
(cd "$out" && sha256sum .claude/gate.d/lib-lag-baseline.py .claude/hooks/lib_long_run.py scripts/gate/changed-paths.sh .claude/gate.d/path-moves.tsv \
  scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py \
  | while read -r sum path; do grep -qF "$sum  $path" "$snapshot" && echo "  ✓ $path $sum" || echo "  ! $path $sum 不在开工快照里（工作区之后又改过）"; done)
echo "  冻结树建在 $out"
