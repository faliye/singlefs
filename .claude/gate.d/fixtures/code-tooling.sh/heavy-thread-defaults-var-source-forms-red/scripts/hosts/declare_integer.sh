#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
declare -i default_threads
default_threads="$(heavy_thread_default)"
[[ -n "${SAMPLE_DECLARE_JOBS:-}" ]] || SAMPLE_DECLARE_JOBS=$default_threads
make -j "$SAMPLE_DECLARE_JOBS"
