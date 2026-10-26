#!/usr/bin/env bash
# 第三轮攻方模型全部步骤：造变体树、35 个入口四棵树的闭包、四个碰得到包装的入口在弄坏树上的闭包、模块级普查（ast）、
# 甲要打包的文件（strace）、打包后在空目录里跑、真内存包装在弄坏树上起跑、X3 被调方换空文件。不变的行汇总进 <工作目录>/summary.txt。
# 用法：run-all.sh <原树> <工作目录>（原树里被判九份的 sha256 要与开工快照相同；只读原树，变体树建在工作目录）
set -u
base="$(cd "$1" && pwd)" work="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$work"
summary="$work/summary.txt"; : > "$summary"
python3 -B "$here/make-trees.py" "$base" "$work/trees" > "$work/make-trees.log" || { echo "make-trees 失败"; exit 3; }
bash "$here/run-closures.sh" "$base" "$work/trees" "$here/entries.txt" "$work/closures" 18 full cut ml mlh > "$work/closures.log"
bash "$here/run-closures.sh" "$base" "$work/trees" "$here/entries-wrapper.txt" "$work/closures-break" 20 ml mlh ml-classbreak ml-defbreak ml-assignbreak mlh-classbreak mlh-defbreak > "$work/closures-break.log"
{ grep -c 'rc=4' "$work/closures.log" | sed 's/^/name=closure_runs_refused count=/'
  python3 -B "$here/summarize-closures.py" "$work/closures" "$here/entries.txt" full full cut ml mlh | sed -E 's/ judging_digest=[0-9a-f]+//'
  python3 -B "$here/summarize-closures.py" "$work/closures" "$here/entries.txt" cut ml | sed -E 's/ judging_digest=[0-9a-f]+//'
  python3 -B "$here/summarize-closures.py" "$work/closures-break" "$here/entries-wrapper.txt" ml ml-classbreak ml-defbreak ml-assignbreak | sed -E 's/ judging_digest=[0-9a-f]+//'
  python3 -B "$here/summarize-closures.py" "$work/closures-break" "$here/entries-wrapper.txt" mlh mlh-classbreak mlh-defbreak | sed -E 's/ judging_digest=[0-9a-f]+//'
  for tree in full cut ml mlh; do
    root="$work/trees/$tree"; [[ $tree == full ]] && root="$base"
    for index in 32 6; do
      echo "name=raw_unit_lines tree=$tree index=$index lines=$(awk -F'\t' '$1!="via" && $1!="refused" && $1!="key" && NF>=4' "$work/closures/c-$tree-$index.txt" | wc -l)"
      python3 -B "$here/module-census-ast.py" "$root" "$work/closures/c-$tree-$index.txt" "$tree:$index" | head -1
    done
  done
  echo "name=not_judging_units_in_cut_check_proof lib_verdict_library=$(grep -cP '^(?!via)[^\t]+\t[^\t]+\tscripts/crash/lib_verdict_library\.py:' "$work/closures/c-cut-32.txt") lib_function_units=$(grep -cP '^(?!via)[^\t]+\t[^\t]+\tscripts/gate/lib_function_units\.py:' "$work/closures/c-cut-32.txt")"
  for tree in full cut ml mlh; do
    echo "name=host_resources_units_in_check_proof tree=$tree units=$(grep -cP '^(?!via)[^\t]+\t[^\t]+\tscripts/hosts/host-resources\.py:' "$work/closures/c-$tree-32.txt")"
  done
  bash "$here/x3-empty-callee.sh" "$base" "$work/trees" "$work/x3"
  bash "$here/needed-files.sh" "$base" "$work/needed" | grep -v '^name=needed_files_run'
  bash "$here/packed-run.sh" "$base" "$work/needed/repo-files.txt" "$work/packed"
  for tree in ml ml-classbreak ml-defbreak ml-assignbreak; do for world in start predict; do
    bash "$here/wrapper-world.sh" "$work/trees/$tree" "$tree" "$world" "$work/worlds"
  done; done
} >> "$summary" 2>&1
cat "$summary"
