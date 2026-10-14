#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
pick_default() { local default_threads; default_threads="$(heavy_thread_default)"; }
[[ -n "${S3_NEVER_THREADS:-}" ]] || S3_NEVER_THREADS=$default_threads
