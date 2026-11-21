#!/usr/bin/env bash
# 样本：orphan_run.sh 的部分文件，读的 SAMPLE_ORPHAN_WORKERS 族里没有一份文件赋过值，是从环境读的
run_device --workers "$SAMPLE_ORPHAN_WORKERS"
