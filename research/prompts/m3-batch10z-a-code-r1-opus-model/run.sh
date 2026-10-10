#!/usr/bin/env bash
# 第 10z 批代码轮第一轮攻方模型的复跑：把两份模型拷进一份冻结副本的拷贝（harness 档测试目录），跑这两个测试目标。
# 用法（仓根）：bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 15 \
#   bash research/prompts/m3-batch10z-a-code-r1-opus-model/run.sh <冻结副本的拷贝> <编译目录>
# 冻结副本本身只读：<冻结副本的拷贝> 先用 rsync -a --exclude target --exclude .git 从 ~/.local/share/singlefs/work/m3-batch10z-a-code-r1/tree/ 拷。
set -euo pipefail
copy="${1:?冻结副本的拷贝}"
target="${2:?编译目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "$model/ledger_attack_10z.rs" "$copy/crates/singlefs-harness/tests/ledger_attack_10z.rs"
cp "$model/p2_blocks_10z.rs" "$copy/crates/singlefs-harness/tests/p2_blocks_10z.rs"
cd "$copy"
CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline -p singlefs-harness \
  --test ledger_attack_10z --test p2_blocks_10z -- --nocapture --test-threads 1
