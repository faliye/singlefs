#!/usr/bin/env bash
# 用法：run_row.sh <行号> <工作目录>：在 <工作目录>/repo 上打 rows.tsv 第 <行号> 行变异、跑它第 5 列的 cargo test、从冻结树还原；
# 日志 <工作目录>/out/row<n>.log，摘要打到 stdout（第 6 列点名的用例那一行、test result、头几处 panic）。
set -u
n="$1"
WORK="$2"
MODEL="$(cd "$(dirname "$0")" && pwd)"
FROZEN=/tmp/claude-1000/m3-formal-code-b17-r1/tree
REPO="$WORK/repo"
args="$(awk -F'\t' -v n="$n" 'NR==n{print $5}' "$MODEL/rows.tsv")"
name="$(awk -F'\t' -v n="$n" 'NR==n{print $6}' "$MODEL/rows.tsv")"
python3 "$MODEL/apply_row.py" "$MODEL/rows.tsv" "$n" "$REPO" apply "$FROZEN" || { echo "row=$n apply_failed"; exit 3; }
cd "$REPO" || exit 3
# shellcheck disable=SC2086
cargo test --offline $args > "$WORK/out/row$n.log" 2>&1
rc=$?
python3 "$MODEL/apply_row.py" "$MODEL/rows.tsv" "$n" "$REPO" revert "$FROZEN" >/dev/null
named_line="$(grep -E "^test ${name} \.\.\. " "$WORK/out/row$n.log" | head -1)"
echo "row=$n cargo_rc=$rc named_test=[$named_line]"
grep -E 'test result' "$WORK/out/row$n.log" | sed "s/^/row=$n /"
grep -E "panicked at|assertion .*failed|^  left|^ right" "$WORK/out/row$n.log" | head -6 | cut -c1-300 | sed "s/^/row=$n /"
