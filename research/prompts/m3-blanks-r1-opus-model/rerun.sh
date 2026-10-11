#!/usr/bin/env bash
# 复跑：bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 15 bash <本目录>/rerun.sh <输出目录>
# 四个纯 Python 模型，不读盘、不编译；输出拼成 <输出目录>/model-output.txt，与本目录那份逐字比。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="${1:?输出目录}"
mkdir -p "$out"
{
  nice -n 19 python3 "$here/queue_bound.py" 5 9 4
  nice -n 19 python3 "$here/queue_bound.py" 4 7 12
  nice -n 19 python3 "$here/tomb_reserve.py"
  nice -n 19 python3 "$here/kind_alias.py"
  nice -n 19 python3 "$here/ring_window.py"
} > "$out/model-output.txt"
if [ -f "$here/model-output.txt" ]; then
  if cmp -s "$out/model-output.txt" "$here/model-output.txt"; then echo "model-output 相同"; else echo "model-output 不同"; exit 1; fi
fi
sha256sum "$out/model-output.txt"
