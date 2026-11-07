#!/usr/bin/env bash
# 样本：没登记的入口，同时试写的拷贝数按内存预算算
probe_count_of_budget() {
  local count=$((gibibytes / 5))
  echo "$count"
}
