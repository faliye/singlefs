#!/usr/bin/env bash
# X2 吸收那一格：在 random_histories.rs 里 SINGLEFS_RANDOM_HISTORY_SEEDS 那一行之后插一行，照仓里现成的 count_from_environment 写法读一个 capped.sh 不设的新线程变量（没设取 8）。
# 用法：x2_absorb_insert.sh <random_histories.rs 的拷贝>
set -eu
f=$1
n=$(grep -n 'count_from_environment("SINGLEFS_RANDOM_HISTORY_SEEDS", 1000);' "$f" | cut -d: -f1)
sed -i "${n}a\\    let sampling_workers = count_from_environment(\"SINGLEFS_RANDOM_HISTORY_SAMPLING_WORKERS\", 8);" "$f"
