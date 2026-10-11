#!/usr/bin/env bash
# m3-formal-code-b16-r1 攻方腿复跑（副本上量的数，主 agent 在副本里核）。
# 用法：bash rerun.sh <工作目录>        （工作目录不存在会建；里面放仓副本 ws/、编译目录 target/、target-test/ 与各段输出）
# 环境：FROZEN=冻结副本根（默认 /tmp/claude-1000/m3-formal-code-b16-r1/tree）；REPO=主仓（默认 /home/fy5090/code/singlefs）；
#       CAP=跑原型的内存上限（默认 8G）；THREADS=线程上限（默认 8）。跑之前设 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON（包装按峰值表拒时要）。
# 不跑任何 checker 档测试目标：原型 c670-probe 只把 singlefs-checker-tier 当库编（用它的撕裂镜像与可撕裂判定），跑的是原型自己的二进制；
# 变异行里点名 checker 档测试的那几行（2225–2229）由 run-mutation-rows.sh 跳过。挂钟：本机 8 线程约 15 分钟（各段估时见报告）。
set -euo pipefail
WORK="${1:?工作目录}"
MODEL="$(cd "$(dirname "$0")/.." && pwd)"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b16-r1/tree}"
REPO="${REPO:-/home/fy5090/code/singlefs}"
CAP="${CAP:-8G}"
THREADS="${THREADS:-8}"
mkdir -p "$WORK"
rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/ws/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/ws/"
sed -i 's#    "crates/singlefs-reference-filesystem",#    "crates/singlefs-reference-filesystem",\n    "crates/c670-probe",#' "$WORK/ws/Cargo.toml"
rm -rf "$WORK/ws/crates/c670-probe"
cp -r "$MODEL/c670-probe" "$WORK/ws/crates/"
build() {
  (cd "$WORK/ws" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 4G \
    bash "$REPO/scripts/hosts/capped.sh" "$THREADS" cargo build --offline --release -p c670-probe)
}
probe() {
  nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" "$CAP" "$WORK/target/release/c670-probe" "$@"
}
restore_core() {
  cp "$FROZEN/crates/singlefs-core/src/transaction.rs" "$WORK/ws/crates/singlefs-core/src/transaction.rs"
}
# 一、基线（今天的 core）
build
probe named --segments-up-to 4 > "$WORK/base-named-short.txt"
probe ops AfterFirstFile PublishOverwrite --segments-up-to 4 > "$WORK/base-ops-overwrite-short.txt"
probe random broad 6 0 8 --segments-up-to 4 --threads "$THREADS" > "$WORK/base-broad6-s0-8-short.txt"
probe random broad 6 8 108 --segments-up-to 6 --second-segments-up-to 6 --threads "$THREADS" > "$WORK/base-broad6-s8-108.txt"
probe random rollback 10 0 40 --segments-up-to 6 --second-segments-up-to 6 --threads "$THREADS" > "$WORK/base-rollback-10-s0-40.txt"
probe random reuse 10 0 40 --segments-up-to 6 --second-segments-up-to 6 --threads "$THREADS" > "$WORK/base-reuse-10-s0-40.txt"
probe random deleting 10 0 32 --segments-up-to 6 --second-segments-up-to 6 --threads "$THREADS" > "$WORK/base-deleting-10-s0-32.txt"
probe random unitwall 10 0 32 --segments-up-to 6 --second-segments-up-to 6 --threads "$THREADS" > "$WORK/base-unitwall-10-s0-32.txt"
probe ops AfterMakeFilesystem CloseAndMountWritable,PublishWithoutUnits,CloseAndMountWritable,PublishWithoutUnits,ColdStartRecover,CloseAndMountWritable --no-second > "$WORK/base-ops-a-nosecond.txt"
for weights in broad rollback reuse deleting; do
  probe random "$weights" 12 0 400 --segments-up-to 0 --threads "$THREADS" > "$WORK/abandon-$weights.txt"
done
# 二、第 2227、2228 行（E171 第 242、269 行）与攻方自造的 V242u
for line in 2227 2228; do
  python3 "$MODEL/tools/apply-mutation.py" "$WORK/ws" "$line"
  build
  probe named --segments-up-to 4 > "$WORK/m$line-named-short.txt"
  probe ops AfterFirstFile PublishOverwrite --segments-up-to 4 > "$WORK/m$line-ops-overwrite.txt"
  restore_core
done
python3 "$MODEL/tools/apply-variant-v242u.py" "$WORK/ws"
build
probe named --segments-up-to 4 > "$WORK/v242u-named.txt"
probe ops AfterFirstFile PublishOverwrite --segments-up-to 4 > "$WORK/v242u-ops-overwrite.txt"
restore_core
# 三、harness 档：两份新用例的基线，再逐行跑变异行（第 2215–2224 行；2225–2229 点名 checker 档，跳过）
(cd "$WORK/ws" && CARGO_TARGET_DIR="$WORK/target-test" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 8G \
  bash "$REPO/scripts/hosts/capped.sh" "$THREADS" cargo test --offline -p singlefs-harness \
  --test model_answers_unmount_with_the_floor_at_the_current_version --test writable_mount_refusal_on_a_crash_state_without_read_faults) > "$WORK/harness-base.log" 2>&1
grep -E '^test |test result' "$WORK/harness-base.log"
: > "$WORK/rows.txt"
bash "$MODEL/tools/run-mutation-rows.sh" "$WORK/ws" "$FROZEN" "$WORK/rows.txt" $(seq 2215 2229)
# 四、K1：放开「树表 0 条不答」那一格之后模型照条款字面答什么、对拍报哪一格
cat "$FROZEN/crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs" \
  "$MODEL/harness-tests/attack_unmount_without_trees.rs" > "$WORK/ws/crates/singlefs-harness/tests/attack_unmount_without_trees.rs"
python3 "$MODEL/tools/apply-mutation.py" "$WORK/ws" 2217
(cd "$WORK/ws" && CARGO_TARGET_DIR="$WORK/target-test" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 8G \
  bash "$REPO/scripts/hosts/capped.sh" "$THREADS" cargo test --offline -p singlefs-harness --test attack_unmount_without_trees -- attack_ --nocapture) > "$WORK/k1.log" 2>&1
cp "$FROZEN/crates/singlefs-harness/src/model.rs" "$WORK/ws/crates/singlefs-harness/src/model.rs"
rm "$WORK/ws/crates/singlefs-harness/tests/attack_unmount_without_trees.rs"
grep -h -E '^== |^first_states|^first_c670=|^second_states|^abandoning|^elapsed' "$WORK"/base-*.txt "$WORK"/abandon-*.txt "$WORK"/m22*.txt "$WORK"/v242u-*.txt
cat "$WORK/rows.txt"
grep 'ATTACK-K1' "$WORK/k1.log"
