#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
for default_threads in 8; do :; done
[[ -n "${S3_FOR_THREADS:-}" ]] || S3_FOR_THREADS=$default_threads
