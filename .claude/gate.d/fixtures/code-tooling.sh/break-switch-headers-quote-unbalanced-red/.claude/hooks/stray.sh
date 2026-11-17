#!/usr/bin/env bash
# 样本钩子：开关那一条点名的项里多一个收尾的括号，复跑读到的点名与写的不是一回事
#   STRAY_BREAK=skip  判法整段跳过
#     「✗ 自检：判法那一格」」
if [[ "${STRAY_BREAK:-}" == skip ]]; then
  echo "  ✗ 自检：判法那一格：应当 1，实际 0"
  exit 1
fi
