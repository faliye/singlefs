#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
[[ -n "${SAMPLE_OR_REGISTRY_JOBS:-}" ]] || SAMPLE_OR_REGISTRY_JOBS=$(heavy_thread_default)
[[ -n "${SAMPLE_OR_CAP_JOBS:-}" ]] || SAMPLE_OR_CAP_JOBS=$SINGLEFS_THREAD_CAP
make -j "$SAMPLE_OR_REGISTRY_JOBS" -l "$SAMPLE_OR_CAP_JOBS"
