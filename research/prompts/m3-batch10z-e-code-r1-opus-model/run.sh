#!/usr/bin/env bash
# 第 10z 批第五块代码轮第一轮攻方模型的复跑：把两份模型拷进冻结副本的一份拷贝（harness 档测试目录），跑这两个测试目标。
# 用法（仓根）：bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 \
#   bash research/prompts/m3-batch10z-e-code-r1-opus-model/run.sh <冻结副本的拷贝> <编译目录>
# 冻结副本本身只读：<冻结副本的拷贝> 先用 rsync -a --exclude target --exclude .git 从 ~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/ 拷。
# 改回去的探针（同样经内存包装）：python3 -I mutate_10ze.py <拷贝> <编译目录> <变异名…>；实现员没证红的行：python3 -I rows_10ze.py <拷贝> <编译目录> <行号…>。
# attack_10ze_rounds 的调度按种子定，同一 ATTACK_10ZE_SCRIPTS 两次跑计数相同；attack_10ze_window 量的是时长，每次不同。
set -euo pipefail
copy="${1:?冻结副本的拷贝}"
target="${2:?编译目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for name in attack_10ze_rounds attack_10ze_window; do
  cp "$model/$name.rs" "$copy/crates/singlefs-harness/tests/$name.rs"
done
cd "$copy"
CARGO_TARGET_DIR="$target" ATTACK_10ZE_SCRIPTS="${ATTACK_10ZE_SCRIPTS:-50000}" nice -n 19 cargo test --offline -p singlefs-harness \
  --test attack_10ze_rounds -- --nocapture --test-threads 1
CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline -p singlefs-harness --test attack_10ze_window -- --nocapture --test-threads 1
