#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
[[ -n "${S3_BRACED_THREADS:-}" ]] || S3_BRACED_THREADS=${default_threads}
