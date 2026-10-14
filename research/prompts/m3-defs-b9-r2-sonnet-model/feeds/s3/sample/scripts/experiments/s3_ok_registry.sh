#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
[[ -n "${S3_OK_THREADS:-}" ]] || S3_OK_THREADS=$default_threads
