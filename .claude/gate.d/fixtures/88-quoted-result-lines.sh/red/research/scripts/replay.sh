#!/usr/bin/env bash
# 门禁 88 号自检样本：只留登记表的最小骨架，不是真的 replay.sh。
TABLE=$(cat <<'TSV'
E99|e99-sample||e99-sample-2026-09-16.out|exact
TSV
)
echo "$TABLE"
