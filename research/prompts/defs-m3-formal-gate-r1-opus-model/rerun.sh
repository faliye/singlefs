#!/usr/bin/env bash
# 复跑这条腿的全部探针：bash rerun.sh <仓根> <草稿目录>（草稿目录要不存在或是空的）。不跑门禁、不起 Verus、不编；hook 只喂 JSON。
set -uo pipefail
REPO="${1:?仓根}"; WORK="${2:?草稿目录}"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$WORK"
python3 "$HERE/d1-hook-cases.py" "$REPO" "$WORK" > "$WORK/d1-output.txt" 2>&1
bash "$HERE/d2-reuse.sh" "$REPO" "$WORK" > "$WORK/d2-output.txt" 2>&1
bash "$HERE/d1b-untracked-results.sh" "$WORK" > "$WORK/d1b-output.txt" 2>&1
for variant in F1 F1F2; do
  mkdir -p "$WORK/$variant-src"
  rsync -a --exclude .git "$WORK/repo/" "$WORK/$variant-src/"
  python3 "$HERE/fix-draft.py" "$WORK/$variant-src" "$variant"
  bash "$HERE/d2-reuse.sh" "$WORK/$variant-src" "$WORK/$variant-work" > "$WORK/d2-$variant-output.txt" 2>&1
done
for name in d2-output d2-F1-output d2-F1F2-output; do
  echo "### $name"; awk -F'\t' -f "$HERE/summarize.awk" "$WORK/$name.txt"
done > "$WORK/d2-summary.txt"
cat "$WORK/d2-summary.txt"
