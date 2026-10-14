#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
[[ -n "${S3_AFTER_THREADS:-}" ]] || S3_AFTER_THREADS=$default_threads
default_threads="$(heavy_thread_default)"
