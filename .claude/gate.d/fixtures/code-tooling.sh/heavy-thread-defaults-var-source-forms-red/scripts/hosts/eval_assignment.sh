#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default)"
eval "default_threads=8"
[[ -n "${SAMPLE_EVAL_JOBS:-}" ]] || SAMPLE_EVAL_JOBS=$default_threads
make -j "$SAMPLE_EVAL_JOBS"
