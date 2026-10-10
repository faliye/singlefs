#!/usr/bin/env bash
# 用法：W=<工作目录> [ARM=臂] piece.sh <日志名> <线程数> <内存上限> <环境变量 K=V …>：经内存包装、经 capped.sh 跑一件原型（臂缺省 jia），日志写 W/out/<日志名>.log。
set -u
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
: "${W:?给 W}"
export MODEL=${B17_MODEL:-$REPO/research/prompts/m3-batch17-code-r1-opus-model} ATTACK_DIR=$W
log=$1; th=$2; cap=$3; shift 3
mkdir -p "$W/out"
cd "$W" && env "$@" ATTACK_THREADS="$th" RUN_CAP="$cap" bash "$MODEL/scripts/run.sh" "${ARM:-jia}" "out/$log.log"
