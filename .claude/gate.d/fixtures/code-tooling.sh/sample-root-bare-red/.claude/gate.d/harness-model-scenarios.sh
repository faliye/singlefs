#!/usr/bin/env bash
# gate-stage: 样本：跑测试的阶段，起名排在 harness-test-environment 前面
ROOT="${1:-.}"
cd "$ROOT" || exit 2
cargo test --release -p sample-crate
