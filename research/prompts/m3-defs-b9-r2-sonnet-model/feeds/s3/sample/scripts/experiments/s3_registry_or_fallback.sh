#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default || echo 8)"
[[ -n "${S3_ORFB_THREADS:-}" ]] || S3_ORFB_THREADS=$default_threads
