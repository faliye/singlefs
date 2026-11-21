#!/usr/bin/env bash
# 样本：capped.sh 的变量清单
variable_names=(
    CARGO_BUILD_JOBS
    SINGLEFS_THREAD_CAP
)
exec "$@"
