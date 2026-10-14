#!/usr/bin/env bash
[[ -n "${SAMPLE_LITERAL_JOBS:-}" ]] || SAMPLE_LITERAL_JOBS=8
make -j "$SAMPLE_LITERAL_JOBS"
