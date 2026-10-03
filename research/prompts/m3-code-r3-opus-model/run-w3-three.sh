#!/usr/bin/env bash
# 用法：run-w3-three.sh <批号> <错实现名|BASE>…   W3 三盘副本（work3）：套上错实现 → 编 → 归属 0,1,2 下切在根槽之前，txg 分三件并行扫 → 写回。整条经内存包装起。
set -u
D=/tmp/claude-1000/m3-code-r3-opus; W=$D/work3; F=/tmp/claude-1000/m3-code-r3/tree
export CARGO_TARGET_DIR=$D/target3 R3_THREE_DEVICES=1 R3_LAYOUT=${R3_LAYOUT:-0,1,2}
batch=$1; shift
for name in "$@"; do
  L=$D/logs/w3/$name-${R3_LAYOUT//,/}; mkdir -p "$L"
  if [ "$name" != BASE ]; then python3 "$D/model/wrongrun3.py" apply "$W" "$F" "$name" || { echo "W3 $name apply-failed"; continue; }; fi
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test r3opus_w3_three --no-run > "$L/build.log" 2>&1)
  brc=$?
  if [ $brc -ne 0 ]; then echo "W3 $name build rc=$brc"; [ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"; continue; fi
  BIN=$(ls -t "$CARGO_TARGET_DIR"/debug/deps/r3opus_w3_three-* | grep -v '\.d$' | head -1)
  rm -f "$D/rc/t$batch"-*.rc
  cd "$W/crates/singlefs-harness"
  n=0
  for txgs in 6,7,8 25,27 26,32; do n=$((n+1))
    { R3_W3_TXGS=$txgs nice -n 19 "$BIN" r3opus_w3_three_device_sweep --exact --nocapture --test-threads 1 > "$L/sweep-$n.log" 2>&1; echo "$?" > "$D/rc/t$batch-$n.rc"; } &
  done
  wait
  got=$(ls "$D/rc/t$batch"-*.rc 2>/dev/null | wc -l)
  if [ "$got" -ne "$n" ]; then echo "W3 $name 作废：rc $got 个，派出 $n 件"; else
    for i in $(seq 1 $n); do echo "W3 $name piece-$i rc=$(cat "$D/rc/t$batch-$i.rc")"; done; fi
  cd "$D"
  [ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"
  batch=$((batch+1))
done
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r3opus
echo DONE
