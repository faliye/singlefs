#!/usr/bin/env bash
# 用法：rerun.sh <空工作目录> [quick|full]
#   从冻结副本拷一份到 <工作目录>/w，核变异表锚点，跑变异行；quick = GA 两行 + Q-F 意图一行 + 抓不到的一个自写变异（m3，去掉被抛弃根的过滤），要前台跑 3 分钟左右；
#   full = 第 14 批新增 10 行 + 步 5 一行 + 丙 1 四行 + m1..m8 八个自写变异 + 两个探针，要分几次前台跑（每次不超过 240 秒），
#   主 agent 起后台跑。全部经 run-with-memory-cap.sh（一条一条，脚本里不嵌包装）。
# 环境：FROZEN（冻结副本根，默认 /tmp/claude-1000/m3-batch14-code-r1/tree）、REPO（仓根，找包装脚本用）、THREADS（默认 7）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$(realpath "$1")"; mode="${2:-quick}"
frozen="${FROZEN:-/tmp/claude-1000/m3-batch14-code-r1/tree}"
[[ -z "$(ls -A "$work" 2>/dev/null)" ]] || { echo "工作目录不空：$work"; exit 2; }
mkdir -p "$work/w/.cargo" "$work/target"
cp -r "$frozen/Cargo.toml" "$frozen/Cargo.lock" "$frozen/crates" "$work/w/"
cp "${REPO:-/home/fy5090/code/singlefs}/.cargo/config.toml" "$work/w/.cargo/"
export MUT_W="$work/w" CARGO_TARGET_DIR="$work/target" RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-正推腿在自己的副本和编译目录里跑变异，与别的 agent 各跑各的}"
python3 "$here/anchors.py" "$work/w" "第 14 批" "抬 F 回收的槽不扣住" "丙 1" "十二棵树"
rows=(2273 2274 2275)
[[ "$mode" == full ]] && rows=(2273 2274 2275 2276 2277 2278 2279 2280 2281 2282 46 2174 2175 2176 2177)
for row in "${rows[@]}"; do echo "=== row $row"; "$here/run-row.sh" "$row" 2>&1 | grep -E 'label|anchor|FAILED$|test result|PANIC|exit:|restored' | cut -c1-420 || true; done
tests="$(cat "$here/tests.args")"
mutants=(m3_no_abandoned_filter)
[[ "$mode" == full ]] && mutants=(m1_no_held_remainder m2_none_arm_immediate m3_no_abandoned_filter m4_max_not_min m5_no_device_filter m6_off_by_one m7_admission_gets_ga m8_all_devices_only_first)
for mutant in "${mutants[@]}"; do
  echo "##### $mutant"
  # shellcheck disable=SC2086
  "$here/run-row.sh" --adhoc crates/singlefs-core/src/mount.rs "$here/mut/$mutant.orig" "$here/mut/$mutant.repl" --no-fail-fast $tests 2>&1 | grep -E 'anchor|FAILED$|test result|exit:|restored' | cut -c1-200 || true
done
if [[ "$mode" == full ]]; then
  for probe in probe-lone-survivor-live-files probe-mount-continuation-time; do
    echo "##### $probe"; nice -n 19 bash "${REPO:-/home/fy5090/code/singlefs}/research/scripts/run-with-memory-cap.sh" 6G bash "${REPO:-/home/fy5090/code/singlefs}/research/scripts/capped.sh" "${THREADS:-7}" bash "$here/$probe.sh" 2>&1 | cut -c1-300
  done
fi
