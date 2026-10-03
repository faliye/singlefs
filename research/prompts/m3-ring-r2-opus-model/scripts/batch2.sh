#!/bin/bash
cd /tmp/claude-1000/m3-ring-r2-opus
# 先跑放槽量直接量（12 次），再重跑 G2 第一批扫描（612 次）。
for s in 4 8; do k=$((3*s+4)); for h in k1 fillable; do
  SINGLEFS_WINDOW_ANCHOR=off ./run.sh d-$s-$h-off window_release_direct $s $h $k
  SINGLEFS_WINDOW_ANCHOR=ring ./run.sh d-$s-$h-ring-cw0 window_release_direct $s $h $k
  SINGLEFS_WINDOW_ANCHOR=ring SINGLEFS_CHECKER_WINDOW=1 SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) ./run.sh d-$s-$h-ring-cw1 window_release_direct $s $h $k
done; done
echo "direct done $(date +%F)" >> out/progress.md
: > out/sweep1.txt
for s in 4 8 16; do
  lo=$((3*s+1)); hi=$((6*s+6))
  for h in $(seq $lo $hi); do
    for hist in k1 k1x2 fillable; do
      for a in ring first_txg; do
        SINGLEFS_WINDOW_ANCHOR=$a ./run.sh sw1 window_static_check $s $hist $h 0 0
        grep -h '^STATIC\|^rc=' out/sw1.out | tr '\n' ' ' >> out/sweep1.txt; echo >> out/sweep1.txt
      done
    done
  done
  echo "sweep1 S=$s done" >> out/progress.md
done
