#!/bin/bash
cd /tmp/claude-1000/m3-ring-r3-sonnet
for s in 4 8; do k=$((3*s+4)); for h in k1 fillable; do
  SINGLEFS_WINDOW_ANCHOR=off ./run.sh d-$s-$h-off window_release_direct $s $h $k
  SINGLEFS_WINDOW_ANCHOR=ring ./run.sh d-$s-$h-ring-cw0 window_release_direct $s $h $k
  SINGLEFS_WINDOW_ANCHOR=ring SINGLEFS_CHECKER_WINDOW=1 SINGLEFS_CHECKER_WINDOW_RING_LEN=$((3*s)) ./run.sh d-$s-$h-ring-cw1 window_release_direct $s $h $k
done; done
