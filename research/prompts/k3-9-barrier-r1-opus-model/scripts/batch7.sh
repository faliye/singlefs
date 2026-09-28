#!/usr/bin/env bash
# 批 7：抬 F 之后复用回收槽的覆盖写（第一截手摆），K = 4、8，2 臂，4 件并行（每件单线程）。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b7-*.rc
i=0
for k in 4 8; do for arm in control mutated; do
  i=$((i+1)); { bash run-arm.sh $arm stage1-floorreuse$k stage1 floorreuse$k 384 16 32; echo "$?" > runs/b7-$i.rc; } &
done; done
wait
n=$(ls runs/b7-*.rc | wc -l); echo "rc files $n of $i"; for k in $(seq 1 $i); do echo "piece $k rc $(cat runs/b7-$k.rc)"; done
