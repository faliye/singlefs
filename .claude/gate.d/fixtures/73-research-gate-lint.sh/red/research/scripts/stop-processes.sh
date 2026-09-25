#!/usr/bin/env bash
# 样本：几种打得到别人进程的终止写法，门禁 73 号的进程安全那一项要逐处按行号报出来（这个文件只拿来扫，不执行）。
set -m
sleep 30 &
job=$!
kill -INT -- -"$job"
for pid in "${worker_pids[@]}"; do
  kill -TERM "$pid"
done
systemctl --user stop 'singlefs-*'
