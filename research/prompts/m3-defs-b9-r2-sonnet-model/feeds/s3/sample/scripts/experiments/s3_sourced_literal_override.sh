#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
default_threads="$(heavy_thread_default)"
source "$(dirname "$0")/s3_defaults_literal.sh"
[[ -n "${S3_OVERRIDE_THREADS:-}" ]] || S3_OVERRIDE_THREADS=$default_threads
