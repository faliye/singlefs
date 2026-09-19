#!/usr/bin/env bash
# 把主工作区的 harness 文件拷进各份副本；proposal* 副本再打上「已知红第 1 条」的补丁。
set -euo pipefail
base=/tmp/claude-1000/m2-supp3-item1
"$base/sync-harness.sh" "$base/mutant-41/repo" "$base/mutant-121/repo" "$base/explore/repo" "$base/lint/repo"
for copy in proposal proposal-mutant-41 proposal-mutant-121; do
  "$base/sync-harness.sh" "$base/$copy/repo"
  python3 "$base/apply-provisional-patch.py" "$base/$copy/repo"
  rustfmt --edition 2021 "$base/$copy/repo/crates/singlefs-harness/src/history.rs" "$base/$copy/repo/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs"
done
