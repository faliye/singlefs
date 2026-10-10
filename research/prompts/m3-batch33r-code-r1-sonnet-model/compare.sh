#!/usr/bin/env bash
# 用法：bash compare.sh <summary 甲> <summary 乙>   各自排序、去掉耗时字段（elapsed=…、finished in …、real/user/sys 行）后逐行比；相同退 0、不同退 1 并列出差异。自己不依赖 SINGLEFS_REPO。
set -euo pipefail
[[ $# -eq 2 ]] || { echo "用法：bash compare.sh <summary 甲> <summary 乙>" >&2; exit 2; }
normalize() { grep -vE '^(real|user|sys)[[:space:]]' "$1" | sed -E 's/ elapsed[_a-z]*=[0-9.]+//g; s/finished in [0-9.]+s//g' | LC_ALL=C sort; }
if diff <(normalize "$1") <(normalize "$2") > /dev/null; then
  echo "SAME $(normalize "$1" | wc -l) 行"
else
  echo "DIFFERENT"; diff <(normalize "$1") <(normalize "$2") || true; exit 1
fi
