#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
default_threads+=0
[[ -n "${SAMPLE_PLUS_JOBS:-}" ]] || SAMPLE_PLUS_JOBS=$default_threads
make -j "$SAMPLE_PLUS_JOBS"
