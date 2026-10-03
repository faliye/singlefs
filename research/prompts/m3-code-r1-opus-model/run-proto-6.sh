#!/usr/bin/env bash
# 第六批：旧快照多发布表（内容表只记带文件的根之后）重跑，原样代码，8 片并行；先摸新全部源文件强制重编。
set -u
R=/tmp/claude-1000/m3-code-r1-opus
L=$R/logs6
mkdir -p $L
cd $R/work2
export CARGO_TARGET_DIR=$R/target2
cmp crates/singlefs-core/src/mount.rs /tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs && echo "mount.rs 与冻结副本相同"
find crates -name '*.rs' -exec touch {} +
cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history --no-run > $L/release-build.log 2>&1; echo "build rc=$?"
STALE=$(ls -t $CARGO_TARGET_DIR/release/deps/z2_stale_snapshot_from_any_publish_of_a_history-* | grep -v '\.d$' | head -1)
rm -f $L/b7-*.rc
start=$(date +%s)
for i in 0 1 2 3 4 5 6 7; do
  { M3R1_MAX_PUBLISHES=3 M3R1_SHARD=$i/8 $STALE --nocapture > $L/stale-m3-shard$i.log 2>&1; echo "$?" > $L/b7-$i.rc; } &
done
wait
echo "rc files: $(ls $L/b7-*.rc | wc -l) seconds=$(( $(date +%s) - start ))"
for i in 0 1 2 3 4 5 6 7; do echo "shard $i rc=$(cat $L/b7-$i.rc)"; done
