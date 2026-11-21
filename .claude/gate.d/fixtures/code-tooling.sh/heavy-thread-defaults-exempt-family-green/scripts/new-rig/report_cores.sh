#!/usr/bin/env bash
# 样本：入口只调同目录的部分文件 report_cores-count.sh；基准那一版里 $(nproc) 那一行就在这一份里
bash "$(dirname "$0")/report_cores-count.sh"
