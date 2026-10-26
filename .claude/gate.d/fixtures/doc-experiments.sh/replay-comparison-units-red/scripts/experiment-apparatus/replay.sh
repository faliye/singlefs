#!/usr/bin/env bash
# 样本复跑表：只摆 TABLE 一段，不跑。
TABLE=$(cat <<'TSV'
E991|@driver_a||e991-a.out|exact
E991|@driver_b||e991-b.out|exact
E991|@driver_c||e991-c.out|exact
E991|@driver_d||e991-d.out|exact
E991|@driver_e||e991-e.out|exact
E991|@driver_g||e991-other.out|exact
E992|@driver_absent||e992-x.out|exact
TSV
)
