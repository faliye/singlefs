#!/usr/bin/env bash
# 把主工作区里这一轮改的 harness 文件拷进各份副本并 touch（副本里的 singlefs-core 保持各自的样子）。
set -euo pipefail
source_root=/home/fy5090/code/singlefs
for copy in "$@"; do
  for file in crates/singlefs-harness/src/history.rs crates/singlefs-harness/src/lib.rs crates/singlefs-harness/src/crash.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs; do
    cp "$source_root/$file" "$copy/$file"
    touch "$copy/$file"
  done
done
