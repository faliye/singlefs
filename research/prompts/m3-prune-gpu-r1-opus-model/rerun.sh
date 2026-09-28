#!/bin/bash
# m3-prune-gpu-r1 云端攻方腿的模型复跑：把仓拷进一个草稿目录，接上 attack.rs 与 E161 装置的两行挂接，编译，逐个跑十个世界。
# 用法：bash rerun.sh <仓根> <草稿目录>   （草稿目录要不存在；跑完输出在 <草稿目录>/out/<世界>.out，与本目录 outputs/ 比）
# 线程上限 4、内存上限 8G；原型里的历史全在内存稀疏盘上录，不建镜像文件；每个世界跑前自己核状态数 ≤ 10⁶。
set -euo pipefail
repository="${1:?仓根}"
scratch="${2:?草稿目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir "$scratch"
rsync -a --exclude target --exclude .git "$repository"/ "$scratch/repo/"
mkdir -p "$scratch/repo/crates/singlefs-checker-tier/e161_attack"
cp "$model/attack.rs" "$scratch/repo/crates/singlefs-checker-tier/e161_attack/attack.rs"
patch "$scratch/repo/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs" < "$model/e161-hook.patch"
cd "$scratch/repo"
CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
  cargo build --release --offline -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split
mkdir "$scratch/out"
for world in t4 torn collide kinds p5 order interior small p6 reuse; do
  nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    "$scratch/target/release/e161_crash_state_dedup_and_time_split" attack "$world" 2>/dev/null \
    | grep -v '^LAYER0' > "$scratch/out/$world.out"
done
for world in t4 torn collide kinds p5 order interior small p6 reuse; do
  if diff <(grep -v 'seconds=' "$scratch/out/$world.out") <(grep -v 'seconds=' "$model/outputs/$world.out") > /dev/null; then
    echo "$world 与存档逐行相同（去掉带 seconds= 的行）"
  else
    echo "$world 与存档不同"
  fi
done
