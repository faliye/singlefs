#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
[[ -n "${SAMPLE_ARITH_JOBS:-}" ]] || SAMPLE_ARITH_JOBS=$((default_threads * 2))
make -j "$SAMPLE_ARITH_JOBS"
