#!/usr/bin/env bash
# 单行 if：没设就给 8
if [[ -z "${SAMPLE_IF_THREADS:-}" ]]; then SAMPLE_IF_THREADS=8; fi
cargo build -j "$SAMPLE_IF_THREADS"
