#!/usr/bin/env bash
# 复跑挂钟：89 秒（实测一次；空真格 15 处 requires 每处起一次桩包装，三种格各跑两遍，没有一步慢）。
# 只读探针：不起 Verus、不起真包装、不跑真阶段。在仓根跑：
#   RUN_WITH_MEMORY_CAP_PARALLEL_REASON="辩方腿各跑各的探针" bash scripts/run-with-memory-cap.sh 4G bash scripts/capped.sh 4 nice -n 19 bash research/prompts/defs-m3-formal-gate-r3-sonnet-model/rerun.sh
set -uo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
scratch="${SCRATCH:-/tmp/claude-1000/defs-m3-formal-gate-r3-sonnet}"
mkdir -p "$scratch"
model="$root/research/prompts/defs-m3-formal-gate-r3-sonnet-model"
echo "### l3-cell-modes：三种格（空真格，桩包装）"
for mode in notrun mixed red; do
  python3 "$model/l3-cell-modes.py" "$root" "$scratch" "$mode" --quiet | grep '^##'
done
echo "### l3-synthetic-stage：合成阶段 source 真的 stage-cells.sh，看汇总行"
for cell in vac-notrun vac-mixed vac-red; do
  echo "=== $cell"
  bash "$model/l3-synthetic-stage.sh" "$root" "$scratch" "$cell" | sed -n '/── 汇总/,$p'
done
echo "### l3-verus-exit-codes"
python3 "$model/l3-verus-exit-codes.py" "$root" "$scratch"
for code in 137 250 252 253; do
  echo "### l3-e165-exit-codes 假包装退 $code"
  python3 "$model/l3-e165-exit-codes.py" "$root" "$scratch" "$code" | cut -c1-200
done
