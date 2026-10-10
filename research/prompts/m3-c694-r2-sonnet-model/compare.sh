#!/usr/bin/env bash
# compare.sh extract <原始输出目录> <抽出目录>   ：从 *.raw 里抽 K4* 开头的判定行，去掉耗时字段（seconds=…），按批合并、LC_ALL=C 排序
# compare.sh compare <抽出目录> <期望目录>       ：逐个文件比，末行报「全同（N 个文件）」或列出不同的文件。不读 SINGLEFS_REPO。
set -euo pipefail
mode="${1:?extract | compare}"
case "$mode" in
extract)
    from="$2"; to="$3"; mkdir -p "$to"
    for raw in "$from"/*.raw; do
        [ -e "$raw" ] || continue
        base="$(basename "$raw" .raw)"
        group="$(echo "$base" | sed -E 's/-[0-9]+$//')"
        # 每行里第一个 K4 记号起的部分（libtest 的「test … ... 」前缀在同一行里）；去 seconds=…
        grep -o 'K4[A-Z0-9]* .*' "$raw" | sed -E 's/ seconds=[0-9.]+//' >> "$to/$group.txt.unsorted" || true
    done
    for unsorted in "$to"/*.txt.unsorted; do
        [ -e "$unsorted" ] || continue
        LC_ALL=C sort "$unsorted" > "${unsorted%.unsorted}"
        rm -f "$unsorted"
    done
    ;;
compare)
    a="$2"; b="$3"; different=0; count=0
    for expected in "$b"/*.txt; do
        count=$((count + 1)); name="$(basename "$expected")"
        if [ ! -e "$a/$name" ] || ! diff -q "$a/$name" "$expected" > /dev/null; then
            echo "不同：$name"; different=$((different + 1))
        fi
    done
    [ "$different" -eq 0 ] && echo "全同（$count 个文件）" || { echo "有 $different 个文件不同（共 $count 个）"; exit 1; }
    ;;
*) echo "用法：compare.sh extract|compare …" >&2; exit 2 ;;
esac
