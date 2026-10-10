#!/usr/bin/env bash
# 用法：run_variant.sh <副本树> <冻结副本> <输出目录> <变体名或 baseline> [测试目标…]
# SKIP_SWEEP=1 时只跑点名的测试目标（不跑探针扫描与 30-1 入库用例）；TAG=<名> 时产物名带 -<名>。
# 两份源码先从冻结副本还原，再换进变体；跑 b30 攻方探针、30-1 入库用例与点名的测试目标，结局写 <输出目录>/<变体>.txt。
set -u
tree=$1; frozen=$2; out=$3; variant=$4; shift 4
here=$(cd "$(dirname "$0")" && pwd)
for f in crates/singlefs-harness/src/model_comparison.rs crates/singlefs-harness/src/model.rs; do
  cp "$frozen/$f" "$tree/$f"
done
cp "$here/../tests/b30_opus_leaf_container_sweep.rs" "$tree/crates/singlefs-harness/tests/"
if [[ $variant != baseline ]]; then
  python3 "$here/apply_variant.py" "$tree" "$variant" || exit 2
fi
tag=${TAG:+-$TAG}; log="$out/$variant$tag.log"; summary="$out/$variant$tag.txt"
: > "$log"
if [[ -n ${SKIP_SWEEP:-} ]]; then targets=("$@"); else targets=(b30_opus_leaf_container_sweep model_compares_the_third_and_fourth_inode_leaf_containers "$@"); fi
for target in "${targets[@]}"; do
  if [[ $target == lib ]]; then sel=(--lib); else sel=(--test "$target"); fi
  echo "== $target" >> "$log"
  "$here/cargo_capped.sh" "$tree" test --offline -p singlefs-harness "${sel[@]}" -- --nocapture >> "$log" 2>&1
  echo "rc=$? target=$target" >> "$log"
done
grep -E '^== |^test .* \.\.\. |^test result|^rc=|^SWEEP .*completed=false|error\[' "$log" | sed -E 's/ finished in [0-9.]+s//' > "$summary"
for f in crates/singlefs-harness/src/model_comparison.rs crates/singlefs-harness/src/model.rs; do
  cp "$frozen/$f" "$tree/$f"
done
grep -E '^rc=|test result' "$summary"
