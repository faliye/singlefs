#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<冻结副本根（含 crates/）> SINGLEFS_MAIN_REPO=<主仓根，取 scripts/> bash rerun.sh <空的输出目录>   （前台约 3 分钟；同一时刻只起一条 cargo；每条 cargo 经内存包装 8G、线程 7）
# 复跑正推腿报告的 R1（sha256）、R2（变异表静态判）、R3（基线 harness 用例）、R4（变异行逐条现跑，checker 档行不跑）、R5（组合变异）；R6 单独在 rerun-adhoc.sh。输出写 <输出目录>/{static,baseline,mutations,combo}.txt，compare.sh 比两次。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"; : "${SINGLEFS_MAIN_REPO:?要设 SINGLEFS_MAIN_REPO}"
out=${1:?要给一个输出目录}; here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
[ -z "$(ls -A "$out" 2>/dev/null)" ] || { echo "输出目录不空：$out" >&2; exit 2; }
mkdir -p "$out/work"; repo=$out/work/repo; mkdir -p "$repo"
cp -a "$SINGLEFS_REPO"/crates "$SINGLEFS_REPO"/Cargo.toml "$SINGLEFS_REPO"/Cargo.lock "$SINGLEFS_REPO"/litmus "$repo"/
export CARGO_TARGET_DIR=$out/work/target
S=$SINGLEFS_MAIN_REPO/scripts
# R1 R2
{ printf 'R1 sha256 OK 份数\t%s\n' "$(cd "$SINGLEFS_REPO" && sha256sum -c "$here/crates-sha256.txt" 2>&1 | grep -c ': OK$' || true)"
  python3 -I "$here/static_anchor.py" "$repo"; } > "$out/static.txt"
# R3 基线
cd "$repo"
nice -n 19 bash "$S/capped.sh" 7 cargo test --offline --no-run -p singlefs-core --lib -p singlefs-harness \
  --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten \
  --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero \
  --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero \
  --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed \
  --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version \
  --test unlink_crash_image_remounted_twice_continues_to_identical_disks \
  --test checker_known_bad_images > "$out/work/build.log" 2>&1
{ nice -n 19 bash "$S/run-with-memory-cap.sh" 4G bash "$S/capped.sh" 7 cargo test --offline -p singlefs-core --lib -- allocation_records_per_device_tests 2>&1 | grep -E '^test result'
  nice -n 19 bash "$S/run-with-memory-cap.sh" 8G bash "$S/capped.sh" 7 cargo test --offline -p singlefs-harness \
  --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten \
  --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero \
  --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero \
  --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed \
  --test read_recovery_fields_follow_the_fallback_to_the_witnessed_version \
  --test unlink_crash_image_remounted_twice_continues_to_identical_disks 2>&1 | grep -E '^test result'
  nice -n 19 bash "$S/run-with-memory-cap.sh" 8G bash "$S/capped.sh" 7 cargo test --offline -p singlefs-harness --test checker_known_bad_images -- an_intent_version_reading_the_shared_extent_root garbling_each_shared_root_pointer a_root_pointer_whose_two_copies_do_not_read_back 2>&1 | grep -E '^test result'
} | sed -E 's/finished in [0-9.]+s//' > "$out/baseline.txt"
# R4 变异行
python3 -I "$here/mutrun.py" "$repo" "$SINGLEFS_MAIN_REPO" '删除线第35批|增补 2 第 61 行|第33批T|第36批 V' | sed -E 's/ failed=.*name=/ name=/' > "$out/mutations.txt"
# R5 组合：第 33 批 T 的「只认带末条标志」行单独存活，与 C_ACCOUNT 那行一起红
python3 -I "$here/combo.py" "$repo" "$SINGLEFS_MAIN_REPO" '第33批T C694 的 C 选法退成只认带末条标志' '删除线第35批 C_ACCOUNT C 不并没施加的发布的账（根槽与末条读回全零）' | sed -E 's/ failed=.* a=/ a=/' > "$out/combo.txt"
cat "$out"/static.txt "$out"/baseline.txt "$out"/mutations.txt "$out"/combo.txt | head -120
