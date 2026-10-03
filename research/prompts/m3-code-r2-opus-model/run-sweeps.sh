#!/usr/bin/env bash
# 用法：run-sweeps.sh <标签> <批号> <测试二进制>
# 在一条内存包装里并行跑 Y3（P 的 txg 6..32 分三片）与 Y2R（txg 6、7、8、27 各一片），每件的退出码写进 <草稿>/rc/b<批号>-<件号>.rc。
set -u
TAG=$1; BATCH=$2; BIN=$3
D=/tmp/claude-1000/m3-code-r2-opus
mkdir -p "$D/logs/$TAG" "$D/rc"
rm -f "$D/rc/b$BATCH"-*.rc
cd "$D/work/crates/singlefs-harness"
{ R2_TXGS=6..14 nice -n 19 "$BIN" r2opus_y3 --nocapture --test-threads 1 > "$D/logs/$TAG/y3-a.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-1.rc"; } &
{ R2_TXGS=15..23 nice -n 19 "$BIN" r2opus_y3 --nocapture --test-threads 1 > "$D/logs/$TAG/y3-b.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-2.rc"; } &
{ R2_TXGS=24..32 nice -n 19 "$BIN" r2opus_y3 --nocapture --test-threads 1 > "$D/logs/$TAG/y3-c.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-3.rc"; } &
{ R2_REREAD_TXGS=6 nice -n 19 "$BIN" r2opus_y2 --nocapture --test-threads 1 > "$D/logs/$TAG/y2r-6.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-4.rc"; } &
{ R2_REREAD_TXGS=7 nice -n 19 "$BIN" r2opus_y2 --nocapture --test-threads 1 > "$D/logs/$TAG/y2r-7.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-5.rc"; } &
{ R2_REREAD_TXGS=8 nice -n 19 "$BIN" r2opus_y2 --nocapture --test-threads 1 > "$D/logs/$TAG/y2r-8.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-6.rc"; } &
{ R2_REREAD_TXGS=27 nice -n 19 "$BIN" r2opus_y2 --nocapture --test-threads 1 > "$D/logs/$TAG/y2r-27.log" 2>&1; echo "$?" > "$D/rc/b$BATCH-7.rc"; } &
wait
n=$(ls "$D/rc/b$BATCH"-*.rc 2>/dev/null | wc -l)
if [ "$n" -ne 7 ]; then echo "BATCH $BATCH 作废：rc 文件 $n 个，派出 7 件"; exit 3; fi
for i in 1 2 3 4 5 6 7; do echo "b$BATCH-$i rc=$(cat "$D/rc/b$BATCH-$i.rc")"; done
