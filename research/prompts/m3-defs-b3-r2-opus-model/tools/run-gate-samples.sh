#!/usr/bin/env bash
# 要主 agent 跑（门禁格子 agent 不起）：把 gate-samples/ 下几份小树逐份拷进 <工作目录>，有 setup.sh 的在拷贝里跑它，再用 <根> 里的门禁阶段只跑点名的那一格判它。
# 用法：bash run-gate-samples.sh <根：冻结副本或仓> <新的工作目录>
#   每份样本打一行 name=gate-sample stage=<阶段> cell=<格> sample=<样本> rc=<退出码>，后面接那一格的 ✓ / ✗ / ! 行与点名行（输出里的假主机名是样本编的）。
# 推的结局写在各目录的 README.expect.txt。
set -uo pipefail
ROOT="$(cd "$1" && pwd)"; WORK="$2"
SAMPLES="$(cd "$(dirname "${BASH_SOURCE[0]}")/../gate-samples" && pwd)"
mkdir -p "$WORK"
run_one() {   # run_one <阶段文件名> <格> <样本目录>
  local stage="$1" cell="$2" sample="$3" copy rc
  copy="$WORK/$(basename "$sample")"
  rm -rf "$copy"; cp -a "$sample" "$copy"
  if [[ -f "$copy/setup.sh" ]]; then (cd "$copy" && bash setup.sh) || { echo "name=gate-sample sample=$(basename "$sample") setup-failed"; return; }; fi
  bash "$ROOT/.claude/gate.d/$stage" --check "$cell" "$copy" > "$copy.log" 2>&1; rc=$?
  echo "name=gate-sample stage=$stage cell=$cell sample=$(basename "$sample") rc=$rc"
  grep -E '✓|✗|^ *!|键名|\.rs:|\.py:|\.sh:|第 [0-9]+ 行' "$copy.log" | head -20
}
for sample in "$SAMPLES"/doc-text/*/; do run_one doc-text.sh private-values "${sample%/}"; done
for sample in "$SAMPLES"/doc-process-records/*/; do run_one doc-process-records.sh abandoned-rounds "${sample%/}"; done
run_one code-tooling.sh heavy-thread-defaults "$SAMPLES/code-tooling/r2-thread-gaps"
