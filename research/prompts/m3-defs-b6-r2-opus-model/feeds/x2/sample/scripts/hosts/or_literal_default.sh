#!/usr/bin/env bash
[[ -n "${SAMPLE_OR_LITERAL_JOBS:-}" ]] || SAMPLE_OR_LITERAL_JOBS=8
make -j "$SAMPLE_OR_LITERAL_JOBS"
