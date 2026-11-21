#!/usr/bin/env bash
# 样本：族外的文件，读的 SAMPLE_STRANGER_WORKERS 在别的文件里赋了值，在这一份里看不到赋值
run_device --workers "$SAMPLE_STRANGER_WORKERS"
