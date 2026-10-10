#!/usr/bin/env bash
# 第 37 批代码轮第一轮正推腿的复跑：三个只读脚本（不编译）加一条 harness 档 cargo（经内存包装）。
# 用法：rerun.sh <冻结副本根，含 crates/> [harness]   不带第二个参数时不跑 cargo。
set -euo pipefail
tree="${1:?冻结副本根}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
e165="$tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
echo "== B1 三趟覆盖与变异推演"; python3 -I "$here/coverage.py" "$e165"
echo "== B2 nff 期望串对 IN_REPORT_ORDER"; python3 -I "$here/kinds.py" "$tree"
echo "== B4 两行变异的锚点命中数与落点"; python3 -I "$here/anchors.py" "$tree"
if [[ "${2:-}" == harness ]]; then
  echo "== B2 harness 档：25 的另一半（不是 checker 档，可跑）"
  target="$(mktemp -d)"; trap 'rm -rf "$target"' EXIT
  (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 8G \
    bash /home/fy5090/code/singlefs/research/scripts/capped.sh 15 \
    cargo test --offline --locked -p singlefs-harness --test new_pool_file_creation_publish)
fi
