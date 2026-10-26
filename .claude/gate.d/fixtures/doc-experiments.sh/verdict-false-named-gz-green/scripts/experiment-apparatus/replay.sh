#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 verdict-false-named 格的样本：只留判读表格所需的最小骨架，不是真的 replay.sh。登记的产物只以 .out.gz 存在。
set -uo pipefail
TABLE=$(cat <<'TSV'
E9002|e9002-sample||e9002-sample-2026-10-21.out.gz|exact
TSV
)
echo "$TABLE"
