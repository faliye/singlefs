#!/usr/bin/env bash
# 建辩方腿用的冻结副本：只拷样本要载入的五份，逐个核 sha256 对得上开工快照里的那几行。
# 用法：bash build_frozen.sh <仓根> <新的冻结副本目录>
set -euo pipefail
repo="$1" out="$2"
[[ ! -e "$out" ]] || { echo "  ✗ $out 已存在" >&2; exit 2; }
for path in .claude/gate.d/lib-lag-baseline.py .claude/hooks/lib_long_run.py .claude/scripts/project_preflight.py scripts/gate/changed-paths.sh .claude/gate.d/path-moves.tsv; do
  mkdir -p "$out/$(dirname "$path")"
  cp -p "$repo/$path" "$out/$path"
done
(cd "$out" && sha256sum .claude/gate.d/lib-lag-baseline.py .claude/hooks/lib_long_run.py scripts/gate/changed-paths.sh .claude/gate.d/path-moves.tsv \
  | while read -r sum path; do grep -F "$sum  $path" "$repo/research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt" >/dev/null || { echo "  ✗ $path 的 sha256 $sum 不在开工快照里" >&2; exit 1; }; done)
echo "  ✓ 冻结副本建在 $out，四份被判文件的 sha256 都在开工快照里"
