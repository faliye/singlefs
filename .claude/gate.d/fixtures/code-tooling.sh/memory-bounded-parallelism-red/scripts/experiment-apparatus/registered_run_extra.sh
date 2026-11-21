#!/usr/bin/env bash
# 样本：名字像登记入口 registered_run.sh 的部分文件、其实不是拆法命名的族外文件，没登记，同时试写的拷贝数按内存预算算
extra_count_of_budget() {
  local count=$((gibibytes / 3))
  echo "$count"
}
