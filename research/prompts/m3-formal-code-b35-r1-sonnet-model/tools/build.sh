#!/usr/bin/env bash
# 只编不跑：被测的 harness 档测试目标 + --lib（日志里要有 Compiling singlefs-checker / singlefs-core / singlefs-harness）
set -u
. "$(dirname "$0")/env.sh"
cd "$S/work" && $RUNNER --no-run -p singlefs-harness --lib \
  --test checker_known_bad_images \
  --test reuse_after_raising_the_floor --test checker_narrow_invariants_and_abandoned_roots --test writable_mount_of_a_formatted_pool \
  --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement \
  --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts \
  > "$S/build.log" 2>&1
echo "exit=$?" >> "$S/build.log"
