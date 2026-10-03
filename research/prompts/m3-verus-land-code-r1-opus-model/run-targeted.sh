#!/usr/bin/env bash
# 用法：run-targeted.sh <副本根> <日志目录> <表.tsv> <行名> <测试目标>…
# 套上那一行 → 只跑 singlefs-harness 里点名的测试目标（--no-fail-fast）→ 换回。汇总行写进 <日志目录>/targeted.txt。外面由调用方套内存包装与线程上限。
set -u
tree=$1; logs=$2; table=$3; item=$4; shift 4
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
mkdir -p "$logs"
targets=()
for target in "$@"; do targets+=(--test "$target"); done
python3 "$here/apply-mutation.py" "$tree" "$table" "$item" || exit 2
start=$(date +%s)
(cd "$tree" && cargo test --offline -p singlefs-harness "${targets[@]}" --no-fail-fast) > "$logs/$item.targeted.log" 2>&1
status=$?
failed=$(grep -E '^test .* \.\.\. FAILED$' "$logs/$item.targeted.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort -u | tr '\n' ' ')
passed=$(grep '^test result' "$logs/$item.targeted.log" | awk '{p += $4} END {print p + 0}')
echo "$item targets=$# rc=$status seconds=$(( $(date +%s) - start )) result_lines=$(grep -c '^test result' "$logs/$item.targeted.log") passed=$passed failed: $failed" >> "$logs/targeted.txt"
python3 "$here/apply-mutation.py" "$tree" --restore
