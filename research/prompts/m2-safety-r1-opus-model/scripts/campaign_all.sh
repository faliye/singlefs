#!/usr/bin/env bash
# 各候选的新失败面：仓里自己的随机历史（每步池级 checker + 理想模型对拍），512 段 × 40 步，
# 缩减版（第一次按 512 段 × 12 种组合估时太长，停下重来）：256 段 × 40 步，盘宽 384 / 4g × 比重 rollback / broad，准入照式子判；
# S1 的候选另加准入关掉的两组。每条命令 8 个线程。
set -euo pipefail
BIN=$1; L=$2; mkdir -p "$L"
declare -A V=(
  [today]="S_OPUS_NONE=1"
  [yi]="S_OPUS_S1_PROTECT=1"
  [jiayi]="S_OPUS_S1_DRY_READS=1 S_OPUS_S1_PROTECT=1"
  [wu]="S_OPUS_S1_DRY_PESSIMISTIC=1"
  [ding]="S_OPUS_S1_EQUIV=1"
  [narrow]="S_OPUS_S2=narrow"
  [clearflag]="S_OPUS_S2=clearflag"
  [infer]="S_OPUS_S3=infer"
  [rowzero]="S_OPUS_S3=row-zero"
  [narrow-rowzero]="S_OPUS_S2=narrow S_OPUS_S3=row-zero"
  [refuse]="S_OPUS_S3=refuse"
  [pre]="S_OPUS_S3=pre"
)
for name in today yi jiayi wu ding narrow clearflag infer rowzero narrow-rowzero refuse pre; do
  adms=judged; case $name in today|yi|jiayi|wu|ding) adms="judged skip";; esac
  for width in 384 4g; do for weights in rollback broad; do for adm in $adms; do
    extra=(); [ "$adm" = skip ] && extra=(S_OPUS_ADMISSION=skip)
    env ${V[$name]} "${extra[@]}" S_OPUS_WEIGHTS=$weights S_OPUS_WIDTH=$width S_OPUS_SEEDS=256 S_OPUS_OPS=40 S_OPUS_THREADS=8 S_OPUS_FIRST_SEED=5100000000 \
      nice -n 19 "$BIN" --exact s1_campaign --nocapture 2>&1 | grep -A2000 '^S1CAMPAIGN' > "$L/campaign-$name-$width-$weights-$adm.log" || true
  done; done; done
  echo "done $name"
done
