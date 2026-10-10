#!/usr/bin/env bash
# 用法：complete.sh <日志…>   只留跑完了（有 FXDONE 那一行）的起点的 FX 行，再交给 fxsum2.sh 汇总（被限时掐断的那个起点整个不算）。
for f in "$@"; do
  awk '
  FNR==NR { if ($1=="FXDONE") { l=$0; sub(/^FXDONE /,"",l); sub(/ elapsed=.*/,"",l); done[l]=1 } next }
  $1=="START_FAILED" { print; next }
  $1 ~ /^FX/ { l=$0; sub(/^FX[A-Z_]* /,"",l); if ($1=="FXSTART") sub(/ \|.*/,"",l); else sub(/ n=.*/,"",l); if (l in done) print }
  ' "$f" "$f"
done
