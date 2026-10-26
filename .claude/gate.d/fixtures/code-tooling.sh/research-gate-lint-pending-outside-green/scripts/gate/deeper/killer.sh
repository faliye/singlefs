#!/usr/bin/env bash
# 样本：scripts/gate/ 的下一层，进程安全那一路只扫到 scripts/<类>/ 为止，这一层不扫（这个文件只拿来被待改清单点名，不执行）。
for pid in "${worker_pids[@]}"; do
  kill -TERM "$pid"
done
