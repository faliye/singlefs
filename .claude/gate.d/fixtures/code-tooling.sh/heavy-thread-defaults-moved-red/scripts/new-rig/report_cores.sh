#!/usr/bin/env bash
# 样本：只把核数打出来，不起线程；登记在排除表里，setup.sh 把它连同所在目录放回搬家之前的位置提交成基准
jobs=$(nproc)
printf '本机 %s 核\n' "$jobs"
