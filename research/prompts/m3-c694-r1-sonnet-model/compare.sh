#!/usr/bin/env bash
# compare.sh extract <原始输出目录> <抽出目录>   把 *.raw 里的结局行抽出来：去掉 test 名前缀、耗时与编译行，排序
# compare.sh compare <抽出目录> <expected 目录>  逐个文件比，全同退 0；不依赖 SINGLEFS_REPO
set -euo pipefail
mode="${1:?extract 或 compare}"
case "$mode" in
extract)
    from="$2"; to="$3"; mkdir -p "$to"
    for kind in probe-none probe-w1 probe-w4; do
        sed -E 's/^test [A-Za-z0-9_]+ \.\.\. //' "$from/$kind.raw.all" | grep -E '^(PROBE W=|PROBE_SESSION W=|SWITCHES)' | LC_ALL=C sort > "$to/$kind.txt"
    done
    for kind in temporary journal c695-none c695-k3c; do
        sed -E 's/^test [A-Za-z0-9_]+ \.\.\. //' "$from/$kind.raw" | grep -E '^(TEMPORARY|JOURNAL|RING|  W=|WIDTH|SWITCHES)' | LC_ALL=C sort > "$to/$kind.txt"
    done
    ;;
compare)
    a="$2"; b="$3"; bad=0
    for file in "$b"/*.txt; do
        name="$(basename "$file")"
        if ! diff -q "$a/$name" "$file" > /dev/null; then echo "不同：$name"; diff "$a/$name" "$file" | head -n 6; bad=1; fi
    done
    [ "$bad" = 0 ] && echo "全同（$(ls "$b"/*.txt | wc -l) 个文件）"
    exit "$bad"
    ;;
*) echo "用法：compare.sh extract|compare …" >&2; exit 2 ;;
esac
