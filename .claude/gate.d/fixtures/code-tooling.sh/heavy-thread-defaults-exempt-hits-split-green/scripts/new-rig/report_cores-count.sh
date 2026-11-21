#!/usr/bin/env bash
# 样本：只把核数打出来，不起线程；是 report_cores.sh 拆出来的部分文件
cores=$(nproc)
printf '本机 %s 核\n' "$cores"
