#!/usr/bin/env bash
# 样本：登记了的入口，走种子的线程数按内存预算算
walker_count_of_budget() {
  local gibibytes=$1
  local count=$(((gibibytes - 8) / 4))
  echo "$count"
}
