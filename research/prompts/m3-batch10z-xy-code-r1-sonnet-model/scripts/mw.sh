#!/usr/bin/env bash
# mw.sh <行文件前缀> <预算秒> [线程=2]：三个树（m1 m2 m3）并行，各跑 <前缀>-<i>.tsv，结果追加到 runs/done-<前缀>-<i>.txt
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"
cd "$W"
for i in 1 2 3; do
  python3 scripts/mutworker.py "$W/tree-m$i" "$W/target-m$i" "scripts/$1-$i.tsv" "runs/done-$1-$i.txt" "$2" 4G "${3:-2}" &
done
wait
for i in 1 2 3; do echo "== $1-$i: $(wc -l < scripts/$1-$i.tsv) 行，已出 $(wc -l < runs/done-$1-$i.txt 2>/dev/null)"; done
