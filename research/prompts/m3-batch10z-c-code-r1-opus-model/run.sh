#!/usr/bin/env bash
# 第 10z 批第三块代码轮第一轮攻方模型的复跑：把三份模型拷进冻结副本的一份拷贝（harness 档测试目录），跑这三个测试目标。
# 用法（仓根）：bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 \
#   bash research/prompts/m3-batch10z-c-code-r1-opus-model/run.sh <冻结副本的拷贝> <编译目录>
# 冻结副本本身只读：<冻结副本的拷贝> 先用 rsync -a --exclude target --exclude .git 从 ~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/ 拷。
# 变异探针：python3 research/prompts/m3-batch10z-c-code-r1-opus-model/mutate_10zc.py <拷贝> <编译目录> [--with-models] <变异名…>（同样经内存包装）。
# k1_5 的随机一格与 k1_4 的同组重跑靠真线程的先后，两次跑的计数会差；定形那几格（k1_4 甲乙、k1_5 stop-in-the-closing-round）每次相同。
set -euo pipefail
copy="${1:?冻结副本的拷贝}"
target="${2:?编译目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for name in attack_10zc_endings attack_10zc_rounds attack_10zc_ledger; do
  cp "$model/$name.rs" "$copy/crates/singlefs-harness/tests/$name.rs"
done
cd "$copy"
CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline -p singlefs-harness \
  --test attack_10zc_endings --test attack_10zc_rounds --test attack_10zc_ledger -- --nocapture --test-threads 1
