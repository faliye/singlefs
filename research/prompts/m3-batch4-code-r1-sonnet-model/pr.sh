#!/usr/bin/env bash
# usage: pr.sh <log> <row numbers...>
log="$1"; shift
names=()
for r in "$@"; do
  names+=("$(sed -n "${r}p" /tmp/claude-1000/m3-batch5-freeze/tree/crates/mutations.tsv | cut -f1)")
done
cd /tmp/claude-1000/m3-batch4-code-r1-sonnet
PROVE_RED_LOG_DIRECTORY=/tmp/claude-1000/m3-batch4-code-r1-sonnet/prove-red-logs \
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出的几个 agent 各跑各的，不是同一条链重复起" \
RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="变异之后要重编 core/checker 与测试二进制，峰值表里的实测值是不重编的" \
timeout 235 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/prove-red.sh --copy /tmp/claude-1000/m3-batch4-code-r1-sonnet/tree --memory 8G singlefs-harness "${names[@]}" > "$log" 2>&1
echo "exit=$?" >> "$log"
