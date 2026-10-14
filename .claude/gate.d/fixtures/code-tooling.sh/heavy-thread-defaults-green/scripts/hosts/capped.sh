#!/usr/bin/env bash
# 样本：capped.sh 的变量清单，E161_THREADS 在里面
variable_names=(
    CARGO_BUILD_JOBS
    E161_THREADS
    SINGLEFS_THREAD_CAP
)
exec "$@"
