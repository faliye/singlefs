#!/usr/bin/env bash
cap_jobs="${SINGLEFS_THREAD_CAP}"
[[ -n "${S3_CAP_THREADS:-}" ]] || S3_CAP_THREADS=$cap_jobs
