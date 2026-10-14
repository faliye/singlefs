#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
[[ -z "${S3_COND_PROBE:-}" ]] && default_threads=8
[[ -n "${S3_COND_THREADS:-}" ]] || S3_COND_THREADS=$default_threads
