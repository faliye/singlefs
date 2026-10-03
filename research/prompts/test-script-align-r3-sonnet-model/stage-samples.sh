#!/usr/bin/env bash
# V3 复现：登记为 samples-light / samples-light-marker 的 13 道门禁阶段，逐道经 stage-selftest.sh --item 喂它的样本，记退出码与判错的样本。
# 样本只读文本与假工具，不起真编译。用法：stage-samples.sh <草稿目录> [阶段文件名…]（不给就 13 道全跑）
S="${1:?用法：stage-samples.sh <草稿目录>}"; mkdir -p "$S"
cd "$(git rev-parse --show-toplevel)" || exit 2
: > "$S/summary.txt"
shift; stages=("$@")
[[ ${#stages[@]} -gt 0 ]] || stages=(checker-tier-layer0-replay.sh checker-independence-and-sync.sh checker-tier-lkmm.sh checker-tier-qemu-device-streams.sh code-source-discipline.sh code-tooling.sh doc-decisions.sh doc-experiments.sh doc-process-records.sh doc-registries.sh doc-text.sh harness-model-differential-and-scenarios.sh harness-test-environment.sh)
for stage in "${stages[@]}"; do
  t0=$SECONDS
  nice -n 19 bash research/scripts/capped.sh 8 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item "$stage" > "$S/$stage.out" 2>&1; rc=$?
  echo "$stage rc=$rc secs=$((SECONDS-t0)) $(tail -n 1 "$S/$stage.out" | cut -c1-120) 判错=$(grep -cE '^\s+✗ ' "$S/$stage.out")" >> "$S/summary.txt"
done
echo DONE >> "$S/summary.txt"
