#!/usr/bin/env bash
# 第 21 批 K1：H2 用例在副本上，回收门槛放宽 0、1、2、5 个 txg（B21_SLACK，副本 allocator.rs 里读环境变量）各跑一次。
set -u
BIN=/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/target/debug/deps/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement-c936ee7c382e79da
for sl in 0 1 2 5; do
  echo "== slack=$sl"
  B21_SLACK=$sl "$BIN" the_first_file_after_nine_warm_up_only_mounts 2>&1 | grep -E '^test |test result|left:|right:|panicked at' | cut -c1-300
done
