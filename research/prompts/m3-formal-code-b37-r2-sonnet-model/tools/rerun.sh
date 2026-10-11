#!/usr/bin/env bash
# 辩方腿 C1/C2 模型的复跑：纯 Python，不编 Rust、不跑装置，所以没有 cargo 要经内存包装。
# 用法：rerun.sh [冻结副本根] [仓根]；模型目录里有 rerun-output.txt 就逐字比，没有就直接打。
# 注意：模型读仓根里的 scripts/experiments/replay.sh 与 research/results/ 下 2026-10-04 的产物；这两处别的会话改了，输出会跟着变。
set -euo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
TREE=${1:-$HOME/.local/share/singlefs/work/m3-formal-code-b37-r2/tree}
REPO=${2:-/home/fy5090/code/singlefs}
out=$(mktemp "${TMPDIR:-/tmp}/m3b37r2-sonnet-rerun.XXXXXX")
trap 'rm -f "$out"' EXIT
nice -n 19 python3 -I "$HERE/tools/model.py" "$TREE" "$REPO" > "$out"
if [ -f "$HERE/rerun-output.txt" ]; then
  diff -u "$HERE/rerun-output.txt" "$out" && echo "与 rerun-output.txt 逐字相同"
else
  cat "$out"
fi
