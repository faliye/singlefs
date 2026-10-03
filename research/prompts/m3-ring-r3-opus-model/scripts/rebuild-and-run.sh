#!/bin/bash
# m3-ring-r3 攻方腿模型复跑：把冻结副本拷进 <草稿目录>/repo、打 core-and-checker.diff（patch -p1）、放进原型 crate、编译，再按批跑。
# 用法：rebuild-and-run.sh <草稿目录> <批> [树，默认 /tmp/claude-1000/m3-batch5-freeze/tree]
#   批：build | h5（G1、放槽量、自证、G4、回退）| sweep（清扫发布崩溃）| shadow（影子账漏改）| fuzz <specs/bNN.txt …>
# 编译经 run-with-memory-cap.sh 4G + capped.sh 7（scripts/build.sh），每次跑经 run-with-memory-cap.sh 1G + capped.sh 1（scripts/run.sh）。
# 并行件的退出码写 <草稿目录>/out/b<批号>-<件号>.rc，最后单独 wait（scripts/fz-batch.sh）。
set -euo pipefail
scratch=$1; what=$2; shift 2
tree=${TREE:-/tmp/claude-1000/m3-batch5-freeze/tree}
model=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$scratch/out" "$scratch/specs"
if [ ! -d "$scratch/repo" ]; then
  rsync -a --exclude target --exclude .git "$tree/" "$scratch/repo/"
  (cd "$scratch/repo" && patch -p1 -s < "$model/core-and-checker.diff")
  mkdir -p "$scratch/repo/crates/ring-window-proto"
  cp -r "$model/ring-window-proto/." "$scratch/repo/crates/ring-window-proto/"
fi
for f in run.sh build.sh fz-batch.sh; do
  sed "s|/tmp/claude-1000/m3-ring-r3-opus|$scratch|g" "$model/scripts/$f" > "$scratch/$f.tmp" && chmod +x "$scratch/$f.tmp" && mv "$scratch/$f.tmp" "$scratch/$f"
done
cp "$model/scripts/fz-summary.py" "$model/scripts/fz-reds.py" "$scratch/"
cp "$model"/scripts/specs/*.txt "$scratch/specs/"
cd "$scratch"
./build.sh
W="SINGLEFS_WINDOW_ANCHOR=ring SINGLEFS_CHECKER_WINDOW=1"
case "$what" in
  build) ;;
  h5)
    ./run.sh g1-s4 fill_bytes_by_kind 4; ./run.sh g1-s8 fill_bytes_by_kind 8
    for s in 4 8; do k=$((3*s+4)); for h in k1 fillable; do
      SINGLEFS_WINDOW_ANCHOR=off ./run.sh d-$s-$h-off window_release_direct $s $h $k
      SINGLEFS_WINDOW_ANCHOR=ring ./run.sh d-$s-$h-ring-cw0 window_release_direct $s $h $k
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) ./run.sh d-$s-$h-ring-cw1 window_release_direct $s $h $k
    done; done
    SINGLEFS_WINDOW_ANCHOR=ring STATIC_NO_WINDOW=1 ./run.sh selfred-k1-s4 window_static_check 4 k1 16 16 0
    ./run.sh g4-k1-s4 orphan_units_probe 4 k1 16 2000; ./run.sh g4-k1x2-s4 orphan_units_probe 4 k1x2 16 2000
    for s in 4 8; do k=$((3*s+4))
      SINGLEFS_WINDOW_ANCHOR=off ./run.sh rb-$s-off rollback_to_stale $s $k
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) SINGLEFS_ROLLBACK_NO_WINDOW=1 ./run.sh rb-$s-ring-nowin rollback_to_stale $s $k
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) ./run.sh rb-$s-ring-win rollback_to_stale $s $k
    done ;;
  sweep)
    for arm in sweep carry; do
      ./run.sh sw-$arm-t0-off sweep_publish_crash 4 $arm 2 4
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=12 ./run.sh sw-$arm-t0-win sweep_publish_crash 4 $arm 2 4
    done ;;
  shadow)
    for nn in 1 2 3; do
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=12 SINGLEFS_CHECKER_DEFER_WINDOW=1 ./run.sh so-win-n$nn shadow_omission 4 $nn 6
      env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=12 SINGLEFS_CHECKER_DEFER_WINDOW=1 SINGLEFS_SHADOW_NO_WINDOW=1 ./run.sh so-noshadow-n$nn shadow_omission 4 $nn 6
    done
    env $W SINGLEFS_CHECKER_WINDOW_RING_LEN=12 SINGLEFS_CHECKER_DEFER_WINDOW=1 SINGLEFS_SHADOW_NO_WINDOW=1 ./run.sh so-noshadow-n2-q shadow_omission 4 2 0 ;;
  fuzz)
    for spec in "$@"; do b=$(basename "$spec" .txt); b=${b#b}; ./fz-batch.sh "$b" "specs/$(basename "$spec")"; done ;;
  *) echo "不认识的批：$what" >&2; exit 2 ;;
esac
