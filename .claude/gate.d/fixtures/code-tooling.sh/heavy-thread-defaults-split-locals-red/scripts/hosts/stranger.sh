#!/usr/bin/env bash
# 样本：这一份自己赋 SAMPLE_STRANGER_WORKERS；stranger_report.sh 不是它的部分文件（钩子拆法是 stranger-<部分>.sh），赋值不算数
SAMPLE_STRANGER_WORKERS="${1:?工作进程数}"
echo stranger
