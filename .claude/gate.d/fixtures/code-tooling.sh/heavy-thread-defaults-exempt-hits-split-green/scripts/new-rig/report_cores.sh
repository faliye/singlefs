#!/usr/bin/env bash
# 样本：入口只调同目录的两份部分文件；基准那一版里两行 $(nproc) 都在这一份里
bash "$(dirname "$0")/report_cores-count.sh"
bash "$(dirname "$0")/report_cores-show.sh"
