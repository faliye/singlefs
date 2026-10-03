#!/usr/bin/env bash
# 复跑 test-script-align-r2 辩方腿的全部探针。用法：bash rerun.sh <草稿目录>（已存在拒绝）。
# 只读本仓；喂钩子的探针只喂 JSON、不起被判的命令；三份 --selftest 是登记为 light 的，在本仓路径上跑。
set -uo pipefail
scratch="${1:?用法：rerun.sh <草稿目录>}"
here="$(cd "$(dirname "$0")" && pwd)"
repository="${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"
[[ ! -e "$scratch" ]] || { echo "草稿目录已存在：$scratch"; exit 2; }
mkdir -p "$scratch"
failed=0
run() {   # run <名> <命令…>：输出写 <草稿>/<名>.out，退出码打一行
  local name="$1"; shift
  "$@" > "$scratch/$name.out" 2>&1; local code=$?
  echo "$name rc=$code"; [[ "$code" == 0 ]] || failed=1
}
cd "$repository"
run kb-facts nice -n 19 bash "$here/probe-kb-facts.sh"
run hook-failures nice -n 19 bash "$here/probe-hook-failures.sh" "$scratch/w-hook-failures"
run dispatch-guard nice -n 19 bash "$here/probe-dispatch-guard.sh" "$scratch/w-dispatch-guard"
run guard-spot-checks nice -n 19 bash "$here/probe-guard-spot-checks.sh" "$scratch/w-guard-spot-checks"
run flat-draft nice -n 19 bash "$here/probe-flat-draft.sh" "$scratch/w-flat-draft"
start="$(date +%s.%N)"
run lib-heavy-tests-selftest nice -n 19 bash research/scripts/capped.sh 8 python3 .claude/hooks/lib_heavy_tests.py --selftest
run heavy-test-guard-selftest nice -n 19 bash research/scripts/capped.sh 8 bash .claude/hooks/heavy-test-guard.sh --selftest
echo "两份钩子自证合计秒数 $(echo "$(date +%s.%N) - $start" | bc)"
run crash-amp-selftest nice -n 19 bash research/scripts/capped.sh 8 bash research/scripts/crash-amplification.sh --selftest
exit "$failed"
