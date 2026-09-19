#!/usr/bin/env bash
# 样本：函数只在 $( ) 里被调用，却靠全局变量把日志路径带出来——父进程拿到的是空的。
run_one() {
  LOG_PATH="/tmp/run.log"
  printf '%s' 0
}
status="$(run_one)"
tail -5 "$LOG_PATH"
echo "$status"
