#!/usr/bin/env bash
# 攻方第三轮 harness 补跑：H04 只跑新用例；第一张表第 11、12 行（K20 两种写法）换 2400M 上限重跑；三行只改读盘的变异跑整个 harness 包。
# 用法：harness-batch-2.sh <仓副本根> <冻结副本根> <出口目录>
set -u
tree=$1; frozen=$2; out=$3
here=$(cd "$(dirname "$0")" && pwd)
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的攻方腿与辩方腿各跑各的，这一条是攻方 harness 草稿" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1800 CARGO_PROFILE_FLAG=--release
HARNESS_MEMORY_CAP=3500M bash "$here/harness-rows.sh" "$tree" "$frozen" "$here/harness-attack-rows-2.tsv" "$out/rows-2" 2
HARNESS_MEMORY_CAP=2400M bash "$here/harness-rows.sh" "$tree" "$frozen" "$here/harness-attack-rows.tsv" "$out/rows-k20" 11 12
RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="整个 harness 包编译加跑，比单个测试目标的峰值大" HARNESS_MEMORY_CAP=6G bash "$here/harness-rows.sh" "$tree" "$frozen" "$here/harness-attack-rows-3.tsv" "$out/rows-all" 5 2 3 4
