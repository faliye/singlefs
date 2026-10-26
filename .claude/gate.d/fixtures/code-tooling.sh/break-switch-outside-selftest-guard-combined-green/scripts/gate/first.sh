#!/usr/bin/env bash
# 样本脚本：关守卫项排在开关表第一条，整值比，只在它是唯一一项时放行
# 弄坏开关：
#   FIRST_BREAK=guard-off  守卫整段不判
#       first-guard（「✗ 自检：守卫没拒」）
#   FIRST_BREAK=drop  丢掉一格
#       first-drop（「✗ 自检：丢了的那一格」）
if [[ -n "${FIRST_BREAK:-}" && "${1:-}" != --selftest && "${FIRST_BREAK}" != guard-off ]]; then
  echo "  ✗ FIRST_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2
  echo "     → 怎么办：去掉 FIRST_BREAK 再跑" >&2
  exit 2
fi
echo "样本：正常"
