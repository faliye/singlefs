#!/usr/bin/env bash
# 第一批原型：重读一次、写零三个后端与真 errno、两张扫描表的估时（只打状态数 / 单发布小表）。
set -u
cd /tmp/claude-1000/m3-code-r1-opus/work2
export CARGO_TARGET_DIR=/tmp/claude-1000/m3-code-r1-opus/target2
L=/tmp/claude-1000/m3-code-r1-opus/logs
cargo test --offline -p m3r1-opus-proto --test z2_reread_one_transient_root_slot_read_error -- --nocapture > $L/reread-orig.log 2>&1; echo "reread rc=$?"
cargo test --offline -p m3r1-opus-proto --test z3_zero_fill_on_the_file_backend_against_the_other_backends -- --nocapture --test-threads=1 > $L/z3-backends.log 2>&1; echo "z3 rc=$?"
M3R1_ENUMERATE=0 cargo test --release --offline -p m3r1-opus-proto --test z2_every_crash_state_of_a_publish_then_a_writable_mount -- --nocapture > $L/crash-counts.log 2>&1; echo "crash-counts rc=$?"
M3R1_MAX_PUBLISHES=1 cargo test --release --offline -p m3r1-opus-proto --test z2_stale_snapshot_from_any_publish_of_a_history -- --nocapture > $L/stale-m1.log 2>&1; echo "stale-m1 rc=$?"
