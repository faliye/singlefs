#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
read -r default_threads <<< 8
[[ -n "${S3_READ_THREADS:-}" ]] || S3_READ_THREADS=$default_threads
