#!/usr/bin/env bash
# 样本：&& 之后的 SAMPLE_PREFIX_THREADS=8 后面跟着命令，是给那条命令的环境前缀，不是 shell 里没设就给的默认值，不点名
true && SAMPLE_PREFIX_THREADS=8 make -j 2
# 样本：弄坏开关护着的那一支（break_is <项> && X=$变量）只在自证打开那条开关时走到，照改前的样子取线程，不是没设就给的默认值，不点名
threads_of_the_device=$(device_thread_count)
break_is build-jobs-from-device-threads && export CARGO_BUILD_JOBS="$threads_of_the_device"
