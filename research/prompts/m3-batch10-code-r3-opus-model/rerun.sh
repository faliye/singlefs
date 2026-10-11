#!/bin/bash
# m3-batch10-code-r3 攻方（opus）原型复跑。
# 用法：bash rerun.sh setup <草稿目录> [冻结副本根]   拷冻结副本、打补丁、放原型、release 编译（cargo 经内存包装）
#       bash rerun.sh list                              列出阶段
#       bash rerun.sh run <草稿目录> <阶段>             跑一个阶段（经内存包装与 capped.sh 31），日志写 <草稿目录>/out/<阶段>.log
#       bash rerun.sh compare <草稿目录>                逐份比 OPUS3-SUM / OPUS3-L0-TALLY / OPUS3-K2Y / OPUS-K3 行（去掉 secs= 与 run_s=）与 out/ 里的原件
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
cap() { nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" "$@"; }
STAGES="fixed-prod fixed-reread_only r7-prod-a r7-reread_only-a r7-prod-b r7-reread_only-b r7-prod-c r7-reread_only-c r7-prod-d r7-prod-e r7-prod-f r7-prod-g r7-reread_only-g r7-reread_only-h r7-zero_over_bad-n cells-prod cells-first_wins l0-mat384-d0E-d0Z l0-ab3845-K22-Z l0-ab3845-K22-Z-ro l0-mat384-q3new k2young k3"
stage_cmd() {
  local s=$1
  case $s in
    fixed-prod) echo "OPUS_FIXED_ONLY=1 OPUS_STEPS=30 OPUS_WORKERS=12 ARM= grid all";;
    fixed-reread_only) echo "OPUS_FIXED_ONLY=1 OPUS_STEPS=30 OPUS_WORKERS=12 ARM=reread_only grid all";;
    r7-prod-a) echo "R7 ARM= grid ab-4g-1-0";;
    r7-reread_only-a) echo "R7 ARM=reread_only grid ab-4g-1-0";;
    r7-prod-b) echo "R7 ARM= grid ab-4g-3-0,ab-4g-3-5";;
    r7-reread_only-b) echo "R7 ARM=reread_only grid ab-4g-3-0,ab-4g-3-5";;
    r7-prod-c) echo "R7 ARM= grid mat-4g-30-8";;
    r7-reread_only-c) echo "R7 ARM=reread_only grid mat-4g-30-8";;
    r7-prod-d) echo "R7 ARM= grid ab-384-1-0,ab-384-3-0";;
    r7-prod-e) echo "R7 ARM= grid ab-384-3-5,narrow-264-6-4";;
    r7-prod-f) echo "R7 ARM= grid mat-384-30-8";;
    r7-prod-g) echo "R7 ARM= grid narrow-384-20-2";;
    r7-reread_only-g) echo "R7 ARM=reread_only grid narrow-264-6-4,narrow-384-20-2";;
    r7-reread_only-h) echo "R7 ARM=reread_only grid ab-384-1-0,mat-384-30-8";;
    r7-zero_over_bad-n) echo "R7 ARM=zero_over_bad grid narrow-264-6-4,narrow-384-20-2";;
    cells-prod) echo "OPUS_STEPS=30 ARM= cells ab-384-1-0,ab-384-3-5";;
    cells-first_wins) echo "OPUS_STEPS=30 ARM=first_wins cells ab-384-1-0,ab-384-3-5";;
    l0-mat384-d0E-d0Z) echo "L0 ARM= l0 mat-384-30-8 f-d0-allE-reread-d0-allZ";;
    l0-ab3845-K22-Z) echo "L0 ARM= l0 ab-384-3-5 f-d1-allE-reread-A0/3-Z";;
    l0-ab3845-K22-Z-ro) echo "L0 ARM=reread_only l0 ab-384-3-5 f-d1-allE-reread-A0/3-Z";;
    l0-mat384-q3new) echo "L0 ARM= l0 mat-384-30-8 f-d1-allE-reread-allE";;
    k2young) echo "ARM= k2young";;
    k3) echo "K3";;
    *) echo "没有阶段 $s" >&2; exit 2;;
  esac
}
case ${1:-} in
  setup)
    W=${2:?草稿目录}; FROZEN=${3:-/tmp/claude-1000/m3-batch10-code-r3/tree}
    mkdir -p "$W"; [ -z "$(ls -A "$W")" ] || { echo "草稿目录不空：$W" >&2; exit 2; }
    mkdir -p "$W/tree" "$W/out"
    rsync -a --exclude target --exclude .git "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/crates" "$W/tree/"
    (cd "$W/tree" && for p in mount-arms rollback-instrument workspace-member; do patch -p1 -s < "$HERE/$p.patch"; done)
    mkdir -p "$W/tree/crates/opus-r3-attack/src"
    cp "$HERE/Cargo.toml" "$W/tree/crates/opus-r3-attack/Cargo.toml"; cp "$HERE/main.rs" "$W/tree/crates/opus-r3-attack/src/main.rs"
    cd "$W/tree"
    export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑攻方原型，第一次在新目录里整编"
    cap 8G bash "$REPO/scripts/hosts/capped.sh" 31 cargo build --offline --release -p opus-r3-attack > "$W/out/build.log" 2>&1
    cap 8G bash "$REPO/scripts/hosts/capped.sh" 31 cargo test --offline --release -p singlefs-harness --test rollback_by_a_forward_publish --no-run >> "$W/out/build.log" 2>&1
    echo "setup 完：$W";;
  list) for s in $STAGES; do echo "$s  $(stage_cmd $s)"; done;;
  run)
    W=${2:?草稿目录}; s=${3:?阶段}; c=$(stage_cmd "$s"); BIN="$W/tree/target/release/opus-r3-attack"
    export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令与起点不同，峰值表里那一次是另一种"
    export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600
    if [ "$c" = K3 ]; then
      T=$(ls -t "$W"/tree/target/release/deps/rollback_by_a_forward_publish-* | grep -v '\.d$' | head -1)
      cd "$W/tree/crates/singlefs-harness"
      cap 4G bash "$REPO/scripts/hosts/capped.sh" 31 "$T" --include-ignored --nocapture --test-threads=4 the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root plain_overwrite_whose_root_slot_write_zeroes > "$W/out/$s.log" 2>&1
    else
      envs=(); args=()
      for w in $c; do
        case $w in
          R7) envs+=(OPUS_SEED=7 OPUS_PLANS=400 OPUS_STEPS=30 OPUS_WORKERS=28);;
          L0) envs+=(OPUS_SEED=7 OPUS_PLANS=400 OPUS_STEPS=0);;
          ARM=*) envs+=("OPUS_R3_ARM=${w#ARM=}");;
          *=*) envs+=("$w");;
          *) args+=("$w");;
        esac
      done
      env "${envs[@]}" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 4G bash "$REPO/scripts/hosts/capped.sh" 31 "$BIN" "${args[@]}" > "$W/out/$s.log" 2>&1
    fi
    echo "rc=$? $s";;
  compare)
    W=${2:?草稿目录}; differ=0; total=0
    for s in $STAGES; do
      total=$((total+1))
      norm() { grep -E 'OPUS3-SUM|OPUS3-L0-TALLY|OPUS3-L0-SCAN|OPUS3-K2Y|OPUS-K3|^test ' "$1" | sed -E 's/ secs=[0-9.]+//; s/ run_s=[0-9.]+//; s/finished in [0-9.]+s//' | sort; }
      if ! diff <(norm "$HERE/out/$s.log") <(norm "$W/out/$s.log") > /dev/null; then echo "不同：$s"; differ=$((differ+1)); fi
    done
    echo "比了 $total 份，$differ 份不同";;
  *) echo "用法见文件头" >&2; exit 2;;
esac
