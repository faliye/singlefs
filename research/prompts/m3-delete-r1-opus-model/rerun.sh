#!/usr/bin/env bash
# m3-delete-r1 攻方腿原型的复跑（副本专用，被攻过零轮）：把仓里 crates/ 拷进草稿目录，打上 core-prototype.diff、放进用例，跑 harness 档的这一个测试目标。
# 用法（经内存包装与线程上限起）：
#   bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 bash research/prompts/m3-delete-r1-opus-model/rerun.sh <仓根> <草稿目录>
# 草稿目录必须还不存在；跑完它里面留着仓副本与编译目录，用完自己删。输出里 `E name=` 行与 run-final-2-e-lines.txt 逐行比（71 行；run-final-e-lines.txt 是前一条腿 7 个用例那一次的 64 行，是它的子集）。
set -euo pipefail
repository=$1
work=$2
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
mkdir "$work"
mkdir "$work/tree"
cp "$repository/Cargo.toml" "$repository/Cargo.lock" "$work/tree/"
rsync -a --exclude target "$repository/crates/" "$work/tree/crates/"
cd "$work/tree"
patch -p1 < "$here/core-prototype.diff"
cp "$here/m3_delete_r1_attack_prototype.rs" crates/singlefs-harness/tests/
CARGO_TARGET_DIR="$work/target" nice -n 19 cargo test -p singlefs-harness --test m3_delete_r1_attack_prototype -- --nocapture --test-threads 1
