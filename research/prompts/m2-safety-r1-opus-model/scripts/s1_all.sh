#!/usr/bin/env bash
# S1 各候选整轮：小盘 240..=384（准入关掉，重现 C542 那一格）持久 / 瞬时两种读故障；4 GiB 产品路径上写 / 屏障失败；
# 产品路径小盘 240..=384 控制组（看候选多拒了几段）。线程上限 8（每个进程单线程，xargs 至多 8 个）。
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=$1; L=$2; mkdir -p "$L"
declare -A V=(
  [today]=""
  [jia]="S_OPUS_S1_DRY_READS=1"
  [yi]="S_OPUS_S1_PROTECT=1"
  [jiayi]="S_OPUS_S1_DRY_READS=1 S_OPUS_S1_PROTECT=1"
  [ding]="S_OPUS_S1_EQUIV=1"
  [wu]="S_OPUS_S1_DRY_PESSIMISTIC=1"
  [yiwu]="S_OPUS_S1_PROTECT=1 S_OPUS_S1_DRY_PESSIMISTIC=1"
)
for name in today jia yi jiayi ding wu yiwu; do
  env_args=(${V[$name]})
  bash "$HERE/s1_scan.sh" "$BIN" "$L/s1-$name-skip-persistent.tsv" S_OPUS_ADMISSION=skip S_OPUS_MODE=persistent "${env_args[@]}"
  bash "$HERE/s1_scan.sh" "$BIN" "$L/s1-$name-skip-transient.tsv" S_OPUS_ADMISSION=skip S_OPUS_MODE=transient "${env_args[@]}"
  bash "$HERE/s1_scan.sh" "$BIN" "$L/s1-$name-judged-persistent.tsv" S_OPUS_MODE=persistent "${env_args[@]}"
  for fk in write barrier; do for idx in 0 1 2 3; do
    env "${env_args[@]}" S_OPUS_WIDTH=4g S_OPUS_N=30 S_OPUS_K_TO=40 S_OPUS_FAULT=$fk S_OPUS_TARGET_INDEX=$idx nice -n 19 "$BIN" --exact s1_write_fault_sweep --nocapture 2>/dev/null | grep '^S1W' > "$L/s1w-$name-4g-$fk-t$idx.tsv" &
  done; done
  wait
  for fk in write barrier; do
    env "${env_args[@]}" S_OPUS_ADMISSION=skip S_OPUS_UNIT_AREA_SLOTS=312 S_OPUS_N=24 S_OPUS_K_TO=40 S_OPUS_FAULT=$fk nice -n 19 "$BIN" --exact s1_write_fault_sweep --nocapture 2>/dev/null | grep '^S1W' > "$L/s1w-$name-312skip-$fk.tsv" &
  done
  wait
  echo "done $name"
done
