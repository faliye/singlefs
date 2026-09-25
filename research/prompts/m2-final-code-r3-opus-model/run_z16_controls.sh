#!/usr/bin/env bash
# Z16 对照扫描：单元区宽 × 覆盖写次数，每格跑一次 z16_control（不注入），回退目标取环里最旧（index = 覆盖写次数 − 1）。8 路并行。
# 用法：run_z16_controls.sh <日志文件>；R3_OPUS_Z16_BIN 指二进制。
set -euo pipefail
OUT=${1:?}; BIN=${R3_OPUS_Z16_BIN:?}
: > "$OUT"
for slots in $(seq 240 4 384); do for n in $(seq 18 35); do echo "$slots $n"; done; done |
  xargs -P 8 -n 2 sh -c 'SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS=$0 R3_OPUS_OVERWRITES=$1 SINGLEFS_R3_OPUS_REPORT=1 nice -n 19 '"$BIN"' --exact z16_control --nocapture 2>&1 | grep -E "R3OPUS" | tr "\n" "|" ; echo' >> "$OUT"
wc -l < "$OUT"
