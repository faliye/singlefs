#!/bin/bash
# m3-ring-r2 攻方腿模型复跑：把冻结副本拷进 <草稿目录>/repo、打 core-and-checker.diff、放进原型 crate、编译，再按批跑。
# 用法：rebuild-and-run.sh <草稿目录> [build|g1|direct|sweep|selfred|rollback|g4|all] [树，默认 /tmp/claude-1000/m3-batch4-freeze/tree]
# 每条编译与跑都经 research/scripts/run-with-memory-cap.sh 与 capped.sh 4（scripts/build.sh、scripts/run.sh 里）。
set -euo pipefail
scratch=$1; what=${2:-build}; tree=${3:-/tmp/claude-1000/m3-batch4-freeze/tree}
model=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$scratch/out"
if [ ! -d "$scratch/repo" ]; then
  rsync -a --exclude target --exclude .git "$tree/" "$scratch/repo/"
  (cd "$scratch/repo" && patch -p2 < "$model/core-and-checker.diff")
  mkdir -p "$scratch/repo/crates/ring-window-proto"
  cp -r "$model/ring-window-proto/." "$scratch/repo/crates/ring-window-proto/"
fi
for f in run.sh build.sh batch2.sh batch3.sh; do
  sed "s|/tmp/claude-1000/m3-ring-r2-opus|$scratch|g" "$model/scripts/$f" > "$scratch/$f.tmp" && chmod +x "$scratch/$f.tmp" && mv "$scratch/$f.tmp" "$scratch/$f"
done
cd "$scratch"
case "$what" in
  build) ./build.sh ;;
  g1) ./build.sh; ./run.sh g1-s4 fill_bytes_by_kind 4; ./run.sh g1-s8 fill_bytes_by_kind 8 ;;
  direct|sweep) ./build.sh; bash ./batch2.sh ;;
  sweep2) ./build.sh; bash ./batch3.sh ;;
  selfred) ./build.sh; SINGLEFS_WINDOW_ANCHOR=ring STATIC_NO_WINDOW=1 ./run.sh selfred-k1-s4 window_static_check 4 k1 16 16 0 ;;
  rollback) ./build.sh; for s in 4 8; do k=$((3*s+4)); SINGLEFS_WINDOW_ANCHOR=off ./run.sh rb-$s-off rollback_to_stale $s $k; SINGLEFS_WINDOW_ANCHOR=ring SINGLEFS_CHECKER_WINDOW=1 SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) SINGLEFS_ROLLBACK_NO_WINDOW=1 ./run.sh rb-$s-ring-nowin rollback_to_stale $s $k; SINGLEFS_WINDOW_ANCHOR=ring SINGLEFS_CHECKER_WINDOW=1 SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) ./run.sh rb-$s-ring-win rollback_to_stale $s $k; done ;;
  g4) ./build.sh; ./run.sh g4-k1-s4 orphan_units_probe 4 k1 16 2000; ./run.sh g4-k1x2-s4 orphan_units_probe 4 k1x2 16 2000 ;;
  all) ./build.sh; "$0" "$scratch" g1 "$tree"; "$0" "$scratch" selfred "$tree"; "$0" "$scratch" rollback "$tree"; "$0" "$scratch" g4 "$tree"; bash ./batch2.sh; bash ./batch3.sh ;;
  *) echo "不认识的批：$what" >&2; exit 2 ;;
esac
