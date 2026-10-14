#!/usr/bin/env bash
source "$(dirname "$0")/../hosts/heavy-threads.sh"
source "$(dirname "$0")/s3_defaults_literal.sh"
[[ -n "${S3_SRCONLY_THREADS:-}" ]] || S3_SRCONLY_THREADS=$default_threads
