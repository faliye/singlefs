#!/usr/bin/env bash
# ${X:=8}：没设就赋 8（与 ${X:-8} 同一个默认，只是顺手赋回变量）
: "${SAMPLE_FILL_THREADS:=8}"
cargo build -j "$SAMPLE_FILL_THREADS"
