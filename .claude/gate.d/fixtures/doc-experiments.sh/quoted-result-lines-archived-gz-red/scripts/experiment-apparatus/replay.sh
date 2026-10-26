#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 quoted-result-lines 格的样本：只留登记表的最小骨架，不是真的 replay.sh。
TABLE=$(cat <<'TSV'
E98|e98-sample||e98-archived-gz-2026-10-22.out.gz|exact
TSV
)
echo "$TABLE"
