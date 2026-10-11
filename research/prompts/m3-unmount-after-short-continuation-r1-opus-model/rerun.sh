#!/usr/bin/env bash
# 复跑：rerun.sh <工作目录> <件表文件> [批号]
# 从冻结副本拷一份到 <工作目录>/tree，打上两份补丁（甲 / 甲-b / 甲-c 由环境变量开，不设就是今天的代码），放进原型，经内存包装编译，
# 再用 run-batch.sh 跑件表（每件一个进程，内存上限默认 1G，PROBE_CAP 可改）。输出在 <工作目录>/out/b<批号>-<件号>.log。
set -euo pipefail
work="$1"; items="$2"; batch="${3:-rerun}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo=/home/fy5090/code/singlefs
frozen=/tmp/claude-1000/m3-unmount-after-short-continuation-r1/tree
mkdir -p "$work/out"
rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/"
(cd "$work/tree" && patch -p1 < "$model/jia-transaction.patch" && patch -p1 < "$model/jia-bc-mount.patch")
cp "$model/unmount_attack_probe.rs" "$work/tree/crates/singlefs-harness/tests/unmount_attack_probe.rs"
export CARGO_TARGET_DIR="$work/target"
(cd "$work/tree" && nice -n 19 bash "$repo/scripts/run-with-memory-cap.sh" 6G bash "$repo/scripts/capped.sh" 10 \
  cargo test --offline -p singlefs-harness --test unmount_attack_probe --no-run)
bin="$(ls -t "$work"/target/debug/deps/unmount_attack_probe-* | grep -v '\.d$' | head -1)"
(cd "$work" && bash "$model/run-batch.sh" "$batch" "$items" "$work/out" "$bin")
