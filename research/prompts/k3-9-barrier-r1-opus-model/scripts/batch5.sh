#!/usr/bin/env bash
# 批 5：修好版本表（带读故障时按挂载真正接着走的那一版排）之后重跑批 4 第 4、5 件，2 件顺序跑。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b5-*.rc
run() { local i=$1; shift; bash run-arm.sh "$@"; echo "$?" > runs/b5-$i.rc; }
K39_MOUNT_READ_FAULT=1 run 1 control s5-first-readfault stage2 first 384 16 32 7 publish
K39_MOUNT_READ_FAULT=1 run 2 control s5-over-readfault stage2 over 384 16 32 7 publish
n=$(ls runs/b5-*.rc | wc -l); echo "rc files $n of 2"; for k in 1 2; do echo "piece $k rc $(cat runs/b5-$k.rc)"; done
