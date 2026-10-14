#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
(( default_threads = 8 ))
[[ -n "${SAMPLE_ARITH_ASSIGN_JOBS:-}" ]] || SAMPLE_ARITH_ASSIGN_JOBS=$default_threads
make -j "$SAMPLE_ARITH_ASSIGN_JOBS"
