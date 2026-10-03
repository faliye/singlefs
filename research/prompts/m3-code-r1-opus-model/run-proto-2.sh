#!/usr/bin/env bash
# 第二批原型：重读一次（原样与去掉重读）、写零三个后端与真 errno、mkfs 打洞用例在 tmpfs 与「内核报不支持」下、两张扫描表的估时。
set -u
cd /tmp/claude-1000/m3-code-r1-opus/work2
export CARGO_TARGET_DIR=/tmp/claude-1000/m3-code-r1-opus/target2
L=/tmp/claude-1000/m3-code-r1-opus/logs
M=crates/singlefs-core/src/mount.rs
cargo test --offline -p m3r1-opus-proto --test z2_reread_one_transient_root_slot_read_error -- --nocapture > $L/reread-orig.log 2>&1; echo "reread-orig rc=$?"
cp $M $L/mount.rs.orig
python3 -c "
p='$M'; s=open(p).read(); o='    let reread = reading_other_than_the_version_root()?;'
assert s.count(o)==1; open(p,'w').write(s.replace(o,'    let reread = first_reading;'))"
cargo test --offline -p m3r1-opus-proto --test z2_reread_one_transient_root_slot_read_error -- --nocapture > $L/reread-no-reread.log 2>&1; echo "reread-no-reread rc=$?"
cp $L/mount.rs.orig $M
cmp $M /tmp/claude-1000/m3-code-r1/tree/$M && echo "mount.rs restored"
cargo test --offline -p m3r1-opus-proto --test z3_zero_fill_on_the_file_backend_against_the_other_backends -- --nocapture --test-threads=1 > $L/z3-backends.log 2>&1; echo "z3 rc=$?"
cargo build --offline -p m3r1-opus-proto --bin with_fallocate_errno > $L/build-wrapper.log 2>&1; echo "build wrapper rc=$?"
cargo test --offline -p singlefs-harness --test mkfs_on_image_files_punches_the_zeroed_ranges --no-run > $L/mkfs-test-build.log 2>&1; echo "build mkfs test rc=$?"
BIN=$(ls -t $CARGO_TARGET_DIR/debug/deps/mkfs_on_image_files_punches_the_zeroed_ranges-* | grep -v '\.d$' | head -1)
echo "mkfs test binary $BIN"
mkdir -p /dev/shm/m3r1-opus-tmp
TMPDIR=/dev/shm/m3r1-opus-tmp $BIN > $L/mkfs-test-on-tmpfs.log 2>&1; echo "mkfs test on tmpfs rc=$?"
rmdir /dev/shm/m3r1-opus-tmp
for E in EOPNOTSUPP ENOSPC; do
  $CARGO_TARGET_DIR/debug/with_fallocate_errno $E $BIN > $L/mkfs-test-fallocate-$E.log 2>&1; echo "mkfs test with fallocate=$E rc=$?"
done
M3R1_ENUMERATE=0 cargo test --release --offline -p m3r1-opus-proto --test z2_every_crash_state_of_a_publish_then_a_writable_mount -- --nocapture > $L/crash-counts.log 2>&1; echo "crash-counts rc=$?"
M3R1_MAX_PUBLISHES=1 cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history -- --nocapture > $L/stale-m1.log 2>&1; echo "stale-m1 rc=$?"
