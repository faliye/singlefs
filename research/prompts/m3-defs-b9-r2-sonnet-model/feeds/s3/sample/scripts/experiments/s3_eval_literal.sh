#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
eval "default_threads=8"
[[ -n "${S3_EVAL_THREADS:-}" ]] || S3_EVAL_THREADS=$default_threads
