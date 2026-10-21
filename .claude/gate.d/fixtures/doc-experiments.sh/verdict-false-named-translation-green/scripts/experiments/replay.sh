#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 verdict-false-named 格的样本：只留判读表格所需的最小骨架，不是真的 replay.sh。
set -uo pipefail
TABLE=$(cat <<'TSV'
E174|e174-sample||e174-sample-2026-10-21.out|exact
TSV
)
echo "$TABLE"
