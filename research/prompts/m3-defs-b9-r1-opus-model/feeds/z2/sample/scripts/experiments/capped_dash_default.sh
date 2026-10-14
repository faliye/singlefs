#!/usr/bin/env bash
# 对照：同一个变量写 :- 默认 8
cargo test -- --test-threads "${RUST_TEST_THREADS:-8}"
