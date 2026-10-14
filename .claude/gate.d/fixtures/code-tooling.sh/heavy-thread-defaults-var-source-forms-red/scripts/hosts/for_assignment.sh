#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
for default_threads in 8; do :; done
[[ -n "${SAMPLE_FOR_JOBS:-}" ]] || SAMPLE_FOR_JOBS=$default_threads
make -j "$SAMPLE_FOR_JOBS"
