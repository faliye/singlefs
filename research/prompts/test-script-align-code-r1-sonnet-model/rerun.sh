#!/usr/bin/env bash
# 复跑：从仓根起。探针 1 编一个只用 std 的小程序（把冻结副本里 common_gpu_adapter/mod.rs 的原样拷贝编进来），经内存包装跑；
# 探针 2 是对冻结副本的 grep，贴出报告里引的零命中与命中。不跑 checker 档测试，不编 checker 档包。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tree="${TREE:-/tmp/claude-1000/test-script-align-code-r1/tree}"
work="${WORK:-/tmp/claude-1000/test-script-align-code-r1-sonnet/probe-build}"
rm -rf "$work"; mkdir -p "$work"
cp "$here/common_gpu_adapter.rs" "$here/gpu_adapter_decision_probe.rs" "$work/"
if ! cmp -s "$here/common_gpu_adapter.rs" "$tree/crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs"; then
  echo "模型里的 common_gpu_adapter.rs 与冻结副本里的 mod.rs 不同"; exit 3
fi
echo "== 探针 1：decide_gpu_test_start 的结局"
nice -n 19 rustc --edition 2021 -O -o "$work/probe" "$work/gpu_adapter_decision_probe.rs" 2>"$work/rustc.err" || { cat "$work/rustc.err"; exit 4; }
# 峰值表里有这条命令的实测峰值就用 auto（1.5 倍），没有（第一次跑）才写明 2G
bash research/scripts/run-with-memory-cap.sh auto "$work/probe" 2>"$work/cap.err"; probe_exit=$?
if [[ $probe_exit -eq 251 ]]; then bash research/scripts/run-with-memory-cap.sh 2G "$work/probe"; probe_exit=$?; else cat "$work/cap.err" >&2; fi
echo "exit=$probe_exit"
crates="$tree/crates"
echo "== 探针 2：PCI 判据在冻结副本里有没有"
echo '$ grep -rnF /sys/bus/pci crates'; grep -rnF '/sys/bus/pci' "$crates"; echo "rc=$? (1 = 零命中)"
echo '$ grep -rnE "0x03|class" common_gpu_adapter/mod.rs gpu_unit_checks.rs'; grep -nE '0x03|class' "$crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs" "$crates/singlefs-checker-tier/src/gpu_unit_checks.rs"; echo "rc=$? (1 = 零命中)"
echo "== 探针 3：没走 first_usable_card_or_report_not_run 的取卡处"
grep -rnE 'usable_gpu_cards\(\)' "$crates/singlefs-checker-tier/tests" "$crates/singlefs-checker-tier/src/crash_judge_dispatch.rs" | grep -v 'common_gpu_adapter'
echo "== 探针 4：取卡落空时的处理"
grep -n 'expect("本机要有一张空闲显存够的 NVIDIA 独显' "$crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs"
grep -n 'let cards = crate::gpu_unit_checks::usable_gpu_cards();' -A1 "$crates/singlefs-checker-tier/src/crash_judge_dispatch.rs"
echo "== 探针 5：NVIDIA_VENDOR_IDENTIFIER 的定义处数"
grep -rn 'const NVIDIA_VENDOR_IDENTIFIER' "$crates"
echo "== 探针 6：SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 读法"
grep -n 'SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO' -B1 -A3 "$crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs" | sed -n 1,8p
echo "== 探针 7：run_history_campaign 装配 sampling 的那一截有没有变异行点它"
grep -c 'weights: \*weights' "$crates/mutations.tsv"
grep -n 'sampling: HistorySampling' "$crates/mutations.tsv" | head -2
