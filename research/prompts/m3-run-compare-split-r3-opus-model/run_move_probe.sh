#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r3）：簇 0「把枚举挪进叶子」真搬家一次，编得过（cargo check），再量
# ① 叶子出发的闭包里有没有 CPU 判器入口 crash.rs、WGSL（第三节第 3 条的检查）；② 两把判器闭包的按项多重集（第 6 条的验收）。
# 用法：run_move_probe.sh <副本根> <草稿目录>（整条经 run-with-memory-cap.sh 起；先跑过 run_closure.sh 编好 closure-probe3）
set -euo pipefail
copy_root="$1"; work="$2"; model="$(cd "$(dirname "$0")" && pwd)"
probe="$work/target-closure-probe3/release/closure-probe3"
trees="$work/trees"; rm -rf "$trees"; mkdir -p "$trees"
for tree in base move move2 swap; do
  mkdir -p "$trees/$tree"; cp "$copy_root/Cargo.toml" "$copy_root/Cargo.lock" "$trees/$tree/"; cp -a "$copy_root/crates" "$trees/$tree/"
done
python3 -I "$model/move_enumeration.py" "$trees/move"
python3 -I "$model/move_enumeration.py" "$trees/move2" --facts-import
# swap：第二轮攻方 R5 那一处对调（记录核对器的区间相交判定，crash.rs:231），语义变了，验收该红
python3 -I - "$trees/swap/crates/singlefs-checker-tier/src/crash.rs" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = "&& later_start < sector_offset + SECTOR_BYTES"; assert s.count(old) == 1
open(p, "w").write(s.replace(old, "&& sector_offset < later_start + SECTOR_BYTES"))
PY
for tree in move move2; do
  (cd "$trees/$tree" && CARGO_TARGET_DIR="$work/target-move" cargo check --offline --quiet -p singlefs-checker-tier --lib --tests 2> "$work/outputs/check-$tree.log") && echo "CHECK tree=$tree compiles=yes" || echo "CHECK tree=$tree compiles=no"
done > "$work/outputs/move-compile.out"
t=crates/singlefs-checker-tier/src
{
  for tree in base move move2; do
    for entry in $t/crash_enumeration.rs $t/crash_facts.rs $t/crash.rs; do
      [ -f "$trees/$tree/$entry" ] || continue
      echo "tree=$tree $("$probe" closure "$trees/$tree" "$entry" | head -1)"
    done
  done
} > "$work/outputs/leaf-closures.out" 2> "$work/outputs/leaf-closures.err"
{
  for mode in items items-loose; do
    for pair in "base move2" "base swap"; do
      set -- $pair
      for entry in $t/crash.rs $t/crash_judge_gpu.rs; do
        "$probe" $mode "$trees/$1" "$trees/$2" "$entry" > "$work/outputs/$mode-$1-$2-$(basename "$entry" .rs).out"
        echo "pair=$1-$2 $(tail -1 "$work/outputs/$mode-$1-$2-$(basename "$entry" .rs).out")"
      done
    done
  done
} > "$work/outputs/cluster0-item-acceptance.out"
echo "PROBE3_MOVE_DONE"
