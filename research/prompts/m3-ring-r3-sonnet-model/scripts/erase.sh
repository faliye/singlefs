#!/bin/bash
cd /tmp/claude-1000/m3-ring-r3-sonnet
for s in 4 8; do k=$((3*s+4)); for h in k1 fillable; do
  ERASE=1 SINGLEFS_WINDOW_ANCHOR=off ./run.sh e-$s-$h-erase erase_release_direct $s $h $k
done; done
