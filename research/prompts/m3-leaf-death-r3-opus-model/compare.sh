#!/usr/bin/env bash
# 比 rerun.sh 的 <输出目录>/summary 与本目录 expected/ 逐份：一样打 same，不一样打 differ 并退 1。
set -uo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：compare.sh <输出目录>}
rc=0
for f in agg.txt coverage.txt clone.txt determinism.txt copy-k2.txt copy-k3.txt existing-r1.txt existing-r2.txt sorted-output-sha256.txt; do
    if cmp -s "$MODEL_DIR/expected/$f" "$OUT/summary/$f"; then echo "same $f"; else echo "differ $f"; rc=1; fi
done
exit $rc
