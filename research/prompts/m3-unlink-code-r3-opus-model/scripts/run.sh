#!/usr/bin/env bash
# 用法：run.sh <二进制> <日志>   经内存包装跑原型（环境变量照传）。
set -u
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型换参数与开层0枚举后内存会涨，峰值表里那次只跑前缀刀"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与正推腿同会话各跑各的原型"
export SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS:-8}
nice -n 19 bash ${SINGLEFS_REPO:-/home/fy5090/code/singlefs}/research/scripts/run-with-memory-cap.sh 2G "$1" > "$2" 2>&1
