#!/usr/bin/env bash
# 用法：bash g2-surface-59-copyset.sh <仓根>
# 与 g2-surface.sh 同一数法，只把 59 号的输入换成它自己拷进每一片的那几样（59 号脚本里 COPIED_INTO_EACH_SHARD 那一行，现抽）
# 加上它经手的 research/scripts/run-with-memory-cap.sh：拿 stage-inputs.tsv 那一行算窄了的话，这里给宽的那一头。
set -uo pipefail
root="$1"; cd "$root" || exit 2
mapfile -t in59 < <(grep -m1 '^COPIED_INTO_EACH_SHARD = ' .claude/gate.d/59-crates-mutation-replay.sh | grep -o '"[^"]*"' | tr -d '"')
in59+=(research/scripts/run-with-memory-cap.sh)
echo "59 号拷进每一片的：${in59[*]}"
total=0; not59=0
while IFS= read -r file; do
  total=$((total + 1)); hit=0
  for p in "${in59[@]}"; do [[ "$file" == "$p" || "$file" == "$p"/* ]] && hit=1; done
  (( hit )) || not59=$((not59 + 1))
done < <(git ls-files -- crates Cargo.toml Cargo.lock litmus research/scripts research/results)
echo "六条路径下已跟踪文件 $total 个，不在 59 号这一宽口径里的：$not59"
