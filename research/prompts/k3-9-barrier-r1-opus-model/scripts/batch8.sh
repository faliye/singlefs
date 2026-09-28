#!/usr/bin/env bash
# 批 8：攻方自提的 checker 改法（K39_CHECKER_VERIFIES_NAMED：下一次挂载会先施加的那一版要过点名验证）在打中的那几格上还中不中。5 件顺序跑。
cd /tmp/claude-1000/k3-9-barrier-r1-opus
rm -f runs/b8-*.rc
export K39_CHECKER_VERIFIES_NAMED=1
run() { local i=$1; shift; bash run-arm.sh "$@"; echo "$?" > runs/b8-$i.rc; echo "$(TZ=Asia/Tokyo date +%F) b8 piece $i done: $*" >> progress.md; }
run 1 mutated s8-floorreuse4-fix stage1 floorreuse4 384 16 32
run 2 control s8-floorreuse4-fix stage1 floorreuse4 384 16 32
run 3 mutated s8-first-fix stage2 first 384 16 32 7 publish
K39_MOUNT_READ_FAULT=1 run 4 control s8-first-readfault-fix stage2 first 384 16 32 7 publish
run 5 mutated s8-over-fix stage2 over 384 16 32 7 publish
n=$(ls runs/b8-*.rc | wc -l); echo "rc files $n of 5"; for k in 1 2 3 4 5; do echo "piece $k rc $(cat runs/b8-$k.rc)"; done
