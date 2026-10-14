#!/usr/bin/env bash
# 攻方样本（Y2）：env | grep | cut 取自己的线程变量
jobs=$(env | grep ^SAMPLE_ENVGREP_JOBS= | cut -d= -f2)
make -j "${jobs:-8}"
