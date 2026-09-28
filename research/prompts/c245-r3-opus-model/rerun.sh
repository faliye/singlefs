#!/usr/bin/env bash
# C245 第三轮云端攻方腿的探针复跑（副本上跑，不动仓）。用法：bash rerun.sh <草稿目录>
# 甲：仓的工作树拷一份，放进探针，跑探针；乙-J：再拷一份，打上 yi-j-*.diff（落后支改看记录），跑钉落后支的现有用例与探针。
# 探针断言的是打中的结局：甲那份 7 条全绿 = 打中都在；乙-J 那份里探针 A1 / A2 预期红（落后支不再误拒）。
set -u
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$MODEL/../../.." && pwd)"
L="${1:?给一个草稿目录}"
mkdir -p "$L"
for arm in jia yij; do
  rsync -a --delete --exclude target --exclude .git "$REPO/" "$L/repo-$arm/"
  cp "$MODEL/c245_r3_opus_probe.rs" "$L/repo-$arm/crates/singlefs-harness/tests/"
done
(cd "$L/repo-yij" && patch -p1 < "$MODEL/yi-j-mount.diff" && patch -p1 < "$MODEL/yi-j-journal.diff") || exit 1
TESTS="entries_after_mount_refuse_swapped_or_behind_devices fsync_drop_and_devices_without_the_selected_version acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration c245_r3_opus_probe"
for arm in jia yij; do
  for t in $TESTS; do
    (cd "$L/repo-$arm" && CARGO_TARGET_DIR="$L/target-$arm" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G \
      bash research/scripts/capped.sh 4 cargo test --offline -p singlefs-harness --test "$t" -- --nocapture) > "$L/$arm-$t.log" 2>&1
    echo "$arm $t exit $?"
  done
done
grep -h "^probe=" "$L"/jia-c245_r3_opus_probe.log | sort
