#!/usr/bin/env bash
# 门禁 doc-experiments.sh 的 verdict-false-named 格的样本：登记表格式没坏，但一行数据都没有——用来核「扫到 0 项不能悄悄算通过」。
set -uo pipefail
TABLE=$(cat <<'TSV'
TSV
)
echo "$TABLE"
