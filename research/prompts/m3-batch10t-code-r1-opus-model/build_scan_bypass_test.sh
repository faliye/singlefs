#!/usr/bin/env bash
# 拼出 K4-2 的原型测试：冻结副本里扫源码那份测试的三段原样拷贝 + scan_bypass_samples.rs。
# 用法：build_scan_bypass_test.sh <冻结副本根> <输出的 .rs>
set -euo pipefail
source_file="$1/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs"
model_directory="$(cd "$(dirname "$0")" && pwd)"
{
  echo '#![allow(dead_code)]'
  sed -n '21,30p' "$source_file"
  sed -n '62,68p' "$source_file"
  sed -n '89,424p' "$source_file"
  cat "$model_directory/scan_bypass_samples.rs"
} > "$2"
