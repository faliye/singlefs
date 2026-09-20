#!/usr/bin/env bash
# check.sh 前后对比：HEAD 副本先热一趟（编译），再 HEAD、主工作区各计一趟。
set -u
scratch=/tmp/claude-1000/m2-supp3-item2-implementer
head_copy=$scratch/copies/head-before
repository=/home/fy5090/code/singlefs
timed() {
  local label=$1 directory=$2
  local start end
  start=$(date +%s)
  (cd "$directory" && nice -n 19 bash .claude/scripts/check.sh) > "$scratch/check-$label.log" 2>&1
  local code=$?
  end=$(date +%s)
  echo "$label：退出码 $code，墙钟 $((end - start)) 秒，$(date -u +%H:%M:%S) UTC 结束" >> "$scratch/check-timing.txt"
}
: > "$scratch/check-timing.txt"
timed head-warm-up "$head_copy"
timed head "$head_copy"
timed current "$repository"
timed head-again "$head_copy"
timed current-again "$repository"
