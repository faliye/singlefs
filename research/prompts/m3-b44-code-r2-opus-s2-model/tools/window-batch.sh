#!/usr/bin/env bash
# 一批：ATTACK_WINDOW=$3 下起 $4 件，件号 i 扫只进内存那一趟里第 i-1、i-1+件数 ……个读序号；退出码各写进 <草稿>/rc/b<批号>-<件号>.rc。
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-b44-code-r2-opus-s2
BIN=$1; BATCH=$2; WINDOW=$3; PIECES=$4
mkdir -p "$W/rc" "$W/out"
rm -f "$W"/rc/b"$BATCH"-*.rc
for piece in $(seq 1 "$PIECES"); do
  { ATTACK_OUTPUT_DIRECTORY="$W/out" ATTACK_WINDOW="$WINDOW" ATTACK_READ_START="$((piece - 1))" ATTACK_READ_STEP="$PIECES" \
      "$BIN" --exact attack_s2_read_fault_window_inside_the_memory_walk --nocapture > "$W/logs/b$BATCH-$piece.log" 2>&1; echo "$?" > "$W/rc/b$BATCH-$piece.rc"; } &
done
wait
count=$(ls "$W"/rc/b"$BATCH"-*.rc 2>/dev/null | wc -l)
if [ "$count" -ne "$PIECES" ]; then echo "batch $BATCH: rc files $count != pieces $PIECES, batch void"; exit 3; fi
for i in $(seq 1 "$PIECES"); do echo "piece $i rc=$(cat "$W/rc/b$BATCH-$i.rc")"; done
