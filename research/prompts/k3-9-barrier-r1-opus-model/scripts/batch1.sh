#!/usr/bin/env bash
# 批 1：第一截手摆，5 个场景 × 2 臂，10 件并行（每件单线程）。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b1-*.rc
i=0
for scenario in first over seq2 seq3 over_i2; do
  for arm in control mutated; do
    i=$((i+1))
    { bash run-arm.sh $arm stage1-$scenario stage1 $scenario 384 16 32; echo "$?" > runs/b1-$i.rc; } &
  done
done
wait
n=$(ls runs/b1-*.rc | wc -l)
echo "rc files $n of $i"
for k in $(seq 1 $i); do echo "piece $k rc $(cat runs/b1-$k.rc)"; done
