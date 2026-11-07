#!/usr/bin/env bash
# 样本：走种子的线程数按内存预算算（注释里写 memory_bound_for 不算）
walker_count_of_budget() {
  local gibibytes=$1
  local count=$(((gibibytes - 8) / 4))
  echo "$count"
}
echo "memory_bound_for 只在出路文字里，不算"
