#!/usr/bin/env bash
# m3-batch19-code-r1 攻方腿（Opus）复跑。用法：bash rerun.sh <阶段> <空工作目录 W>；`bash rerun.sh list W` 按次序列出阶段，逐个前台跑（每个阶段 4 分钟以内）。
# 冻结树认 FROZEN_TREE（默认 /tmp/claude-1000/m3-batch19-code-r1/tree）；仓根认 SINGLEFS_REPO。线程上限 ATTACK_THREADS（默认 6）。
# 只编跑 harness 档测试与自己的原型 crate（attack-b19-probe，名字不带 layer0、不是 checker 档包的测试）；每条编跑都经 run-with-memory-cap.sh。
# 最后 `bash rerun.sh summary W` 把 W/out 里的判定行汇成 W/summary.txt，`bash rerun.sh compare W` 与本目录 out/summary.txt 逐行比，一致退 0。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN_TREE=${FROZEN_TREE:-/tmp/claude-1000/m3-batch19-code-r1/tree}
STAGE=${1:?阶段名}
W=${2:?工作目录}
T=${ATTACK_THREADS:-6}
mkdir -p "$W/out"
W=$(cd "$W" && pwd)
export CARGO_TARGET_DIR=$W/target
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与别的会话各跑各的"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一个原型二进制按 MODE 跑不同的活，峰值表只记了最小的那一种；整编峰值比增量编大"
CAP="nice -n 19 bash $REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="bash $REPO/scripts/hosts/capped.sh"
TESTFILE=the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads
probe() { local out=$1; shift; (cd "$W" && env "$@" $CAP 3G $CAPPED "$T" "$W/target/release/attack-b19-probe" > "$W/out/$out" 2>&1); }
n34() { local s=$1 arms=$2; probe "n34-${s//:/-}-a${arms//,/}.txt" MODE=n34 ARMS="$arms" SHAPES="$s" PRINT_PREFIX=1 DUMP_LEGAL_NOSPACE=1; }
STAGES="setup tsv-mut arms-testfile mut-tests qf-c2 n7 n7-region n34-176-0-1 n34-176-3-1 n34-240-0-1 n34-176-0-36 n34-176-0-8 n34-176-0-24 n34-176-0-30 n34-176-0-44 n34-176-0-52 n34-192-0-36 n34-240-0-36 n34-176-3-36 lone-before lone-k3-n15-arm0 lone-k3-n15-arm6 summary compare"
case "$STAGE" in
  list) echo "$STAGES" | tr ' ' '\n' ;;
  setup)
    rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/" "$W/tree/"
    rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/" "$W/pristine/"
    python3 "$MODEL/scripts/make_arm_patch.py" "$W/tree"
    bash "$MODEL/scripts/install_probe.sh" "$W/tree" "$MODEL/probe"
    (cd "$W/tree" && $CAP 6G $CAPPED "$T" cargo build --release --offline -p attack-b19-probe) > "$W/out/build.txt" 2>&1
    (cd "$W/tree" && $CAP 6G $CAPPED "$T" cargo test --offline -p singlefs-harness --test $TESTFILE --test unlink_mount_continuation_and_the_emptied_version --test unlink_writes_per_transaction_do_not_grow_with_the_file --test inode_number_watermark_pressed_back_reissues_a_number --no-run) >> "$W/out/build.txt" 2>&1
    tail -n 3 "$W/out/build.txt" ;;
  tsv-mut)
    # 冻结副本原样（不打臂补丁）的拷贝上照 crates/mutations.tsv 原文施加第 19 批 甲、N4 两行，跑那份测试文件；跑完还原。
    export CARGO_TARGET_DIR=$W/target-pristine
    for m in '删除批第 19 批 甲' '删除批第 19 批 N4'; do
      python3 "$MODEL/scripts/apply_mutation.py" "$W/pristine" "$FROZEN_TREE" "$FROZEN_TREE/crates/mutations.tsv" "$m"
      (cd "$W/pristine" && $CAP 6G $CAPPED "$T" cargo test --offline -p singlefs-harness --test $TESTFILE) > "$W/out/tsv-$(echo "$m" | awk '{print $4}').txt" 2>&1 || true
    done
    python3 "$MODEL/scripts/apply_mutation.py" "$W/pristine" "$FROZEN_TREE" --restore crates/singlefs-core/src/mount.rs ;;
  arms-testfile)
    for a in 0 1 2 3 4 5 6; do (cd "$W/tree" && B19_ARM=$a $CAP 3G $CAPPED "$T" cargo test --offline -p singlefs-harness --test $TESTFILE) > "$W/out/testfile-arm$a.txt" 2>&1 || true; done ;;
  mut-tests)
    (cd "$W/tree" && B19_MUT=q-unlink-mono $CAP 3G $CAPPED "$T" cargo test --offline -p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version) > "$W/out/test-qf-q-unlink-mono.txt" 2>&1 || true
    (cd "$W/tree" && B19_MUT=c2-third $CAP 3G $CAPPED "$T" cargo test --offline -p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file -- --nocapture) > "$W/out/test-n6-c2-third.txt" 2>&1 || true
    (cd "$W/tree" && $CAP 3G $CAPPED "$T" cargo test --offline -p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number) > "$W/out/test-n7-case-one.txt" 2>&1 || true ;;
  qf-c2)
    probe qf-legal.txt MODE=qf; probe qf-mono.txt MODE=qf B19_MUT=q-unlink-mono
    probe c2-legal.txt MODE=c2 UNITS=433; probe c2-third.txt MODE=c2 UNITS=433 B19_MUT=c2-third ;;
  n7) probe n7.txt MODE=n7 NS=1,2,3 COMBOS="c3;c3,u3;c3,u4;c3,u1;c3,r0;c3,u3,r1;c3,x;c3,u4,x;c3,w,x;c3,u3,c1;c3,u3,u4;c3,u2,r1,u2;c3,x,c2;c2,r0,c2;c3,u3,x;c5,u2,u3,r1,x;c3,w,w,x,c1" ;;
  n7-region) python3 "$MODEL/scripts/n7_region.py" "$W/out/n7.txt" > "$W/out/n7-region.txt"; cat "$W/out/n7-region.txt" ;;
  n34-176-0-1) n34 176:0:1 0,1,3,4,5,6 ;;
  n34-176-3-1) n34 176:3:1 0,3,6 ;;
  n34-240-0-1) n34 240:0:1 0,3,6 ;;
  n34-176-0-36) n34 176:0:36 0,2 ;;
  n34-176-0-8) n34 176:0:8 0,2 ;;
  n34-176-0-24) n34 176:0:24 0,2 ;;
  n34-176-0-30) n34 176:0:30 0,2 ;;
  n34-176-0-44) n34 176:0:44 0,2 ;;
  n34-176-0-52) n34 176:0:52 0,2 ;;
  n34-192-0-36) n34 192:0:36 0,2 ;;
  n34-240-0-36) n34 240:0:36 0,2 ;;
  n34-176-3-36) n34 176:3:36 0,2 ;;
  lone-before) probe lone-before.txt MODE=lone ARMS=0 LONE_N= SHAPES="176:0:1;176:1:1;176:3:1;176:0:36" ;;
  lone-k3-n15-arm0) probe lone-k3-n15-arm0.txt MODE=lone ARMS=0 LONE_N=15 SHAPES=176:3:1 ;;
  lone-k3-n15-arm6) probe lone-k3-n15-arm6.txt MODE=lone ARMS=6 LONE_N=15 SHAPES=176:3:1 ;;
  summary)
    (cd "$W/out" && for f in $(ls | sort); do
       grep -hE '^(REGION|MISMATCH|START|ENUM|HIST|PREFIX|LEGALFAIL|LONE|LONEH|LONEBEFORE|COMBO|OFF|OFFENUM|ON|ONSUM|QF|C2STEP|C2RUN|test |test result|thread .* panicked at)' "$f" | sed -e "s/ elapsed=[0-9.]*s//" -e 's/ ([0-9]*)//' -e 's/finished in [0-9.]*s//' | sort | sed "s|^|$f: |"
     done) > "$W/summary.txt"; wc -l "$W/summary.txt" ;;
  compare) diff <(sort "$MODEL/out/summary.txt") <(sort "$W/summary.txt") && echo "summary 逐行一致" ;;
  *) echo "没有这个阶段：$STAGE（bash rerun.sh list W）" >&2; exit 2 ;;
esac
