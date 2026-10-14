#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
[[ -n "${S3_ARITHUSE_THREADS:-}" ]] || S3_ARITHUSE_THREADS=$((default_threads))
