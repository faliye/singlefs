#!/bin/bash
# 取样复跑第二轮攻方第一批全域静态判法：S=4，洞 txg 取 $1..$2，三种历史、两种锚；每行格式与原 sweep1.txt 相同
cd /tmp/claude-1000/m3-ring-r3-sonnet
lo=$1; hi=$2; outfile=$3
for h in $(seq $lo $hi); do for hist in k1 k1x2 fillable; do for a in ring first_txg; do
  SINGLEFS_WINDOW_ANCHOR=$a ./run.sh sw1 window_static_check 4 $hist $h 0 0
  grep -h '^STATIC\|^rc=' out/sw1.out | tr '\n' ' ' >> $outfile; echo >> $outfile
done; done; done
