#!/usr/bin/env bash
source scripts/hosts/heavy-threads.sh
if ! threads="$(heavy_thread_default)"; then exit 2; fi
[[ -n "${SAMPLE_AFTER_IF_JOBS:-}" ]] || SAMPLE_AFTER_IF_JOBS=${threads}
make -j "$SAMPLE_AFTER_IF_JOBS"
