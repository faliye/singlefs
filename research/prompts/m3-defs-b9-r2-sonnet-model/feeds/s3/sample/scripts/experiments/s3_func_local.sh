#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
pick_default() { local default_threads; default_threads="$(heavy_thread_default)"; }
pick_default
[[ -n "${S3_FUNC_THREADS:-}" ]] || S3_FUNC_THREADS=$default_threads
