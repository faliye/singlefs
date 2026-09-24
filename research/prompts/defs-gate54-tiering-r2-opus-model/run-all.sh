#!/usr/bin/env bash
# 复跑全部场景：bash run-all.sh [草稿根目录]。只在草稿根目录下建合成仓，真仓一处不碰。
# 输出写进本目录 outputs/。整趟在这台机器上约 20 分钟（假 cargo，nice 19，与别的 cargo 抢核）。
set -uo pipefail
MODEL="$(cd "$(dirname "$0")" && pwd)"
export RUNS_BASE="${1:-/tmp/claude-1000/defs54-r2-attack/runs}"
export RUNS="$RUNS_BASE/main"
rm -rf "$RUNS_BASE"; mkdir -p "$RUNS" "$MODEL/outputs"
( cd "$MODEL" && sha256sum -c --quiet sha256sums.txt ) || { echo "模型文件与 sha256sums.txt 不符"; exit 2; }
for scenario in h-a-late-stage-54 h-b-revert-stale-cell h-b2-forced-recheck h-c-foreign-run-deletes v1-outpath-scenes; do
  nice -n 19 bash "$MODEL/$scenario.sh" > "$MODEL/outputs/$scenario.out" 2>&1
  echo "$scenario 退出码 $?：$MODEL/outputs/$scenario.out"
done
nice -n 19 bash "$MODEL/fix-arms.sh"
