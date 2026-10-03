#!/usr/bin/env bash
# 用法：run-wrongs.sh <批号起> <错实现名:跑什么>…   跑什么 = y3（Y3 只切根槽之前、四个入口、txg 6..32，回退目标两种）| y2r（Y2R 四个 txg）| none
# 每个错实现：套上 → 编 → 跑 entries 整个文件 → 跑扫描 → 从冻结副本写回原样。全部在攻方副本里。
set -u
D=/tmp/claude-1000/m3-code-r2-opus
W=$D/work
F=/tmp/claude-1000/m3-code-r2/tree
export CARGO_TARGET_DIR=$D/target
batch=$1; shift
for spec in "$@"; do
  name=${spec%%:*}; what=${spec#*:}
  L=$D/logs/wrong/$name; mkdir -p "$L"
  python3 "$D/model/wrongrun.py" apply "$W" "$F" "$name" || { echo "WRONG $name apply-failed"; continue; }
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test r2opus_entries_proto --test entries_after_mount_refuse_swapped_or_behind_devices --no-run > "$L/build.log" 2>&1)
  brc=$?
  if [ $brc -ne 0 ]; then echo "WRONG $name build rc=$brc"; python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"; continue; fi
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices > "$L/entries.log" 2>&1)
  erc=$?
  echo "WRONG $name entries rc=$erc $(grep -o 'test result:.*' "$L/entries.log" | head -1) failed=[$(grep -o '^test [a-z_0-9]* \.\.\. FAILED' "$L/entries.log" | awk '{print $2}' | tr '\n' ' ')]"
  BIN=$(ls -t "$CARGO_TARGET_DIR"/debug/deps/r2opus_entries_proto-* | grep -v '\.d$' | head -1)
  rm -f "$D/rc/w$batch"-*.rc
  cd "$W/crates/singlefs-harness"
  if [ "$what" = y3 ]; then
    n=0
    for range in 6..14 15..23 24..32; do
      for rb in first prev; do
        n=$((n+1))
        if [ $rb = prev ]; then extra="R2_ROLLBACK_PREV=1 R2_ENTRIES=rollback"; else extra=""; fi
        { env R2_CUTS=before R2_TXGS=$range $extra nice -n 19 "$BIN" r2opus_y3 --nocapture --test-threads 1 > "$L/y3-$range-$rb.log" 2>&1; echo "$?" > "$D/rc/w$batch-$n.rc"; } &
      done
    done
    expected=6
  elif [ "$what" = y2r ]; then
    n=0
    for txg in 6 7 8 27; do
      n=$((n+1))
      { R2_REREAD_TXGS=$txg nice -n 19 "$BIN" r2opus_y2 --nocapture --test-threads 1 > "$L/y2r-$txg.log" 2>&1; echo "$?" > "$D/rc/w$batch-$n.rc"; } &
    done
    expected=4
  else
    expected=0
  fi
  wait
  got=$(ls "$D/rc/w$batch"-*.rc 2>/dev/null | wc -l)
  if [ "$got" -ne "$expected" ]; then echo "WRONG $name 扫描作废：rc $got 个，派出 $expected 件"; else
    for i in $(seq 1 $expected); do echo "WRONG $name sweep-$i rc=$(cat "$D/rc/w$batch-$i.rc")"; done; fi
  cd "$D"
  python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"
  batch=$((batch+1))
done
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r2opus
echo DONE
