#!/usr/bin/env bash
# 批 4：I-3.10 那一格的对照与读故障（带屏障臂 + 挂载读不出一份单元），6 件顺序跑，每件预算 10^6。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b4-*.rc
run() { local i=$1; shift; bash run-arm.sh "$@"; echo "$?" > runs/b4-$i.rc; echo "$(TZ=Asia/Tokyo date +%F) b4 piece $i done: $* $(tail -1 runs/$2-$1.log 2>/dev/null)" >> progress.md; }
run 1 mutated s4-first stage2 first 384 16 32 7 publish
run 2 control s4-first stage2 first 384 16 32 7 publish
run 3 control s4-seq2 stage2 seq2 384 16 16 7 publish
K39_MOUNT_READ_FAULT=1 run 4 control s4-first-readfault stage2 first 384 16 32 7 publish
K39_MOUNT_READ_FAULT=1 run 5 control s4-over-readfault stage2 over 384 16 32 7 publish
run 6 mutated s4-over stage2 over 384 16 32 7 publish
n=$(ls runs/b4-*.rc | wc -l)
echo "rc files $n of 6"
for k in 1 2 3 4 5 6; do echo "piece $k rc $(cat runs/b4-$k.rc)"; done
