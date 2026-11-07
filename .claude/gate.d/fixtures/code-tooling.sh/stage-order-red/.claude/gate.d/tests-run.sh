#!/usr/bin/env bash
# gate-stage: 样本：跑测试的阶段，起名 tests-run 排到了 test-environment 后面（- 小于 s）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
cargo test --release -p sample-crate
