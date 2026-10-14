#!/usr/bin/env bash
# 样本：printenv 从环境里读线程变量，没设就是空
threads="$(printenv SAMPLE_PRINTENV_THREADS)"
exec make -j "$threads"
