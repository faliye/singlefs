#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
default_threads+=0
[[ -n "${S3_PLUSEQ_THREADS:-}" ]] || S3_PLUSEQ_THREADS=$default_threads
