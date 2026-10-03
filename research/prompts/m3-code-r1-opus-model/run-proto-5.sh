#!/usr/bin/env bash
# 第五批（第四批的原型那一半作废重跑：T06 变异编出的产物留在编译目录里，见报告「这条腿自己的限度」），两件并行：
# ① work 上三个错实现（A、D、E）各跑 harness 全包与 core 全部单测；② work2 上先摸新全部源文件强制重编，再跑根环转圈的旧快照表（原样）、
# 合法崩溃态 + 可写挂载三片（原样），与 rollback 一条流在 T06 下。
set -u
R=/tmp/claude-1000/m3-code-r1-opus
L=$R/logs5
mkdir -p $L
part_full_suite() {
  cd $R/work
  CARGO_TARGET_DIR=$R/target python3 $R/model/mutrun.py $R/work $R/model/survivors-full-suite-ade.tsv $L/full-suite > $L/full-suite.out 2>&1
}
part_proto() {
  cd $R/work2
  export CARGO_TARGET_DIR=$R/target2
  find crates -name '*.rs' -exec touch {} +
  cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history --test z2_every_crash_state_of_a_publish_then_a_writable_mount --no-run -j 4 > $L/release-build.log 2>&1 || return 1
  STALE=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_stale_snapshot_from_any_publish_of_a_history-* | grep -v '\.d$' | head -1)
  CRASH=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_every_crash_state_of_a_publish_then_a_writable_mount-* | grep -v '\.d$' | head -1)
  sha256sum $STALE $CRASH crates/singlefs-core/src/mount.rs
  M3R1_PREFIX_OVERWRITES=22 M3R1_MAX_PUBLISHES=1 $STALE --nocapture > $L/stale-wrap-orig.log 2>&1; echo "stale wrap orig rc=$?"
  rm -f $L/b5-*.rc
  { M3R1_THREADS=2 M3R1_FLOWS=rollback,create_inode,crash_remount,overwrite+raise_floor,create_inode+raise_floor $CRASH --nocapture > $L/crash-part0.log 2>&1; echo "$?" > $L/b5-0.rc; } &
  { M3R1_THREADS=2 M3R1_FLOWS=overwrite+rollback,raise_floor+rollback,create_inode+rollback,crash_remount+rollback,rollback+rollback,overwrite+crash_remount,raise_floor+crash_remount,rollback+crash_remount $CRASH --nocapture > $L/crash-part1.log 2>&1; echo "$?" > $L/b5-1.rc; } &
  { M3R1_THREADS=2 M3R1_FLOWS=create_inode+crash_remount,crash_remount+crash_remount,overwrite+create_inode,crash_remount+create_inode $CRASH --nocapture > $L/crash-part2.log 2>&1; echo "$?" > $L/b5-2.rc; } &
  wait
  echo "crash parts rc files: $(ls $L/b5-*.rc | wc -l)"; for i in 0 1 2; do echo "crash part $i rc=$(cat $L/b5-$i.rc)"; done
  F=$(python3 $R/model/apply-mutation.py $R/work2 $R/model/table14.tsv T06)
  M3R1_FLOWS=rollback cargo test --release --offline -p m3r1-opus-proto --test z2_every_crash_state_of_a_publish_then_a_writable_mount -j 4 -- --nocapture > $L/crash-rollback-T06.log 2>&1; echo "crash rollback T06 rc=$?"
  python3 $R/model/apply-mutation.py $R/work2 --restore $F
  cmp crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "work2 mount.rs restored"
}
rm -f $L/b6-*.rc
{ part_full_suite; echo "$?" > $L/b6-0.rc; } &
{ part_proto > $L/part-proto.out 2>&1; echo "$?" > $L/b6-1.rc; } &
wait
echo "parts rc files: $(ls $L/b6-*.rc | wc -l) full_suite=$(cat $L/b6-0.rc) proto=$(cat $L/b6-1.rc)"
cmp $R/work/crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "work mount.rs restored"
