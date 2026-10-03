#!/usr/bin/env bash
# 用法：run-j2.sh <工作树> <冻结拷贝根> <变异表> <出口目录> <测试参数…>；逐行套、跑、拷回原文件（lib.rs）
set -u
tree=$1; frozen=$2; table=$3; out=$4; shift 4
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示给定内存上限十二G，harness 用例按提示上限之内的六G跑"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的几个腿各跑各的"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1500
rel=crates/singlefs-replay-prefix/src/lib.rs
total=$(wc -l < "$table")
for row in $(seq 2 "$total"); do
    name=$(awk -F'\t' -v r="$row" 'NR==r{print $1}' "$table")
    cp "$frozen/$rel" "$tree/$rel"
    if ! python3 "$here/apply-row.py" "$table" "$row" "$tree" > "$out/apply-$row.txt"; then
        printf '%s\tANCHOR\t%s\n' "$row" "$name" >> "$out/summary.tsv"; continue
    fi
    ( cd "$tree" && nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 10 cargo test --release --offline "$@" ) > "$out/test-$row.log" 2>&1
    status=$?
    failed=$(grep -E '^test .* \.\.\. FAILED$' "$out/test-$row.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | tr '\n' ' ')
    passed=$(grep -cE '^test .* \.\.\. ok$' "$out/test-$row.log")
    if grep -qE '^error(\[E[0-9]+\])?: (could not compile|aborting)|^error\[E[0-9]+\]' "$out/test-$row.log"; then v=BUILD_ERROR; elif [[ -n "$failed" ]]; then v=RED; else v=GREEN; fi
    printf '%s\t%s\trc=%s\tpassed=%s\tfailed=%s\t%s\n' "$row" "$v" "$status" "$passed" "$failed" "$name" >> "$out/summary.tsv"
done
cp "$frozen/$rel" "$tree/$rel"
