#!/usr/bin/env bash
# unreadable-at-mount-r2 云端攻方腿：Y2 两张网格在各副本上跑（顺序跑，每条经 capped.sh 16 与 run-with-memory-cap.sh 16G）。
set -uo pipefail
W=/tmp/claude-1000/unreadable-at-mount-r2/opus
REPO=/home/fy5090/code/singlefs
CAP=$REPO/research/scripts/capped.sh; MEM=$REPO/research/scripts/run-with-memory-cap.sh
run() {
  local copy=$1 target=$2
  local started=$(date +%s)
  (cd "$W/$copy" && TMPDIR="$W/tmp" CARGO_TARGET_DIR="$W/target-$copy" SINGLEFS_R2_B_COMPARE=1 nice -n 19 bash "$CAP" 16 bash "$MEM" 16G \
     cargo test --offline --release -p singlefs-harness --test "$target" -- --nocapture > "$W/runs/$copy-$target.log" 2>&1)
  echo "run $copy $target exit=$? seconds=$(( $(date +%s) - started ))"
}
for copy in base a b; do run $copy c393_candidates_grid; done
for copy in base jia jiaeio; do run $copy c331_candidates_grid; done
echo done
