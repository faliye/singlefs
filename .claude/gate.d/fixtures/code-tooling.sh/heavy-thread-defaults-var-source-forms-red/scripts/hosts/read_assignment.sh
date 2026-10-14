#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
read -r default_threads <<< 8
[[ -n "${SAMPLE_READ_JOBS:-}" ]] || SAMPLE_READ_JOBS=$default_threads
make -j "$SAMPLE_READ_JOBS"
