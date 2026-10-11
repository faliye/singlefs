#!/usr/bin/env bash
# m3-delete-r3 辩方腿原型的复跑（副本专用，被攻过零轮）：把冻结副本的 crates/ 拷进草稿目录，打 m3-delete-r2 攻方的 core-prototype-r2.diff
# 与辩方的 core-intent-carry.diff（意图那次只改 inode 记录），把攻方的原型用例文件与本目录的追加段拼成一个测试目标，跑 harness 档的
# g1_ 四个用例与 g2_ 一个用例，再跑 ring_window_cells.py 对拍。
# 用法（经内存包装与线程上限起；上限给 6G，峰值表里没有这条命令时要 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON）：
#   bash scripts/hosts/run-with-memory-cap.sh 6G bash scripts/hosts/capped.sh 5 bash research/prompts/m3-delete-r3-sonnet-model/rerun.sh <仓根> <冻结副本根> <草稿目录>
# 冻结副本根 = /tmp/claude-1000/m3-delete-r2/tree（清单 research/prompts/m3-delete-r3-snapshot/crates-sha256.txt）。草稿目录必须还不存在，用完自己删。
# 输出末尾：「R name= 行相同」「cells.out 相同」两行；R 行次序固定（g1_ 四个用例按字母序在前，g2_ 在后）。
set -euo pipefail
repository=$1
frozen=$2
work=$3
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
mkdir "$work"
mkdir "$work/tree"
cp "$frozen/Cargo.toml" "$frozen/Cargo.lock" "$work/tree/"
rsync -a --exclude target "$frozen/crates/" "$work/tree/crates/"
cd "$work/tree"
patch -p1 < "$repository/research/prompts/m3-delete-r2-opus-model/core-prototype-r2.diff"
patch -p1 < "$repository/research/prompts/m3-delete-r2-sonnet-model/core-intent-carry.diff"
cat "$repository/research/prompts/m3-delete-r2-opus-model/m3_delete_r2_attack_prototype.rs" "$here/m3_delete_r3_defense_cells_part.rs" \
  > crates/singlefs-harness/tests/m3_delete_r3_defense_cells.rs
export CARGO_TARGET_DIR="$work/target"
nice -n 19 cargo test --offline -p singlefs-harness --test m3_delete_r3_defense_cells g1_ -- --nocapture --test-threads 1 > "$work/g1.log" 2>&1
nice -n 19 cargo test --offline -p singlefs-harness --test m3_delete_r3_defense_cells g2_ -- --nocapture --test-threads 1 > "$work/g2.log" 2>&1
grep -h -o 'R name=.*' "$work/g1.log" "$work/g2.log" > "$work/r-lines.txt"
if diff -q "$work/r-lines.txt" "$here/run-final-r-lines.txt" > /dev/null; then echo "R name= 行相同"; else echo "R name= 行不同" >&2; exit 1; fi
cp "$work/r-lines.txt" "$work/run-final-r-lines.txt"
cp "$here/ring_window_cells.py" "$work/"
python3 "$work/ring_window_cells.py" > "$work/cells.out"
if diff -q "$work/cells.out" "$here/cells.out" > /dev/null; then echo "cells.out 相同"; else echo "cells.out 不同" >&2; exit 1; fi
