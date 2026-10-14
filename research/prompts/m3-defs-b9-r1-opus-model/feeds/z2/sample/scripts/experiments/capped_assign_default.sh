#!/usr/bin/env bash
# capped.sh 设的变量，没经 capped.sh 起时 := 赋 8
: "${RUST_TEST_THREADS:=8}"
cargo test -- --test-threads "$RUST_TEST_THREADS"
