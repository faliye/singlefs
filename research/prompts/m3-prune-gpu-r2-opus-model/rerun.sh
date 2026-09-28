#!/bin/bash
# m3-prune-gpu-r2 云端攻方腿的模型复跑：把仓拷进一个草稿目录，放进 attack.rs（第一轮攻方原型加三行挂 r2）与 r2.rs，
# 打上 E161 装置的两行挂接（第一轮那份）与 crates 里的两处探针（crates-probes.patch：池级 checker 扫描方向三处聚合的输入摘要、层 0 计划哈希与回收谓词的外露口），
# 开 verdict-store 特性编译，逐个跑十五个世界，与 outputs/ 比；再拷一份、把真实现改成先根后记录（transaction-root-before-records.diff，第一轮 F2 调查员那份），
# 编译后只跑 r2-b5-implementation 一个世界，与 outputs/r2-b5-implementation-root-before-records.out 比。
# 用法：bash rerun.sh <仓根> <草稿目录>   （草稿目录要不存在；输出在 <草稿目录>/out/<世界>.out）
# 比法：只剥掉 seconds= 与 build_seconds= 两个耗时字段，整行不扔。
# 线程上限 4、内存上限 8G；历史全在内存稀疏盘上录；每个世界跑前自己核状态数不超过 10⁶。
# 世界 r2-b7、r2-b8、r2-d1 在 /tmp/claude-1000/m3-prune-gpu-r2-opus/kv/ 下建 RocksDB 库，跑完各自删掉。
set -euo pipefail
repository="${1:?仓根}"
scratch="${2:?草稿目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir "$scratch"
rsync -a --exclude target --exclude .git "$repository"/ "$scratch/repo/"
mkdir -p "$scratch/repo/crates/singlefs-checker-tier/e161_attack"
cp "$model/attack.rs" "$model/r2.rs" "$scratch/repo/crates/singlefs-checker-tier/e161_attack/"
patch "$scratch/repo/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs" < "$model/e161-hook.patch"
(cd "$scratch/repo" && patch -p1 < "$model/crates-probes.patch")
rsync -a "$scratch/repo/" "$scratch/repo-root-before-records/"
patch "$scratch/repo-root-before-records/crates/singlefs-core/src/transaction.rs" < "$model/transaction-root-before-records.diff"
for variant in repo repo-root-before-records; do
  (cd "$scratch/$variant" && CARGO_TARGET_DIR="$scratch/target-$variant" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    cargo build --release --offline -p singlefs-checker-tier --features verdict-store --bin e161_crash_state_dedup_and_time_split)
done
mkdir "$scratch/out"
run_world() {
  local variant="$1" world="$2" name="$3"
  # shellcheck disable=SC2086
  (cd "$scratch/$variant" && nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    "$scratch/target-$variant/release/e161_crash_state_dedup_and_time_split" attack $world 2>/dev/null \
    | grep -v '^LAYER0' > "$scratch/out/$name.out")
}
worlds=(r2-b1 r2-b2-versions r2-b2-downstream r2-b3 r2-b4 r2-b4-messages r2-f1-shifted r2-b5 r2-b5-legal-reuse r2-b5-implementation r2-b6 "r2-b7 150" r2-b8 r2-d1 r2-kinds)
names=()
for world in "${worlds[@]}"; do
  name="${world// /_}"
  run_world repo "$world" "$name"
  names+=("$name")
done
run_world repo-root-before-records r2-b5-implementation r2-b5-implementation-root-before-records
names+=(r2-b5-implementation-root-before-records)
strip_seconds() { sed -E 's/ (build_)?seconds=[0-9.]+//g' "$1"; }
for name in "${names[@]}"; do
  if diff <(strip_seconds "$scratch/out/$name.out") <(strip_seconds "$model/outputs/$name.out") > /dev/null; then
    echo "$name 与存档逐行相同（只剥掉 seconds= 与 build_seconds= 字段）"
  else
    echo "$name 与存档不同"
  fi
done
