#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
source scripts/hosts/sample_defaults.sh
[[ -n "${SAMPLE_SOURCED_JOBS:-}" ]] || SAMPLE_SOURCED_JOBS=$default_threads
make -j "$SAMPLE_SOURCED_JOBS"
