#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<冻结副本根> SINGLEFS_MAIN_REPO=<主仓根> bash rerun-adhoc.sh <空的输出目录>   （前台约 2 分钟）
# R6：allocator.rs 里 `protected.extend(self.placements_named_by_unapplied_journal_records());` 整句删掉，跑 core lib 与 10 个相关 harness 二进制：预期 nfailed=0（没有用例钉它；变异表里也没有行锚它）。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"; : "${SINGLEFS_MAIN_REPO:?要设 SINGLEFS_MAIN_REPO}"
out=${1:?要给一个输出目录}; here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
[ -z "$(ls -A "$out" 2>/dev/null)" ] || { echo "输出目录不空：$out" >&2; exit 2; }
mkdir -p "$out/work"; repo=$out/work/repo; mkdir -p "$repo"
cp -a "$SINGLEFS_REPO"/crates "$SINGLEFS_REPO"/Cargo.toml "$SINGLEFS_REPO"/Cargo.lock "$SINGLEFS_REPO"/litmus "$repo"/
export CARGO_TARGET_DIR=$out/work/target
printf 'R6 变异表里锚 protected.extend 的行数\t%s\n' "$(grep -c 'protected.extend' "$repo/crates/mutations.tsv" || true)" > "$out/adhoc.txt"
python3 -I "$here/adhoc.py" "$repo" "$SINGLEFS_MAIN_REPO" crates/singlefs-core/src/allocator.rs '        protected.extend(self.placements_named_by_unapplied_journal_records());\n' '' \
  -p singlefs-core --lib -p singlefs-harness \
  --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten \
  --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero \
  --test reuse_after_raising_the_floor --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring \
  --test raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic --test unreadable_abandoned_root_slot \
  --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping \
  --test abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots \
  --test an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor --test checker_narrow_invariants_and_abandoned_roots \
  | sed -E 's/ failed=.* nfailed=/ nfailed=/' >> "$out/adhoc.txt"
cat "$out/adhoc.txt"
