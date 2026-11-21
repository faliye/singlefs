#!/usr/bin/env bash
# 样本：只把核数打出来，不起线程；是 report_cores.sh 拆出来的部分文件
shown=$(nproc)
printf '核数 %s\n' "$shown"
