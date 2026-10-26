#!/usr/bin/env bash
# 样本脚本：弄坏开关只在自证里认，平常跑设了就拒
# 弄坏开关：
#   GUARDED_BREAK=skip  判法整段跳过
#       guarded-red（「✗ 自检：该红没红」）
if [[ -n "${GUARDED_BREAK:-}" && "${1:-}" != --selftest ]]; then
  echo "  ✗ GUARDED_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2
  echo "     → 怎么办：去掉 GUARDED_BREAK 再跑" >&2
  exit 2
fi
echo "样本：正常"
