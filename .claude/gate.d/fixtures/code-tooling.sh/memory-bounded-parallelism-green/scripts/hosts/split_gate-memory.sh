#!/usr/bin/env bash
# 样本：split_gate.sh 的部分文件，走种子的线程数按内存预算算
walker_count_of_split_gate() {
  local gibibytes=$1
  local count=$(((gibibytes - 8) / 4))
  echo "$count"
}
