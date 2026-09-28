#!/usr/bin/env bash
# 批 6：复用回收槽的覆盖写（第一截手摆），K = 30、60，2 臂，4 件并行（每件单线程）。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b6-*.rc
i=0
for k in 30 60; do for arm in control mutated; do
  i=$((i+1)); { bash run-arm.sh $arm stage1-overmany$k stage1 overmany$k 384 16 32; echo "$?" > runs/b6-$i.rc; } &
done; done
wait
n=$(ls runs/b6-*.rc | wc -l); echo "rc files $n of $i"; for k in $(seq 1 $i); do echo "piece $k rc $(cat runs/b6-$k.rc)"; done
