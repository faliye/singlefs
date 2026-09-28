#!/bin/bash
# A7「B1 只当标签」的会红检查：两种类型写法（弱：标签有 Display；收严：标签不给任何格式化、只交只写的覆盖报告）下，
# 四种把标签当复用键的写法各编一次，编不过就是检查红了。rustc 只编元数据，不跑任何东西。
# 用法：bash a7_label_guard.sh <输出目录>
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:?输出目录}"
mkdir -p "$out"
bypass_compiling_under_weak=0
for guard in weak strong; do
  for case in label_as_store_key label_in_a_hash_map forge_the_key_from_a_label label_text_as_an_in_memory_key; do
    source_file="$here/case_${case}_${guard}.rs"
    if rustc --edition 2021 --crate-type lib --emit=metadata -o "$out/${case}_${guard}.rmeta" "$source_file" > "$out/${case}_${guard}.log" 2>&1; then
      verdict=compiles
      if [ "$guard" = weak ]; then bypass_compiling_under_weak=$((bypass_compiling_under_weak + 1)); fi
    else
      verdict="refused:$(grep -oE 'error\[E[0-9]+\]' "$out/${case}_${guard}.log" | head -1 | tr -d '[]' | sed 's/error//')"
    fi
    echo "E7RESULT name=r3_a7_label_guard guard=$guard case=$case $verdict"
  done
done
echo "E7RESULT name=r3_a7_label_guard_summary label_as_key_compiling_under_the_weak_guard=$bypass_compiling_under_weak must_be_nonzero=$bypass_compiling_under_weak"
