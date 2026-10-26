#!/usr/bin/env bash
# 复跑第 37 批代码轮第二轮攻方腿（Opus）的模型：纯 Python，不编 Rust、不起 cargo、不跑任何 checker 档测试。
# 用法：bash tools/rerun.sh [冻结副本根] [仓根]
#   冻结副本根默认 ~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree；仓根默认这个目录往上三层（读 scripts/experiment-apparatus/replay.sh 与 research/results/ 里的 e165 产物）。
# 输出与同目录 expected-output.txt 逐字比，不同退 1；源码里模型依赖的那几行对不上退 3（模型作废）；模型算的今天每条正式跑的对照集与产物不同退 4。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="${1:-$HOME/.local/share/singlefs/work/m3-formal-code-b37-r2/tree}"
repo="${2:-$(cd "$here/../../.." && pwd)}"
out="$(mktemp)"
trap 'rm -f "$out"' EXIT
nice -n 19 python3 -I "$here/c13_family_axis.py" "$root" "$repo" > "$out"
if cmp -s "$out" "$here/expected-output.txt"; then
  echo "✓ 模型输出与 expected-output.txt 逐字相同（$(wc -l < "$out") 行）"
else
  diff "$here/expected-output.txt" "$out" | head -40
  echo "✗ 模型输出与 expected-output.txt 不同"
  exit 1
fi
