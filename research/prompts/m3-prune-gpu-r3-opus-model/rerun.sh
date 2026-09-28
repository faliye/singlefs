#!/bin/bash
# m3-prune-gpu-r3 云端攻方腿的模型复跑。用法：bash rerun.sh <仓根> <一个不存在的草稿目录>
# 1. 仓拷进草稿目录，放进第一、二轮攻方原型（attack.rs、r2.rs 原样）与这一轮的 r3.rs，打 E161 挂接、crates 探针与 r3 挂接三份补丁，
#    开 verdict-store 特性编 E161 装置，跑五个 r3 世界；
# 2. 再拷一份，variants.sh 在它上面逐个换一处真代码（crates/mutations.tsv 的三行、第一轮 F2 调查员的先根后记录、harness 的 F1 候选、
#    只挪一行、mkfs 的水位），增量重编、跑点名的世界，compare_dumps.py 比「身份 + 属性」下漏几个；
# 3. comment_flip.sh 在同一份副本上跑一条 harness 档测试两次（原样 / 只改一行注释）；
# 4. 不编译的几件：流水线模型、A1 两份扫描、A9 静态扫描、A7 类型检查（rustc 只出元数据）、A8 指纹；
# 5. 逐个与 outputs/ 比，只剥掉 seconds= 与耗时括注两类字段，整行不扔。
# 线程上限 4、内存上限 8G；历史全在内存稀疏盘上录；每个世界跑前自己核状态数不超过 10⁶。不跑 checker 档测试、不跑名字带 layer0 的目标、不碰 GPU。
set -euo pipefail
repository="${1:?仓根}"; scratch="${2:?草稿目录}"
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir "$scratch"; mkdir "$scratch/out"
rsync -a --exclude target --exclude .git "$repository"/ "$scratch/repo/"
mkdir -p "$scratch/repo/crates/singlefs-checker-tier/e161_attack"
cp "$model/attack.rs" "$model/r2.rs" "$model/r3.rs" "$scratch/repo/crates/singlefs-checker-tier/e161_attack/"
patch -s "$scratch/repo/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs" < "$model/e161-hook.patch"
(cd "$scratch/repo" && patch -s -p1 < "$model/crates-probes.patch" && patch -s -p1 < "$model/r3-hook.patch")
rsync -a "$scratch/repo/" "$scratch/variant/"
(cd "$scratch/repo" && CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
  cargo build --release --offline -p singlefs-checker-tier --features verdict-store --bin e161_crash_state_dedup_and_time_split) > "$scratch/build.log" 2>&1
for world in r3-dump r3-panic-location r3-f1-torn r3-multi-record r3-messages-by-batch; do
  (cd "$scratch/repo" && nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    "$scratch/target/release/e161_crash_state_dedup_and_time_split" attack "$world" 2>/dev/null | grep -v '^LAYER0' > "$scratch/out/$world.out")
done
bash "$model/variants.sh" "$scratch/variant" "$scratch/target-variant" "$scratch/variants" > "$scratch/variants.log" 2>&1
cp "$scratch"/variants/*--*.out "$scratch/out/"
for variant in m57-no-barrier-between-records-and-root m23-recovery-chains-other-instances root-before-records f1-candidates-in-the-harness m457-checker-payload-checksum-offset mkfs-watermark; do
  python3 "$model/compare_dumps.py" "$variant" "$scratch/variants/base--r3-dump.out" "$scratch/variants/$variant--r3-dump.out"
done > "$scratch/out/r3-attribute-reuse.out"
bash "$model/comment_flip.sh" "$scratch/variant" "$scratch/target-debug" "$scratch/comment" | grep '^E7RESULT' > "$scratch/out/r3-a9-comment-flip.out"
python3 "$model/pipeline_model.py" > "$scratch/out/r3-pipeline.out"
python3 "$model/a1_scan.py" "$repository" > "$scratch/out/r3-a1-scan.out"
bash "$model/a1_rename_history.sh" "$repository" > "$scratch/out/r3-a1-rename-history.out"
python3 "$model/a9_static_scan.py" "$repository" > "$scratch/out/r3-a9-static-scan.out"
python3 "$model/rust_tokens.py" selftest > "$scratch/out/r3-rust-tokens-selftest.out"
bash "$model/a7/a7_label_guard.sh" "$scratch/a7" > "$scratch/out/r3-a7-label-guard.out"
bash "$model/a8_fingerprint.sh" "$repository" "$scratch/a8" | grep '^E7RESULT' > "$scratch/out/r3-a8-fingerprint.out"
strip_seconds() { sed -E 's/ (build_)?seconds=[0-9.]+//g' "$1"; }
for produced in "$scratch"/out/*.out; do
  name="$(basename "$produced")"
  if diff <(strip_seconds "$produced") <(strip_seconds "$model/outputs/$name") > /dev/null; then
    echo "$name 与存档逐行相同（只剥掉 seconds= 与 build_seconds= 字段）"
  else
    echo "$name 与存档不同"
  fi
done
