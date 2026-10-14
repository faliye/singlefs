#!/usr/bin/env bash
cap_jobs="${SINGLEFS_THREAD_CAP}"
[[ -n "${SAMPLE_CAP_JOBS:-}" ]] || SAMPLE_CAP_JOBS=$cap_jobs
make -j "$SAMPLE_CAP_JOBS"
