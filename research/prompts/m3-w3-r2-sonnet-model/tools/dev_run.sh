#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<仓根> dev_run.sh <装置二进制目录> <臂> <族> <结果文件> <序号逗号表> [逐步打印序号]
set -uo pipefail
BIN=${1:?}; ARM=${2:?}; FAM=${3:?}; OUT=${4:?}; IDX=${5:?}; TRACE=${6:-}
R=${SINGLEFS_REPO:?}
env E175_ARM=$ARM E175_WORKER_THREADS=6 E175_TIME_BUDGET_SECONDS=100000 E175_FAMILY=$FAM E175_RESULT_FILE=$OUT E175_INDICES=$IDX W3_TRACE_INDICES=$TRACE \
  nice -n 19 bash $R/scripts/hosts/capped.sh 6 bash $R/scripts/hosts/run-with-memory-cap.sh 4G $BIN/w3-$ARM --ignored --exact run_one_family_on_one_arm_into_the_result_file > /dev/null
