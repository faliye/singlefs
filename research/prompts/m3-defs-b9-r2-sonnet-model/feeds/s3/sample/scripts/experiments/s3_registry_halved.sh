#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads=$(( $(heavy_thread_default) / 2 ))
[[ -n "${S3_HALF_THREADS:-}" ]] || S3_HALF_THREADS=$default_threads
