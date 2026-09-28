#!/usr/bin/env bash
# 批 3：第二、三截（崩溃镜像上可写挂载，挂载那一段层 0 全量展开），6 件顺序跑（每件内部 10 线程），每件预算 10^6 个状态。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b3-*.rc
run() { local i=$1; shift; bash run-arm.sh "$@"; echo "$?" > runs/b3-$i.rc; echo "$(TZ=Asia/Tokyo date +%F) b3 piece $i done: $* $(tail -1 runs/$2-$1.log 2>/dev/null)" >> progress.md; }
run 1 mutated s2-over stage2 over 384 16 32 7 publish
run 2 mutated s2-seq2 stage2 seq2 384 16 16 7 publish
run 3 mutated s2-seq3 stage2 seq3 384 16 8 7 none
run 4 mutated s2-over_i2 stage2 over_i2 384 16 16 4 remount
run 5 control s2-over stage2 over 384 16 32 7 publish
run 6 mutated s2-first stage2 first 384 16 32 5 publish
wait
n=$(ls runs/b3-*.rc | wc -l)
echo "rc files $n of 6"
for k in 1 2 3 4 5 6; do echo "piece $k rc $(cat runs/b3-$k.rc)"; done
