#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
(( default_threads = 8 ))
[[ -n "${S3_ARITH_THREADS:-}" ]] || S3_ARITH_THREADS=$default_threads
