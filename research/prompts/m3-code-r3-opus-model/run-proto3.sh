#!/usr/bin/env bash
# 用法：run-proto3.sh <批号> <错实现名|BASE> <ru|rgc|w3|all>
# 套上错实现 → 编原型 r3opus_entries_proto → 按 txg 分件并行跑原型里的扫描（每件 --test-threads 1）→ 写回原样。整条脚本经内存包装起。
set -u
D=/tmp/claude-1000/m3-code-r3-opus; W=$D/work; F=/tmp/claude-1000/m3-code-r3/tree
export CARGO_TARGET_DIR=$D/target
batch=$1; name=$2; what=$3
L=$D/logs/proto/$name; mkdir -p "$L"
if [ "$name" != BASE ]; then python3 "$D/model/wrongrun3.py" apply "$W" "$F" "$name" || exit 3; fi
(cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test r3opus_entries_proto --no-run > "$L/build.log" 2>&1)
brc=$?
if [ $brc -ne 0 ]; then echo "PROTO $name build rc=$brc"; [ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"; exit 4; fi
BIN=$(ls -t "$CARGO_TARGET_DIR"/debug/deps/r3opus_entries_proto-* | grep -v '\.d$' | head -1)
rm -f "$D/rc/b$batch"-*.rc
cd "$W/crates/singlefs-harness"
n=0
if [ "$what" = ru ] || [ "$what" = all ]; then
  for txg in 6 7 8; do n=$((n+1))
    { R3_TXGS=$txg nice -n 19 "$BIN" r3opus_ru_one_corrupted_read_of_the_publish_root_slot --exact --nocapture --test-threads 1 > "$L/ru-$txg.log" 2>&1; echo "$?" > "$D/rc/b$batch-$n.rc"; } &
  done
fi
if [ "$what" = rgc ] || [ "$what" = all ]; then
  for txg in 8 26 29 32; do n=$((n+1))
    { R3_RGC_TXGS=$txg nice -n 19 "$BIN" r3opus_rgc_region_two_through_every_entry --exact --nocapture --test-threads 1 > "$L/rgc-$txg.log" 2>&1; echo "$?" > "$D/rc/b$batch-$n.rc"; } &
  done
fi
if [ "$what" = w3 ] || [ "$what" = all ]; then
  n=$((n+1))
  { nice -n 19 "$BIN" r3opus_w3_ --nocapture --test-threads 1 > "$L/w3.log" 2>&1; echo "$?" > "$D/rc/b$batch-$n.rc"; } &
fi
wait
got=$(ls "$D/rc/b$batch"-*.rc 2>/dev/null | wc -l)
if [ "$got" -ne "$n" ]; then echo "PROTO $name 作废：rc $got 个，派出 $n 件"; else
  for i in $(seq 1 $n); do echo "PROTO $name piece-$i rc=$(cat "$D/rc/b$batch-$i.rc")"; done; fi
cd "$D"
[ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r3opus
echo DONE
