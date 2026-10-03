#!/usr/bin/env bash
# 第三批：重读一次原样复跑；崩溃态 + 可写挂载的估时（rollback 一条流）；多发布旧快照全表 8 片并行；几个错实现在单发布旧快照表与崩溃态表上的结局。
set -u
R=/tmp/claude-1000/m3-code-r1-opus
cd $R/work2
export CARGO_TARGET_DIR=$R/target2
L=$R/logs
find crates/m3r1-opus-proto -name '*.rs' -exec touch {} +
cargo test --offline -p m3r1-opus-proto --test z2_reread_one_transient_root_slot_read_error -- --nocapture > $L/reread-orig.log 2>&1; echo "reread-orig rc=$?"
cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history --test z2_every_crash_state_of_a_publish_then_a_writable_mount --no-run > $L/release-build.log 2>&1; echo "release build rc=$?"
STALE=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_stale_snapshot_from_any_publish_of_a_history-* | grep -v '\.d$' | head -1)
CRASH=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_every_crash_state_of_a_publish_then_a_writable_mount-* | grep -v '\.d$' | head -1)
echo "STALE=$STALE CRASH=$CRASH"
start=$(date +%s); M3R1_FLOWS=rollback M3R1_THREADS=8 $CRASH --nocapture > $L/crash-rollback.log 2>&1; echo "crash rollback rc=$? seconds=$(( $(date +%s) - start ))"
rm -f $L/b1-*.rc
start=$(date +%s)
for i in 0 1 2 3 4 5 6 7; do
  { M3R1_MAX_PUBLISHES=3 M3R1_SHARD=$i/8 $STALE --nocapture > $L/stale-m3-shard$i.log 2>&1; echo "$?" > $L/b1-$i.rc; } &
done
wait
echo "stale m3 rc files: $(ls $L/b1-*.rc | wc -l) seconds=$(( $(date +%s) - start ))"
for i in 0 1 2 3 4 5 6 7; do echo "shard $i rc=$(cat $L/b1-$i.rc)"; done
for MUT in A_only_device_zero E_any_self_verified_root C_mount_never_root B_txg_only; do
  F=$(python3 $R/model/apply-mutation.py $R/work2 $R/model/wrong-impls.tsv $MUT)
  cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history -- --nocapture > $L/stale-m1-$MUT.log 2>&1 <<< "" ; echo "stale-m1 $MUT rc=$?"
  python3 $R/model/apply-mutation.py $R/work2 --restore $F
done
F=$(python3 $R/model/apply-mutation.py $R/work2 $R/model/table14.tsv T06)
M3R1_FLOWS=rollback cargo test --release --offline -p m3r1-opus-proto --test z2_every_crash_state_of_a_publish_then_a_writable_mount -- --nocapture > $L/crash-rollback-T06.log 2>&1; echo "crash rollback T06 rc=$?"
python3 $R/model/apply-mutation.py $R/work2 --restore $F
cmp crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "mount.rs restored"
