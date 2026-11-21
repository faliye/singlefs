#!/usr/bin/env bash
# 样本：阶段入口照五道阶段的写法取格并行的缺省路数：环境里给了照给的，没给经包装函数 sample_default_jobs 取登记值
source "$(dirname "$0")/stage_library.sh"
SAMPLE_STAGE_JOBS="${SAMPLE_STAGE_JOBS:-$(sample_default_jobs)}"
run_cells "$SAMPLE_STAGE_JOBS"
