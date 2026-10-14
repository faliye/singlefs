#!/usr/bin/env bash
workers="$(printenv SAMPLE_PRINTENV_WORKERS)"
exec make -j "$workers"
