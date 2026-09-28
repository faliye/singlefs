#!/usr/bin/env bash
# 本腿对快照自带用例与几份原型的单跑（harness 档与原型包，不含 checker 档包的测试）。用法：bash checks.sh <草稿目录> <仓根>
set -u
D=${1:?草稿目录}; R=${2:?仓根}
CAP="bash $R/research/scripts/run-with-memory-cap.sh 8G bash $R/research/scripts/capped.sh 8"
L=$D/logs
cd "$D/snapshot" || exit 2
nice -n 19 $CAP cargo test --offline --release -p opus-r3-proto --no-run > "$L/build4.log" 2>&1; echo "build4 rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --no-run \
  --test a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error \
  --test entries_after_mount_refuse_swapped_or_behind_devices \
  --test checker_known_bad_images \
  --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over \
  --test rollback_by_a_forward_publish \
  --test publish_order_matches_litmus > "$L/build5.log" 2>&1; echo "build5 rc=$?"
nice -n 19 $CAP cargo test --offline --release -p opus-r3-proto --test opus_r3_x6_system_configuration_fields_core_and_checker -- --nocapture --test-threads 2 > "$L/x6-fields.log" 2>&1; echo "x6-fields rc=$?"
nice -n 19 $CAP cargo test --offline --release -p opus-r3-proto --test opus_r3_x1_single_fault_leaves_the_pool_read_only -- --nocapture --test-threads 1 > "$L/x1-readonly.log" 2>&1; echo "x1 rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error > "$L/repo-x5-3.log" 2>&1; echo "repo-x5-3 rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test publish_order_matches_litmus > "$L/repo-litmus.log" 2>&1; echo "repo-litmus rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices -- --include-ignored > "$L/repo-entries.log" 2>&1; echo "repo-entries rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test checker_known_bad_images -- devices_whose_own_device_numbers_are_swapped > "$L/repo-i714.log" 2>&1; echo "repo-i714 rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over -- the_rollback_takes_the_inode_number_watermark > "$L/repo-watermark.log" 2>&1; echo "repo-watermark rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored --list > "$L/repo-rollback-list.log" 2>&1; echo "repo-rollback-list rc=$?"
cd "$D/mut736" || exit 2
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over -- the_rollback_takes_the_inode_number_watermark > "$L/mut736-watermark.log" 2>&1; echo "mut736-watermark rc=$?"
nice -n 19 $CAP cargo test --offline --release -p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_rollback_takes_the_watermarks > "$L/mut736-old-target.log" 2>&1; echo "mut736-old-target rc=$?"
