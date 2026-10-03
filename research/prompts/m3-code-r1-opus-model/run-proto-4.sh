#!/usr/bin/env bash
# 第四批，两件并行：① 仓副本 work 上，七个放得过三个靶文件的错实现各跑 harness 全包与 core 全部单测（另加不改源码的基线）；
# ② 原型副本 work2 上，根环转过一圈之后的旧快照表（原样与 E 错实现）、合法崩溃态 + 可写挂载（覆盖写收尾的流除外）三片。
set -u
R=/tmp/claude-1000/m3-code-r1-opus
L=$R/logs
part_full_suite() {
  cd $R/work
  CARGO_TARGET_DIR=$R/target python3 $R/model/mutrun.py $R/work $R/model/survivors-full-suite.tsv $L/full-suite > $L/full-suite.out 2>&1
}
part_proto() {
  cd $R/work2
  export CARGO_TARGET_DIR=$R/target2
  cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history --test z2_every_crash_state_of_a_publish_then_a_writable_mount --no-run -j 4 > $L/release-build-4.log 2>&1 || return 1
  STALE=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_stale_snapshot_from_any_publish_of_a_history-* | grep -v '\.d$' | head -1)
  CRASH=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_every_crash_state_of_a_publish_then_a_writable_mount-* | grep -v '\.d$' | head -1)
  M3R1_PREFIX_OVERWRITES=22 M3R1_MAX_PUBLISHES=1 $STALE --nocapture > $L/stale-wrap-orig.log 2>&1
  rm -f $L/b2-*.rc
  { M3R1_THREADS=2 M3R1_FLOWS=rollback,create_inode,crash_remount,overwrite+raise_floor,create_inode+raise_floor $CRASH --nocapture > $L/crash-part0.log 2>&1; echo "$?" > $L/b2-0.rc; } &
  { M3R1_THREADS=2 M3R1_FLOWS=overwrite+rollback,raise_floor+rollback,create_inode+rollback,crash_remount+rollback,rollback+rollback,overwrite+crash_remount,raise_floor+crash_remount,rollback+crash_remount $CRASH --nocapture > $L/crash-part1.log 2>&1; echo "$?" > $L/b2-1.rc; } &
  { M3R1_THREADS=2 M3R1_FLOWS=create_inode+crash_remount,crash_remount+crash_remount,overwrite+create_inode,crash_remount+create_inode $CRASH --nocapture > $L/crash-part2.log 2>&1; echo "$?" > $L/b2-2.rc; } &
  wait
  echo "crash parts rc files: $(ls $L/b2-*.rc | wc -l)"; for i in 0 1 2; do echo "crash part $i rc=$(cat $L/b2-$i.rc)"; done
  F=$(python3 $R/model/apply-mutation.py $R/work2 $R/model/wrong-impls.tsv E_any_self_verified_root)
  M3R1_PREFIX_OVERWRITES=22 M3R1_MAX_PUBLISHES=1 cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history -j 4 -- --nocapture > $L/stale-wrap-E.log 2>&1; echo "stale wrap E rc=$?"
  python3 $R/model/apply-mutation.py $R/work2 --restore $F
  cmp crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "work2 mount.rs restored"
}
rm -f $L/b3-*.rc
{ part_full_suite; echo "$?" > $L/b3-0.rc; } &
{ part_proto > $L/part-proto.out 2>&1; echo "$?" > $L/b3-1.rc; } &
wait
echo "parts rc files: $(ls $L/b3-*.rc | wc -l) full_suite=$(cat $L/b3-0.rc) proto=$(cat $L/b3-1.rc)"
cmp $R/work/crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "work mount.rs restored"
