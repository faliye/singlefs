#!/usr/bin/env bash
# 只读探针：不起 Verus、不编装置、不起真包装、不执行被判的命令。在仓根跑（派发的上限 4G、线程 4）：
#   RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的辩方腿各跑各的探针，与别的会话互不相干" bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 4 nice -n 19 bash research/prompts/defs-m3-formal-gate-r2-sonnet-model/rerun.sh
set -uo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
scratch="${SCRATCH:-/tmp/claude-1000/defs-m3-formal-gate-r2-sonnet}"
mkdir -p "$scratch"
model="$root/research/prompts/defs-m3-formal-gate-r2-sonnet-model"
echo "### e3-verus-exit-codes"
python3 "$model/e3-verus-exit-codes.py" "$root" "$scratch"
for code in 0 137 250 251 252 253 254; do
  echo "### e3-e165-exit-codes 假包装退 $code"
  python3 "$model/e3-e165-exit-codes.py" "$root" "$scratch" "$code"
done
echo "### e3-mixed-cell"
python3 "$model/e3-mixed-cell.py" "$root" "$scratch"
echo "### e4-step1-today"
bash "$model/e4-step1-today.sh" "$root"
