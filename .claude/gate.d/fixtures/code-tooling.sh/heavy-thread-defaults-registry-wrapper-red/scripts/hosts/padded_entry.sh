#!/usr/bin/env bash
# 样本：经 padded_default_jobs 取缺省路数
source "$(dirname "$0")/padded_library.sh"
SAMPLE_PADDED_JOBS="${SAMPLE_PADDED_JOBS:-$(padded_default_jobs)}"
run_cells "$SAMPLE_PADDED_JOBS"
