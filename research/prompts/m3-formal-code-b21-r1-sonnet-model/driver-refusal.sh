#!/usr/bin/env bash
set -u
BIN=/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/target/debug/deps/writable_mount_refusal_on_a_crash_state_without_read_faults-570f5697dea4fee3
"$BIN" --nocapture 2>&1 | grep -E '^test |test result|PROBE2' | cut -c1-300
