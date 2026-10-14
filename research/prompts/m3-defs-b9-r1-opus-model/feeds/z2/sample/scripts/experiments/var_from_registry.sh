#!/usr/bin/env bash
# 默认值先从登记位取进变量，再在没设时赋给装置的线程变量（取的是登记值，不是钉数）
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
[[ -n "${SAMPLE_SWEEP_THREADS:-}" ]] || SAMPLE_SWEEP_THREADS=$default_threads
export SAMPLE_SWEEP_THREADS
