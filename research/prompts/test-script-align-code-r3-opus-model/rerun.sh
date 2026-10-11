#!/usr/bin/env bash
# test-script-align-code-r3 攻方腿（Opus）的复跑：在一个空的工作目录里从冻结副本拷一份仓（tree），打 r2 攻方的看判器输入原文的口子
# （research/prompts/test-script-align-code-r2-opus-model/patch_tree.py）与本腿的 patch_mutations.py（变异表第 1587、1731、1732、1733 行
# 各一个环境变量开关，默认关、与冻结副本同义），照本目录的原型源码建 proto（依赖 tree，特性 verdict-store,gpu），编、跑，原样输出落进 <工作目录>/runs/。
# 再打 patch_fix.py（本腿提的改法 B0R，环境变量 R3_FIX_B0_RECOVER_THE_BASE 开，默认关）。
# 另按变异表第 1741、1729、1730 行造只改原文的仓副本（make_text_variants.py），给依赖判法与 C604 源码扫描两个重算用（判的是原文，不编译）。
# 用法：bash rerun.sh <空的工作目录> [冻结副本，默认 /tmp/claude-1000/test-script-align-code-r3/tree]
# 不跑 checker 档包的任何测试；跑的都是原型二进制，一次一条，经 scripts/run-with-memory-cap.sh；线程上限 12（capped.sh 12）。
# 展开 16 那一趟（j1_diff-16）几分钟；只复跑别的设 R3_RERUN_QUICK=1。
set -u
model="$(cd "$(dirname "$0")" && pwd)"
repository="$(cd "$model/../../.." && pwd)"
work="${1:?要给工作目录}"; frozen="${2:-/tmp/claude-1000/test-script-align-code-r3/tree}"
cap="$repository/scripts/run-with-memory-cap.sh"; capped="$repository/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑原型：编译与原型二进制峰值表里没有"
mkdir -p "$work/runs" "$work/proto/.cargo" "$work/variants"
export R3_SCRATCH="$work/runs" CARGO="$(command -v cargo)" R2_THREADS=12
rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/"
python3 "$repository/research/prompts/test-script-align-code-r2-opus-model/patch_tree.py" "$work/tree" && python3 "$model/patch_mutations.py" "$work/tree" && python3 "$model/patch_fix.py" "$work/tree" || { echo "补丁打不上"; exit 1; }
cp -r "$model/proto/src" "$model/proto/Cargo.toml" "$work/proto/"
mkdir -p "$work/proto/src/common" "$work/proto/src/common_crash_points"
python3 "$model/make_common.py" "$work/tree" "$work/proto/src/common/mod.rs"
sed 's/use super::common::/use crate::common::/' "$work/tree/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs" > "$work/proto/src/common_crash_points/mod.rs"
cp "$frozen/.cargo/config.toml" "$work/proto/.cargo/"; cp "$frozen/Cargo.lock" "$work/proto/"
python3 "$model/make_text_variants.py" "$frozen" "$work/variants" 1741 1729 1730 > "$work/runs/variants.log" || { echo "变异原文造不出"; exit 1; }
(cd "$work/proto" && nice -n 19 bash "$cap" 12G bash "$capped" 12 env CARGO_TARGET_DIR="$work/proto/target" cargo build --release --offline --bins) > "$work/runs/build.log" 2>&1 || { echo "proto 编不过"; exit 1; }
run() { local name="$1"; shift; (cd "$work/proto" && nice -n 19 bash "$cap" 8G bash "$capped" 12 env "$@") > "$work/runs/$name.out" 2> "$work/runs/$name.err"; echo "$name exit=$?"; }
run b0_crashed_base-6 ./target/release/b0_crashed_base
run b0_crashed_base-6-mut1731 R3_MUT_1731=1 ./target/release/b0_crashed_base
run b0_crashed_base-6-fixB0R R3_FIX_B0_RECOVER_THE_BASE=1 ./target/release/b0_crashed_base
run named_tests-baseline ./target/release/named_tests
run named_tests-fixB0R R3_FIX_B0_RECOVER_THE_BASE=1 ./target/release/named_tests
for m in 1587 1731 1732 1733; do run "named_tests-mut$m" "R3_MUT_$m=1" ./target/release/named_tests; done
run dep_check ./target/release/dep_check "$frozen" "$work/variants/row1741"
run scan_check ./target/release/scan_check "$frozen/crates/singlefs-checker-tier" "$work/variants/row1729/crates/singlefs-checker-tier" "$work/variants/row1730/crates/singlefs-checker-tier"
run l2_matrix ./target/release/l2_matrix
run j1_diff-6 R2_EXPAND_UP_TO=6 ./target/release/j1_diff
if [[ "${R3_RERUN_QUICK:-0}" != 1 ]]; then
  run j1_diff-16-rollback R2_EXPAND_UP_TO=16 R2_ONLY=rollback,two_flows ./target/release/j1_diff
fi
K3_TREE="$frozen" python3 "$repository/research/prompts/test-script-align-code-r2-opus-model/k3check.py" "$model/outputs/inscope.tsv" > "$work/runs/k3check.out"
# harness 档那五行变异的证红（outputs/prove-red.out）另起，在一份冻结副本的拷贝上：
#   bash scripts/capped.sh 12 bash scripts/prove-red.sh --copy <冻结副本的拷贝> --memory 16G singlefs-harness <outputs/prove-red.out 里的五个变异名>
grep -h -E '^(B0CB |B0CB_TOTAL|NTEND|DEP_TOTAL|SCAN_VERDICT|L2ZERO|J1DIFF |J1SPLIT|J1DIFF_TOTAL)' "$work"/runs/*.out
