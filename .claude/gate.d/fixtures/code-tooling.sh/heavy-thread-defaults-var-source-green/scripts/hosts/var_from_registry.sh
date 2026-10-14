#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
[[ -n "${SAMPLE_SWEEP_JOBS:-}" ]] || SAMPLE_SWEEP_JOBS=$default_threads
make -j "$SAMPLE_SWEEP_JOBS"
