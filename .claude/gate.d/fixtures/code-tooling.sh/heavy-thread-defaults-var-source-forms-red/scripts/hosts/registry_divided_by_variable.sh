#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
divisor=2
default_threads="$(( $(heavy_thread_default) / divisor ))"
[[ -n "${SAMPLE_DIVIDED_JOBS:-}" ]] || SAMPLE_DIVIDED_JOBS=$default_threads
make -j "$SAMPLE_DIVIDED_JOBS"
