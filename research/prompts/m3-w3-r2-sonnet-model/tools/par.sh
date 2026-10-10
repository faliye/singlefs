#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<仓根> par.sh <装置二进制目录> <输出目录> <件> …   件 = 臂:族:起:止（序号闭区间）
# 各件并行，结果写 <输出目录>/dev-<族>-<臂>-<起>-<止>.out，退出码写 <输出目录>/piece-<族>-<臂>-<起>-<止>.exit，全部等齐后数文件、有非 0 退 1。
set -uo pipefail
BIN=${1:?}; OUT=${2:?}; shift 2
HERE=$(cd "$(dirname "$0")" && pwd)
for piece in "$@"; do
  IFS=: read -r arm fam lo hi <<<"$piece"
  rm -f "$OUT/piece-$fam-$arm-$lo-$hi.exit" "$OUT/dev-$fam-$arm-$lo-$hi.out"
  { bash "$HERE/dev_run.sh" "$BIN" "$arm" "$fam" "$OUT/dev-$fam-$arm-$lo-$hi.out" "$(seq -s, "$lo" "$hi")"; echo "$?" > "$OUT/piece-$fam-$arm-$lo-$hi.exit"; } &
done
wait
bad=0; seen=0
for piece in "$@"; do
  IFS=: read -r arm fam lo hi <<<"$piece"
  f="$OUT/piece-$fam-$arm-$lo-$hi.exit"
  if [ -f "$f" ]; then seen=$((seen+1)); [ "$(cat "$f")" = 0 ] || bad=1; else bad=1; fi
done
echo "件数=$# 退出码文件=$seen 有非 0=$bad"
exit $bad
