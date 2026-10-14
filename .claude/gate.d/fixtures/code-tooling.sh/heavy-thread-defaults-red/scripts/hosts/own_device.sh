#!/usr/bin/env bash
# 样本：shell 装置读自己的线程变量，没设就停；LOCAL_WORKERS 是文件自己赋的局部变量，不算
LOCAL_WORKERS="${2:?工作进程数}"
run_device --threads "${SAMPLE_SHELL_THREADS:?没设}" --workers "$LOCAL_WORKERS"
