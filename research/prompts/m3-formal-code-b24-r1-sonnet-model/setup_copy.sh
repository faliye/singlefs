#!/usr/bin/env bash
# 把冻结副本的 crates 拷到草稿目录当编译副本（排除 target），取仓根 Cargo.toml / Cargo.lock，放进 harness 档探针。
set -euo pipefail
FROZEN=/tmp/claude-1000/m3-formal-code-b24-r1/tree
WORK=/tmp/claude-1000/m3-formal-code-b24-r1-sonnet/copy
REPO=/home/fy5090/code/singlefs
MODEL=$REPO/research/prompts/m3-formal-code-b24-r1-sonnet-model
mkdir -p "$WORK"
rsync -a --delete --exclude target "$FROZEN/crates" "$WORK/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/"
# 别的活的半成品，不在这一轮里（挪开）
rm -f "$WORK/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
cp "$MODEL/probe/b24_segment_probe.rs" "$WORK/crates/singlefs-harness/tests/b24_segment_probe.rs"
cp "$MODEL/probe/b24_b1_probe.rs" "$WORK/crates/singlefs-harness/tests/b24_b1_probe.rs"
# 撕裂探针 = 冻结副本里原测试文件 + 追加的一条（探针要用原文件里的私有函数，所以拼在同一个文件里）
cat "$FROZEN/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs" "$MODEL/probe/b24_torn_probe_tail.rs" > "$WORK/crates/singlefs-harness/tests/b24_torn_probe.rs"
