#!/usr/bin/env bash
# 纯 Python，读冻结副本里的装置源码与 research/results/ 下 2026-10-04 的产物；不编 Rust、不跑装置。
# 用法：bash research/prompts/m3-formal-code-b37-r3-sonnet-model/tools/rerun.sh [冻结副本根] [结果目录]
# 有同目录上一层的 rerun-output.txt 就逐字比，末行说相同与否。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tree="${1:-$HOME/.local/share/singlefs/work/m3-formal-code-b37-r3/tree}"
results="${2:-$(cd "$here/../../../.." && pwd)/research/results}"
src="$tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
out="$(mktemp)"
{
  echo "## check_sets：测试里写死的 14 份集合对点名的产物"
  python3 -I "$here/check_sets.py" "$src" "$results"
  echo "## model：源码谓词模型"
  RESULTS="$results" python3 -I "$here/model.py" "$src"
} > "$out"
cat "$out"
if [[ -f "$here/../rerun-output.txt" ]]; then
  if cmp -s "$out" "$here/../rerun-output.txt"; then echo "与 rerun-output.txt 逐字相同"; else echo "与 rerun-output.txt 不同"; diff "$out" "$here/../rerun-output.txt" | head -20; fi
fi
rm -f "$out"
