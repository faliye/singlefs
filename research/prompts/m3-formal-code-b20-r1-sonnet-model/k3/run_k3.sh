#!/usr/bin/env bash
# 在副本 $W 上：基线、行 2368、行 2369 各跑一次真用例与探针（harness 档，经内存包装、线程上限 10）。
# 用法：bash run_k3.sh   （W、T、R 三个路径按本轮的写死，见下）
set -u
W=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work
T=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/target
R=/home/fy5090/code/singlefs
M=$R/research/prompts/m3-formal-code-b20-r1-sonnet-model
OUT=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/k3-out
mkdir -p "$OUT"
cp "$M/k3/probe_k3.rs" "$W/crates/singlefs-harness/tests/probe_k3.rs"
cd "$W"
export CARGO_TARGET_DIR=$T
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一轮派出的正推腿与攻方腿各跑各的，各用各的副本与编译目录"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="变异副本重编 harness 的峰值大于只跑测试的峰值，取 8G 默认上限"
run() {  # $1 标签
  for tgt in model_answers_unmount_with_the_floor_at_the_current_version probe_k3; do
    bash $R/scripts/run-with-memory-cap.sh 8G bash $R/scripts/capped.sh 10 nice -n 19 \
      cargo test --offline -p singlefs-harness --test $tgt -- --nocapture --test-threads=1 > "$OUT/$1.$tgt.log" 2>&1
    echo "$1 $tgt rc=$?" >> "$OUT/summary.txt"
  done
}
: > "$OUT/summary.txt"
cp "$W/crates/singlefs-harness/src/model.rs" "$OUT/model.rs.orig"
run baseline
python3 $M/anchor_check.py /tmp/claude-1000/m3-formal-code-b20-r1/tree 2368 2369 --emit "$OUT/emit" > "$OUT/anchors.txt"
for row in 2368 2369; do
  cp "$OUT/emit/row$row.rs" "$W/crates/singlefs-harness/src/model.rs"
  run row$row
done
cp "$OUT/model.rs.orig" "$W/crates/singlefs-harness/src/model.rs"
cmp "$OUT/model.rs.orig" "$W/crates/singlefs-harness/src/model.rs" && echo "model.rs 已还原" >> "$OUT/summary.txt"
