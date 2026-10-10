#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=… ringwin.sh <模型二进制> <输出目录> 环:深 …   例 4:7 6:9
# 只用 create / rollback1 / empty / mount 四个字母，V1 臂、清扫阈值从不（模型 main 组第 2 项），历史长到超过环长；各件并行。
set -uo pipefail
BIN=${1:?}; OUT=${2:?}; shift 2
for p in "$@"; do
  IFS=: read -r ring depth <<<"$p"
  rm -f "$OUT/model-ringwin-$ring-$depth.exit"
  { W3_OPS=create,rollback1,empty,mount W3_RING=$ring W3_LONGEST=$depth nice -n 19 bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh" 4G "$BIN" main 2 > "$OUT/model-ringwin-$ring-$depth.out" 2> "$OUT/model-ringwin-$ring-$depth.err"; echo "$?" > "$OUT/model-ringwin-$ring-$depth.exit"; } &
done
wait
bad=0
for p in "$@"; do IFS=: read -r ring depth <<<"$p"; [ "$(cat "$OUT/model-ringwin-$ring-$depth.exit" 2>/dev/null)" = 0 ] || bad=1; done
echo "ringwin 有非 0=$bad"
exit $bad
