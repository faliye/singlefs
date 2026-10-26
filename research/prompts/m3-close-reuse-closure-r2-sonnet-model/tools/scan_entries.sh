#!/usr/bin/env bash
# scan_entries.sh <树> <入口清单> <输出>：每个入口收一次闭包，一行：rc、单元数、拒算数、单元行排序后的 sha256（行号去掉）、入口。
set -u
tree="$1"; list="$2"; out="$3"
cd "$tree" || exit 2
: > "$out"
while read -r entry; do
  text="$(timeout 300 nice -n 19 python3 scripts/gate/lib_function_units.py closure . "$entry" 2>&1)"; rc=$?
  units="$(printf '%s\n' "$text" | awk -F'\t' '$1!="refused" && $1!="key" && NF>=4' | wc -l)"
  refused="$(printf '%s\n' "$text" | awk -F'\t' '$1=="refused"' | wc -l)"
  digest="$(printf '%s\n' "$text" | awk -F'\t' '$1!="refused" && $1!="key" && NF>=4 {split($3,a,":"); print $1"\t"$2"\t"a[1]"\t"$4}' | sort | sha256sum | cut -c1-12)"
  echo "rc=$rc units=$units refused=$refused digest=$digest $entry" >> "$out"
done < "$list"
