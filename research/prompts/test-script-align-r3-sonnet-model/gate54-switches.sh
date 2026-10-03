#!/usr/bin/env bash
# V3 复现 + L3 现场：54 号门禁文件头列的弄坏开关，逐个打开去喂它的样本（登记为 samples-light-marker 的那一道，只用假 cargo，不起真编译），
# 记下判错的样本。这正是「复跑方」的手工版——今天没有任何一格门禁替人做这件事（L3）。
# 用法：gate54-switches.sh <草稿目录>；每个开关约 5 秒
S="${1:?用法：gate54-switches.sh <草稿目录>}"; mkdir -p "$S"
cd "$(git rev-parse --show-toplevel)" || exit 2
: > "$S/summary.txt"
run() { # <变量=值> <文件头写的该判错的样本>
  local assignment=$1 expected=$2
  env "$assignment" nice -n 19 bash research/scripts/capped.sh 8 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item checker-tier-layer0-replay.sh > "$S/${assignment//[^A-Za-z0-9=-]/_}.out" 2>&1; local rc=$?
  local wrong; wrong=$(grep -E '^\s+✗ checker-tier-layer0-replay.sh' "$S/${assignment//[^A-Za-z0-9=-]/_}.out" | sed -E 's/^\s+✗ checker-tier-layer0-replay.sh\s+//; s/ .*//' | sort -u | paste -sd, -)
  echo "$assignment rc=$rc 判错的样本=[${wrong:-无}] 文件头写的=[$expected]" >> "$S/summary.txt"
}
env nice -n 19 bash research/scripts/capped.sh 8 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item checker-tier-layer0-replay.sh > "$S/baseline.out" 2>&1; echo "不开开关 rc=$?" >> "$S/summary.txt"
run GATE_LAYER0_BREAK=no-findings-file 'red 或 green（没点名）'
run GATE_LAYER0_BREAK=ignore-unfinished 'red 或 green（没点名）'
run GATE_LAYER0_BREAK=ignore-red-states 'red 或 green（没点名）'
run GATE_LAYER0_BREAK=no-truncation 'red 或 green（没点名）'
run GATE_LAYER0_BREAK=static-check-skipped static-check-red
run CRASH_CASE_CHECK_BREAK=rows-ignored static-check-red
run GATE_LAYER0_BREAK=no-feature-pass quick-features-green
run GATE_LAYER0_BREAK=features-not-from-source quick-features-green
run GATE_LAYER0_BREAK=ignore-feature-pass-exit quick-features-red
run GATE_LAYER0_BREAK=ignore-zero-feature-files quick-features-zero
run GATE_LAYER0_BREAK=ignore-gpu-not-run quick-features-gpu-not-run
run GATE_LAYER0_BREAK=sample-tools-unchecked sample-cargo-not-executable
echo DONE >> "$S/summary.txt"
