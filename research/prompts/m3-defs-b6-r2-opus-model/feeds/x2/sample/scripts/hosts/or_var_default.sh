#!/usr/bin/env bash
fallback=8
[[ -n "${SAMPLE_OR_VAR_JOBS:-}" ]] || SAMPLE_OR_VAR_JOBS=$fallback
make -j "$SAMPLE_OR_VAR_JOBS"
