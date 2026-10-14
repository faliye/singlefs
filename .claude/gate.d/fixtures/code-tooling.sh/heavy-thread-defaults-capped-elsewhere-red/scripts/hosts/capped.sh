#!/usr/bin/env bash
# 样本：capped.sh 的变量清单，装置自己的线程变量（SAMPLE_*）都不在里面
variable_names=(
    CARGO_BUILD_JOBS
    SINGLEFS_THREAD_CAP
)
exec "$@"
