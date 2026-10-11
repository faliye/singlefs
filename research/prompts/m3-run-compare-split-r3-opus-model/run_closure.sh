#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r3）：编 closure-probe3（只用 std），在副本上算两份随机历史设点测试文件出发的闭包（H8 那一把）。
# 用法：run_closure.sh <副本根> <草稿目录>（整条经 run-with-memory-cap.sh 起）；输出落 <草稿目录>/outputs/
set -euo pipefail
copy_root="$1"; work="$2"; model="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$work/closure-probe3/src" "$work/outputs"
cp "$model/closure-probe3/Cargo.toml" "$work/closure-probe3/Cargo.toml"
sed "s|COPY_ROOT|$copy_root|" "$model/closure-probe3/src/main.rs.template" > "$work/closure-probe3/src/main.rs"
(cd "$work/closure-probe3" && CARGO_TARGET_DIR="$work/target-closure-probe3" cargo build --release --offline --quiet)
probe="$work/target-closure-probe3/release/closure-probe3"
t=crates/singlefs-checker-tier
{
  for entry in $t/tests/crash_injection_campaign.rs $t/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs $t/src/crash_injection.rs $t/src/crash.rs $t/src/crash_facts.rs $t/src/crash_judge_tables.rs; do
    "$probe" closure "$copy_root" "$entry" | grep -v '^CLOSURE_FILE'
  done
} > "$work/outputs/history-family-closures.out"
"$probe" closure "$copy_root" $t/tests/crash_injection_campaign.rs > "$work/outputs/closure-files-campaign.out"
echo "PROBE3_CLOSURE_DONE"
