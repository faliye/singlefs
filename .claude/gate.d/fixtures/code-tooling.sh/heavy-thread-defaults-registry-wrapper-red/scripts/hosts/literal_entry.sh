#!/usr/bin/env bash
# 样本：经 literal_default_jobs 取缺省路数
source "$(dirname "$0")/literal_library.sh"
SAMPLE_LITERAL_JOBS="${SAMPLE_LITERAL_JOBS:-$(literal_default_jobs)}"
run_cells "$SAMPLE_LITERAL_JOBS"
