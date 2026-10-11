#!/usr/bin/env bash
# 逐个变异目录跑 cargo test（release），每个变异输出每条用例的 ok / FAILED；经内存包装与线程上限。
set -uo pipefail
WORK=${1:?工作目录}
REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="正推腿小模型，与别的活并行"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="小模型没有峰值表项，统一给一个上限"
for d in "$WORK"/*/; do
  name=$(basename "$d")
    out=$(cd "$d" && CARGO_TARGET_DIR="$d/target" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 2G bash "$REPO/scripts/hosts/capped.sh" 7 cargo test --offline --release 2>&1)
  code=$?
  echo "== variant=$name exit=$code"
  echo "$out" | grep -E '^test .*(ok|FAILED)$|error(\[|:)|test result|^ +(left|right):|panicked at' | sed 's/^test tests:://'
done
