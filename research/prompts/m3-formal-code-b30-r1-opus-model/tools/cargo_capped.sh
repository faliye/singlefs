#!/usr/bin/env bash
# 用法：cargo_capped.sh <树> <cargo 参数…>：在树里经内存包装（8G）与线程上限（默认 7，CAP 可改）跑 cargo。
# 包装与线程脚本取主仓的（REPO，默认 /home/fy5090/code/singlefs）。
set -u
tree=$1; shift
repo=${REPO:-/home/fy5090/code/singlefs}
cd "$tree" || exit 2
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-b30 攻方腿模型在自己的副本与编译目录里跑 harness 档}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示定 cargo 一律给 8G}"
exec nice -n 19 bash "$repo/scripts/run-with-memory-cap.sh" 8G bash "$repo/scripts/capped.sh" "${CAP:-7}" cargo "$@"
