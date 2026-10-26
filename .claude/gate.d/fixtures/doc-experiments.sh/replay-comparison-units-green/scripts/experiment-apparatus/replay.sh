#!/usr/bin/env bash
# 样本复跑表：只摆 TABLE 一段，不跑。
TABLE=$(cat <<'TSV'
E991|@driver_a||e991-a.out|exact
E991|@driver_b||e991-b.out|exact
E992|@driver_b||e992-x.out|exact
TSV
)
