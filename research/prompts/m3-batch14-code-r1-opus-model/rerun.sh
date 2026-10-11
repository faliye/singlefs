#!/usr/bin/env bash
# m3-batch14-code-r1 攻方腿（Opus）复跑。用法：bash rerun.sh <阶段> <空工作目录 W>，阶段按下面 STAGES 的次序逐个跑（每段前台 4 分钟以内）；
# `bash rerun.sh all W` 按次序跑完全部（整套约 50 分钟，要放后台的话由主 agent 起）。最后 `bash rerun.sh compare W` 与 out/ 比判定行。
# 只跑 harness 档与自己的原型 crate（名字不带 layer0、不是 checker 档包）；线程上限 ATTACK_THREADS（默认 7），同一批并行件数不超过它。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
MODEL=$REPO/research/prompts/m3-batch14-code-r1-opus-model
FROZEN=${FROZEN_TREE:-/tmp/claude-1000/m3-batch14-code-r1/tree}
STAGE=${1:?阶段名}
export ATTACK_DIR=${2:?给一个空的工作目录}
mkdir -p "$ATTACK_DIR/out"
cd "$ATTACK_DIR"
S=$MODEL/scripts
N1_NARROW="MODE=n1 FAMILY=narrow ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= OVERWRITES=0,1,2,3,4,5,6,7,8,9,10 UNITS=1,2,4,8,14 BATCHES=0,1,3,7,13"
N1_CYCLE="MODE=n1 FAMILY=cycle ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= REMOUNTS=0,1 OVERWRITES_AFTER=0,2 OVERWRITES_AFTER_UNMOUNT=0,1,2,4,8"
N1_NCYCLE="MODE=n1 FAMILY=ncycle ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= OVERWRITES=0,2,5,8 UNITS=1,4,14 BATCHES=0,1,3 OVERWRITES_AFTER_UNMOUNT=0,1,2,4"
N1_ABANDON="MODE=n1 FAMILY=abandon ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= WIDTHS=384,256,240,4 OVERWRITES=1,2,3,5,8 OVERWRITES_AFTER=0,1,2,4"
R_NARROW="MODE=remount FAMILY=narrow UNITS=1,4,14 BATCHES=0,1,3 RCUT_PRINT=5"
R_PLAIN="MODE=remount FAMILY=plain OVERWRITES=0,1,3,7 OVERWRITES_AFTER=0,1 RCUT_PRINT=5"
arm_tree() { # <臂名> <补丁…>：冻结副本拷一份、打补丁
  local arm=$1; shift
  rsync -a --exclude target --exclude .git "$FROZEN/" "$arm/"
  (cd "$arm" && patch -s -p0 < "$MODEL/patches/workspace-member.patch")
  for p in "$@"; do (cd "$arm" && patch -s -p0 < "$MODEL/patches/$p"); done
}
test_tree() { # <名> <补丁>：冻结副本拷一份、打一个变异或改法补丁，跑点名的 harness 测试目标
  local name=$1 p=$2; shift 2
  rsync -a --exclude target --exclude .git "$FROZEN/" "t-$name/"
  (cd "t-$name" && patch -s -p0 < "$MODEL/patches/$p")
  local args=(); for t in "$@"; do args+=(--test "$t"); done
  (cd "t-$name" && CARGO_TARGET_DIR=$ATTACK_DIR/target-tests RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上编跑 harness 测试目标" \
    RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿副本上的变异测试与原型扫描并行各跑各的" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=180 \
    nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 8G bash "$REPO/scripts/capped.sh" "${ATTACK_THREADS:-7}" \
    cargo test --offline --no-fail-fast -p singlefs-harness "${args[@]}") > "out/tests-$name.log" 2>&1 || true
}
run_stage() {
  case "$1" in
    build-ga) arm_tree ga; bash "$S/build.sh" ga > build-ga.log 2>&1 ;;
    build-imm) arm_tree imm imm-bing1-releases-everything.patch; bash "$S/build.sh" imm > build-imm.log 2>&1 ;;
    build-held) arm_tree held held-unmount-holds-everything.patch; bash "$S/build.sh" held > build-held.log 2>&1 ;;
    build-fx) arm_tree fx fx-mount-holds-above-the-newest-root-on-every-device.patch; bash "$S/build.sh" fx > build-fx.log 2>&1 ;;
    n1-narrow-*-a) a=${1#n1-narrow-}; a=${a%-a}; env $N1_NARROW bash "$S/sweep-batch.sh" 1 "$a" "n1n-$a" SLOTS 176 192 208 224 240 256 ;;
    n1-narrow-*) a=${1#n1-narrow-}; a=${a%-b}; env $N1_NARROW bash "$S/sweep-batch.sh" 2 "$a" "n1n-$a" SLOTS 272 288 304 320 336 352 ;;
    n1-cycle-*) a=${1#n1-cycle-}; env $N1_CYCLE bash "$S/grid-batch.sh" 3 "$a" "n1c-$a" WIDTHS 4 OVERWRITES 0,2,5,9,14
      env $N1_CYCLE OVERWRITES=0,2,5,9,14 bash "$S/sweep-batch.sh" 4 "$a" "n1c-$a-s" WIDTHS 384 256 240 ;;
    n1-ncycle-*) a=${1#n1-ncycle-}; env $N1_NCYCLE bash "$S/sweep-batch.sh" 5 "$a" "n1nc-$a" SLOTS 176 208 240 272 304 336 ;;
    n1-abandon) for a in ga imm held; do env $N1_ABANDON bash "$S/run.sh" "$a" "out/n1a-$a.log"; done ;;
    r-narrow-*) a=${1#r-narrow-}; ow=${a##*-}; a=${a%-*}; env $R_NARROW bash "$S/grid-batch.sh" 6 "$a" "rn-$a" SLOTS 176,208,240,272,304,336 OVERWRITES "$ow" ;;
    r-plain-*-a) a=${1#r-plain-}; a=${a%-a}; env $R_PLAIN bash "$S/grid-batch.sh" 7 "$a" "rp-$a" WIDTHS 384,256,240 REMOUNTS 0,1 ;;
    r-plain-*-c) a=${1#r-plain-}; a=${a%-c}; env $R_PLAIN bash "$S/grid-batch.sh" 9 "$a" "rp-$a" WIDTHS 384,256,240 REMOUNTS 2 ;;
    r-plain-*-b) a=${1#r-plain-}; a=${a%-b}; env $R_PLAIN bash "$S/grid-batch.sh" 8 "$a" "rp-$a" WIDTHS 4 REMOUNTS 0,1,2 ;;
    r-h2-detail) MODE=remount FAMILY=list SHAPES="176:0:4:3;" RCUT_PRINT=100000 bash "$S/run.sh" ga out/r-h2-ga-detail.log ;;
    renum-h2-1) MODE=remount FAMILY=list SHAPES="176:0:4:3;" RENUM_N=1 RCUT_PRINT=0 bash "$S/run.sh" ga out/renum-h2-ga-1.log ;;
    renum-h2-16) MODE=remount FAMILY=list SHAPES="176:0:4:3;" RENUM_N=16 RCUT_PRINT=0 bash "$S/run.sh" ga out/renum-h2-ga-16.log ;;
    qf) MODE=qf SLOTS=176,208,240,272,304,336 OVERWRITES=0,3 UNITS=1,4,14 BATCHES=0,1,3 bash "$S/run.sh" ga out/qf-ga.log ;;
    tests-mq1) test_tree mq1 mq1-session-drops-the-unlink-time.patch unlink_mount_continuation_and_the_emptied_version admission_raises_the_floor_before_refusing unlink_in_histories_judged_by_the_model mount_transparency_judged_field_by_field_after_a_writable_mount ;;
    tests-mq3) test_tree mq3 mq3-intent-keeps-the-previous-time.patch unlink_mount_continuation_and_the_emptied_version unlink_in_histories_judged_by_the_model mount_transparency_judged_field_by_field_after_a_writable_mount ;;
    tests-mg1) test_tree mg1 mg1-max-over-devices.patch a_lone_surviving_device_reads_back_what_it_read_before_the_unmount admission_raises_the_floor_before_refusing model_answers_unmount_with_the_floor_at_the_current_version unlink_mount_continuation_and_the_emptied_version reuse_after_raising_the_floor ;;
    tests-mg4) test_tree mg4 mg4-remainder-not-reclaimed.patch a_lone_surviving_device_reads_back_what_it_read_before_the_unmount admission_raises_the_floor_before_refusing model_answers_unmount_with_the_floor_at_the_current_version unlink_mount_continuation_and_the_emptied_version reuse_after_raising_the_floor mount_transparency_judged_field_by_field_after_a_writable_mount unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence ;;
    tests-fx) test_tree fx fx-mount-holds-above-the-newest-root-on-every-device.patch reuse_after_raising_the_floor admission_raises_the_floor_before_refusing a_lone_surviving_device_reads_back_what_it_read_before_the_unmount unlink_mount_continuation_and_the_emptied_version writable_mount_takes_a_second_instance mount_transparency_judged_field_by_field_after_a_writable_mount raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window ;;
    compare) bash "$S/compare.sh" "$MODEL/out" "$ATTACK_DIR/out" ;;
    *) echo "不认得的阶段 $1"; exit 2 ;;
  esac
}
STAGES="build-ga build-imm build-held build-fx n1-narrow-ga-a n1-narrow-ga-b n1-narrow-imm-a n1-narrow-imm-b n1-narrow-held-a n1-narrow-held-b
n1-cycle-ga n1-cycle-imm n1-ncycle-ga n1-ncycle-imm n1-abandon
r-narrow-ga-0 r-narrow-ga-2 r-narrow-ga-5 r-narrow-ga-8 r-narrow-imm-0 r-narrow-imm-2 r-narrow-imm-5 r-narrow-imm-8
r-narrow-held-0 r-narrow-held-2 r-narrow-held-5 r-narrow-held-8 r-narrow-fx-0 r-narrow-fx-2 r-narrow-fx-5 r-narrow-fx-8
r-plain-ga-a r-plain-ga-b r-plain-ga-c r-plain-imm-a r-plain-imm-b r-plain-imm-c r-plain-held-a r-plain-held-b r-plain-held-c
r-plain-fx-a r-plain-fx-b r-plain-fx-c
r-h2-detail renum-h2-1 renum-h2-16 qf tests-mq1 tests-mq3 tests-mg1 tests-mg4 tests-fx compare"
if [ "$STAGE" = all ]; then for s in $STAGES; do echo "== $s"; run_stage "$s"; done; else run_stage "$STAGE"; fi
