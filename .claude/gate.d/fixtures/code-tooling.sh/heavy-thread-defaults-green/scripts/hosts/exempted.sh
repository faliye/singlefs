#!/usr/bin/env bash
# 样本：只把核数打出来，不起线程；登记在排除表里
jobs=$(nproc)
printf '本机 %s 核\n' "$jobs"
