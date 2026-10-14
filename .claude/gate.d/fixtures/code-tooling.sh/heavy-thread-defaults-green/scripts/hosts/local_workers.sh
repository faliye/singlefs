#!/usr/bin/env bash
# 样本：WORKERS 是文件自己赋的局部变量，不是从环境读的，不算装置自己的线程变量
WORKERS="${2:?工作进程数}"
run_device --workers "$WORKERS"
step=1; SAMPLE_INLINE_WORKERS="${3:?内联赋的局部变量}"
run_device --lanes "$SAMPLE_INLINE_WORKERS"
