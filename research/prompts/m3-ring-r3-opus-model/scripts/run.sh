#!/bin/bash
# 用法：run.sh <输出名> <二进制> <参数…>；环境变量照传。
out=$1; shift; bin=$1; shift
cd /tmp/claude-1000/m3-ring-r3-opus
mkdir -p out
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r3 攻方腿原型，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一条命令换环境变量选不同的臂，峰值不同" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh ${PROTO_MEM:-1G} bash /home/fy5090/code/singlefs/research/scripts/capped.sh ${PROTO_THREADS:-1} ./target/release/$bin "$@" > out/$out.out 2> out/$out.err
echo "rc=$?" >> out/$out.out
