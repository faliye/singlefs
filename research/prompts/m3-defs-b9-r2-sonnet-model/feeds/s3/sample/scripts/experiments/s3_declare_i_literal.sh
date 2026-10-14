#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
declare -i default_threads=8
[[ -n "${S3_DECL_THREADS:-}" ]] || S3_DECL_THREADS=$default_threads
