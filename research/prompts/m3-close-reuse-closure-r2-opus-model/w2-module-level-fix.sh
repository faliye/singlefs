#!/usr/bin/env bash
# W2：我提的闭包改法（make-variants.py 的 cut-ml：模块有单元进闭包时，它模块级的 import、assign、class 一律进闭包）量一遍——
# 在每棵变体树里用那棵树自己的 lib_function_units.py / lib_verdict_library.py 收两个入口的闭包、算判法单元指纹（judging-units），
# 报单元数、拒算、lib_long_run.py 的单元数、memory_peaks_lib.py 的 import subprocess 在不在、键。四棵树并行，每件写自己的行文件，wait 之后数齐再读。
# 用法：w2-module-level-fix.sh <变体目录> <输出目录>
set -u
variants="$1" out="$2"
mkdir -p "$out"
trees=(cut cut-ml cut-mlc cut-ml-long-run cut-ml-mpl cut-ml-hr)
rm -f "$out"/ml-line-*.txt
for tree in "${trees[@]}"; do
  { root="$variants/$tree"
    for entry in scripts/gate/verus-proof-check.py::check_proof scripts/mutation/broad-mutation.py::Runner.escalate_step; do
      label="$(printf '%s' "$entry" | tr '/:.' '___')"
      (cd "$root" && python3 -B scripts/gate/lib_function_units.py closure . "$entry") > "$out/ml-closure-$tree-$label.txt" 2>&1
      rc=$?
      c="$out/ml-closure-$tree-$label.txt"
      key="$(cd "$root" && python3 -B scripts/crash/lib_verdict_library.py judging-units . "$entry" 2>/dev/null | awk -F'\t' '$1=="key"{print substr($2,1,16)}')"
      echo "name=module_level_fix tree=$tree entry=$entry rc=$rc units=$(awk -F'\t' 'NF>=4 && $1!="refused" && $1!="key"' "$c" | wc -l) lib_long_run_units=$(grep -cP '\t\.claude/hooks/lib_long_run\.py:' "$c") memory_peaks_lib_import_subprocess=$(grep -cP '^import\tsubprocess\tscripts/hosts/memory_peaks_lib\.py:' "$c") key=${key:-none} refusals=$(awk -F'\t' '$1=="refused"{print $2" "$3}' "$c" | tr '\n' ',')"
    done > "$out/ml-line-$tree.txt"; } &
done
wait
count=$(ls "$out"/ml-line-*.txt 2>/dev/null | wc -l)
[[ $count -eq ${#trees[@]} ]] || { echo "name=module_level_fix_incomplete files=$count（这一批作废）"; exit 0; }
for tree in "${trees[@]}"; do cat "$out/ml-line-$tree.txt"; done
