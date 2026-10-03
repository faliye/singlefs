#!/usr/bin/env bash
# 用法：run-full-par.sh <批号> <错实现名|BASE>
# 套上错实现 → 编 harness 全部测试目标与 core --lib → harness 每个测试二进制（名字带 r2opus 的不跑）分 4 路并行、每路 --test-threads 3、--skip r2opus；
# 再跑 core --lib。每个二进制的退出码写进 rc 目录；跑完从冻结副本写回原样。
set -u
D=/tmp/claude-1000/m3-code-r2-opus; W=$D/work; F=/tmp/claude-1000/m3-code-r2/tree
export CARGO_TARGET_DIR=$D/target
batch=$1; name=$2
L=$D/logs/full/$name; mkdir -p "$L"
if [ "$name" != BASE ]; then python3 "$D/model/wrongrun.py" apply "$W" "$F" "$name" || exit 3; fi
(cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --tests --no-run > "$L/build.log" 2>&1 && nice -n 19 cargo test --offline -p singlefs-core --lib --no-run >> "$L/build.log" 2>&1)
brc=$?
if [ $brc -ne 0 ]; then echo "FULL $name build rc=$brc"; [ "$name" != BASE ] && python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"; exit 4; fi
ls "$W/crates/singlefs-harness/tests/"*.rs | xargs -n1 basename | sed 's/\.rs$//' | grep -v '^r2opus' > "$L/targets.txt"
: > "$L/binaries.txt"
while read -r t; do b=$(ls -t "$CARGO_TARGET_DIR/debug/deps/$t"-* 2>/dev/null | grep -v '\.d$' | head -1); echo "$t $b" >> "$L/binaries.txt"; done < "$L/targets.txt"
rm -rf "$D/rc/f$batch"; mkdir -p "$D/rc/f$batch"
cd "$W/crates/singlefs-harness"
for s in 0 1 2 3; do
  { i=0; while read -r t b; do
      if [ $((i % 4)) -eq $s ]; then
        nice -n 19 "$b" --skip r2opus --test-threads 3 > "$L/$t.log" 2>&1; echo "$?" > "$D/rc/f$batch/$t.rc"
      fi
      i=$((i+1))
    done < "$L/binaries.txt"; echo done > "$D/rc/f$batch/stream-$s.done"; } &
done
wait
cd "$W" && nice -n 19 cargo test --offline -p singlefs-core --lib > "$L/core-lib.log" 2>&1; echo "$?" > "$D/rc/f$batch/core-lib.rc"
streams=$(ls "$D/rc/f$batch"/stream-*.done | wc -l)
nb=$(wc -l < "$L/binaries.txt"); nrc=$(ls "$D/rc/f$batch"/*.rc | grep -vc core-lib)
nonzero=$(for f in "$D/rc/f$batch"/*.rc; do [ "$(cat "$f")" != 0 ] && basename "$f" .rc; done | tr '\n' ' ')
echo "FULL $name streams=$streams binaries=$nb rc_files=$nrc nonzero=[$nonzero] core_lib=$(grep -o 'test result:.*passed' "$L/core-lib.log")"
echo "FULL $name failed_tests=[$(cat "$L"/*.log | grep -o '^test [A-Za-z_0-9:]* \.\.\. FAILED' | awk '{print $2}' | tr '\n' ' ')]"
[ "$name" != BASE ] && python3 "$D/model/wrongrun.py" restore "$W" "$F" "$name"
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r2opus
echo DONE
