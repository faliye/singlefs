#!/usr/bin/env bash
# 一批并行件：batch.sh <工作目录> <批号> <件表文件>；件表一行一件：<日志名> <原型用例名> [<环境变量>…]。
# 每件经 run-with-memory-cap.sh（3G、线程 3），退出码写进 <工作目录>/runs/b<批号>-<件号>.rc；开跑前删掉这一批的 .rc，跑完数一遍。
set -uo pipefail
ROOT=/home/fy5090/code/singlefs
WORK=${1:?}; BATCH=${2:?}; JOBS=${3:?}
BIN=${ATK_BIN:?要 ATK_BIN}
OUT=$WORK/runs
mkdir -p "$OUT"
rm -f "$OUT"/b"$BATCH"-*.rc
n=0
while read -r log name envs; do
  [ -z "$log" ] && continue
  n=$((n+1))
  { bash "$ROOT"/scripts/run-with-memory-cap.sh 3G env $envs nice -n 19 bash "$ROOT"/scripts/capped.sh 3 "$BIN" --ignored --nocapture --exact "$name" > "$OUT/$log" 2>&1; echo "$?" > "$OUT"/b"$BATCH"-$n.rc; } &
done < "$JOBS"
wait
got=$(ls "$OUT"/b"$BATCH"-*.rc 2>/dev/null | wc -l)
if [ "$got" != "$n" ]; then echo "批 $BATCH 作废：派了 $n 件，.rc $got 份"; exit 1; fi
for i in $(seq 1 "$n"); do echo "b$BATCH-$i rc=$(cat "$OUT"/b"$BATCH"-$i.rc)"; done
