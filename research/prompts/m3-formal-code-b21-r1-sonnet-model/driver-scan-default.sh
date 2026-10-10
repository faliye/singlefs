#!/usr/bin/env bash
set -u
BIN=/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/target/debug/deps/b21_scan-fb72249c9aef285f
for sl in 0 1; do B21_SLACK=$sl B21_S=8 B21_J=default "$BIN" --nocapture --test-threads 1 2>&1 | grep -E 'ROW' | sed 's/^test scan ... //'; done
