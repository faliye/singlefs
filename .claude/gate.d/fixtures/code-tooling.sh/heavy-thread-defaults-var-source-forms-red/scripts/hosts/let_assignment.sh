#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
let "default_threads = 8"
[[ -n "${SAMPLE_LET_JOBS:-}" ]] || SAMPLE_LET_JOBS=$default_threads
make -j "$SAMPLE_LET_JOBS"
