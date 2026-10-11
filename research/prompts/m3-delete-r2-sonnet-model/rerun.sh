#!/usr/bin/env bash
# m3-delete-r2 辩方腿的复跑（副本专用，被攻过零轮）：把冻结副本 crates/ 拷进草稿目录，先打第一轮攻方原型补丁（m3-delete-r1-opus-model/core-prototype.diff），
# 再打本目录的 core-intent-carry.diff（keep = u64::MAX 表示「意图那一次」：extent 与数据单元照抄、只把 inode 1 记录写成 nlink = 0），放进用例，跑 harness 档的这一个测试目标。
# 用法（经内存包装与线程上限起；峰值表里没有这条命令时上限可以写 2G）：
#   bash scripts/hosts/run-with-memory-cap.sh 2G bash scripts/hosts/capped.sh 8 bash research/prompts/m3-delete-r2-sonnet-model/rerun.sh <仓根> <冻结副本根> <草稿目录>
# 冻结副本根 = /tmp/claude-1000/m3-delete-r2/tree（下面要 Cargo.toml、Cargo.lock 与 crates/）。草稿目录必须还不存在；跑完它里面留着副本与编译目录，用完自己删。
# 输出里 `D name=` 行与 run-final-d-lines.txt 逐行比（含 4 个用例）。
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
patch -p1 < "$repository/research/prompts/m3-delete-r1-opus-model/core-prototype.diff"
patch -p1 < "$here/core-intent-carry.diff"
cp "$here/m3_delete_r2_defense_cells.rs" crates/singlefs-harness/tests/
CARGO_TARGET_DIR="$work/target" nice -n 19 cargo test --offline -p singlefs-harness --test m3_delete_r2_defense_cells -- --nocapture --test-threads 1
