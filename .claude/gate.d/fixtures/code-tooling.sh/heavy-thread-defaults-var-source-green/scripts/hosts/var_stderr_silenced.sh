#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
default_threads="$(heavy_thread_default 2>/dev/null)" || exit 2
[[ -n "${SAMPLE_SILENCED_JOBS:-}" ]] || SAMPLE_SILENCED_JOBS=$default_threads
make -j "$SAMPLE_SILENCED_JOBS"
