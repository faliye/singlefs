#!/usr/bin/env bash
# test-script-align-code-r2 攻方腿（Opus）的复跑：在一个空的工作目录里从冻结副本拷一份仓（tree），打两处只给原型用的补丁
# （patch_tree.py：看判器输入原文的口子；patch_tree_fix.py：攻方腿提的改法，环境变量 R2_FIX_BOUND_BY_THE_BASE_ROOT 开，默认关、与冻结副本同），
# 照本目录的原型源码建 proto（依赖 tree，特性 verdict-store,gpu），编、跑，原样输出落进 <工作目录>/runs/。
# 用法：bash rerun.sh <空的工作目录> [冻结副本，默认 /tmp/claude-1000/test-script-align-code-r2/tree]
# 不跑 checker 档包的任何测试；跑的都是原型二进制，一次一条，经 scripts/hosts/run-with-memory-cap.sh；线程上限 12（capped.sh 12）。
# 展开 16 的四批合起来约 300 万个状态判两遍以内，本腿实跑挂钟见报告；只复跑展开 6 那几样设 R2_RERUN_QUICK=1。
set -u
model="$(cd "$(dirname "$0")" && pwd)"
repository="$(cd "$model/../../.." && pwd)"
work="${1:?要给工作目录}"; frozen="${2:-/tmp/claude-1000/test-script-align-code-r2/tree}"
cap="$repository/scripts/hosts/run-with-memory-cap.sh"; capped="$repository/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑原型：编译与展开 16 的那几趟峰值表里没有"
mkdir -p "$work/runs" "$work/proto/.cargo"
export R2_SCRATCH="$work/runs" CARGO="$(command -v cargo)" R2_THREADS=12
rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/"
python3 "$model/patch_tree.py" "$work/tree" && python3 "$model/patch_tree_fix.py" "$work/tree" || { echo "补丁打不上"; exit 1; }
cp -r "$model/proto/src" "$model/proto/Cargo.toml" "$work/proto/"
mkdir -p "$work/proto/src/common"
python3 "$model/make_common.py" "$work/tree" "$work/proto/src/common/mod.rs"
sed 's/use super::common::/use crate::common::/' "$work/tree/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs" > "$work/proto/src/common_crash_points/mod.rs"
cp "$frozen/.cargo/config.toml" "$work/proto/.cargo/"; cp "$frozen/Cargo.lock" "$work/proto/"
(cd "$work/proto" && nice -n 19 bash "$cap" 12G bash "$capped" 12 cargo build --release --offline --bins) > "$work/runs/build.log" 2>&1 || { echo "proto 编不过"; exit 1; }
run() { local name="$1"; shift; (cd "$work/proto" && nice -n 19 bash "$cap" 8G bash "$capped" 12 env "$@") > "$work/runs/$name.out" 2> "$work/runs/$name.err"; echo "$name exit=$?"; }
run j1_flows-6 R2_EXPAND_UP_TO=6 ./target/release/j1_flows
run r1_tests-as-merged ./target/release/r1_tests
run r1_tests-fix-base-root-bound R2_FIX_BOUND_BY_THE_BASE_ROOT=1 ./target/release/r1_tests
run m3_twin ./target/release/m3_twin
run m3_candidates ./target/release/m3_candidates
run j1_diff-6 R2_EXPAND_UP_TO=6 ./target/release/j1_diff
run g1_outcomes-live R2_LIVE=1 ./target/release/g1_outcomes
run g1_outcomes-icd-missing R2_LIVE=1 VK_ICD_FILENAMES=/nonexistent/nvidia_icd.json VK_DRIVER_FILES=/nonexistent/nvidia_icd.json ./target/release/g1_outcomes
if [[ "${R2_RERUN_QUICK:-0}" != 1 ]]; then
  run j1_diff-16-rollback R2_EXPAND_UP_TO=16 R2_ONLY=rollback ./target/release/j1_diff
  run j1_diff-16-A R2_EXPAND_UP_TO=16 R2_ONLY=later_unit,later_journal,later_root_payload,later_sysconfig ./target/release/j1_diff
  run j1_diff-16-B R2_EXPAND_UP_TO=16 R2_ONLY=last_root_txg_jump ./target/release/j1_diff
  run j1_diff-16-C R2_EXPAND_UP_TO=16 R2_ONLY=versions_after_d,judged_root_moved ./target/release/j1_diff
fi
awk -F'\t' 'NR==1585||NR==1587||NR==1590||NR==1595||NR==1609||NR>=1674{print NR"\t"$0}' "$frozen/crates/mutations.tsv" > "$work/runs/inscope.tsv"
K3_TREE="$frozen" python3 "$model/k3check.py" "$work/runs/inscope.tsv" > "$work/runs/k3check.out"
grep -h -E '^(J1FLOW|J1PAIR|J1SPLIT|J1DIFF |J1DIFF_TOTAL|R1TEST|M3POINT|M3CAND_TOTAL|G1)' "$work"/runs/*.out
grep -c '^ROW' "$work/runs/k3check.out"
