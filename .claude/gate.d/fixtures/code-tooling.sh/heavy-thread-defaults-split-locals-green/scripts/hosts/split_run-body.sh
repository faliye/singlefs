#!/usr/bin/env bash
# 样本：split_run.sh 的部分文件，读入口里赋了值的 SAMPLE_SPLIT_WORKERS（拆分之前在同一份文件里）
run_device --workers "$SAMPLE_SPLIT_WORKERS"
