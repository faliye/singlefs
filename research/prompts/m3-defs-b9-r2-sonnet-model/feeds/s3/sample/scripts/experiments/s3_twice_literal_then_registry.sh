#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads=8
default_threads="$(heavy_thread_default)"
[[ -n "${S3_TWICEB_THREADS:-}" ]] || S3_TWICEB_THREADS=$default_threads
