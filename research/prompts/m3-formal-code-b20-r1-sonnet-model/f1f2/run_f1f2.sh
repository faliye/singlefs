#!/usr/bin/env bash
# 把探针拷进副本的 harness 测试目录，harness 档编译并跑（经内存包装、线程上限 10），输出落草稿目录。
set -u
W=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work
export CARGO_TARGET_DIR=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/target
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一轮派出的正推腿与攻方腿各跑各的，各用各的副本与编译目录"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="腿的探针在副本里编译，峰值大于只跑测试的峰值，取默认 8G"
R=/home/fy5090/code/singlefs
OUT=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/f1f2-out; mkdir -p "$OUT"
cp "$R/research/prompts/m3-formal-code-b20-r1-sonnet-model/f1f2/probe_f1f2.rs" "$W/crates/singlefs-harness/tests/probe_f1f2.rs"
cd "$W"
bash $R/scripts/run-with-memory-cap.sh 8G bash $R/scripts/capped.sh 10 nice -n 19 \
  cargo test --offline -p singlefs-harness --test probe_f1f2 -- --nocapture --test-threads=1 > "$OUT/probe.log" 2>&1
echo "rc=$?" >> "$OUT/probe.log"
