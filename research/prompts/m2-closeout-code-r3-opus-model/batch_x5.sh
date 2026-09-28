#!/usr/bin/env bash
# X5 ① 批跑（m2-closeout-code-r3 云端攻方腿）：用法 bash batch_x5.sh <批号 2|3> <快照副本目录> <日志目录> <仓根>
# 批 2：件 1–3 第二轮那一套（不展开崩溃态），种子 1–6 两个一件、读序号步长 7（末 300 次逐个）；件 4：种子 1–6 的回退 / 抬 F / 卸载不注入、
#       立刻掉电全量展开，接着种子 1 的回退逐个写序号、屏障序号注入之后立刻掉电全量展开。
# 批 3：长历史档（OPUS_R3_LONG=1，种子 7–10）。件 1–2 第二轮那一套；件 3：回退 / 抬 F / 卸载不注入全量展开，
#       接着种子 7–10 的抬 F 逐个屏障序号注入之后全量展开；件 4：短历史种子 1、3、4、6 的回退逐个写序号、屏障序号注入之后全量展开
#       （批 2 件 4 后半截的重跑：那一次 oracle 的版本表漏了「只靠记录施加出来的那一版」，4 段误报）。
set -u
B=${1:?批号}; S=${2:?快照副本}; L=${3:?日志目录}; R=${4:?仓根}
cd "$S" || exit 2
T="--offline --release -p opus-r3-proto --test opus_r3_x5_resample_single_fault_then_power_loss -- --nocapture"
CAP="bash $R/research/scripts/run-with-memory-cap.sh 8G bash $R/research/scripts/capped.sh"
rm -f "$L"/b"$B"-*.rc
case "$B" in
  2)
    { OPUS_R3_SEEDS=1,2 OPUS_R3_ENUMERATE=0 OPUS_R3_READ_STRIDE=7 nice -n 19 $CAP 1 cargo test $T > "$L/x5a-1.log" 2>&1; echo "$?" > "$L/b$B-1.rc"; } &
    { OPUS_R3_SEEDS=3,4 OPUS_R3_ENUMERATE=0 OPUS_R3_READ_STRIDE=7 nice -n 19 $CAP 1 cargo test $T > "$L/x5a-2.log" 2>&1; echo "$?" > "$L/b$B-2.rc"; } &
    { OPUS_R3_SEEDS=5,6 OPUS_R3_ENUMERATE=0 OPUS_R3_READ_STRIDE=7 nice -n 19 $CAP 1 cargo test $T > "$L/x5a-3.log" 2>&1; echo "$?" > "$L/b$B-3.rc"; } &
    { OPUS_R3_SEEDS=1,2,3,4,5,6 OPUS_R3_ENTRIES=rollback,raise_floor_to_the_admission_ceiling,unmount OPUS_R3_KINDS=none nice -n 19 $CAP 5 cargo test $T > "$L/x5b.log" 2>&1; rc_b=$?; OPUS_R3_SEEDS=1 OPUS_R3_ENTRIES=rollback OPUS_R3_KINDS=write,barrier nice -n 19 $CAP 5 cargo test $T > "$L/x5c.log" 2>&1; rc_c=$?; echo "$rc_b $rc_c" > "$L/b$B-4.rc"; } &
    pieces=4 ;;
  3)
    { OPUS_R3_LONG=1 OPUS_R3_SEEDS=7,8 OPUS_R3_ENUMERATE=0 OPUS_R3_READ_STRIDE=7 nice -n 19 $CAP 1 cargo test $T > "$L/x5d-1.log" 2>&1; echo "$?" > "$L/b$B-1.rc"; } &
    { OPUS_R3_LONG=1 OPUS_R3_SEEDS=9,10 OPUS_R3_ENUMERATE=0 OPUS_R3_READ_STRIDE=7 nice -n 19 $CAP 1 cargo test $T > "$L/x5d-2.log" 2>&1; echo "$?" > "$L/b$B-2.rc"; } &
    { OPUS_R3_LONG=1 OPUS_R3_SEEDS=7,8,9,10 OPUS_R3_ENTRIES=rollback,raise_floor_to_the_admission_ceiling,unmount OPUS_R3_KINDS=none nice -n 19 $CAP 3 cargo test $T > "$L/x5e.log" 2>&1; rc_e=$?; OPUS_R3_LONG=1 OPUS_R3_SEEDS=7,8,9,10 OPUS_R3_ENTRIES=raise_floor_to_the_admission_ceiling OPUS_R3_KINDS=barrier OPUS_R3_BUDGET=300000 nice -n 19 $CAP 3 cargo test $T > "$L/x5f.log" 2>&1; rc_f=$?; echo "$rc_e $rc_f" > "$L/b$B-3.rc"; } &
    { OPUS_R3_SEEDS=1,3,4,6 OPUS_R3_ENTRIES=rollback OPUS_R3_KINDS=write,barrier nice -n 19 $CAP 3 cargo test $T > "$L/x5g.log" 2>&1; echo "$?" > "$L/b$B-4.rc"; } &
    pieces=4 ;;
  *) echo "没有批 $B"; exit 2 ;;
esac
wait
n=$(ls "$L"/b"$B"-*.rc 2>/dev/null | wc -l)
if [ "$n" -ne "$pieces" ]; then echo "批 $B 作废：rc 文件 $n 个，派出 $pieces 件"; exit 1; fi
for i in $(seq 1 "$pieces"); do echo "b$B-$i rc=$(cat "$L/b$B-$i.rc")"; done
