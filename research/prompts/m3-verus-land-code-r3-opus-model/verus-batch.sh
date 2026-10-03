#!/usr/bin/env bash
# 攻方第三轮的 Verus 量：攻方 7 行在 rlimit 10 上；verus-mutations.tsv 全表在 rlimit 10、20、40 上（期望片段按门禁的判法判）。
# 用法：verus-batch.sh <核 crate 的干净副本（目录名 singlefs-replay-prefix）> <出口目录>
set -u
clean=$1; out=$2
here=$(cd "$(dirname "$0")" && pwd)
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的攻方腿与辩方腿各跑各的，这一条是攻方 Verus 草稿" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1800 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="改坏的核与换了 rlimit 的副本比峰值表那一次未改的核可能大，按一G给"
bash "$here/verus-rows.sh" "$clean" "$here/verus-attack-rows.tsv" 10 "$out/attack-r10" 2 3 4 5 6 7 8
table=$clean/verus-mutations.tsv
mapfile -t rows < <(awk '!/^#/ && NF {print NR}' "$table")
for rlimit in 10 20 40; do
    bash "$here/verus-rows.sh" "$clean" "$table" "$rlimit" "$out/table-r$rlimit" "${rows[@]}"
done
