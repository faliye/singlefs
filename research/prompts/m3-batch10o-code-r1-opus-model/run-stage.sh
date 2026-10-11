#!/bin/bash
# 用法：bash run-stage.sh <草稿目录> <阶段>；阶段名照 m3-batch10i 攻方 rerun.sh，另加 k2push-*（T2 回退不拒臂）
set -uo pipefail
W=$1; s=$2; REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方原型，与别的会话并行" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令起点不同"
S1="ab-384-1-0,ab-384-3-0,ab-384-3-5"
case $s in
  scan1-prod-u8-s*) envs="OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=${s##*-s} OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40"; sps=$S1;;
  live-prod-ab384) envs="OPUS_STEPS=40"; sps=$S1;;
  ab384-owu) envs="OPUS_OPS=ow-unmount OPUS_STEPS=30"; sps=$S1;;
  live-prod-*) envs="OPUS_STEPS=40"; sps=${s#live-prod-};;
  base-prod) envs="OPUS_BASELINE=1 OPUS_STEPS=40"; sps="ab-384-1-0,ab-384-3-0,ab-384-3-5,mat-384-30-8,narrow-264-6-4,narrow-384-20-2";;
  k2-refuse-s*) envs="OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=${S1CLASS:-V} OPUS_OPS=${OPS:-overwrite} OPUS_STEPS=${s##*-s} OPUS_ROLLBACK_PICK=${PICK:-0}"; sps="mat-384-30-8,ab-384-3-5";;
  k2-push-s*) envs="OPUS_T2_ARM=push OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=${S1CLASS:-V} OPUS_OPS=${OPS:-overwrite} OPUS_STEPS=${s##*-s} OPUS_ROLLBACK_PICK=${PICK:-0}"; sps="mat-384-30-8,ab-384-3-5";;
  *) echo "没有阶段 $s" >&2; exit 2;;
esac
sps=${SPS:-$sps}
tag=$s${PICK:+-p$PICK}${S1CLASS:+-c$S1CLASS}${OPS:+-o$OPS}${SPS:+-$SPS}
env $envs OPUS_WORKERS=14 nice -n 19 bash $REPO/scripts/hosts/run-with-memory-cap.sh 3G bash $REPO/scripts/hosts/capped.sh 15 $W/tree/target/release/opus-o1-attack s1grid $sps > $W/out/$tag.log 2> $W/out/$tag.err
echo "rc=$? $tag"
