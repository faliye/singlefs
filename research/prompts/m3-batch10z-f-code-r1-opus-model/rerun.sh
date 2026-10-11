#!/usr/bin/env bash
# 第 10z 批第六块代码轮攻方（opus）模型复跑。用法：bash rerun.sh <草稿目录>（仓根下跑；经内存包装、线程上限 8；只编、跑 harness 档）
# 在草稿目录里建两份冻结副本拷贝：copy-a 原样、copy-b 打攻方改法 F-K1、F-K3（fix-fk1-fk3.diff），各用自己的编译目录。
# 产出：out-a.txt（原样：K1-2、K1-3、K2-2 三条模型）、out-b.txt（改法上的 K1-2、K1-3）、mutants.txt（harness 档 25 条改坏）、unwind.txt（Rust 语义）。
set -euo pipefail
D=${1:?草稿目录}
M=$(cd "$(dirname "$0")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree
mkdir -p "$D"
for c in a b; do
  rsync -a --delete --exclude target --exclude .git "$FROZEN/" "$D/copy-$c/"
  cp "$M/attack_10zf_ledger.rs" "$M/attack_10zf_rounds.rs" "$D/copy-$c/crates/singlefs-harness/tests/"
done
patch "$D/copy-b/crates/singlefs-harness/src/gpu_memory_ledger.rs" < "$M/fix-fk1-fk3.diff"
ROOT=$PWD
run_tests() { # $1 拷贝  $2 输出  其余：cargo test 的目标
  local copy=$1 out=$2; shift 2
  (cd "$D/copy-$copy" && CARGO_TARGET_DIR="$D/target-$copy" nice -n 19 bash $ROOT/scripts/hosts/run-with-memory-cap.sh 3G bash "$ROOT/scripts/hosts/capped.sh" 8 \
     cargo test --offline --no-fail-fast -p singlefs-harness "$@" -- --nocapture --test-threads 1) > "$D/$out.log" 2>&1
  sed 's/^test [a-z0-9_]* \.\.\. //' "$D/$out.log" | grep '^ATTACK' > "$D/$out.txt" || true
  wc -l < "$D/$out.txt"
}
run_tests a out-a --test attack_10zf_ledger --test attack_10zf_rounds
run_tests b out-b --test attack_10zf_ledger
COPY="$D/copy-a" CARGO_TARGET_DIR="$D/target-a" LOGS="$D/mut-logs" FROZEN="$FROZEN" \
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 3G bash scripts/hosts/capped.sh 8 bash "$M/mutate-harness.sh" "$M/all-mutants.tsv" 1 25 > "$D/mutants.txt" 2>&1
rustc -O -o "$D/unwind_in_drop" "$M/unwind_in_drop.rs"
for arm in as-shipped not-while-panicking; do
  set +e; nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 1G "$D/unwind_in_drop" $arm > "$D/unwind-$arm.out" 2> "$D/unwind-$arm.err"; rc=$?; set -e
  echo "UNWIND arm=$arm rc=$rc stdout_lines=$(grep -c UNWIND "$D/unwind-$arm.out") abort_line=$(grep -c 'panic in a destructor during cleanup' "$D/unwind-$arm.err")"
done > "$D/unwind.txt"
echo "rerun done: $D"
