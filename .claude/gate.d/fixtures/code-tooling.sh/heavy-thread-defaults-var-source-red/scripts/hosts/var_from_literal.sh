#!/usr/bin/env bash
fallback_jobs=8
[[ -n "${SAMPLE_LITERAL_VAR_JOBS:-}" ]] || SAMPLE_LITERAL_VAR_JOBS=$fallback_jobs
make -j "$SAMPLE_LITERAL_VAR_JOBS"
