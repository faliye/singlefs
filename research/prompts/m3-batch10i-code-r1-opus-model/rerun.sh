#!/bin/bash
# m3-batch10i-code-r1 攻方（opus）原型复跑。
# 用法：bash rerun.sh setup <草稿目录> [冻结副本根]   拷冻结副本、打补丁、放原型、release 编译（经内存包装与 capped.sh 31）
#       bash rerun.sh list                              列出阶段
#       bash rerun.sh run <草稿目录> <阶段>             跑一个阶段，日志写 <草稿目录>/out/<阶段>.log
#       bash rerun.sh compare <草稿目录>                逐份比 OPUSI1-RUN 行（排序后）与本目录 out/ 里的原件
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑攻方原型，与别的会话并行"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令、起点与线程数不同，峰值表里那一次是另一种"
S1="s1grid ab-384-1-0,ab-384-3-0,ab-384-3-5"
stage_env() {
  case $1 in
    scan1-prod-u8-s6) echo "OPUS_I1_ARM= OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=6 OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    scan1-prod-u8-s9) echo "OPUS_I1_ARM= OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=9 OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    scan1-prod-u8-s12) echo "OPUS_I1_ARM= OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=12 OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    scan1-prod-u8-s15) echo "OPUS_I1_ARM= OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=15 OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    scan1-ledger-u8-s6|scan1-ledger-u8-s9|scan1-ledger-u8-s12|scan1-ledger-u8-s15|scan1-refuse-u8-s6|scan1-exact-u8-s6|scan1-asread-u8-s6)
      local arm=${1#scan1-}; arm=${arm%%-*}; local st=${1##*-s}
      echo "OPUS_I1_ARM=$arm OPUS_SEQ_UNITS_AFTER_LATENT=8 OPUS_SEQ_FROM_STEP=$st OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    live-prod-ab384) echo "OPUS_I1_ARM= OPUS_STEPS=40|$S1";;
    live-refuse-ab384|live-ledger-ab384|live-asread-ab384) local a=${1#live-}; echo "OPUS_I1_ARM=${a%-ab384} OPUS_STEPS=40|$S1";;
    live-exact-ab384) echo "OPUS_I1_ARM=exact OPUS_STEPS=40|$S1";;
    live-exact-ab384-A) echo "OPUS_I1_ARM=exact OPUS_S1_ONLY_CLASS=A OPUS_STEPS=40|$S1";;
    ab384-owu) echo "OPUS_I1_ARM= OPUS_OPS=ow-unmount OPUS_STEPS=30|$S1";;
    live-prod-mat-384-30-8|live-exact-mat-384-30-8|live-refuse-mat-384-30-8|live-asread-mat-384-30-8|live-prod-narrow-264-6-4|live-exact-narrow-264-6-4|live-refuse-narrow-264-6-4|live-prod-narrow-384-20-2)
      local r=${1#live-}; local a=${r%%-*}; local sp=${r#*-}; [ "$a" = prod ] && a=; echo "OPUS_I1_ARM=$a OPUS_STEPS=40|s1grid $sp";;
    base-prod) echo "OPUS_I1_ARM= OPUS_BASELINE=1 OPUS_STEPS=40|s1grid ab-384-1-0,ab-384-3-0,ab-384-3-5,mat-384-30-8,narrow-264-6-4,narrow-384-20-2";;
    k2-refuse-3) echo "OPUS_I1_ARM=refuse OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=V OPUS_STEPS=3|s1grid mat-384-30-8,ab-384-3-5";;
    k2-prod-3) echo "OPUS_I1_ARM= OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=V OPUS_STEPS=3|s1grid mat-384-30-8,ab-384-3-5";;
    k2-prod-s6|k2-prod-s9|k2-prod-s12|k2-prod-s15) echo "OPUS_I1_ARM= OPUS_ROLLBACK_AT_END=1 OPUS_S1_ONLY_CLASS=V OPUS_STEPS=${1##*-s}|s1grid mat-384-30-8,ab-384-3-5";;
    *) echo "没有阶段 $1" >&2; exit 2;;
  esac
}
STAGES="scan1-prod-u8-s6 scan1-prod-u8-s9 scan1-prod-u8-s12 scan1-prod-u8-s15 scan1-ledger-u8-s6 scan1-ledger-u8-s9 scan1-ledger-u8-s12 scan1-ledger-u8-s15 scan1-refuse-u8-s6 scan1-exact-u8-s6 scan1-asread-u8-s6 live-prod-ab384 live-refuse-ab384 live-ledger-ab384 live-asread-ab384 live-exact-ab384 live-exact-ab384-A ab384-owu live-prod-mat-384-30-8 live-exact-mat-384-30-8 live-refuse-mat-384-30-8 live-asread-mat-384-30-8 live-prod-narrow-264-6-4 live-exact-narrow-264-6-4 live-refuse-narrow-264-6-4 live-prod-narrow-384-20-2 base-prod k2-prod-3 k2-refuse-3 k2-prod-s6 k2-prod-s9 k2-prod-s12 k2-prod-s15"
case ${1:-} in
  setup)
    W=${2:?草稿目录}; FROZEN=${3:-/tmp/claude-1000/m3-batch10i-code-r1/tree}
    mkdir -p "$W"; [ -z "$(ls -A "$W")" ] || { echo "草稿目录不空：$W" >&2; exit 2; }
    mkdir -p "$W/tree" "$W/out"
    rsync -a --exclude target --exclude .git "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/crates" "$W/tree/"
    (cd "$W/tree" && patch -p1 -s < "$HERE/mount-arms-i1.patch" && patch -p1 -s < "$HERE/workspace-member.patch")
    mkdir -p "$W/tree/crates/opus-i1-attack/src"
    cp "$HERE/Cargo.toml" "$W/tree/crates/opus-i1-attack/Cargo.toml"; cp "$HERE/main.rs" "$W/tree/crates/opus-i1-attack/src/main.rs"
    cd "$W/tree" && nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 5G bash "$REPO/scripts/capped.sh" 15 cargo build --offline --release -p opus-i1-attack > "$W/out/build.log" 2>&1
    echo "setup 完：$W";;
  list) for s in $STAGES; do echo "$s  $(stage_env $s)"; done;;
  run)
    W=${2:?草稿目录}; s=${3:?阶段}; spec=$(stage_env "$s"); envs=${spec%%|*}; args=${spec#*|}
    env $envs OPUS_WORKERS=14 nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 3G bash "$REPO/scripts/capped.sh" 15 "$W/tree/target/release/opus-i1-attack" $args > "$W/out/$s.log" 2>&1
    echo "rc=$? $s";;
  compare)
    W=${2:?草稿目录}; differ=0; total=0
    for s in $STAGES; do
      total=$((total+1))
      norm() { grep -E '^OPUSI1-(RUN|SUM)' "$1" | sort; }
      if ! diff <(norm "$HERE/out/$s.log") <(norm "$W/out/$s.log") > /dev/null; then echo "不同：$s"; differ=$((differ+1)); fi
    done
    echo "比了 $total 份，$differ 份不同";;
  *) echo "用法见文件头" >&2; exit 2;;
esac
