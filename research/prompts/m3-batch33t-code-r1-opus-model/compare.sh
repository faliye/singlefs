# 用法：bash compare.sh <复跑出来的 summary.txt>（与本目录 expected/summary.txt 比；比之前两边各自排序、去掉耗时字段；不依赖 SINGLEFS_REPO）
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
got="${1:?要复跑出来的 summary.txt}"
norm() { sed -e 's/; finished in [^ ]*//' -e 's/ finished in [0-9.]*s//' "$1" | LC_ALL=C sort; }
if diff <(norm "$here/expected/summary.txt") <(norm "$got") > /dev/null; then
  echo "全同（$(wc -l < "$got") 行）"
else
  diff <(norm "$here/expected/summary.txt") <(norm "$got") | head -n 40
  echo "有差异"
  exit 1
fi
