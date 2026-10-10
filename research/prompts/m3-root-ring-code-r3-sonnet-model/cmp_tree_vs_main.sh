#!/usr/bin/env bash
# 冻结副本 crates/ 下每份 .rs / .wgsl / .tsv 与主工作区同路径逐字节比；只读。
T=/tmp/claude-1000/m3-root-ring-code-r3/tree
cd /home/fy5090/code/singlefs
n=0; d=0
while read -r f; do
  n=$((n+1))
  if [ -f "$f" ]; then cmp -s "$T/$f" "$f" || { echo "DIFF $f"; d=$((d+1)); }; else echo "MISSING $f"; d=$((d+1)); fi
done < <(cd "$T" && find crates -type f \( -name '*.rs' -o -name '*.wgsl' -o -name '*.tsv' \) | sort)
echo "compared=$n differ=$d"
