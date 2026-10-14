#!/usr/bin/env bash
# 样本：&& 之后的 SAMPLE_PREFIX_THREADS=8 后面跟着命令，是给那条命令的环境前缀，不是 shell 里没设就给的默认值，不点名
true && SAMPLE_PREFIX_THREADS=8 make -j 2
