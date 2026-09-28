#!/usr/bin/env bash
# 样本：.claude/scripts/ 里的包装。拒绝不带出路；并行跑完用不带参数的 wait 收，红了几个它一个字都不说。
# 只有 .claude/scripts/ 进了射程，这两处才报得出来。
shared="$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/check.sh"
if [[ ! -f "$shared" ]]; then
  echo "  ✗ 找不到共享脚本：$shared"
  exit 1
fi
for part in first second; do
  bash "$shared" "$part" &
done
wait
