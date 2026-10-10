#!/usr/bin/env bash
# m3-retire-r1 正推腿的小模型（纯 Python，不读 crates/，不编译）：退役记录的写法（集合差）、四种扫描重建读者、checker 候选判据 a b c d。
# 用法：
#   SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> run       # 跑 model.py 300 段历史 × 每段 60 步（经内存包装，上限 4G），产物 <工作目录>/out.txt
#   bash rerun.sh <工作目录> compare                         # 排序后与 expected/summary.txt 逐行比（不依赖 SINGLEFS_REPO）
#   SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> all        # run、compare 依次
# 产物里没有耗时字段。只用 python3 标准库；-I 隔离模式，不读当前目录。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="${1:?用法：rerun.sh <工作目录> run|compare|all}"
stage="${2:?用法：rerun.sh <工作目录> run|compare|all}"
histories=300
ops=60

do_run() {
  : "${SINGLEFS_REPO:?要设 SINGLEFS_REPO=<仓根>}"
  mkdir -p "$work"
  bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh" 4G nice -n 19 python3 -I "$here/model.py" "$histories" "$ops" > "$work/out.txt"
  echo "run 完成：$work/out.txt（$(wc -l < "$work/out.txt") 行）"
}

do_compare() {
  [[ -f "$work/out.txt" ]] || { echo "没有 $work/out.txt，先跑 run" >&2; exit 4; }
  if diff <(sort "$here/expected/summary.txt") <(sort "$work/out.txt"); then
    echo "compare：与 expected/summary.txt 逐行相同（排序后）"
  else
    echo "compare：有不同的行" >&2
    exit 1
  fi
}

case "$stage" in
  run) do_run ;;
  compare) do_compare ;;
  all) do_run; do_compare ;;
  *) echo "用法：rerun.sh <工作目录> run|compare|all" >&2; exit 2 ;;
esac
