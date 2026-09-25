#!/usr/bin/env bash
# 用法：s1_scan.sh <测试二进制> <输出文件> [环境变量赋值……]
# 单元区 240..=384 槽每 4 槽一档（37 档）× 覆盖写 S_OPUS_N_FROM..=S_OPUS_N_TO（默认 18..=35），每档一个进程，至多 8 个并行。
set -euo pipefail
BIN=$1; OUT=$2; shift 2
for kv in "$@"; do export "$kv"; done
seq 240 4 384 | xargs -P 8 -I{} env S_OPUS_UNIT_AREA_SLOTS={} nice -n 19 "$BIN" --exact s1_scan_one_width --nocapture 2>/dev/null | grep '^S1ROW' | sort -t$'\t' -k2,2n -k3,3n -k5,5n -k6,6 > "$OUT"
wc -l < "$OUT"
