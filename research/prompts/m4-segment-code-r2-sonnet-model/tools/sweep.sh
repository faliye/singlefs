#!/usr/bin/env bash
# 辩方腿随机历史扫描的并行驱动：sweep.sh <二进制> <比重> <种子偏移起点> <每进程几段> <进程数> <标签> <池级 checker 判哪些步> [步数]
# 日志写在 $WORK/logs/<标签>-<i>.log，退出码写在 $WORK/logs/<标签>-<i>.exit；整条经 run-with-memory-cap.sh 跑（每个进程 4G）。
set -u
BIN="$1"; WEIGHTS="$2"; START="$3"; PER="$4"; PROCS="$5"; TAG="$6"; CHECK="$7"; OPS="${8:-40}"
REPO_ROOT="${REPO_ROOT:?}"; WORK="${WORK:?}"
mkdir -p "$WORK/logs"
rm -f "$WORK/logs/$TAG"-*.log "$WORK/logs/$TAG"-*.exit
for ((i = 0; i < PROCS; i++)); do
  {
    DEFENSE_TRACE=1 DEFENSE_WEIGHTS="$WEIGHTS" DEFENSE_SEED_START=$((START + i * PER)) DEFENSE_SEED_COUNT="$PER" DEFENSE_OPS="$OPS" DEFENSE_CHECK="$CHECK" \
      nice -n 19 timeout 215 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 4G "$BIN" --ignored --nocapture --exact defense_random_sweep \
      > "$WORK/logs/$TAG-$i.log" 2>&1
    echo "$?" > "$WORK/logs/$TAG-$i.exit"
  } &
done
wait
echo "exits: $(cat "$WORK"/logs/"$TAG"-*.exit | sort | uniq -c | tr '\n' ' ')"
echo "files: $(ls "$WORK"/logs/"$TAG"-*.exit | wc -l) of $PROCS"
grep -h 'DEFENSE_SUMMARY' "$WORK"/logs/"$TAG"-*.log | awk '{for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="not_completed")nc+=a[2]; if(a[1]=="reds")r+=a[2]; if(a[1]=="count")c+=a[2]}} END{print "segments="c" not_completed="nc" reds="r}'
echo "trace_batches=$(cat "$WORK"/logs/"$TAG"-*.log | grep -c DEFENSE_TRACE) zero_relocation=$(cat "$WORK"/logs/"$TAG"-*.log | grep DEFENSE_TRACE | grep -c 'relocated=0')"
