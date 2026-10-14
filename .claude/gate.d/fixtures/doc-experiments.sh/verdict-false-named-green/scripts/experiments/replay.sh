#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 verdict-false-named 格的样本：只留判读表格所需的最小骨架，不是真的 replay.sh。
set -uo pipefail
TABLE=$(cat <<'TSV'
E9001|e9001-sample||e9001-sample-2026-09-24.out|exact
E7|e7-sample||e7-sample-2026-09-24.out|exact
TSV
)
echo "$TABLE"
