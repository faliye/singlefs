#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 results-cited 格的样本：只留登记表的骨架，不是真的 replay.sh。
set -uo pipefail
TABLE=$(cat <<'TSV'
E95|e95-sample||e95-sample.r2.out|exact
TSV
)
echo "$TABLE"
