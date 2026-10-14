#!/usr/bin/env bash
# 夹下限：已经取好，只是不许小于 1
JOBS_FROM_REGISTRY=$(bash scripts/hosts/capped.sh default echo ok >/dev/null; echo 4)
[[ "$SAMPLE_CLAMP_JOBS" -lt 1 ]] && SAMPLE_CLAMP_JOBS=1
