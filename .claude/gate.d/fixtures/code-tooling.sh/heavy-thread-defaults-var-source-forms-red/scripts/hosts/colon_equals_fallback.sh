#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)" || true
: "${default_threads:=8}"
[[ -n "${SAMPLE_COLON_JOBS:-}" ]] || SAMPLE_COLON_JOBS=$default_threads
make -j "$SAMPLE_COLON_JOBS"
