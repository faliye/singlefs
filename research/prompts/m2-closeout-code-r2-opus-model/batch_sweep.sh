#!/usr/bin/env bash
# Y1/Y7 单故障序号扫：两件并行（四个小入口全扫；崩了再可写挂载按取样扫读）。用法：bash batch_sweep.sh <批号> <日志目录> <快照副本目录> <target 目录>
set -u
B=${1:?批号}; L=${2:?日志目录}; S=${3:?快照副本}; T=${4:?target}
R=<仓根>
export CARGO_TARGET_DIR="$T"
cd "$S"
CAP="bash $R/research/scripts/capped.sh 2 bash $R/research/scripts/run-with-memory-cap.sh 8G"
rm -f "$L"/b"$B"-*.rc
{ OPUS_Y1_ENTRIES=session_overwrite,unmount,raise_floor_to_the_admission_ceiling,rollback_to_1_4 nice -n 19 $CAP cargo test -p singlefs-harness --release --test opus_r2_y1_y7_single_fault_ordinal_sweep -- --nocapture > "$L/b$B-1.log" 2>&1; echo "$?" > "$L/b$B-1.rc"; } &
{ OPUS_Y1_ENTRIES=crash_remount OPUS_Y1_STRIDE=4999 OPUS_Y1_READ_HEAD=600 OPUS_Y1_READ_TAIL=1500 nice -n 19 $CAP cargo test -p singlefs-harness --release --test opus_r2_y1_y7_single_fault_ordinal_sweep -- --nocapture > "$L/b$B-2.log" 2>&1; echo "$?" > "$L/b$B-2.rc"; } &
wait
n=$(ls "$L"/b"$B"-*.rc 2>/dev/null | wc -l)
if [ "$n" -ne 2 ]; then echo "批 $B 作废：rc 文件 $n 个，派出 2 件"; exit 1; fi
for i in 1 2; do echo "b$B-$i rc=$(cat "$L/b$B-$i.rc")"; done
