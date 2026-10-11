#!/bin/bash
# 任意一阶段（rerun.sh setup 之后）：bash rerun-stage.sh <草稿目录> <阶段名> "<环境变量串>" <起点,起点…>；日志写 <草稿目录>/out/<阶段名>.log
# 本轮用过：scan1-today-u8-s6（OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=6 OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40，起点 ab-384-1-0,ab-384-3-0,ab-384-3-5，s9、s12、s15 同形）、
#   live-today-ab384（OPUS_STEPS=40，同三个起点）、live-today-mat（OPUS_STEPS=40，起点 mat-384-30-8）、k2-today-3（OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=V OPUS_STEPS=3，起点 mat-384-30-8）
set -u
REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑正推腿原型，与别的会话并行"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令与线程数不同，峰值表里那一次是另一种"
W=${1:?草稿目录}; name=${2:?阶段名}; envs=${3:?环境变量串}; sp=${4:?起点}
env $envs OPUS_WORKERS=14 nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 3G bash "$REPO/scripts/capped.sh" 15 \
  "$W/tree/target/release/t2-arm-probe" s1grid "$sp" > "$W/out/$name.log" 2>&1
echo "rc=$? $name"
