#!/usr/bin/env bash
# 对照：printenv 读、没设回退 8（M2 认成读）
workers="$(printenv SAMPLE_PRINTENV_THREADS || echo 8)"
cargo build -j "$workers"
