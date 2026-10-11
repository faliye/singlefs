#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r3）：拿第一轮攻方的内核量法（research/prompts/m3-run-compare-split-r1-opus-model/，原样拷到草稿目录里跑）
# 在 E5、E7、E9 三份底子上各量一遍三个现成内核与两个 P14 探针入口（E9 把 MAX_RECORDS 抬到 2048、改了 common.wgsl）。
# 用法：run_kernel_measure_e7_e9.sh <仓根> <草稿目录>（整条经 run-with-memory-cap.sh 起）；输出落 <草稿目录>/ks-<底子>/kernel-measure.out
set -euo pipefail
repo="$1"; work="$2"
rm -rf "$work/r1-opus-model"; cp -a "$repo/research/prompts/m3-run-compare-split-r1-opus-model" "$work/r1-opus-model"
for base in e5 e7 e9; do
  mkdir -p "$work/ks-$base"
  bash "$work/r1-opus-model/kernel_measure.sh" "$HOME/.local/share/singlefs/work/m3-merge-impl/$base/repo" "$work/ks-$base" > "$work/ks-$base/kernel-measure.out"
done
