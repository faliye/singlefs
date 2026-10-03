#!/usr/bin/env bash
# 用法：run-wrongs3.sh <名|BASE>…  每个：套上错实现（BASE 不套）→ 编 → 跑 entries 整个文件、稀疏盘对拍那份、core --lib → 从冻结副本写回原样。
# 全部在攻方副本里；整条脚本经内存包装起。
set -u
D=/tmp/claude-1000/m3-code-r3-opus; W=$D/work; F=/tmp/claude-1000/m3-code-r3/tree
export CARGO_TARGET_DIR=$D/target
for name in "$@"; do
  L=$D/logs/w2/$name; mkdir -p "$L"
  if [ "$name" != BASE ]; then python3 "$D/model/wrongrun3.py" apply "$W" "$F" "$name" || { echo "WRONG $name apply-failed"; continue; }; fi
  s=$(date +%s)
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices --test sparse_devices_refuse_requests_outside_the_device_like_the_file_backend --test r3opus_entries_proto --no-run > "$L/build.log" 2>&1 \
     && nice -n 19 cargo test --offline -p singlefs-core --lib --no-run >> "$L/build.log" 2>&1)
  brc=$?
  if [ $brc -ne 0 ]; then echo "WRONG $name build rc=$brc"; [ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"; continue; fi
  for t in entries_after_mount_refuse_swapped_or_behind_devices sparse_devices_refuse_requests_outside_the_device_like_the_file_backend; do
    (cd "$W" && nice -n 19 cargo test --offline -p singlefs-harness --test $t > "$L/$t.log" 2>&1); rc=$?
    echo "WRONG $name $t rc=$rc $(grep -o 'test result:.*' "$L/$t.log" | head -1 | cut -c1-90) failed=[$(grep -o '^test [A-Za-z_0-9:]* \.\.\. FAILED' "$L/$t.log" | awk '{print $2}' | tr '\n' ' ')]"
  done
  (cd "$W" && nice -n 19 cargo test --offline -p singlefs-core --lib > "$L/core-lib.log" 2>&1); rc=$?
  echo "WRONG $name core-lib rc=$rc $(grep -o 'test result:.*' "$L/core-lib.log" | head -1 | cut -c1-90) failed=[$(grep -o '^test [A-Za-z_0-9:]* \.\.\. FAILED' "$L/core-lib.log" | awk '{print $2}' | tr '\n' ' ')]"
  echo "WRONG $name seconds=$(( $(date +%s) - s ))"
  [ "$name" != BASE ] && python3 "$D/model/wrongrun3.py" restore "$W" "$F" "$name"
done
diff -rq --exclude=target "$F/crates" "$W/crates" | grep -v r3opus
echo DONE
