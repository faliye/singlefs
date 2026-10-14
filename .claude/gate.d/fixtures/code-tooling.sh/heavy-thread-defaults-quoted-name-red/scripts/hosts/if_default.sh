#!/usr/bin/env bash
if [[ -z "${SAMPLE_IF_FALLBACK_THREADS:-}" ]]; then SAMPLE_IF_FALLBACK_THREADS=8; fi
exec make -j "$SAMPLE_IF_FALLBACK_THREADS"
