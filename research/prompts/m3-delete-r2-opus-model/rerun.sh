#!/usr/bin/env bash
# m3-delete-r2 攻方腿原型的复跑（副本专用，被攻过零轮）：把冻结副本的 crates/ 拷进草稿目录，打上 core-prototype-r2.diff
# （m3-delete-r1 攻方原型那份 diff 加这一轮的账房钩子与两个准入开关），放进用例，跑 harness 档的这一个测试目标。
# 用法（经内存包装与线程上限起）：
#   bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 8 bash research/prompts/m3-delete-r2-opus-model/rerun.sh <冻结副本根> <草稿目录>
# 冻结副本根是 /tmp/claude-1000/m3-delete-r2/tree（清单 research/prompts/m3-delete-r2-snapshot/crates-sha256.txt）。
# 草稿目录必须还不存在；跑完它里面留着仓副本与编译目录，用完自己删。输出里的 `E name=` 行与 run-final-e-lines.txt 逐行比。
set -euo pipefail
repository=$1
work=$2
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
mkdir "$work"
mkdir "$work/tree"
cp "$repository/Cargo.toml" "$repository/Cargo.lock" "$work/tree/"
rsync -a --exclude target "$repository/crates/" "$work/tree/crates/"
cd "$work/tree"
patch -p1 < "$here/core-prototype-r2.diff"
cp "$here/m3_delete_r2_attack_prototype.rs" crates/singlefs-harness/tests/
CARGO_TARGET_DIR="$work/target" nice -n 19 cargo test --offline -p singlefs-harness --test m3_delete_r2_attack_prototype -- --nocapture --test-threads 1
