#!/bin/bash
# 从 12 份 grid 日志抽 summary.txt 与 pivot.txt（报告 K2-2 节引的两份）。用法：bash summarize.sh <放 grid-*.log 的目录>
# 被拒的那几段日志里没有 checker_red 字段，pivot 的 red 列把它们也数进去了：读 pivot 时 refused 不为 0 的格看 refused，不看 red。
set -euo pipefail
cd "${1:?目录}"
for f in grid-*.log; do
  grep -h '^name=k2-cell' "$f" | while read -r l; do
    echo "${f#grid-}|$(echo "$l" | grep -o 'cell="[^"]*"' | sed 's/cell=//;s/"//g')|$(echo "$l" | grep -o 'result=[A-Za-z]*([A-Za-z]*\|result=Ok')|$(echo "$l" | grep -o 'ring=[A-Za-z]*')|$(echo "$l" | grep -o 'isolated=[^]]*]' | tr -d ' ' | sed 's/DeviceIdentity//g;s/isolated=//')|$(echo "$l" | grep -o 'free_c_places_after_mount=[0-9]*' | sed 's/.*=//')|$(echo "$l" | grep -o 'reused_after_publishes=[0-9]* txg=[0-9]*\|no_reuse_after=[0-9]*\|overwrite_refused_at=[0-9]*')|$(echo "$l" | grep -o 'floor_after=[0-9]*')|$(echo "$l" | grep -o 'checker_red=.*')"
  done
done | sed 's/\.log|/|/' > summary.txt
awk -F'|' '{split($1,a,"-"); arm=a[3]; key=$2; red=$9; res=$3; iso=$5; c[key"|"arm]++; if (red!="checker_red={}") r[key"|"arm]++; if (iso ~ /\(\(0\),0\)/) z[key"|"arm]++; if (res!="result=Ok") e[key"|"arm]++; cells[key]=1} END {for (k in cells) { printf "%s", k; split("rr2 before merge",A," "); for (i=1;i<=3;i++){kk=k"|"A[i]; printf " | %s: n=%d red=%d iso0=%d refused=%d", A[i], c[kk], r[kk], z[kk], e[kk]} print ""}}' summary.txt | sort > pivot.txt
wc -l summary.txt pivot.txt
