#!/usr/bin/env bash
# 复跑全部：bash run-all.sh <草稿目录>；草稿目录下要有仓副本 copy/（tar 拷，见报告），各模型在草稿目录里建自己的临时目录。
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; SCRATCH=${1:?草稿目录}
[[ -d "$SCRATCH/copy" ]] || { mkdir -p "$SCRATCH/copy" && tar -C /home/user/singlefs --exclude=./target --exclude=./.git --exclude=./research/target -cf - . | tar -C "$SCRATCH/copy" -xf -; }
echo "# probes"; SCRATCH="$SCRATCH" bash "$HERE/probes.sh"
echo "# run-with-memory-cap --check 4G"; (cd /home/user/singlefs && bash research/scripts/run-with-memory-cap.sh --check 4G; echo "exit=$?") 2>&1
echo "# g1"; bash "$HERE/g1-hash-divergence.sh" "$SCRATCH"
echo "# g3"; bash "$HERE/g3-sample-numbering.sh" "$SCRATCH"
echo "# g5"; bash "$HERE/g5-mutate-markers.sh" "$SCRATCH"
echo "# g6"; bash "$HERE/g6-relabel-crates-anchor.sh" "$SCRATCH"
echo "# cite"; bash "$HERE/cite.sh" < "$HERE/citations.tsv" | grep -c .
