#!/bin/bash
# 攻方腿 m3-root-ring-code-r3 的复跑：在冻结副本的拷贝上打补丁、放原型，逐段经内存包装跑。
# 用法：bash rebuild-and-run.sh <草稿目录> <段…>；段：k1-scan k1-long k1-long-369 k1-fix-a k1-reread k1-reread-fix-b k2-zeros enum-0 enum-2
# 输出在 <草稿目录>/out/<段>.log；每段的判读行以 SCAN / LONG / REREAD / RS3Z / ENUM 开头。
set -u
work="${1:?草稿目录}"; shift
model="$(cd "$(dirname "$0")" && pwd)"
repo=/home/fy5090/code/singlefs
frozen=/tmp/claude-1000/m3-root-ring-code-r3/tree
mkdir -p "$work/out"
if [[ ! -d "$work/tree" ]]; then
  rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/" || exit 11
  (cd "$work/tree" && patch -p1 < "$model/probes.patch") || exit 12
  cp "$model/r3_attack_sweep_probe.rs" "$work/tree/crates/singlefs-harness/tests/" || exit 13
  mkdir -p "$work/tree/crates/r3-attack-enum/src"
  cp "$model/r3-attack-enum/Cargo.toml" "$work/tree/crates/r3-attack-enum/" || exit 14
  cp "$model/r3-attack-enum/src/main.rs" "$work/tree/crates/r3-attack-enum/src/" || exit 15
fi
run() {
  local name="$1"; shift
  (cd "$work/tree" && env RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600 \
     RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑攻方腿原型，与别的会话各跑各的" \
     RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="探针取样量随参数变大，峰值会涨" "$@" \
     nice -n 19 bash "$repo/scripts/hosts/capped.sh" 10 \
     bash "$repo/scripts/hosts/run-with-memory-cap.sh" 4G cargo "${cargo_args[@]}") > "$work/out/$name.log" 2>&1
  echo "$name rc=$?"
}
for step in "$@"; do
  case "$step" in
    k1-scan) cargo_args=(test -p singlefs-harness --test r3_attack_sweep_probe --release -- --nocapture k1_scan); run "$step" PROBE_MOUNTS=40 PROBE_THREADS=8 ;;
    k1-long) cargo_args=(test -p singlefs-harness --test r3_attack_sweep_probe --release -- --nocapture k1_periodic); run "$step" PROBE_MOUNTS=60 PROBE_CHECK_EVERY=5 ;;
    k1-long-369) cargo_args=(test -p singlefs-harness --test r3_attack_sweep_probe --release -- --nocapture k1_periodic); run "$step" PROBE_MOUNTS=420 PROBE_THRESHOLD=369 PROBE_CHECK_EVERY=50 ;;
    k1-fix-a) cargo_args=(test -p singlefs-harness --test r3_attack_sweep_probe --release -- --nocapture k1_periodic); run "$step" R3_ATTACK_FIX_A=1 PROBE_MOUNTS=60 PROBE_CHECK_EVERY=5 ;;
    k1-adversarial) cargo_args=(test -p singlefs-harness --test r3_attack_sweep_probe --release -- --nocapture k1_adversarial); run "$step" PROBE_WARM=0 PROBE_GOAL=12 PROBE_BUDGET=3000 ;;
    k1-reread) cargo_args=(test -p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount --release -- --nocapture r3_attack); run "$step" ;;
    k1-reread-fix-b) cargo_args=(test -p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount --release -- --nocapture r3_attack); run "$step" R3_ATTACK_FIX_B=1 ;;
    k2-zeros) cargo_args=(test -p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount --release -- --nocapture r3_attack); run "$step" ;;
    enum-0) cargo_args=(run -p r3-attack-enum --release); run "$step" SINGLEFS_LAYER0_THREADS=10 PROBE_NTH=0 ;;
    enum-2) cargo_args=(run -p r3-attack-enum --release); run "$step" SINGLEFS_LAYER0_THREADS=10 PROBE_NTH=2 ;;
    *) echo "不认的段：$step"; exit 2 ;;
  esac
done
