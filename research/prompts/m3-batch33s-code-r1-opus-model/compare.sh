# 用法：bash compare.sh <summary.txt> <expected-summary.txt>   （两份各自排序、去掉耗时字段再逐行比；一致退 0，不一致打 diff 退 1；不读 SINGLEFS_REPO）
set -euo pipefail
normalize() { sed -E 's/; finished in [0-9.]+s//; s/ finished in [0-9.]+s//' "$1" | sort -u; }
if diff <(normalize "$1") <(normalize "$2"); then
    echo "COMPARE same lines=$(normalize "$1" | wc -l)"
else
    echo "COMPARE differ"
    exit 1
fi
