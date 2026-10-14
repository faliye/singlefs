#!/usr/bin/env bash
fallback_jobs=8
default_threads="${SINGLEFS_THREAD_CAP:-$fallback_jobs}"
[[ -n "${SAMPLE_CAP_FALLBACK_JOBS:-}" ]] || SAMPLE_CAP_FALLBACK_JOBS=$default_threads
make -j "$SAMPLE_CAP_FALLBACK_JOBS"
