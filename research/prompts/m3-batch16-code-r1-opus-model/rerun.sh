#!/usr/bin/env bash
# m3-batch16-code-r1 攻方腿（Opus）复跑。用法：bash rerun.sh <阶段> <空工作目录 W>；`bash rerun.sh list W` 列出阶段，按列出的次序逐个跑
# （每个阶段前台 4 分钟以内；整套约 45 个阶段、一个半小时上下，要一口气跑的由主 agent 放后台：`bash rerun.sh all W`）。
# 最后 `bash rerun.sh compare W`：重算 W/out 的判定行，与本目录 out/summary.txt 逐行比，一致才退 0。
# 只编跑 harness 档与自己的原型 crate（名字不带 layer0、不是 checker 档包的测试）；冻结树路径认 FROZEN_TREE。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
MODEL=$REPO/research/prompts/m3-batch16-code-r1-opus-model
FROZEN=${FROZEN_TREE:-/tmp/claude-1000/m3-batch16-code-r1/tree}
STAGE=${1:?阶段名}
export ATTACK_DIR=${2:?给一个空的工作目录}
mkdir -p "$ATTACK_DIR/out"
cd "$ATTACK_DIR"
S=$MODEL/scripts
arm_tree() { # <臂名> <补丁…>：冻结副本拷一份、加原型 crate、打补丁
  local arm=$1; shift
  rsync -a --exclude target --exclude .git "$FROZEN/" "$arm/"
  (cd "$arm" && patch -s -p0 < "$MODEL/patches/workspace-member.patch")
  for p in "$@"; do (cd "$arm" && patch -s -p0 < "$MODEL/patches/$p"); done
  mkdir -p "$arm/crates/attack-b16-probe/src"
  cp "$MODEL/probe/Cargo.toml" "$arm/crates/attack-b16-probe/Cargo.toml"
  cp "$MODEL/probe/main.rs" "$arm/crates/attack-b16-probe/src/main.rs"
}
build() { ATTACK_THREADS=${ATTACK_THREADS_BUILD:-15} bash "$S/build.sh" "$1" > "build-$1.log" 2>&1; }
T1() { ATTACK_THREADS=1 "$@"; }
NARROW="AFTER_J=0 MODE=r2 FAMILY=narrow BATCHES=0,2,3 RCUT_PRINT=3"
SL=168,184,200,232,264,296,328
narrow3() { # <批号基> <臂> <前缀> <段 1|2|3> [额外环境…]：形 R 窄盘族一格（三段各在 4 分钟以内）
  local b=$1 arm=$2 pre=$3 part=$4; shift 4
  case $part in
    1) env "$@" $NARROW UNITS=2,4,8 ATTACK_THREADS=1 bash "$S/grid-batch.sh" "$b" "$arm" "$pre" SLOTS $SL OVERWRITES 0,3 ;;
    2) env "$@" $NARROW UNITS=2,4 ATTACK_THREADS=1 bash "$S/grid-batch.sh" "$b" "$arm" "$pre" SLOTS $SL OVERWRITES 1,6 ;;
    3) env "$@" $NARROW UNITS=8 ATTACK_THREADS=1 bash "$S/grid-batch.sh" "$b" "$arm" "$pre-u8" SLOTS $SL OVERWRITES 1,6 ;;
  esac
}
PLAIN="OVERWRITES_AFTER=0,1 AFTER_J=0,2 MODE=r2 FAMILY=plain RCUT_PRINT=3"
plain_small() { local b=$1 arm=$2 pre=$3; shift 3
  for w in 384 256 240; do env "$@" $PLAIN WIDTHS=$w ATTACK_THREADS=1 bash "$S/grid-batch.sh" "$b" "$arm" "$pre-w$w" OVERWRITES 0,1,2,4 REMOUNTS 0,1,2; done; }
L0() { env "$@" ATTACK_THREADS=14 bash "$S/run.sh" "${ARM:?}" "out/${LOG:?}.log"; }
N1N="MODE=n1 FAMILY=narrow ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= OVERWRITES=1,4,9 UNITS=3,6,11 BATCHES=2,5"
N1NC="MODE=n1 FAMILY=ncycle ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= OVERWRITES=1,3,6 UNITS=2,6 BATCHES=2 OVERWRITES_AFTER_UNMOUNT=0"
N1A="MODE=n1 FAMILY=abandon ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= WIDTHS=384,256,240,4 OVERWRITES=4,6,7 OVERWRITES_AFTER=3,5"
N1C="MODE=n1 FAMILY=cycle ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= OVERWRITES=1,3,7,11 REMOUNTS=0,2 OVERWRITES_AFTER=1,3 OVERWRITES_AFTER_UNMOUNT=3,5,6"
second() { local arm=$1 b=$2
  env $N1N ATTACK_THREADS=1 bash "$S/sweep-batch.sh" "$b" "$arm" "s2n-$arm" SLOTS 168 200 232 264 296 328 360
  env $N1NC ATTACK_THREADS=1 bash "$S/sweep-batch.sh" "$((b+1))" "$arm" "s2nc-$arm" SLOTS 184 216 248 280 312
  env $N1A ATTACK_THREADS=1 bash "$S/run.sh" "$arm" "out/s2a-$arm.log"
  env $N1C ATTACK_THREADS=1 bash "$S/sweep-batch.sh" "$((b+2))" "$arm" "s2c-$arm" WIDTHS 4 384 256 240; }
