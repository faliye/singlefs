#!/usr/bin/env bash
# 编基线与 F5 变异（mutations.tsv 2366 行那一行）两份纯函数模型，经内存包装跑，输出落草稿目录。
set -u
M=/home/fy5090/code/singlefs/research/prompts/m3-formal-code-b20-r1-sonnet-model/pure
R=/home/fy5090/code/singlefs
D=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/pure-out; mkdir -p $D
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一轮派出的正推腿与攻方腿各跑各的，各用各的副本与编译目录"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="腿自己的小模型，用默认上限"
python3 - <<'PY'
import os
M=os.environ.get("M","/home/fy5090/code/singlefs/research/prompts/m3-formal-code-b20-r1-sonnet-model/pure")
D="/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/pure-out"
src=open(M+"/pure_base.rs",encoding="utf-8").read()
frm="    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]"
to="    first_write_of_the_segment + last_write_within_the_crash_window"
assert src.count(frm)==1
open(D+"/pure_row2366.rs","w",encoding="utf-8").write(src.replace(frm,to))
open(D+"/pure_base.rs","w",encoding="utf-8").write(src)
PY
for v in base row2366; do
  rustc --edition 2021 -o $D/pure_$v $D/pure_$v.rs 2> $D/compile_$v.log || { echo "编译 $v 失败"; cat $D/compile_$v.log | head -20; continue; }
  bash $R/research/scripts/run-with-memory-cap.sh 512M $D/pure_$v > $D/run_$v.out 2>&1; echo "$v rc=$?" >> $D/rc.txt
done
