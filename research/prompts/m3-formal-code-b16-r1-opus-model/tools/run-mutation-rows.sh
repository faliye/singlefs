#!/usr/bin/env bash
# 在攻方副本上逐行跑 crates/mutations.tsv 的 harness 档变异行（不跑 checker 档那几行）：施加 → 按那一行第 5 列的 cargo 参数跑 → 看第 6 列点名的用例红没红 → 从冻结副本还原。
# 用法：run-mutation-rows.sh <副本根> <冻结副本根> <结果文件> <行号>…
set -u
ws="$1"; frozen="$2"; out="$3"; shift 3
tools="$(cd "$(dirname "$0")" && pwd)"
for line in "$@"; do
  row="$(sed -n "${line}p" "$ws/crates/mutations.tsv")"
  path="$(printf '%s' "$row" | cut -f2)"
  args="$(printf '%s' "$row" | cut -f5)"
  named="$(printf '%s' "$row" | cut -f6)"
  case "$args" in *singlefs-checker-tier*) echo "line=$line skipped=checker-tier" >> "$out"; continue;; esac
  python3 "$tools/apply-mutation.py" "$ws" "$line" > /dev/null || { echo "line=$line apply-failed" >> "$out"; continue; }
  log="$(dirname "$out")/row-$line.log"
  # shellcheck disable=SC2086
  (cd "$ws" && CARGO_TARGET_DIR="$(dirname "$out")/target-test" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 8 cargo test --offline $args) > "$log" 2>&1
  rc=$?
  cp "$frozen/$path" "$ws/$path"
  verdict="$(grep -E "^test ${named} \.\.\. " "$log" | sed 's/.* \.\.\. //' | head -1)"
  compile="$(grep -c '^error\[E' "$log")"
  echo "line=$line rc=$rc named=${verdict:-absent} compile_errors=$compile" >> "$out"
done
