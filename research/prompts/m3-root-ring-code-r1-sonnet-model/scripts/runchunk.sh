#!/usr/bin/env bash
# runchunk.sh <起> <止> <日志>  按 alltests.txt 的行区间跑一批 --test
set -u
cd /tmp/claude-1000/m3-root-ring-code-r1-sonnet/work
args=()
for t in $(sed -n "$1,$2p" /tmp/claude-1000/m3-root-ring-code-r1-sonnet/alltests.txt); do args+=(--test "$t"); done
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="主 agent 同时派了正推与攻方两条腿，各跑各的探针" nice -n 19 timeout 225 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 15 cargo test --offline --release -p singlefs-harness --no-fail-fast "${args[@]}" > "$3" 2>&1
echo "exit=$?"; grep -E '^test .* FAILED|test result: FAILED|Running tests' "$3" | grep -E 'FAILED' | head -20; grep -c 'test result: ok' "$3"
