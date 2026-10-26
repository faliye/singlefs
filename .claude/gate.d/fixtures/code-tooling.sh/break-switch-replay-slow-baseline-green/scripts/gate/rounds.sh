#!/usr/bin/env bash
# 样本脚本：自证自带开关轮——外面设开关当场被守卫拒；不设开关时自证要把它自己那一轮跑完（这里睡 3 秒代替），比单条开关趟慢得多
# 弄坏开关：
#   ROUNDS_BREAK=skip  判法整段跳过
#       rounds-skip（「✗ 自检：该红没红」）
if [[ -n "${ROUNDS_BREAK:-}" ]]; then
  echo "  ✗ ROUNDS_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 ROUNDS_BREAK 再跑" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  sleep 3
  echo "  ✓ 自检通过（连同自带的开关轮）"
fi
