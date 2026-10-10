#!/usr/bin/env bash
# 只编不跑：被测的 harness 档测试目标 + --lib（日志里要有 Compiling singlefs-checker / singlefs-core / singlefs-harness）
set -u
. "$(dirname "$0")/env.sh"
cd "$S/work" && $RUNNER --no-run -p singlefs-harness --lib \
  --test checker_known_bad_images \
  --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts \
  --test pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts \
  --test mount_transparency_judged_field_by_field_after_a_writable_mount \
  --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader \
  --test pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader \
  --test reuse_after_raising_the_floor --test checker_narrow_invariants_and_abandoned_roots --test writable_mount_of_a_formatted_pool \
  > "$S/build.log" 2>&1
echo "exit=$?" >> "$S/build.log"