n7_tests() {
  (cd n7clamp && CARGO_TARGET_DIR=$ATTACK_DIR/target-n7 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=180 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上编跑五个 harness 测试目标，峰值未登记" \
    RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿副本与别的会话各跑各的" nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 6G \
    bash "$REPO/scripts/capped.sh" 15 cargo test --offline --no-fail-fast "$@" -p singlefs-harness \
    --test unlink_mount_continuation_and_the_emptied_version --test unlink_single_file_publishes \
    --test unlink_idempotence_criteria_and_rollback_to_the_emptied_version --test unlink_in_histories_judged_by_the_model \
    --test mount_transparency_judged_field_by_field_after_a_writable_mount); }
run_stage() {
  case "$1" in
    build-jia) arm_tree jia; build jia ;;
    build-ga) arm_tree ga ga.patch; build ga ;;
    build-nh) arm_tree nh nh.patch; build nh ;;
    build-imm) arm_tree imm imm.patch; build imm ;;
    build-n7clamp) arm_tree n7clamp n7-clamp.patch; build n7clamp ;;
    a-jia-[123]) narrow3 1 jia a-jia "${1##*-}" ;;
    a-ga-[123]) narrow3 3 ga a-ga "${1##*-}" ;;
    b-jia-h1-[123]) narrow3 5 jia b-jia-h1 "${1##*-}" HIDE_DEV=1 ;;
    b-nh-h1-[123]) narrow3 7 nh b-nh-h1 "${1##*-}" HIDE_DEV=1 ;;
    b-jia-h0-1) narrow3 18 jia b-jia-h0 1 HIDE_DEV=0 ;;
    b-jia-e1-1) narrow3 27 jia b-jia-e1 1 HIDE_DEV=1 HIDE_ERR=1 ;;
    c-jia|c-ga) env $NARROW UNITS=2,4,8 ATTACK_THREADS=1 bash "$S/grid-batch.sh" 10 "${1#c-}" "$1" SLOTS 136,144,152,160 OVERWRITES 0,1,3 ;;
    p-jia-w4-a|p-ga-w4-a) a=${1#p-}; a=${a%%-*}; env $PLAIN WIDTHS=4 ATTACK_THREADS=1 bash "$S/grid-batch.sh" 11 "$a" "p-$a-w4" OVERWRITES 0,1,2,4 REMOUNTS 0,1 ;;
    p-jia-w4-b|p-ga-w4-b) a=${1#p-}; a=${a%%-*}; env $PLAIN WIDTHS=4 ATTACK_THREADS=1 bash "$S/grid-batch.sh" 12 "$a" "p-$a-w4" OVERWRITES 0,1,2,4 REMOUNTS 2 ;;
    p-jia-small|p-ga-small) a=${1#p-}; a=${a%-small}; plain_small 13 "$a" "p-$a" ;;
    q-jia-h1-small|q-nh-h1-small) a=${1#q-}; a=${a%%-*}; plain_small 17 "$a" "q-$a-h1" HIDE_DEV=1 ;;
    twice) rm -f b19-*.rc; i=0
      for s in 264:6:4:0 200:1:4:2 232:0:4:3; do for a in jia ga; do i=$((i+1))
        { TWICE=1 MODE=r2 FAMILY=list SHAPES="$s;" RCUT_PRINT=3 ATTACK_THREADS=1 bash "$S/run.sh" $a "out/tw-$a-$s.log"; echo "$?" > "b19-$i.rc"; } &
      done; done
      for a in jia ga; do i=$((i+1)); { TWICE=1 MODE=r2 FAMILY=plainlist SHAPES="240:0:1:1;" RCUT_PRINT=3 ATTACK_THREADS=1 bash "$S/run.sh" $a "out/tw-$a-p240.log"; echo "$?" > "b19-$i.rc"; } & done
      wait; [ "$(ls b19-*.rc | wc -l)" -eq 8 ] || { echo "twice 作废：.rc 份数对不上"; exit 1; }; cat b19-*.rc ;;
    e1) ARM=jia LOG=e1-renum-h2-jia-hide1 L0 HIDE_DEV=1 RENUM_N=1 MODE=r2 FAMILY=list SHAPES="176:0:4:3;" RCUT_PRINT=0 ;;
    e2) ARM=jia LOG=e2-renum-h2-jia L0 RENUM_N=1 MODE=r2 FAMILY=list SHAPES="176:0:4:3;" RCUT_PRINT=0 ;;
    l1-jia) ARM=jia LOG=l1-jia L0 RENUM_N=15 MODE=r2 FAMILY=list SHAPES="264:6:4:0;" RCUT_PRINT=0 ;;
    l1-jia-hide1) ARM=jia LOG=l1-jia-hide1 L0 HIDE_DEV=1 RENUM_N=15 MODE=r2 FAMILY=list SHAPES="264:6:4:0;" RCUT_PRINT=0 ;;
    l1-nh-hide1) ARM=nh LOG=l1-nh-hide1 L0 HIDE_DEV=1 RENUM_N=15 MODE=r2 FAMILY=list SHAPES="264:6:4:0;" RCUT_PRINT=0 ;;
    l1-ga) ARM=ga LOG=l1-ga L0 RENUM_N=15 MODE=r2 FAMILY=list SHAPES="264:6:4:0;" RCUT_PRINT=0 ;;
    l2) ARM=jia LOG=l2-jia-hide1 L0 HIDE_DEV=1 RENUM_N=1 MODE=r2 FAMILY=plainlist SHAPES="240:0:1:1;" RCUT_PRINT=0
        ARM=jia LOG=l2-jia L0 RENUM_N=1 MODE=r2 FAMILY=plainlist SHAPES="240:0:1:1;" RCUT_PRINT=0
        ARM=nh LOG=l2-nh-hide1 L0 HIDE_DEV=1 RENUM_N=1 MODE=r2 FAMILY=plainlist SHAPES="240:0:1:1;" RCUT_PRINT=0
        ARM=ga LOG=l2-ga L0 RENUM_N=1 MODE=r2 FAMILY=plainlist SHAPES="240:0:1:1;" RCUT_PRINT=0 ;;
    second-jia) second jia 20 ;;
    second-imm) second imm 23 ;;
    qf) MODE=qf OVERWRITES=1,4 UNITS=2,6,11 BATCHES=1,2 QF_TIMES=1700000000,1,1788000001 ATTACK_THREADS=1 bash "$S/sweep-batch.sh" 26 jia qf-jia SLOTS 184 216 248 280 312
        MODE=qf SLOTS=216 OVERWRITES=1 UNITS=6 BATCHES=1 QF_TIMES=1700000000,1788000001 ATTACK_THREADS=1 bash "$S/run.sh" n7clamp out/qf-n7clamp.log ;;
    unmount-hide) for h in 0 1; do for e in 0 1; do
        HIDE_DEV=$h HIDE_ERR=$e MODE=unmounthide FAMILY=narrow OVERWRITES=0,1,3,6 UNITS=2,4,8 BATCHES=0,2,3 ATTACK_THREADS=1 bash "$S/sweep-batch.sh" 28 jia "uh-n-$h-$e" SLOTS 168 184 200 232 264 296 328
        HIDE_DEV=$h HIDE_ERR=$e MODE=unmounthide FAMILY=plain OVERWRITES=0,1,2,4 REMOUNTS=0,1,2 OVERWRITES_AFTER=0,1 ATTACK_THREADS=1 bash "$S/sweep-batch.sh" 29 jia "uh-p-$h-$e" WIDTHS 4 384 256 240
      done; done ;;
    tests-n7clamp-build) n7_tests --no-run > n7-build.log 2>&1 ;;
    tests-n7clamp) n7_tests > out/tests-n7clamp.log 2>&1 || true; grep '^test result' out/tests-n7clamp.log ;;
    compare) bash "$S/summary-all.sh" "$ATTACK_DIR" > "$ATTACK_DIR/summary.txt"
      if diff "$MODEL/out/summary.txt" "$ATTACK_DIR/summary.txt" > "$ATTACK_DIR/summary.diff"; then echo "compare: 判定行与 out/summary.txt 逐行相同（$(wc -l < "$ATTACK_DIR/summary.txt") 行）"; else echo "compare: 有不同，见 $ATTACK_DIR/summary.diff"; exit 1; fi ;;
    *) echo "不认得的阶段 $1"; exit 2 ;;
  esac
}
STAGES="build-jia build-ga build-nh build-imm build-n7clamp
a-jia-1 a-jia-2 a-jia-3 a-ga-1 a-ga-2 a-ga-3 b-jia-h1-1 b-jia-h1-2 b-jia-h1-3 b-nh-h1-1 b-nh-h1-2 b-nh-h1-3 b-jia-h0-1 b-jia-e1-1 c-jia c-ga
p-jia-w4-a p-jia-w4-b p-jia-small p-ga-w4-a p-ga-w4-b p-ga-small q-jia-h1-small q-nh-h1-small
twice e1 e2 l1-jia l1-jia-hide1 l1-nh-hide1 l1-ga l2 second-jia second-imm qf unmount-hide tests-n7clamp-build tests-n7clamp compare"
case "$STAGE" in
  list) echo $STAGES ;;
  all) for s in $STAGES; do echo "== $s"; run_stage "$s"; done ;;
  *) run_stage "$STAGE" ;;
esac
