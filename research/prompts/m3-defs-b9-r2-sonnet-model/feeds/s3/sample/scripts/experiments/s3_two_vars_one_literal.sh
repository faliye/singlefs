#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
good_threads="$(heavy_thread_default)"
bad_threads=8
[[ -n "${S3_TWOV_THREADS:-}" ]] || S3_TWOV_THREADS=$good_threads
[[ -n "${S3_TWOV2_THREADS:-}" ]] || S3_TWOV2_THREADS=$bad_threads
