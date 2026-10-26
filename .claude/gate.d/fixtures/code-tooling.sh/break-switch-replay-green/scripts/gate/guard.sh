#!/usr/bin/env bash
# 样本脚本：弄坏开关只在自证起的进程里认；外面设了就拒（自证自带开关轮，复跑不在这里做）
# 弄坏开关：
#   GUARD_BREAK=skip  判法整段跳过
#       guard-skip（「✗ 自检：该红没红」）
if [[ -n "${GUARD_BREAK:-}" ]]; then
  echo "  ✗ GUARD_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 GUARD_BREAK 再跑" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then echo "  ✓ 自检通过"; fi
