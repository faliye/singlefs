#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
printf -v default_threads %s 8
[[ -n "${SAMPLE_PRINTF_JOBS:-}" ]] || SAMPLE_PRINTF_JOBS=$default_threads
make -j "$SAMPLE_PRINTF_JOBS"
