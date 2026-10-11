#!/usr/bin/env bash
# m3-conc-r1 攻方模型复跑：五个确定的模型重跑一遍，与入库的 .out 逐字节比。
# fsync_probe.py 是本机取样、不复跑相等，不在这里跑；c58_alarm_model.py 读入库的 fsync_probe.out。
# c58_selfproof_sweep.py 是接手段加的：把 c58_alarm_model.py 第三部分写死的负载条目数放开扫。
# 用法（仓根）：bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-conc-r1-opus-model/rerun.sh <草稿目录>
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="${1:?草稿目录}"
mkdir -p "$out"
python3 "$here/c58_alarm_model.py" "$here/fsync_probe.out" > "$out/c58.out"
python3 "$here/ring_window_inflight.py" > "$out/ring_window_inflight.out"
python3 "$here/fsync_return_model.py" > "$out/fsync_return_model.out"
python3 "$here/k2_capacity.py" > "$out/k2_capacity.out"
python3 "$here/c58_selfproof_sweep.py" > "$out/c58_selfproof_sweep.out"
status=0
for f in c58 ring_window_inflight fsync_return_model k2_capacity c58_selfproof_sweep; do
  if cmp -s "$here/$f.out" "$out/$f.out"; then echo "same $f.out"; else echo "DIFF $f.out"; status=1; fi
done
exit "$status"
