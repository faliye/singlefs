#!/usr/bin/env bash
set -u
BIN=$(ls -t /tmp/claude-1000/m3-formal-code-b21-r1-sonnet/target/debug/deps/b21_k3_witness-* | grep -v '\.d$' | head -1)
"$BIN" --nocapture 2>&1 | grep -E '^K3|test result|test k3'
