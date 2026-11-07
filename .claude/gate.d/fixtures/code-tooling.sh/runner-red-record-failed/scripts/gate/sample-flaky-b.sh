#!/usr/bin/env bash
# 样本：自证在样本根有开关文件 .red-switch 时退 1、没有时退 0（开关文件不在任何单元的闭包里，不进键：同一把键这一趟判红、上一趟判绿）
[[ "${1:-}" == --selftest ]] || exit 2
if [[ -e "$(dirname "${BASH_SOURCE[0]}")/../../.red-switch" ]]; then
  echo "  ✗ b 的自证判错：样本根的开关文件 .red-switch 在"
  exit 1
fi
echo "  ✓ b 的自证 1 条都判对"
exit 0
