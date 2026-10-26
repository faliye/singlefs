#!/usr/bin/env bash
# V3：仓里今天每一处 inline-script-not-literal 拒算（shell 与 python 各 dump 一遍），列文件:行与那一行原文（至多 170 字）。用法：inline_refusals.sh <冻结副本>
set -u
cd "$1" || exit 2
for language in shell python; do
  echo "== $language"
  python3 scripts/gate/lib_function_units.py dump . --language "$language" 2>/dev/null | awk -F'\t' '$1=="refused" && $2=="inline-script-not-literal"{print $3"\t"$4}' | cut -c1-170
done
