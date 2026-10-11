#!/bin/bash
# 用法：run.sh <输出名> <二进制> <参数…>；环境变量照传。经内存包装（PROTO_MEM，默认 1G）与 capped.sh（PROTO_THREADS，默认 1）。
here=$(cd "$(dirname "$0")" && pwd)
out=$1; shift; bin=$1; shift
mkdir -p "$here/out"
cd "$here"
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-root-ring-code-r1 攻方腿原型，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一条命令换参数，峰值不同" nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh ${PROTO_MEM:-1G} bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh ${PROTO_THREADS:-1} ./target/release/$bin "$@" > out/$out.out 2> out/$out.err
echo "rc=$?" >> out/$out.out
