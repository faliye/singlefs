#!/usr/bin/env bash
# unreadable-at-mount-r1 云端攻方腿复跑：从冻结副本建六份副本（今天、甲、乙、丙、(a)、(d)），打上本目录的 diff，拷进原型用例与调查员用例，
# 逐份编译、跑，最后对齐汇总。用法：bash rerun.sh <冻结副本根（含 crates/）> <工作目录（新建）>
# 跑编译出来的代码都经 run-with-memory-cap.sh 16G 与 capped.sh 16。
set -euo pipefail
FROZEN=${1:?冻结副本根}; WORK=${2:?工作目录}
MODEL=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
CAP=$REPO/research/scripts/capped.sh; MEM=$REPO/research/scripts/run-with-memory-cap.sh
INVESTIGATOR=$REPO/research/prompts/closeout-recheck-2026-09-28
mkdir -p "$WORK/runs" "$WORK/tmp"
for copy in base jia yi bing a d; do
  mkdir -p "$WORK/$copy/.cargo"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/$copy/"; cp "$REPO/.cargo/config.toml" "$WORK/$copy/.cargo/"
  rsync -a --exclude target "$FROZEN/crates" "$WORK/$copy/"
  (cd "$WORK/$copy" && patch -p1 --quiet < "$MODEL/common-mod.rs.diff")
  if [ "$copy" != base ]; then (cd "$WORK/$copy" && patch -p1 --quiet < "$MODEL/candidate-$copy.diff"); fi
  cp "$MODEL/c331_candidates_grid.rs" "$MODEL/c393_candidates_grid.rs" \
     "$INVESTIGATOR/c331_newer_instance_roots_unreadable_then_readable_again.rs" \
     "$INVESTIGATOR/c393_abandoned_root_account_unreadable_at_remount.rs" "$WORK/$copy/crates/singlefs-harness/tests/"
done
run() {
  local copy=$1 target=$2
  (cd "$WORK/$copy" && TMPDIR="$WORK/tmp" CARGO_TARGET_DIR="$WORK/target-$copy" nice -n 19 bash "$CAP" 16 bash "$MEM" 16G \
     cargo test --offline --release -p singlefs-harness --test "$target" -- --nocapture > "$WORK/runs/$copy-$target.log" 2>&1) || echo "run $copy $target exit=$?"
}
for copy in base jia yi bing; do run $copy c331_candidates_grid; run $copy c331_newer_instance_roots_unreadable_then_readable_again; done
for copy in base a d; do run $copy c393_candidates_grid; run $copy c393_abandoned_root_account_unreadable_at_remount; done
for copy in base a d; do cp "$WORK/runs/$copy-c393_candidates_grid.log" "$WORK/runs/$copy-c393_candidates_grid-v2.log"; done
python3 "$MODEL/summarize_c331.py" "$WORK/runs" base jia yi bing > "$WORK/c331-summary.txt"
python3 "$MODEL/summarize_c393.py" "$WORK/runs" -v2 base a d > "$WORK/c393-summary.txt"
echo "done: $WORK/c331-summary.txt $WORK/c393-summary.txt"
