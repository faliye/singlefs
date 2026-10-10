#!/usr/bin/env bash
# 用法：compare.sh <复跑的 summary.txt> <基线 summary.txt>：两份都按行排序之后逐行比（不依赖 SINGLEFS_REPO）；一样退 0，不一样打 diff 退 1。
set -euo pipefail
diff <(LC_ALL=C sort "$1") <(LC_ALL=C sort "$2") && echo "SAME $(wc -l < "$1") 行"
