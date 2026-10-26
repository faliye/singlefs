#!/usr/bin/env bash
# 从仓根拷出开工快照 judged-sha256.txt 点名的那 15 份与它们载入的库，建一棵冻结树，逐个核 sha256 对得上开工快照。
# 用法：bash build_frozen.sh <仓根> <新的冻结树目录>
set -euo pipefail
repo="$1" out="$2"
[[ ! -e "$out" ]] || { echo "  ✗ $out 已存在" >&2; exit 2; }
snapshot="$repo/research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt"
mkdir -p "$out"
while read -r _sum path; do mkdir -p "$out/$(dirname "$path")"; cp -p "$repo/$path" "$out/$path"; done < "$snapshot"
mkdir -p "$out/.claude/singlefs-ai-sop" "$out/.claude/hooks"
rsync -a "$repo/.claude/scripts" "$out/.claude/"
rsync -a --exclude fixtures "$repo/.claude/singlefs-ai-sop/scripts" "$out/.claude/singlefs-ai-sop/"
for f in "$repo"/.claude/hooks/*.py; do [[ -e "$out/.claude/hooks/$(basename "$f")" ]] || cp -p "$f" "$out/.claude/hooks/"; done
d=scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes
rsync -a --exclude __pycache__ --ignore-existing "$repo/$d/" "$out/$d/"
(cd "$out" && sha256sum -c "$snapshot")
