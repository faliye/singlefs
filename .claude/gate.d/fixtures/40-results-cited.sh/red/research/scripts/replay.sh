#!/usr/bin/env bash
# 门禁 40 号自检样本：只留登记表的骨架，不是真的 replay.sh。
set -uo pipefail
TABLE=$(cat <<'TSV'
E97|e97-sample||e97-sample.r2.out|exact
TSV
)
echo "$TABLE"
