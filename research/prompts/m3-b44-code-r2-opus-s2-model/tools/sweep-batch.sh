#!/usr/bin/env bash
# 一批：起 N 件，每件扫 start..reads 里步长 STEP 的那几个读序号；退出码各写进 <草稿>/rc/b<批号>-<件号>.rc。
set -u
W=/home/fy5090/.local/share/singlefs/work/m3-b44-code-r2-opus-s2
BIN=$1; TEST=$2; BATCH=$3; STEP=$4; shift 4
mkdir -p "$W/rc" "$W/out"
rm -f "$W"/rc/b"$BATCH"-*.rc
piece=0
for start in "$@"; do
  piece=$((piece + 1))
  { ATTACK_OUTPUT_DIRECTORY="$W/out" ATTACK_READ_START="$start" ATTACK_READ_STEP="$STEP" \
      "$BIN" --exact "$TEST" --nocapture > "$W/logs/b$BATCH-$piece.log" 2>&1; echo "$?" > "$W/rc/b$BATCH-$piece.rc"; } &
done
wait
count=$(ls "$W"/rc/b"$BATCH"-*.rc 2>/dev/null | wc -l)
if [ "$count" -ne "$piece" ]; then echo "batch $BATCH: rc files $count != pieces $piece, batch void"; exit 3; fi
for i in $(seq 1 "$piece"); do echo "piece $i rc=$(cat "$W/rc/b$BATCH-$i.rc")"; done
