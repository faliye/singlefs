#!/usr/bin/env bash
# 在同一个包装槽里把全部 E165 副本二进制逐个跑完（light 档，线程 4）：基线、卸载记号版、各变异版各一遍合法一族与对抗一族。
set -u
cd /tmp/claude-1000/m3-formal-r3-sonnet/work
mkdir -p runs
: > runs/batch-e165.exits
run() {  # run <名> <二进制> <参数…>   （环境变量由调用处在前面设）
  name="$1"; bin="$2"; shift 2
  ( export E165_THREADS=4; ./"$bin" "$@" > runs/$name.out 2> runs/$name.err; echo "$name exit=$?" >> runs/batch-e165.exits )
}
run base-legal bin-baseline-patched --bounds light --family legal
run base-adversarial bin-baseline-patched --bounds light --family adversarial
( export S1_MARKER=1; run base-legal-marker bin-baseline-patched --bounds light --family legal )
for v in 406 513 518 1003 1004 24 512 36; do
  [ -x bin-v$v ] || { echo "bin-v$v 缺" >> runs/batch-e165.exits; continue; }
  run v$v-legal bin-v$v --bounds light --family legal
  run v$v-adversarial bin-v$v --bounds light --family adversarial
done
echo batch-done >> runs/batch-e165.exits
