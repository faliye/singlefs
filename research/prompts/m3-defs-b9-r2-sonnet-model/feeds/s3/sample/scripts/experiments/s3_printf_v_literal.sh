#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
printf -v default_threads %s 8
[[ -n "${S3_PRINTFV_THREADS:-}" ]] || S3_PRINTFV_THREADS=$default_threads
