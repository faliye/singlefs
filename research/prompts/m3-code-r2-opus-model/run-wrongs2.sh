#!/usr/bin/env bash
# 用法：run-wrongs2.sh <批号起> <错实现名|BASE:模式[+模式…]>…
# 模式：y3（同 run-wrongs.sh）| y2t（恰好连坏两次，四个 txg）| h5（长度 0 三后端原型）| full（harness 全包 --skip r2opus，加 core --lib）
# BASE 不套错实现，跑原样。全部在攻方副本里；跑完从冻结副本写回原样。
set -u
D=/tmp/claude-1000/m3-code-r2-opus
W=$D/work
F=/tmp/claude-1000/m3-code-r2/tree
export CARGO_TARGET_DIR=$D/target
batch=$1; shift
for spec in "$@"; do
  name=${spec%%:*}; modes=${spec#*:}
  L=$D/logs/wrong/$name; mkdir -p "$L"
  if [ "$name" != BASE ]; then python3 "$D/model/wrongrun.py" apply "$W" "$F" "$name" || { echo "WRONG $name apply-failed"; continue; }; fi
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --tests --no-run > "$L/build2.log" 2>&1 && nice -n 19 cargo test --offline -p singlefs-core --lib --no-run >> "$L/build2.log" 2>&1)
  brc=$?
  if [ $brc -ne 0 ]; then echo "WRONG $name build rc=$brc"; [ "$name" != BASE ] && python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"; continue; fi
  BIN=$(ls -t "$CARGO_TARGET_DIR"/debug/deps/r2opus_entries_proto-* | grep -v '\.d$' | head -1)
  for mode in ${modes//+/ }; do
    rm -f "$D/rc/x$batch"-*.rc
    expected=0
    cd "$W/crates/singlefs-harness"
    case $mode in
      y3)
        n=0
        for range in 6..14 15..23 24..32; do
          for rb in first prev; do
            n=$((n+1))
            if [ $rb = prev ]; then extra="R2_ROLLBACK_PREV=1 R2_ENTRIES=rollback"; else extra=""; fi
            { env R2_CUTS=before R2_TXGS=$range $extra nice -n 19 "$BIN" r2opus_y3 --nocapture --test-threads 1 > "$L/y3-$range-$rb.log" 2>&1; echo "$?" > "$D/rc/x$batch-$n.rc"; } &
          done
        done
        expected=6 ;;
      y2t)
        n=0
        for txg in 6 7 8 27; do
          n=$((n+1))
          { R2_REREAD_TXGS=$txg nice -n 19 "$BIN" r2opus_y2_exactly --nocapture --test-threads 1 > "$L/y2t-$txg.log" 2>&1; echo "$?" > "$D/rc/x$batch-$n.rc"; } &
        done
        expected=4 ;;
      entries)
        (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices > "$L/entries.log" 2>&1); echo "WRONG $name entries rc=$? $(grep -o 'test result:.*' "$L/entries.log" | head -1) failed=[$(grep -o '^test [a-z_0-9]* \.\.\. FAILED' "$L/entries.log" | awk '{print $2}' | tr '\n' ' ')]" ;;
      h5)
        (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test r2opus_zero_length_on_three_backends --test sparse_devices_refuse_requests_outside_the_device_like_the_file_backend -- --nocapture > "$L/h5.log" 2>&1); echo "WRONG $name h5 rc=$? $(grep -o 'H5-SUMMARY.*' "$L/h5.log") $(grep -c 'test result: ok' "$L/h5.log") ok-binaries" ;;
      full)
        (cd "$W" && nice -n 19 cargo test --offline --no-fail-fast -p singlefs-harness -- --skip r2opus > "$L/full-harness.log" 2>&1); hrc=$?
        (cd "$W" && nice -n 19 cargo test --offline --no-fail-fast -p singlefs-core --lib > "$L/full-core.log" 2>&1); crc=$?
        echo "WRONG $name full harness_rc=$hrc core_rc=$crc results=$(cat "$L/full-harness.log" "$L/full-core.log" | grep -c '^test result:') failed_results=$(cat "$L/full-harness.log" "$L/full-core.log" | grep -c '^test result: FAILED') failed=[$(cat "$L/full-harness.log" "$L/full-core.log" | grep -o '^test [A-Za-z_0-9:]* \.\.\. FAILED' | awk '{print $2}' | tr '\n' ' ')]" ;;
    esac
    wait
    got=$(ls "$D/rc/x$batch"-*.rc 2>/dev/null | wc -l)
    if [ "$got" -ne "$expected" ]; then echo "WRONG $name $mode 作废：rc $got 个，派出 $expected 件"; else
      for i in $(seq 1 $expected); do echo "WRONG $name $mode-$i rc=$(cat "$D/rc/x$batch-$i.rc")"; done; fi
    batch=$((batch+1))
    cd "$D"
  done
  [ "$name" != BASE ] && python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"
done
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r2opus
echo DONE
