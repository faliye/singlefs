#!/bin/bash
# 用法：run.sh <草稿目录> <输出名> <二进制> <参数…>：经内存包装 2G 与 capped.sh 1 跑，输出进 <草稿目录>/out/<输出名>.out，末行 rc=。
scratch=$1; shift; out=$1; shift; bin=$1; shift
mkdir -p "$scratch/out"
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-root-ring-code-r2 攻方腿原型，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一条命令换参数" nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh ${PROTO_MEM:-2G} bash /home/fy5090/code/singlefs/scripts/capped.sh ${PROTO_THREADS:-1} "$scratch/target/release/$bin" "$@" > "$scratch/out/$out.out" 2> "$scratch/out/$out.err"
echo "rc=$?" >> "$scratch/out/$out.out"
