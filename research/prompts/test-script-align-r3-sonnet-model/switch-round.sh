#!/usr/bin/env bash
# V3 复现：mutate.sh 自证的 16 个弄坏开关逐个顺序打开，各记退出码与「  ✗ 自检：」行数（与 /tmp/claude-1000/tooling-writer-mutate-parallel-reason/switch-round.sh 同法）。
# 用法：switch-round.sh <草稿目录> [只重跑的开关名…]；约 12 分钟
S="${1:?用法：switch-round.sh <草稿目录> [开关名…]}"; shift
cd "$(git rev-parse --show-toplevel)" || exit 2
mkdir -p "$S"; : > "$S/summary.txt"
switches=("$@"); [[ ${#switches[@]} -gt 0 ]] || switches=(memoryascaught memorynotfail droprow countblind sharedcopy nofeatures outertimeout swallowwrapper queueasinvalid squeezedascrashed honorbreakoutsideselftest noparallelreason overridecallerreason baselinequeueasred exportparallelreason nocap)
for switch in "${switches[@]}"; do
  if [[ "$switch" == nocap ]]; then variable=RUN_WITH_MEMORY_CAP_BREAK; else variable=MUTATE_BREAK; fi
  if env "$variable=$switch" nice -n 19 bash research/scripts/capped.sh 8 bash research/scripts/mutate.sh --selftest >"$S/$switch.out" 2>&1; then rc=0; else rc=$?; fi
  echo "$switch rc=$rc failures=$(grep -c '^  ✗ 自检：' "$S/$switch.out")" >>"$S/summary.txt"
done
echo DONE >>"$S/summary.txt"
