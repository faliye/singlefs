#!/usr/bin/env bash
[[ -n "${SAMPLE_OR_ARITH_JOBS:-}" ]] || SAMPLE_OR_ARITH_JOBS=$((4 * 2))
make -j "$SAMPLE_OR_ARITH_JOBS"
