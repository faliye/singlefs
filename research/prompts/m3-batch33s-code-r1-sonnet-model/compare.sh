#!/usr/bin/env bash
# 用法：bash compare.sh <rerun.out 甲> <rerun.out 乙>   （比两次复跑；排序后逐行比，去掉耗时字段；不读 SINGLEFS_REPO；相同退 0，不同打差异退 1）
set -euo pipefail
a=${1:?甲}; b=${2:?乙}
norm() { sed -E 's/finished in [0-9.]+s//; s/elapsed=[0-9.]+[a-z]*//g' "$1" | LC_ALL=C sort; }
diff <(norm "$a") <(norm "$b") && echo "相同"
