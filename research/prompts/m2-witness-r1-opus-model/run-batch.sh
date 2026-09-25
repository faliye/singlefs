#!/usr/bin/env bash
# m2-witness-r1 云端攻方腿：一批臂，每臂一个进程（单线程），至多 4 个同时跑（线程上限 4）。
# 用法：run-batch.sh <批名> <行文件>；行文件每行「放处 写序 物理块 模式 [参数…]」。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
binary=$here/repo/target/release/e158_root_choice_repair
batch=$1
lines=$2
mkdir -p "$here/out/$batch"
run_one() {
    local placement=$1 order=$2 pbs=$3 mode=$4; shift 4
    local name="$mode-$placement-$order-$pbs${*:+-$(echo "$*" | tr ' ' '_')}"
    local start end status=0
    start=$(date +%s)
    SINGLEFS_WITNESS_PLACEMENT=$placement SINGLEFS_WITNESS_ORDER=$order E158_PBS=$pbs nice -n 19 "$binary" "$mode" "$@" > "$here/out/$batch/$name.out" 2> "$here/out/$batch/$name.err" || status=$?
    end=$(date +%s)
    echo "$name exit=$status seconds=$((end - start))" >> "$here/out/$batch/timings.txt"
}
export -f run_one
export here binary batch
grep -v '^#' "$lines" | grep -v '^$' | xargs -P 4 -L 1 bash -c 'run_one "$@"' _
echo "batch $batch finished" >> "$here/out/$batch/timings.txt"
