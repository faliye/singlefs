#!/usr/bin/env bash
cap_jobs="${SINGLEFS_THREAD_CAP:-8}"
[[ -n "${S3_CAPFB_THREADS:-}" ]] || S3_CAPFB_THREADS=$cap_jobs
