#!/bin/bash
cd /tmp/claude-1000/m3-ring-r2-opus
# G2 第二批：被判那次挂载之后接用户动作（覆盖写 3S+4 次，再可选回退到窗口里最旧的候选根 + 2 次覆盖写），全域静态判法。
: > out/sweep2.txt
for s in 4 8; do
  lo=$((3*s+1)); hi=$((3*s+12)); k=$((3*s+4))
  for h in $(seq $lo $hi); do
    for hist in k1 k1x2 fillable; do
      for a in ring first_txg; do
        for rb in 0 1; do
          SINGLEFS_WINDOW_ANCHOR=$a ./run.sh sw2 window_static_check $s $hist $h $k $rb
          grep -h '^STATIC\|^rc=' out/sw2.out | tr '\n' ' ' >> out/sweep2.txt; echo >> out/sweep2.txt
        done
      done
    done
  done
  echo "sweep2 S=$s done" >> out/progress.md
done
