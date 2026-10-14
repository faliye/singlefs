#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
mixed_jobs="$(heavy_thread_default)"
[[ "${1:-}" == quick ]] && mixed_jobs=8
[[ -n "${SAMPLE_MIXED_JOBS:-}" ]] || SAMPLE_MIXED_JOBS=$mixed_jobs
make -j "$SAMPLE_MIXED_JOBS"
