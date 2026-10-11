#!/usr/bin/env bash
# m3-batch27-code-r1 正推腿的复跑：在冻结副本的拷贝上（一）算 J1 的纸面链模型（二）核变异行的锚点命中数与参数筛
# （三）harness 档：对照跑一遍，再把 mutations.tsv 里点名 harness 测试的三行（2934、2935、2937）逐行改坏、要求点名的测试红。
# GPU 那几行（2938–2942）要 checker 档包与 GPU 卡，子 agent 不跑，这里只核锚点与参数筛。
#
# 环境变量：
#   SINGLEFS_REPO  仓路径（取 scripts/run-with-memory-cap.sh 与 capped.sh）；必须给
#   M3B27_TREE     冻结副本，默认 /tmp/claude-1000/m3-batch27-code-r1/tree
#   M3B27_OUT      输出目录，默认 mktemp 新建；已存在且非空就停
# 编译 6G、跑 4G，线程上限 15；编译目录在输出目录里（自己的 target）。
# 输出：$M3B27_OUT/results.txt（排序后）与 expected.txt 比，对得上打印 COMPARE-OK。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tree="${M3B27_TREE:-/tmp/claude-1000/m3-batch27-code-r1/tree}"
out="${M3B27_OUT:-$(mktemp -d /tmp/claude-1000/m3-batch27-code-r1-sonnet-rerun.XXXXXX)}"
mkdir -p "$out"
if [ -n "$(ls -A "$out")" ]; then echo "输出目录 $out 非空" >&2; exit 2; fi
cap="$SINGLEFS_REPO/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_REPO/scripts/capped.sh"
export CARGO_TARGET_DIR="$out/target"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示规定编译 6G、跑 4G，显式写上限不用 auto"
results="$out/results.txt"
: > "$results"

python3 "$here/chain_model.py" | sed 's/^/chain_model: /' >> "$results"
python3 "$here/apply_row.py" --check "$tree" 2934 2935 2937 2938 2939 2940 2941 2942 | sed 's/^/row_check: /' >> "$results"

run_harness() {  # $1 = 标签, $2 = 树；点名的 harness 测试在这个树上 ok 还是 FAILED
    local label="$1" dir="$2" log="$out/$1.log"
    ( cd "$dir" && nice -n 19 bash "$cap" 6G bash "$capped" 15 cargo test --offline --locked -p singlefs-harness --test checker_known_bad_images --no-run ) > "$log" 2>&1
    set +e
    ( cd "$dir" && nice -n 19 bash "$cap" 4G bash "$capped" 15 cargo test --offline --locked -p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back ) >> "$log" 2>&1
    local code=$?
    set -e
    echo "harness_run: $label exit_ok=$((code == 0))" >> "$results"
    grep -E '^test (an_inode_tree_entry_of_type_zero_with_an_identity|an_inode_tree_entry_with_an_unregistered_type).* \.\.\. (ok|FAILED)$' "$log" \
        | sed -E "s/^test /harness_run: $label test /" >> "$results" || true
}

cp -r "$tree" "$out/base"
run_harness baseline "$out/base"
for row in 2934 2935 2937; do
    cp -r "$tree" "$out/row$row"
    python3 "$here/apply_row.py" "$out/row$row" "$row" | sed 's/^/mutation: /' >> "$results"
    run_harness "mutated-row-$row" "$out/row$row"
done

sort "$results" > "$out/results.sorted.txt"
sort "$here/expected.txt" > "$out/expected.sorted.txt"
if diff "$out/expected.sorted.txt" "$out/results.sorted.txt" > "$out/compare.diff"; then echo "COMPARE-OK ($(wc -l < "$out/results.sorted.txt") 行)"; else echo "COMPARE-DIFF"; cat "$out/compare.diff"; exit 1; fi
