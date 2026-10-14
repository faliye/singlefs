#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
let default_threads=8
[[ -n "${S3_LET_THREADS:-}" ]] || S3_LET_THREADS=$default_threads
