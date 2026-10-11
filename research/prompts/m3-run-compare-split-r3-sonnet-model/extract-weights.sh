#!/usr/bin/env bash
# 防守腿（m3-run-compare-split-r3）：从 9/29、9/30 两趟全量日志里取 CRASH_AMPLIFICATION_HOST_WEIGHTS 行，
# 配上紧随其后的第一条 flow= 行（流名与展开深度），看同一条流在不同趟的权重。只读日志。
set -euo pipefail
for f in full-run-20260929-f.log full-run-20260929-g.log full-run-20260930-i.log; do
  echo "== $f"
  awk '/HOST_WEIGHTS local/ {w=$0} /^CRASH_AMPLIFICATION flow=/ { if (w!="") { match($0,/flow=[^ ]+/); fl=substr($0,RSTART,RLENGTH); match($0,/expand_up_to=[0-9]+/); ex=substr($0,RSTART,RLENGTH); print NR": "w" -> "fl" "ex; w="" } }' "$HOME/.local/share/singlefs/crash-amplification/$f"
done
