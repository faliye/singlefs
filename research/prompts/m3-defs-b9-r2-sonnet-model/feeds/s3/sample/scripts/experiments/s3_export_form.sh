#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
export default_threads="$(heavy_thread_default)"
[[ -n "${S3_EXP_THREADS:-}" ]] || S3_EXP_THREADS=$default_threads
